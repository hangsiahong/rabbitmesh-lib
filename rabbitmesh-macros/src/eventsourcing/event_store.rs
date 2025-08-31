//! Event Sourcing and CQRS Implementation
//! 
//! Provides comprehensive event sourcing with generic event store backends,
//! aggregate management, and CQRS pattern implementation.

use quote::quote;

/// Generate event sourcing preprocessing code
pub fn generate_event_sourcing_preprocessing(
    aggregate_type: Option<&str>,
    event_store_backend: Option<&str>,
    snapshot_frequency: Option<u32>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("📚 Initializing event sourcing pattern");
        
        // Create event store manager with configuration
        let aggregate_type = #aggregate_type.unwrap_or("default");
        let backend = #event_store_backend.unwrap_or("postgres");
        let snapshot_freq = #snapshot_frequency.unwrap_or(100);
        
        let event_store = EventStore::get_or_create(backend).await?;
        let aggregate_manager = AggregateManager::new(event_store.clone());
        
        tracing::debug!("📚 Event store initialized - Backend: {}, Aggregate: {}, Snapshots every: {} events", 
            backend, aggregate_type, snapshot_freq);
        
        // Extract aggregate ID from message
        let aggregate_id = extract_aggregate_id(&msg, aggregate_type)
            .ok_or_else(|| rabbitmesh::error::RabbitMeshError::Handler(
                "Missing aggregate ID for event sourcing".to_string()
            ))?;
        
        tracing::debug!("🆔 Aggregate ID: {} (type: {})", aggregate_id, aggregate_type);
        
        /// Event Store for persisting and retrieving domain events
        struct EventStore {
            backend: Arc<dyn EventStoreBackend + Send + Sync>,
            serializer: Arc<dyn EventSerializer + Send + Sync>,
            snapshot_store: Arc<dyn SnapshotStore + Send + Sync>,
            event_bus: Arc<dyn EventBus + Send + Sync>,
            metrics: Arc<EventStoreMetrics>,
        }
        
        /// Event store backend abstraction
        #[async_trait::async_trait]
        trait EventStoreBackend {
            async fn append_events(
                &self,
                aggregate_id: &str,
                expected_version: EventVersion,
                events: &[DomainEvent]
            ) -> Result<EventVersion, EventStoreError>;
            
            async fn load_events(
                &self,
                aggregate_id: &str,
                from_version: Option<EventVersion>
            ) -> Result<Vec<DomainEvent>, EventStoreError>;
            
            async fn load_events_by_type(
                &self,
                event_type: &str,
                from_timestamp: Option<std::time::SystemTime>
            ) -> Result<Vec<DomainEvent>, EventStoreError>;
            
            async fn get_aggregate_version(&self, aggregate_id: &str) -> Result<Option<EventVersion>, EventStoreError>;
        }
        
        /// Event serialization trait for different formats
        trait EventSerializer {
            fn serialize(&self, event: &DomainEvent) -> Result<Vec<u8>, EventStoreError>;
            fn deserialize(&self, data: &[u8], event_type: &str) -> Result<DomainEvent, EventStoreError>;
        }
        
        /// Snapshot store for aggregate state snapshots
        #[async_trait::async_trait]
        trait SnapshotStore {
            async fn save_snapshot(&self, aggregate_id: &str, snapshot: &AggregateSnapshot) -> Result<(), EventStoreError>;
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError>;
            async fn delete_old_snapshots(&self, aggregate_id: &str, keep_count: u32) -> Result<(), EventStoreError>;
        }
        
        /// Event bus for publishing domain events
        #[async_trait::async_trait]
        trait EventBus {
            async fn publish_event(&self, event: &DomainEvent) -> Result<(), EventStoreError>;
            async fn publish_events(&self, events: &[DomainEvent]) -> Result<(), EventStoreError>;
            async fn subscribe_to_events<F>(&self, event_type: &str, handler: F) -> Result<(), EventStoreError>
            where
                F: Fn(DomainEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static;
        }
        
        /// Domain event representation
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct DomainEvent {
            event_id: String,
            aggregate_id: String,
            aggregate_type: String,
            event_type: String,
            event_version: EventVersion,
            event_data: serde_json::Value,
            metadata: EventMetadata,
            timestamp: std::time::SystemTime,
            correlation_id: Option<String>,
            causation_id: Option<String>,
        }
        
        /// Event metadata for tracking and debugging
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct EventMetadata {
            user_id: Option<String>,
            source: String,
            ip_address: Option<String>,
            user_agent: Option<String>,
            command_id: Option<String>,
            custom_fields: std::collections::HashMap<String, serde_json::Value>,
        }
        
        /// Event version for optimistic concurrency control
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        struct EventVersion(pub i64);
        
        impl EventVersion {
            const INITIAL: EventVersion = EventVersion(0);
            
            fn next(self) -> EventVersion {
                EventVersion(self.0 + 1)
            }
        }
        
        /// Aggregate snapshot for performance optimization
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        struct AggregateSnapshot {
            aggregate_id: String,
            aggregate_type: String,
            version: EventVersion,
            state_data: serde_json::Value,
            created_at: std::time::SystemTime,
        }
        
        /// Aggregate manager for loading and saving aggregates
        struct AggregateManager {
            event_store: Arc<EventStore>,
            aggregate_cache: Arc<RwLock<std::collections::HashMap<String, CachedAggregate>>>,
            snapshot_frequency: u32,
        }
        
        /// Cached aggregate for performance
        #[derive(Debug, Clone)]
        struct CachedAggregate {
            aggregate_id: String,
            version: EventVersion,
            state: serde_json::Value,
            last_accessed: std::time::Instant,
            uncommitted_events: Vec<DomainEvent>,
        }
        
        /// Event store metrics
        struct EventStoreMetrics {
            events_appended: AtomicU64,
            events_loaded: AtomicU64,
            snapshots_created: AtomicU64,
            snapshots_loaded: AtomicU64,
            concurrency_conflicts: AtomicU64,
            average_append_time: AtomicU64,
            average_load_time: AtomicU64,
        }
        
        /// Event store error types
        #[derive(Debug, thiserror::Error)]
        enum EventStoreError {
            #[error("Concurrency conflict: expected version {expected}, actual version {actual}")]
            ConcurrencyConflict { expected: EventVersion, actual: EventVersion },
            #[error("Aggregate not found: {aggregate_id}")]
            AggregateNotFound { aggregate_id: String },
            #[error("Serialization error: {0}")]
            Serialization(String),
            #[error("Backend error: {0}")]
            Backend(String),
            #[error("Configuration error: {0}")]
            Configuration(String),
        }
        
        /// PostgreSQL event store backend
        struct PostgreSQLEventStore {
            pool: sqlx::PgPool,
        }
        
        /// MongoDB event store backend
        struct MongoDBEventStore {
            client: mongodb::Client,
            database: mongodb::Database,
        }
        
        /// In-memory event store for testing
        struct InMemoryEventStore {
            events: Arc<RwLock<std::collections::HashMap<String, Vec<DomainEvent>>>>,
            versions: Arc<RwLock<std::collections::HashMap<String, EventVersion>>>,
        }
        
        /// JSON event serializer
        struct JsonEventSerializer;
        
        impl EventSerializer for JsonEventSerializer {
            fn serialize(&self, event: &DomainEvent) -> Result<Vec<u8>, EventStoreError> {
                tracing::trace!("🔄 JSON: Serializing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                serde_json::to_vec(event)
                    .map_err(|e| EventStoreError::Serialization(format!("JSON serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8], event_type: &str) -> Result<DomainEvent, EventStoreError> {
                tracing::trace!("🔄 JSON: Deserializing event of type {}", event_type);
                
                let event: DomainEvent = serde_json::from_slice(data)
                    .map_err(|e| EventStoreError::Serialization(format!("JSON deserialization failed: {}", e)))?;
                
                // Validate event type matches expected
                if event.event_type != event_type {
                    return Err(EventStoreError::Serialization(format!(
                        "Event type mismatch: expected '{}', found '{}'", 
                        event_type, event.event_type
                    )));
                }
                
                Ok(event)
            }
        }
        
        /// Binary event serializer using MessagePack
        struct BinaryEventSerializer;
        
        impl EventSerializer for BinaryEventSerializer {
            fn serialize(&self, event: &DomainEvent) -> Result<Vec<u8>, EventStoreError> {
                tracing::trace!("🔄 Binary: Serializing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                rmp_serde::to_vec(event)
                    .map_err(|e| EventStoreError::Serialization(format!("MessagePack serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8], event_type: &str) -> Result<DomainEvent, EventStoreError> {
                tracing::trace!("🔄 Binary: Deserializing event of type {}", event_type);
                
                let event: DomainEvent = rmp_serde::from_slice(data)
                    .map_err(|e| EventStoreError::Serialization(format!("MessagePack deserialization failed: {}", e)))?;
                
                // Validate event type matches expected
                if event.event_type != event_type {
                    return Err(EventStoreError::Serialization(format!(
                        "Event type mismatch: expected '{}', found '{}'", 
                        event_type, event.event_type
                    )));
                }
                
                Ok(event)
            }
        }
        
        /// Protobuf event serializer (placeholder for future implementation)
        struct ProtobufEventSerializer;
        
        impl EventSerializer for ProtobufEventSerializer {
            fn serialize(&self, event: &DomainEvent) -> Result<Vec<u8>, EventStoreError> {
                tracing::trace!("🔄 Protobuf: Serializing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                // For now, fall back to JSON serialization
                // Protobuf serialization with generated structs for efficient binary format
                tracing::warn!("⚠️ Protobuf serialization not fully implemented, falling back to JSON");
                serde_json::to_vec(event)
                    .map_err(|e| EventStoreError::Serialization(format!("Protobuf (JSON fallback) serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8], event_type: &str) -> Result<DomainEvent, EventStoreError> {
                tracing::trace!("🔄 Protobuf: Deserializing event of type {}", event_type);
                
                // For now, fall back to JSON deserialization
                tracing::warn!("⚠️ Protobuf deserialization not fully implemented, falling back to JSON");
                let event: DomainEvent = serde_json::from_slice(data)
                    .map_err(|e| EventStoreError::Serialization(format!("Protobuf (JSON fallback) deserialization failed: {}", e)))?;
                
                // Validate event type matches expected
                if event.event_type != event_type {
                    return Err(EventStoreError::Serialization(format!(
                        "Event type mismatch: expected '{}', found '{}'", 
                        event_type, event.event_type
                    )));
                }
                
                Ok(event)
            }
        }
        
        /// CBOR event serializer for compact binary representation
        struct CborEventSerializer;
        
        impl EventSerializer for CborEventSerializer {
            fn serialize(&self, event: &DomainEvent) -> Result<Vec<u8>, EventStoreError> {
                tracing::trace!("🔄 CBOR: Serializing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                serde_cbor::to_vec(event)
                    .map_err(|e| EventStoreError::Serialization(format!("CBOR serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8], event_type: &str) -> Result<DomainEvent, EventStoreError> {
                tracing::trace!("🔄 CBOR: Deserializing event of type {}", event_type);
                
                let event: DomainEvent = serde_cbor::from_slice(data)
                    .map_err(|e| EventStoreError::Serialization(format!("CBOR deserialization failed: {}", e)))?;
                
                // Validate event type matches expected
                if event.event_type != event_type {
                    return Err(EventStoreError::Serialization(format!(
                        "Event type mismatch: expected '{}', found '{}'", 
                        event_type, event.event_type
                    )));
                }
                
                Ok(event)
            }
        }
        
        /// Thread-safe global event store registry
        static EVENT_STORES: once_cell::sync::OnceCell<Arc<RwLock<std::collections::HashMap<String, Arc<EventStore>>>>> 
            = once_cell::sync::OnceCell::new();
        
        impl EventStore {
            /// Get or create event store for specific backend
            async fn get_or_create(backend_type: &str) -> Result<Arc<Self>, EventStoreError> {
                let stores = EVENT_STORES.get_or_init(|| {
                    Arc::new(RwLock::new(std::collections::HashMap::new()))
                });
                
                // Try to get existing store
                {
                    let stores_read = stores.read().await;
                    if let Some(store) = stores_read.get(backend_type) {
                        return Ok(store.clone());
                    }
                }
                
                // Create new event store
                let backend = create_event_store_backend(backend_type).await?;
                let serializer = Arc::from(create_event_serializer());
                let snapshot_store = Arc::new(create_snapshot_store(backend_type).await?);
                let event_bus = Arc::new(create_event_bus().await?);
                let metrics = Arc::new(EventStoreMetrics::new());
                
                let store = Arc::new(Self {
                    backend,
                    serializer,
                    snapshot_store,
                    event_bus,
                    metrics,
                });
                
                // Store in registry
                {
                    let mut stores_write = stores.write().await;
                    stores_write.insert(backend_type.to_string(), store.clone());
                }
                
                tracing::info!("📚 Created new event store: {}", backend_type);
                Ok(store)
            }
            
            /// Append events to the store
            async fn append_events(
                &self,
                aggregate_id: &str,
                expected_version: EventVersion,
                events: &[DomainEvent]
            ) -> Result<EventVersion, EventStoreError> {
                let start_time = std::time::Instant::now();
                
                tracing::debug!("📝 Appending {} events for aggregate {}", events.len(), aggregate_id);
                
                // Append to backend
                let new_version = self.backend.append_events(aggregate_id, expected_version, events).await?;
                
                // Update metrics
                self.metrics.events_appended.fetch_add(events.len() as u64, std::sync::atomic::Ordering::Relaxed);
                self.metrics.average_append_time.store(
                    start_time.elapsed().as_millis() as u64,
                    std::sync::atomic::Ordering::Relaxed
                );
                
                // Publish events to event bus
                if let Err(e) = self.event_bus.publish_events(events).await {
                    tracing::warn!("Failed to publish events to bus: {}", e);
                }
                
                tracing::debug!("✅ Appended {} events, new version: {:?}", events.len(), new_version);
                Ok(new_version)
            }
            
            /// Load events for aggregate
            async fn load_events(
                &self,
                aggregate_id: &str,
                from_version: Option<EventVersion>
            ) -> Result<Vec<DomainEvent>, EventStoreError> {
                let start_time = std::time::Instant::now();
                
                tracing::debug!("📖 Loading events for aggregate {} from version {:?}", 
                    aggregate_id, from_version);
                
                let events = self.backend.load_events(aggregate_id, from_version).await?;
                
                // Update metrics
                self.metrics.events_loaded.fetch_add(events.len() as u64, std::sync::atomic::Ordering::Relaxed);
                self.metrics.average_load_time.store(
                    start_time.elapsed().as_millis() as u64,
                    std::sync::atomic::Ordering::Relaxed
                );
                
                tracing::debug!("✅ Loaded {} events for aggregate {}", events.len(), aggregate_id);
                Ok(events)
            }
            
            /// Create snapshot for aggregate
            async fn create_snapshot(
                &self,
                aggregate_id: &str,
                version: EventVersion,
                state: serde_json::Value
            ) -> Result<(), EventStoreError> {
                let snapshot = AggregateSnapshot {
                    aggregate_id: aggregate_id.to_string(),
                    aggregate_type: aggregate_type.to_string(),
                    version,
                    state_data: state,
                    created_at: std::time::SystemTime::now(),
                };
                
                self.snapshot_store.save_snapshot(aggregate_id, &snapshot).await?;
                self.metrics.snapshots_created.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                tracing::debug!("📸 Created snapshot for aggregate {} at version {:?}", aggregate_id, version);
                Ok(())
            }
            
            /// Load snapshot for aggregate
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError> {
                let snapshot = self.snapshot_store.load_snapshot(aggregate_id).await?;
                
                if snapshot.is_some() {
                    self.metrics.snapshots_loaded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                
                Ok(snapshot)
            }
        }
        
        impl AggregateManager {
            fn new(event_store: Arc<EventStore>) -> Self {
                Self {
                    event_store,
                    aggregate_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    snapshot_frequency: snapshot_freq,
                }
            }
            
            /// Load aggregate from event store
            async fn load_aggregate(&self, aggregate_id: &str) -> Result<CachedAggregate, EventStoreError> {
                // Check cache first
                {
                    let cache = self.aggregate_cache.read().await;
                    if let Some(cached) = cache.get(aggregate_id) {
                        if cached.last_accessed.elapsed() < std::time::Duration::from_secs(300) { // 5 minutes
                            tracing::debug!("📋 Loaded aggregate {} from cache", aggregate_id);
                            return Ok(cached.clone());
                        }
                    }
                }
                
                // Load from event store
                let mut version = EventVersion::INITIAL;
                let mut state = serde_json::Value::Null;
                
                // Try to load from snapshot first
                if let Some(snapshot) = self.event_store.load_snapshot(aggregate_id).await? {
                    version = snapshot.version;
                    state = snapshot.state_data;
                    tracing::debug!("📸 Loaded aggregate {} from snapshot at version {:?}", aggregate_id, version);
                }
                
                // Load events since snapshot
                let events = self.event_store.load_events(aggregate_id, Some(version)).await?;
                
                // Apply events to rebuild state
                for event in &events {
                    state = self.apply_event_to_state(state, event)?;
                    version = event.event_version;
                }
                
                let cached_aggregate = CachedAggregate {
                    aggregate_id: aggregate_id.to_string(),
                    version,
                    state,
                    last_accessed: std::time::Instant::now(),
                    uncommitted_events: Vec::new(),
                };
                
                // Cache the aggregate
                {
                    let mut cache = self.aggregate_cache.write().await;
                    cache.insert(aggregate_id.to_string(), cached_aggregate.clone());
                }
                
                tracing::debug!("📚 Loaded aggregate {} at version {:?}", aggregate_id, version);
                Ok(cached_aggregate)
            }
            
            /// Save aggregate with uncommitted events
            async fn save_aggregate(&self, mut aggregate: CachedAggregate) -> Result<(), EventStoreError> {
                if aggregate.uncommitted_events.is_empty() {
                    return Ok(()); // Nothing to save
                }
                
                tracing::debug!("💾 Saving aggregate {} with {} uncommitted events", 
                    aggregate.aggregate_id, aggregate.uncommitted_events.len());
                
                // Append events to store
                let new_version = self.event_store.append_events(
                    &aggregate.aggregate_id,
                    aggregate.version,
                    &aggregate.uncommitted_events
                ).await?;
                
                // Update aggregate version
                aggregate.version = new_version;
                aggregate.uncommitted_events.clear();
                
                // Check if we should create a snapshot
                if new_version.0 % (self.snapshot_frequency as i64) == 0 {
                    self.event_store.create_snapshot(
                        &aggregate.aggregate_id,
                        new_version,
                        aggregate.state.clone()
                    ).await?;
                }
                
                // Update cache
                {
                    let mut cache = self.aggregate_cache.write().await;
                    cache.insert(aggregate.aggregate_id.clone(), aggregate);
                }
                
                tracing::debug!("✅ Saved aggregate at version {:?}", new_version);
                Ok(())
            }
            
            /// Apply event to aggregate state using proper aggregate pattern
            fn apply_event_to_state(&self, mut state: serde_json::Value, event: &DomainEvent) -> Result<serde_json::Value, EventStoreError> {
                tracing::debug!("🎯 Applying event {} to aggregate {}", event.event_type, event.aggregate_id);
                
                // Initialize state if null
                if state.is_null() {
                    state = serde_json::json!({
                        "aggregate_id": event.aggregate_id,
                        "aggregate_type": event.aggregate_type,
                        "version": 0,
                        "created_at": event.timestamp,
                        "updated_at": event.timestamp,
                        "state": "active",
                        "data": {}
                    });
                }
                
                // Ensure state is an object for manipulation
                let state_obj = match state.as_object_mut() {
                    Some(obj) => obj,
                    None => return Err(EventStoreError::Serialization(
                        "Invalid aggregate state: expected JSON object".to_string()
                    )),
                };
                
                // Update version and timestamp
                state_obj.insert("version".to_string(), serde_json::json!(event.event_version.0));
                state_obj.insert("updated_at".to_string(), serde_json::json!(event.timestamp));
                state_obj.insert("last_event_type".to_string(), serde_json::Value::String(event.event_type.clone()));
                
                // Apply event based on type using command pattern
                match event.event_type.as_str() {
                    // Create/Initialize events
                    event_type if event_type.ends_with("Created") || event_type.ends_with("Initialized") => {
                        self.apply_creation_event(state_obj, event)?;
                    }
                    
                    // Update events
                    event_type if event_type.ends_with("Updated") || event_type.ends_with("Modified") => {
                        self.apply_update_event(state_obj, event)?;
                    }
                    
                    // Delete/Remove events
                    event_type if event_type.ends_with("Deleted") || event_type.ends_with("Removed") => {
                        self.apply_deletion_event(state_obj, event)?;
                    }
                    
                    // Status change events
                    event_type if event_type.contains("Status") || event_type.ends_with("Activated") || 
                                  event_type.ends_with("Deactivated") || event_type.ends_with("Suspended") => {
                        self.apply_status_change_event(state_obj, event)?;
                    }
                    
                    // Property change events
                    event_type if event_type.ends_with("Changed") || event_type.ends_with("Set") => {
                        self.apply_property_change_event(state_obj, event)?;
                    }
                    
                    // Collection events (add/remove items)
                    event_type if event_type.ends_with("Added") || event_type.ends_with("Appended") => {
                        self.apply_collection_add_event(state_obj, event)?;
                    }
                    
                    event_type if event_type.ends_with("Removed") || event_type.ends_with("Cleared") => {
                        self.apply_collection_remove_event(state_obj, event)?;
                    }
                    
                    // Custom business events - apply data directly with validation
                    _ => {
                        self.apply_generic_business_event(state_obj, event)?;
                    }
                }
                
                // Validate final state consistency
                self.validate_aggregate_state(&state)?;
                
                tracing::debug!("✅ Applied event {} to aggregate {} at version {}", 
                    event.event_type, event.aggregate_id, event.event_version.0);
                
                Ok(state)
            }
            
            /// Apply creation/initialization events
            fn apply_creation_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                // Set initial state
                state.insert("state".to_string(), serde_json::Value::String("active".to_string()));
                
                // Merge event data into aggregate data
                if let Some(data_field) = state.get_mut("data") {
                    if let serde_json::Value::Object(data_obj) = data_field {
                        if let serde_json::Value::Object(event_data) = &event.event_data {
                            for (key, value) in event_data {
                                data_obj.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Apply update/modification events
            fn apply_update_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                // Merge changes into existing data
                if let Some(data_field) = state.get_mut("data") {
                    if let serde_json::Value::Object(data_obj) = data_field {
                        if let serde_json::Value::Object(event_data) = &event.event_data {
                            // Handle different update patterns
                            if let Some(changes) = event_data.get("changes") {
                                if let serde_json::Value::Object(changes_obj) = changes {
                                    for (key, value) in changes_obj {
                                        data_obj.insert(key.clone(), value.clone());
                                    }
                                }
                            } else {
                                // Direct field updates
                                for (key, value) in event_data {
                                    if !["aggregate_id", "version", "timestamp"].contains(&key.as_str()) {
                                        data_obj.insert(key.clone(), value.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Apply deletion/removal events
            fn apply_deletion_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                // Mark as deleted instead of removing
                state.insert("state".to_string(), serde_json::Value::String("deleted".to_string()));
                state.insert("deleted_at".to_string(), serde_json::json!(event.timestamp));
                
                // Store deletion reason if provided
                if let serde_json::Value::Object(event_data) = &event.event_data {
                    if let Some(reason) = event_data.get("reason") {
                        state.insert("deletion_reason".to_string(), reason.clone());
                    }
                }
                
                Ok(())
            }
            
            /// Apply status change events
            fn apply_status_change_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                if let serde_json::Value::Object(event_data) = &event.event_data {
                    // Update status
                    if let Some(new_status) = event_data.get("status").or_else(|| event_data.get("new_status")) {
                        state.insert("state".to_string(), new_status.clone());
                    }
                    
                    // Store previous status if provided
                    if let Some(prev_status) = event_data.get("previous_status") {
                        state.insert("previous_state".to_string(), prev_status.clone());
                    }
                    
                    // Store status change reason
                    if let Some(reason) = event_data.get("reason") {
                        state.insert("status_change_reason".to_string(), reason.clone());
                    }
                }
                
                Ok(())
            }
            
            /// Apply property change events
            fn apply_property_change_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                if let serde_json::Value::Object(event_data) = &event.event_data {
                    if let Some(data_field) = state.get_mut("data") {
                        if let serde_json::Value::Object(data_obj) = data_field {
                            // Handle property changes with old/new value tracking
                            if let (Some(property), Some(new_value)) = (event_data.get("property"), event_data.get("new_value")) {
                                if let Some(property_name) = property.as_str() {
                                    // Store old value in change history if not exists
                                    if let Some(old_value) = data_obj.get(property_name).cloned() {
                                        let history_key = format!("{}_history", property_name);
                                        let mut history = data_obj.get(&history_key)
                                            .and_then(|v| v.as_array())
                                            .cloned()
                                            .unwrap_or_default();
                                        
                                        history.push(serde_json::json!({
                                            "value": old_value,
                                            "changed_at": event.timestamp,
                                            "event_id": event.event_id
                                        }));
                                        
                                        // Keep only last 10 changes
                                        if history.len() > 10 {
                                            history = history.into_iter().skip(history.len() - 10).collect();
                                        }
                                        
                                        data_obj.insert(history_key, serde_json::Value::Array(history));
                                    }
                                    
                                    // Set new value
                                    data_obj.insert(property_name.to_string(), new_value.clone());
                                }
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Apply collection add events
            fn apply_collection_add_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                if let serde_json::Value::Object(event_data) = &event.event_data {
                    if let Some(data_field) = state.get_mut("data") {
                        if let serde_json::Value::Object(data_obj) = data_field {
                            // Handle collection additions
                            if let (Some(collection), Some(item)) = (event_data.get("collection"), event_data.get("item")) {
                                if let Some(collection_name) = collection.as_str() {
                                    let mut collection_array = data_obj.get(collection_name)
                                        .and_then(|v| v.as_array())
                                        .cloned()
                                        .unwrap_or_default();
                                    
                                    collection_array.push(item.clone());
                                    data_obj.insert(collection_name.to_string(), serde_json::Value::Array(collection_array));
                                }
                            } else if let Some(items) = event_data.get("items") {
                                // Bulk add
                                if let serde_json::Value::Array(items_array) = items {
                                    if let Some(collection_name) = event_data.get("collection").and_then(|v| v.as_str()) {
                                        let mut collection_array = data_obj.get(collection_name)
                                            .and_then(|v| v.as_array())
                                            .cloned()
                                            .unwrap_or_default();
                                        
                                        collection_array.extend(items_array.clone());
                                        data_obj.insert(collection_name.to_string(), serde_json::Value::Array(collection_array));
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Apply collection remove events
            fn apply_collection_remove_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                if let serde_json::Value::Object(event_data) = &event.event_data {
                    if let Some(data_field) = state.get_mut("data") {
                        if let serde_json::Value::Object(data_obj) = data_field {
                            if let Some(collection_name) = event_data.get("collection").and_then(|v| v.as_str()) {
                                if event_data.get("clear").and_then(|v| v.as_bool()).unwrap_or(false) {
                                    // Clear entire collection
                                    data_obj.insert(collection_name.to_string(), serde_json::Value::Array(vec![]));
                                } else if let Some(item_id) = event_data.get("item_id") {
                                    // Remove specific item by ID
                                    if let Some(collection_array) = data_obj.get_mut(collection_name) {
                                        if let serde_json::Value::Array(ref mut array) = collection_array {
                                            array.retain(|item| {
                                                item.get("id").map(|id| id != item_id).unwrap_or(true)
                                            });
                                        }
                                    }
                                } else if let Some(index) = event_data.get("index").and_then(|v| v.as_u64()) {
                                    // Remove by index
                                    if let Some(collection_array) = data_obj.get_mut(collection_name) {
                                        if let serde_json::Value::Array(ref mut array) = collection_array {
                                            if (index as usize) < array.len() {
                                                array.remove(index as usize);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Apply generic business events
            fn apply_generic_business_event(&self, state: &mut serde_json::Map<String, serde_json::Value>, event: &DomainEvent) -> Result<(), EventStoreError> {
                tracing::debug!("🔧 Applying generic business event: {}", event.event_type);
                
                // Apply event data with namespace isolation
                if let Some(data_field) = state.get_mut("data") {
                    if let serde_json::Value::Object(data_obj) = data_field {
                        // Create event-specific namespace
                        let event_namespace = format!("events.{}", event.event_type.to_lowercase());
                        let mut event_specific_data = data_obj.get(&event_namespace)
                            .and_then(|v| v.as_object())
                            .cloned()
                            .unwrap_or_default();
                        
                        // Merge event data
                        if let serde_json::Value::Object(event_data) = &event.event_data {
                            for (key, value) in event_data {
                                event_specific_data.insert(key.clone(), value.clone());
                            }
                        }
                        
                        data_obj.insert(event_namespace, serde_json::Value::Object(event_specific_data));
                        
                        // Also apply to root level for commonly accessed fields
                        if let serde_json::Value::Object(event_data) = &event.event_data {
                            for (key, value) in event_data {
                                // Only apply non-system fields to root level
                                if !["aggregate_id", "version", "timestamp", "event_id"].contains(&key.as_str()) {
                                    data_obj.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
                
                Ok(())
            }
            
            /// Validate aggregate state for consistency
            fn validate_aggregate_state(&self, state: &serde_json::Value) -> Result<(), EventStoreError> {
                let state_obj = state.as_object().ok_or_else(|| {
                    EventStoreError::Serialization("Invalid aggregate state: not an object".to_string())
                })?;
                
                // Check required fields
                let required_fields = ["aggregate_id", "aggregate_type", "version"];
                for field in &required_fields {
                    if !state_obj.contains_key(*field) {
                        return Err(EventStoreError::Serialization(
                            format!("Missing required field in aggregate state: {}", field)
                        ));
                    }
                }
                
                // Validate version is non-negative
                if let Some(version) = state_obj.get("version").and_then(|v| v.as_i64()) {
                    if version < 0 {
                        return Err(EventStoreError::Serialization(
                            "Invalid aggregate version: must be non-negative".to_string()
                        ));
                    }
                }
                
                // Validate state value
                if let Some(state_value) = state_obj.get("state").and_then(|v| v.as_str()) {
                    let valid_states = ["active", "inactive", "deleted", "suspended", "pending"];
                    if !valid_states.contains(&state_value) {
                        tracing::warn!("🚨 Unusual aggregate state: {}", state_value);
                    }
                }
                
                Ok(())
            }
        }
        
        /// PostgreSQL backend implementation
        #[async_trait::async_trait]
        impl EventStoreBackend for PostgreSQLEventStore {
            async fn append_events(
                &self,
                aggregate_id: &str,
                expected_version: EventVersion,
                events: &[DomainEvent]
            ) -> Result<EventVersion, EventStoreError> {
                let mut tx = self.pool.begin().await
                    .map_err(|e| EventStoreError::Backend(format!("Transaction start failed: {}", e)))?;
                
                // Check current version for concurrency control
                let current_version: Option<i64> = sqlx::query_scalar(
                    "SELECT MAX(event_version) FROM events WHERE aggregate_id = $1"
                )
                .bind(aggregate_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Version check failed: {}", e)))?;
                
                let current_version = current_version.map(EventVersion).unwrap_or(EventVersion::INITIAL);
                
                if current_version != expected_version {
                    return Err(EventStoreError::ConcurrencyConflict {
                        expected: expected_version,
                        actual: current_version,
                    });
                }
                
                // Insert events
                let mut new_version = expected_version;
                for event in events {
                    new_version = new_version.next();
                    
                    sqlx::query(
                        r#"
                        INSERT INTO events (
                            event_id, aggregate_id, aggregate_type, event_type, 
                            event_version, event_data, metadata, timestamp, 
                            correlation_id, causation_id
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                        "#
                    )
                    .bind(&event.event_id)
                    .bind(&event.aggregate_id)
                    .bind(&event.aggregate_type)
                    .bind(&event.event_type)
                    .bind(new_version.0)
                    .bind(&event.event_data)
                    .bind(&event.metadata)
                    .bind(event.timestamp)
                    .bind(&event.correlation_id)
                    .bind(&event.causation_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Event insert failed: {}", e)))?;
                }
                
                tx.commit().await
                    .map_err(|e| EventStoreError::Backend(format!("Transaction commit failed: {}", e)))?;
                
                Ok(new_version)
            }
            
            async fn load_events(
                &self,
                aggregate_id: &str,
                from_version: Option<EventVersion>
            ) -> Result<Vec<DomainEvent>, EventStoreError> {
                let from_ver = from_version.unwrap_or(EventVersion::INITIAL).0;
                
                let rows = sqlx::query(
                    r#"
                    SELECT event_id, aggregate_id, aggregate_type, event_type,
                           event_version, event_data, metadata, timestamp,
                           correlation_id, causation_id
                    FROM events 
                    WHERE aggregate_id = $1 AND event_version > $2
                    ORDER BY event_version
                    "#
                )
                .bind(aggregate_id)
                .bind(from_ver)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Event load failed: {}", e)))?;
                
                let mut events = Vec::new();
                for row in rows {
                    let event = DomainEvent {
                        event_id: row.get("event_id"),
                        aggregate_id: row.get("aggregate_id"),
                        aggregate_type: row.get("aggregate_type"),
                        event_type: row.get("event_type"),
                        event_version: EventVersion(row.get("event_version")),
                        event_data: row.get("event_data"),
                        metadata: row.get("metadata"),
                        timestamp: row.get("timestamp"),
                        correlation_id: row.get("correlation_id"),
                        causation_id: row.get("causation_id"),
                    };
                    events.push(event);
                }
                
                Ok(events)
            }
            
            async fn load_events_by_type(
                &self,
                event_type: &str,
                from_timestamp: Option<std::time::SystemTime>
            ) -> Result<Vec<DomainEvent>, EventStoreError> {
                // Implementation for loading events by type
                let _ = (event_type, from_timestamp);
                Ok(Vec::new())
            }
            
            async fn get_aggregate_version(&self, aggregate_id: &str) -> Result<Option<EventVersion>, EventStoreError> {
                let version: Option<i64> = sqlx::query_scalar(
                    "SELECT MAX(event_version) FROM events WHERE aggregate_id = $1"
                )
                .bind(aggregate_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Version query failed: {}", e)))?;
                
                Ok(version.map(EventVersion))
            }
        }
        
        /// In-memory event store backend implementation for testing
        #[async_trait::async_trait]
        impl EventStoreBackend for InMemoryEventStore {
            async fn append_events(
                &self,
                aggregate_id: &str,
                expected_version: EventVersion,
                events: &[DomainEvent]
            ) -> Result<EventVersion, EventStoreError> {
                tracing::debug!("📝 InMemory: Appending {} events for aggregate {}", events.len(), aggregate_id);
                
                // Check and update version atomically
                {
                    let mut versions = self.versions.write().await;
                    let current_version = versions.get(aggregate_id).copied().unwrap_or(EventVersion::INITIAL);
                    
                    if current_version != expected_version {
                        return Err(EventStoreError::ConcurrencyConflict {
                            expected: expected_version,
                            actual: current_version,
                        });
                    }
                    
                    // Calculate new version
                    let mut new_version = expected_version;
                    for _ in events {
                        new_version = new_version.next();
                    }
                    
                    // Update version first
                    versions.insert(aggregate_id.to_string(), new_version);
                    
                    // Append events to storage
                    {
                        let mut events_store = self.events.write().await;
                        let aggregate_events = events_store.entry(aggregate_id.to_string()).or_insert_with(Vec::new);
                        
                        // Clone and adjust event versions
                        let mut event_version = expected_version;
                        for event in events {
                            event_version = event_version.next();
                            let mut adjusted_event = event.clone();
                            adjusted_event.event_version = event_version;
                            aggregate_events.push(adjusted_event);
                        }
                    }
                    
                    tracing::debug!("✅ InMemory: Appended {} events, new version: {:?}", events.len(), new_version);
                    Ok(new_version)
                }
            }
            
            async fn load_events(
                &self,
                aggregate_id: &str,
                from_version: Option<EventVersion>
            ) -> Result<Vec<DomainEvent>, EventStoreError> {
                tracing::debug!("📖 InMemory: Loading events for aggregate {} from version {:?}", 
                    aggregate_id, from_version);
                
                let events_store = self.events.read().await;
                let from_ver = from_version.unwrap_or(EventVersion::INITIAL);
                
                let filtered_events = events_store
                    .get(aggregate_id)
                    .map(|events| {
                        events.iter()
                            .filter(|event| event.event_version > from_ver)
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                
                tracing::debug!("✅ InMemory: Loaded {} events for aggregate {}", 
                    filtered_events.len(), aggregate_id);
                
                Ok(filtered_events)
            }
            
            async fn load_events_by_type(
                &self,
                event_type: &str,
                from_timestamp: Option<std::time::SystemTime>
            ) -> Result<Vec<DomainEvent>, EventStoreError> {
                tracing::debug!("🔍 InMemory: Loading events by type '{}' from timestamp {:?}", 
                    event_type, from_timestamp);
                
                let events_store = self.events.read().await;
                let from_time = from_timestamp.unwrap_or(std::time::UNIX_EPOCH);
                
                let mut matching_events = Vec::new();
                
                for (_aggregate_id, events) in events_store.iter() {
                    for event in events {
                        if event.event_type == event_type && event.timestamp >= from_time {
                            matching_events.push(event.clone());
                        }
                    }
                }
                
                // Sort by timestamp for consistent ordering
                matching_events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                
                tracing::debug!("✅ InMemory: Found {} events of type '{}'", 
                    matching_events.len(), event_type);
                
                Ok(matching_events)
            }
            
            async fn get_aggregate_version(&self, aggregate_id: &str) -> Result<Option<EventVersion>, EventStoreError> {
                let versions = self.versions.read().await;
                let version = versions.get(aggregate_id).copied();
                
                tracing::debug!("🔍 InMemory: Aggregate {} version: {:?}", aggregate_id, version);
                Ok(version)
            }
        }
        
        impl EventStoreMetrics {
            fn new() -> Self {
                Self {
                    events_appended: AtomicU64::new(0),
                    events_loaded: AtomicU64::new(0),
                    snapshots_created: AtomicU64::new(0),
                    snapshots_loaded: AtomicU64::new(0),
                    concurrency_conflicts: AtomicU64::new(0),
                    average_append_time: AtomicU64::new(0),
                    average_load_time: AtomicU64::new(0),
                }
            }
        }
        
        /// Utility functions
        
        /// Extract aggregate ID from message
        fn extract_aggregate_id(msg: &rabbitmesh::Message, aggregate_type: &str) -> Option<String> {
            // Try header first
            if let Some(headers) = &msg.headers {
                if let Some(id) = headers.get("aggregate_id").and_then(|v| v.as_str()) {
                    return Some(id.to_string());
                }
                
                // Try type-specific header
                let header_key = format!("{}_id", aggregate_type);
                if let Some(id) = headers.get(&header_key).and_then(|v| v.as_str()) {
                    return Some(id.to_string());
                }
            }
            
            // Try payload
            if let Some(obj) = msg.payload.as_object() {
                let id_fields = ["id", "aggregate_id", &format!("{}_id", aggregate_type)];
                for field in &id_fields {
                    if let Some(id) = obj.get(*field).and_then(|v| v.as_str()) {
                        return Some(id.to_string());
                    }
                }
            }
            
            None
        }
        
        /// PostgreSQL snapshot store implementation
        struct PostgreSQLSnapshotStore {
            pool: sqlx::PgPool,
        }
        
        #[async_trait::async_trait]
        impl SnapshotStore for PostgreSQLSnapshotStore {
            async fn save_snapshot(&self, aggregate_id: &str, snapshot: &AggregateSnapshot) -> Result<(), EventStoreError> {
                tracing::debug!("📸 PostgreSQL: Saving snapshot for aggregate {} at version {:?}", 
                    aggregate_id, snapshot.version);
                
                sqlx::query(
                    r#"
                    INSERT INTO snapshots (
                        aggregate_id, aggregate_type, version, state_data, created_at
                    ) VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (aggregate_id) 
                    DO UPDATE SET 
                        aggregate_type = EXCLUDED.aggregate_type,
                        version = EXCLUDED.version,
                        state_data = EXCLUDED.state_data,
                        created_at = EXCLUDED.created_at
                    "#
                )
                .bind(&snapshot.aggregate_id)
                .bind(&snapshot.aggregate_type)
                .bind(snapshot.version.0)
                .bind(&snapshot.state_data)
                .bind(snapshot.created_at)
                .execute(&self.pool)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Snapshot save failed: {}", e)))?;
                
                tracing::debug!("✅ PostgreSQL: Saved snapshot for aggregate {}", aggregate_id);
                Ok(())
            }
            
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError> {
                tracing::debug!("📖 PostgreSQL: Loading snapshot for aggregate {}", aggregate_id);
                
                let row = sqlx::query(
                    r#"
                    SELECT aggregate_id, aggregate_type, version, state_data, created_at
                    FROM snapshots 
                    WHERE aggregate_id = $1
                    ORDER BY version DESC 
                    LIMIT 1
                    "#
                )
                .bind(aggregate_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Snapshot load failed: {}", e)))?;
                
                if let Some(row) = row {
                    let snapshot = AggregateSnapshot {
                        aggregate_id: row.get("aggregate_id"),
                        aggregate_type: row.get("aggregate_type"),
                        version: EventVersion(row.get("version")),
                        state_data: row.get("state_data"),
                        created_at: row.get("created_at"),
                    };
                    
                    tracing::debug!("✅ PostgreSQL: Loaded snapshot for aggregate {} at version {:?}", 
                        aggregate_id, snapshot.version);
                    Ok(Some(snapshot))
                } else {
                    tracing::debug!("📭 PostgreSQL: No snapshot found for aggregate {}", aggregate_id);
                    Ok(None)
                }
            }
            
            async fn delete_old_snapshots(&self, aggregate_id: &str, keep_count: u32) -> Result<(), EventStoreError> {
                tracing::debug!("🗑️ PostgreSQL: Cleaning old snapshots for aggregate {}, keeping {}", 
                    aggregate_id, keep_count);
                
                sqlx::query(
                    r#"
                    DELETE FROM snapshots 
                    WHERE aggregate_id = $1 
                    AND version NOT IN (
                        SELECT version FROM snapshots 
                        WHERE aggregate_id = $1 
                        ORDER BY version DESC 
                        LIMIT $2
                    )
                    "#
                )
                .bind(aggregate_id)
                .bind(keep_count as i64)
                .execute(&self.pool)
                .await
                .map_err(|e| EventStoreError::Backend(format!("Snapshot cleanup failed: {}", e)))?;
                
                tracing::debug!("✅ PostgreSQL: Cleaned old snapshots for aggregate {}", aggregate_id);
                Ok(())
            }
        }
        
        /// In-memory snapshot store for testing
        struct InMemorySnapshotStore {
            snapshots: Arc<RwLock<std::collections::HashMap<String, AggregateSnapshot>>>,
        }
        
        #[async_trait::async_trait]
        impl SnapshotStore for InMemorySnapshotStore {
            async fn save_snapshot(&self, aggregate_id: &str, snapshot: &AggregateSnapshot) -> Result<(), EventStoreError> {
                tracing::debug!("📸 InMemory: Saving snapshot for aggregate {} at version {:?}", 
                    aggregate_id, snapshot.version);
                
                let mut snapshots = self.snapshots.write().await;
                snapshots.insert(aggregate_id.to_string(), snapshot.clone());
                
                tracing::debug!("✅ InMemory: Saved snapshot for aggregate {}", aggregate_id);
                Ok(())
            }
            
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError> {
                tracing::debug!("📖 InMemory: Loading snapshot for aggregate {}", aggregate_id);
                
                let snapshots = self.snapshots.read().await;
                let snapshot = snapshots.get(aggregate_id).cloned();
                
                if snapshot.is_some() {
                    tracing::debug!("✅ InMemory: Loaded snapshot for aggregate {}", aggregate_id);
                } else {
                    tracing::debug!("📭 InMemory: No snapshot found for aggregate {}", aggregate_id);
                }
                
                Ok(snapshot)
            }
            
            async fn delete_old_snapshots(&self, aggregate_id: &str, keep_count: u32) -> Result<(), EventStoreError> {
                tracing::debug!("🗑️ InMemory: Cleaning old snapshots for aggregate {} (keep_count: {} - not applicable for in-memory)", 
                    aggregate_id, keep_count);
                
                // For in-memory store, we only keep one snapshot per aggregate
                // So this is effectively a no-op but we log it for completeness
                Ok(())
            }
        }
        
        /// File-based snapshot store implementation
        struct FileSnapshotStore {
            base_path: std::path::PathBuf,
        }
        
        #[async_trait::async_trait]
        impl SnapshotStore for FileSnapshotStore {
            async fn save_snapshot(&self, aggregate_id: &str, snapshot: &AggregateSnapshot) -> Result<(), EventStoreError> {
                tracing::debug!("📸 File: Saving snapshot for aggregate {} at version {:?}", 
                    aggregate_id, snapshot.version);
                
                let snapshot_dir = self.base_path.join("snapshots").join(&snapshot.aggregate_type);
                tokio::fs::create_dir_all(&snapshot_dir).await
                    .map_err(|e| EventStoreError::Backend(format!("Failed to create snapshot directory: {}", e)))?;
                
                let snapshot_file = snapshot_dir.join(format!("{}.json", aggregate_id));
                let snapshot_data = serde_json::to_string_pretty(snapshot)
                    .map_err(|e| EventStoreError::Serialization(format!("Snapshot serialization failed: {}", e)))?;
                
                tokio::fs::write(&snapshot_file, snapshot_data).await
                    .map_err(|e| EventStoreError::Backend(format!("Snapshot file write failed: {}", e)))?;
                
                tracing::debug!("✅ File: Saved snapshot for aggregate {} to {:?}", aggregate_id, snapshot_file);
                Ok(())
            }
            
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError> {
                tracing::debug!("📖 File: Loading snapshot for aggregate {}", aggregate_id);
                
                // We need to find the snapshot file, but we don't know the aggregate type
                // So we'll search in all subdirectories
                let snapshots_dir = self.base_path.join("snapshots");
                
                if !snapshots_dir.exists() {
                    tracing::debug!("📭 File: Snapshots directory doesn't exist");
                    return Ok(None);
                }
                
                let mut entries = tokio::fs::read_dir(&snapshots_dir).await
                    .map_err(|e| EventStoreError::Backend(format!("Failed to read snapshots directory: {}", e)))?;
                
                while let Some(entry) = entries.next_entry().await
                    .map_err(|e| EventStoreError::Backend(format!("Failed to read directory entry: {}", e)))? {
                    
                    if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                        let snapshot_file = entry.path().join(format!("{}.json", aggregate_id));
                        
                        if snapshot_file.exists() {
                            let content = tokio::fs::read_to_string(&snapshot_file).await
                                .map_err(|e| EventStoreError::Backend(format!("Failed to read snapshot file: {}", e)))?;
                            
                            let snapshot: AggregateSnapshot = serde_json::from_str(&content)
                                .map_err(|e| EventStoreError::Serialization(format!("Snapshot deserialization failed: {}", e)))?;
                            
                            tracing::debug!("✅ File: Loaded snapshot for aggregate {} from {:?}", aggregate_id, snapshot_file);
                            return Ok(Some(snapshot));
                        }
                    }
                }
                
                tracing::debug!("📭 File: No snapshot found for aggregate {}", aggregate_id);
                Ok(None)
            }
            
            async fn delete_old_snapshots(&self, aggregate_id: &str, keep_count: u32) -> Result<(), EventStoreError> {
                tracing::debug!("🗑️ File: Cleaning old snapshots for aggregate {} (keep_count: {} - not applicable for file store)", 
                    aggregate_id, keep_count);
                
                // For file-based store, we only keep one snapshot per aggregate
                // So this is effectively a no-op
                Ok(())
            }
        }
        
        /// Redis snapshot store implementation
        struct RedisSnapshotStore {
            client: redis::Client,
        }
        
        #[async_trait::async_trait]
        impl SnapshotStore for RedisSnapshotStore {
            async fn save_snapshot(&self, aggregate_id: &str, snapshot: &AggregateSnapshot) -> Result<(), EventStoreError> {
                tracing::debug!("📸 Redis: Saving snapshot for aggregate {} at version {:?}", 
                    aggregate_id, snapshot.version);
                
                let mut conn = self.client.get_async_connection().await
                    .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?;
                
                let snapshot_data = serde_json::to_string(snapshot)
                    .map_err(|e| EventStoreError::Serialization(format!("Snapshot serialization failed: {}", e)))?;
                
                let key = format!("snapshot:{}", aggregate_id);
                let _: () = redis::cmd("SET")
                    .arg(&key)
                    .arg(&snapshot_data)
                    .arg("EX")
                    .arg(86400 * 7) // Expire after 7 days
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Redis SET failed: {}", e)))?;
                
                // Also store in a sorted set for easier querying by version
                let zset_key = format!("snapshots:{}:versions", snapshot.aggregate_type);
                let _: () = redis::cmd("ZADD")
                    .arg(&zset_key)
                    .arg(snapshot.version.0)
                    .arg(aggregate_id)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Redis ZADD failed: {}", e)))?;
                
                tracing::debug!("✅ Redis: Saved snapshot for aggregate {}", aggregate_id);
                Ok(())
            }
            
            async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<AggregateSnapshot>, EventStoreError> {
                tracing::debug!("📖 Redis: Loading snapshot for aggregate {}", aggregate_id);
                
                let mut conn = self.client.get_async_connection().await
                    .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?;
                
                let key = format!("snapshot:{}", aggregate_id);
                let result: Option<String> = redis::cmd("GET")
                    .arg(&key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Redis GET failed: {}", e)))?;
                
                if let Some(data) = result {
                    let snapshot: AggregateSnapshot = serde_json::from_str(&data)
                        .map_err(|e| EventStoreError::Serialization(format!("Snapshot deserialization failed: {}", e)))?;
                    
                    tracing::debug!("✅ Redis: Loaded snapshot for aggregate {} at version {:?}", 
                        aggregate_id, snapshot.version);
                    Ok(Some(snapshot))
                } else {
                    tracing::debug!("📭 Redis: No snapshot found for aggregate {}", aggregate_id);
                    Ok(None)
                }
            }
            
            async fn delete_old_snapshots(&self, aggregate_id: &str, keep_count: u32) -> Result<(), EventStoreError> {
                tracing::debug!("🗑️ Redis: Cleaning old snapshots for aggregate {} (keep_count: {} - not applicable for Redis TTL)", 
                    aggregate_id, keep_count);
                
                // Redis snapshots have TTL, so this is effectively a no-op
                // But we could implement version-based cleanup if needed
                Ok(())
            }
        }
        
        /// Create event store backend
        async fn create_event_store_backend(backend_type: &str) -> Result<Arc<dyn EventStoreBackend + Send + Sync>, EventStoreError> {
            match backend_type.to_lowercase().as_str() {
                "postgres" | "postgresql" => {
                    let database_url = std::env::var("DATABASE_URL")
                        .map_err(|_| EventStoreError::Configuration("DATABASE_URL not set".to_string()))?;
                    
                    let pool = sqlx::PgPool::connect(&database_url).await
                        .map_err(|e| EventStoreError::Backend(format!("PostgreSQL connection failed: {}", e)))?;
                    
                    Ok(Arc::new(PostgreSQLEventStore { pool }))
                }
                "memory" | "test" => {
                    Ok(Arc::new(InMemoryEventStore {
                        events: Arc::new(RwLock::new(std::collections::HashMap::new())),
                        versions: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    }))
                }
                _ => Err(EventStoreError::Configuration(format!("Unsupported backend: {}", backend_type))),
            }
        }
        
        /// Create event serializer based on configuration
        fn create_event_serializer() -> Box<dyn EventSerializer + Send + Sync> {
            let serializer_type = std::env::var("EVENT_SERIALIZER_TYPE")
                .unwrap_or_else(|_| "json".to_string());
            
            tracing::debug!("🔄 Creating event serializer for type: {}", serializer_type);
            
            match serializer_type.to_lowercase().as_str() {
                "json" => {
                    tracing::info!("🔄 Using JSON event serializer");
                    Box::new(JsonEventSerializer)
                }
                "binary" | "messagepack" | "msgpack" => {
                    tracing::info!("🔄 Using MessagePack binary event serializer");
                    Box::new(BinaryEventSerializer)
                }
                "cbor" => {
                    tracing::info!("🔄 Using CBOR event serializer");
                    Box::new(CborEventSerializer)
                }
                "protobuf" | "proto" => {
                    tracing::info!("🔄 Using Protobuf event serializer (with JSON fallback)");
                    Box::new(ProtobufEventSerializer)
                }
                _ => {
                    tracing::warn!("🚨 Unsupported serializer type '{}', falling back to JSON", serializer_type);
                    Box::new(JsonEventSerializer)
                }
            }
        }
        
        /// Create snapshot store
        async fn create_snapshot_store(backend_type: &str) -> Result<Box<dyn SnapshotStore + Send + Sync>, EventStoreError> {
            tracing::debug!("📸 Creating snapshot store for backend: {}", backend_type);
            
            match backend_type.to_lowercase().as_str() {
                "postgres" | "postgresql" => {
                    let database_url = std::env::var("DATABASE_URL")
                        .map_err(|_| EventStoreError::Configuration("DATABASE_URL not set for PostgreSQL snapshot store".to_string()))?;
                    
                    let pool = sqlx::PgPool::connect(&database_url).await
                        .map_err(|e| EventStoreError::Backend(format!("PostgreSQL connection failed for snapshot store: {}", e)))?;
                    
                    // Create snapshots table if it doesn't exist
                    sqlx::query(
                        r#"
                        CREATE TABLE IF NOT EXISTS snapshots (
                            aggregate_id VARCHAR PRIMARY KEY,
                            aggregate_type VARCHAR NOT NULL,
                            version BIGINT NOT NULL,
                            state_data JSONB NOT NULL,
                            created_at TIMESTAMP WITH TIME ZONE NOT NULL
                        )
                        "#
                    )
                    .execute(&pool)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Failed to create snapshots table: {}", e)))?;
                    
                    // Create index for faster queries
                    sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_aggregate_type ON snapshots(aggregate_type)")
                        .execute(&pool)
                        .await
                        .map_err(|e| EventStoreError::Backend(format!("Failed to create snapshots index: {}", e)))?;
                    
                    tracing::info!("📸 Created PostgreSQL snapshot store");
                    Ok(Box::new(PostgreSQLSnapshotStore { pool }))
                }
                "memory" | "test" => {
                    tracing::info!("📸 Created in-memory snapshot store");
                    Ok(Box::new(InMemorySnapshotStore {
                        snapshots: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    }))
                }
                "file" => {
                    let base_path = std::env::var("SNAPSHOT_FILE_PATH")
                        .unwrap_or_else(|_| "./data".to_string());
                    
                    let path = std::path::PathBuf::from(base_path);
                    tokio::fs::create_dir_all(&path).await
                        .map_err(|e| EventStoreError::Backend(format!("Failed to create snapshot directory: {}", e)))?;
                    
                    tracing::info!("📸 Created file-based snapshot store at {:?}", path);
                    Ok(Box::new(FileSnapshotStore { base_path: path }))
                }
                "redis" => {
                    // Redis snapshot store implementation
                    let redis_url = std::env::var("REDIS_URL")
                        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
                    
                    let client = redis::Client::open(redis_url.as_str())
                        .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?;
                    
                    // Test connection
                    let mut conn = client.get_async_connection().await
                        .map_err(|e| EventStoreError::Backend(format!("Redis connection test failed: {}", e)))?;
                    
                    let _: String = redis::cmd("PING")
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| EventStoreError::Backend(format!("Redis ping failed: {}", e)))?;
                    
                    tracing::info!("📸 Created Redis snapshot store");
                    Ok(Box::new(RedisSnapshotStore { client }))
                }
                _ => {
                    tracing::warn!("🚨 Unsupported snapshot store backend '{}', falling back to in-memory", backend_type);
                    Ok(Box::new(InMemorySnapshotStore {
                        snapshots: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    }))
                }
            }
        }
        
        /// RabbitMQ event bus implementation
        struct RabbitMQEventBus {
            channel: Arc<tokio::sync::Mutex<lapin::Channel>>,
            exchange_name: String,
        }
        
        #[async_trait::async_trait]
        impl EventBus for RabbitMQEventBus {
            async fn publish_event(&self, event: &DomainEvent) -> Result<(), EventStoreError> {
                tracing::debug!("📢 RabbitMQ: Publishing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                let channel = self.channel.lock().await;
                
                // Serialize event
                let payload = serde_json::to_vec(event)
                    .map_err(|e| EventStoreError::Serialization(format!("Event serialization failed: {}", e)))?;
                
                // Create routing key based on aggregate type and event type
                let routing_key = format!("{}.{}", event.aggregate_type.to_lowercase(), event.event_type.to_lowercase());
                
                // Publish to exchange
                let publish_result = channel.basic_publish(
                    &self.exchange_name,
                    &routing_key,
                    lapin::options::BasicPublishOptions::default(),
                    &payload,
                    lapin::BasicProperties::default()
                        .with_message_id(event.event_id.clone().into())
                        .with_correlation_id(event.correlation_id.clone().map(|id| id.into()))
                        .with_timestamp(event.timestamp.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default().as_secs())
                        .with_content_type("application/json".into())
                        .with_content_encoding("utf-8".into())
                ).await
                .map_err(|e| EventStoreError::Backend(format!("RabbitMQ publish failed: {}", e)))?;
                
                // Wait for confirmation
                publish_result.await
                    .map_err(|e| EventStoreError::Backend(format!("RabbitMQ publish confirmation failed: {}", e)))?;
                
                tracing::debug!("✅ RabbitMQ: Published event {} with routing key {}", 
                    event.event_type, routing_key);
                Ok(())
            }
            
            async fn publish_events(&self, events: &[DomainEvent]) -> Result<(), EventStoreError> {
                tracing::debug!("📢 RabbitMQ: Publishing {} events", events.len());
                
                for event in events {
                    self.publish_event(event).await?;
                }
                
                tracing::debug!("✅ RabbitMQ: Published {} events", events.len());
                Ok(())
            }
            
            async fn subscribe_to_events<F>(&self, event_type: &str, handler: F) -> Result<(), EventStoreError>
            where
                F: Fn(DomainEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
            {
                tracing::debug!("🔔 RabbitMQ: Subscribing to events of type {}", event_type);
                
                let channel = self.channel.lock().await;
                
                // Create queue for this event type
                let queue_name = format!("events.{}", event_type.to_lowercase());
                let queue = channel.queue_declare(
                    &queue_name,
                    lapin::options::QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    lapin::types::FieldTable::default(),
                ).await
                .map_err(|e| EventStoreError::Backend(format!("RabbitMQ queue declare failed: {}", e)))?;
                
                // Bind queue to exchange with routing pattern
                let routing_pattern = format!("*.{}", event_type.to_lowercase());
                channel.queue_bind(
                    &queue_name,
                    &self.exchange_name,
                    &routing_pattern,
                    lapin::options::QueueBindOptions::default(),
                    lapin::types::FieldTable::default(),
                ).await
                .map_err(|e| EventStoreError::Backend(format!("RabbitMQ queue bind failed: {}", e)))?;
                
                // Start consumer
                let mut consumer = channel.basic_consume(
                    &queue_name,
                    &format!("event-consumer-{}", event_type),
                    lapin::options::BasicConsumeOptions::default(),
                    lapin::types::FieldTable::default(),
                ).await
                .map_err(|e| EventStoreError::Backend(format!("RabbitMQ consume failed: {}", e)))?;
                
                tracing::info!("🔔 RabbitMQ: Started consuming events of type {} from queue {}", 
                    event_type, queue_name);
                
                // Spawn consumer task
                let handler = Arc::new(handler);
                tokio::spawn(async move {
                    while let Some(delivery) = consumer.next().await {
                        match delivery {
                            Ok(delivery) => {
                                // Deserialize event
                                match serde_json::from_slice::<DomainEvent>(&delivery.data) {
                                    Ok(event) => {
                                        tracing::debug!("📨 RabbitMQ: Received event {}", event.event_type);
                                        
                                        // Call handler
                                        handler(event).await;
                                        
                                        // Acknowledge message
                                        if let Err(e) = delivery.ack(lapin::options::BasicAckOptions::default()).await {
                                            tracing::error!("❌ RabbitMQ: Failed to ack message: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ RabbitMQ: Failed to deserialize event: {}", e);
                                        // Reject and don't requeue malformed messages
                                        if let Err(e) = delivery.reject(lapin::options::BasicRejectOptions { requeue: false }).await {
                                            tracing::error!("❌ RabbitMQ: Failed to reject message: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("❌ RabbitMQ: Consumer error: {}", e);
                                break;
                            }
                        }
                    }
                });
                
                Ok(())
            }
        }
        
        /// In-memory event bus for testing
        struct InMemoryEventBus {
            subscribers: Arc<RwLock<std::collections::HashMap<String, Vec<Box<dyn Fn(DomainEvent) + Send + Sync>>>>>,
            published_events: Arc<RwLock<Vec<DomainEvent>>>,
        }
        
        #[async_trait::async_trait]
        impl EventBus for InMemoryEventBus {
            async fn publish_event(&self, event: &DomainEvent) -> Result<(), EventStoreError> {
                tracing::debug!("📢 InMemory: Publishing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                // Store event for debugging
                {
                    let mut published = self.published_events.write().await;
                    published.push(event.clone());
                }
                
                // Call subscribers
                {
                    let subscribers = self.subscribers.read().await;
                    if let Some(handlers) = subscribers.get(&event.event_type) {
                        for handler in handlers {
                            handler(event.clone());
                        }
                    }
                    
                    // Also call wildcard subscribers
                    if let Some(handlers) = subscribers.get("*") {
                        for handler in handlers {
                            handler(event.clone());
                        }
                    }
                }
                
                tracing::debug!("✅ InMemory: Published event {}", event.event_type);
                Ok(())
            }
            
            async fn publish_events(&self, events: &[DomainEvent]) -> Result<(), EventStoreError> {
                tracing::debug!("📢 InMemory: Publishing {} events", events.len());
                
                for event in events {
                    self.publish_event(event).await?;
                }
                
                tracing::debug!("✅ InMemory: Published {} events", events.len());
                Ok(())
            }
            
            async fn subscribe_to_events<F>(&self, event_type: &str, handler: F) -> Result<(), EventStoreError>
            where
                F: Fn(DomainEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
            {
                tracing::debug!("🔔 InMemory: Subscribing to events of type {}", event_type);
                
                // Note: In-memory implementation is simplified and doesn't support async handlers
                // This is mainly for testing purposes
                tracing::warn!("⚠️ InMemory event bus has limited functionality - mainly for testing");
                
                Ok(())
            }
        }
        
        /// Redis event bus implementation using pub/sub
        struct RedisEventBus {
            client: redis::Client,
        }
        
        #[async_trait::async_trait]
        impl EventBus for RedisEventBus {
            async fn publish_event(&self, event: &DomainEvent) -> Result<(), EventStoreError> {
                tracing::debug!("📢 Redis: Publishing event {} for aggregate {}", 
                    event.event_type, event.aggregate_id);
                
                let mut conn = self.client.get_async_connection().await
                    .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?;
                
                // Serialize event
                let payload = serde_json::to_string(event)
                    .map_err(|e| EventStoreError::Serialization(format!("Event serialization failed: {}", e)))?;
                
                // Create channel name
                let channel = format!("events.{}", event.event_type.to_lowercase());
                
                // Publish to Redis channel
                let _: i32 = redis::cmd("PUBLISH")
                    .arg(&channel)
                    .arg(&payload)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| EventStoreError::Backend(format!("Redis publish failed: {}", e)))?;
                
                tracing::debug!("✅ Redis: Published event {} to channel {}", 
                    event.event_type, channel);
                Ok(())
            }
            
            async fn publish_events(&self, events: &[DomainEvent]) -> Result<(), EventStoreError> {
                tracing::debug!("📢 Redis: Publishing {} events", events.len());
                
                for event in events {
                    self.publish_event(event).await?;
                }
                
                tracing::debug!("✅ Redis: Published {} events", events.len());
                Ok(())
            }
            
            async fn subscribe_to_events<F>(&self, event_type: &str, handler: F) -> Result<(), EventStoreError>
            where
                F: Fn(DomainEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
            {
                tracing::debug!("🔔 Redis: Subscribing to events of type {}", event_type);
                
                let mut pubsub = self.client.get_async_connection().await
                    .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?
                    .into_pubsub();
                
                let channel = format!("events.{}", event_type.to_lowercase());
                pubsub.subscribe(&channel).await
                    .map_err(|e| EventStoreError::Backend(format!("Redis subscribe failed: {}", e)))?;
                
                tracing::info!("🔔 Redis: Subscribed to channel {}", channel);
                
                // Spawn subscriber task
                let handler = Arc::new(handler);
                tokio::spawn(async move {
                    let mut stream = pubsub.on_message();
                    while let Some(msg) = stream.next().await {
                        let payload: String = msg.get_payload().unwrap_or_default();
                        match serde_json::from_str::<DomainEvent>(&payload) {
                            Ok(event) => {
                                tracing::debug!("📨 Redis: Received event {}", event.event_type);
                                handler(event).await;
                            }
                            Err(e) => {
                                tracing::error!("❌ Redis: Failed to deserialize event: {}", e);
                            }
                        }
                    }
                });
                
                Ok(())
            }
        }
        
        /// Create event bus
        async fn create_event_bus() -> Result<Box<dyn EventBus + Send + Sync>, EventStoreError> {
            let bus_type = std::env::var("EVENT_BUS_TYPE").unwrap_or_else(|_| "rabbitmq".to_string());
            
            tracing::debug!("📢 Creating event bus for type: {}", bus_type);
            
            match bus_type.to_lowercase().as_str() {
                "rabbitmq" | "rabbit" => {
                    let amqp_url = std::env::var("AMQP_URL")
                        .unwrap_or_else(|_| "amqp://localhost:5672".to_string());
                    
                    let connection = lapin::Connection::connect(&amqp_url, lapin::ConnectionProperties::default()).await
                        .map_err(|e| EventStoreError::Backend(format!("RabbitMQ connection failed: {}", e)))?;
                    
                    let channel = connection.create_channel().await
                        .map_err(|e| EventStoreError::Backend(format!("RabbitMQ channel creation failed: {}", e)))?;
                    
                    // Declare exchange for events
                    let exchange_name = std::env::var("EVENT_EXCHANGE_NAME")
                        .unwrap_or_else(|_| "domain.events".to_string());
                    
                    channel.exchange_declare(
                        &exchange_name,
                        lapin::ExchangeKind::Topic,
                        lapin::options::ExchangeDeclareOptions {
                            durable: true,
                            ..Default::default()
                        },
                        lapin::types::FieldTable::default(),
                    ).await
                    .map_err(|e| EventStoreError::Backend(format!("RabbitMQ exchange declare failed: {}", e)))?;
                    
                    // Enable publisher confirms
                    channel.confirm_select(lapin::options::ConfirmSelectOptions::default()).await
                        .map_err(|e| EventStoreError::Backend(format!("RabbitMQ confirm select failed: {}", e)))?;
                    
                    tracing::info!("📢 Created RabbitMQ event bus with exchange: {}", exchange_name);
                    Ok(Box::new(RabbitMQEventBus {
                        channel: Arc::new(tokio::sync::Mutex::new(channel)),
                        exchange_name,
                    }))
                }
                "redis" => {
                    let redis_url = std::env::var("REDIS_URL")
                        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
                    
                    let client = redis::Client::open(redis_url.as_str())
                        .map_err(|e| EventStoreError::Backend(format!("Redis connection failed: {}", e)))?;
                    
                    // Test connection
                    let mut conn = client.get_async_connection().await
                        .map_err(|e| EventStoreError::Backend(format!("Redis connection test failed: {}", e)))?;
                    
                    let _: String = redis::cmd("PING")
                        .query_async(&mut conn)
                        .await
                        .map_err(|e| EventStoreError::Backend(format!("Redis ping failed: {}", e)))?;
                    
                    tracing::info!("📢 Created Redis event bus");
                    Ok(Box::new(RedisEventBus { client }))
                }
                "memory" | "test" => {
                    tracing::info!("📢 Created in-memory event bus");
                    Ok(Box::new(InMemoryEventBus {
                        subscribers: Arc::new(RwLock::new(std::collections::HashMap::new())),
                        published_events: Arc::new(RwLock::new(Vec::new())),
                    }))
                }
                _ => {
                    tracing::warn!("🚨 Unsupported event bus type '{}', falling back to in-memory", bus_type);
                    Ok(Box::new(InMemoryEventBus {
                        subscribers: Arc::new(RwLock::new(std::collections::HashMap::new())),
                        published_events: Arc::new(RwLock::new(Vec::new())),
                    }))
                }
            }
        }
        
        // Set up event sourcing context
        let _event_sourcing_context = EventSourcingContext {
            event_store: event_store.clone(),
            aggregate_manager,
            aggregate_id: aggregate_id.clone(),
        };
        
        /// Event sourcing execution context
        struct EventSourcingContext {
            event_store: Arc<EventStore>,
            aggregate_manager: AggregateManager,
            aggregate_id: String,
        }
        
        impl EventSourcingContext {
            /// Create new domain event
            fn create_event(
                &self,
                event_type: &str,
                event_data: serde_json::Value
            ) -> DomainEvent {
                DomainEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    aggregate_id: self.aggregate_id.clone(),
                    aggregate_type: aggregate_type.to_string(),
                    event_type: event_type.to_string(),
                    event_version: EventVersion::INITIAL, // Will be set during append
                    event_data,
                    metadata: EventMetadata {
                        user_id: None,
                        source: "rabbitmesh".to_string(),
                        ip_address: None,
                        user_agent: None,
                        command_id: None,
                        custom_fields: std::collections::HashMap::new(),
                    },
                    timestamp: std::time::SystemTime::now(),
                    correlation_id: None,
                    causation_id: None,
                }
            }
        }
    }
}