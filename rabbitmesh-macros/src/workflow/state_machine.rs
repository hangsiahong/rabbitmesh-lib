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
#[derive(Debug, Default, Clone)]
pub struct StateMachineMetrics {
    pub total_transitions: AtomicU64,
    pub successful_transitions: AtomicU64,
    pub failed_transitions: AtomicU64,
    pub total_execution_time_ms: AtomicU64,
    pub events_processed: AtomicU64,
    pub events_queued: AtomicU64,
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
        _guard: &str,
        _event_data: &Context,
    ) -> Result<bool, StateMachineError> {
        // In a real implementation, this would evaluate the guard expression
        // For now, we assume all guards pass
        Ok(true)
    }

    /// Execute action
    async fn execute_action(
        &self,
        action: &str,
        _event_data: &Context,
    ) -> Result<(), StateMachineError> {
        // In a real implementation, this would execute the action
        // This could involve calling external services, updating databases, etc.
        tracing::info!("Executing action: {}", action);
        Ok(())
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