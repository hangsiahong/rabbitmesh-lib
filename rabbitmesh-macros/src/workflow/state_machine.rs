//! State Machine Module
//! 
//! Provides finite state machine implementation for workflows with
//! hierarchical states, guards, actions, and comprehensive state management.

use quote::quote;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;
use std::time::Instant;
use regex::Regex;
use serde_json::Value;

/// State machine configuration
#[derive(Debug, Clone)]
pub struct StateMachineConfig {
    /// Enable state history tracking
    pub track_history: bool,
    /// Maximum history size
    pub max_history_size: usize,
    /// Enable parallel state execution
    pub parallel_execution: bool,
    /// Enable state validation
    pub validate_transitions: bool,
    /// Enable event queuing
    pub queue_events: bool,
    /// Maximum event queue size
    pub max_queue_size: usize,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        Self {
            track_history: true,
            max_history_size: 1000,
            parallel_execution: false,
            validate_transitions: true,
            queue_events: true,
            max_queue_size: 10000,
        }
    }
}

/// State identifier
pub type StateId = String;

/// Event identifier
pub type EventId = String;

/// State machine context data
pub type Context = HashMap<String, serde_json::Value>;

/// State definition
#[derive(Debug, Clone)]
pub struct State {
    pub id: StateId,
    pub name: String,
    pub description: Option<String>,
    pub entry_actions: Vec<String>,
    pub exit_actions: Vec<String>,
    pub is_final: bool,
    pub is_initial: bool,
    pub substates: Vec<StateId>,
    pub parent_state: Option<StateId>,
}

/// Transition definition
#[derive(Debug, Clone)]
pub struct Transition {
    pub id: String,
    pub from_state: StateId,
    pub to_state: StateId,
    pub event: EventId,
    pub guard: Option<String>,
    pub actions: Vec<String>,
    pub priority: i32,
}

/// State machine event
#[derive(Debug, Clone)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub data: Context,
    pub timestamp: Instant,
}

/// State machine execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub current_states: HashSet<StateId>,
    pub context: Context,
    pub history: Vec<StateTransition>,
    pub event_queue: Vec<Event>,
    pub metrics: StateMachineMetrics,
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from_state: Option<StateId>,
    pub to_state: StateId,
    pub event: EventId,
    pub timestamp: Instant,
    pub execution_time_ms: u64,
}

/// State machine metrics
#[derive(Debug, Default)]
pub struct StateMachineMetrics {
    pub total_transitions: AtomicU64,
    pub successful_transitions: AtomicU64,
    pub failed_transitions: AtomicU64,
    pub total_execution_time_ms: AtomicU64,
    pub events_processed: AtomicU64,
    pub events_queued: AtomicU64,
}

impl Clone for StateMachineMetrics {
    fn clone(&self) -> Self {
        Self {
            total_transitions: AtomicU64::new(self.total_transitions.load(std::sync::atomic::Ordering::Relaxed)),
            successful_transitions: AtomicU64::new(self.successful_transitions.load(std::sync::atomic::Ordering::Relaxed)),
            failed_transitions: AtomicU64::new(self.failed_transitions.load(std::sync::atomic::Ordering::Relaxed)),
            total_execution_time_ms: AtomicU64::new(self.total_execution_time_ms.load(std::sync::atomic::Ordering::Relaxed)),
            events_processed: AtomicU64::new(self.events_processed.load(std::sync::atomic::Ordering::Relaxed)),
            events_queued: AtomicU64::new(self.events_queued.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}

/// State machine errors
#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("Invalid transition from {from_state} to {to_state} with event {event}")]
    InvalidTransition {
        from_state: StateId,
        to_state: StateId,
        event: EventId,
    },
    #[error("State not found: {state_id}")]
    StateNotFound { state_id: StateId },
    #[error("Event not found: {event_id}")]
    EventNotFound { event_id: EventId },
    #[error("Guard condition failed for transition: {transition_id}")]
    GuardFailed { transition_id: String },
    #[error("Action execution failed: {action} - {reason}")]
    ActionFailed { action: String, reason: String },
    #[error("Event queue full: maximum size {max_size} reached")]
    EventQueueFull { max_size: usize },
    #[error("State machine configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("Execution context error: {source}")]
    ContextError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// State machine definition
pub struct StateMachine {
    config: StateMachineConfig,
    states: HashMap<StateId, State>,
    transitions: HashMap<String, Transition>,
    execution_context: Arc<RwLock<ExecutionContext>>,
}

impl StateMachine {
    /// Create a new state machine
    pub fn new(config: StateMachineConfig) -> Self {
        Self {
            config,
            states: HashMap::new(),
            transitions: HashMap::new(),
            execution_context: Arc::new(RwLock::new(ExecutionContext {
                current_states: HashSet::new(),
                context: HashMap::new(),
                history: Vec::new(),
                event_queue: Vec::new(),
                metrics: StateMachineMetrics::default(),
            })),
        }
    }

    /// Add a state to the state machine
    pub fn add_state(&mut self, state: State) -> Result<(), StateMachineError> {
        if self.states.contains_key(&state.id) {
            return Err(StateMachineError::ConfigurationError {
                message: format!("State {} already exists", state.id),
            });
        }

        // Set initial state if marked as initial
        if state.is_initial {
            if let Ok(mut ctx) = self.execution_context.write() {
                ctx.current_states.insert(state.id.clone());
            }
        }

        self.states.insert(state.id.clone(), state);
        Ok(())
    }

    /// Add a transition to the state machine
    pub fn add_transition(&mut self, transition: Transition) -> Result<(), StateMachineError> {
        // Validate that source and target states exist
        if !self.states.contains_key(&transition.from_state) {
            return Err(StateMachineError::StateNotFound {
                state_id: transition.from_state.clone(),
            });
        }

        if !self.states.contains_key(&transition.to_state) {
            return Err(StateMachineError::StateNotFound {
                state_id: transition.to_state.clone(),
            });
        }

        self.transitions.insert(transition.id.clone(), transition);
        Ok(())
    }

    /// Process an event
    pub async fn process_event(&self, event: Event) -> Result<(), StateMachineError> {
        let start_time = Instant::now();

        if self.config.queue_events {
            if let Ok(mut ctx) = self.execution_context.write() {
                if ctx.event_queue.len() >= self.config.max_queue_size {
                    return Err(StateMachineError::EventQueueFull {
                        max_size: self.config.max_queue_size,
                    });
                }
                ctx.event_queue.push(event.clone());
                ctx.metrics.events_queued.fetch_add(1, Ordering::Relaxed);
            }
        }

        let result = self.execute_transition(&event).await;
        
        // Update metrics
        if let Ok(mut ctx) = self.execution_context.write() {
            ctx.metrics.events_processed.fetch_add(1, Ordering::Relaxed);
            ctx.metrics.total_execution_time_ms.fetch_add(
                start_time.elapsed().as_millis() as u64,
                Ordering::Relaxed
            );

            match &result {
                Ok(_) => ctx.metrics.successful_transitions.fetch_add(1, Ordering::Relaxed),
                Err(_) => ctx.metrics.failed_transitions.fetch_add(1, Ordering::Relaxed),
            };
        }

        result
    }

    /// Execute state transition
    async fn execute_transition(&self, event: &Event) -> Result<(), StateMachineError> {
        let current_states = {
            if let Ok(ctx) = self.execution_context.read() {
                ctx.current_states.clone()
            } else {
                return Err(StateMachineError::ContextError {
                    source: "Failed to read execution context".into(),
                });
            }
        };

        // Find valid transitions for the current event
        let valid_transitions: Vec<&Transition> = self.transitions
            .values()
            .filter(|t| {
                t.event == event.id && 
                current_states.contains(&t.from_state)
            })
            .collect();

        if valid_transitions.is_empty() {
            return Err(StateMachineError::InvalidTransition {
                from_state: current_states.iter().next().unwrap_or(&"none".to_string()).clone(),
                to_state: "none".to_string(),
                event: event.id.clone(),
            });
        }

        // Sort by priority and execute highest priority transition
        let mut sorted_transitions = valid_transitions;
        sorted_transitions.sort_by(|a, b| b.priority.cmp(&a.priority));

        let transition = sorted_transitions[0];

        // Check guard conditions
        if let Some(ref guard) = transition.guard {
            if !self.evaluate_guard(guard, &event.data).await? {
                return Err(StateMachineError::GuardFailed {
                    transition_id: transition.id.clone(),
                });
            }
        }

        // Execute transition
        self.execute_state_transition(transition, event).await
    }

    /// Execute state transition
    async fn execute_state_transition(
        &self,
        transition: &Transition,
        event: &Event,
    ) -> Result<(), StateMachineError> {
        let transition_start = Instant::now();

        // Execute exit actions for current state
        if let Some(from_state) = self.states.get(&transition.from_state) {
            for action in &from_state.exit_actions {
                self.execute_action(action, &event.data).await?;
            }
        }

        // Execute transition actions
        for action in &transition.actions {
            self.execute_action(action, &event.data).await?;
        }

        // Update state
        if let Ok(mut ctx) = self.execution_context.write() {
            ctx.current_states.remove(&transition.from_state);
            ctx.current_states.insert(transition.to_state.clone());

            // Record transition in history
            if self.config.track_history {
                let state_transition = StateTransition {
                    from_state: Some(transition.from_state.clone()),
                    to_state: transition.to_state.clone(),
                    event: event.id.clone(),
                    timestamp: transition_start,
                    execution_time_ms: transition_start.elapsed().as_millis() as u64,
                };

                ctx.history.push(state_transition);

                // Trim history if needed
                if ctx.history.len() > self.config.max_history_size {
                    ctx.history.remove(0);
                }
            }

            ctx.metrics.total_transitions.fetch_add(1, Ordering::Relaxed);
        }

        // Execute entry actions for new state
        if let Some(to_state) = self.states.get(&transition.to_state) {
            for action in &to_state.entry_actions {
                self.execute_action(action, &event.data).await?;
            }
        }

        Ok(())
    }

    /// Evaluate guard condition
    async fn evaluate_guard(
        &self,
        guard: &str,
        event_data: &Context,
    ) -> Result<bool, StateMachineError> {
        let start_time = Instant::now();
        
        tracing::debug!("Evaluating guard expression: {}", guard);
        
        // Parse and evaluate the guard expression
        let result = match self.parse_and_evaluate_expression(guard, event_data).await {
            Ok(value) => {
                // Convert value to boolean
                match value {
                    Value::Bool(b) => b,
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::String(s) => !s.is_empty(),
                    Value::Array(arr) => !arr.is_empty(),
                    Value::Object(obj) => !obj.is_empty(),
                    Value::Null => false,
                }
            }
            Err(e) => {
                tracing::error!("Guard evaluation failed: {} - Error: {}", guard, e);
                return Err(StateMachineError::GuardFailed {
                    transition_id: format!("guard_evaluation_error: {}", e),
                });
            }
        };
        
        let evaluation_time = start_time.elapsed();
        tracing::debug!(
            "Guard '{}' evaluated to {} in {:?}", 
            guard, result, evaluation_time
        );
        
        // Update metrics
        if let Ok(ctx) = self.execution_context.write() {
            ctx.metrics.total_execution_time_ms.fetch_add(
                evaluation_time.as_millis() as u64,
                Ordering::Relaxed
            );
        }
        
        Ok(result)
    }

    /// Execute action
    async fn execute_action(
        &self,
        action: &str,
        event_data: &Context,
    ) -> Result<(), StateMachineError> {
        let start_time = Instant::now();
        
        tracing::info!("Executing action: {}", action);
        
        // Parse the action and execute it
        let result = self.parse_and_execute_action(action, event_data).await;
        
        let execution_time = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                tracing::info!(
                    "Action '{}' executed successfully in {:?}", 
                    action, execution_time
                );
            }
            Err(e) => {
                tracing::error!(
                    "Action '{}' execution failed in {:?}: {}", 
                    action, execution_time, e
                );
            }
        }
        
        // Update metrics
        if let Ok(ctx) = self.execution_context.write() {
            ctx.metrics.total_execution_time_ms.fetch_add(
                execution_time.as_millis() as u64,
                Ordering::Relaxed
            );
        }
        
        result
    }

    /// Parse and evaluate a guard/conditional expression
    fn parse_and_evaluate_expression<'a>(
        &'a self,
        expression: &'a str,
        context: &'a Context,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        Box::pin(async move {
        let trimmed_expr = expression.trim();
        
        // Handle different types of expressions
        if trimmed_expr.starts_with("context.") {
            // Context variable access: context.field_name
            let field_name = &trimmed_expr[8..]; // Remove "context."
            Ok(context.get(field_name).cloned().unwrap_or(Value::Null))
        } else if trimmed_expr.contains("==") || trimmed_expr.contains("!=") || 
                 trimmed_expr.contains("<") || trimmed_expr.contains(">") ||
                 trimmed_expr.contains("<=") || trimmed_expr.contains(">=") {
            // Comparison expressions
            self.evaluate_comparison_expression(trimmed_expr, context).await
        } else if trimmed_expr.contains("&&") || trimmed_expr.contains("||") {
            // Logical expressions
            self.evaluate_logical_expression(trimmed_expr, context).await
        } else if trimmed_expr.starts_with("regex(") && trimmed_expr.ends_with(")") {
            // Regex expressions: regex(pattern, text)
            self.evaluate_regex_expression(trimmed_expr, context).await
        } else if trimmed_expr == "true" {
            Ok(Value::Bool(true))
        } else if trimmed_expr == "false" {
            Ok(Value::Bool(false))
        } else if let Ok(num) = trimmed_expr.parse::<f64>() {
            Ok(Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0))))
        } else if trimmed_expr.starts_with('"') && trimmed_expr.ends_with('"') {
            // String literal
            let string_value = &trimmed_expr[1..trimmed_expr.len()-1];
            Ok(Value::String(string_value.to_string()))
        } else {
            // Try to parse as JSON or return as string
            serde_json::from_str(trimmed_expr)
                .or_else(|_| Ok(Value::String(trimmed_expr.to_string())))
        }
        })
    }

    /// Evaluate comparison expressions (==, !=, <, >, <=, >=)
    fn evaluate_comparison_expression<'a>(
        &'a self,
        expression: &'a str,
        context: &'a Context,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        Box::pin(async move {
        // Parse comparison operators in order of precedence
        let operators = ["<=", ">=", "==", "!=", "<", ">"];
        
        for op in &operators {
            if let Some(pos) = expression.find(op) {
                let left = expression[..pos].trim();
                let right = expression[pos + op.len()..].trim();
                
                let left_val = self.parse_and_evaluate_expression(left, context).await?;
                let right_val = self.parse_and_evaluate_expression(right, context).await?;
                
                let result = match *op {
                    "==" => self.values_equal(&left_val, &right_val),
                    "!=" => !self.values_equal(&left_val, &right_val),
                    "<" => self.compare_values(&left_val, &right_val)? < 0,
                    ">" => self.compare_values(&left_val, &right_val)? > 0,
                    "<=" => self.compare_values(&left_val, &right_val)? <= 0,
                    ">=" => self.compare_values(&left_val, &right_val)? >= 0,
                    _ => false,
                };
                
                return Ok(Value::Bool(result));
            }
        }
        
        Err("Invalid comparison expression".into())
        })
    }

    /// Evaluate logical expressions (&& and ||)
    fn evaluate_logical_expression<'a>(
        &'a self,
        expression: &'a str,
        context: &'a Context,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        Box::pin(async move {
        if let Some(pos) = expression.find("&&") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            
            let left_val = self.parse_and_evaluate_expression(left, context).await?;
            let right_val = self.parse_and_evaluate_expression(right, context).await?;
            
            let left_bool = self.value_to_bool(&left_val);
            let right_bool = self.value_to_bool(&right_val);
            
            Ok(Value::Bool(left_bool && right_bool))
        } else if let Some(pos) = expression.find("||") {
            let left = expression[..pos].trim();
            let right = expression[pos + 2..].trim();
            
            let left_val = self.parse_and_evaluate_expression(left, context).await?;
            let right_val = self.parse_and_evaluate_expression(right, context).await?;
            
            let left_bool = self.value_to_bool(&left_val);
            let right_bool = self.value_to_bool(&right_val);
            
            Ok(Value::Bool(left_bool || right_bool))
        } else {
            Err("Invalid logical expression".into())
        }
        })
    }

    /// Evaluate regex expressions
    fn evaluate_regex_expression<'a>(
        &'a self,
        expression: &'a str,
        context: &'a Context,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a>> {
        Box::pin(async move {
        // Parse regex(pattern, text) format
        let inner = &expression[6..expression.len()-1]; // Remove "regex(" and ")"
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            return Err("Regex expression must have format: regex(pattern, text)".into());
        }
        
        let pattern = parts[0].trim().trim_matches('"');
        let text_expr = parts[1].trim();
        
        let text_val = self.parse_and_evaluate_expression(text_expr, context).await?;
        let text = match text_val {
            Value::String(s) => s,
            _ => text_val.to_string(),
        };
        
        let regex = Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
        
        Ok(Value::Bool(regex.is_match(&text)))
        })
    }

    /// Parse and execute an action
    async fn parse_and_execute_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        let trimmed_action = action.trim();
        
        if trimmed_action.starts_with("log(") && trimmed_action.ends_with(")") {
            // Logging action: log(level, message)
            self.execute_log_action(trimmed_action, context).await
        } else if trimmed_action.starts_with("set_context(") && trimmed_action.ends_with(")") {
            // Context update action: set_context(key, value)
            self.execute_set_context_action(trimmed_action, context).await
        } else if trimmed_action.starts_with("metrics(") && trimmed_action.ends_with(")") {
            // Metrics action: metrics(metric_name, value, tags)
            self.execute_metrics_action(trimmed_action, context).await
        } else if trimmed_action.starts_with("delay(") && trimmed_action.ends_with(")") {
            // Delay action: delay(milliseconds)
            self.execute_delay_action(trimmed_action, context).await
        } else if trimmed_action.starts_with("notify(") && trimmed_action.ends_with(")") {
            // Notification action: notify(event_type, data)
            self.execute_notify_action(trimmed_action, context).await
        } else {
            // Generic action execution - log as info and succeed
            tracing::info!("Executing generic action: {}", trimmed_action);
            Ok(())
        }
    }

    /// Execute logging action
    async fn execute_log_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        // Parse log(level, message) format
        let inner = &action[4..action.len()-1]; // Remove "log(" and ")"
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            return Err(StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: "Log action must have format: log(level, message)".to_string(),
            });
        }
        
        let level = parts[0].trim().trim_matches('"');
        let message_expr = parts[1].trim();
        
        let message_val = self.parse_and_evaluate_expression(message_expr, context).await
            .map_err(|e| StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: format!("Failed to evaluate message expression: {}", e),
            })?;
        
        let message = match message_val {
            Value::String(s) => s,
            _ => message_val.to_string(),
        };
        
        match level.to_lowercase().as_str() {
            "error" => tracing::error!("{}", message),
            "warn" => tracing::warn!("{}", message),
            "info" => tracing::info!("{}", message),
            "debug" => tracing::debug!("{}", message),
            "trace" => tracing::trace!("{}", message),
            _ => tracing::info!("{}", message),
        }
        
        Ok(())
    }

    /// Execute context update action
    async fn execute_set_context_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        // Parse set_context(key, value) format
        let inner = &action[12..action.len()-1]; // Remove "set_context(" and ")"
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            return Err(StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: "Set context action must have format: set_context(key, value)".to_string(),
            });
        }
        
        let key = parts[0].trim().trim_matches('"').to_string();
        let value_expr = parts[1].trim();
        
        let value = self.parse_and_evaluate_expression(value_expr, context).await
            .map_err(|e| StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: format!("Failed to evaluate value expression: {}", e),
            })?;
        
        // Update execution context
        if let Ok(mut ctx) = self.execution_context.write() {
            ctx.context.insert(key.clone(), value.clone());
            tracing::debug!("Updated context: {} = {:?}", key, value);
        }
        
        Ok(())
    }

    /// Execute metrics action
    async fn execute_metrics_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        // Parse metrics(metric_name, value, tags) format
        let inner = &action[8..action.len()-1]; // Remove "metrics(" and ")"
        let parts: Vec<&str> = inner.split(',').collect();
        
        if parts.len() < 2 {
            return Err(StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: "Metrics action must have format: metrics(metric_name, value) or metrics(metric_name, value, tags)".to_string(),
            });
        }
        
        let metric_name = parts[0].trim().trim_matches('"');
        let value_expr = parts[1].trim();
        
        let value = self.parse_and_evaluate_expression(value_expr, context).await
            .map_err(|e| StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: format!("Failed to evaluate value expression: {}", e),
            })?;
        
        let numeric_value = match value {
            Value::Number(n) => n.as_f64().unwrap_or(0.0),
            Value::Bool(b) => if b { 1.0 } else { 0.0 },
            Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        };
        
        tracing::info!("Recording metric: {} = {}", metric_name, numeric_value);
        
        // Metrics integration - can be extended to push to Prometheus, InfluxDB, etc.
        // Currently using structured logging which can be consumed by metrics aggregators
        
        Ok(())
    }

    /// Execute delay action
    async fn execute_delay_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        // Parse delay(milliseconds) format
        let inner = &action[6..action.len()-1]; // Remove "delay(" and ")"
        
        let delay_val = self.parse_and_evaluate_expression(inner, context).await
            .map_err(|e| StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: format!("Failed to evaluate delay expression: {}", e),
            })?;
        
        let delay_ms = match delay_val {
            Value::Number(n) => n.as_u64().unwrap_or(0),
            Value::String(s) => s.parse::<u64>().unwrap_or(0),
            _ => 0,
        };
        
        if delay_ms > 0 {
            tracing::debug!("Executing delay: {}ms", delay_ms);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
        
        Ok(())
    }

    /// Execute notification action
    async fn execute_notify_action(
        &self,
        action: &str,
        context: &Context,
    ) -> Result<(), StateMachineError> {
        // Parse notify(event_type, data) format
        let inner = &action[7..action.len()-1]; // Remove "notify(" and ")"
        let parts: Vec<&str> = inner.splitn(2, ',').collect();
        
        if parts.len() != 2 {
            return Err(StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: "Notify action must have format: notify(event_type, data)".to_string(),
            });
        }
        
        let event_type = parts[0].trim().trim_matches('"');
        let data_expr = parts[1].trim();
        
        let data = self.parse_and_evaluate_expression(data_expr, context).await
            .map_err(|e| StateMachineError::ActionFailed {
                action: action.to_string(),
                reason: format!("Failed to evaluate data expression: {}", e),
            })?;
        
        tracing::info!("Sending notification: {} with data: {:?}", event_type, data);
        
        // Notification system integration - can be extended to use webhooks, message queues, etc.
        // Currently using structured logging which can trigger external notification systems
        
        Ok(())
    }

    /// Helper method to compare values
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => {
                a.as_f64().unwrap_or(0.0) == b.as_f64().unwrap_or(0.0)
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            // Cross-type comparisons
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                s.parse::<f64>().map_or(false, |parsed| parsed == n.as_f64().unwrap_or(0.0))
            }
            (Value::Bool(b), Value::Number(n)) | (Value::Number(n), Value::Bool(b)) => {
                let bool_as_num = if *b { 1.0 } else { 0.0 };
                bool_as_num == n.as_f64().unwrap_or(0.0)
            }
            (Value::Bool(b), Value::String(s)) | (Value::String(s), Value::Bool(b)) => {
                s.to_lowercase() == if *b { "true" } else { "false" }
            }
            _ => false,
        }
    }

    /// Helper method to compare values for ordering
    fn compare_values(&self, left: &Value, right: &Value) -> Result<i8, Box<dyn std::error::Error + Send + Sync>> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let a_val = a.as_f64().unwrap_or(0.0);
                let b_val = b.as_f64().unwrap_or(0.0);
                Ok(if a_val < b_val { -1 } else if a_val > b_val { 1 } else { 0 })
            }
            (Value::String(a), Value::String(b)) => {
                Ok(if a < b { -1 } else if a > b { 1 } else { 0 })
            }
            (Value::Number(n), Value::String(s)) => {
                if let Ok(s_num) = s.parse::<f64>() {
                    let n_val = n.as_f64().unwrap_or(0.0);
                    Ok(if n_val < s_num { -1 } else if n_val > s_num { 1 } else { 0 })
                } else {
                    Err("Cannot compare number with non-numeric string".into())
                }
            }
            (Value::String(s), Value::Number(n)) => {
                if let Ok(s_num) = s.parse::<f64>() {
                    let n_val = n.as_f64().unwrap_or(0.0);
                    Ok(if s_num < n_val { -1 } else if s_num > n_val { 1 } else { 0 })
                } else {
                    Err("Cannot compare non-numeric string with number".into())
                }
            }
            _ => Err("Cannot compare these value types".into()),
        }
    }

    /// Helper method to convert value to boolean
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
            Value::String(s) => !s.is_empty() && s.to_lowercase() != "false" && s != "0",
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
        }
    }

    /// Get current states
    pub fn get_current_states(&self) -> HashSet<StateId> {
        if let Ok(ctx) = self.execution_context.read() {
            ctx.current_states.clone()
        } else {
            HashSet::new()
        }
    }

    /// Get state history
    pub fn get_history(&self) -> Vec<StateTransition> {
        if let Ok(ctx) = self.execution_context.read() {
            ctx.history.clone()
        } else {
            Vec::new()
        }
    }

    /// Get metrics
    pub fn get_metrics(&self) -> StateMachineMetrics {
        if let Ok(ctx) = self.execution_context.read() {
            ctx.metrics.clone()
        } else {
            StateMachineMetrics::default()
        }
    }

    /// Reset state machine to initial state
    pub fn reset(&self) -> Result<(), StateMachineError> {
        if let Ok(mut ctx) = self.execution_context.write() {
            ctx.current_states.clear();
            ctx.context.clear();
            ctx.history.clear();
            ctx.event_queue.clear();

            // Find and set initial states
            for state in self.states.values() {
                if state.is_initial {
                    ctx.current_states.insert(state.id.clone());
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "StateMachine:")?;
        writeln!(f, "  States: {}", self.states.len())?;
        writeln!(f, "  Transitions: {}", self.transitions.len())?;
        
        if let Ok(ctx) = self.execution_context.read() {
            writeln!(f, "  Current States: {:?}", ctx.current_states)?;
            writeln!(f, "  History Size: {}", ctx.history.len())?;
        }

        Ok(())
    }
}

/// State machine builder for fluent configuration
pub struct StateMachineBuilder {
    config: StateMachineConfig,
    states: Vec<State>,
    transitions: Vec<Transition>,
}

impl StateMachineBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: StateMachineConfig::default(),
            states: Vec::new(),
            transitions: Vec::new(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: StateMachineConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a state
    pub fn add_state(mut self, state: State) -> Self {
        self.states.push(state);
        self
    }

    /// Add a transition
    pub fn add_transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Build the state machine
    pub fn build(self) -> Result<StateMachine, StateMachineError> {
        let mut sm = StateMachine::new(self.config);

        for state in self.states {
            sm.add_state(state)?;
        }

        for transition in self.transitions {
            sm.add_transition(transition)?;
        }

        Ok(sm)
    }
}

impl Default for StateMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate state machine preprocessing code
pub fn generate_state_machine_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::collections::{HashMap, HashSet};
        use std::sync::Arc;
        use std::env;
        use serde_json::Value;

        // Initialize state machine configuration from environment
        let state_machine_config = {
            let track_history: bool = env::var("RABBITMESH_SM_TRACK_HISTORY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_history_size: usize = env::var("RABBITMESH_SM_MAX_HISTORY_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            let parallel_execution: bool = env::var("RABBITMESH_SM_PARALLEL_EXECUTION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false);

            let validate_transitions: bool = env::var("RABBITMESH_SM_VALIDATE_TRANSITIONS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let queue_events: bool = env::var("RABBITMESH_SM_QUEUE_EVENTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_queue_size: usize = env::var("RABBITMESH_SM_MAX_QUEUE_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000);

            rabbitmesh_macros::workflow::state_machine::StateMachineConfig {
                track_history,
                max_history_size,
                parallel_execution,
                validate_transitions,
                queue_events,
                max_queue_size,
            }
        };

        // Initialize state machine builder
        let state_machine_builder = rabbitmesh_macros::workflow::state_machine::StateMachineBuilder::new()
            .with_config(state_machine_config);

        // State machine will be configured with specific states and transitions
        // based on the service implementation
        let _state_machine_builder_ref = state_machine_builder;

        Ok(())
    }
}