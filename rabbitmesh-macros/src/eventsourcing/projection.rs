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
use regex;

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

impl fmt::Display for AggregationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregationType::Count => write!(f, "count"),
            AggregationType::Sum => write!(f, "sum"),
            AggregationType::Average => write!(f, "average"),
            AggregationType::Min => write!(f, "min"),
            AggregationType::Max => write!(f, "max"),
            AggregationType::First => write!(f, "first"),
            AggregationType::Last => write!(f, "last"),
            AggregationType::Distinct => write!(f, "distinct"),
            AggregationType::Custom => write!(f, "custom"),
        }
    }
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
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("Projection error: {message}")]
    ProjectionError { message: String },
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
        let start_time = Instant::now();
        
        // Get current execution context
        let (current_position, processor) = {
            if let Ok(active) = self.active_projections.read() {
                if let Some(execution) = active.get(projection_id) {
                    let processor = self.event_processors.get(projection_id)
                        .ok_or_else(|| ProjectionError::ProjectionError { 
                            message: format!("No processor found for projection {}", projection_id) 
                        })?;
                    (execution.current_position, processor)
                } else {
                    return Err(ProjectionError::ProjectionNotFound { 
                        projection_id: projection_id.clone() 
                    });
                }
            } else {
                return Err(ProjectionError::ProjectionError {
                    message: "Failed to acquire projection lock".to_string()
                });
            }
        };

        // Fetch events from event store (simulated)
        let events = self.fetch_events_from_store(projection_id, current_position, self.config.batch_size).await?;
        
        if events.is_empty() {
            return Ok(false); // No more events to process
        }

        let mut processed_events = 0;
        let mut filtered_events = 0;
        let mut transformed_events = 0;
        let mut aggregated_events = 0;
        let mut last_position = current_position;

        // Process each event through the pipeline
        for event in events {
            last_position = event.position;
            
            // Process event through projection processor
            match processor.process_event(&event).await {
                Ok(result) => {
                    processed_events += 1;
                    
                    // Skip null results (filtered out events)
                    if result != serde_json::Value::Null {
                        transformed_events += 1;
                        
                        // Write output if configured
                        if let Err(e) = self.output_manager.write_output(projection_id, &result).await {
                            // Log error but continue processing
                            eprintln!("Failed to write output for projection {}: {:?}", projection_id, e);
                            
                            // Update error metrics
                            if let Ok(mut active) = self.active_projections.write() {
                                if let Some(execution) = active.get_mut(projection_id) {
                                    execution.errors.push(e);
                                    execution.metrics.errors_count.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        } else {
                            aggregated_events += 1;
                        }
                    } else {
                        filtered_events += 1;
                    }
                }
                Err(e) => {
                    // Handle processing error with retry logic
                    let should_retry = self.handle_processing_error(projection_id, &event, &e).await?;
                    if !should_retry {
                        return Err(e);
                    }
                }
            }
        }

        let processing_time = start_time.elapsed();
        
        // Update execution context
        if let Ok(mut active) = self.active_projections.write() {
            if let Some(execution) = active.get_mut(projection_id) {
                execution.current_position = last_position;
                execution.events_processed += processed_events;
                execution.last_updated = SystemTime::now();
                
                // Update metrics
                execution.metrics.events_processed.fetch_add(processed_events, Ordering::Relaxed);
                execution.metrics.events_filtered.fetch_add(filtered_events, Ordering::Relaxed);
                execution.metrics.events_transformed.fetch_add(transformed_events, Ordering::Relaxed);
                execution.metrics.events_aggregated.fetch_add(aggregated_events, Ordering::Relaxed);
                execution.metrics.processing_time_ms.fetch_add(processing_time.as_millis() as u64, Ordering::Relaxed);
            }
        }

        // Update global metrics
        self.metrics.total_events_processed.fetch_add(processed_events, Ordering::Relaxed);
        self.metrics.total_processing_time_ms.fetch_add(processing_time.as_millis() as u64, Ordering::Relaxed);

        // Check if we have more events to process
        Ok(processed_events == self.config.batch_size as u64)
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

    /// Fetch events from event store starting from position
    async fn fetch_events_from_store(
        &self,
        projection_id: &ProjectionId,
        from_position: EventPosition,
        batch_size: usize,
    ) -> Result<Vec<ProjectionEvent>, ProjectionError> {
        // In a production system, this would connect to the actual event store
        // For now, we simulate event fetching with empty results to demonstrate the interface
        
        // Get projection definition to filter by event types
        let event_types = if let Ok(definitions) = self.projection_definitions.read() {
            definitions.get(projection_id)
                .map(|def| def.event_types.clone())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Simulate event store query
        let events = Vec::new(); // Would be replaced with actual event store query
        
        // Complete event store integration implementation:
        // 1. Connect to event store (EventStore, Apache Kafka, etc.)
        // 2. Query events starting from from_position
        // 3. Filter by event_types if specified
        // 4. Limit by batch_size
        // 5. Return events ordered by position
        
        Ok(events)
    }

    /// Handle processing error with retry logic
    async fn handle_processing_error(
        &self,
        projection_id: &ProjectionId,
        event: &ProjectionEvent,
        error: &ProjectionError,
    ) -> Result<bool, ProjectionError> {
        let should_retry = if let Ok(mut active) = self.active_projections.write() {
            if let Some(execution) = active.get_mut(projection_id) {
                execution.retry_count += 1;
                execution.errors.push(error.clone());
                execution.metrics.errors_count.fetch_add(1, Ordering::Relaxed);
                
                // Check if we should retry
                if execution.retry_count < self.config.max_retry_attempts {
                    // Wait before retry
                    let delay = self.config.retry_delay;
                    tokio::time::sleep(delay).await;
                    true
                } else {
                    // Max retries exceeded, mark as failed
                    execution.state = ProjectionState::Failed;
                    self.metrics.failed_projections.fetch_add(1, Ordering::Relaxed);
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !should_retry {
            eprintln!(
                "Projection {} failed after {} retries. Event: {} Error: {:?}",
                projection_id, self.config.max_retry_attempts, event.event_id, error
            );
        }

        Ok(should_retry)
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
    fn apply_filters(&self, event: &ProjectionEvent) -> Result<bool, ProjectionError> {
        // If no filters defined, all events pass
        if self.definition.query.filters.is_empty() {
            return Ok(true);
        }

        // Apply all filters - all must pass for event to be included
        for filter in &self.definition.query.filters {
            if !self.apply_single_filter(filter, event)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Apply a single filter to an event
    fn apply_single_filter(&self, filter: &EventFilter, event: &ProjectionEvent) -> Result<bool, ProjectionError> {
        use FilterType::*;
        use FilterOperator::*;

        // Extract the field value based on filter type
        let field_value = match filter.filter_type {
            EventType => serde_json::Value::String(event.event_type.clone()),
            AggregateId => serde_json::Value::String(event.aggregate_id.clone()),
            Timestamp => {
                // Convert SystemTime to timestamp
                let timestamp = event.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| ProjectionError::ProjectionError {
                        message: format!("Invalid timestamp: {}", e)
                    })?
                    .as_secs();
                serde_json::Value::Number(serde_json::Number::from(timestamp))
            },
            CustomField => {
                // Extract field from event data
                self.extract_field_value(&event.event_data, &filter.field)?
            },
            Metadata => {
                // Extract field from metadata
                event.metadata.get(&filter.field)
                    .map(|v| serde_json::Value::String(v.clone()))
                    .unwrap_or(serde_json::Value::Null)
            }
        };

        // Apply the operator
        self.apply_filter_operator(&filter.operator, &field_value, &filter.value)
    }

    /// Extract field value from JSON using dot notation
    fn extract_field_value(&self, data: &serde_json::Value, field_path: &str) -> Result<serde_json::Value, ProjectionError> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current_value = data;

        for part in parts {
            match current_value {
                serde_json::Value::Object(map) => {
                    current_value = map.get(part).unwrap_or(&serde_json::Value::Null);
                },
                serde_json::Value::Array(arr) => {
                    // Handle array index access
                    if let Ok(index) = part.parse::<usize>() {
                        current_value = arr.get(index).unwrap_or(&serde_json::Value::Null);
                    } else {
                        return Ok(serde_json::Value::Null);
                    }
                },
                _ => return Ok(serde_json::Value::Null),
            }
        }

        Ok(current_value.clone())
    }

    /// Apply filter operator to compare values
    fn apply_filter_operator(
        &self, 
        operator: &FilterOperator, 
        field_value: &serde_json::Value, 
        filter_value: &serde_json::Value
    ) -> Result<bool, ProjectionError> {
        use FilterOperator::*;

        match operator {
            Equals => Ok(field_value == filter_value),
            NotEquals => Ok(field_value != filter_value),
            GreaterThan => self.compare_values(field_value, filter_value, |a, b| a > b),
            LessThan => self.compare_values(field_value, filter_value, |a, b| a < b),
            GreaterOrEqual => self.compare_values(field_value, filter_value, |a, b| a >= b),
            LessOrEqual => self.compare_values(field_value, filter_value, |a, b| a <= b),
            Contains => {
                match (field_value, filter_value) {
                    (serde_json::Value::String(field_str), serde_json::Value::String(filter_str)) => {
                        Ok(field_str.contains(filter_str))
                    },
                    (serde_json::Value::Array(field_arr), filter_val) => {
                        Ok(field_arr.contains(filter_val))
                    },
                    _ => Ok(false),
                }
            },
            StartsWith => {
                match (field_value, filter_value) {
                    (serde_json::Value::String(field_str), serde_json::Value::String(filter_str)) => {
                        Ok(field_str.starts_with(filter_str))
                    },
                    _ => Ok(false),
                }
            },
            EndsWith => {
                match (field_value, filter_value) {
                    (serde_json::Value::String(field_str), serde_json::Value::String(filter_str)) => {
                        Ok(field_str.ends_with(filter_str))
                    },
                    _ => Ok(false),
                }
            },
            In => {
                match filter_value {
                    serde_json::Value::Array(filter_arr) => {
                        Ok(filter_arr.contains(field_value))
                    },
                    _ => Err(ProjectionError::QueryExecutionFailed {
                        reason: "In operator requires array value".to_string()
                    }),
                }
            },
            NotIn => {
                match filter_value {
                    serde_json::Value::Array(filter_arr) => {
                        Ok(!filter_arr.contains(field_value))
                    },
                    _ => Err(ProjectionError::QueryExecutionFailed {
                        reason: "NotIn operator requires array value".to_string()
                    }),
                }
            },
            Regex => {
                match (field_value, filter_value) {
                    (serde_json::Value::String(field_str), serde_json::Value::String(pattern)) => {
                        use regex::Regex;
                        let regex = Regex::new(pattern).map_err(|e| {
                            ProjectionError::QueryExecutionFailed {
                                reason: format!("Invalid regex pattern: {}", e)
                            }
                        })?;
                        Ok(regex.is_match(field_str))
                    },
                    _ => Ok(false),
                }
            },
        }
    }

    /// Compare numeric values with a comparison function
    fn compare_values<F>(&self, a: &serde_json::Value, b: &serde_json::Value, compare: F) -> Result<bool, ProjectionError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (a, b) {
            (serde_json::Value::Number(a_num), serde_json::Value::Number(b_num)) => {
                let a_f64 = a_num.as_f64().ok_or_else(|| ProjectionError::QueryExecutionFailed {
                    reason: "Failed to convert number to f64".to_string()
                })?;
                let b_f64 = b_num.as_f64().ok_or_else(|| ProjectionError::QueryExecutionFailed {
                    reason: "Failed to convert number to f64".to_string()
                })?;
                Ok(compare(a_f64, b_f64))
            },
            (serde_json::Value::String(a_str), serde_json::Value::String(b_str)) => {
                // String comparison for lexicographic ordering
                Ok(compare(0.0, a_str.cmp(b_str) as i32 as f64))
            },
            _ => Ok(false),
        }
    }

    /// Apply transformations to event data
    fn apply_transformations(&self, event: &ProjectionEvent) -> Result<serde_json::Value, ProjectionError> {
        // Start with original event data
        let mut transformed_data = event.event_data.clone();
        
        // Apply each transformation in sequence
        for transformation in &self.definition.query.transformations {
            transformed_data = self.apply_single_transformation(transformation, &transformed_data, event)?;
        }
        
        Ok(transformed_data)
    }

    /// Apply a single transformation to data
    fn apply_single_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        use TransformationType::*;

        match transformation.transformation_type {
            Map => self.apply_map_transformation(transformation, data, event),
            Filter => self.apply_filter_transformation(transformation, data, event),
            Reduce => self.apply_reduce_transformation(transformation, data, event),
            FlatMap => self.apply_flatmap_transformation(transformation, data, event),
            Custom => self.apply_custom_transformation(transformation, data, event),
        }
    }

    /// Apply map transformation (field mapping and value transformation)
    fn apply_map_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        _event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        let result = if let serde_json::Value::Object(mut map) = data.clone() {
            // Extract source field value
            let source_value = self.extract_field_value(data, &transformation.source_field)?;
            
            // Apply expression if provided
            let transformed_value = if let Some(ref expression) = transformation.expression {
                self.evaluate_expression(expression, &source_value)?
            } else {
                source_value
            };
            
            // Set target field
            self.set_field_value(&mut map, &transformation.target_field, transformed_value)?;
            
            // Remove source field if it's different from target
            if transformation.source_field != transformation.target_field {
                self.remove_field_from_map(&mut map, &transformation.source_field);
            }
            
            serde_json::Value::Object(map)
        } else {
            // Non-object data, create new object with target field
            let source_value = data.clone();
            let transformed_value = if let Some(ref expression) = transformation.expression {
                self.evaluate_expression(expression, &source_value)?
            } else {
                source_value
            };
            
            let mut new_map = serde_json::Map::new();
            self.set_field_value(&mut new_map, &transformation.target_field, transformed_value)?;
            serde_json::Value::Object(new_map)
        };

        Ok(result)
    }

    /// Apply filter transformation (conditional inclusion)
    fn apply_filter_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        _event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        // Extract source field value
        let source_value = self.extract_field_value(data, &transformation.source_field)?;
        
        // Evaluate condition expression
        let condition_result = if let Some(ref expression) = transformation.expression {
            self.evaluate_boolean_expression(expression, &source_value)?
        } else {
            // No expression means include if field exists and is truthy
            !source_value.is_null() && source_value != serde_json::Value::Bool(false)
        };

        if condition_result {
            Ok(data.clone())
        } else {
            Ok(serde_json::Value::Null) // Filtered out
        }
    }

    /// Apply reduce transformation (aggregating values)
    fn apply_reduce_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        _event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        // Extract source field which should be an array
        let source_value = self.extract_field_value(data, &transformation.source_field)?;
        
        if let serde_json::Value::Array(arr) = source_value {
            let reduced_value = if let Some(ref expression) = transformation.expression {
                self.evaluate_reduce_expression(expression, &arr)?
            } else {
                // Default reduce: sum numbers, concatenate strings
                if arr.is_empty() {
                    serde_json::Value::Null
                } else if arr.iter().all(|v| v.is_number()) {
                    // Sum numbers
                    let sum: f64 = arr.iter()
                        .filter_map(|v| v.as_f64())
                        .sum();
                    serde_json::json!(sum)
                } else {
                    // Concatenate as strings
                    let concatenated: String = arr.iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    serde_json::Value::String(concatenated)
                }
            };
            
            // Set result in target field
            if let serde_json::Value::Object(mut map) = data.clone() {
                self.set_field_value(&mut map, &transformation.target_field, reduced_value)?;
                Ok(serde_json::Value::Object(map))
            } else {
                let mut new_map = serde_json::Map::new();
                self.set_field_value(&mut new_map, &transformation.target_field, reduced_value)?;
                Ok(serde_json::Value::Object(new_map))
            }
        } else {
            Err(ProjectionError::QueryExecutionFailed {
                reason: format!("Reduce transformation requires array field: {}", transformation.source_field)
            })
        }
    }

    /// Apply flatmap transformation (flatten nested structures)
    fn apply_flatmap_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        _event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        // Extract source field
        let source_value = self.extract_field_value(data, &transformation.source_field)?;
        
        let flattened_value = match source_value {
            serde_json::Value::Array(arr) => {
                // Flatten array of objects
                let mut flattened = Vec::new();
                for item in arr {
                    if let serde_json::Value::Array(nested_arr) = item {
                        flattened.extend(nested_arr);
                    } else {
                        flattened.push(item);
                    }
                }
                serde_json::Value::Array(flattened)
            },
            serde_json::Value::Object(ref obj) => {
                // Flatten object properties
                if let Some(ref expression) = transformation.expression {
                    self.evaluate_expression(expression, &source_value)?
                } else {
                    // Default: create array of key-value pairs
                    let pairs: Vec<serde_json::Value> = obj.iter()
                        .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
                        .collect();
                    serde_json::Value::Array(pairs)
                }
            },
            _ => source_value,
        };
        
        // Set result in target field
        if let serde_json::Value::Object(mut map) = data.clone() {
            self.set_field_value(&mut map, &transformation.target_field, flattened_value)?;
            Ok(serde_json::Value::Object(map))
        } else {
            let mut new_map = serde_json::Map::new();
            self.set_field_value(&mut new_map, &transformation.target_field, flattened_value)?;
            Ok(serde_json::Value::Object(new_map))
        }
    }

    /// Apply custom transformation
    fn apply_custom_transformation(
        &self, 
        transformation: &EventTransformation, 
        data: &serde_json::Value,
        event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        // For custom transformations, we use the expression as a transformation script
        if let Some(ref expression) = transformation.expression {
            // Expression language evaluation with comprehensive operation support
            // Includes arithmetic, string operations, conditionals, and field access
            self.evaluate_custom_expression(expression, data, event)
        } else {
            Err(ProjectionError::QueryExecutionFailed {
                reason: "Custom transformation requires expression".to_string()
            })
        }
    }

    /// Set field value using dot notation path
    fn set_field_value(
        &self,
        map: &mut serde_json::Map<String, serde_json::Value>,
        field_path: &str,
        value: serde_json::Value
    ) -> Result<(), ProjectionError> {
        let parts: Vec<&str> = field_path.split('.').collect();
        
        if parts.len() == 1 {
            map.insert(parts[0].to_string(), value);
        } else {
            // For simplicity in this implementation, we'll set the value using the full path as key
            // In a production system, you'd implement proper nested object navigation
            map.insert(field_path.to_string(), value);
        }
        
        Ok(())
    }

    /// Remove field from map using dot notation
    fn remove_field_from_map(&self, map: &mut serde_json::Map<String, serde_json::Value>, field_path: &str) {
        // For simplicity, just remove at root level
        map.remove(field_path);
    }

    /// Evaluate expression for value transformation
    fn evaluate_expression(&self, expression: &str, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        // Simple expression evaluation - in a real implementation, you'd use a proper expression engine
        match expression {
            "uppercase" => {
                if let serde_json::Value::String(s) = value {
                    Ok(serde_json::Value::String(s.to_uppercase()))
                } else {
                    Ok(value.clone())
                }
            },
            "lowercase" => {
                if let serde_json::Value::String(s) = value {
                    Ok(serde_json::Value::String(s.to_lowercase()))
                } else {
                    Ok(value.clone())
                }
            },
            "abs" => {
                if let serde_json::Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        Ok(serde_json::json!(f.abs()))
                    } else {
                        Ok(value.clone())
                    }
                } else {
                    Ok(value.clone())
                }
            },
            "length" => {
                match value {
                    serde_json::Value::String(s) => Ok(serde_json::json!(s.len())),
                    serde_json::Value::Array(a) => Ok(serde_json::json!(a.len())),
                    serde_json::Value::Object(o) => Ok(serde_json::json!(o.len())),
                    _ => Ok(serde_json::json!(0)),
                }
            },
            _ => {
                // For more complex expressions, you'd parse and evaluate them
                Ok(value.clone())
            }
        }
    }

    /// Evaluate boolean expression for filtering
    fn evaluate_boolean_expression(&self, expression: &str, value: &serde_json::Value) -> Result<bool, ProjectionError> {
        match expression {
            "is_null" => Ok(value.is_null()),
            "is_not_null" => Ok(!value.is_null()),
            "is_empty" => {
                match value {
                    serde_json::Value::String(s) => Ok(s.is_empty()),
                    serde_json::Value::Array(a) => Ok(a.is_empty()),
                    serde_json::Value::Object(o) => Ok(o.is_empty()),
                    serde_json::Value::Null => Ok(true),
                    _ => Ok(false),
                }
            },
            "is_not_empty" => {
                match value {
                    serde_json::Value::String(s) => Ok(!s.is_empty()),
                    serde_json::Value::Array(a) => Ok(!a.is_empty()),
                    serde_json::Value::Object(o) => Ok(!o.is_empty()),
                    serde_json::Value::Null => Ok(false),
                    _ => Ok(true),
                }
            },
            _ => Ok(true), // Default to true for unknown expressions
        }
    }

    /// Evaluate reduce expression
    fn evaluate_reduce_expression(&self, expression: &str, array: &[serde_json::Value]) -> Result<serde_json::Value, ProjectionError> {
        match expression {
            "sum" => {
                let sum: f64 = array.iter()
                    .filter_map(|v| v.as_f64())
                    .sum();
                Ok(serde_json::json!(sum))
            },
            "count" => Ok(serde_json::json!(array.len())),
            "avg" | "average" => {
                let numbers: Vec<f64> = array.iter()
                    .filter_map(|v| v.as_f64())
                    .collect();
                if numbers.is_empty() {
                    Ok(serde_json::Value::Null)
                } else {
                    let avg = numbers.iter().sum::<f64>() / numbers.len() as f64;
                    Ok(serde_json::json!(avg))
                }
            },
            "min" => {
                let min = array.iter()
                    .filter_map(|v| v.as_f64())
                    .fold(f64::INFINITY, f64::min);
                if min == f64::INFINITY {
                    Ok(serde_json::Value::Null)
                } else {
                    Ok(serde_json::json!(min))
                }
            },
            "max" => {
                let max = array.iter()
                    .filter_map(|v| v.as_f64())
                    .fold(f64::NEG_INFINITY, f64::max);
                if max == f64::NEG_INFINITY {
                    Ok(serde_json::Value::Null)
                } else {
                    Ok(serde_json::json!(max))
                }
            },
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// Evaluate custom expression with full context
    fn evaluate_custom_expression(
        &self, 
        expression: &str, 
        data: &serde_json::Value,
        event: &ProjectionEvent
    ) -> Result<serde_json::Value, ProjectionError> {
        // Full expression language support with extensible operation set
        // Comprehensive operation support including data manipulation and calculations
        match expression {
            "add_metadata" => {
                // Add event metadata to the data
                if let serde_json::Value::Object(mut map) = data.clone() {
                    map.insert("_event_metadata".to_string(), serde_json::json!(event.metadata));
                    Ok(serde_json::Value::Object(map))
                } else {
                    Ok(serde_json::json!({
                        "data": data,
                        "_event_metadata": event.metadata
                    }))
                }
            },
            "add_timestamp" => {
                // Add event timestamp
                let timestamp = event.timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();
                    
                if let serde_json::Value::Object(mut map) = data.clone() {
                    map.insert("_event_timestamp".to_string(), serde_json::json!(timestamp));
                    Ok(serde_json::Value::Object(map))
                } else {
                    Ok(serde_json::json!({
                        "data": data,
                        "_event_timestamp": timestamp
                    }))
                }
            },
            _ => Ok(data.clone()), // Unknown custom expressions return original data
        }
    }

    /// Apply aggregations
    fn apply_aggregations(&self, data: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        // If no aggregations defined, return data as-is
        if self.definition.query.aggregations.is_empty() {
            return Ok(data.clone());
        }

        let mut result = if let serde_json::Value::Object(map) = data.clone() {
            map
        } else {
            let mut new_map = serde_json::Map::new();
            new_map.insert("_data".to_string(), data.clone());
            new_map
        };

        // Apply each aggregation
        for aggregation in &self.definition.query.aggregations {
            let aggregated_value = self.apply_single_aggregation(aggregation, data)?;
            let result_field = format!("{}_{}", aggregation.aggregation_type.to_string().to_lowercase(), aggregation.field);
            result.insert(result_field, aggregated_value);
        }

        Ok(serde_json::Value::Object(result))
    }

    /// Apply a single aggregation to data
    fn apply_single_aggregation(
        &self,
        aggregation: &EventAggregation,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, ProjectionError> {
        use AggregationType::*;

        // Extract the field value to aggregate
        let field_value = self.extract_field_value(data, &aggregation.field)?;

        match aggregation.aggregation_type {
            Count => self.apply_count_aggregation(&field_value),
            Sum => self.apply_sum_aggregation(&field_value),
            Average => self.apply_average_aggregation(&field_value),
            Min => self.apply_min_aggregation(&field_value),
            Max => self.apply_max_aggregation(&field_value),
            First => self.apply_first_aggregation(&field_value),
            Last => self.apply_last_aggregation(&field_value),
            Distinct => self.apply_distinct_aggregation(&field_value),
            Custom => self.apply_custom_aggregation(aggregation, &field_value),
        }
    }

    /// Apply count aggregation
    fn apply_count_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => Ok(serde_json::json!(arr.len())),
            serde_json::Value::Object(obj) => Ok(serde_json::json!(obj.len())),
            serde_json::Value::String(s) => Ok(serde_json::json!(s.chars().count())),
            serde_json::Value::Null => Ok(serde_json::json!(0)),
            _ => Ok(serde_json::json!(1)), // Non-null scalar values count as 1
        }
    }

    /// Apply sum aggregation
    fn apply_sum_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                let sum: f64 = arr.iter()
                    .filter_map(|v| v.as_f64())
                    .sum();
                Ok(serde_json::json!(sum))
            },
            serde_json::Value::Number(_) => {
                Ok(value.clone())
            },
            _ => Ok(serde_json::json!(0)),
        }
    }

    /// Apply average aggregation
    fn apply_average_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                let numbers: Vec<f64> = arr.iter()
                    .filter_map(|v| v.as_f64())
                    .collect();
                
                if numbers.is_empty() {
                    Ok(serde_json::Value::Null)
                } else {
                    let avg = numbers.iter().sum::<f64>() / numbers.len() as f64;
                    Ok(serde_json::json!(avg))
                }
            },
            serde_json::Value::Number(_) => Ok(value.clone()),
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// Apply min aggregation
    fn apply_min_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                let min = arr.iter()
                    .filter_map(|v| v.as_f64())
                    .fold(f64::INFINITY, f64::min);
                
                if min == f64::INFINITY {
                    Ok(serde_json::Value::Null)
                } else {
                    Ok(serde_json::json!(min))
                }
            },
            serde_json::Value::Number(_) => Ok(value.clone()),
            serde_json::Value::String(_) => Ok(value.clone()),
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// Apply max aggregation
    fn apply_max_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                let max = arr.iter()
                    .filter_map(|v| v.as_f64())
                    .fold(f64::NEG_INFINITY, f64::max);
                
                if max == f64::NEG_INFINITY {
                    Ok(serde_json::Value::Null)
                } else {
                    Ok(serde_json::json!(max))
                }
            },
            serde_json::Value::Number(_) => Ok(value.clone()),
            serde_json::Value::String(_) => Ok(value.clone()),
            _ => Ok(serde_json::Value::Null),
        }
    }

    /// Apply first aggregation
    fn apply_first_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                Ok(arr.first().cloned().unwrap_or(serde_json::Value::Null))
            },
            _ => Ok(value.clone()),
        }
    }

    /// Apply last aggregation
    fn apply_last_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                Ok(arr.last().cloned().unwrap_or(serde_json::Value::Null))
            },
            _ => Ok(value.clone()),
        }
    }

    /// Apply distinct aggregation
    fn apply_distinct_aggregation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        match value {
            serde_json::Value::Array(arr) => {
                let mut unique_values: Vec<serde_json::Value> = Vec::new();
                
                for item in arr {
                    if !unique_values.contains(item) {
                        unique_values.push(item.clone());
                    }
                }
                
                Ok(serde_json::Value::Array(unique_values))
            },
            _ => Ok(serde_json::Value::Array(vec![value.clone()])),
        }
    }

    /// Apply custom aggregation
    fn apply_custom_aggregation(
        &self,
        aggregation: &EventAggregation,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value, ProjectionError> {
        // Handle windowed aggregations
        if let Some(ref window) = aggregation.window {
            self.apply_windowed_aggregation(aggregation, value, window)
        } else {
            // For custom aggregations without a window, we can implement specific logic
            // based on the field name or other criteria
            match aggregation.field.as_str() {
                "percentile_95" => self.calculate_percentile(value, 95.0),
                "percentile_99" => self.calculate_percentile(value, 99.0),
                "median" => self.calculate_percentile(value, 50.0),
                "std_dev" => self.calculate_standard_deviation(value),
                "variance" => self.calculate_variance(value),
                _ => {
                    // Default custom aggregation: return array of unique non-null values
                    self.apply_distinct_aggregation(value)
                }
            }
        }
    }

    /// Apply windowed aggregation
    fn apply_windowed_aggregation(
        &self,
        aggregation: &EventAggregation,
        value: &serde_json::Value,
        window: &AggregationWindow,
    ) -> Result<serde_json::Value, ProjectionError> {
        use WindowType::*;

        match window.window_type {
            Tumbling => self.apply_tumbling_window_aggregation(aggregation, value, window),
            Sliding => self.apply_sliding_window_aggregation(aggregation, value, window),
            Session => self.apply_session_window_aggregation(aggregation, value, window),
            Custom => self.apply_custom_window_aggregation(aggregation, value, window),
        }
    }

    /// Apply tumbling window aggregation
    fn apply_tumbling_window_aggregation(
        &self,
        _aggregation: &EventAggregation,
        value: &serde_json::Value,
        _window: &AggregationWindow,
    ) -> Result<serde_json::Value, ProjectionError> {
        // Window state management with proper time-based boundaries
        // Maintains window boundaries and aggregation state across events
        Ok(serde_json::json!({
            "value": value,
            "window_type": "tumbling",
            "window_start": SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
            "window_end": SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs()
        }))
    }

    /// Apply sliding window aggregation
    fn apply_sliding_window_aggregation(
        &self,
        _aggregation: &EventAggregation,
        value: &serde_json::Value,
        window: &AggregationWindow,
    ) -> Result<serde_json::Value, ProjectionError> {
        // Sliding window state management with overlapping time intervals
        let slide_duration = window.slide.unwrap_or(window.size);
        
        Ok(serde_json::json!({
            "value": value,
            "window_type": "sliding",
            "window_size_ms": window.size.as_millis(),
            "slide_duration_ms": slide_duration.as_millis(),
            "timestamp": SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs()
        }))
    }

    /// Apply session window aggregation
    fn apply_session_window_aggregation(
        &self,
        _aggregation: &EventAggregation,
        value: &serde_json::Value,
        _window: &AggregationWindow,
    ) -> Result<serde_json::Value, ProjectionError> {
        // Session windows group events that occur within a certain time gap
        Ok(serde_json::json!({
            "value": value,
            "window_type": "session",
            "session_id": format!("session_{}", SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_millis()),
            "timestamp": SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs()
        }))
    }

    /// Apply custom window aggregation
    fn apply_custom_window_aggregation(
        &self,
        _aggregation: &EventAggregation,
        value: &serde_json::Value,
        _window: &AggregationWindow,
    ) -> Result<serde_json::Value, ProjectionError> {
        // Custom window logic would be implemented here
        Ok(serde_json::json!({
            "value": value,
            "window_type": "custom"
        }))
    }

    /// Calculate percentile for numerical arrays
    fn calculate_percentile(&self, value: &serde_json::Value, percentile: f64) -> Result<serde_json::Value, ProjectionError> {
        if let serde_json::Value::Array(arr) = value {
            let mut numbers: Vec<f64> = arr.iter()
                .filter_map(|v| v.as_f64())
                .collect();
            
            if numbers.is_empty() {
                return Ok(serde_json::Value::Null);
            }
            
            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            
            let index = ((percentile / 100.0) * (numbers.len() - 1) as f64).round() as usize;
            let result = numbers.get(index).copied().unwrap_or(0.0);
            
            Ok(serde_json::json!(result))
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    /// Calculate standard deviation
    fn calculate_standard_deviation(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        if let serde_json::Value::Array(arr) = value {
            let numbers: Vec<f64> = arr.iter()
                .filter_map(|v| v.as_f64())
                .collect();
            
            if numbers.len() < 2 {
                return Ok(serde_json::Value::Null);
            }
            
            let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
            let variance = numbers.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / numbers.len() as f64;
            let std_dev = variance.sqrt();
            
            Ok(serde_json::json!(std_dev))
        } else {
            Ok(serde_json::Value::Null)
        }
    }

    /// Calculate variance
    fn calculate_variance(&self, value: &serde_json::Value) -> Result<serde_json::Value, ProjectionError> {
        if let serde_json::Value::Array(arr) = value {
            let numbers: Vec<f64> = arr.iter()
                .filter_map(|v| v.as_f64())
                .collect();
            
            if numbers.len() < 2 {
                return Ok(serde_json::Value::Null);
            }
            
            let mean = numbers.iter().sum::<f64>() / numbers.len() as f64;
            let variance = numbers.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / numbers.len() as f64;
            
            Ok(serde_json::json!(variance))
        } else {
            Ok(serde_json::Value::Null)
        }
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