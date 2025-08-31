//! Choreography Module
//! 
//! Provides decentralized choreographed workflow patterns where services
//! coordinate through events without central orchestration.

use quote::quote;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use std::fmt;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// Choreography configuration
#[derive(Debug, Clone)]
pub struct ChoreographyConfig {
    /// Maximum concurrent choreographies
    pub max_concurrent_choreographies: usize,
    /// Event buffer size per participant
    pub event_buffer_size: usize,
    /// Maximum choreography execution time
    pub max_execution_time: Duration,
    /// Enable event persistence
    pub enable_event_persistence: bool,
    /// Enable compensation tracking
    pub enable_compensation_tracking: bool,
    /// Event delivery timeout
    pub event_delivery_timeout: Duration,
    /// Maximum retry attempts for failed events
    pub max_event_retry_attempts: u32,
}

impl Default for ChoreographyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_choreographies: 200,
            event_buffer_size: 1000,
            max_execution_time: Duration::from_secs(3600), // 1 hour
            enable_event_persistence: true,
            enable_compensation_tracking: true,
            event_delivery_timeout: Duration::from_secs(30),
            max_event_retry_attempts: 3,
        }
    }
}

/// Choreography identifier
pub type ChoreographyId = String;

/// Participant identifier
pub type ParticipantId = String;

/// Event type identifier
pub type EventType = String;

/// Choreography definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoreographyDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub participants: Vec<ParticipantDefinition>,
    pub event_flows: Vec<EventFlow>,
    pub compensation_flows: Vec<CompensationFlow>,
    pub timeout: Option<Duration>,
}

/// Participant in choreography
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantDefinition {
    pub id: ParticipantId,
    pub name: String,
    pub role: ParticipantRole,
    pub endpoint: Option<String>,
    pub events_produced: Vec<EventType>,
    pub events_consumed: Vec<EventType>,
    pub compensation_events: Vec<EventType>,
}

/// Roles that participants can have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Initiator,
    Coordinator,
    Participant,
    Compensator,
    Observer,
}

/// Event flow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFlow {
    pub id: String,
    pub trigger_event: EventType,
    pub source_participant: ParticipantId,
    pub target_participants: Vec<ParticipantId>,
    pub condition: Option<String>,
    pub transformation: Option<String>,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<EventRetryPolicy>,
}

/// Compensation flow for saga pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationFlow {
    pub id: String,
    pub trigger_condition: String,
    pub compensation_events: Vec<CompensationEvent>,
    pub execution_order: CompensationOrder,
}

/// Individual compensation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationEvent {
    pub id: String,
    pub event_type: EventType,
    pub target_participant: ParticipantId,
    pub compensation_data: HashMap<String, serde_json::Value>,
}

/// Order of compensation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationOrder {
    Parallel,
    Sequential,
    ReverseOrder,
}

/// Retry policy for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

/// Choreography execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChoreographyStatus {
    Starting,
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
    TimedOut,
}

/// Event in choreography
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoreographyEvent {
    pub id: String,
    pub event_type: EventType,
    pub source_participant: ParticipantId,
    pub target_participants: Vec<ParticipantId>,
    pub payload: HashMap<String, serde_json::Value>,
    pub correlation_id: String,
    pub timestamp: SystemTime,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

/// Event delivery status
#[derive(Debug, Clone, PartialEq)]
pub enum EventDeliveryStatus {
    Pending,
    InTransit,
    Delivered,
    Failed,
    Compensated,
}

/// Event delivery tracking
#[derive(Debug, Clone)]
pub struct EventDelivery {
    pub event: ChoreographyEvent,
    pub target_participant: ParticipantId,
    pub status: EventDeliveryStatus,
    pub attempt_count: u32,
    pub last_attempt: Option<SystemTime>,
    pub error: Option<String>,
}

/// Choreography execution context
#[derive(Debug)]
pub struct ChoreographyExecution {
    pub id: ChoreographyId,
    pub definition: ChoreographyDefinition,
    pub status: ChoreographyStatus,
    pub correlation_id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub participants_status: HashMap<ParticipantId, ParticipantStatus>,
    pub event_history: Vec<ChoreographyEvent>,
    pub pending_deliveries: Vec<EventDelivery>,
    pub compensation_events: Vec<ChoreographyEvent>,
    pub metrics: ChoreographyExecutionMetrics,
}

/// Participant execution status
#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantStatus {
    Ready,
    Processing,
    Completed,
    Failed,
    Compensating,
    Compensated,
}

/// Choreography execution metrics
#[derive(Debug, Default)]
pub struct ChoreographyExecutionMetrics {
    pub total_events: AtomicU64,
    pub successful_deliveries: AtomicU64,
    pub failed_deliveries: AtomicU64,
    pub compensation_events: AtomicU64,
    pub average_event_processing_time_ms: AtomicU64,
    pub participant_failures: AtomicU64,
}

/// Overall choreography manager metrics
#[derive(Debug, Default)]
pub struct ChoreographyMetrics {
    pub total_choreographies: AtomicU64,
    pub active_choreographies: AtomicU64,
    pub completed_choreographies: AtomicU64,
    pub failed_choreographies: AtomicU64,
    pub compensated_choreographies: AtomicU64,
    pub total_events_processed: AtomicU64,
    pub average_choreography_duration_ms: AtomicU64,
}

/// Choreography errors
#[derive(Debug, thiserror::Error)]
pub enum ChoreographyError {
    #[error("Choreography not found: {choreography_id}")]
    ChoreographyNotFound { choreography_id: ChoreographyId },
    #[error("Participant not found: {participant_id}")]
    ParticipantNotFound { participant_id: ParticipantId },
    #[error("Event delivery failed: {event_id} to {participant_id} - {reason}")]
    EventDeliveryFailed {
        event_id: String,
        participant_id: ParticipantId,
        reason: String,
    },
    #[error("Choreography execution limit exceeded")]
    ExecutionLimitExceeded,
    #[error("Invalid choreography definition: {message}")]
    InvalidDefinition { message: String },
    #[error("Compensation failed: {choreography_id} - {reason}")]
    CompensationFailed {
        choreography_id: ChoreographyId,
        reason: String,
    },
    #[error("Event timeout: {event_id}")]
    EventTimeout { event_id: String },
    #[error("Participant failure: {participant_id} - {reason}")]
    ParticipantFailure {
        participant_id: ParticipantId,
        reason: String,
    },
    #[error("Choreography error: {source}")]
    ChoreographyError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Main choreography manager
pub struct ChoreographyManager {
    config: ChoreographyConfig,
    active_choreographies: Arc<RwLock<HashMap<ChoreographyId, ChoreographyExecution>>>,
    choreography_definitions: Arc<RwLock<HashMap<String, ChoreographyDefinition>>>,
    participant_registry: Arc<RwLock<HashMap<ParticipantId, ParticipantDefinition>>>,
    metrics: Arc<ChoreographyMetrics>,
    event_dispatcher: EventDispatcher,
    compensation_manager: CompensationManager,
}

/// Event dispatcher for choreographed events
pub struct EventDispatcher {
    config: ChoreographyConfig,
    delivery_queues: Arc<RwLock<HashMap<ParticipantId, VecDeque<EventDelivery>>>>,
    event_sender: mpsc::UnboundedSender<EventDelivery>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<EventDelivery>>>,
}

/// Compensation manager for handling rollbacks
pub struct CompensationManager {
    config: ChoreographyConfig,
    active_compensations: Arc<RwLock<HashMap<ChoreographyId, Vec<CompensationEvent>>>>,
}

impl ChoreographyManager {
    /// Create a new choreography manager
    pub fn new(config: ChoreographyConfig) -> Self {
        Self {
            event_dispatcher: EventDispatcher::new(config.clone()),
            compensation_manager: CompensationManager::new(config.clone()),
            config,
            active_choreographies: Arc::new(RwLock::new(HashMap::new())),
            choreography_definitions: Arc::new(RwLock::new(HashMap::new())),
            participant_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(ChoreographyMetrics::default()),
        }
    }

    /// Register a choreography definition
    pub fn register_choreography(
        &self,
        definition: ChoreographyDefinition,
    ) -> Result<(), ChoreographyError> {
        // Validate choreography definition
        self.validate_choreography_definition(&definition)?;

        // Register participants
        for participant in &definition.participants {
            if let Ok(mut registry) = self.participant_registry.write() {
                registry.insert(participant.id.clone(), participant.clone());
            }
        }

        // Store definition
        if let Ok(mut definitions) = self.choreography_definitions.write() {
            definitions.insert(definition.id.clone(), definition);
        }

        Ok(())
    }

    /// Start a choreography execution
    pub async fn start_choreography(
        &self,
        choreography_id: String,
        definition_id: String,
        initial_event: ChoreographyEvent,
    ) -> Result<ChoreographyId, ChoreographyError> {
        // Check execution limits
        if let Ok(active) = self.active_choreographies.read() {
            if active.len() >= self.config.max_concurrent_choreographies {
                return Err(ChoreographyError::ExecutionLimitExceeded);
            }
        }

        // Get choreography definition
        let definition = {
            if let Ok(definitions) = self.choreography_definitions.read() {
                definitions.get(&definition_id).cloned().ok_or_else(|| {
                    ChoreographyError::ChoreographyNotFound {
                        choreography_id: definition_id.clone(),
                    }
                })?
            } else {
                return Err(ChoreographyError::ChoreographyNotFound {
                    choreography_id: definition_id,
                });
            }
        };

        // Create execution context
        let mut execution = ChoreographyExecution {
            id: choreography_id.clone(),
            definition,
            status: ChoreographyStatus::Starting,
            correlation_id: initial_event.correlation_id.clone(),
            start_time: SystemTime::now(),
            end_time: None,
            participants_status: HashMap::new(),
            event_history: vec![initial_event.clone()],
            pending_deliveries: Vec::new(),
            compensation_events: Vec::new(),
            metrics: ChoreographyExecutionMetrics::default(),
        };

        // Initialize participant statuses
        for participant in &execution.definition.participants {
            execution.participants_status.insert(participant.id.clone(), ParticipantStatus::Ready);
        }

        execution.status = ChoreographyStatus::Running;

        // Store execution context
        if let Ok(mut active) = self.active_choreographies.write() {
            active.insert(choreography_id.clone(), execution);
        }

        // Update metrics
        self.metrics.total_choreographies.fetch_add(1, Ordering::Relaxed);
        self.metrics.active_choreographies.fetch_add(1, Ordering::Relaxed);

        // Process initial event
        self.process_choreography_event(&choreography_id, initial_event).await?;

        Ok(choreography_id)
    }

    /// Process a choreography event
    pub async fn process_choreography_event(
        &self,
        choreography_id: &ChoreographyId,
        event: ChoreographyEvent,
    ) -> Result<(), ChoreographyError> {
        // Find matching event flows
        let flows = {
            if let Ok(active) = self.active_choreographies.read() {
                if let Some(execution) = active.get(choreography_id) {
                    execution.definition.event_flows
                        .iter()
                        .filter(|flow| flow.trigger_event == event.event_type && 
                                       flow.source_participant == event.source_participant)
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    return Err(ChoreographyError::ChoreographyNotFound {
                        choreography_id: choreography_id.clone(),
                    });
                }
            } else {
                return Err(ChoreographyError::ChoreographyNotFound {
                    choreography_id: choreography_id.clone(),
                });
            }
        };

        // Process each matching flow
        for flow in flows {
            self.process_event_flow(choreography_id, &event, &flow).await?;
        }

        // Update choreography status
        self.update_choreography_status(choreography_id).await?;

        Ok(())
    }

    /// Process an event flow
    async fn process_event_flow(
        &self,
        choreography_id: &ChoreographyId,
        event: &ChoreographyEvent,
        flow: &EventFlow,
    ) -> Result<(), ChoreographyError> {
        // Check condition if present
        if let Some(ref condition) = flow.condition {
            if !self.evaluate_condition(condition, &event.payload).await? {
                return Ok(()); // Skip this flow
            }
        }

        // Transform event if needed
        let transformed_payload = if let Some(ref transformation) = flow.transformation {
            self.apply_transformation(transformation, &event.payload).await?
        } else {
            event.payload.clone()
        };

        // Create delivery events for target participants
        for target in &flow.target_participants {
            let delivery_event = ChoreographyEvent {
                id: format!("{}_{}", event.id, target),
                event_type: event.event_type.clone(),
                source_participant: event.source_participant.clone(),
                target_participants: vec![target.clone()],
                payload: transformed_payload.clone(),
                correlation_id: event.correlation_id.clone(),
                timestamp: SystemTime::now(),
                retry_count: 0,
                metadata: event.metadata.clone(),
            };

            let delivery = EventDelivery {
                event: delivery_event,
                target_participant: target.clone(),
                status: EventDeliveryStatus::Pending,
                attempt_count: 0,
                last_attempt: None,
                error: None,
            };

            // Add to pending deliveries
            if let Ok(mut active) = self.active_choreographies.write() {
                if let Some(execution) = active.get_mut(choreography_id) {
                    execution.pending_deliveries.push(delivery.clone());
                }
            }

            // Dispatch event
            self.event_dispatcher.dispatch_event(delivery).await?;
        }

        Ok(())
    }

    /// Evaluate a condition expression
    async fn evaluate_condition(
        &self,
        _condition: &str,
        _payload: &HashMap<String, serde_json::Value>,
    ) -> Result<bool, ChoreographyError> {
        // In a real implementation, this would evaluate the condition expression
        // For now, we assume all conditions are true
        Ok(true)
    }

    /// Apply transformation to event payload
    async fn apply_transformation(
        &self,
        _transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        // In a real implementation, this would apply the transformation
        // For now, we return the payload unchanged
        Ok(payload.clone())
    }

    /// Update choreography execution status
    async fn update_choreography_status(&self, choreography_id: &ChoreographyId) -> Result<(), ChoreographyError> {
        if let Ok(mut active) = self.active_choreographies.write() {
            if let Some(execution) = active.get_mut(choreography_id) {
                // Check if all participants are completed
                let all_completed = execution.participants_status
                    .values()
                    .all(|status| *status == ParticipantStatus::Completed);

                // Check if any participant failed
                let any_failed = execution.participants_status
                    .values()
                    .any(|status| *status == ParticipantStatus::Failed);

                if all_completed {
                    execution.status = ChoreographyStatus::Completed;
                    execution.end_time = Some(SystemTime::now());
                    
                    // Update metrics
                    self.metrics.active_choreographies.fetch_sub(1, Ordering::Relaxed);
                    self.metrics.completed_choreographies.fetch_add(1, Ordering::Relaxed);
                    
                } else if any_failed {
                    execution.status = ChoreographyStatus::Failed;
                    execution.end_time = Some(SystemTime::now());
                    
                    // Start compensation if enabled
                    if self.config.enable_compensation_tracking {
                        self.compensation_manager.start_compensation(choreography_id, execution).await?;
                        execution.status = ChoreographyStatus::Compensating;
                    }
                    
                    // Update metrics
                    self.metrics.active_choreographies.fetch_sub(1, Ordering::Relaxed);
                    self.metrics.failed_choreographies.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        Ok(())
    }

    /// Validate choreography definition
    fn validate_choreography_definition(
        &self,
        definition: &ChoreographyDefinition,
    ) -> Result<(), ChoreographyError> {
        // Check for valid participants
        if definition.participants.is_empty() {
            return Err(ChoreographyError::InvalidDefinition {
                message: "Choreography must have at least one participant".to_string(),
            });
        }

        // Check for valid event flows
        if definition.event_flows.is_empty() {
            return Err(ChoreographyError::InvalidDefinition {
                message: "Choreography must have at least one event flow".to_string(),
            });
        }

        // Validate event flow references
        let participant_ids: HashSet<_> = definition.participants.iter().map(|p| &p.id).collect();
        
        for flow in &definition.event_flows {
            if !participant_ids.contains(&flow.source_participant) {
                return Err(ChoreographyError::InvalidDefinition {
                    message: format!("Unknown source participant: {}", flow.source_participant),
                });
            }
            
            for target in &flow.target_participants {
                if !participant_ids.contains(target) {
                    return Err(ChoreographyError::InvalidDefinition {
                        message: format!("Unknown target participant: {}", target),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get choreography status
    pub fn get_choreography_status(&self, choreography_id: &ChoreographyId) -> Option<ChoreographyStatus> {
        if let Ok(active) = self.active_choreographies.read() {
            active.get(choreography_id).map(|execution| execution.status.clone())
        } else {
            None
        }
    }

    /// Get metrics
    pub fn get_metrics(&self) -> ChoreographyMetrics {
        ChoreographyMetrics {
            total_choreographies: AtomicU64::new(self.metrics.total_choreographies.load(Ordering::Relaxed)),
            active_choreographies: AtomicU64::new(self.metrics.active_choreographies.load(Ordering::Relaxed)),
            completed_choreographies: AtomicU64::new(self.metrics.completed_choreographies.load(Ordering::Relaxed)),
            failed_choreographies: AtomicU64::new(self.metrics.failed_choreographies.load(Ordering::Relaxed)),
            compensated_choreographies: AtomicU64::new(self.metrics.compensated_choreographies.load(Ordering::Relaxed)),
            total_events_processed: AtomicU64::new(self.metrics.total_events_processed.load(Ordering::Relaxed)),
            average_choreography_duration_ms: AtomicU64::new(self.metrics.average_choreography_duration_ms.load(Ordering::Relaxed)),
        }
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(config: ChoreographyConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            delivery_queues: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    /// Dispatch an event for delivery
    pub async fn dispatch_event(&self, delivery: EventDelivery) -> Result<(), ChoreographyError> {
        // Add to delivery queue
        if let Ok(mut queues) = self.delivery_queues.write() {
            let queue = queues.entry(delivery.target_participant.clone()).or_insert_with(VecDeque::new);
            
            if queue.len() >= self.config.event_buffer_size {
                return Err(ChoreographyError::EventDeliveryFailed {
                    event_id: delivery.event.id.clone(),
                    participant_id: delivery.target_participant.clone(),
                    reason: "Event buffer full".to_string(),
                });
            }
            
            queue.push_back(delivery.clone());
        }

        // Send for processing
        self.event_sender.send(delivery).map_err(|e| {
            ChoreographyError::EventDeliveryFailed {
                event_id: "unknown".to_string(),
                participant_id: "unknown".to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(())
    }
}

impl CompensationManager {
    /// Create a new compensation manager
    pub fn new(config: ChoreographyConfig) -> Self {
        Self {
            config,
            active_compensations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start compensation for a failed choreography
    pub async fn start_compensation(
        &self,
        choreography_id: &ChoreographyId,
        _execution: &ChoreographyExecution,
    ) -> Result<(), ChoreographyError> {
        // In a real implementation, this would:
        // 1. Identify completed operations that need compensation
        // 2. Execute compensation events in the appropriate order
        // 3. Track compensation progress
        // 4. Handle compensation failures

        tracing::info!("Starting compensation for choreography: {}", choreography_id);
        Ok(())
    }
}

impl fmt::Display for ChoreographyManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.get_metrics();
        writeln!(f, "ChoreographyManager:")?;
        writeln!(f, "  Total Choreographies: {}", metrics.total_choreographies.load(Ordering::Relaxed))?;
        writeln!(f, "  Active Choreographies: {}", metrics.active_choreographies.load(Ordering::Relaxed))?;
        writeln!(f, "  Completed: {}", metrics.completed_choreographies.load(Ordering::Relaxed))?;
        writeln!(f, "  Failed: {}", metrics.failed_choreographies.load(Ordering::Relaxed))?;
        Ok(())
    }
}

/// Generate choreography preprocessing code
pub fn generate_choreography_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;

        // Initialize choreography configuration from environment
        let choreography_config = {
            let max_concurrent_choreographies: usize = env::var("RABBITMESH_MAX_CONCURRENT_CHOREOGRAPHIES")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200);

            let event_buffer_size: usize = env::var("RABBITMESH_EVENT_BUFFER_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            let max_execution_time_secs: u64 = env::var("RABBITMESH_CHOREOGRAPHY_MAX_EXECUTION_TIME_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600);

            let enable_event_persistence: bool = env::var("RABBITMESH_ENABLE_EVENT_PERSISTENCE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_compensation_tracking: bool = env::var("RABBITMESH_ENABLE_COMPENSATION_TRACKING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let event_delivery_timeout_secs: u64 = env::var("RABBITMESH_EVENT_DELIVERY_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30);

            let max_event_retry_attempts: u32 = env::var("RABBITMESH_MAX_EVENT_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3);

            rabbitmesh_macros::workflow::choreography::ChoreographyConfig {
                max_concurrent_choreographies,
                event_buffer_size,
                max_execution_time: Duration::from_secs(max_execution_time_secs),
                enable_event_persistence,
                enable_compensation_tracking,
                event_delivery_timeout: Duration::from_secs(event_delivery_timeout_secs),
                max_event_retry_attempts,
            }
        };

        // Initialize choreography manager
        let choreography_manager = Arc::new(
            rabbitmesh_macros::workflow::choreography::ChoreographyManager::new(choreography_config)
        );

        // Choreography manager is ready for choreography registration and execution
        let _choreography_manager_ref = choreography_manager.clone();

        Ok(())
    }
}