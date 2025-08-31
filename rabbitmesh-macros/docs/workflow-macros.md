# Workflow Macros

Advanced workflow orchestration with state machines, saga patterns, choreography, and long-running process management for complex business processes.

## Overview

Workflow macros enable sophisticated business process automation, from simple state machines to complex distributed sagas, providing reliable coordination across microservices.

---

## State Machine Macros

### `#[state_machine]`

**Purpose**: Implements finite state machines for managing complex entity lifecycles and business processes.

**Usage**:
```rust
#[state_machine]
#[derive(Debug, Clone)]
pub enum OrderState {
    #[initial]
    Pending,
    
    #[on_enter("validate_payment")]
    PaymentProcessing,
    
    #[on_enter("allocate_inventory")]
    InventoryAllocated,
    
    #[on_enter("prepare_shipment")]  
    Preparing,
    
    #[on_enter("track_shipment")]
    Shipped,
    
    #[final_state]
    Delivered,
    
    #[error_state]
    Cancelled,
    
    #[error_state]
    Failed,
}

#[state_transitions]
impl OrderState {
    #[transition(from = "Pending", to = "PaymentProcessing")]
    #[guard("payment_method_valid")]
    async fn process_payment(&mut self, event: PaymentInitiated) -> Result<(), OrderError> {
        // Payment processing logic
        Ok(())
    }
    
    #[transition(from = "PaymentProcessing", to = "InventoryAllocated")]
    #[guard("payment_successful")]
    async fn allocate_inventory(&mut self, event: PaymentCompleted) -> Result<(), OrderError> {
        // Inventory allocation logic
        Ok(())
    }
    
    #[transition(from = "InventoryAllocated", to = "Preparing")]
    #[guard("inventory_available")]
    async fn prepare_order(&mut self, event: InventoryAllocated) -> Result<(), OrderError> {
        // Order preparation logic
        Ok(())
    }
    
    #[transition(from = "Preparing", to = "Shipped")]
    async fn ship_order(&mut self, event: OrderPrepared) -> Result<(), OrderError> {
        // Shipping logic
        Ok(())
    }
    
    #[transition(from = "Shipped", to = "Delivered")]
    async fn deliver_order(&mut self, event: DeliveryConfirmed) -> Result<(), OrderError> {
        // Delivery confirmation logic
        Ok(())
    }
    
    #[transition(from = "*", to = "Cancelled")]
    #[guard("cancellation_allowed")]
    async fn cancel_order(&mut self, event: CancellationRequested) -> Result<(), OrderError> {
        // Cancellation logic with compensation
        Ok(())
    }
}
```

**State Machine Features**:
- **Declarative States**: Define states with attributes
- **Guarded Transitions**: Conditions that must be met for transitions
- **Entry/Exit Actions**: Actions executed when entering/leaving states
- **Error Handling**: Automatic error state transitions
- **Persistence**: Automatic state persistence
- **Event-Driven**: React to domain events

**When to Use**: For complex entity lifecycles, business processes, and workflow management.

---

### `#[state_machine_persist]`

**Purpose**: Automatically persists state machine state to ensure durability across restarts.

**Usage**:
```rust
#[service_method("POST /orders/:id/events")]
#[require_auth]
#[state_machine_persist("order_states")]
pub async fn handle_order_event(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    let event: OrderEvent = msg.deserialize_payload()?;
    
    // Load persisted state machine
    let mut order_fsm = load_order_state_machine(order_id).await?;
    
    // Process event through state machine
    order_fsm.handle_event(event).await?;
    
    // State automatically persisted after successful transition
    Ok(RpcResponse::success(&order_fsm.current_state(), 0)?)
}
```

**When to Use**: For state machines that must survive service restarts and failures.

---

## Saga Pattern Macros

### `#[saga]`

**Purpose**: Implements the saga pattern for distributed transaction management across microservices.

**Usage**:
```rust
#[saga("order_fulfillment")]
pub struct OrderFulfillmentSaga {
    pub order_id: String,
    pub customer_id: String,
    pub payment_id: Option<String>,
    pub inventory_reservation_id: Option<String>,
    pub shipping_id: Option<String>,
}

#[saga_steps]
impl OrderFulfillmentSaga {
    #[step("charge_payment")]
    #[compensate("refund_payment")]
    async fn charge_payment(&mut self) -> Result<PaymentResult, SagaError> {
        let payment_result = charge_customer_payment(&self.customer_id, &self.order_id).await?;
        self.payment_id = Some(payment_result.payment_id);
        Ok(payment_result)
    }
    
    #[step("reserve_inventory")]
    #[compensate("release_inventory")]
    async fn reserve_inventory(&mut self) -> Result<InventoryResult, SagaError> {
        let reservation = reserve_order_inventory(&self.order_id).await?;
        self.inventory_reservation_id = Some(reservation.reservation_id);
        Ok(reservation)
    }
    
    #[step("arrange_shipping")]
    #[compensate("cancel_shipping")]
    async fn arrange_shipping(&mut self) -> Result<ShippingResult, SagaError> {
        let shipping = arrange_order_shipping(&self.order_id).await?;
        self.shipping_id = Some(shipping.shipping_id);
        Ok(shipping)
    }
    
    // Compensation actions (automatically called on failure)
    async fn refund_payment(&mut self) -> Result<(), SagaError> {
        if let Some(payment_id) = &self.payment_id {
            refund_customer_payment(payment_id).await?;
        }
        Ok(())
    }
    
    async fn release_inventory(&mut self) -> Result<(), SagaError> {
        if let Some(reservation_id) = &self.inventory_reservation_id {
            release_inventory_reservation(reservation_id).await?;
        }
        Ok(())
    }
    
    async fn cancel_shipping(&mut self) -> Result<(), SagaError> {
        if let Some(shipping_id) = &self.shipping_id {
            cancel_shipping_arrangement(shipping_id).await?;
        }
        Ok(())
    }
}
```

**Saga Execution**:
```rust
#[service_method("POST /orders/:id/fulfill")]
#[require_auth]
pub async fn fulfill_order(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    let fulfillment_request: FulfillmentRequest = msg.deserialize_payload()?;
    
    // Create and execute saga
    let mut saga = OrderFulfillmentSaga {
        order_id: order_id.clone(),
        customer_id: fulfillment_request.customer_id,
        payment_id: None,
        inventory_reservation_id: None, 
        shipping_id: None,
    };
    
    // Execute saga steps with automatic compensation on failure
    let result = saga.execute().await?;
    
    match result {
        SagaResult::Success => {
            Ok(RpcResponse::success(&json!({"status": "fulfilled"}), 0)?)
        }
        SagaResult::Compensated(error) => {
            Err(format!("Order fulfillment failed: {}, compensated successfully", error))
        }
        SagaResult::Failed(error) => {
            Err(format!("Order fulfillment failed: {}, compensation may be incomplete", error))
        }
    }
}
```

**Saga Features**:
- **Automatic Compensation**: Rollback completed steps on failure
- **Step Ordering**: Configurable step execution order
- **Parallel Steps**: Execute independent steps concurrently
- **Timeout Handling**: Handle step timeouts with compensation
- **Persistence**: Saga state persistence for failure recovery
- **Monitoring**: Built-in saga execution monitoring

**When to Use**: For distributed transactions spanning multiple microservices.

---

### `#[saga_parallel]`

**Purpose**: Executes independent saga steps in parallel for better performance.

**Usage**:
```rust
#[saga("user_onboarding")]
pub struct UserOnboardingSaga {
    pub user_id: String,
    pub email: String,
    pub profile_created: bool,
    pub email_verified: bool,
    pub welcome_sent: bool,
}

#[saga_steps]
impl UserOnboardingSaga {
    #[step("create_profile")]
    #[compensate("delete_profile")]
    async fn create_user_profile(&mut self) -> Result<(), SagaError> {
        create_user_profile_record(&self.user_id).await?;
        self.profile_created = true;
        Ok(())
    }
    
    #[saga_parallel([
        "send_verification_email",
        "send_welcome_email",
        "setup_default_preferences"
    ])]
    async fn parallel_onboarding_tasks(&mut self) -> Result<(), SagaError> {
        // These steps execute in parallel as they're independent
        Ok(())
    }
    
    #[step("send_verification_email")]
    #[compensate("mark_email_unverified")]
    async fn send_verification_email(&mut self) -> Result<(), SagaError> {
        send_email_verification(&self.email).await?;
        Ok(())
    }
    
    #[step("send_welcome_email")] 
    async fn send_welcome_email(&mut self) -> Result<(), SagaError> {
        send_welcome_message(&self.email).await?;
        self.welcome_sent = true;
        Ok(())
    }
    
    #[step("setup_default_preferences")]
    #[compensate("remove_preferences")]
    async fn setup_default_preferences(&mut self) -> Result<(), SagaError> {
        setup_user_default_preferences(&self.user_id).await?;
        Ok(())
    }
}
```

**When to Use**: When saga steps can be executed independently to improve performance.

---

## Orchestration Macros

### `#[orchestrator]`

**Purpose**: Implements centralized workflow orchestration for complex multi-service processes.

**Usage**:
```rust
#[orchestrator("loan_approval")]
pub struct LoanApprovalOrchestrator {
    application_id: String,
    applicant_info: ApplicantInfo,
    credit_score: Option<u32>,
    income_verified: bool,
    risk_assessment: Option<RiskLevel>,
    approval_decision: Option<ApprovalDecision>,
}

#[workflow_steps]
impl LoanApprovalOrchestrator {
    #[step("credit_check")]
    #[timeout("30s")]
    #[retry(attempts = 2)]
    async fn perform_credit_check(&mut self) -> Result<CreditCheckResult, WorkflowError> {
        let result = call_credit_bureau(&self.applicant_info).await?;
        self.credit_score = Some(result.score);
        Ok(result)
    }
    
    #[step("income_verification")]
    #[timeout("60s")]
    #[parallel_with("employment_verification")]
    async fn verify_income(&mut self) -> Result<IncomeResult, WorkflowError> {
        let verification = verify_applicant_income(&self.applicant_info).await?;
        self.income_verified = verification.verified;
        Ok(verification)
    }
    
    #[step("employment_verification")]
    #[timeout("60s")]
    async fn verify_employment(&mut self) -> Result<EmploymentResult, WorkflowError> {
        verify_applicant_employment(&self.applicant_info).await
    }
    
    #[step("risk_assessment")]
    #[depends_on(["credit_check", "income_verification", "employment_verification"])]
    async fn assess_risk(&mut self) -> Result<RiskAssessment, WorkflowError> {
        let assessment = calculate_loan_risk(
            self.credit_score.unwrap(),
            self.income_verified,
            &self.applicant_info
        ).await?;
        
        self.risk_assessment = Some(assessment.risk_level);
        Ok(assessment)
    }
    
    #[step("approval_decision")]
    #[depends_on(["risk_assessment"])]
    async fn make_approval_decision(&mut self) -> Result<ApprovalDecision, WorkflowError> {
        let decision = determine_loan_approval(
            &self.risk_assessment.as_ref().unwrap(),
            &self.applicant_info
        ).await?;
        
        self.approval_decision = Some(decision.clone());
        Ok(decision)
    }
    
    #[step("notification")]
    #[depends_on(["approval_decision"])]
    async fn notify_applicant(&mut self) -> Result<(), WorkflowError> {
        let decision = self.approval_decision.as_ref().unwrap();
        send_approval_notification(&self.applicant_info.email, decision).await?;
        Ok(())
    }
}
```

**Workflow Execution**:
```rust
#[service_method("POST /loans/:id/process")]
#[require_auth]
pub async fn process_loan_application(msg: Message) -> Result<RpcResponse, String> {
    let application_id = msg.get_path_param("id")?;
    let applicant_info = fetch_applicant_info(&application_id).await?;
    
    // Create and execute orchestrated workflow
    let mut orchestrator = LoanApprovalOrchestrator {
        application_id: application_id.clone(),
        applicant_info,
        credit_score: None,
        income_verified: false,
        risk_assessment: None,
        approval_decision: None,
    };
    
    // Execute workflow with automatic step coordination
    let result = orchestrator.execute().await?;
    
    Ok(RpcResponse::success(&result.approval_decision, 0)?)
}
```

**Orchestration Features**:
- **Dependency Management**: Automatic step ordering based on dependencies
- **Parallel Execution**: Execute independent steps concurrently
- **Error Handling**: Configurable error handling and recovery
- **Timeout Management**: Per-step timeout configuration
- **State Persistence**: Workflow state persistence for recovery
- **Progress Tracking**: Real-time workflow progress monitoring

**When to Use**: For complex workflows requiring centralized coordination and dependency management.

---

## Choreography Macros

### `#[choreography]`

**Purpose**: Implements decentralized workflow coordination through event choreography.

**Usage**:
```rust
#[choreography("order_processing")]
pub struct OrderProcessingChoreography;

#[event_handlers]
impl OrderProcessingChoreography {
    #[on_event("OrderCreated")]
    #[publish("PaymentRequested")]
    async fn handle_order_created(event: OrderCreated) -> Result<PaymentRequested, ChoreographyError> {
        // Validate order and request payment
        let payment_request = PaymentRequested {
            order_id: event.order_id,
            amount: event.total_amount,
            customer_id: event.customer_id,
            payment_method: event.payment_method,
        };
        
        Ok(payment_request)
    }
    
    #[on_event("PaymentCompleted")]
    #[publish("InventoryReserved")]
    async fn handle_payment_completed(event: PaymentCompleted) -> Result<InventoryReserved, ChoreographyError> {
        // Reserve inventory after successful payment
        let reservation = InventoryReserved {
            order_id: event.order_id,
            items: fetch_order_items(&event.order_id).await?,
            reservation_id: generate_reservation_id(),
        };
        
        Ok(reservation)
    }
    
    #[on_event("InventoryReserved")]
    #[publish("ShippingScheduled")]
    async fn handle_inventory_reserved(event: InventoryReserved) -> Result<ShippingScheduled, ChoreographyError> {
        // Schedule shipping after inventory reservation
        let shipping = ShippingScheduled {
            order_id: event.order_id,
            estimated_delivery: calculate_delivery_date().await?,
            tracking_number: generate_tracking_number(),
        };
        
        Ok(shipping)
    }
    
    #[on_event("PaymentFailed")]
    #[publish("OrderCancelled")]
    async fn handle_payment_failed(event: PaymentFailed) -> Result<OrderCancelled, ChoreographyError> {
        // Cancel order on payment failure
        let cancellation = OrderCancelled {
            order_id: event.order_id,
            reason: "Payment failed".to_string(),
            cancelled_at: chrono::Utc::now(),
        };
        
        Ok(cancellation)
    }
    
    #[on_event("InventoryUnavailable")]
    #[publish("PaymentRefunded")]
    async fn handle_inventory_unavailable(event: InventoryUnavailable) -> Result<PaymentRefunded, ChoreographyError> {
        // Refund payment if inventory is unavailable
        let refund = PaymentRefunded {
            order_id: event.order_id,
            payment_id: fetch_payment_id(&event.order_id).await?,
            refund_amount: event.order_amount,
            reason: "Inventory unavailable".to_string(),
        };
        
        Ok(refund)
    }
}
```

**Choreography Features**:
- **Event-Driven**: React to domain events across services
- **Decentralized**: No central coordinator required
- **Loose Coupling**: Services communicate only through events
- **Resilient**: Built-in retry and error handling
- **Scalable**: Natural horizontal scaling through event distribution

**When to Use**: For workflows where services should remain loosely coupled and autonomous.

---

## Long-Running Process Macros

### `#[long_running_process]`

**Purpose**: Manages processes that span extended periods with state persistence and recovery.

**Usage**:
```rust
#[long_running_process("user_trial")]
#[persist_state("user_trials")]
pub struct UserTrialProcess {
    pub user_id: String,
    pub trial_start: DateTime<Utc>,
    pub trial_end: DateTime<Utc>,
    pub reminders_sent: u32,
    pub trial_extended: bool,
    pub conversion_completed: bool,
}

#[process_steps]
impl UserTrialProcess {
    #[schedule(delay = "7 days")]
    async fn send_trial_reminder(&mut self) -> Result<(), ProcessError> {
        send_trial_reminder_email(&self.user_id).await?;
        self.reminders_sent += 1;
        Ok(())
    }
    
    #[schedule(delay = "14 days")]
    async fn send_conversion_offer(&mut self) -> Result<(), ProcessError> {
        send_conversion_offer_email(&self.user_id).await?;
        Ok(())
    }
    
    #[schedule(delay = "25 days")]
    async fn send_final_reminder(&mut self) -> Result<(), ProcessError> {
        send_final_trial_reminder(&self.user_id).await?;
        Ok(())
    }
    
    #[schedule(at = "trial_end")]
    async fn end_trial(&mut self) -> Result<(), ProcessError> {
        if !self.conversion_completed {
            disable_user_trial_features(&self.user_id).await?;
            send_trial_ended_notification(&self.user_id).await?;
        }
        Ok(())
    }
    
    #[on_event("UserUpgraded")]
    async fn handle_user_upgrade(&mut self, event: UserUpgraded) -> Result<(), ProcessError> {
        if event.user_id == self.user_id {
            self.conversion_completed = true;
            // Cancel remaining scheduled tasks
            self.cancel_pending_tasks().await?;
        }
        Ok(())
    }
    
    #[on_event("TrialExtensionRequested")]
    async fn handle_trial_extension(&mut self, event: TrialExtensionRequested) -> Result<(), ProcessError> {
        if event.user_id == self.user_id && !self.trial_extended {
            self.trial_end += chrono::Duration::days(7);
            self.trial_extended = true;
            
            // Reschedule end_trial task
            self.reschedule_task("end_trial", self.trial_end).await?;
        }
        Ok(())
    }
}
```

**Long-Running Process Features**:
- **Scheduled Tasks**: Execute tasks at specific times or after delays
- **Event Handling**: React to events during the process lifetime
- **State Persistence**: Automatic state persistence across restarts
- **Process Recovery**: Resume processes after service restarts
- **Task Cancellation**: Cancel or reschedule pending tasks
- **Timeout Management**: Handle process and step timeouts

**When to Use**: For business processes that span hours, days, or weeks.

---

## Workflow Monitoring Macros

### `#[workflow_monitor]`

**Purpose**: Adds comprehensive monitoring and observability to workflow execution.

**Usage**:
```rust
#[service_method("POST /workflows/order-fulfillment")]
#[require_auth]
#[workflow_monitor(
    track_duration = true,
    track_step_performance = true,
    alert_on_failure = true,
    export_metrics = true
)]
pub async fn start_order_fulfillment(msg: Message) -> Result<RpcResponse, String> {
    let order_request: OrderFulfillmentRequest = msg.deserialize_payload()?;
    
    // Workflow automatically monitored:
    // - Overall workflow duration
    // - Individual step performance
    // - Failure rates and error patterns
    // - Resource utilization
    // - Success/failure alerts
    
    let workflow_id = start_fulfillment_workflow(order_request).await?;
    
    Ok(RpcResponse::success(&json!({
        "workflow_id": workflow_id,
        "status": "started"
    }), 0)?)
}
```

**Monitoring Capabilities**:
- **Performance Metrics**: Step duration, workflow throughput
- **Error Tracking**: Failure rates, error categorization
- **Resource Usage**: Memory, CPU, network utilization
- **Business Metrics**: Process success rates, SLA compliance
- **Real-time Dashboards**: Workflow status visualization

**When to Use**: For production workflows requiring operational visibility.

---

## Integration Examples

### Complete E-commerce Order Workflow
```rust
#[orchestrator("ecommerce_order")]
#[workflow_monitor]
pub struct EcommerceOrderWorkflow {
    order_id: String,
    customer_id: String,
    items: Vec<OrderItem>,
    payment_method: PaymentMethod,
    shipping_address: Address,
    
    // State tracking
    payment_completed: bool,
    inventory_reserved: bool,
    shipping_arranged: bool,
    notifications_sent: bool,
}

#[workflow_steps]
impl EcommerceOrderWorkflow {
    #[step("validate_order")]
    #[timeout("10s")]
    async fn validate_order(&mut self) -> Result<ValidationResult, WorkflowError> {
        // Validate order details, pricing, availability
        validate_order_data(&self.order_id, &self.items).await
    }
    
    #[step("process_payment")]
    #[depends_on(["validate_order"])]
    #[timeout("30s")]
    #[retry(attempts = 2)]
    async fn process_payment(&mut self) -> Result<PaymentResult, WorkflowError> {
        let payment_result = charge_payment(
            &self.customer_id,
            &self.payment_method,
            calculate_total(&self.items)?
        ).await?;
        
        self.payment_completed = true;
        Ok(payment_result)
    }
    
    #[step("reserve_inventory")]
    #[depends_on(["process_payment"])]
    #[timeout("20s")]
    async fn reserve_inventory(&mut self) -> Result<InventoryResult, WorkflowError> {
        let reservation = reserve_items(&self.items).await?;
        self.inventory_reserved = true;
        Ok(reservation)
    }
    
    #[step("arrange_shipping")]
    #[depends_on(["reserve_inventory"])]
    #[timeout("15s")]
    async fn arrange_shipping(&mut self) -> Result<ShippingResult, WorkflowError> {
        let shipping = create_shipping_label(
            &self.shipping_address,
            &self.items
        ).await?;
        
        self.shipping_arranged = true;
        Ok(shipping)
    }
    
    #[step("send_notifications")]
    #[depends_on(["arrange_shipping"])]
    #[parallel_steps([
        "send_order_confirmation",
        "send_shipping_notification", 
        "update_inventory_system"
    ])]
    async fn send_notifications(&mut self) -> Result<(), WorkflowError> {
        self.notifications_sent = true;
        Ok(())
    }
    
    #[step("send_order_confirmation")]
    async fn send_order_confirmation(&mut self) -> Result<(), WorkflowError> {
        send_email_confirmation(&self.customer_id, &self.order_id).await
    }
    
    #[step("send_shipping_notification")]
    async fn send_shipping_notification(&mut self) -> Result<(), WorkflowError> {
        send_shipping_update(&self.customer_id, &self.order_id).await
    }
    
    #[step("update_inventory_system")]
    async fn update_inventory_system(&mut self) -> Result<(), WorkflowError> {
        update_inventory_counts(&self.items).await
    }
    
    // Compensation logic
    #[on_failure("process_payment")]
    async fn handle_payment_failure(&mut self) -> Result<(), WorkflowError> {
        send_payment_failure_notification(&self.customer_id).await?;
        Ok(())
    }
    
    #[on_failure("reserve_inventory")]
    async fn handle_inventory_failure(&mut self) -> Result<(), WorkflowError> {
        if self.payment_completed {
            refund_payment(&self.order_id).await?;
        }
        send_inventory_failure_notification(&self.customer_id).await?;
        Ok(())
    }
}
```

### Distributed Saga with Choreography
```rust
// Service A: Order Service
#[choreography("distributed_order")]
pub struct OrderServiceChoreography;

#[event_handlers]
impl OrderServiceChoreography {
    #[on_event("OrderPlaced")]
    #[publish("PaymentRequested")]
    async fn handle_order_placed(event: OrderPlaced) -> Result<PaymentRequested, ChoreographyError> {
        // Validate and prepare payment request
        Ok(PaymentRequested {
            order_id: event.order_id,
            amount: event.amount,
            customer_id: event.customer_id,
        })
    }
    
    #[on_event("PaymentCompleted")]
    #[publish("InventoryCheckRequested")]
    async fn handle_payment_completed(event: PaymentCompleted) -> Result<InventoryCheckRequested, ChoreographyError> {
        Ok(InventoryCheckRequested {
            order_id: event.order_id,
            items: fetch_order_items(&event.order_id).await?,
        })
    }
    
    #[on_event("InventoryConfirmed")]
    #[publish("FulfillmentRequested")]
    async fn handle_inventory_confirmed(event: InventoryConfirmed) -> Result<FulfillmentRequested, ChoreographyError> {
        Ok(FulfillmentRequested {
            order_id: event.order_id,
            shipping_address: fetch_shipping_address(&event.order_id).await?,
        })
    }
    
    #[on_event("PaymentFailed", "InventoryUnavailable")]
    #[publish("OrderFailed")]
    async fn handle_order_failure(event: DomainEvent) -> Result<OrderFailed, ChoreographyError> {
        let order_id = match event {
            DomainEvent::PaymentFailed(e) => e.order_id,
            DomainEvent::InventoryUnavailable(e) => e.order_id,
            _ => return Err(ChoreographyError::UnexpectedEvent),
        };
        
        Ok(OrderFailed {
            order_id,
            reason: format!("Order failed due to: {:?}", event),
        })
    }
}

// Service B: Payment Service
#[choreography("payment_processing")]
pub struct PaymentServiceChoreography;

#[event_handlers]
impl PaymentServiceChoreography {
    #[on_event("PaymentRequested")]
    #[publish("PaymentCompleted", "PaymentFailed")]
    async fn handle_payment_request(event: PaymentRequested) -> Result<PaymentEvent, ChoreographyError> {
        match process_payment(&event).await {
            Ok(result) => Ok(PaymentEvent::Completed(PaymentCompleted {
                order_id: event.order_id,
                payment_id: result.payment_id,
                amount: event.amount,
            })),
            Err(error) => Ok(PaymentEvent::Failed(PaymentFailed {
                order_id: event.order_id,
                error_code: error.code,
                error_message: error.message,
            })),
        }
    }
}
```

### Long-Running Customer Onboarding Process
```rust
#[long_running_process("customer_onboarding")]
#[persist_state("onboarding_processes")]
pub struct CustomerOnboardingProcess {
    customer_id: String,
    email: String,
    onboarding_started: DateTime<Utc>,
    email_verified: bool,
    profile_completed: bool,
    first_purchase_made: bool,
    engagement_score: u32,
}

#[process_steps]
impl CustomerOnboardingProcess {
    #[schedule(delay = "1 hour")]
    async fn send_welcome_email(&mut self) -> Result<(), ProcessError> {
        send_welcome_email(&self.email).await?;
        Ok(())
    }
    
    #[schedule(delay = "1 day")]
    #[condition("!email_verified")]
    async fn send_email_verification_reminder(&mut self) -> Result<(), ProcessError> {
        send_verification_reminder(&self.email).await?;
        Ok(())
    }
    
    #[schedule(delay = "3 days")]
    #[condition("email_verified && !profile_completed")]
    async fn send_profile_completion_reminder(&mut self) -> Result<(), ProcessError> {
        send_profile_reminder(&self.email).await?;
        Ok(())
    }
    
    #[schedule(delay = "7 days")]
    #[condition("profile_completed && !first_purchase_made")]
    async fn send_first_purchase_incentive(&mut self) -> Result<(), ProcessError> {
        send_purchase_incentive(&self.customer_id).await?;
        Ok(())
    }
    
    #[schedule(delay = "14 days")]
    async fn calculate_engagement_score(&mut self) -> Result<(), ProcessError> {
        self.engagement_score = calculate_customer_engagement(&self.customer_id).await?;
        
        if self.engagement_score < 30 {
            send_reengagement_campaign(&self.customer_id).await?;
        }
        
        Ok(())
    }
    
    #[schedule(delay = "30 days")]
    async fn complete_onboarding(&mut self) -> Result<(), ProcessError> {
        mark_onboarding_complete(&self.customer_id).await?;
        
        if self.engagement_score > 70 {
            send_loyalty_program_invitation(&self.customer_id).await?;
        }
        
        Ok(())
    }
    
    #[on_event("EmailVerified")]
    async fn handle_email_verified(&mut self, event: EmailVerified) -> Result<(), ProcessError> {
        if event.customer_id == self.customer_id {
            self.email_verified = true;
            send_verification_success_email(&self.email).await?;
        }
        Ok(())
    }
    
    #[on_event("ProfileCompleted")]
    async fn handle_profile_completed(&mut self, event: ProfileCompleted) -> Result<(), ProcessError> {
        if event.customer_id == self.customer_id {
            self.profile_completed = true;
            send_profile_completion_confirmation(&self.email).await?;
        }
        Ok(())
    }
    
    #[on_event("FirstPurchaseMade")]
    async fn handle_first_purchase(&mut self, event: FirstPurchaseMade) -> Result<(), ProcessError> {
        if event.customer_id == self.customer_id {
            self.first_purchase_made = true;
            send_first_purchase_congratulations(&self.email).await?;
        }
        Ok(())
    }
}
```

---

## Best Practices

### 1. Choose Appropriate Workflow Patterns
```rust
// Simple entity lifecycle - State Machine
#[state_machine]
enum OrderState { Pending, Processing, Shipped, Delivered }

// Distributed transaction - Saga
#[saga("payment_processing")]
struct PaymentSaga { /* compensation logic */ }

// Complex coordination - Orchestrator
#[orchestrator("loan_approval")]
struct LoanOrchestrator { /* centralized logic */ }

// Loose coupling - Choreography  
#[choreography("event_driven")]
struct EventChoreography { /* event handlers */ }

// Extended processes - Long-Running Process
#[long_running_process("trial_management")]
struct TrialProcess { /* scheduled tasks */ }
```

### 2. Implement Proper Error Handling
```rust
#[saga_steps]
impl OrderSaga {
    #[step("charge_payment")]
    #[compensate("refund_payment")]
    #[timeout("30s")]
    #[retry(attempts = 2)]
    async fn charge_payment(&mut self) -> Result<PaymentResult, SagaError> {
        // Implement with proper error handling
    }
    
    // Always implement compensation
    async fn refund_payment(&mut self) -> Result<(), SagaError> {
        // Reliable compensation logic
    }
}
```

### 3. Use Monitoring and Observability
```rust
#[orchestrator("critical_workflow")]
#[workflow_monitor]
#[metrics]
#[audit_log]
pub struct CriticalWorkflow {
    // Monitor performance and compliance
}
```

### 4. Handle State Persistence
```rust
#[long_running_process("user_journey")]
#[persist_state("user_journeys")]
pub struct UserJourneyProcess {
    // State automatically persisted and recovered
}
```

---

This completes the Workflow Macros documentation, providing comprehensive business process automation and coordination capabilities for complex microservice architectures.