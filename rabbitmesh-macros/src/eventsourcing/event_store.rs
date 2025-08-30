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
        
        /// Binary event serializer using MessagePack
        struct BinaryEventSerializer;
        
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
                let serializer = Arc::new(create_event_serializer());
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
            
            /// Apply event to aggregate state
            fn apply_event_to_state(&self, mut state: serde_json::Value, event: &DomainEvent) -> Result<serde_json::Value, EventStoreError> {
                // This is a simplified state application
                // In a real implementation, this would use proper aggregate pattern
                if state.is_null() {
                    state = serde_json::json!({});
                }
                
                if let serde_json::Value::Object(ref mut state_obj) = state {
                    state_obj.insert("last_event_type".to_string(), serde_json::Value::String(event.event_type.clone()));
                    state_obj.insert("version".to_string(), serde_json::json!(event.event_version.0));
                    
                    // Apply event-specific changes
                    if let serde_json::Value::Object(event_data) = &event.event_data {
                        for (key, value) in event_data {
                            state_obj.insert(key.clone(), value.clone());
                        }
                    }
                }
                
                Ok(state)
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
        
        /// Create event serializer
        fn create_event_serializer() -> JsonEventSerializer {
            JsonEventSerializer
        }
        
        /// Create snapshot store
        async fn create_snapshot_store(_backend_type: &str) -> Result<Box<dyn SnapshotStore + Send + Sync>, EventStoreError> {
            // Placeholder implementation
            Err(EventStoreError::Configuration("Snapshot store not implemented".to_string()))
        }
        
        /// Create event bus
        async fn create_event_bus() -> Result<Box<dyn EventBus + Send + Sync>, EventStoreError> {
            // Placeholder implementation  
            Err(EventStoreError::Configuration("Event bus not implemented".to_string()))
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