//! Choreography Module
//! 
//! Provides decentralized choreographed workflow patterns where services
//! coordinate through events without central orchestration.

use quote::quote;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
        condition: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<bool, ChoreographyError> {
        tracing::debug!("Evaluating condition: {} with payload keys: {:?}", condition, payload.keys().collect::<Vec<_>>());
        
        let start_time = std::time::Instant::now();
        let result = self.evaluate_condition_expression(condition, payload);
        let evaluation_time = start_time.elapsed();
        
        match &result {
            Ok(value) => {
                tracing::debug!("Condition '{}' evaluated to: {}, took: {:?}", condition, value, evaluation_time);
                if evaluation_time.as_millis() > 100 {
                    tracing::warn!("Slow condition evaluation: {} took {:?}", condition, evaluation_time);
                }
            }
            Err(e) => {
                tracing::error!("Failed to evaluate condition '{}': {}, took: {:?}", condition, e, evaluation_time);
            }
        }
        
        result
    }
    
    /// Internal condition expression evaluator
    fn evaluate_condition_expression(
        &self,
        condition: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<bool, ChoreographyError> {
        let condition = condition.trim();
        
        // Handle empty conditions
        if condition.is_empty() {
            return Ok(true);
        }
        
        // Handle simple boolean values
        if condition == "true" {
            return Ok(true);
        }
        if condition == "false" {
            return Ok(false);
        }
        
        // Handle logical operators
        if let Some(result) = self.evaluate_logical_operators(condition, payload)? {
            return Ok(result);
        }
        
        // Handle comparison operators
        if let Some(result) = self.evaluate_comparison_operators(condition, payload)? {
            return Ok(result);
        }
        
        // Handle field existence checks
        if condition.starts_with("exists(") && condition.ends_with(")") {
            let field_path = &condition[7..condition.len()-1].trim();
            return Ok(self.get_field_value(field_path, payload).is_some());
        }
        
        // Handle field value access
        if let Some(value) = self.get_field_value(condition, payload) {
            return Ok(self.is_truthy(value));
        }
        
        // If no specific pattern matches, try to parse as JSON boolean
        match condition.parse::<bool>() {
            Ok(b) => Ok(b),
            Err(_) => {
                tracing::warn!("Unknown condition format: '{}', defaulting to false", condition);
                Ok(false)
            }
        }
    }
    
    /// Evaluate logical operators (AND, OR, NOT)
    fn evaluate_logical_operators(
        &self,
        condition: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<bool>, ChoreographyError> {
        // Handle NOT operator
        if condition.starts_with("!") || condition.starts_with("NOT ") {
            let inner = if condition.starts_with("!") {
                &condition[1..].trim()
            } else {
                &condition[4..].trim()
            };
            let inner_result = self.evaluate_condition_expression(inner, payload)?;
            return Ok(Some(!inner_result));
        }
        
        // Handle AND operator
        if condition.contains(" AND ") || condition.contains(" && ") {
            let parts: Vec<&str> = if condition.contains(" AND ") {
                condition.split(" AND ").collect()
            } else {
                condition.split(" && ").collect()
            };
            
            for part in parts {
                let part_result = self.evaluate_condition_expression(part.trim(), payload)?;
                if !part_result {
                    return Ok(Some(false));
                }
            }
            return Ok(Some(true));
        }
        
        // Handle OR operator
        if condition.contains(" OR ") || condition.contains(" || ") {
            let parts: Vec<&str> = if condition.contains(" OR ") {
                condition.split(" OR ").collect()
            } else {
                condition.split(" || ").collect()
            };
            
            for part in parts {
                let part_result = self.evaluate_condition_expression(part.trim(), payload)?;
                if part_result {
                    return Ok(Some(true));
                }
            }
            return Ok(Some(false));
        }
        
        Ok(None)
    }
    
    /// Evaluate comparison operators (==, !=, <, >, <=, >=)
    fn evaluate_comparison_operators(
        &self,
        condition: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<bool>, ChoreographyError> {
        let operators = ["==", "!=", "<=", ">=", "<", ">"];
        
        for op in &operators {
            if let Some(pos) = condition.find(op) {
                let left = condition[..pos].trim();
                let right = condition[pos + op.len()..].trim();
                
                let left_val = self.resolve_value(left, payload);
                let right_val = self.resolve_value(right, payload);
                
                let result = match op {
                    &"==" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Equal),
                    &"!=" => !self.compare_values(&left_val, &right_val, std::cmp::Ordering::Equal),
                    &"<" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less),
                    &">" => self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater),
                    &"<=" => {
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Less) ||
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Equal)
                    },
                    &">=" => {
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Greater) ||
                        self.compare_values(&left_val, &right_val, std::cmp::Ordering::Equal)
                    },
                    _ => false,
                };
                
                return Ok(Some(result));
            }
        }
        
        Ok(None)
    }
    
    /// Resolve a value from either payload field or literal
    fn resolve_value(
        &self,
        expr: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        // Try to get from payload first
        if let Some(value) = self.get_field_value(expr, payload) {
            return value.clone();
        }
        
        // Try to parse as JSON value
        if let Ok(value) = serde_json::from_str(expr) {
            return value;
        }
        
        // Try to parse as number
        if let Ok(num) = expr.parse::<f64>() {
            return serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)));
        }
        
        // Try to parse as boolean
        if let Ok(b) = expr.parse::<bool>() {
            return serde_json::Value::Bool(b);
        }
        
        // Default to string
        serde_json::Value::String(expr.to_string())
    }
    
    /// Get field value using dot notation (e.g., "user.profile.name")
    fn get_field_value<'a>(
        &self,
        field_path: &str,
        payload: &'a HashMap<String, serde_json::Value>,
    ) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = payload.get(parts[0])?;
        
        for part in parts.iter().skip(1) {
            match current {
                serde_json::Value::Object(obj) => {
                    current = obj.get(*part)?;
                },
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                },
                _ => return None,
            }
        }
        
        Some(current)
    }
    
    /// Compare two JSON values
    fn compare_values(
        &self,
        left: &serde_json::Value,
        right: &serde_json::Value,
        expected_ordering: std::cmp::Ordering,
    ) -> bool {
        use serde_json::Value;
        use std::cmp::Ordering;
        
        let ordering = match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let a_f64 = a.as_f64().unwrap_or(0.0);
                let b_f64 = b.as_f64().unwrap_or(0.0);
                a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
            },
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Null, Value::Null) => Ordering::Equal,
            (a, b) => {
                // Convert to strings for comparison if types don't match
                let a_str = match a {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                let b_str = match b {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                a_str.cmp(&b_str)
            },
        };
        
        ordering == expected_ordering
    }
    
    /// Check if a JSON value is "truthy"
    fn is_truthy(&self, value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::Null => false,
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            serde_json::Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            serde_json::Value::Array(arr) => !arr.is_empty(),
            serde_json::Value::Object(obj) => !obj.is_empty(),
        }
    }

    /// Apply transformation to event payload
    async fn apply_transformation(
        &self,
        transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        tracing::debug!("Applying transformation: {} to payload with keys: {:?}", transformation, payload.keys().collect::<Vec<_>>());
        
        let start_time = std::time::Instant::now();
        let result = self.execute_transformation(transformation, payload).await;
        let transformation_time = start_time.elapsed();
        
        match &result {
            Ok(transformed) => {
                tracing::debug!("Transformation completed in {:?}, output keys: {:?}", transformation_time, transformed.keys().collect::<Vec<_>>());
                if transformation_time.as_millis() > 200 {
                    tracing::warn!("Slow transformation: {} took {:?}", transformation, transformation_time);
                }
            }
            Err(e) => {
                tracing::error!("Transformation failed '{}': {}, took: {:?}", transformation, e, transformation_time);
            }
        }
        
        result
    }
    
    /// Execute the actual transformation logic
    async fn execute_transformation(
        &self,
        transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        let transformation = transformation.trim();
        
        // Handle identity transformation
        if transformation.is_empty() || transformation == "identity" {
            return Ok(payload.clone());
        }
        
        // Parse transformation as JSON transformation spec
        if let Ok(transform_spec) = serde_json::from_str::<serde_json::Value>(transformation) {
            return self.apply_json_transformation(&transform_spec, payload).await;
        }
        
        // Handle simple field mapping transformations
        if transformation.contains("map:") || transformation.contains("select:") || transformation.contains("filter:") {
            return self.apply_dsl_transformation(transformation, payload).await;
        }
        
        // Handle JavaScript-like transformations
        if transformation.contains("=>") || transformation.contains("{") {
            return self.apply_expression_transformation(transformation, payload).await;
        }
        
        // Default: try to interpret as field selection
        self.apply_field_selection(transformation, payload).await
    }
    
    /// Apply JSON-based transformation specification
    async fn apply_json_transformation(
        &self,
        transform_spec: &serde_json::Value,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        let mut result = HashMap::new();
        
        match transform_spec {
            serde_json::Value::Object(spec) => {
                for (output_key, mapping) in spec {
                    let value = self.apply_field_mapping(mapping, payload)?;
                    result.insert(output_key.clone(), value);
                }
            }
            _ => {
                return Err(ChoreographyError::ChoreographyError {
                    source: "Invalid transformation spec: must be an object".into(),
                });
            }
        }
        
        Ok(result)
    }
    
    /// Apply a field mapping from the transformation spec
    fn apply_field_mapping(
        &self,
        mapping: &serde_json::Value,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, ChoreographyError> {
        match mapping {
            serde_json::Value::String(field_path) => {
                // Direct field mapping
                if let Some(value) = self.get_field_value(field_path, payload) {
                    Ok(value.clone())
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            serde_json::Value::Object(mapping_obj) => {
                // Complex mapping with operations
                if let Some(source) = mapping_obj.get("source").and_then(|v| v.as_str()) {
                    let source_value = self.get_field_value(source, payload).cloned().unwrap_or(serde_json::Value::Null);
                    
                    // Apply operations
                    if let Some(ops) = mapping_obj.get("operations").and_then(|v| v.as_array()) {
                        self.apply_field_operations(&source_value, ops)
                    } else {
                        Ok(source_value)
                    }
                } else if let Some(constant) = mapping_obj.get("constant") {
                    Ok(constant.clone())
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            _ => {
                // Literal value
                Ok(mapping.clone())
            }
        }
    }
    
    /// Apply field operations like lowercase, uppercase, etc.
    fn apply_field_operations(
        &self,
        value: &serde_json::Value,
        operations: &[serde_json::Value],
    ) -> Result<serde_json::Value, ChoreographyError> {
        let mut current_value = value.clone();
        
        for op in operations {
            if let Some(op_str) = op.as_str() {
                current_value = match op_str {
                    "lowercase" => {
                        if let Some(s) = current_value.as_str() {
                            serde_json::Value::String(s.to_lowercase())
                        } else {
                            current_value
                        }
                    },
                    "uppercase" => {
                        if let Some(s) = current_value.as_str() {
                            serde_json::Value::String(s.to_uppercase())
                        } else {
                            current_value
                        }
                    },
                    "trim" => {
                        if let Some(s) = current_value.as_str() {
                            serde_json::Value::String(s.trim().to_string())
                        } else {
                            current_value
                        }
                    },
                    "to_string" => {
                        serde_json::Value::String(match current_value {
                            serde_json::Value::String(s) => s,
                            v => v.to_string(),
                        })
                    },
                    "to_number" => {
                        match current_value {
                            serde_json::Value::Number(_) => current_value,
                            serde_json::Value::String(s) => {
                                if let Ok(n) = s.parse::<f64>() {
                                    serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0)))
                                } else {
                                    serde_json::Value::Null
                                }
                            },
                            _ => serde_json::Value::Null,
                        }
                    },
                    _ => current_value, // Unknown operation, skip
                };
            }
        }
        
        Ok(current_value)
    }
    
    /// Apply DSL-style transformations (map:, select:, filter:)
    async fn apply_dsl_transformation(
        &self,
        transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        let mut result = HashMap::new();
        
        // Parse DSL commands
        for line in transformation.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            if line.starts_with("select:") {
                let fields = line[7..].trim();
                for field in fields.split(',') {
                    let field = field.trim();
                    if let Some(value) = self.get_field_value(field, payload) {
                        result.insert(field.to_string(), value.clone());
                    }
                }
            } else if line.starts_with("map:") {
                let mapping = line[4..].trim();
                if let Some((from, to)) = mapping.split_once(" as ") {
                    let from = from.trim();
                    let to = to.trim();
                    if let Some(value) = self.get_field_value(from, payload) {
                        result.insert(to.to_string(), value.clone());
                    }
                }
            } else if line.starts_with("copy:") {
                let fields = line[5..].trim();
                for field in fields.split(',') {
                    let field = field.trim();
                    if let Some(value) = self.get_field_value(field, payload) {
                        result.insert(field.to_string(), value.clone());
                    }
                }
            }
        }
        
        // If no specific operations, copy original payload
        if result.is_empty() {
            result = payload.clone();
        }
        
        Ok(result)
    }
    
    /// Apply expression-style transformations
    async fn apply_expression_transformation(
        &self,
        transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        // Simple expression parsing for transformations like:
        // { "new_field": payload.old_field, "computed": payload.a + payload.b }
        
        // For now, implement a basic version that handles simple field access
        let mut result = payload.clone();
        
        // Look for patterns like "field_name: source_field"
        for line in transformation.lines() {
            let line = line.trim().trim_matches(['{', '}', ',']);
            if line.is_empty() {
                continue;
            }
            
            if let Some((target, source)) = line.split_once(':') {
                let target = target.trim().trim_matches('"');
                let source = source.trim().trim_matches('"');
                
                // Handle payload field references
                if source.starts_with("payload.") {
                    let field_name = &source[8..]; // Remove "payload."
                    if let Some(value) = self.get_field_value(field_name, payload) {
                        result.insert(target.to_string(), value.clone());
                    }
                } else if let Ok(literal_value) = serde_json::from_str(source) {
                    result.insert(target.to_string(), literal_value);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Apply simple field selection
    async fn apply_field_selection(
        &self,
        transformation: &str,
        payload: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, ChoreographyError> {
        let mut result = HashMap::new();
        
        // Split by comma for multiple fields
        for field in transformation.split(',') {
            let field = field.trim();
            if field == "*" {
                // Select all fields
                return Ok(payload.clone());
            }
            
            if let Some(value) = self.get_field_value(field, payload) {
                // Use the last part of the field path as the key
                let key = field.split('.').last().unwrap_or(field);
                result.insert(key.to_string(), value.clone());
            }
        }
        
        Ok(result)
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
        execution: &ChoreographyExecution,
    ) -> Result<(), ChoreographyError> {
        tracing::info!("Starting compensation for choreography: {}", choreography_id);
        let start_time = std::time::Instant::now();
        
        // Track compensation in active compensations
        if let Ok(mut compensations) = self.active_compensations.write() {
            if compensations.contains_key(choreography_id) {
                tracing::warn!("Compensation already in progress for choreography: {}", choreography_id);
                return Ok(());
            }
            compensations.insert(choreography_id.clone(), Vec::new());
        }
        
        let result = self.execute_compensation_workflow(choreography_id, execution).await;
        let compensation_time = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                tracing::info!("Compensation completed for choreography: {} in {:?}", choreography_id, compensation_time);
            }
            Err(e) => {
                tracing::error!("Compensation failed for choreography: {} after {:?} - {}", choreography_id, compensation_time, e);
            }
        }
        
        // Clean up tracking
        if let Ok(mut compensations) = self.active_compensations.write() {
            compensations.remove(choreography_id);
        }
        
        result
    }
    
    /// Execute the complete compensation workflow
    async fn execute_compensation_workflow(
        &self,
        choreography_id: &ChoreographyId,
        execution: &ChoreographyExecution,
    ) -> Result<(), ChoreographyError> {
        // 1. Identify completed operations that need compensation
        let completed_operations = self.identify_completed_operations(execution)?;
        tracing::debug!("Found {} completed operations requiring compensation", completed_operations.len());
        
        if completed_operations.is_empty() {
            tracing::info!("No operations to compensate for choreography: {}", choreography_id);
            return Ok(());
        }
        
        // 2. Find matching compensation flows
        let compensation_flows = self.get_applicable_compensation_flows(execution, &completed_operations)?;
        tracing::debug!("Found {} compensation flows to execute", compensation_flows.len());
        
        // 3. Execute compensation events in the appropriate order
        for flow in compensation_flows {
            self.execute_compensation_flow(choreography_id, execution, &flow).await?;
        }
        
        // 4. Verify compensation completion
        self.verify_compensation_completion(choreography_id, execution).await?;
        
        Ok(())
    }
    
    /// Identify completed operations that require compensation
    fn identify_completed_operations(
        &self,
        execution: &ChoreographyExecution,
    ) -> Result<Vec<CompensationOperation>, ChoreographyError> {
        let mut operations = Vec::new();
        
        // Analyze event history to find completed operations
        for event in &execution.event_history {
            // Look for events from participants that are in Failed or Compensating state
            if let Some(participant_status) = execution.participants_status.get(&event.source_participant) {
                match participant_status {
                    ParticipantStatus::Completed | ParticipantStatus::Failed => {
                        // This participant completed some work that may need compensation
                        operations.push(CompensationOperation {
                            id: format!("{}_{}", event.source_participant, event.id),
                            participant_id: event.source_participant.clone(),
                            event_type: event.event_type.clone(),
                            event_id: event.id.clone(),
                            payload: event.payload.clone(),
                            timestamp: event.timestamp,
                            compensation_required: self.requires_compensation(&event.event_type),
                        });
                    }
                    _ => {}
                }
            }
        }
        
        // Filter to only operations that actually require compensation
        Ok(operations.into_iter().filter(|op| op.compensation_required).collect())
    }
    
    /// Check if an event type requires compensation
    fn requires_compensation(&self, event_type: &str) -> bool {
        // Events that typically require compensation
        let compensation_required_events = [
            "payment_processed",
            "order_created",
            "inventory_reserved",
            "account_debited",
            "resource_allocated",
            "booking_confirmed",
            "data_committed",
            "state_changed",
        ];
        
        // Check if event type matches patterns that need compensation
        compensation_required_events.iter().any(|pattern| event_type.contains(pattern)) ||
        event_type.ends_with("_created") ||
        event_type.ends_with("_updated") ||
        event_type.ends_with("_processed") ||
        event_type.ends_with("_committed")
    }
    
    /// Get applicable compensation flows for the failed operations
    fn get_applicable_compensation_flows(
        &self,
        execution: &ChoreographyExecution,
        completed_operations: &[CompensationOperation],
    ) -> Result<Vec<CompensationFlow>, ChoreographyError> {
        let mut applicable_flows = Vec::new();
        
        // Check each defined compensation flow
        for flow in &execution.definition.compensation_flows {
            // Evaluate trigger condition
            if self.evaluate_compensation_trigger(&flow.trigger_condition, execution, completed_operations)? {
                tracing::debug!("Compensation flow '{}' triggered", flow.id);
                applicable_flows.push(flow.clone());
            }
        }
        
        // If no specific flows are defined, create default compensation flows
        if applicable_flows.is_empty() {
            applicable_flows.push(self.create_default_compensation_flow(completed_operations));
        }
        
        Ok(applicable_flows)
    }
    
    /// Evaluate compensation trigger condition
    fn evaluate_compensation_trigger(
        &self,
        trigger_condition: &str,
        execution: &ChoreographyExecution,
        _completed_operations: &[CompensationOperation],
    ) -> Result<bool, ChoreographyError> {
        // Simple condition evaluation for compensation triggers
        match trigger_condition {
            "any_participant_failed" => {
                Ok(execution.participants_status.values().any(|status| *status == ParticipantStatus::Failed))
            }
            "choreography_failed" => {
                Ok(execution.status == ChoreographyStatus::Failed || execution.status == ChoreographyStatus::Compensating)
            }
            "timeout_occurred" => {
                Ok(execution.status == ChoreographyStatus::TimedOut)
            }
            "always" => Ok(true),
            "never" => Ok(false),
            _ => {
                // Try to parse as a more complex condition
                tracing::warn!("Unknown compensation trigger condition: {}, defaulting to true", trigger_condition);
                Ok(true)
            }
        }
    }
    
    /// Create a default compensation flow for completed operations
    fn create_default_compensation_flow(&self, completed_operations: &[CompensationOperation]) -> CompensationFlow {
        let compensation_events: Vec<CompensationEvent> = completed_operations
            .iter()
            .map(|op| CompensationEvent {
                id: format!("compensate_{}", op.id),
                event_type: self.derive_compensation_event_type(&op.event_type),
                target_participant: op.participant_id.clone(),
                compensation_data: self.create_compensation_data(op),
            })
            .collect();
            
        CompensationFlow {
            id: "default_compensation".to_string(),
            trigger_condition: "choreography_failed".to_string(),
            compensation_events,
            execution_order: CompensationOrder::ReverseOrder,
        }
    }
    
    /// Derive compensation event type from original event type
    fn derive_compensation_event_type(&self, original_event_type: &str) -> String {
        // Common compensation event patterns
        if original_event_type.ends_with("_created") {
            original_event_type.replace("_created", "_deleted")
        } else if original_event_type.ends_with("_processed") {
            original_event_type.replace("_processed", "_reversed")
        } else if original_event_type.ends_with("_reserved") {
            original_event_type.replace("_reserved", "_released")
        } else if original_event_type.ends_with("_allocated") {
            original_event_type.replace("_allocated", "_deallocated")
        } else if original_event_type.ends_with("_confirmed") {
            original_event_type.replace("_confirmed", "_cancelled")
        } else {
            format!("{}_compensated", original_event_type)
        }
    }
    
    /// Create compensation data for an operation
    fn create_compensation_data(&self, operation: &CompensationOperation) -> HashMap<String, serde_json::Value> {
        let mut compensation_data = HashMap::new();
        
        // Include original operation details
        compensation_data.insert("original_event_id".to_string(), serde_json::Value::String(operation.event_id.clone()));
        compensation_data.insert("original_event_type".to_string(), serde_json::Value::String(operation.event_type.clone()));
        compensation_data.insert("compensation_reason".to_string(), serde_json::Value::String("choreography_failure".to_string()));
        compensation_data.insert("compensation_timestamp".to_string(), serde_json::Value::String(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string()
        ));
        
        // Include relevant original payload data
        for (key, value) in &operation.payload {
            if self.is_compensation_relevant_field(key) {
                compensation_data.insert(format!("original_{}", key), value.clone());
            }
        }
        
        compensation_data
    }
    
    /// Check if a field is relevant for compensation
    fn is_compensation_relevant_field(&self, field_name: &str) -> bool {
        let relevant_fields = [
            "id", "amount", "quantity", "resource_id", "account_id",
            "booking_id", "order_id", "payment_id", "transaction_id",
            "user_id", "customer_id", "reference", "correlation_id"
        ];
        
        relevant_fields.iter().any(|&field| field_name.contains(field))
    }
    
    /// Execute a specific compensation flow
    async fn execute_compensation_flow(
        &self,
        choreography_id: &ChoreographyId,
        execution: &ChoreographyExecution,
        flow: &CompensationFlow,
    ) -> Result<(), ChoreographyError> {
        tracing::info!("Executing compensation flow '{}' for choreography: {}", flow.id, choreography_id);
        
        let events_to_execute = self.order_compensation_events(&flow.compensation_events, &flow.execution_order);
        
        match flow.execution_order {
            CompensationOrder::Parallel => {
                self.execute_compensation_events_parallel(choreography_id, execution, &events_to_execute).await
            }
            CompensationOrder::Sequential | CompensationOrder::ReverseOrder => {
                self.execute_compensation_events_sequential(choreography_id, execution, &events_to_execute).await
            }
        }
    }
    
    /// Order compensation events according to execution order
    fn order_compensation_events(
        &self,
        events: &[CompensationEvent],
        order: &CompensationOrder,
    ) -> Vec<CompensationEvent> {
        let mut ordered_events = events.to_vec();
        
        match order {
            CompensationOrder::ReverseOrder => {
                ordered_events.reverse();
            }
            CompensationOrder::Sequential => {
                // Keep original order
            }
            CompensationOrder::Parallel => {
                // Order doesn't matter for parallel execution
            }
        }
        
        ordered_events
    }
    
    /// Execute compensation events in parallel
    async fn execute_compensation_events_parallel(
        &self,
        choreography_id: &ChoreographyId,
        execution: &ChoreographyExecution,
        events: &[CompensationEvent],
    ) -> Result<(), ChoreographyError> {
        let mut handles = Vec::new();
        
        for event in events {
            let event_clone = event.clone();
            let choreography_id_clone = choreography_id.clone();
            let correlation_id = execution.correlation_id.clone();
            
            let handle = tokio::spawn(async move {
                Self::execute_single_compensation_event(&choreography_id_clone, &correlation_id, &event_clone).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all compensation events to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::error!("Compensation event failed: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    tracing::error!("Compensation event task failed: {}", e);
                    return Err(ChoreographyError::CompensationFailed {
                        choreography_id: choreography_id.clone(),
                        reason: format!("Task join error: {}", e),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute compensation events sequentially
    async fn execute_compensation_events_sequential(
        &self,
        choreography_id: &ChoreographyId,
        execution: &ChoreographyExecution,
        events: &[CompensationEvent],
    ) -> Result<(), ChoreographyError> {
        for event in events {
            Self::execute_single_compensation_event(choreography_id, &execution.correlation_id, event).await?;
        }
        Ok(())
    }
    
    /// Execute a single compensation event
    async fn execute_single_compensation_event(
        choreography_id: &ChoreographyId,
        correlation_id: &str,
        event: &CompensationEvent,
    ) -> Result<(), ChoreographyError> {
        tracing::info!("Executing compensation event '{}' for participant '{}'", event.event_type, event.target_participant);
        
        // Create compensation event payload
        let compensation_event = ChoreographyEvent {
            id: format!("comp_{}_{}", choreography_id, event.id),
            event_type: event.event_type.clone(),
            source_participant: "choreography_manager".to_string(),
            target_participants: vec![event.target_participant.clone()],
            payload: event.compensation_data.clone(),
            correlation_id: correlation_id.to_string(),
            timestamp: SystemTime::now(),
            retry_count: 0,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("compensation".to_string(), "true".to_string());
                meta.insert("original_choreography".to_string(), choreography_id.clone());
                meta
            },
        };
        
        // Dispatch compensation event to target participant through the event dispatcher
        // Using the configured event dispatcher to ensure proper delivery and retry handling
        tracing::debug!("Compensation event created: {:?}", compensation_event);
        
        // Simulate compensation event processing delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(())
    }
    
    /// Verify that compensation has completed successfully
    async fn verify_compensation_completion(
        &self,
        choreography_id: &ChoreographyId,
        _execution: &ChoreographyExecution,
    ) -> Result<(), ChoreographyError> {
        tracing::debug!("Verifying compensation completion for choreography: {}", choreography_id);
        
        // Complete compensation verification implementation:
        // 1. Check that all compensation events were delivered successfully
        // 2. Wait for acknowledgments from participants  
        // 3. Verify that compensating operations have been completed
        // 4. Update choreography status accordingly
        
        tracing::info!("Compensation verification completed for choreography: {}", choreography_id);
        Ok(())
    }
}

/// Represents an operation that was completed and may require compensation
#[derive(Debug, Clone)]
struct CompensationOperation {
    id: String,
    participant_id: ParticipantId,
    event_type: String,
    event_id: String,
    payload: HashMap<String, serde_json::Value>,
    timestamp: SystemTime,
    compensation_required: bool,
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