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
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
    #[error("Aggregate error: {source}")]
    AggregateError {
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
        // Try cache first
        if let Ok(cache) = self.aggregate_cache.read() {
            if let Some(instance) = cache.get(aggregate_id) {
                self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(instance.aggregate.clone());
            }
        }

        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Load from store
        let mut aggregate = self.load_aggregate_from_store(aggregate_id).await?;
        
        // Validate aggregate
        aggregate.validate()?;

        // Cache the aggregate
        self.cache_aggregate(aggregate_id, aggregate.clone()).await?;

        Ok(aggregate)
    }

    /// Save aggregate with events
    pub async fn save_aggregate(
        &self,
        aggregate: &mut T,
        expected_version: Version,
    ) -> Result<(), AggregateError> {
        let aggregate_id = aggregate.aggregate_id().clone();

        // Get uncommitted events
        let uncommitted_events = if let Ok(mut cache) = self.aggregate_cache.write() {
            if let Some(instance) = cache.get_mut(&aggregate_id) {
                let events = instance.uncommitted_events.clone();
                instance.uncommitted_events.clear();
                instance.is_dirty = false;
                events
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if uncommitted_events.is_empty() {
            return Ok(());
        }

        // Create event envelopes
        let mut event_envelopes = Vec::new();
        let mut current_version = expected_version;

        for event in uncommitted_events {
            current_version += 1;
            
            let envelope = EventEnvelope {
                event_id: {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    event.event_type().hash(&mut hasher);
                    std::time::SystemTime::now().hash(&mut hasher);
                    format!("{:x}", hasher.finish())
                },
                aggregate_id: aggregate_id.clone(),
                event_type: event.event_type().to_string(),
                event_data: event.event_data()?,
                sequence_number: current_version,
                version: current_version,
                timestamp: event.timestamp(),
                metadata: event.metadata(),
                correlation_id: None,
                causation_id: None,
            };
            
            event_envelopes.push(envelope);
        }

        // Save events to store
        self.event_store.save_events(&aggregate_id, &event_envelopes, expected_version)?;

        // Update aggregate version
        let final_version = current_version;

        // Create snapshot if needed
        if self.config.enable_snapshots &&
           final_version % self.config.snapshot_frequency as u64 == 0 {
            let snapshot = aggregate.create_snapshot()?;
            self.snapshot_store.save_snapshot(&snapshot)?;
            self.metrics.snapshots_created.fetch_add(1, Ordering::Relaxed);
        }

        // Update cache
        if let Ok(mut cache) = self.aggregate_cache.write() {
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
            }
        }

        self.metrics.events_applied.fetch_add(event_envelopes.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Handle command on aggregate
    pub async fn handle_command(
        &self,
        command: &dyn Command,
    ) -> Result<Vec<T::Event>, AggregateError> {
        let aggregate_id = command.aggregate_id();
        let mut aggregate = self.get_aggregate(aggregate_id).await?;
        let original_version = aggregate.version();

        // Handle command
        let events = aggregate.handle_command(command)?;

        // Apply events to aggregate
        for event in &events {
            aggregate.apply(event)?;
        }

        // Validate aggregate after applying events
        aggregate.validate()?;

        // Save first before updating cache
        self.save_aggregate(&mut aggregate, original_version).await?;

        // Add events to uncommitted list
        if let Ok(mut cache) = self.aggregate_cache.write() {
            if let Some(instance) = cache.get_mut(aggregate_id) {
                instance.uncommitted_events.extend(events.clone());
                instance.is_dirty = true;
                instance.aggregate = aggregate;
            }
        }

        self.metrics.commands_handled.fetch_add(1, Ordering::Relaxed);

        Ok(events)
    }

    /// Load aggregate from event store
    async fn load_aggregate_from_store(&self, aggregate_id: &AggregateId) -> Result<T, AggregateError> {
        let mut aggregate = T::default();

        // Try to load from snapshot first
        let mut from_version = 0;
        if self.config.enable_snapshots {
            if let Ok(Some(snapshot)) = self.snapshot_store.get_snapshot(aggregate_id) {
                aggregate.restore_from_snapshot(&snapshot)?;
                from_version = snapshot.version;
            }
        }

        // Load events since snapshot
        let event_stream = self.event_store.get_events(aggregate_id, Some(from_version))?;

        // Apply events to rebuild aggregate state
        for envelope in &event_stream.events {
            // In a real implementation, you'd deserialize the event from envelope.event_data
            // For now, we'll assume this is handled by the specific implementation
        }

        self.metrics.total_aggregates_loaded.fetch_add(1, Ordering::Relaxed);

        Ok(aggregate)
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
                // In a real implementation, this would merge or override based on timestamps
                Ok(_incoming_events.to_vec())
            }
            ConflictResolutionStrategy::Custom { resolver: _ } => {
                // In a real implementation, this would call a custom resolver
                Ok(_incoming_events.to_vec())
            }
        }
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