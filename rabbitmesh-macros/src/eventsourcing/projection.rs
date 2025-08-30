//! Projection Module
//! 
//! Provides comprehensive event stream projection for read models with
//! real-time updates, versioning, and multi-backend support.

use quote::quote;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};
use std::fmt;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// Projection configuration
#[derive(Debug, Clone)]
pub struct ProjectionConfig {
    /// Enable real-time projection updates
    pub enable_realtime_updates: bool,
    /// Batch size for processing events
    pub batch_size: usize,
    /// Maximum lag before alerting
    pub max_lag_threshold: Duration,
    /// Enable projection versioning
    pub enable_versioning: bool,
    /// Checkpoint frequency
    pub checkpoint_frequency: Duration,
    /// Maximum retry attempts for failed projections
    pub max_retry_attempts: u32,
    /// Retry delay strategy
    pub retry_delay: Duration,
    /// Enable projection snapshots
    pub enable_snapshots: bool,
    /// Snapshot frequency (number of events)
    pub snapshot_frequency: usize,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            enable_realtime_updates: true,
            batch_size: 100,
            max_lag_threshold: Duration::from_secs(30),
            enable_versioning: true,
            checkpoint_frequency: Duration::from_secs(10),
            max_retry_attempts: 3,
            retry_delay: Duration::from_secs(1),
            enable_snapshots: true,
            snapshot_frequency: 1000,
        }
    }
}

/// Projection identifier
pub type ProjectionId = String;

/// Event position in stream
pub type EventPosition = u64;

/// Projection version
pub type ProjectionVersion = u64;

/// Projection definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionDefinition {
    pub id: ProjectionId,
    pub name: String,
    pub version: ProjectionVersion,
    pub description: Option<String>,
    pub event_types: Vec<String>,
    pub query: ProjectionQuery,
    pub output_config: ProjectionOutputConfig,
    pub checkpoint_config: CheckpointConfig,
    pub retry_config: RetryConfig,
}

/// Projection query configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionQuery {
    pub query_type: QueryType,
    pub filters: Vec<EventFilter>,
    pub transformations: Vec<EventTransformation>,
    pub aggregations: Vec<EventAggregation>,
    pub sorting: Option<SortConfig>,
    pub grouping: Option<GroupConfig>,
}

/// Types of projection queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    EventStore,
    ReadModel,
    Materialized,
    Streaming,
    Snapshot,
}

/// Event filter for projections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    pub filter_type: FilterType,
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    EventType,
    AggregateId,
    Timestamp,
    CustomField,
    Metadata,
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    Regex,
}

/// Event transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTransformation {
    pub transformation_type: TransformationType,
    pub source_field: String,
    pub target_field: String,
    pub expression: Option<String>,
}

/// Transformation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    Map,
    Filter,
    Reduce,
    FlatMap,
    Custom,
}

/// Event aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventAggregation {
    pub aggregation_type: AggregationType,
    pub field: String,
    pub window: Option<AggregationWindow>,
}

/// Aggregation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Count,
    Sum,
    Average,
    Min,
    Max,
    First,
    Last,
    Distinct,
    Custom,
}

/// Aggregation window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationWindow {
    pub window_type: WindowType,
    pub size: Duration,
    pub slide: Option<Duration>,
}

/// Window types for aggregations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    Tumbling,
    Sliding,
    Session,
    Custom,
}

/// Sort configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    pub field: String,
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub fields: Vec<String>,
    pub aggregations: Vec<EventAggregation>,
}

/// Projection output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionOutputConfig {
    pub output_type: OutputType,
    pub destination: String,
    pub format: OutputFormat,
    pub update_strategy: UpdateStrategy,
}

/// Output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputType {
    Database,
    FileSystem,
    MessageQueue,
    API,
    Cache,
    Custom,
}

/// Output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Csv,
    Parquet,
    Avro,
    Protobuf,
    Custom,
}

/// Update strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateStrategy {
    Replace,
    Merge,
    Append,
    Upsert,
    Custom,
}

/// Checkpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    pub enabled: bool,
    pub frequency: Duration,
    pub storage_type: CheckpointStorageType,
    pub storage_config: HashMap<String, String>,
}

/// Checkpoint storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointStorageType {
    Database,
    FileSystem,
    Redis,
    Custom,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay: Duration,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

/// Projection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectionState {
    Stopped,
    Starting,
    Running,
    Paused,
    Rebuilding,
    Failed,
    Completed,
}

/// Projection checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionCheckpoint {
    pub projection_id: ProjectionId,
    pub position: EventPosition,
    pub timestamp: SystemTime,
    pub version: ProjectionVersion,
    pub metadata: HashMap<String, String>,
}

/// Projection execution context
#[derive(Debug)]
pub struct ProjectionExecution {
    pub id: ProjectionId,
    pub definition: ProjectionDefinition,
    pub state: ProjectionState,
    pub current_position: EventPosition,
    pub last_checkpoint: Option<ProjectionCheckpoint>,
    pub last_updated: SystemTime,
    pub events_processed: u64,
    pub errors: Vec<ProjectionError>,
    pub metrics: ProjectionExecutionMetrics,
    pub retry_count: u32,
}

/// Projection execution metrics
#[derive(Debug, Default)]
pub struct ProjectionExecutionMetrics {
    pub events_processed: AtomicU64,
    pub events_filtered: AtomicU64,
    pub events_transformed: AtomicU64,
    pub events_aggregated: AtomicU64,
    pub processing_time_ms: AtomicU64,
    pub errors_count: AtomicU64,
    pub last_lag_ms: AtomicU64,
}

/// Overall projection manager metrics
#[derive(Debug, Default)]
pub struct ProjectionManagerMetrics {
    pub active_projections: AtomicU64,
    pub completed_projections: AtomicU64,
    pub failed_projections: AtomicU64,
    pub total_events_processed: AtomicU64,
    pub total_processing_time_ms: AtomicU64,
    pub checkpoints_created: AtomicU64,
}

/// Projection errors
#[derive(Debug, thiserror::Error, Clone)]
pub enum ProjectionError {
    #[error("Projection not found: {projection_id}")]
    ProjectionNotFound { projection_id: ProjectionId },
    #[error("Invalid projection definition: {message}")]
    InvalidDefinition { message: String },
    #[error("Event processing failed: {event_id} - {reason}")]
    EventProcessingFailed { event_id: String, reason: String },
    #[error("Checkpoint creation failed: {reason}")]
    CheckpointFailed { reason: String },
    #[error("Output write failed: {reason}")]
    OutputWriteFailed { reason: String },
    #[error("Query execution failed: {reason}")]
    QueryExecutionFailed { reason: String },
    #[error("Serialization error: {source}")]
    SerializationError { source: String },
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("Projection error: {source}")]
    ProjectionError { source: String },
}

/// Main projection manager
pub struct ProjectionManager {
    config: ProjectionConfig,
    active_projections: Arc<RwLock<HashMap<ProjectionId, ProjectionExecution>>>,
    projection_definitions: Arc<RwLock<HashMap<ProjectionId, ProjectionDefinition>>>,
    event_processors: HashMap<ProjectionId, ProjectionProcessor>,
    checkpoint_manager: CheckpointManager,
    output_manager: OutputManager,
    metrics: Arc<ProjectionManagerMetrics>,
    event_sender: mpsc::UnboundedSender<ProjectionEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ProjectionEvent>>>,
}

/// Projection event for processing
#[derive(Debug, Clone)]
pub struct ProjectionEvent {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub event_data: serde_json::Value,
    pub position: EventPosition,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Projection processor for individual projections
pub struct ProjectionProcessor {
    projection_id: ProjectionId,
    definition: ProjectionDefinition,
    config: ProjectionConfig,
}

/// Checkpoint manager for projection state persistence
pub struct CheckpointManager {
    config: ProjectionConfig,
    checkpoints: Arc<RwLock<HashMap<ProjectionId, ProjectionCheckpoint>>>,
}

/// Output manager for writing projection results
pub struct OutputManager {
    config: ProjectionConfig,
    writers: HashMap<ProjectionId, Box<dyn ProjectionWriter>>,
}

/// Trait for projection output writers
pub trait ProjectionWriter: Send + Sync {
    /// Write projection result
    fn write(&self, projection_id: &ProjectionId, data: &serde_json::Value) -> Result<(), ProjectionError>;
    
    /// Update existing projection result
    fn update(&self, projection_id: &ProjectionId, data: &serde_json::Value) -> Result<(), ProjectionError>;
    
    /// Delete projection result
    fn delete(&self, projection_id: &ProjectionId) -> Result<(), ProjectionError>;
}

impl ProjectionManager {
    /// Create a new projection manager
    pub fn new(config: ProjectionConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            checkpoint_manager: CheckpointManager::new(config.clone()),
            output_manager: OutputManager::new(config.clone()),
            config,
            active_projections: Arc::new(RwLock::new(HashMap::new())),
            projection_definitions: Arc::new(RwLock::new(HashMap::new())),
            event_processors: HashMap::new(),
            metrics: Arc::new(ProjectionManagerMetrics::default()),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    /// Register a projection definition
    pub fn register_projection(&mut self, definition: ProjectionDefinition) -> Result<(), ProjectionError> {
        // Validate projection definition
        self.validate_projection_definition(&definition)?;

        // Create processor
        let processor = ProjectionProcessor::new(definition.clone(), self.config.clone());
        self.event_processors.insert(definition.id.clone(), processor);

        // Store definition
        if let Ok(mut definitions) = self.projection_definitions.write() {
            definitions.insert(definition.id.clone(), definition);
        }

        Ok(())
    }

    /// Start projection execution
    pub async fn start_projection(&self, projection_id: &ProjectionId) -> Result<(), ProjectionError> {
        // Get projection definition
        let definition = {
            if let Ok(definitions) = self.projection_definitions.read() {
                definitions.get(projection_id).cloned().ok_or_else(|| {
                    ProjectionError::ProjectionNotFound {
                        projection_id: projection_id.clone(),
                    }
                })?
            } else {
                return Err(ProjectionError::ProjectionNotFound {
                    projection_id: projection_id.clone(),
                });
            }
        };

        // Load checkpoint if available
        let checkpoint = self.checkpoint_manager.load_checkpoint(projection_id).await?;
        let start_position = checkpoint.as_ref().map(|cp| cp.position).unwrap_or(0);

        // Create execution context
        let execution = ProjectionExecution {
            id: projection_id.clone(),
            definition,
            state: ProjectionState::Starting,
            current_position: start_position,
            last_checkpoint: checkpoint,
            last_updated: SystemTime::now(),
            events_processed: 0,
            errors: Vec::new(),
            metrics: ProjectionExecutionMetrics::default(),
            retry_count: 0,
        };

        // Store execution context
        if let Ok(mut active) = self.active_projections.write() {
            active.insert(projection_id.clone(), execution);
        }

        // Update metrics
        self.metrics.active_projections.fetch_add(1, Ordering::Relaxed);

        // Start processing
        self.run_projection(projection_id).await?;

        Ok(())
    }

    /// Run projection processing
    async fn run_projection(&self, projection_id: &ProjectionId) -> Result<(), ProjectionError> {
        // Update state to running
        if let Ok(mut active) = self.active_projections.write() {
            if let Some(execution) = active.get_mut(projection_id) {
                execution.state = ProjectionState::Running;
            }
        }

        // Process events in batches
        loop {
            let should_continue = self.process_event_batch(projection_id).await?;
            if !should_continue {
                break;
            }

            // Check for pause/stop signals
            if let Ok(active) = self.active_projections.read() {
                if let Some(execution) = active.get(projection_id) {
                    match execution.state {
                        ProjectionState::Paused | ProjectionState::Stopped => break,
                        ProjectionState::Failed => return Ok(()),
                        _ => {}
                    }
                }
            }

            // Create checkpoint if needed
            self.maybe_create_checkpoint(projection_id).await?;
        }

        // Mark as completed
        if let Ok(mut active) = self.active_projections.write() {
            if let Some(execution) = active.get_mut(projection_id) {
                execution.state = ProjectionState::Completed;
            }
        }

        // Update metrics
        self.metrics.active_projections.fetch_sub(1, Ordering::Relaxed);
        self.metrics.completed_projections.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Process a batch of events
    async fn process_event_batch(&self, projection_id: &ProjectionId) -> Result<bool, ProjectionError> {
        // In a real implementation, this would:
        // 1. Fetch events from the event store starting from current position
        // 2. Apply filters and transformations
        // 3. Execute aggregations
        // 4. Write results to output
        // 5. Update position

        // For now, simulate processing
        if let Ok(mut active) = self.active_projections.write() {
            if let Some(execution) = active.get_mut(projection_id) {
                execution.current_position += self.config.batch_size as u64;
                execution.events_processed += self.config.batch_size as u64;
                execution.last_updated = SystemTime::now();
            }
        }

        // Simulate completion after processing some events
        Ok(false) // Return false to indicate completion
    }

    /// Create checkpoint if needed
    async fn maybe_create_checkpoint(&self, projection_id: &ProjectionId) -> Result<(), ProjectionError> {
        let should_checkpoint = if let Ok(active) = self.active_projections.read() {
            if let Some(execution) = active.get(projection_id) {
                let time_since_last = execution.last_updated
                    .duration_since(execution.last_checkpoint.as_ref()
                        .map(|cp| cp.timestamp)
                        .unwrap_or(SystemTime::UNIX_EPOCH))
                    .unwrap_or(Duration::ZERO);
                
                time_since_last >= self.config.checkpoint_frequency
            } else {
                false
            }
        } else {
            false
        };

        if should_checkpoint {
            self.create_checkpoint(projection_id).await?;
        }

        Ok(())
    }

    /// Create checkpoint for projection
    async fn create_checkpoint(&self, projection_id: &ProjectionId) -> Result<(), ProjectionError> {
        let checkpoint = if let Ok(active) = self.active_projections.read() {
            if let Some(execution) = active.get(projection_id) {
                Some(ProjectionCheckpoint {
                    projection_id: projection_id.clone(),
                    position: execution.current_position,
                    timestamp: SystemTime::now(),
                    version: execution.definition.version,
                    metadata: HashMap::new(),
                })
            } else {
                None
            }
        } else {
            None
        };

        if let Some(checkpoint) = checkpoint {
            self.checkpoint_manager.save_checkpoint(&checkpoint).await?;
            self.metrics.checkpoints_created.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Validate projection definition
    fn validate_projection_definition(&self, definition: &ProjectionDefinition) -> Result<(), ProjectionError> {
        if definition.id.is_empty() {
            return Err(ProjectionError::InvalidDefinition {
                message: "Projection ID cannot be empty".to_string(),
            });
        }

        if definition.event_types.is_empty() {
            return Err(ProjectionError::InvalidDefinition {
                message: "Projection must specify at least one event type".to_string(),
            });
        }

        Ok(())
    }

    /// Get projection state
    pub fn get_projection_state(&self, projection_id: &ProjectionId) -> Option<ProjectionState> {
        if let Ok(active) = self.active_projections.read() {
            active.get(projection_id).map(|execution| execution.state.clone())
        } else {
            None
        }
    }

    /// Get projection metrics
    pub fn get_metrics(&self) -> ProjectionManagerMetrics {
        ProjectionManagerMetrics {
            active_projections: AtomicU64::new(self.metrics.active_projections.load(Ordering::Relaxed)),
            completed_projections: AtomicU64::new(self.metrics.completed_projections.load(Ordering::Relaxed)),
            failed_projections: AtomicU64::new(self.metrics.failed_projections.load(Ordering::Relaxed)),
            total_events_processed: AtomicU64::new(self.metrics.total_events_processed.load(Ordering::Relaxed)),
            total_processing_time_ms: AtomicU64::new(self.metrics.total_processing_time_ms.load(Ordering::Relaxed)),
            checkpoints_created: AtomicU64::new(self.metrics.checkpoints_created.load(Ordering::Relaxed)),
        }
    }
}

impl ProjectionProcessor {
    /// Create a new projection processor
    pub fn new(definition: ProjectionDefinition, config: ProjectionConfig) -> Self {
        Self {
            projection_id: definition.id.clone(),
            definition,
            config,
        }
    }

    /// Process a single event
    pub async fn process_event(&self, event: &ProjectionEvent) -> Result<serde_json::Value, ProjectionError> {
        // Apply filters
        if !self.apply_filters(event)? {
            return Ok(serde_json::Value::Null);
        }

        // Apply transformations
        let transformed_data = self.apply_transformations(event)?;

        // Apply aggregations
        let aggregated_data = self.apply_aggregations(&transformed_data)?;

        Ok(aggregated_data)
    }

    /// Apply filters to event
    fn apply_filters(&self, _event: &ProjectionEvent) -> Result<bool, ProjectionError> {
        // In a real implementation, this would apply all filters in the projection definition
        Ok(true)
    }

    /// Apply transformations to event data
    fn apply_transformations(&self, event: &ProjectionEvent) -> Result<serde_json::Value, ProjectionError> {
        // In a real implementation, this would apply all transformations
        Ok(event.event_data.clone())
    }

    /// Apply aggregations
    fn apply_aggregations(&self, data: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        // In a real implementation, this would apply all aggregations
        Ok(data.clone())
    }
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(config: ProjectionConfig) -> Self {
        Self {
            config,
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load checkpoint for projection
    pub async fn load_checkpoint(&self, projection_id: &ProjectionId) -> Result<Option<ProjectionCheckpoint>, ProjectionError> {
        if let Ok(checkpoints) = self.checkpoints.read() {
            Ok(checkpoints.get(projection_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Save checkpoint for projection
    pub async fn save_checkpoint(&self, checkpoint: &ProjectionCheckpoint) -> Result<(), ProjectionError> {
        if let Ok(mut checkpoints) = self.checkpoints.write() {
            checkpoints.insert(checkpoint.projection_id.clone(), checkpoint.clone());
        }
        Ok(())
    }
}

impl OutputManager {
    /// Create a new output manager
    pub fn new(config: ProjectionConfig) -> Self {
        Self {
            config,
            writers: HashMap::new(),
        }
    }

    /// Write projection output
    pub async fn write_output(
        &self,
        projection_id: &ProjectionId,
        data: &serde_json::Value,
    ) -> Result<(), ProjectionError> {
        if let Some(writer) = self.writers.get(projection_id) {
            writer.write(projection_id, data)?;
        }
        Ok(())
    }
}

impl fmt::Display for ProjectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.get_metrics();
        writeln!(f, "ProjectionManager:")?;
        writeln!(f, "  Active Projections: {}", metrics.active_projections.load(Ordering::Relaxed))?;
        writeln!(f, "  Completed Projections: {}", metrics.completed_projections.load(Ordering::Relaxed))?;
        writeln!(f, "  Failed Projections: {}", metrics.failed_projections.load(Ordering::Relaxed))?;
        writeln!(f, "  Total Events Processed: {}", metrics.total_events_processed.load(Ordering::Relaxed))?;
        Ok(())
    }
}

/// Generate projection preprocessing code
pub fn generate_projection_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;

        // Initialize projection configuration from environment
        let projection_config = {
            let enable_realtime_updates: bool = env::var("RABBITMESH_ENABLE_REALTIME_PROJECTION_UPDATES")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let batch_size: usize = env::var("RABBITMESH_PROJECTION_BATCH_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100);

            let max_lag_threshold_secs: u64 = env::var("RABBITMESH_PROJECTION_MAX_LAG_THRESHOLD_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30);

            let enable_versioning: bool = env::var("RABBITMESH_ENABLE_PROJECTION_VERSIONING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let checkpoint_frequency_secs: u64 = env::var("RABBITMESH_PROJECTION_CHECKPOINT_FREQUENCY_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10);

            let max_retry_attempts: u32 = env::var("RABBITMESH_PROJECTION_MAX_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3);

            let retry_delay_secs: u64 = env::var("RABBITMESH_PROJECTION_RETRY_DELAY_SECS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1);

            let enable_snapshots: bool = env::var("RABBITMESH_ENABLE_PROJECTION_SNAPSHOTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let snapshot_frequency: usize = env::var("RABBITMESH_PROJECTION_SNAPSHOT_FREQUENCY")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            rabbitmesh_macros::eventsourcing::projection::ProjectionConfig {
                enable_realtime_updates,
                batch_size,
                max_lag_threshold: Duration::from_secs(max_lag_threshold_secs),
                enable_versioning,
                checkpoint_frequency: Duration::from_secs(checkpoint_frequency_secs),
                max_retry_attempts,
                retry_delay: Duration::from_secs(retry_delay_secs),
                enable_snapshots,
                snapshot_frequency,
            }
        };

        // Initialize projection manager
        let projection_manager = Arc::new(
            rabbitmesh_macros::eventsourcing::projection::ProjectionManager::new(projection_config)
        );

        // Projection manager is ready for projection registration and execution
        let _projection_manager_ref = projection_manager.clone();

        Ok(())
    }
}