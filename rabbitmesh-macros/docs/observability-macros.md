# Observability Macros

Comprehensive observability stack with Prometheus metrics, structured logging, distributed tracing, and custom monitoring for production-ready microservices.

## Overview

Observability macros provide complete visibility into your microservice behavior with minimal setup. They automatically instrument your services with metrics, logs, traces, and health monitoring.

---

## Metrics Macros

### `#[metrics]`

**Purpose**: Automatically instruments methods with comprehensive metrics collection.

**Usage**:
```rust
#[service_method("GET /users/:id")]
#[require_auth]
#[metrics]
pub async fn get_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let user = fetch_user(user_id).await?;
    Ok(RpcResponse::success(&user, 0)?)
}
```

**Automatically Collected Metrics**:
- **Request Count**: Total requests per endpoint
- **Response Time**: Request duration histogram
- **Error Rate**: Success/failure rates
- **Active Requests**: Current concurrent requests
- **Response Sizes**: Response payload size distribution

**Generated Metrics**:
```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",endpoint="/users/:id",status="200"} 1234

# HELP http_request_duration_seconds HTTP request duration
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{method="GET",endpoint="/users/:id",le="0.1"} 100
http_request_duration_seconds_bucket{method="GET",endpoint="/users/:id",le="0.5"} 450
http_request_duration_seconds_sum{method="GET",endpoint="/users/:id"} 123.45
http_request_duration_seconds_count{method="GET",endpoint="/users/:id"} 500

# HELP http_requests_active Currently active HTTP requests  
# TYPE http_requests_active gauge
http_requests_active{method="GET",endpoint="/users/:id"} 5
```

**When to Use**: On all service methods for comprehensive observability.

---

### `#[metrics(custom)]` 

**Purpose**: Instruments methods with custom metrics and labels.

**Usage**:
```rust
#[service_method("POST /orders")]
#[require_auth]
#[metrics(
    counter = "orders_created_total",
    histogram = "order_processing_duration",
    labels = ["user_tier", "order_type"]
)]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    let user_tier = msg.get_user_context()?.tier;
    
    // Custom metrics automatically recorded with labels:
    // orders_created_total{user_tier="premium",order_type="subscription"}
    // order_processing_duration{user_tier="premium",order_type="subscription"}
    
    let order = process_order_creation(order_data).await?;
    Ok(RpcResponse::success(&order, 0)?)
}
```

**Custom Metric Types**:
- **counter**: Monotonic counters (requests, events, etc.)
- **gauge**: Current values (queue depth, connections, etc.)  
- **histogram**: Value distributions (durations, sizes, etc.)
- **summary**: Quantiles and totals

**When to Use**: For domain-specific metrics beyond standard HTTP metrics.

---

### `#[counter(name, labels)]`

**Purpose**: Increments a counter metric.

**Usage**:
```rust
#[service_method("POST /users/login")]
#[counter("user_logins_total", ["user_type", "login_method"])]
pub async fn user_login(msg: Message) -> Result<RpcResponse, String> {
    let credentials: LoginRequest = msg.deserialize_payload()?;
    
    // Automatically increments: user_logins_total{user_type="premium",login_method="oauth"}
    let result = authenticate_user(credentials).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**When to Use**: For counting discrete events and occurrences.

---

### `#[gauge(name, value)]`

**Purpose**: Sets a gauge metric to a specific value.

**Usage**:
```rust
#[service_method("GET /queue/status")]
#[gauge("queue_depth", "get_queue_depth()")]
pub async fn get_queue_status(msg: Message) -> Result<RpcResponse, String> {
    let status = get_queue_status().await?;
    
    // Automatically sets: queue_depth = get_queue_depth() result
    Ok(RpcResponse::success(&status, 0)?)
}
```

**When to Use**: For tracking current values that can go up or down.

---

### `#[histogram(name, buckets)]`

**Purpose**: Records values in a histogram for distribution analysis.

**Usage**:
```rust
#[service_method("POST /data/process")]
#[histogram("data_processing_size_bytes", [1000, 10000, 100000, 1000000])]
pub async fn process_data(msg: Message) -> Result<RpcResponse, String> {
    let data: ProcessDataRequest = msg.deserialize_payload()?;
    let data_size = calculate_data_size(&data);
    
    // Automatically records data_size in histogram buckets
    let result = process_large_dataset(data).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**When to Use**: For analyzing distributions of durations, sizes, or other measurable values.

---

## Logging Macros

### `#[log(level)]`

**Purpose**: Adds structured logging to service methods.

**Usage**:
```rust
#[service_method("POST /payments")]
#[require_auth]
#[log("info")]
pub async fn process_payment(msg: Message) -> Result<RpcResponse, String> {
    let payment: PaymentRequest = msg.deserialize_payload()?;
    
    // Automatically logs:
    // INFO [payment_service] Processing payment request 
    //   method=POST path=/payments user_id=123 amount=99.99 currency=USD
    
    let result = charge_payment(payment).await?;
    
    // Automatically logs response:
    // INFO [payment_service] Payment processed successfully
    //   method=POST path=/payments user_id=123 transaction_id=txn_456 duration_ms=234
    
    Ok(RpcResponse::success(&result, 0)?)
}
```

**Log Levels**: `trace`, `debug`, `info`, `warn`, `error`

**Automatically Logged Fields**:
- Request metadata (method, path, user_id)
- Request/response timing
- Error details (for failed requests)
- Custom context fields

**When to Use**: On all service methods for request/response logging.

---

### `#[structured_log]`

**Purpose**: Adds rich structured logging with custom fields.

**Usage**:
```rust
#[service_method("POST /orders/:id/fulfill")]
#[require_auth]
#[structured_log(
    fields = ["order_id", "warehouse_id", "shipping_method"],
    include_payload = false,
    include_response = true
)]
pub async fn fulfill_order(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    let fulfillment: FulfillmentRequest = msg.deserialize_payload()?;
    
    // Structured log output:
    // {
    //   "timestamp": "2023-12-07T10:30:00Z",
    //   "level": "info", 
    //   "service": "order_service",
    //   "method": "fulfill_order",
    //   "order_id": "ord_123",
    //   "warehouse_id": "wh_456", 
    //   "shipping_method": "express",
    //   "user_id": "usr_789",
    //   "trace_id": "abc123",
    //   "span_id": "def456"
    // }
    
    let result = process_fulfillment(order_id, fulfillment).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**Configuration Options**:
- **fields**: Custom fields to extract and log
- **include_payload**: Whether to log request payload
- **include_response**: Whether to log response data
- **sensitive_fields**: Fields to mask in logs

**When to Use**: For detailed observability and debugging of complex operations.

---

### `#[audit_log(action)]`

**Purpose**: Creates audit logs for compliance and security monitoring.

**Usage**:
```rust
#[service_method("DELETE /users/:id")]
#[require_auth]
#[require_role("admin")]
#[audit_log(action = "user_deletion")]
pub async fn delete_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let admin_user = msg.get_user_id()?;
    
    // Audit log entry:
    // {
    //   "timestamp": "2023-12-07T10:30:00Z",
    //   "event_type": "audit",
    //   "action": "user_deletion",
    //   "actor": {
    //     "user_id": "admin_123",
    //     "role": "admin",
    //     "ip_address": "192.168.1.100"
    //   },
    //   "resource": {
    //     "type": "user",
    //     "id": "user_456"
    //   },
    //   "result": "success",
    //   "session_id": "sess_789"
    // }
    
    delete_user_by_id(user_id).await?;
    Ok(RpcResponse::success(&"User deleted", 0)?)
}
```

**When to Use**: For security-sensitive operations requiring audit trails.

---

## Tracing Macros

### `#[trace]`

**Purpose**: Adds distributed tracing spans to service methods.

**Usage**:
```rust
#[service_method("GET /recommendations/:user_id")]
#[require_auth]
#[trace]
pub async fn get_recommendations(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("user_id")?;
    
    // Creates span: "recommendation_service.get_recommendations"
    // Automatically includes:
    // - trace_id, span_id, parent_span_id
    // - Service name and method name
    // - Start/end timestamps
    // - Success/error status
    
    let preferences = fetch_user_preferences(user_id).await?;
    let recommendations = generate_recommendations(user_id, preferences).await?;
    
    Ok(RpcResponse::success(&recommendations, 0)?)
}
```

**Span Attributes**:
- **service.name**: Service identifier
- **operation.name**: Method name  
- **http.method**: HTTP method
- **http.url**: Request URL
- **http.status_code**: Response status
- **user.id**: Authenticated user ID

**When to Use**: On all service methods for distributed tracing.

---

### `#[trace_async]`

**Purpose**: Optimized tracing for async operations with proper span management.

**Usage**:
```rust
#[service_method("POST /data/analysis")]
#[require_auth]
#[trace_async]
pub async fn analyze_data(msg: Message) -> Result<RpcResponse, String> {
    let analysis_request: AnalysisRequest = msg.deserialize_payload()?;
    
    // Span properly maintained across await points
    let data = fetch_large_dataset(analysis_request.dataset_id).await?;
    let results = perform_complex_analysis(data).await?;
    let report = generate_analysis_report(results).await?;
    
    Ok(RpcResponse::success(&report, 0)?)
}
```

**When to Use**: For long-running async operations with multiple await points.

---

### `#[trace_custom(name, attributes)]`

**Purpose**: Creates custom spans with specific names and attributes.

**Usage**:
```rust
#[service_method("POST /ml/training")]
#[require_auth]
#[trace_custom(
    name = "ml_model_training",
    attributes = ["model_type", "dataset_size", "training_algorithm"]
)]
pub async fn train_model(msg: Message) -> Result<RpcResponse, String> {
    let training_request: ModelTrainingRequest = msg.deserialize_payload()?;
    
    // Span: "ml_model_training" with custom attributes:
    // - model_type: "neural_network"
    // - dataset_size: 1000000
    // - training_algorithm: "gradient_descent"
    
    let model = train_ml_model(training_request).await?;
    Ok(RpcResponse::success(&model, 0)?)
}
```

**When to Use**: For domain-specific operations requiring specialized span information.

---

## Performance Monitoring Macros

### `#[performance_monitor]`

**Purpose**: Comprehensive performance monitoring with detailed timing breakdown.

**Usage**:
```rust
#[service_method("GET /complex-report")]
#[require_auth]
#[performance_monitor]
pub async fn generate_complex_report(msg: Message) -> Result<RpcResponse, String> {
    let report_params: ReportRequest = msg.deserialize_payload()?;
    
    // Automatically tracks timing for:
    // - Database queries
    // - External API calls  
    // - CPU-intensive operations
    // - Memory allocation
    
    let data = fetch_report_data(report_params).await?;
    let processed_data = process_report_data(data).await?;
    let report = format_report(processed_data).await?;
    
    // Performance metrics recorded:
    // - method_duration_total
    // - db_query_duration
    // - api_call_duration  
    // - cpu_usage_percent
    // - memory_usage_bytes
    
    Ok(RpcResponse::success(&report, 0)?)
}
```

**Monitored Resources**:
- **CPU Usage**: Method-level CPU consumption
- **Memory Usage**: Memory allocation tracking  
- **Database Performance**: Query timing and counts
- **External APIs**: Call latency and success rates
- **I/O Operations**: File and network I/O timing

**When to Use**: For performance-critical methods requiring detailed analysis.

---

### `#[resource_monitor]`

**Purpose**: Monitors system resource usage during method execution.

**Usage**:
```rust
#[service_method("POST /data/import")]
#[require_auth]
#[resource_monitor(
    track_memory = true,
    track_cpu = true,
    track_io = true,
    alert_thresholds = {
        "memory_mb": 1000,
        "cpu_percent": 80
    }
)]
pub async fn import_large_dataset(msg: Message) -> Result<RpcResponse, String> {
    let import_request: DataImportRequest = msg.deserialize_payload()?;
    
    // Resource monitoring active:
    // - Memory usage tracked
    // - CPU usage tracked
    // - I/O operations tracked
    // - Alerts sent if thresholds exceeded
    
    let result = process_large_import(import_request).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**When to Use**: For resource-intensive operations that need monitoring.

---

## Health Monitoring Macros

### `#[health_check]`

**Purpose**: Creates health check endpoints with dependency validation.

**Usage**:
```rust
#[service_impl]
impl UserService {
    #[health_check]
    async fn health() -> HealthStatus {
        let mut health = HealthStatus::new();
        
        // Automatically checks:
        // - Service responsiveness
        // - Database connectivity
        // - External service dependencies
        // - Cache availability
        // - Message queue connectivity
        
        health.add_check("database", check_database_connection().await);
        health.add_check("redis", check_redis_connection().await);
        health.add_check("external_api", check_external_api().await);
        
        health
    }
    
    #[health_check(path = "/ready")]
    async fn readiness() -> HealthStatus {
        let mut health = HealthStatus::new();
        health.add_check("service_started", true);
        health.add_check("dependencies_ready", check_all_dependencies().await);
        health
    }
}
```

**Generated Endpoints**:
- `GET /health` - Liveness check
- `GET /ready` - Readiness check  
- `GET /health/detailed` - Detailed health information

**When to Use**: Essential for container orchestration and load balancing.

---

### `#[circuit_breaker_monitor]`

**Purpose**: Monitors circuit breaker status and health.

**Usage**:
```rust
#[service_method("GET /external-data")]
#[circuit_breaker("external_api", failure_threshold = 5)]
#[circuit_breaker_monitor]
pub async fn get_external_data(msg: Message) -> Result<RpcResponse, String> {
    // Circuit breaker metrics automatically tracked:
    // - circuit_breaker_state{name="external_api"} (open/closed/half_open)
    // - circuit_breaker_failures_total{name="external_api"}
    // - circuit_breaker_successes_total{name="external_api"}
    
    let data = call_external_api().await?;
    Ok(RpcResponse::success(&data, 0)?)
}
```

**When to Use**: For monitoring resilience patterns and dependency health.

---

## Error Monitoring Macros

### `#[error_tracking]`

**Purpose**: Comprehensive error tracking and alerting.

**Usage**:
```rust
#[service_method("POST /payments/process")]
#[require_auth]
#[error_tracking(
    alert_on = ["PaymentDeclined", "InsufficientFunds"],
    include_stack_trace = true,
    notify_channels = ["slack", "email"]
)]
pub async fn process_payment(msg: Message) -> Result<RpcResponse, String> {
    let payment: PaymentRequest = msg.deserialize_payload()?;
    
    // Error tracking includes:
    // - Error type and message
    // - Stack trace (if enabled)
    // - Request context
    // - User information
    // - Alert notifications
    
    match charge_payment_card(payment).await {
        Ok(result) => Ok(RpcResponse::success(&result, 0)?),
        Err(PaymentError::Declined) => {
            // Alert sent to configured channels
            Err("Payment declined".to_string())
        }
        Err(e) => Err(e.to_string())
    }
}
```

**Error Information Captured**:
- Error type and message
- Stack trace and source location
- Request context and user information  
- Environment and service information
- Error frequency and patterns

**When to Use**: For critical operations requiring immediate error notification.

---

## Integration Examples

### Complete Observability Stack
```rust
#[service_method("POST /api/v1/orders")]
#[require_auth]
#[validate]
#[metrics(
    counter = "orders_total",
    histogram = "order_processing_duration",
    labels = ["payment_method", "user_tier"]
)]
#[structured_log(
    fields = ["order_id", "amount", "payment_method"],
    include_payload = false
)]
#[trace_async]
#[performance_monitor]
#[error_tracking(alert_on = ["PaymentFailed", "InventoryError"])]
#[audit_log(action = "order_creation")]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    
    // Full observability stack active:
    // 1. Metrics collection (counters, histograms, custom labels)
    // 2. Structured logging with custom fields
    // 3. Distributed tracing across async operations
    // 4. Performance monitoring (CPU, memory, I/O)
    // 5. Error tracking with alerting
    // 6. Audit logging for compliance
    
    let order = process_order_creation(order_data).await?;
    Ok(RpcResponse::success(&order, 0)?)
}
```

### Microservice Health Monitoring
```rust
#[service_impl]
impl OrderService {
    #[health_check]
    async fn health() -> HealthStatus {
        let mut health = HealthStatus::new();
        
        // Database health
        health.add_check("database", {
            check_postgres_connection().await
        });
        
        // Cache health  
        health.add_check("redis", {
            check_redis_connection().await
        });
        
        // External dependencies
        health.add_check("payment_service", {
            check_service_endpoint("http://payment-service/health").await
        });
        
        health.add_check("inventory_service", {
            check_service_endpoint("http://inventory-service/health").await
        });
        
        // Resource availability
        health.add_check("disk_space", {
            get_disk_usage().await < 0.9 // Less than 90% full
        });
        
        health.add_check("memory_usage", {
            get_memory_usage().await < 0.8 // Less than 80% used
        });
        
        health
    }
    
    #[service_method("GET /orders")]
    #[require_auth]
    #[metrics]
    #[log("info")]
    #[trace]
    pub async fn list_orders(msg: Message) -> Result<RpcResponse, String> {
        let orders = fetch_user_orders(msg.get_user_id()?).await?;
        Ok(RpcResponse::success(&orders, 0)?)
    }
}
```

### Performance-Critical Service
```rust
#[service_method("POST /ml/inference")]
#[require_auth]
#[metrics(
    histogram = "inference_duration_seconds",
    counter = "inference_requests_total",
    labels = ["model_version", "input_size_bucket"]
)]
#[performance_monitor]
#[resource_monitor(
    track_memory = true,
    track_cpu = true,
    alert_thresholds = {
        "memory_mb": 2000,
        "cpu_percent": 90
    }
)]
#[trace_custom(
    name = "ml_inference",
    attributes = ["model_id", "input_features", "confidence_threshold"]
)]
pub async fn run_inference(msg: Message) -> Result<RpcResponse, String> {
    let inference_request: InferenceRequest = msg.deserialize_payload()?;
    
    // Comprehensive monitoring for ML inference:
    // - Request timing and throughput metrics
    // - Resource usage monitoring with alerts
    // - Custom tracing for ML operations
    // - Performance profiling
    
    let model = load_model(&inference_request.model_id).await?;
    let predictions = model.predict(&inference_request.features).await?;
    
    Ok(RpcResponse::success(&InferenceResponse {
        predictions,
        model_version: model.version,
        confidence_scores: predictions.confidence,
    }, 0)?)
}
```

---

## Best Practices

### 1. Layer Observability Appropriately
```rust
// Basic endpoints
#[metrics]
#[log("info")]

// Business-critical endpoints
#[metrics]
#[structured_log]
#[trace]
#[audit_log]

// Performance-sensitive endpoints  
#[metrics]
#[trace]
#[performance_monitor]
#[resource_monitor]

// External service integrations
#[metrics]
#[trace]
#[circuit_breaker_monitor]
#[error_tracking]
```

### 2. Configure Appropriate Log Levels
```rust
// Development
#[log("debug")]

// Staging
#[log("info")]

// Production
#[log("warn")] // Only for error conditions
#[structured_log] // For detailed analysis
```

### 3. Use Custom Metrics for Business Logic
```rust
#[metrics(
    counter = "orders_processed_total",
    histogram = "order_value_distribution",
    labels = ["customer_segment", "product_category"]
)]
```

### 4. Implement Comprehensive Health Checks
```rust
#[health_check]
async fn health() -> HealthStatus {
    // Check all critical dependencies
    // Include resource availability
    // Test actual functionality, not just connectivity
}
```

---

This completes the Observability Macros documentation, providing complete visibility and monitoring capabilities for production microservices.