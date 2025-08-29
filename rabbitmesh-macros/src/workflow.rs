//! Universal Workflow and State Management Macros for RabbitMesh
//! 
//! This module provides comprehensive workflow and state management macros that work
//! across ALL project types and business processes:
//!
//! **Project Types Supported:**
//! - 🛒 E-commerce (order processing, inventory management, customer onboarding)
//! - 💰 Finance (loan approval, trading workflows, compliance processes)
//! - 🏥 Healthcare (patient care workflows, appointment scheduling, treatment protocols)
//! - 🏭 IoT (device provisioning, data processing pipelines, alert workflows)
//! - 🎮 Gaming (player onboarding, match workflows, tournament management)
//! - 📱 Social Media (content moderation, user verification, engagement campaigns)
//! - 🏢 Enterprise (employee onboarding, approval processes, document workflows)
//! - 🎓 Education (enrollment processes, grading workflows, course management)
//! - 📦 Logistics (shipment tracking, route optimization, delivery workflows)
//! - 🌍 GIS/Mapping (data processing pipelines, spatial analysis workflows)
//!
//! **Workflow Patterns:**
//! - State Machines (FSM, Hierarchical, Parallel)
//! - Saga Patterns (Orchestration, Choreography)
//! - Approval Workflows (Sequential, Parallel, Conditional)
//! - Event Sourcing (Event streams, Projections, Snapshots)
//! - CQRS (Command/Query separation, Event handlers)
//! - Workflow Orchestration (BPMN, Temporal, Cadence)

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Attribute, LitStr, LitInt, LitBool};
use std::collections::HashMap;
use crate::universal::{MacroAttribute, MacroValue, MacroContext};

/// Universal workflow and state management processor
pub struct WorkflowProcessor;

impl WorkflowProcessor {
    /// Generate workflow logic for any macro attribute
    pub fn generate_workflow_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        match attr.name.as_str() {
            "state_machine" => Self::generate_state_machine_logic(attr, context),
            "saga" => Self::generate_saga_logic(attr, context),
            "workflow" => Self::generate_workflow_logic_base(attr, context),
            "approval_required" => Self::generate_approval_workflow(attr, context),
            "event_sourced" => Self::generate_event_sourcing(attr, context),
            "cqrs" => Self::generate_cqrs_logic(attr, context),
            "projection" => Self::generate_projection_logic(attr, context),
            "state_transition" => Self::generate_state_transition(attr, context),
            "workflow_step" => Self::generate_workflow_step(attr, context),
            "parallel_execution" => Self::generate_parallel_execution(attr, context),
            "conditional_flow" => Self::generate_conditional_flow(attr, context),
            "retry_policy" => Self::generate_retry_policy(attr, context),
            "timeout_handler" => Self::generate_timeout_handler(attr, context),
            _ => quote! {}
        }
    }

    /// Generate state machine logic
    fn generate_state_machine_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let states = Self::extract_states(attr);
        let initial_state = Self::extract_string_arg(attr, "initial").unwrap_or_else(|| "initial".to_string());
        let transitions = Self::extract_transitions(attr);
        let state_guards = Self::extract_state_guards(attr);
        let domain_states = Self::generate_domain_state_machine(context);
        
        quote! {
            // Universal state machine
            debug!("🔄 Initializing state machine");
            
            let state_machine = rabbitmesh_workflow::StateMachine::new()
                .with_initial_state(#initial_state)
                .with_states(vec![#(#states),*])
                .with_context(#context.service_name, #context.method_name);
            
            #domain_states
            
            // Configure transitions
            let transitions = vec![#(#transitions),*];
            for transition in transitions {
                state_machine.add_transition(transition).await;
            }
            
            // Configure state guards
            let guards = vec![#(#state_guards),*];
            for guard in guards {
                state_machine.add_guard(guard).await;
            }
            
            // Load current state from message or persistence
            let current_state = rabbitmesh_workflow::load_state(&msg, &state_machine).await?;
            debug!("🔄 Current state: {}", current_state);
            
            // Validate state transition is allowed
            let target_state = #context.method_name; // Method represents target state
            if !state_machine.can_transition(&current_state, target_state).await {
                return Ok(rabbitmesh::RpcResponse::error(&format!("Invalid state transition from {} to {}", current_state, target_state)));
            }
        }
    }

    /// Generate saga pattern logic
    fn generate_saga_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let saga_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| format!("{}_{}_saga", context.service_name, context.method_name));
        let saga_type = Self::extract_string_arg(attr, "type").unwrap_or_else(|| "orchestration".to_string());
        let compensation_handler = Self::extract_string_arg(attr, "compensate");
        let saga_steps = Self::extract_saga_steps(attr);
        let domain_saga = Self::generate_domain_saga(context);
        
        quote! {
            // Universal saga pattern
            debug!("⚔️ Initializing saga: {} (type: {})", #saga_name, #saga_type);
            
            let saga = rabbitmesh_workflow::Saga::new(#saga_name)
                .with_type(#saga_type)
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            #domain_saga
            
            // Configure saga steps
            let saga_steps = vec![#(#saga_steps),*];
            for step in saga_steps {
                saga.add_step(step).await;
            }
            
            // Configure compensation handler
            if let Some(compensate_fn) = #compensation_handler {
                saga.set_compensation_handler_name(compensate_fn).await;
            }
            
            // Register saga for distributed coordination
            let saga_id = rabbitmesh_workflow::register_saga(&saga, &msg).await?;
            debug!("⚔️ Saga registered with ID: {}", saga_id);
            
            // Execute saga will be handled after method completion
            let saga_execution_enabled = true;
        }
    }

    /// Generate base workflow logic
    fn generate_workflow_logic_base(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let workflow_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| format!("{}_{}_workflow", context.service_name, context.method_name));
        let workflow_type = Self::extract_string_arg(attr, "type").unwrap_or_else(|| "sequential".to_string());
        let workflow_steps = Self::extract_workflow_steps(attr);
        let parallel_branches = Self::extract_parallel_branches(attr);
        let domain_workflow = Self::generate_domain_workflow(context);
        
        quote! {
            // Universal workflow orchestration
            debug!("🎭 Starting workflow: {} (type: {})", #workflow_name, #workflow_type);
            
            let workflow = rabbitmesh_workflow::Workflow::new(#workflow_name)
                .with_type(#workflow_type)
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            #domain_workflow
            
            // Configure workflow steps
            match #workflow_type.as_str() {
                "sequential" => {
                    let steps = vec![#(#workflow_steps),*];
                    workflow.set_sequential_steps(steps).await;
                },
                "parallel" => {
                    let branches = vec![#(#parallel_branches),*];
                    workflow.set_parallel_branches(branches).await;
                },
                "conditional" => {
                    // Conditional logic will be determined at runtime
                    workflow.enable_conditional_execution().await;
                },
                _ => {
                    warn!("⚠️ Unknown workflow type: {}", #workflow_type);
                }
            }
            
            // Start workflow execution
            let workflow_id = rabbitmesh_workflow::start_workflow(&workflow, &msg).await?;
            debug!("🎭 Workflow started with ID: {}", workflow_id);
        }
    }

    /// Generate approval workflow
    fn generate_approval_workflow(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let approval_threshold = Self::extract_approval_threshold(attr);
        let approver_roles = Self::extract_approver_roles(attr);
        let auto_approve_conditions = Self::extract_auto_approve_conditions(attr);
        let escalation_rules = Self::extract_escalation_rules(attr);
        let domain_approval = Self::generate_domain_approval(context);
        
        quote! {
            // Universal approval workflow
            debug!("✅ Setting up approval workflow");
            
            let approval_workflow = rabbitmesh_workflow::ApprovalWorkflow::new()
                .with_service(#context.service_name)
                .with_method(#context.method_name)
                .with_threshold_condition(#approval_threshold);
            
            #domain_approval
            
            // Configure approver roles
            let approver_roles = vec![#(#approver_roles),*];
            approval_workflow.set_approver_roles(approver_roles).await;
            
            // Configure auto-approval conditions
            let auto_approve_conditions = vec![#(#auto_approve_conditions),*];
            for condition in auto_approve_conditions {
                if rabbitmesh_workflow::evaluate_condition(&msg, &condition).await? {
                    debug!("✅ Auto-approval condition met: {}", condition);
                    return Ok(rabbitmesh::RpcResponse::success("Auto-approved".to_string(), 0)?);
                }
            }
            
            // Configure escalation rules
            let escalation_rules = vec![#(#escalation_rules),*];
            approval_workflow.set_escalation_rules(escalation_rules).await;
            
            // Create approval request
            let approval_request = rabbitmesh_workflow::ApprovalRequest::new()
                .with_workflow_id(approval_workflow.get_id())
                .with_request_data(&msg)
                .with_timestamp(chrono::Utc::now());
            
            // Submit for approval (method execution will be paused)
            let approval_id = rabbitmesh_workflow::submit_for_approval(approval_request).await?;
            debug!("✅ Submitted for approval with ID: {}", approval_id);
            
            // Return pending response
            return Ok(rabbitmesh::RpcResponse::success(
                format!("Request submitted for approval. Approval ID: {}", approval_id), 
                202 // HTTP 202 Accepted
            )?);
        }
    }

    /// Generate event sourcing logic
    fn generate_event_sourcing(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let event_store = Self::extract_string_arg(attr, "store").unwrap_or_else(|| "default".to_string());
        let aggregate_type = Self::extract_string_arg(attr, "aggregate").unwrap_or_else(|| context.service_name.clone());
        let event_version = Self::extract_int_arg(attr, "version").unwrap_or(1);
        let snapshots_enabled = Self::extract_bool_arg(attr, "snapshots").unwrap_or(false);
        let domain_events = Self::generate_domain_events(context);
        
        quote! {
            // Universal event sourcing
            debug!("📚 Setting up event sourcing (store: {}, aggregate: {})", #event_store, #aggregate_type);
            
            let event_store = rabbitmesh_workflow::EventStore::new(#event_store)
                .with_aggregate_type(#aggregate_type)
                .with_version(#event_version as u64);
            
            #domain_events
            
            // Load aggregate from events
            let aggregate_id = msg.get_path_param("id")
                .or_else(|| msg.get_header("aggregate_id"))
                .unwrap_or_else(|| "unknown".to_string());
            
            let mut aggregate = if #snapshots_enabled {
                // Load from snapshot first, then replay events
                rabbitmesh_workflow::load_aggregate_from_snapshot(&event_store, &aggregate_id).await?
                    .unwrap_or_else(|| rabbitmesh_workflow::create_new_aggregate(#aggregate_type, &aggregate_id))
            } else {
                // Replay all events from beginning
                rabbitmesh_workflow::load_aggregate_from_events(&event_store, &aggregate_id).await?
                    .unwrap_or_else(|| rabbitmesh_workflow::create_new_aggregate(#aggregate_type, &aggregate_id))
            };
            
            debug!("📚 Aggregate loaded: {} (version: {})", aggregate_id, aggregate.version());
            
            // Events will be generated and stored after method execution
            let event_sourcing_enabled = true;
        }
    }

    /// Generate CQRS logic
    fn generate_cqrs_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let command_type = Self::extract_string_arg(attr, "command");
        let query_type = Self::extract_string_arg(attr, "query");
        let event_handlers = Self::extract_event_handlers(attr);
        let read_models = Self::extract_read_models(attr);
        let domain_cqrs = Self::generate_domain_cqrs(context);
        
        quote! {
            // Universal CQRS pattern
            debug!("🎯 Setting up CQRS");
            
            let cqrs_handler = rabbitmesh_workflow::CQRSHandler::new()
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            #domain_cqrs
            
            // Determine if this is a command or query
            let is_command = #command_type.is_some() || msg.is_mutating_operation();
            let is_query = #query_type.is_some() || !msg.is_mutating_operation();
            
            if is_command {
                debug!("🎯 Processing as command");
                
                // Validate command
                let command_type = #command_type.unwrap_or_else(|| #context.method_name.to_string());
                let command = rabbitmesh_workflow::Command::new(command_type)
                    .with_payload(&msg)
                    .with_timestamp(chrono::Utc::now());
                
                // Process command (will generate events)
                cqrs_handler.process_command(command).await?;
                
            } else if is_query {
                debug!("🎯 Processing as query");
                
                // Process query from read model
                let query_type = #query_type.unwrap_or_else(|| #context.method_name.to_string());
                let query = rabbitmesh_workflow::Query::new(query_type)
                    .with_parameters(&msg)
                    .with_timestamp(chrono::Utc::now());
                
                // Execute query against read model
                let query_result = cqrs_handler.process_query(query).await?;
                return Ok(rabbitmesh::RpcResponse::success(query_result, 0)?);
            }
            
            // Configure event handlers for read model updates
            let event_handlers = vec![#(#event_handlers),*];
            for handler in event_handlers {
                cqrs_handler.register_event_handler(handler).await;
            }
            
            // Configure read models
            let read_models = vec![#(#read_models),*];
            for model in read_models {
                cqrs_handler.register_read_model(model).await;
            }
        }
    }

    /// Generate projection logic
    fn generate_projection_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let projection_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| format!("{}_{}_projection", context.service_name, context.method_name));
        let event_types = Self::extract_event_types(attr);
        let projection_store = Self::extract_string_arg(attr, "store").unwrap_or_else(|| "default".to_string());
        let update_strategy = Self::extract_string_arg(attr, "strategy").unwrap_or_else(|| "incremental".to_string());
        
        quote! {
            // Universal event projection
            debug!("📊 Setting up projection: {} (strategy: {})", #projection_name, #update_strategy);
            
            let projection = rabbitmesh_workflow::Projection::new(#projection_name)
                .with_store(#projection_store)
                .with_update_strategy(#update_strategy);
            
            // Configure event types to project
            let event_types = vec![#(#event_types),*];
            for event_type in event_types {
                projection.subscribe_to_event_type(event_type).await;
            }
            
            // Process projection update
            match #update_strategy.as_str() {
                "incremental" => {
                    projection.update_incremental(&msg).await?;
                },
                "snapshot" => {
                    projection.create_snapshot(&msg).await?;
                },
                "rebuild" => {
                    projection.rebuild_from_events().await?;
                },
                _ => {
                    warn!("⚠️ Unknown projection strategy: {}", #update_strategy);
                }
            }
            
            debug!("📊 Projection updated: {}", #projection_name);
        }
    }

    /// Generate state transition logic
    fn generate_state_transition(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let from_state = Self::extract_string_arg(attr, "from").unwrap_or_else(|| "any".to_string());
        let to_state = Self::extract_string_arg(attr, "to").unwrap_or_else(|| "next".to_string());
        let resource_type = Self::extract_string_arg(attr, "resource");
        let guard_conditions = Self::extract_guard_conditions(attr);
        let domain_transition = Self::generate_domain_state_transition(context);
        
        quote! {
            // Universal state transition
            debug!("🔄 Processing state transition: {} -> {}", #from_state, #to_state);
            
            #domain_transition
            
            // Load current resource state
            let resource_id = msg.get_path_param("id")
                .or_else(|| msg.get_header("resource_id"))
                .unwrap_or_else(|| "unknown".to_string());
                
            let current_state = if let Some(resource_type) = #resource_type {
                rabbitmesh_workflow::load_resource_state(&resource_id, resource_type).await?
            } else {
                rabbitmesh_workflow::load_generic_state(&resource_id).await?
            };
            
            // Validate current state matches expected 'from' state
            if #from_state != "any" && current_state != #from_state {
                return Ok(rabbitmesh::RpcResponse::error(&format!(
                    "Invalid state transition: expected state '{}', found '{}'", 
                    #from_state, current_state
                )));
            }
            
            // Evaluate guard conditions
            let guard_conditions = vec![#(#guard_conditions),*];
            for condition in guard_conditions {
                if !rabbitmesh_workflow::evaluate_guard_condition(&msg, &condition).await? {
                    return Ok(rabbitmesh::RpcResponse::error(&format!(
                        "State transition guard failed: {}", condition
                    )));
                }
            }
            
            // State transition will be committed after successful method execution
            let pending_state_transition = (#to_state.to_string(), resource_id);
        }
    }

    /// Generate workflow step logic
    fn generate_workflow_step(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let step_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| context.method_name.clone());
        let step_order = Self::extract_int_arg(attr, "order").unwrap_or(1);
        let depends_on = Self::extract_dependencies(attr);
        let retry_policy = Self::extract_step_retry_policy(attr);
        
        quote! {
            // Universal workflow step
            debug!("🪜 Executing workflow step: {} (order: {})", #step_name, #step_order);
            
            let workflow_step = rabbitmesh_workflow::WorkflowStep::new(#step_name)
                .with_order(#step_order as u32)
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            // Check dependencies
            let dependencies = vec![#(#depends_on),*];
            for dependency in dependencies {
                if !rabbitmesh_workflow::check_step_completed(&dependency).await? {
                    return Ok(rabbitmesh::RpcResponse::error(&format!(
                        "Workflow step dependency not met: {}", dependency
                    )));
                }
            }
            
            // Configure retry policy
            if let Some(retry_config) = #retry_policy {
                workflow_step.set_retry_policy(retry_config).await;
            }
            
            // Mark step as started
            rabbitmesh_workflow::mark_step_started(&workflow_step).await?;
            
            // Step completion will be marked after method execution
            let workflow_step_tracking = workflow_step;
        }
    }

    /// Generate parallel execution logic
    fn generate_parallel_execution(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let max_concurrency = Self::extract_int_arg(attr, "max_concurrency").unwrap_or(10);
        let wait_for_all = Self::extract_bool_arg(attr, "wait_for_all").unwrap_or(true);
        let failure_strategy = Self::extract_string_arg(attr, "failure_strategy").unwrap_or_else(|| "fail_fast".to_string());
        let parallel_tasks = Self::extract_parallel_tasks(attr);
        
        quote! {
            // Universal parallel execution
            debug!("⚡ Setting up parallel execution (max_concurrency: {}, wait_for_all: {})", 
                   #max_concurrency, #wait_for_all);
            
            let parallel_executor = rabbitmesh_workflow::ParallelExecutor::new()
                .with_max_concurrency(#max_concurrency as usize)
                .with_failure_strategy(#failure_strategy);
            
            // Configure parallel tasks
            let parallel_tasks = vec![#(#parallel_tasks),*];
            for task in parallel_tasks {
                parallel_executor.add_task(task).await;
            }
            
            // Execute parallel tasks
            let parallel_results = if #wait_for_all {
                parallel_executor.execute_and_wait_all().await?
            } else {
                parallel_executor.execute_and_wait_any().await?
            };
            
            debug!("⚡ Parallel execution completed with {} results", parallel_results.len());
            
            // Method execution will be part of parallel task coordination
            let parallel_execution_enabled = true;
        }
    }

    /// Generate conditional flow logic
    fn generate_conditional_flow(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let conditions = Self::extract_conditions(attr);
        let default_action = Self::extract_string_arg(attr, "default");
        let condition_evaluator = Self::extract_string_arg(attr, "evaluator").unwrap_or_else(|| "default".to_string());
        
        quote! {
            // Universal conditional flow
            debug!("🤔 Evaluating conditional flow");
            
            let conditional_flow = rabbitmesh_workflow::ConditionalFlow::new()
                .with_evaluator(#condition_evaluator);
            
            // Evaluate conditions in order
            let conditions = vec![#(#conditions),*];
            let mut condition_met = false;
            
            for (condition_expr, action) in conditions {
                if rabbitmesh_workflow::evaluate_expression(&msg, &condition_expr).await? {
                    debug!("🤔 Condition met: {} -> {}", condition_expr, action);
                    conditional_flow.execute_action(&action, &msg).await?;
                    condition_met = true;
                    break;
                }
            }
            
            // Execute default action if no conditions met
            if !condition_met {
                if let Some(default) = #default_action {
                    debug!("🤔 No conditions met, executing default: {}", default);
                    conditional_flow.execute_action(&default, &msg).await?;
                } else {
                    debug!("🤔 No conditions met and no default action configured");
                }
            }
        }
    }

    /// Generate retry policy logic
    fn generate_retry_policy(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let max_attempts = Self::extract_int_arg(attr, "max_attempts").unwrap_or(3);
        let initial_delay = Self::extract_int_arg(attr, "initial_delay").unwrap_or(1000);
        let backoff_strategy = Self::extract_string_arg(attr, "backoff").unwrap_or_else(|| "exponential".to_string());
        let retry_conditions = Self::extract_retry_conditions(attr);
        
        quote! {
            // Universal retry policy
            debug!("🔄 Setting up retry policy (max_attempts: {}, strategy: {})", 
                   #max_attempts, #backoff_strategy);
            
            let retry_policy = rabbitmesh_workflow::RetryPolicy::new()
                .with_max_attempts(#max_attempts as usize)
                .with_initial_delay(std::time::Duration::from_millis(#initial_delay as u64))
                .with_backoff_strategy(#backoff_strategy);
            
            // Configure retry conditions
            let retry_conditions = vec![#(#retry_conditions),*];
            for condition in retry_conditions {
                retry_policy.add_retry_condition(condition).await;
            }
            
            // Retry logic will be applied around method execution
            let retry_policy_enabled = retry_policy;
        }
    }

    /// Generate timeout handler logic
    fn generate_timeout_handler(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let timeout_duration = Self::extract_int_arg(attr, "timeout").unwrap_or(30000);
        let timeout_action = Self::extract_string_arg(attr, "action").unwrap_or_else(|| "cancel".to_string());
        let escalation_timeout = Self::extract_int_arg(attr, "escalation_timeout");
        
        quote! {
            // Universal timeout handler
            debug!("⏰ Setting up timeout handler (timeout: {}ms, action: {})", 
                   #timeout_duration, #timeout_action);
            
            let timeout_handler = rabbitmesh_workflow::TimeoutHandler::new()
                .with_timeout(std::time::Duration::from_millis(#timeout_duration as u64))
                .with_action(#timeout_action);
            
            // Configure escalation timeout if specified
            if let Some(escalation_ms) = #escalation_timeout {
                timeout_handler.with_escalation_timeout(
                    std::time::Duration::from_millis(escalation_ms as u64)
                ).await;
            }
            
            // Start timeout monitoring
            let timeout_handle = timeout_handler.start_monitoring().await?;
            
            // Timeout will be cancelled after successful method execution
            let timeout_monitoring_enabled = timeout_handle;
        }
    }

    // Domain-specific workflow generators

    /// Generate domain-specific state machine
    fn generate_domain_state_machine(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce state machine configurations
                if #method_name.contains("order") {
                    state_machine.add_states(vec![
                        "pending", "confirmed", "payment_processing", "paid", 
                        "shipped", "delivered", "cancelled", "refunded"
                    ]).await;
                    
                    state_machine.add_transition("pending", "confirmed", "confirm_order").await;
                    state_machine.add_transition("confirmed", "payment_processing", "process_payment").await;
                    state_machine.add_transition("payment_processing", "paid", "payment_successful").await;
                    state_machine.add_transition("paid", "shipped", "ship_order").await;
                    state_machine.add_transition("shipped", "delivered", "deliver_order").await;
                    
                    // Cancellation transitions
                    state_machine.add_transition("pending", "cancelled", "cancel_order").await;
                    state_machine.add_transition("confirmed", "cancelled", "cancel_order").await;
                    
                    // Refund transitions
                    state_machine.add_transition("paid", "refunded", "refund_order").await;
                    state_machine.add_transition("delivered", "refunded", "refund_order").await;
                } else if #method_name.contains("product") {
                    state_machine.add_states(vec![
                        "draft", "pending_review", "approved", "published", 
                        "out_of_stock", "discontinued"
                    ]).await;
                }
            },
            "finance" => quote! {
                // Finance state machine configurations
                if #method_name.contains("trade") || #method_name.contains("order") {
                    state_machine.add_states(vec![
                        "pending", "submitted", "partial_filled", "filled", 
                        "cancelled", "rejected", "expired"
                    ]).await;
                    
                    state_machine.add_transition("pending", "submitted", "submit_order").await;
                    state_machine.add_transition("submitted", "partial_filled", "partial_fill").await;
                    state_machine.add_transition("partial_filled", "filled", "complete_fill").await;
                    state_machine.add_transition("submitted", "cancelled", "cancel_order").await;
                } else if #method_name.contains("loan") || #method_name.contains("credit") {
                    state_machine.add_states(vec![
                        "application", "under_review", "approved", "funded", 
                        "rejected", "defaulted", "paid_off"
                    ]).await;
                }
            },
            "healthcare" => quote! {
                // Healthcare state machine configurations
                if #method_name.contains("appointment") {
                    state_machine.add_states(vec![
                        "requested", "scheduled", "confirmed", "in_progress", 
                        "completed", "cancelled", "no_show"
                    ]).await;
                } else if #method_name.contains("treatment") {
                    state_machine.add_states(vec![
                        "prescribed", "started", "in_progress", "completed", 
                        "discontinued", "modified"
                    ]).await;
                }
            },
            "iot" => quote! {
                // IoT state machine configurations  
                if #method_name.contains("device") {
                    state_machine.add_states(vec![
                        "provisioning", "active", "inactive", "maintenance", 
                        "error", "decommissioned"
                    ]).await;
                } else if #method_name.contains("sensor") {
                    state_machine.add_states(vec![
                        "calibrating", "monitoring", "alerting", "offline", "error"
                    ]).await;
                }
            },
            "gaming" => quote! {
                // Gaming state machine configurations
                if #method_name.contains("match") {
                    state_machine.add_states(vec![
                        "waiting", "starting", "in_progress", "paused", 
                        "completed", "abandoned", "cancelled"
                    ]).await;
                } else if #method_name.contains("player") {
                    state_machine.add_states(vec![
                        "registering", "active", "inactive", "suspended", "banned"
                    ]).await;
                }
            },
            "logistics" => quote! {
                // Logistics state machine configurations
                if #method_name.contains("shipment") {
                    state_machine.add_states(vec![
                        "created", "picked_up", "in_transit", "out_for_delivery", 
                        "delivered", "returned", "lost", "damaged"
                    ]).await;
                }
            },
            _ => quote! {
                // Generic state machine
                state_machine.add_states(vec!["initial", "processing", "completed", "failed"]).await;
            }
        }
    }

    /// Generate domain-specific saga
    fn generate_domain_saga(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce saga configurations
                if #method_name.contains("order") {
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("validate_inventory")
                        .with_service("inventory-service")
                        .with_compensation("release_inventory_hold")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("process_payment")
                        .with_service("payment-service")
                        .with_compensation("refund_payment")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("create_shipment")
                        .with_service("shipping-service")
                        .with_compensation("cancel_shipment")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("send_confirmation")
                        .with_service("notification-service")
                        .with_compensation("send_cancellation_notice")
                    ).await;
                }
            },
            "finance" => quote! {
                // Finance saga configurations
                if #method_name.contains("transfer") {
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("validate_source_account")
                        .with_service("account-service")
                        .with_compensation("unlock_source_account")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("validate_destination_account")
                        .with_service("account-service")
                        .with_compensation("unlock_destination_account")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("perform_transfer")
                        .with_service("transfer-service")
                        .with_compensation("reverse_transfer")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("update_balances")
                        .with_service("balance-service")
                        .with_compensation("revert_balance_updates")
                    ).await;
                }
            },
            "healthcare" => quote! {
                // Healthcare saga configurations
                if #method_name.contains("appointment") {
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("check_provider_availability")
                        .with_service("scheduling-service")
                        .with_compensation("release_time_slot")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("verify_patient_eligibility")
                        .with_service("insurance-service")
                        .with_compensation("release_authorization")
                    ).await;
                    
                    saga.add_step(rabbitmesh_workflow::SagaStep::new("book_appointment")
                        .with_service("appointment-service")
                        .with_compensation("cancel_appointment")
                    ).await;
                }
            },
            _ => quote! {
                // Generic saga steps
                debug!("🔧 Using generic saga configuration");
            }
        }
    }

    /// Generate domain-specific workflow
    fn generate_domain_workflow(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce workflow optimizations
                workflow.set_timeout(std::time::Duration::from_secs(300)).await; // 5 minutes for order processing
                workflow.enable_parallel_processing(vec!["inventory_check", "payment_validation"]).await;
            },
            "finance" => quote! {
                // Finance workflow optimizations
                workflow.set_timeout(std::time::Duration::from_secs(30)).await; // 30 seconds for trading
                workflow.enable_strict_ordering().await; // Ensure transaction ordering
                workflow.enable_audit_logging().await; // Required for compliance
            },
            "healthcare" => quote! {
                // Healthcare workflow optimizations  
                workflow.enable_audit_logging().await; // HIPAA compliance
                workflow.enable_encryption().await; // PHI protection
                workflow.set_timeout(std::time::Duration::from_secs(600)).await; // 10 minutes for medical procedures
            },
            "iot" => quote! {
                // IoT workflow optimizations
                workflow.enable_batch_processing().await; // Handle high-volume sensor data
                workflow.set_timeout(std::time::Duration::from_secs(10)).await; // Fast processing for real-time data
            },
            _ => quote! {
                // Generic workflow configuration
                workflow.set_timeout(std::time::Duration::from_secs(60)).await;
            }
        }
    }

    /// Generate domain-specific approval workflow
    fn generate_domain_approval(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "finance" => quote! {
                // Finance approval workflows
                if #method_name.contains("transfer") || #method_name.contains("payment") {
                    approval_workflow.add_approval_tier("manager", 10000.0).await; // $10K limit
                    approval_workflow.add_approval_tier("director", 100000.0).await; // $100K limit  
                    approval_workflow.add_approval_tier("cfo", 1000000.0).await; // $1M limit
                } else if #method_name.contains("trade") {
                    approval_workflow.add_approval_tier("senior_trader", 50000.0).await;
                    approval_workflow.add_approval_tier("risk_manager", 500000.0).await;
                }
            },
            "healthcare" => quote! {
                // Healthcare approval workflows
                if #method_name.contains("prescription") || #method_name.contains("treatment") {
                    approval_workflow.add_approval_tier("attending_physician", 1.0).await;
                    approval_workflow.add_approval_tier("department_head", 2.0).await; // For experimental treatments
                }
            },
            "ecommerce" => quote! {
                // E-commerce approval workflows
                if #method_name.contains("refund") || #method_name.contains("discount") {
                    approval_workflow.add_approval_tier("supervisor", 500.0).await;
                    approval_workflow.add_approval_tier("manager", 2000.0).await;
                }
            },
            _ => quote! {
                // Generic approval workflow
                approval_workflow.add_approval_tier("manager", 1000.0).await;
            }
        }
    }

    /// Generate domain-specific events
    fn generate_domain_events(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce domain events
                event_store.register_event_type("OrderCreated").await;
                event_store.register_event_type("OrderConfirmed").await;
                event_store.register_event_type("PaymentProcessed").await;
                event_store.register_event_type("OrderShipped").await;
                event_store.register_event_type("OrderDelivered").await;
                event_store.register_event_type("OrderCancelled").await;
                event_store.register_event_type("ProductUpdated").await;
                event_store.register_event_type("InventoryChanged").await;
            },
            "finance" => quote! {
                // Finance domain events
                event_store.register_event_type("TradeOrderPlaced").await;
                event_store.register_event_type("TradeExecuted").await;
                event_store.register_event_type("PortfolioUpdated").await;
                event_store.register_event_type("RiskLimitBreached").await;
                event_store.register_event_type("AccountDebited").await;
                event_store.register_event_type("AccountCredited").await;
            },
            "healthcare" => quote! {
                // Healthcare domain events
                event_store.register_event_type("PatientRegistered").await;
                event_store.register_event_type("AppointmentScheduled").await;
                event_store.register_event_type("DiagnosisRecorded").await;
                event_store.register_event_type("TreatmentStarted").await;
                event_store.register_event_type("TreatmentCompleted").await;
            },
            _ => quote! {
                // Generic domain events
                event_store.register_event_type("EntityCreated").await;
                event_store.register_event_type("EntityUpdated").await;
                event_store.register_event_type("EntityDeleted").await;
            }
        }
    }

    /// Generate domain-specific CQRS
    fn generate_domain_cqrs(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce CQRS configuration
                cqrs_handler.register_command_handler("CreateOrder", Box::new(CreateOrderHandler)).await;
                cqrs_handler.register_command_handler("UpdateInventory", Box::new(UpdateInventoryHandler)).await;
                
                cqrs_handler.register_query_handler("GetProductCatalog", Box::new(ProductCatalogQuery)).await;
                cqrs_handler.register_query_handler("GetOrderHistory", Box::new(OrderHistoryQuery)).await;
                
                cqrs_handler.register_read_model("ProductCatalogView").await;
                cqrs_handler.register_read_model("CustomerOrderView").await;
            },
            "finance" => quote! {
                // Finance CQRS configuration
                cqrs_handler.register_command_handler("PlaceTradeOrder", Box::new(PlaceTradeOrderHandler)).await;
                cqrs_handler.register_command_handler("UpdatePortfolio", Box::new(UpdatePortfolioHandler)).await;
                
                cqrs_handler.register_query_handler("GetPortfolioSummary", Box::new(PortfolioSummaryQuery)).await;
                cqrs_handler.register_query_handler("GetMarketData", Box::new(MarketDataQuery)).await;
                
                cqrs_handler.register_read_model("PortfolioView").await;
                cqrs_handler.register_read_model("MarketDataView").await;
            },
            _ => quote! {
                // Generic CQRS configuration
                debug!("🔧 Using generic CQRS configuration");
            }
        }
    }

    /// Generate domain-specific state transition
    fn generate_domain_state_transition(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce state transition validations
                if #method_name.contains("ship_order") {
                    // Additional validation for shipping
                    if !rabbitmesh_workflow::validate_shipping_address(&msg).await? {
                        return Ok(rabbitmesh::RpcResponse::error("Invalid shipping address"));
                    }
                    if !rabbitmesh_workflow::validate_inventory_availability(&msg).await? {
                        return Ok(rabbitmesh::RpcResponse::error("Insufficient inventory"));
                    }
                }
            },
            "finance" => quote! {
                // Finance state transition validations
                if #method_name.contains("execute_trade") {
                    // Additional validation for trade execution
                    if !rabbitmesh_workflow::validate_market_hours().await? {
                        return Ok(rabbitmesh::RpcResponse::error("Market is closed"));
                    }
                    if !rabbitmesh_workflow::validate_risk_limits(&msg).await? {
                        return Ok(rabbitmesh::RpcResponse::error("Risk limits exceeded"));
                    }
                }
            },
            _ => quote! {
                // Generic state transition
                debug!("🔧 Using generic state transition validation");
            }
        }
    }

    // Helper methods for extracting configuration

    /// Extract states from attribute
    fn extract_states(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut states = Vec::new();
        
        if let Some(MacroValue::Array(state_array)) = attr.args.get("states") {
            for state in state_array {
                if let MacroValue::String(state_str) = state {
                    states.push(quote! { #state_str.to_string() });
                }
            }
        }
        
        states
    }

    /// Extract transitions from attribute
    fn extract_transitions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut transitions = Vec::new();
        
        if let Some(MacroValue::Array(transition_array)) = attr.args.get("transitions") {
            for transition in transition_array {
                if let MacroValue::String(transition_str) = transition {
                    // Parse transition format: "from_state -> to_state : trigger"
                    transitions.push(quote! { 
                        rabbitmesh_workflow::Transition::from_string(#transition_str) 
                    });
                }
            }
        }
        
        transitions
    }

    /// Extract state guards
    fn extract_state_guards(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut guards = Vec::new();
        
        if let Some(MacroValue::Array(guard_array)) = attr.args.get("guards") {
            for guard in guard_array {
                if let MacroValue::String(guard_str) = guard {
                    guards.push(quote! { 
                        rabbitmesh_workflow::StateGuard::from_expression(#guard_str) 
                    });
                }
            }
        }
        
        guards
    }

    /// Extract saga steps
    fn extract_saga_steps(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut steps = Vec::new();
        
        if let Some(MacroValue::Array(step_array)) = attr.args.get("steps") {
            for step in step_array {
                if let MacroValue::String(step_str) = step {
                    steps.push(quote! { 
                        rabbitmesh_workflow::SagaStep::from_string(#step_str) 
                    });
                }
            }
        }
        
        steps
    }

    /// Extract workflow steps
    fn extract_workflow_steps(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut steps = Vec::new();
        
        if let Some(MacroValue::Array(step_array)) = attr.args.get("steps") {
            for step in step_array {
                if let MacroValue::String(step_str) = step {
                    steps.push(quote! { 
                        rabbitmesh_workflow::WorkflowStep::from_string(#step_str) 
                    });
                }
            }
        }
        
        steps
    }

    /// Extract parallel branches
    fn extract_parallel_branches(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut branches = Vec::new();
        
        if let Some(MacroValue::Array(branch_array)) = attr.args.get("branches") {
            for branch in branch_array {
                if let MacroValue::String(branch_str) = branch {
                    branches.push(quote! { 
                        rabbitmesh_workflow::ParallelBranch::from_string(#branch_str) 
                    });
                }
            }
        }
        
        branches
    }

    /// Extract approval threshold
    fn extract_approval_threshold(attr: &MacroAttribute) -> TokenStream {
        if let Some(MacroValue::String(threshold_expr)) = attr.args.get("threshold") {
            quote! { #threshold_expr.to_string() }
        } else {
            quote! { "amount > 1000".to_string() }
        }
    }

    /// Extract approver roles
    fn extract_approver_roles(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut roles = Vec::new();
        
        if let Some(MacroValue::Array(role_array)) = attr.args.get("approver_roles") {
            for role in role_array {
                if let MacroValue::String(role_str) = role {
                    roles.push(quote! { #role_str.to_string() });
                }
            }
        }
        
        roles
    }

    /// Extract auto-approve conditions
    fn extract_auto_approve_conditions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut conditions = Vec::new();
        
        if let Some(MacroValue::Array(condition_array)) = attr.args.get("auto_approve_if") {
            for condition in condition_array {
                if let MacroValue::String(condition_str) = condition {
                    conditions.push(quote! { #condition_str.to_string() });
                }
            }
        }
        
        conditions
    }

    /// Extract escalation rules
    fn extract_escalation_rules(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut rules = Vec::new();
        
        if let Some(MacroValue::Array(rule_array)) = attr.args.get("escalation_rules") {
            for rule in rule_array {
                if let MacroValue::String(rule_str) = rule {
                    rules.push(quote! { 
                        rabbitmesh_workflow::EscalationRule::from_string(#rule_str) 
                    });
                }
            }
        }
        
        rules
    }

    /// Extract event handlers
    fn extract_event_handlers(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut handlers = Vec::new();
        
        if let Some(MacroValue::Array(handler_array)) = attr.args.get("event_handlers") {
            for handler in handler_array {
                if let MacroValue::String(handler_str) = handler {
                    let handler_ident = format_ident!("{}", handler_str);
                    handlers.push(quote! { 
                        Box::new(#handler_ident) as Box<dyn rabbitmesh_workflow::EventHandler>
                    });
                }
            }
        }
        
        handlers
    }

    /// Extract read models
    fn extract_read_models(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut models = Vec::new();
        
        if let Some(MacroValue::Array(model_array)) = attr.args.get("read_models") {
            for model in model_array {
                if let MacroValue::String(model_str) = model {
                    models.push(quote! { #model_str.to_string() });
                }
            }
        }
        
        models
    }

    /// Extract event types
    fn extract_event_types(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut types = Vec::new();
        
        if let Some(MacroValue::Array(type_array)) = attr.args.get("event_types") {
            for event_type in type_array {
                if let MacroValue::String(type_str) = event_type {
                    types.push(quote! { #type_str.to_string() });
                }
            }
        }
        
        types
    }

    /// Extract guard conditions
    fn extract_guard_conditions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut conditions = Vec::new();
        
        if let Some(MacroValue::Array(condition_array)) = attr.args.get("guards") {
            for condition in condition_array {
                if let MacroValue::String(condition_str) = condition {
                    conditions.push(quote! { #condition_str.to_string() });
                }
            }
        }
        
        conditions
    }

    /// Extract dependencies
    fn extract_dependencies(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut deps = Vec::new();
        
        if let Some(MacroValue::Array(dep_array)) = attr.args.get("depends_on") {
            for dep in dep_array {
                if let MacroValue::String(dep_str) = dep {
                    deps.push(quote! { #dep_str.to_string() });
                }
            }
        }
        
        deps
    }

    /// Extract step retry policy
    fn extract_step_retry_policy(attr: &MacroAttribute) -> TokenStream {
        if let Some(MacroValue::String(policy_str)) = attr.args.get("retry_policy") {
            quote! { Some(rabbitmesh_workflow::RetryPolicy::from_string(#policy_str)) }
        } else {
            quote! { None }
        }
    }

    /// Extract parallel tasks
    fn extract_parallel_tasks(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut tasks = Vec::new();
        
        if let Some(MacroValue::Array(task_array)) = attr.args.get("tasks") {
            for task in task_array {
                if let MacroValue::String(task_str) = task {
                    tasks.push(quote! { 
                        rabbitmesh_workflow::ParallelTask::from_string(#task_str) 
                    });
                }
            }
        }
        
        tasks
    }

    /// Extract conditions
    fn extract_conditions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut conditions = Vec::new();
        
        if let Some(MacroValue::Array(condition_array)) = attr.args.get("conditions") {
            for condition in condition_array {
                if let MacroValue::String(condition_str) = condition {
                    // Parse condition format: "expression -> action"
                    let parts: Vec<&str> = condition_str.split("->").map(|s| s.trim()).collect();
                    if parts.len() == 2 {
                        let expr = parts[0];
                        let action = parts[1];
                        conditions.push(quote! { 
                            (#expr.to_string(), #action.to_string()) 
                        });
                    }
                }
            }
        }
        
        conditions
    }

    /// Extract retry conditions
    fn extract_retry_conditions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut conditions = Vec::new();
        
        if let Some(MacroValue::Array(condition_array)) = attr.args.get("retry_on") {
            for condition in condition_array {
                if let MacroValue::String(condition_str) = condition {
                    conditions.push(quote! { 
                        rabbitmesh_workflow::RetryCondition::from_string(#condition_str) 
                    });
                }
            }
        }
        
        conditions
    }

    /// Extract string argument
    fn extract_string_arg(attr: &MacroAttribute, key: &str) -> Option<String> {
        if let Some(MacroValue::String(value)) = attr.args.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    /// Extract integer argument
    fn extract_int_arg(attr: &MacroAttribute, key: &str) -> Option<i64> {
        if let Some(MacroValue::Integer(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Extract boolean argument
    fn extract_bool_arg(attr: &MacroAttribute, key: &str) -> Option<bool> {
        if let Some(MacroValue::Boolean(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Infer domain from context
    fn infer_domain_from_context(context: &MacroContext) -> String {
        let service_name = context.service_name.to_lowercase();
        let method_name = context.method_name.to_lowercase();
        
        // Check service name patterns
        if service_name.contains("ecommerce") || service_name.contains("shop") || service_name.contains("product") || service_name.contains("cart") {
            return "ecommerce".to_string();
        }
        if service_name.contains("finance") || service_name.contains("trading") || service_name.contains("bank") || service_name.contains("payment") {
            return "finance".to_string();
        }
        if service_name.contains("health") || service_name.contains("medical") || service_name.contains("patient") {
            return "healthcare".to_string();
        }
        if service_name.contains("iot") || service_name.contains("sensor") || service_name.contains("device") {
            return "iot".to_string();
        }
        if service_name.contains("game") || service_name.contains("gaming") || service_name.contains("player") {
            return "gaming".to_string();
        }
        if service_name.contains("social") || service_name.contains("media") || service_name.contains("feed") {
            return "social".to_string();
        }
        if service_name.contains("logistics") || service_name.contains("shipping") || service_name.contains("delivery") {
            return "logistics".to_string();
        }
        
        // Check method name patterns
        if method_name.contains("product") || method_name.contains("cart") || method_name.contains("order") {
            return "ecommerce".to_string();
        }
        if method_name.contains("trade") || method_name.contains("portfolio") || method_name.contains("account") {
            return "finance".to_string();
        }
        if method_name.contains("patient") || method_name.contains("medical") || method_name.contains("health") {
            return "healthcare".to_string();
        }
        if method_name.contains("sensor") || method_name.contains("device") || method_name.contains("telemetry") {
            return "iot".to_string();
        }
        if method_name.contains("player") || method_name.contains("game") || method_name.contains("score") {
            return "gaming".to_string();
        }
        if method_name.contains("post") || method_name.contains("feed") || method_name.contains("social") {
            return "social".to_string();
        }
        if method_name.contains("ship") || method_name.contains("deliver") || method_name.contains("route") {
            return "logistics".to_string();
        }
        
        "generic".to_string()
    }
}