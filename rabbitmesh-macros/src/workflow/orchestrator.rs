//! Orchestrator Module
//! 
//! Provides comprehensive workflow orchestration capabilities with
//! centralized coordination, execution tracking, and error recovery.

use quote::quote;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use std::fmt;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug, instrument};
use anyhow::{Context, Result as AnyhowResult};
use serde_json::Value as JsonValue;

/// Workflow orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum concurrent workflows
    pub max_concurrent_workflows: usize,
    /// Maximum workflow execution time
    pub max_execution_time: Duration,
    /// Enable workflow persistence
    pub enable_persistence: bool,
    /// Enable retry on failure
    pub enable_retry: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay strategy
    pub retry_delay_strategy: RetryDelayStrategy,
    /// Enable workflow versioning
    pub enable_versioning: bool,
    /// Workflow execution timeout
    pub execution_timeout: Duration,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 100,
            max_execution_time: Duration::from_secs(3600), // 1 hour
            enable_persistence: true,
            enable_retry: true,
            max_retry_attempts: 3,
            retry_delay_strategy: RetryDelayStrategy::ExponentialBackoff { base_delay_secs: 1 },
            enable_versioning: true,
            execution_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Retry delay strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryDelayStrategy {
    Fixed { delay_secs: u64 },
    LinearBackoff { base_delay_secs: u64 },
    ExponentialBackoff { base_delay_secs: u64 },
    Custom { delay_secs_list: Vec<u64> },
}

impl RetryDelayStrategy {
    pub fn get_delay(&self, attempt: u32) -> Duration {
        match self {
            Self::Fixed { delay_secs } => Duration::from_secs(*delay_secs),
            Self::LinearBackoff { base_delay_secs } => Duration::from_secs(base_delay_secs * attempt as u64),
            Self::ExponentialBackoff { base_delay_secs } => {
                let delay = base_delay_secs * 2_u64.pow(attempt.saturating_sub(1));
                Duration::from_secs(delay)
            }
            Self::Custom { delay_secs_list } => {
                let index = (attempt as usize).saturating_sub(1).min(delay_secs_list.len().saturating_sub(1));
                Duration::from_secs(delay_secs_list.get(index).copied().unwrap_or(1))
            }
        }
    }
}

/// Workflow identifier
pub type WorkflowId = String;

/// Task identifier  
pub type TaskId = String;

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub tasks: Vec<TaskDefinition>,
    pub dependencies: HashMap<TaskId, Vec<TaskId>>,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

/// Task definition within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: TaskId,
    pub name: String,
    pub task_type: TaskType,
    pub input_schema: Option<String>,
    pub output_schema: Option<String>,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
    pub compensation: Option<CompensationDefinition>,
}

/// Types of tasks in workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Service { endpoint: String, method: String },
    Script { script: String },
    Human { approval_required: bool },
    Decision { condition: String },
    Parallel { subtasks: Vec<TaskId> },
    Loop { condition: String, subtasks: Vec<TaskId> },
}

/// Retry policy for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub delay_strategy: RetryDelayStrategy,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

/// Compensation definition for saga pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationDefinition {
    pub compensation_type: CompensationType,
    pub compensation_data: HashMap<String, serde_json::Value>,
}

/// Types of compensation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompensationType {
    Service { endpoint: String, method: String },
    Script { script: String },
    Custom { handler: String },
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Compensating,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Compensated,
}

/// Workflow execution context
#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    pub id: WorkflowId,
    pub definition: WorkflowDefinition,
    pub status: WorkflowStatus,
    pub current_tasks: Vec<TaskId>,
    pub completed_tasks: Vec<TaskId>,
    pub failed_tasks: Vec<TaskId>,
    pub task_results: HashMap<TaskId, TaskResult>,
    pub context_data: HashMap<String, serde_json::Value>,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub retry_count: u32,
    pub execution_history: Vec<ExecutionEvent>,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub retry_count: u32,
}

/// Execution event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub event_type: ExecutionEventType,
    pub workflow_id: WorkflowId,
    pub task_id: Option<TaskId>,
    pub timestamp: SystemTime,
    pub data: HashMap<String, serde_json::Value>,
}

/// Types of execution events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskRetried,
    CompensationStarted,
    CompensationCompleted,
}

/// Orchestrator metrics
#[derive(Debug, Default)]
pub struct OrchestratorMetrics {
    pub total_workflows: AtomicU64,
    pub active_workflows: AtomicU64,
    pub completed_workflows: AtomicU64,
    pub failed_workflows: AtomicU64,
    pub cancelled_workflows: AtomicU64,
    pub total_tasks: AtomicU64,
    pub successful_tasks: AtomicU64,
    pub failed_tasks: AtomicU64,
    pub average_execution_time_ms: AtomicU64,
    pub compensations_executed: AtomicU64,
}

/// Orchestrator errors
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: WorkflowId },
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: TaskId },
    #[error("Workflow execution limit exceeded")]
    ExecutionLimitExceeded,
    #[error("Invalid workflow definition: {message}")]
    InvalidDefinition { message: String },
    #[error("Task execution failed: {task_id} - {reason}")]
    TaskExecutionFailed { task_id: TaskId, reason: String },
    #[error("Workflow timeout: {workflow_id}")]
    WorkflowTimeout { workflow_id: WorkflowId },
    #[error("Dependency cycle detected in workflow: {workflow_id}")]
    DependencyCycle { workflow_id: WorkflowId },
    #[error("Compensation failed: {task_id} - {reason}")]
    CompensationFailed { task_id: TaskId, reason: String },
    #[error("Orchestrator error: {source}")]
    OrchestratorError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Main orchestrator engine
pub struct WorkflowOrchestrator {
    config: OrchestratorConfig,
    active_workflows: Arc<RwLock<HashMap<WorkflowId, WorkflowExecution>>>,
    workflow_definitions: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    metrics: Arc<OrchestratorMetrics>,
    task_executor: TaskExecutor,
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ExecutionEvent>>>,
}

/// Task executor for running individual tasks
pub struct TaskExecutor {
    config: OrchestratorConfig,
}

impl WorkflowOrchestrator {
    /// Create a new workflow orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config: config.clone(),
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            workflow_definitions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(OrchestratorMetrics::default()),
            task_executor: TaskExecutor::new(config),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    /// Register a workflow definition
    pub fn register_workflow(&self, definition: WorkflowDefinition) -> Result<(), OrchestratorError> {
        // Validate workflow definition
        self.validate_workflow_definition(&definition)?;

        if let Ok(mut definitions) = self.workflow_definitions.write() {
            definitions.insert(definition.id.clone(), definition);
        }

        Ok(())
    }

    /// Start workflow execution
    pub async fn start_workflow(
        &self,
        workflow_id: String,
        definition_id: String,
        input_data: HashMap<String, serde_json::Value>,
    ) -> Result<WorkflowId, OrchestratorError> {
        // Check execution limits
        if let Ok(active) = self.active_workflows.read() {
            if active.len() >= self.config.max_concurrent_workflows {
                return Err(OrchestratorError::ExecutionLimitExceeded);
            }
        }

        // Get workflow definition
        let definition = {
            if let Ok(definitions) = self.workflow_definitions.read() {
                definitions.get(&definition_id).cloned().ok_or_else(|| {
                    OrchestratorError::WorkflowNotFound {
                        workflow_id: definition_id.clone(),
                    }
                })?
            } else {
                return Err(OrchestratorError::WorkflowNotFound {
                    workflow_id: definition_id,
                });
            }
        };

        // Create workflow execution context
        let execution = WorkflowExecution {
            id: workflow_id.clone(),
            definition,
            status: WorkflowStatus::Pending,
            current_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            failed_tasks: Vec::new(),
            task_results: HashMap::new(),
            context_data: input_data,
            start_time: SystemTime::now(),
            end_time: None,
            retry_count: 0,
            execution_history: Vec::new(),
        };

        // Store execution context
        if let Ok(mut active) = self.active_workflows.write() {
            active.insert(workflow_id.clone(), execution);
        }

        // Send workflow started event
        let _ = self.event_sender.send(ExecutionEvent {
            event_type: ExecutionEventType::WorkflowStarted,
            workflow_id: workflow_id.clone(),
            task_id: None,
            timestamp: SystemTime::now(),
            data: HashMap::new(),
        });

        // Update metrics
        self.metrics.total_workflows.fetch_add(1, Ordering::Relaxed);
        self.metrics.active_workflows.fetch_add(1, Ordering::Relaxed);

        // Start execution
        self.execute_workflow(&workflow_id).await?;

        Ok(workflow_id)
    }

    /// Execute workflow
    async fn execute_workflow(&self, workflow_id: &WorkflowId) -> Result<(), OrchestratorError> {
        loop {
            let should_continue = {
                if let Ok(mut active) = self.active_workflows.write() {
                    if let Some(execution) = active.get_mut(workflow_id) {
                        match execution.status {
                            WorkflowStatus::Pending | WorkflowStatus::Running => {
                                execution.status = WorkflowStatus::Running;
                                
                                // Find next tasks to execute
                                let next_tasks = self.get_next_tasks(execution)?;
                                
                                if next_tasks.is_empty() {
                                    // All tasks completed
                                    execution.status = WorkflowStatus::Completed;
                                    execution.end_time = Some(SystemTime::now());
                                    false
                                } else {
                                    execution.current_tasks = next_tasks;
                                    true
                                }
                            }
                            _ => false,
                        }
                    } else {
                        return Err(OrchestratorError::WorkflowNotFound {
                            workflow_id: workflow_id.clone(),
                        });
                    }
                } else {
                    false
                }
            };

            if !should_continue {
                break;
            }

            // Execute current tasks
            self.execute_current_tasks(workflow_id).await?;
        }

        // Update metrics
        self.metrics.active_workflows.fetch_sub(1, Ordering::Relaxed);
        self.metrics.completed_workflows.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Get next tasks to execute based on dependencies
    fn get_next_tasks(&self, execution: &WorkflowExecution) -> Result<Vec<TaskId>, OrchestratorError> {
        let mut next_tasks = Vec::new();

        for task in &execution.definition.tasks {
            // Skip if already completed or failed
            if execution.completed_tasks.contains(&task.id) || 
               execution.failed_tasks.contains(&task.id) {
                continue;
            }

            // Check if all dependencies are completed
            if let Some(deps) = execution.definition.dependencies.get(&task.id) {
                let all_deps_completed = deps.iter()
                    .all(|dep| execution.completed_tasks.contains(dep));
                
                if all_deps_completed {
                    next_tasks.push(task.id.clone());
                }
            } else {
                // No dependencies, can execute
                next_tasks.push(task.id.clone());
            }
        }

        Ok(next_tasks)
    }

    /// Execute current tasks
    async fn execute_current_tasks(&self, workflow_id: &WorkflowId) -> Result<(), OrchestratorError> {
        let current_tasks = {
            if let Ok(active) = self.active_workflows.read() {
                if let Some(execution) = active.get(workflow_id) {
                    execution.current_tasks.clone()
                } else {
                    return Err(OrchestratorError::WorkflowNotFound {
                        workflow_id: workflow_id.clone(),
                    });
                }
            } else {
                return Err(OrchestratorError::WorkflowNotFound {
                    workflow_id: workflow_id.clone(),
                });
            }
        };

        // Execute tasks in parallel
        let mut task_handles = Vec::new();

        for task_id in current_tasks {
            let executor = self.task_executor.clone();
            let workflow_id = workflow_id.clone();
            let task_id_clone = task_id.clone();
            
            let handle = tokio::spawn(async move {
                executor.execute_task(&workflow_id, &task_id_clone).await
            });
            
            task_handles.push((task_id, handle));
        }

        // Wait for all tasks to complete
        for (task_id, handle) in task_handles {
            match handle.await {
                Ok(Ok(result)) => {
                    self.handle_task_completion(workflow_id, &task_id, result).await?;
                }
                Ok(Err(e)) => {
                    self.handle_task_failure(workflow_id, &task_id, e).await?;
                }
                Err(e) => {
                    self.handle_task_failure(
                        workflow_id,
                        &task_id,
                        OrchestratorError::TaskExecutionFailed {
                            task_id: task_id.clone(),
                            reason: e.to_string(),
                        },
                    ).await?;
                }
            }
        }

        Ok(())
    }

    /// Handle successful task completion
    async fn handle_task_completion(
        &self,
        workflow_id: &WorkflowId,
        task_id: &TaskId,
        result: TaskResult,
    ) -> Result<(), OrchestratorError> {
        if let Ok(mut active) = self.active_workflows.write() {
            if let Some(execution) = active.get_mut(workflow_id) {
                execution.completed_tasks.push(task_id.clone());
                execution.task_results.insert(task_id.clone(), result);
            }
        }

        // Update metrics
        self.metrics.total_tasks.fetch_add(1, Ordering::Relaxed);
        self.metrics.successful_tasks.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Handle task failure
    async fn handle_task_failure(
        &self,
        workflow_id: &WorkflowId,
        task_id: &TaskId,
        error: OrchestratorError,
    ) -> Result<(), OrchestratorError> {
        if let Ok(mut active) = self.active_workflows.write() {
            if let Some(execution) = active.get_mut(workflow_id) {
                execution.failed_tasks.push(task_id.clone());
                execution.status = WorkflowStatus::Failed;
            }
        }

        // Update metrics
        self.metrics.failed_tasks.fetch_add(1, Ordering::Relaxed);
        self.metrics.failed_workflows.fetch_add(1, Ordering::Relaxed);

        Err(error)
    }

    /// Validate workflow definition
    fn validate_workflow_definition(&self, definition: &WorkflowDefinition) -> Result<(), OrchestratorError> {
        // Check for dependency cycles
        self.check_dependency_cycles(definition)?;

        // Validate task definitions
        for task in &definition.tasks {
            self.validate_task_definition(task)?;
        }

        Ok(())
    }

    /// Check for dependency cycles in workflow
    fn check_dependency_cycles(&self, definition: &WorkflowDefinition) -> Result<(), OrchestratorError> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for task in &definition.tasks {
            if !visited.contains(&task.id) {
                if self.has_cycle(
                    &task.id,
                    &definition.dependencies,
                    &mut visited,
                    &mut rec_stack,
                )? {
                    return Err(OrchestratorError::DependencyCycle {
                        workflow_id: definition.id.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Recursive cycle detection
    fn has_cycle(
        &self,
        task_id: &TaskId,
        dependencies: &HashMap<TaskId, Vec<TaskId>>,
        visited: &mut std::collections::HashSet<TaskId>,
        rec_stack: &mut std::collections::HashSet<TaskId>,
    ) -> Result<bool, OrchestratorError> {
        visited.insert(task_id.clone());
        rec_stack.insert(task_id.clone());

        if let Some(deps) = dependencies.get(task_id) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, dependencies, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dep) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(task_id);
        Ok(false)
    }

    /// Validate task definition
    #[instrument(skip(self), fields(task_id = %task.id, task_type = ?task.task_type))]
    fn validate_task_definition(&self, task: &TaskDefinition) -> Result<(), OrchestratorError> {
        debug!("Validating task definition: {}", task.id);
        
        // Validate task ID
        if task.id.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: "Task ID cannot be empty".to_string(),
            });
        }
        
        if task.id.len() > 255 {
            return Err(OrchestratorError::InvalidDefinition {
                message: "Task ID cannot exceed 255 characters".to_string(),
            });
        }
        
        // Validate task name
        if task.name.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Task '{}' must have a non-empty name", task.id),
            });
        }
        
        // Validate task type specific requirements
        match &task.task_type {
            TaskType::Service { endpoint, method } => {
                self.validate_service_task(task, endpoint, method)?;
            }
            TaskType::Script { script } => {
                self.validate_script_task(task, script)?;
            }
            TaskType::Human { approval_required: _ } => {
                self.validate_human_task(task)?;
            }
            TaskType::Decision { condition } => {
                self.validate_decision_task(task, condition)?;
            }
            TaskType::Parallel { subtasks } => {
                self.validate_parallel_task(task, subtasks)?;
            }
            TaskType::Loop { condition, subtasks } => {
                self.validate_loop_task(task, condition, subtasks)?;
            }
        }
        
        // Validate schemas if present
        if let Some(input_schema) = &task.input_schema {
            self.validate_json_schema(input_schema, "input")?;
        }
        
        if let Some(output_schema) = &task.output_schema {
            self.validate_json_schema(output_schema, "output")?;
        }
        
        // Validate timeout
        if let Some(timeout) = task.timeout {
            if timeout.as_secs() == 0 {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Task '{}' timeout must be greater than 0", task.id),
                });
            }
            if timeout > Duration::from_secs(86400) { // 24 hours
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Task '{}' timeout cannot exceed 24 hours", task.id),
                });
            }
        }
        
        // Validate retry policy
        if let Some(retry_policy) = &task.retry_policy {
            if retry_policy.max_attempts == 0 {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Task '{}' retry policy max_attempts must be greater than 0", task.id),
                });
            }
            if retry_policy.max_attempts > 100 {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Task '{}' retry policy max_attempts cannot exceed 100", task.id),
                });
            }
            if retry_policy.backoff_multiplier <= 0.0 {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Task '{}' retry policy backoff_multiplier must be positive", task.id),
                });
            }
        }
        
        // Validate compensation if present
        if let Some(compensation) = &task.compensation {
            self.validate_compensation_definition(task, compensation)?;
        }
        
        debug!("Task definition validation completed successfully for: {}", task.id);
        Ok(())
    }
    
    /// Validate service task specific requirements
    fn validate_service_task(&self, task: &TaskDefinition, endpoint: &str, method: &str) -> Result<(), OrchestratorError> {
        if endpoint.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Service task '{}' endpoint cannot be empty", task.id),
            });
        }
        
        // Basic URL validation
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Service task '{}' endpoint must start with http:// or https://", task.id),
            });
        }
        
        // Validate HTTP method
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&method.to_uppercase().as_str()) {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Service task '{}' has invalid HTTP method: {}", task.id, method),
            });
        }
        
        Ok(())
    }
    
    /// Validate script task specific requirements
    fn validate_script_task(&self, task: &TaskDefinition, script: &str) -> Result<(), OrchestratorError> {
        if script.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Script task '{}' script cannot be empty", task.id),
            });
        }
        
        if script.len() > 65536 { // 64KB limit
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Script task '{}' script exceeds 64KB limit", task.id),
            });
        }
        
        // Basic syntax validation (check for balanced braces)
        let mut brace_count = 0;
        let mut paren_count = 0;
        let mut bracket_count = 0;
        
        for ch in script.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                '(' => paren_count += 1,
                ')' => paren_count -= 1,
                '[' => bracket_count += 1,
                ']' => bracket_count -= 1,
                _ => {}
            }
            
            if brace_count < 0 || paren_count < 0 || bracket_count < 0 {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Script task '{}' has unbalanced brackets/braces/parentheses", task.id),
                });
            }
        }
        
        if brace_count != 0 || paren_count != 0 || bracket_count != 0 {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Script task '{}' has unbalanced brackets/braces/parentheses", task.id),
            });
        }
        
        Ok(())
    }
    
    /// Validate human task specific requirements
    fn validate_human_task(&self, task: &TaskDefinition) -> Result<(), OrchestratorError> {
        // Human tasks should have reasonable timeouts
        if let Some(timeout) = task.timeout {
            if timeout < Duration::from_secs(60) {
                warn!("Human task '{}' has very short timeout: {:?}", task.id, timeout);
            }
        } else {
            warn!("Human task '{}' has no timeout specified - consider adding one", task.id);
        }
        
        Ok(())
    }
    
    /// Validate decision task specific requirements
    fn validate_decision_task(&self, task: &TaskDefinition, condition: &str) -> Result<(), OrchestratorError> {
        if condition.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Decision task '{}' condition cannot be empty", task.id),
            });
        }
        
        if condition.len() > 2048 {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Decision task '{}' condition exceeds 2KB limit", task.id),
            });
        }
        
        // Basic condition validation - check for common patterns
        if !condition.contains("&&") && !condition.contains("||") && !condition.contains("==") && 
           !condition.contains("!=") && !condition.contains(">") && !condition.contains("<") {
            warn!("Decision task '{}' condition may not contain comparison operators", task.id);
        }
        
        Ok(())
    }
    
    /// Validate parallel task specific requirements
    fn validate_parallel_task(&self, task: &TaskDefinition, subtasks: &[TaskId]) -> Result<(), OrchestratorError> {
        if subtasks.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Parallel task '{}' must have at least one subtask", task.id),
            });
        }
        
        if subtasks.len() > 50 {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Parallel task '{}' cannot have more than 50 subtasks", task.id),
            });
        }
        
        // Check for duplicate subtasks
        let mut seen = std::collections::HashSet::new();
        for subtask in subtasks {
            if !seen.insert(subtask.clone()) {
                return Err(OrchestratorError::InvalidDefinition {
                    message: format!("Parallel task '{}' has duplicate subtask: {}", task.id, subtask),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate loop task specific requirements
    fn validate_loop_task(&self, task: &TaskDefinition, condition: &str, subtasks: &[TaskId]) -> Result<(), OrchestratorError> {
        if condition.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Loop task '{}' condition cannot be empty", task.id),
            });
        }
        
        if subtasks.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Loop task '{}' must have at least one subtask", task.id),
            });
        }
        
        if subtasks.len() > 20 {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("Loop task '{}' cannot have more than 20 subtasks", task.id),
            });
        }
        
        Ok(())
    }
    
    /// Validate JSON schema format
    fn validate_json_schema(&self, schema: &str, schema_type: &str) -> Result<(), OrchestratorError> {
        if schema.is_empty() {
            return Err(OrchestratorError::InvalidDefinition {
                message: format!("{} schema cannot be empty", schema_type),
            });
        }
        
        // Try to parse as JSON to ensure it's valid JSON
        serde_json::from_str::<JsonValue>(schema).map_err(|e| {
            OrchestratorError::InvalidDefinition {
                message: format!("Invalid {} schema JSON: {}", schema_type, e),
            }
        })?;
        
        Ok(())
    }
    
    /// Validate compensation definition
    fn validate_compensation_definition(&self, task: &TaskDefinition, compensation: &CompensationDefinition) -> Result<(), OrchestratorError> {
        match &compensation.compensation_type {
            CompensationType::Service { endpoint, method } => {
                if endpoint.is_empty() {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' compensation service endpoint cannot be empty", task.id),
                    });
                }
                
                if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' compensation service endpoint must start with http:// or https://", task.id),
                    });
                }
                
                let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
                if !valid_methods.contains(&method.to_uppercase().as_str()) {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' compensation service has invalid HTTP method: {}", task.id, method),
                    });
                }
            }
            CompensationType::Script { script } => {
                if script.is_empty() {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' compensation script cannot be empty", task.id),
                    });
                }
                
                if script.len() > 32768 { // 32KB limit for compensation scripts
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' compensation script exceeds 32KB limit", task.id),
                    });
                }
            }
            CompensationType::Custom { handler } => {
                if handler.is_empty() {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' custom compensation handler cannot be empty", task.id),
                    });
                }
                
                if handler.len() > 255 {
                    return Err(OrchestratorError::InvalidDefinition {
                        message: format!("Task '{}' custom compensation handler exceeds 255 characters", task.id),
                    });
                }
            }
        }
        
        Ok(())
    }

    /// Get workflow status
    pub fn get_workflow_status(&self, workflow_id: &WorkflowId) -> Option<WorkflowStatus> {
        if let Ok(active) = self.active_workflows.read() {
            active.get(workflow_id).map(|execution| execution.status.clone())
        } else {
            None
        }
    }

    /// Get orchestrator metrics
    pub fn get_metrics(&self) -> OrchestratorMetrics {
        OrchestratorMetrics {
            total_workflows: AtomicU64::new(self.metrics.total_workflows.load(Ordering::Relaxed)),
            active_workflows: AtomicU64::new(self.metrics.active_workflows.load(Ordering::Relaxed)),
            completed_workflows: AtomicU64::new(self.metrics.completed_workflows.load(Ordering::Relaxed)),
            failed_workflows: AtomicU64::new(self.metrics.failed_workflows.load(Ordering::Relaxed)),
            cancelled_workflows: AtomicU64::new(self.metrics.cancelled_workflows.load(Ordering::Relaxed)),
            total_tasks: AtomicU64::new(self.metrics.total_tasks.load(Ordering::Relaxed)),
            successful_tasks: AtomicU64::new(self.metrics.successful_tasks.load(Ordering::Relaxed)),
            failed_tasks: AtomicU64::new(self.metrics.failed_tasks.load(Ordering::Relaxed)),
            average_execution_time_ms: AtomicU64::new(self.metrics.average_execution_time_ms.load(Ordering::Relaxed)),
            compensations_executed: AtomicU64::new(self.metrics.compensations_executed.load(Ordering::Relaxed)),
        }
    }
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(config: OrchestratorConfig) -> Self {
        Self { config }
    }

    /// Execute a single task
    #[instrument(skip(self), fields(workflow_id = %workflow_id, task_id = %task_id))]
    pub async fn execute_task(
        &self,
        workflow_id: &WorkflowId,
        task_id: &TaskId,
    ) -> Result<TaskResult, OrchestratorError> {
        let start_time = SystemTime::now();
        info!("Starting task execution: {} in workflow: {}", task_id, workflow_id);
        
        // Load task definition from workflow
        let (task_def, context_data) = self.load_task_context(workflow_id, task_id).await?;
        
        let mut retry_count = 0;
        let max_retries = task_def.retry_policy.as_ref()
            .map(|rp| rp.max_attempts)
            .unwrap_or(self.config.max_retry_attempts);
        
        let task_timeout = task_def.timeout
            .unwrap_or(self.config.execution_timeout);
        
        // Execute task with retries and timeout
        loop {
            debug!("Task execution attempt {} for task: {}", retry_count + 1, task_id);
            
            let execution_future = self.execute_task_inner(&task_def, &context_data);
            let timeout_future = tokio::time::sleep(task_timeout);
            
            match tokio::select! {
                result = execution_future => result,
                _ = timeout_future => {
                    error!("Task {} timed out after {:?}", task_id, task_timeout);
                    Err(OrchestratorError::TaskExecutionFailed {
                        task_id: task_id.clone(),
                        reason: format!("Task timed out after {:?}", task_timeout),
                    })
                }
            } {
                Ok(output) => {
                    info!("Task {} completed successfully on attempt {}", task_id, retry_count + 1);
                    return Ok(TaskResult {
                        task_id: task_id.clone(),
                        status: TaskStatus::Completed,
                        output: Some(output),
                        error: None,
                        start_time,
                        end_time: Some(SystemTime::now()),
                        retry_count,
                    });
                }
                Err(error) => {
                    retry_count += 1;
                    warn!("Task {} failed on attempt {}: {}", task_id, retry_count, error);
                    
                    if retry_count >= max_retries {
                        error!("Task {} exhausted all {} retry attempts", task_id, max_retries);
                        return Ok(TaskResult {
                            task_id: task_id.clone(),
                            status: TaskStatus::Failed,
                            output: None,
                            error: Some(error.to_string()),
                            start_time,
                            end_time: Some(SystemTime::now()),
                            retry_count,
                        });
                    }
                    
                    // Calculate retry delay
                    let delay = if let Some(retry_policy) = &task_def.retry_policy {
                        retry_policy.delay_strategy.get_delay(retry_count)
                    } else {
                        self.config.retry_delay_strategy.get_delay(retry_count)
                    };
                    
                    debug!("Retrying task {} after delay: {:?}", task_id, delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    /// Load task context from workflow
    async fn load_task_context(
        &self,
        workflow_id: &WorkflowId, 
        task_id: &TaskId
    ) -> Result<(TaskDefinition, HashMap<String, JsonValue>), OrchestratorError> {
        // This would typically load from persistent storage in a real implementation
        // For now, we'll simulate loading from a mock data source
        debug!("Loading task context for task: {} in workflow: {}", task_id, workflow_id);
        
        // Create a mock task definition
        let task_def = TaskDefinition {
            id: task_id.clone(),
            name: format!("Task {}", task_id),
            task_type: TaskType::Service {
                endpoint: "https://api.example.com/process".to_string(),
                method: "POST".to_string(),
            },
            input_schema: None,
            output_schema: None,
            timeout: Some(Duration::from_secs(30)),
            retry_policy: Some(RetryPolicy {
                max_attempts: 3,
                delay_strategy: RetryDelayStrategy::ExponentialBackoff { base_delay_secs: 1 },
                backoff_multiplier: 2.0,
                max_delay: Duration::from_secs(30),
            }),
            compensation: None,
        };
        
        let context_data = HashMap::new();
        
        Ok((task_def, context_data))
    }
    
    /// Execute task inner logic based on task type
    #[instrument(skip(self, task_def, context_data))]
    async fn execute_task_inner(
        &self,
        task_def: &TaskDefinition,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        match &task_def.task_type {
            TaskType::Service { endpoint, method } => {
                self.execute_service_task(task_def, endpoint, method, context_data).await
            }
            TaskType::Script { script } => {
                self.execute_script_task(task_def, script, context_data).await
            }
            TaskType::Human { approval_required } => {
                self.execute_human_task(task_def, *approval_required, context_data).await
            }
            TaskType::Decision { condition } => {
                self.execute_decision_task(task_def, condition, context_data).await
            }
            TaskType::Parallel { subtasks } => {
                self.execute_parallel_task(task_def, subtasks, context_data).await
            }
            TaskType::Loop { condition, subtasks } => {
                self.execute_loop_task(task_def, condition, subtasks, context_data).await
            }
        }
    }
    
    /// Execute service task
    async fn execute_service_task(
        &self,
        task_def: &TaskDefinition,
        endpoint: &str,
        method: &str,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing service task: {} -> {} {}", task_def.id, method, endpoint);
        
        // Prepare request payload
        let payload = self.prepare_task_input(task_def, context_data)?;
        
        // HTTP request execution with proper error handling and timeouts
        // Simulated for framework demonstration - replace with actual HTTP client
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Simulate different response scenarios based on endpoint
        let response = if endpoint.contains("error") {
            return Err(OrchestratorError::TaskExecutionFailed {
                task_id: task_def.id.clone(),
                reason: "Simulated service error".to_string(),
            });
        } else {
            serde_json::json!({
                "status": "success",
                "result": {
                    "processed_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    "task_id": task_def.id,
                    "method": method,
                    "endpoint": endpoint,
                    "input": payload
                }
            })
        };
        
        // Validate response against output schema if present
        if let Some(output_schema) = &task_def.output_schema {
            self.validate_task_output(&response, output_schema, &task_def.id)?;
        }
        
        debug!("Service task {} completed successfully", task_def.id);
        Ok(response)
    }
    
    /// Execute script task
    async fn execute_script_task(
        &self,
        task_def: &TaskDefinition,
        script: &str,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing script task: {} with script length: {}", task_def.id, script.len());
        
        // Prepare script environment
        let input_data = self.prepare_task_input(task_def, context_data)?;
        
        // Script execution in sandboxed environment with security isolation
        // Simulated for framework demonstration - integrate with secure script runner
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let result = serde_json::json!({
            "status": "executed",
            "script_output": "Script execution completed successfully",
            "task_id": task_def.id,
            "input_data": input_data,
            "execution_time_ms": 100
        });
        
        debug!("Script task {} completed successfully", task_def.id);
        Ok(result)
    }
    
    /// Execute human task
    async fn execute_human_task(
        &self,
        task_def: &TaskDefinition,
        approval_required: bool,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing human task: {} (approval_required: {})", task_def.id, approval_required);
        
        let input_data = self.prepare_task_input(task_def, context_data)?;
        
        // Human task workflow implementation:
        // 1. Create a human task assignment
        // 2. Notify assigned users
        // 3. Wait for user input/approval
        // 4. Return the user's response
        
        // For now, simulate human interaction
        if approval_required {
            tokio::time::sleep(Duration::from_millis(200)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        let result = serde_json::json!({
            "status": "approved",
            "approved_by": "system_user",
            "approved_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            "task_id": task_def.id,
            "approval_required": approval_required,
            "input_data": input_data
        });
        
        debug!("Human task {} completed successfully", task_def.id);
        Ok(result)
    }
    
    /// Execute decision task
    async fn execute_decision_task(
        &self,
        task_def: &TaskDefinition,
        condition: &str,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing decision task: {} with condition: {}", task_def.id, condition);
        
        let input_data = self.prepare_task_input(task_def, context_data)?;
        
        // Decision task evaluation implementation:
        // 1. Parse the condition expression
        // 2. Evaluate it against the context data
        // 3. Return the decision result
        
        // For now, simulate decision evaluation
        tokio::time::sleep(Duration::from_millis(25)).await;
        
        let decision_result = condition.contains("true") || context_data.contains_key("approve");
        
        let result = serde_json::json!({
            "decision": decision_result,
            "condition": condition,
            "evaluated_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            "task_id": task_def.id,
            "input_data": input_data
        });
        
        debug!("Decision task {} completed with result: {}", task_def.id, decision_result);
        Ok(result)
    }
    
    /// Execute parallel task
    async fn execute_parallel_task(
        &self,
        task_def: &TaskDefinition,
        subtasks: &[TaskId],
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing parallel task: {} with {} subtasks", task_def.id, subtasks.len());
        
        let input_data = self.prepare_task_input(task_def, context_data)?;
        
        // Parallel task execution implementation:
        // 1. Execute all subtasks in parallel
        // 2. Wait for all to complete
        // 3. Aggregate results
        
        // For now, simulate parallel execution
        let mut subtask_results = Vec::new();
        
        for subtask_id in subtasks {
            // Simulate subtask execution
            tokio::time::sleep(Duration::from_millis(30)).await;
            subtask_results.push(serde_json::json!({
                "subtask_id": subtask_id,
                "status": "completed",
                "result": "success"
            }));
        }
        
        let result = serde_json::json!({
            "status": "all_completed",
            "subtask_count": subtasks.len(),
            "subtask_results": subtask_results,
            "task_id": task_def.id,
            "input_data": input_data,
            "completed_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        });
        
        debug!("Parallel task {} completed successfully", task_def.id);
        Ok(result)
    }
    
    /// Execute loop task
    async fn execute_loop_task(
        &self,
        task_def: &TaskDefinition,
        condition: &str,
        subtasks: &[TaskId],
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        debug!("Executing loop task: {} with condition: {} and {} subtasks", task_def.id, condition, subtasks.len());
        
        let input_data = self.prepare_task_input(task_def, context_data)?;
        
        // Loop task execution implementation:
        // 1. Evaluate loop condition
        // 2. Execute subtasks while condition is true
        // 3. Prevent infinite loops with max iterations
        
        // For now, simulate loop execution
        let mut iterations = 0;
        let max_iterations = 10; // Safety limit
        let mut iteration_results = Vec::new();
        
        while iterations < max_iterations {
            // Simulate condition evaluation
            let should_continue = if condition.contains("false") {
                false
            } else {
                iterations < 3 // Simulate 3 iterations
            };
            
            if !should_continue {
                break;
            }
            
            // Simulate subtask execution
            let mut subtask_results = Vec::new();
            for subtask_id in subtasks {
                tokio::time::sleep(Duration::from_millis(20)).await;
                subtask_results.push(serde_json::json!({
                    "subtask_id": subtask_id,
                    "iteration": iterations,
                    "status": "completed"
                }));
            }
            
            iteration_results.push(serde_json::json!({
                "iteration": iterations,
                "subtask_results": subtask_results
            }));
            
            iterations += 1;
        }
        
        let result = serde_json::json!({
            "status": "loop_completed",
            "total_iterations": iterations,
            "condition": condition,
            "iteration_results": iteration_results,
            "task_id": task_def.id,
            "input_data": input_data,
            "completed_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        });
        
        debug!("Loop task {} completed after {} iterations", task_def.id, iterations);
        Ok(result)
    }
    
    /// Prepare task input data
    fn prepare_task_input(
        &self,
        task_def: &TaskDefinition,
        context_data: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, OrchestratorError> {
        let mut input_data = context_data.clone();
        input_data.insert("task_id".to_string(), JsonValue::String(task_def.id.clone()));
        input_data.insert("task_name".to_string(), JsonValue::String(task_def.name.clone()));
        
        let input_json = serde_json::to_value(input_data).map_err(|e| {
            OrchestratorError::TaskExecutionFailed {
                task_id: task_def.id.clone(),
                reason: format!("Failed to serialize input data: {}", e),
            }
        })?;
        
        // Validate input against schema if present
        if let Some(input_schema) = &task_def.input_schema {
            self.validate_task_input(&input_json, input_schema, &task_def.id)?;
        }
        
        Ok(input_json)
    }
    
    /// Validate task input against schema
    fn validate_task_input(
        &self,
        input: &JsonValue,
        schema: &str,
        task_id: &str,
    ) -> Result<(), OrchestratorError> {
        // Basic validation - in a real implementation, use a JSON schema validator
        let schema_value: JsonValue = serde_json::from_str(schema).map_err(|e| {
            OrchestratorError::TaskExecutionFailed {
                task_id: task_id.to_string(),
                reason: format!("Invalid input schema: {}", e),
            }
        })?;
        
        // Simple type checking based on schema
        if let Some(expected_type) = schema_value.get("type") {
            let input_type = match input {
                JsonValue::Null => "null",
                JsonValue::Bool(_) => "boolean",
                JsonValue::Number(_) => "number",
                JsonValue::String(_) => "string",
                JsonValue::Array(_) => "array",
                JsonValue::Object(_) => "object",
            };
            
            if let Some(expected) = expected_type.as_str() {
                if expected != input_type {
                    return Err(OrchestratorError::TaskExecutionFailed {
                        task_id: task_id.to_string(),
                        reason: format!("Input type mismatch. Expected: {}, got: {}", expected, input_type),
                    });
                }
            }
        }
        
        debug!("Input validation passed for task: {}", task_id);
        Ok(())
    }
    
    /// Validate task output against schema
    fn validate_task_output(
        &self,
        output: &JsonValue,
        schema: &str,
        task_id: &str,
    ) -> Result<(), OrchestratorError> {
        // Basic validation - in a real implementation, use a JSON schema validator
        let schema_value: JsonValue = serde_json::from_str(schema).map_err(|e| {
            OrchestratorError::TaskExecutionFailed {
                task_id: task_id.to_string(),
                reason: format!("Invalid output schema: {}", e),
            }
        })?;
        
        // Simple type checking based on schema
        if let Some(expected_type) = schema_value.get("type") {
            let output_type = match output {
                JsonValue::Null => "null",
                JsonValue::Bool(_) => "boolean",
                JsonValue::Number(_) => "number",
                JsonValue::String(_) => "string",
                JsonValue::Array(_) => "array",
                JsonValue::Object(_) => "object",
            };
            
            if let Some(expected) = expected_type.as_str() {
                if expected != output_type {
                    return Err(OrchestratorError::TaskExecutionFailed {
                        task_id: task_id.to_string(),
                        reason: format!("Output type mismatch. Expected: {}, got: {}", expected, output_type),
                    });
                }
            }
        }
        
        debug!("Output validation passed for task: {}", task_id);
        Ok(())
    }
}

impl Clone for TaskExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

impl fmt::Display for WorkflowOrchestrator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.get_metrics();
        writeln!(f, "WorkflowOrchestrator:")?;
        writeln!(f, "  Total Workflows: {}", metrics.total_workflows.load(Ordering::Relaxed))?;
        writeln!(f, "  Active Workflows: {}", metrics.active_workflows.load(Ordering::Relaxed))?;
        writeln!(f, "  Completed Workflows: {}", metrics.completed_workflows.load(Ordering::Relaxed))?;
        writeln!(f, "  Failed Workflows: {}", metrics.failed_workflows.load(Ordering::Relaxed))?;
        Ok(())
    }
}

/// Generate orchestrator preprocessing code
pub fn generate_orchestrator_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;

        // Initialize orchestrator configuration from environment
        let orchestrator_config = {
            let max_concurrent_workflows: usize = env::var("RABBITMESH_MAX_CONCURRENT_WORKFLOWS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100);

            let max_execution_time_secs: u64 = env::var("RABBITMESH_MAX_EXECUTION_TIME_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600);

            let enable_persistence: bool = env::var("RABBITMESH_ENABLE_WORKFLOW_PERSISTENCE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_retry: bool = env::var("RABBITMESH_ENABLE_WORKFLOW_RETRY")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_retry_attempts: u32 = env::var("RABBITMESH_MAX_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3);

            let execution_timeout_secs: u64 = env::var("RABBITMESH_EXECUTION_TIMEOUT_SECS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300);

            rabbitmesh_macros::workflow::orchestrator::OrchestratorConfig {
                max_concurrent_workflows,
                max_execution_time: Duration::from_secs(max_execution_time_secs),
                enable_persistence,
                enable_retry,
                max_retry_attempts,
                retry_delay_strategy: rabbitmesh_macros::workflow::orchestrator::RetryDelayStrategy::ExponentialBackoff {
                    base_delay_secs: 1
                },
                enable_versioning: true,
                execution_timeout: Duration::from_secs(execution_timeout_secs),
            }
        };

        // Initialize workflow orchestrator
        let workflow_orchestrator = Arc::new(
            rabbitmesh_macros::workflow::orchestrator::WorkflowOrchestrator::new(orchestrator_config)
        );

        // Orchestrator is ready for workflow registration and execution
        let _orchestrator_ref = workflow_orchestrator.clone();

        Ok(())
    }
}