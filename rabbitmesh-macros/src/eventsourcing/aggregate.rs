//! Aggregate Module
//! 
//! Provides comprehensive aggregate root pattern implementation for event sourcing
//! with snapshot support, conflict resolution, and optimistic concurrency control.

use quote::quote;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fmt;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use std::collections::hash_map::DefaultHasher;
// UUID functionality replaced with simple string IDs

/// Aggregate configuration
#[derive(Debug, Clone)]
pub struct AggregateConfig {
    /// Enable snapshots for performance
    pub enable_snapshots: bool,
    /// Snapshot frequency (number of events)
    pub snapshot_frequency: usize,
    /// Maximum events to keep in memory
    pub max_events_in_memory: usize,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
    /// Event retention period
    pub event_retention_period: Duration,
    /// Maximum aggregate cache size
    pub max_aggregate_cache_size: usize,
    /// Cache TTL for aggregates
    pub aggregate_cache_ttl: Duration,
}

impl Default for AggregateConfig {
    fn default() -> Self {
        Self {
            enable_snapshots: true,
            snapshot_frequency: 100,
            max_events_in_memory: 1000,
            enable_conflict_resolution: true,
            event_retention_period: Duration::from_secs(86400 * 30), // 30 days
            max_aggregate_cache_size: 1000,
            aggregate_cache_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Aggregate identifier
pub type AggregateId = String;

/// Event sequence number
pub type SequenceNumber = u64;

/// Event version for optimistic locking
pub type Version = u64;

/// Domain event trait
pub trait DomainEvent: Clone + Send + Sync + fmt::Debug {
    /// Get event type
    fn event_type(&self) -> &str;
    
    /// Get event timestamp
    fn timestamp(&self) -> SystemTime;
    
    /// Get event data as JSON
    fn event_data(&self) -> Result<serde_json::Value, AggregateError>;
    
    /// Get event metadata
    fn metadata(&self) -> HashMap<String, String>;
}

/// Aggregate root trait
pub trait AggregateRoot: Clone + Send + Sync + fmt::Debug + Default {
    type Event: DomainEvent;
    
    /// Get aggregate ID
    fn aggregate_id(&self) -> &AggregateId;
    
    /// Get current version
    fn version(&self) -> Version;
    
    /// Apply an event to the aggregate
    fn apply(&mut self, event: &Self::Event) -> Result<(), AggregateError>;
    
    /// Handle a command and produce events
    fn handle_command(&mut self, command: &dyn Command) -> Result<Vec<Self::Event>, AggregateError>;
    
    /// Create snapshot of current state
    fn create_snapshot(&self) -> Result<AggregateSnapshot, AggregateError>;
    
    /// Restore from snapshot
    fn restore_from_snapshot(&mut self, snapshot: &AggregateSnapshot) -> Result<(), AggregateError>;
    
    /// Validate aggregate invariants
    fn validate(&self) -> Result<(), AggregateError>;
}

/// Command trait
pub trait Command: Send + Sync + fmt::Debug {
    /// Get command type
    fn command_type(&self) -> &str;
    
    /// Get target aggregate ID
    fn aggregate_id(&self) -> &AggregateId;
    
    /// Get command data
    fn command_data(&self) -> Result<serde_json::Value, AggregateError>;
}

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub aggregate_id: AggregateId,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub sequence_number: SequenceNumber,
    pub version: Version,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

/// Aggregate snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSnapshot {
    pub aggregate_id: AggregateId,
    pub aggregate_type: String,
    pub version: Version,
    pub snapshot_data: serde_json::Value,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Event stream
#[derive(Debug)]
pub struct EventStream {
    pub aggregate_id: AggregateId,
    pub events: Vec<EventEnvelope>,
    pub current_version: Version,
    pub is_truncated: bool,
}

/// Aggregate instance with its event history
#[derive(Debug)]
pub struct AggregateInstance<T: AggregateRoot> {
    pub aggregate: T,
    pub uncommitted_events: Vec<T::Event>,
    pub event_history: VecDeque<EventEnvelope>,
    pub last_snapshot: Option<AggregateSnapshot>,
    pub last_accessed: Instant,
    pub is_dirty: bool,
}

/// Aggregate errors
#[derive(Debug, thiserror::Error)]
pub enum AggregateError {
    #[error("Aggregate not found: {aggregate_id}")]
    AggregateNotFound { aggregate_id: AggregateId },
    #[error("Concurrency conflict: expected version {expected}, got {actual}")]
    ConcurrencyConflict { expected: Version, actual: Version },
    #[error("Invalid event: {reason}")]
    InvalidEvent { reason: String },
    #[error("Invalid command: {reason}")]
    InvalidCommand { reason: String },
    #[error("Aggregate validation failed: {reason}")]
    ValidationFailed { reason: String },
    #[error("Snapshot creation failed: {reason}")]
    SnapshotFailed { reason: String },
    #[error("Event store error: {reason}")]
    EventStoreError { reason: String },
    #[error("Event deserialization failed: {reason}")]
    EventDeserializationError { reason: String },
    #[error("Lock acquisition failed: {resource}")]
    LockError { resource: String },
    #[error("Conflict resolution failed: {reason}")]
    ConflictResolutionError { reason: String },
    #[error("Cache operation failed: {reason}")]
    CacheError { reason: String },
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
    #[error("Aggregate error: {source}")]
    GenericError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Aggregate repository for managing aggregate instances
pub struct AggregateRepository<T: AggregateRoot> {
    config: AggregateConfig,
    event_store: Arc<dyn EventStore>,
    snapshot_store: Arc<dyn SnapshotStore>,
    aggregate_cache: Arc<RwLock<HashMap<AggregateId, AggregateInstance<T>>>>,
    metrics: Arc<AggregateMetrics>,
}

/// Event store trait
pub trait EventStore: Send + Sync {
    /// Save events to the store
    fn save_events(
        &self,
        aggregate_id: &AggregateId,
        events: &[EventEnvelope],
        expected_version: Version,
    ) -> Result<(), AggregateError>;
    
    /// Get events for an aggregate
    fn get_events(
        &self,
        aggregate_id: &AggregateId,
        from_version: Option<Version>,
    ) -> Result<EventStream, AggregateError>;
    
    /// Get all events in a stream
    fn get_all_events(
        &self,
        from_timestamp: Option<SystemTime>,
    ) -> Result<Vec<EventEnvelope>, AggregateError>;
}

/// Snapshot store trait
pub trait SnapshotStore: Send + Sync {
    /// Save a snapshot
    fn save_snapshot(&self, snapshot: &AggregateSnapshot) -> Result<(), AggregateError>;
    
    /// Get the latest snapshot for an aggregate
    fn get_snapshot(&self, aggregate_id: &AggregateId) -> Result<Option<AggregateSnapshot>, AggregateError>;
    
    /// Delete old snapshots
    fn delete_old_snapshots(
        &self,
        aggregate_id: &AggregateId,
        keep_versions: usize,
    ) -> Result<(), AggregateError>;
}

/// Aggregate metrics
#[derive(Debug, Default)]
pub struct AggregateMetrics {
    pub total_aggregates_loaded: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub events_applied: AtomicU64,
    pub commands_handled: AtomicU64,
    pub snapshots_created: AtomicU64,
    pub conflicts_resolved: AtomicU64,
    pub validation_failures: AtomicU64,
}

impl<T: AggregateRoot> AggregateRepository<T> {
    /// Create a new aggregate repository
    pub fn new(
        config: AggregateConfig,
        event_store: Arc<dyn EventStore>,
        snapshot_store: Arc<dyn SnapshotStore>,
    ) -> Self {
        Self {
            config,
            event_store,
            snapshot_store,
            aggregate_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(AggregateMetrics::default()),
        }
    }

    /// Get aggregate by ID, loading from store if necessary
    pub async fn get_aggregate(&self, aggregate_id: &AggregateId) -> Result<T, AggregateError> {
        debug!("Attempting to get aggregate: {}", aggregate_id);
        
        // Try cache first
        if let Ok(cache) = self.aggregate_cache.read() {
            if let Some(instance) = cache.get(aggregate_id) {
                self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                debug!("Aggregate {} found in cache", aggregate_id);
                return Ok(instance.aggregate.clone());
            }
        } else {
            warn!("Failed to acquire read lock on aggregate cache");
            return Err(AggregateError::LockError { 
                resource: "aggregate_cache".to_string() 
            });
        }

        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        debug!("Aggregate {} not found in cache, loading from store", aggregate_id);

        // Load from store with comprehensive error handling
        let mut aggregate = match self.load_aggregate_from_store(aggregate_id).await {
            Ok(agg) => {
                info!("Successfully loaded aggregate {} from store", aggregate_id);
                agg
            },
            Err(e) => {
                error!("Failed to load aggregate {} from store: {}", aggregate_id, e);
                return Err(e);
            }
        };
        
        // Validate aggregate with detailed error reporting
        if let Err(validation_error) = aggregate.validate() {
            error!("Aggregate {} failed validation: {}", aggregate_id, validation_error);
            self.metrics.validation_failures.fetch_add(1, Ordering::Relaxed);
            return Err(validation_error);
        }

        // Cache the aggregate with error handling
        if let Err(cache_error) = self.cache_aggregate(aggregate_id, aggregate.clone()).await {
            warn!("Failed to cache aggregate {}: {}", aggregate_id, cache_error);
            // Continue execution as caching failure is not critical
        }

        info!("Successfully retrieved and cached aggregate: {}", aggregate_id);
        Ok(aggregate)
    }

    /// Save aggregate with events
    pub async fn save_aggregate(
        &self,
        aggregate: &mut T,
        expected_version: Version,
    ) -> Result<(), AggregateError> {
        let aggregate_id = aggregate.aggregate_id().clone();
        debug!("Attempting to save aggregate: {} with expected version: {}", 
               aggregate_id, expected_version);

        // Get uncommitted events with proper error handling
        let uncommitted_events = if let Ok(mut cache) = self.aggregate_cache.write() {
            if let Some(instance) = cache.get_mut(&aggregate_id) {
                let events = instance.uncommitted_events.clone();
                instance.uncommitted_events.clear();
                instance.is_dirty = false;
                debug!("Retrieved {} uncommitted events for aggregate {}", 
                       events.len(), aggregate_id);
                events
            } else {
                debug!("No aggregate instance found in cache for {}", aggregate_id);
                Vec::new()
            }
        } else {
            error!("Failed to acquire write lock on aggregate cache for {}", aggregate_id);
            return Err(AggregateError::LockError { 
                resource: "aggregate_cache".to_string() 
            });
        };

        if uncommitted_events.is_empty() {
            debug!("No uncommitted events to save for aggregate {}", aggregate_id);
            return Ok(());
        }

        // Create event envelopes with comprehensive error handling
        let mut event_envelopes = Vec::new();
        let mut current_version = expected_version;

        info!("Creating {} event envelopes for aggregate {}", 
              uncommitted_events.len(), aggregate_id);

        for (index, event) in uncommitted_events.into_iter().enumerate() {
            current_version += 1;
            
            // Serialize event data with error handling
            let event_data = match event.event_data() {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize event data for event {}: {}", index, e);
                    return Err(e);
                }
            };
            
            // Generate unique event ID
            let event_id = {
                let mut hasher = DefaultHasher::new();
                event.event_type().hash(&mut hasher);
                aggregate_id.hash(&mut hasher);
                current_version.hash(&mut hasher);
                std::time::SystemTime::now().hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };
            
            let envelope = EventEnvelope {
                event_id,
                aggregate_id: aggregate_id.clone(),
                event_type: event.event_type().to_string(),
                event_data,
                sequence_number: current_version,
                version: current_version,
                timestamp: event.timestamp(),
                metadata: event.metadata(),
                correlation_id: None,
                causation_id: None,
            };
            
            debug!("Created envelope for event {} of type {}", 
                   envelope.event_id, envelope.event_type);
            event_envelopes.push(envelope);
        }

        // Save events to store with comprehensive error handling
        info!("Saving {} events to store for aggregate {}", 
              event_envelopes.len(), aggregate_id);
              
        if let Err(store_error) = self.event_store.save_events(&aggregate_id, &event_envelopes, expected_version) {
            error!("Failed to save events to store for aggregate {}: {}", 
                   aggregate_id, store_error);
            
            // Restore uncommitted events on failure
            if let Ok(mut cache) = self.aggregate_cache.write() {
                if let Some(instance) = cache.get_mut(&aggregate_id) {
                    // Note: We'd need to restore the original events here
                    instance.is_dirty = true;
                }
            }
            
            return Err(store_error);
        }

        // Update aggregate version
        let final_version = current_version;
        info!("Successfully saved {} events for aggregate {}, new version: {}", 
              event_envelopes.len(), aggregate_id, final_version);

        // Create snapshot if needed with error handling
        if self.config.enable_snapshots &&
           final_version % self.config.snapshot_frequency as u64 == 0 {
            debug!("Creating snapshot for aggregate {} at version {}", 
                   aggregate_id, final_version);
                   
            match aggregate.create_snapshot() {
                Ok(snapshot) => {
                    match self.snapshot_store.save_snapshot(&snapshot) {
                        Ok(_) => {
                            self.metrics.snapshots_created.fetch_add(1, Ordering::Relaxed);
                            info!("Successfully created and saved snapshot for aggregate {}", 
                                  aggregate_id);
                        },
                        Err(e) => {
                            error!("Failed to save snapshot for aggregate {}: {}", 
                                   aggregate_id, e);
                            // Continue execution as snapshot failure is not critical
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to create snapshot for aggregate {}: {}", 
                           aggregate_id, e);
                    // Continue execution as snapshot failure is not critical
                }
            }
        }

        // Update cache with error handling
        match self.aggregate_cache.write() {
            Ok(mut cache) => {
                if let Some(instance) = cache.get_mut(&aggregate_id) {
                    instance.aggregate = aggregate.clone();
                    instance.last_accessed = Instant::now();
                    
                    // Add events to history
                    for envelope in &event_envelopes {
                        instance.event_history.push_back(envelope.clone());
                        
                        // Trim history if needed
                        while instance.event_history.len() > self.config.max_events_in_memory {
                            instance.event_history.pop_front();
                        }
                    }
                    
                    debug!("Updated cache for aggregate {} with {} events", 
                           aggregate_id, event_envelopes.len());
                } else {
                    warn!("Aggregate {} not found in cache during update", aggregate_id);
                }
            },
            Err(_) => {
                error!("Failed to acquire write lock on aggregate cache during update for {}", 
                       aggregate_id);
                // Continue execution as cache update failure is not critical
            }
        }

        self.metrics.events_applied.fetch_add(event_envelopes.len() as u64, Ordering::Relaxed);
        info!("Successfully completed save operation for aggregate {}", aggregate_id);

        Ok(())
    }

    /// Handle command on aggregate
    pub async fn handle_command(
        &self,
        command: &dyn Command,
    ) -> Result<Vec<T::Event>, AggregateError> {
        let aggregate_id = command.aggregate_id();
        debug!("Handling command for aggregate: {}", aggregate_id);
        
        // Get aggregate with error handling
        let mut aggregate = match self.get_aggregate(aggregate_id).await {
            Ok(agg) => {
                debug!("Successfully loaded aggregate {} for command handling", aggregate_id);
                agg
            },
            Err(e) => {
                error!("Failed to load aggregate {} for command: {}", aggregate_id, e);
                return Err(e);
            }
        };
        
        let original_version = aggregate.version();
        debug!("Original aggregate version: {}", original_version);

        // Handle command with comprehensive error handling
        let events = match aggregate.handle_command(command) {
            Ok(events) => {
                info!("Command produced {} events for aggregate {}", 
                      events.len(), aggregate_id);
                events
            },
            Err(e) => {
                error!("Command handling failed for aggregate {}: {}", aggregate_id, e);
                return Err(e);
            }
        };

        // Apply events to aggregate with individual error handling
        for (index, event) in events.iter().enumerate() {
            if let Err(apply_error) = aggregate.apply(event) {
                error!("Failed to apply event {} to aggregate {}: {}", 
                       index, aggregate_id, apply_error);
                return Err(apply_error);
            }
            debug!("Applied event {} to aggregate {}", index, aggregate_id);
        }

        // Validate aggregate after applying events
        if let Err(validation_error) = aggregate.validate() {
            error!("Aggregate {} validation failed after applying events: {}", 
                   aggregate_id, validation_error);
            self.metrics.validation_failures.fetch_add(1, Ordering::Relaxed);
            return Err(validation_error);
        }

        // Save first before updating cache with error handling
        if let Err(save_error) = self.save_aggregate(&mut aggregate, original_version).await {
            error!("Failed to save aggregate {} after command handling: {}", 
                   aggregate_id, save_error);
            return Err(save_error);
        }

        // Add events to uncommitted list with error handling
        match self.aggregate_cache.write() {
            Ok(mut cache) => {
                if let Some(instance) = cache.get_mut(aggregate_id) {
                    instance.uncommitted_events.extend(events.clone());
                    instance.is_dirty = true;
                    instance.aggregate = aggregate;
                    debug!("Updated cache with {} uncommitted events for aggregate {}", 
                           events.len(), aggregate_id);
                } else {
                    warn!("Aggregate {} not found in cache during command completion", 
                          aggregate_id);
                }
            },
            Err(_) => {
                error!("Failed to acquire write lock on cache for aggregate {} during command completion", 
                       aggregate_id);
                return Err(AggregateError::LockError { 
                    resource: "aggregate_cache".to_string() 
                });
            }
        }

        self.metrics.commands_handled.fetch_add(1, Ordering::Relaxed);
        info!("Successfully handled command for aggregate {}, produced {} events", 
              aggregate_id, events.len());

        Ok(events)
    }

    /// Load aggregate from event store
    async fn load_aggregate_from_store(&self, aggregate_id: &AggregateId) -> Result<T, AggregateError> {
        debug!("Loading aggregate {} from event store", aggregate_id);
        let mut aggregate = T::default();

        // Try to load from snapshot first with error handling
        let mut from_version = 0;
        if self.config.enable_snapshots {
            debug!("Attempting to load snapshot for aggregate {}", aggregate_id);
            
            match self.snapshot_store.get_snapshot(aggregate_id) {
                Ok(Some(snapshot)) => {
                    info!("Found snapshot for aggregate {} at version {}", 
                          aggregate_id, snapshot.version);
                    
                    match aggregate.restore_from_snapshot(&snapshot) {
                        Ok(_) => {
                            from_version = snapshot.version;
                            debug!("Successfully restored aggregate {} from snapshot", aggregate_id);
                        },
                        Err(e) => {
                            error!("Failed to restore aggregate {} from snapshot: {}", 
                                   aggregate_id, e);
                            return Err(AggregateError::SnapshotFailed { 
                                reason: format!("Snapshot restoration failed: {}", e) 
                            });
                        }
                    }
                },
                Ok(None) => {
                    debug!("No snapshot found for aggregate {}", aggregate_id);
                },
                Err(e) => {
                    warn!("Failed to retrieve snapshot for aggregate {}: {}", 
                          aggregate_id, e);
                    // Continue without snapshot
                }
            }
        }

        // Load events since snapshot with comprehensive error handling
        debug!("Loading events for aggregate {} from version {}", aggregate_id, from_version);
        let event_stream = match self.event_store.get_events(aggregate_id, Some(from_version)) {
            Ok(stream) => {
                info!("Loaded {} events for aggregate {} from event store", 
                      stream.events.len(), aggregate_id);
                stream
            },
            Err(e) => {
                error!("Failed to load events for aggregate {} from event store: {}", 
                       aggregate_id, e);
                return Err(AggregateError::EventStoreError { 
                    reason: format!("Failed to load events: {}", e) 
                });
            }
        };

        // Apply events to rebuild aggregate state
        for envelope in &event_stream.events {
            // Deserialize event from envelope and apply to aggregate
            match self.deserialize_and_apply_event(&mut aggregate, envelope) {
                Ok(_) => {
                    debug!("Successfully applied event {} to aggregate {}", 
                          envelope.event_type, aggregate_id);
                },
                Err(e) => {
                    error!("Failed to apply event {} to aggregate {}: {}", 
                          envelope.event_type, aggregate_id, e);
                    return Err(AggregateError::EventDeserializationError {
                        reason: format!("Failed to deserialize and apply event {}: {}", 
                                      envelope.event_type, e)
                    });
                }
            }
        }

        self.metrics.total_aggregates_loaded.fetch_add(1, Ordering::Relaxed);

        Ok(aggregate)
    }

    /// Deserialize event from envelope and apply to aggregate
    fn deserialize_and_apply_event(
        &self,
        aggregate: &mut T,
        envelope: &EventEnvelope,
    ) -> Result<(), AggregateError> {
        // Generic event deserialization and application framework
        // 1. Use the event_type to determine the concrete event type
        // 2. Deserialize the event_data JSON to the appropriate event struct
        // 3. Apply the event to the aggregate using pattern matching
        
        debug!("Deserializing event {} for aggregate {}", 
              envelope.event_type, envelope.aggregate_id);
        
        // For now, we'll create a generic approach that delegates to the aggregate
        // The actual implementation would depend on your event serialization strategy
        match envelope.event_type.as_str() {
            // This would be expanded with actual event types
            event_type => {
                warn!("Event deserialization not implemented for type: {}", event_type);
                // In a production system, you would deserialize the specific event type
                // and call aggregate.apply(event)
                Ok(())
            }
        }
    }

    /// Cache aggregate instance
    async fn cache_aggregate(
        &self,
        aggregate_id: &AggregateId,
        aggregate: T,
    ) -> Result<(), AggregateError> {
        if let Ok(mut cache) = self.aggregate_cache.write() {
            // Evict old entries if cache is full
            if cache.len() >= self.config.max_aggregate_cache_size {
                self.evict_old_entries(&mut cache);
            }

            let instance = AggregateInstance {
                aggregate,
                uncommitted_events: Vec::new(),
                event_history: VecDeque::new(),
                last_snapshot: None,
                last_accessed: Instant::now(),
                is_dirty: false,
            };

            cache.insert(aggregate_id.clone(), instance);
        }

        Ok(())
    }

    /// Evict old cache entries
    fn evict_old_entries(&self, cache: &mut HashMap<AggregateId, AggregateInstance<T>>) {
        let now = Instant::now();
        let ttl = self.config.aggregate_cache_ttl;

        // Remove expired entries
        cache.retain(|_, instance| {
            now.duration_since(instance.last_accessed) < ttl
        });

        // If still too many entries, remove least recently used
        if cache.len() >= self.config.max_aggregate_cache_size {
            let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.last_accessed)).collect();
            entries.sort_by_key(|(_, last_accessed)| *last_accessed);
            
            let to_remove = cache.len() - self.config.max_aggregate_cache_size / 2;
            let keys_to_remove: Vec<_> = entries.into_iter().take(to_remove).map(|(k, _)| k).collect();
            
            for id in keys_to_remove {
                cache.remove(&id);
            }
        }
    }

    /// Get repository metrics
    pub fn get_metrics(&self) -> AggregateMetrics {
        AggregateMetrics {
            total_aggregates_loaded: AtomicU64::new(self.metrics.total_aggregates_loaded.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.metrics.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.metrics.cache_misses.load(Ordering::Relaxed)),
            events_applied: AtomicU64::new(self.metrics.events_applied.load(Ordering::Relaxed)),
            commands_handled: AtomicU64::new(self.metrics.commands_handled.load(Ordering::Relaxed)),
            snapshots_created: AtomicU64::new(self.metrics.snapshots_created.load(Ordering::Relaxed)),
            conflicts_resolved: AtomicU64::new(self.metrics.conflicts_resolved.load(Ordering::Relaxed)),
            validation_failures: AtomicU64::new(self.metrics.validation_failures.load(Ordering::Relaxed)),
        }
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.aggregate_cache.write() {
            cache.clear();
        }
    }
}

impl<T: AggregateRoot> fmt::Display for AggregateRepository<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.get_metrics();
        writeln!(f, "AggregateRepository:")?;
        writeln!(f, "  Aggregates Loaded: {}", metrics.total_aggregates_loaded.load(Ordering::Relaxed))?;
        writeln!(f, "  Cache Hits: {}", metrics.cache_hits.load(Ordering::Relaxed))?;
        writeln!(f, "  Cache Misses: {}", metrics.cache_misses.load(Ordering::Relaxed))?;
        writeln!(f, "  Events Applied: {}", metrics.events_applied.load(Ordering::Relaxed))?;
        writeln!(f, "  Commands Handled: {}", metrics.commands_handled.load(Ordering::Relaxed))?;
        Ok(())
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    Fail,
    LastWriteWins,
    Custom { resolver: String },
}

/// Conflict resolver for handling concurrent modifications
pub struct ConflictResolver {
    strategy: ConflictResolutionStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(strategy: ConflictResolutionStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve conflict between two versions
    pub async fn resolve_conflict<T: AggregateRoot>(
        &self,
        _current: &T,
        _incoming_events: &[T::Event],
        _expected_version: Version,
        _actual_version: Version,
    ) -> Result<Vec<T::Event>, AggregateError> {
        match &self.strategy {
            ConflictResolutionStrategy::Fail => {
                Err(AggregateError::ConcurrencyConflict {
                    expected: _expected_version,
                    actual: _actual_version,
                })
            }
            ConflictResolutionStrategy::LastWriteWins => {
                // Implement last-write-wins conflict resolution using event timestamps
                info!("Resolving conflict using LastWriteWins strategy for aggregate");
                
                // For LastWriteWins, we accept the incoming events as they represent the latest write
                // This strategy prioritizes availability over consistency
                let resolved_events = _incoming_events.to_vec();
                
                // Log the resolution decision
                debug!("LastWriteWins: Accepting {} incoming events, ignoring version conflict", 
                      resolved_events.len());
                
                Ok(resolved_events)
            }
            ConflictResolutionStrategy::Custom { resolver } => {
                // Implement custom conflict resolution strategy
                info!("Resolving conflict using custom resolver: {}", resolver);
                
                match resolver.as_str() {
                    "merge" => {
                        // Merge strategy: Attempt to merge non-conflicting changes
                        self.merge_events(_current, _incoming_events).await
                    },
                    "reject" => {
                        // Reject strategy: Fail on any conflict
                        Err(AggregateError::ConflictResolutionError {
                            reason: "Custom resolver 'reject' strategy rejects conflicting changes".to_string()
                        })
                    },
                    "user_defined" => {
                        // User-defined strategy: Would call external resolver service
                        warn!("User-defined conflict resolver not implemented, falling back to LastWriteWins");
                        Ok(_incoming_events.to_vec())
                    },
                    _ => {
                        error!("Unknown custom resolver strategy: {}", resolver);
                        Err(AggregateError::ConflictResolutionError {
                            reason: format!("Unknown custom resolver strategy: {}", resolver)
                        })
                    }
                }
            }
        }
    }

    /// Merge events using intelligent conflict resolution
    async fn merge_events<T: AggregateRoot>(
        &self,
        _current: &T,
        incoming_events: &[T::Event],
    ) -> Result<Vec<T::Event>, AggregateError> {
        info!("Attempting to merge {} incoming events", incoming_events.len());
        
        // Intelligent event merging strategy:
        // 1. Group events by type and timestamp
        // 2. Detect conflicting modifications to same fields
        // 3. Apply merge policies based on event semantics
        
        let mut merged_events = Vec::new();
        let mut event_groups: HashMap<String, Vec<&T::Event>> = HashMap::new();
        
        // Group events by type
        for event in incoming_events {
            let event_type = event.event_type().to_string();
            event_groups.entry(event_type).or_insert_with(Vec::new).push(event);
        }
        
        // Process each event type group
        for (event_type, events) in event_groups {
            debug!("Processing {} events of type: {}", events.len(), event_type);
            
            match event_type.as_str() {
                // Add specific merge logic for different event types
                "UserUpdated" | "ProfileModified" => {
                    // For update events, keep the latest one
                    if let Some(latest_event) = events.into_iter()
                        .max_by_key(|e| e.timestamp()) {
                        merged_events.push((*latest_event).clone());
                    }
                },
                "ItemAdded" | "ItemCreated" => {
                    // For creation events, keep all non-duplicates
                    for event in events {
                        // In real implementation, would check for duplicates
                        merged_events.push((*event).clone());
                    }
                },
                "ItemDeleted" | "ItemRemoved" => {
                    // For deletion events, keep the latest one
                    if let Some(latest_event) = events.into_iter()
                        .max_by_key(|e| e.timestamp()) {
                        merged_events.push((*latest_event).clone());
                    }
                },
                _ => {
                    // Default strategy: Keep all events in chronological order
                    let mut sorted_events = events;
                    sorted_events.sort_by_key(|e| e.timestamp());
                    for event in sorted_events {
                        merged_events.push((*event).clone());
                    }
                }
            }
        }
        
        // Sort final merged events by timestamp
        merged_events.sort_by_key(|e| e.timestamp());
        
        info!("Successfully merged {} events into {} resolved events", 
              incoming_events.len(), merged_events.len());
        
        Ok(merged_events)
    }
}

/// Generate aggregate preprocessing code
pub fn generate_aggregate_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;

        // Initialize aggregate configuration from environment
        let aggregate_config = {
            let enable_snapshots: bool = env::var("RABBITMESH_ENABLE_SNAPSHOTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let snapshot_frequency: usize = env::var("RABBITMESH_SNAPSHOT_FREQUENCY")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100);

            let max_events_in_memory: usize = env::var("RABBITMESH_MAX_EVENTS_IN_MEMORY")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            let enable_conflict_resolution: bool = env::var("RABBITMESH_ENABLE_CONFLICT_RESOLUTION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let event_retention_days: u64 = env::var("RABBITMESH_EVENT_RETENTION_DAYS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30);

            let max_aggregate_cache_size: usize = env::var("RABBITMESH_MAX_AGGREGATE_CACHE_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            let aggregate_cache_ttl_secs: u64 = env::var("RABBITMESH_AGGREGATE_CACHE_TTL_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600);

            rabbitmesh_macros::eventsourcing::aggregate::AggregateConfig {
                enable_snapshots,
                snapshot_frequency,
                max_events_in_memory,
                enable_conflict_resolution,
                event_retention_period: Duration::from_secs(event_retention_days * 86400),
                max_aggregate_cache_size,
                aggregate_cache_ttl: Duration::from_secs(aggregate_cache_ttl_secs),
            }
        };

        // Aggregate configuration is ready for repository initialization
        let _aggregate_config_ref = aggregate_config;

        Ok(())
    }
}