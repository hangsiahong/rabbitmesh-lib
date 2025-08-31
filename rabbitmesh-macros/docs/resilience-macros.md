# Resilience Macros

Production-ready resilience patterns including circuit breakers, retries with backoff, timeouts, bulkheads, and graceful degradation for fault-tolerant microservices.

## Overview

Resilience macros implement proven fault tolerance patterns that protect your services from cascading failures, handle transient errors gracefully, and maintain service availability under adverse conditions.

---

## Circuit Breaker Macros

### `#[circuit_breaker(name)]`

**Purpose**: Implements the circuit breaker pattern to prevent cascading failures when calling external services.

**Usage**:
```rust
#[service_method("GET /user-recommendations")]
#[require_auth]
#[circuit_breaker("recommendation_service")]
pub async fn get_recommendations(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_user_id()?;
    
    // Circuit breaker automatically:
    // - Monitors failure rate
    // - Opens circuit after threshold failures
    // - Allows periodic test requests
    // - Closes circuit when service recovers
    
    let recommendations = call_recommendation_service(user_id).await?;
    Ok(RpcResponse::success(&recommendations, 0)?)
}
```

**Circuit States**:
- **Closed**: Normal operation, all requests pass through
- **Open**: Service unavailable, requests fail fast
- **Half-Open**: Testing recovery, limited requests allowed

**Default Configuration**:
- **Failure Threshold**: 5 failures in 10 requests
- **Recovery Timeout**: 30 seconds
- **Success Threshold**: 3 consecutive successes to close

**When to Use**: For all external service calls and unreliable dependencies.

---

### `#[circuit_breaker(name, config)]`

**Purpose**: Circuit breaker with custom configuration for specific failure scenarios.

**Usage**:
```rust
#[service_method("POST /payments/charge")]
#[require_auth]
#[circuit_breaker("payment_processor", {
    failure_threshold = 3,
    failure_rate_threshold = 0.5,
    minimum_requests = 10,
    recovery_timeout = "60s",
    success_threshold = 5
})]
pub async fn charge_payment(msg: Message) -> Result<RpcResponse, String> {
    let payment: PaymentRequest = msg.deserialize_payload()?;
    
    // Custom circuit breaker for payment service:
    // - Opens after 50% failure rate with min 10 requests
    // - Stays open for 60 seconds
    // - Requires 5 consecutive successes to close
    
    let result = process_payment_charge(payment).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**Configuration Options**:
- **failure_threshold**: Number of failures to trigger open
- **failure_rate_threshold**: Percentage of failures (0.0-1.0)
- **minimum_requests**: Minimum requests before calculating rate
- **recovery_timeout**: How long circuit stays open
- **success_threshold**: Successes needed to close circuit

**When to Use**: For critical services requiring fine-tuned failure handling.

---

### `#[fallback(handler)]`

**Purpose**: Provides fallback responses when circuit breaker is open.

**Usage**:
```rust
#[service_method("GET /product-reviews")]
#[circuit_breaker("review_service")]
#[fallback("cached_reviews_fallback")]
pub async fn get_product_reviews(msg: Message) -> Result<RpcResponse, String> {
    let product_id = msg.get_path_param("product_id")?;
    
    // Primary: Fetch from review service
    let reviews = fetch_reviews_from_service(product_id).await?;
    Ok(RpcResponse::success(&reviews, 0)?)
}

// Fallback function
async fn cached_reviews_fallback(msg: Message) -> Result<RpcResponse, String> {
    let product_id = msg.get_path_param("product_id")?;
    
    // Fallback: Return cached reviews or default message
    let cached_reviews = get_cached_reviews(product_id).await
        .unwrap_or_else(|_| vec![]);
    
    if cached_reviews.is_empty() {
        Ok(RpcResponse::success(&json!({
            "message": "Reviews temporarily unavailable",
            "reviews": []
        }), 0)?)
    } else {
        Ok(RpcResponse::success(&cached_reviews, 0)?)
    }
}
```

**When to Use**: When you can provide meaningful degraded functionality during outages.

---

## Retry Macros

### `#[retry(attempts)]`

**Purpose**: Automatically retries failed operations with configurable attempts and backoff.

**Usage**:
```rust
#[service_method("POST /data/sync")]
#[require_auth]
#[retry(attempts = 3)]
pub async fn sync_data(msg: Message) -> Result<RpcResponse, String> {
    let sync_request: DataSyncRequest = msg.deserialize_payload()?;
    
    // Automatically retries up to 3 times on:
    // - Network timeouts
    // - Temporary service unavailability  
    // - Transient database errors
    // - 5xx HTTP responses
    
    let result = perform_data_synchronization(sync_request).await?;
    Ok(RpcResponse::success(&result, 0)?)
}
```

**Default Retry Behavior**:
- **Exponential Backoff**: 1s, 2s, 4s delays
- **Jitter**: Random delay variation to prevent thundering herd
- **Retryable Errors**: Network, timeout, 5xx responses
- **Non-Retryable**: 4xx client errors, authentication failures

**When to Use**: For operations that may fail due to transient issues.

---

### `#[retry(strategy)]`

**Purpose**: Advanced retry configuration with custom strategies.

**Usage**:
```rust
#[service_method("POST /external/api-call")]
#[require_auth]
#[retry({
    attempts = 5,
    strategy = "exponential",
    base_delay = "500ms",
    max_delay = "30s",
    jitter = true,
    retry_on = ["timeout", "connection_error", "rate_limit"],
    dont_retry_on = ["authentication_error", "validation_error"]
})]
pub async fn call_external_api(msg: Message) -> Result<RpcResponse, String> {
    let api_request: ExternalApiRequest = msg.deserialize_payload()?;
    
    // Advanced retry with:
    // - 5 attempts with exponential backoff
    // - Base delay 500ms, max 30s
    // - Jitter to prevent synchronization
    // - Specific error conditions for retry
    
    let response = call_third_party_api(api_request).await?;
    Ok(RpcResponse::success(&response, 0)?)
}
```

**Retry Strategies**:
- **fixed**: Fixed delay between attempts
- **linear**: Linearly increasing delay
- **exponential**: Exponentially increasing delay
- **custom**: Custom backoff function

**When to Use**: For complex external integrations requiring specific retry behavior.

---

### `#[idempotent]`

**Purpose**: Ensures operations can be safely retried without side effects.

**Usage**:
```rust
#[service_method("POST /orders/:id/confirm")]
#[require_auth]
#[retry(attempts = 3)]
#[idempotent]
pub async fn confirm_order(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    
    // Operation is idempotent:
    // - Multiple calls with same parameters produce same result
    // - Safe to retry without unintended side effects
    // - Uses idempotency keys to prevent duplicate processing
    
    let confirmation = confirm_order_processing(order_id).await?;
    Ok(RpcResponse::success(&confirmation, 0)?)
}
```

**Idempotency Features**:
- **Request Deduplication**: Uses request fingerprinting
- **Idempotency Keys**: Supports client-provided keys
- **Result Caching**: Caches results for duplicate requests
- **State Tracking**: Prevents partial state changes

**When to Use**: For state-changing operations that may be retried.

---

## Timeout Macros

### `#[timeout(duration)]`

**Purpose**: Sets maximum execution time for operations to prevent hanging requests.

**Usage**:
```rust
#[service_method("GET /reports/complex")]
#[require_auth]
#[timeout("30s")]
pub async fn generate_complex_report(msg: Message) -> Result<RpcResponse, String> {
    let report_params: ReportRequest = msg.deserialize_payload()?;
    
    // Operation automatically cancelled after 30 seconds
    // Prevents:
    // - Resource exhaustion from hanging operations
    // - Poor user experience from slow responses
    // - Cascade failures from accumulated delays
    
    let report = generate_detailed_report(report_params).await?;
    Ok(RpcResponse::success(&report, 0)?)
}
```

**Timeout Formats**:
- **Seconds**: `"30s"`, `"2.5s"`
- **Minutes**: `"5m"`, `"1.5m"`  
- **Milliseconds**: `"500ms"`, `"1500ms"`

**When to Use**: For all operations with external dependencies or potential for long execution.

---

### `#[timeout_cascade(parent_timeout)]`

**Purpose**: Implements cascading timeouts that respect parent operation timeouts.

**Usage**:
```rust
#[service_method("GET /user-dashboard")]
#[require_auth]
#[timeout("10s")]
pub async fn get_user_dashboard(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_user_id()?;
    
    // Calls to sub-operations inherit reduced timeouts
    let profile = get_user_profile(user_id).await?;      // Max 3s
    let orders = get_recent_orders(user_id).await?;       // Max 3s  
    let recommendations = get_recommendations(user_id).await?; // Max 3s
    
    Ok(RpcResponse::success(&DashboardResponse {
        profile,
        orders,
        recommendations,
    }, 0)?)
}

#[timeout_cascade(percentage = 30)]
async fn get_user_profile(user_id: String) -> Result<UserProfile, String> {
    // Inherits 30% of parent timeout (3s out of 10s)
    fetch_user_profile_data(user_id).await
}
```

**When to Use**: For complex operations with multiple sub-operations.

---

## Bulkhead Macros

### `#[bulkhead(pool)]`

**Purpose**: Isolates resources to prevent failure propagation using the bulkhead pattern.

**Usage**:
```rust
#[service_method("POST /orders")]
#[require_auth]
#[bulkhead("order_processing", capacity = 50)]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order: CreateOrderRequest = msg.deserialize_payload()?;
    
    // Order processing isolated to dedicated resource pool:
    // - Max 50 concurrent order operations
    // - Prevents other operations from starving order processing
    // - Prevents order processing from overwhelming system
    
    let result = process_new_order(order).await?;
    Ok(RpcResponse::success(&result, 0)?)
}

#[service_method("GET /search")]
#[bulkhead("search_queries", capacity = 100)]
pub async fn search_products(msg: Message) -> Result<RpcResponse, String> {
    let query: SearchRequest = msg.deserialize_payload()?;
    
    // Search isolated to separate resource pool:
    // - Max 100 concurrent search operations
    // - Heavy search queries don't impact order processing
    
    let results = perform_product_search(query).await?;
    Ok(RpcResponse::success(&results, 0)?)
}
```

**Bulkhead Types**:
- **Thread Pool**: Separate thread pools for different operations
- **Connection Pool**: Dedicated database connections
- **Memory Pool**: Isolated memory allocation
- **CPU Time**: CPU time slice allocation

**When to Use**: To prevent resource contention between different operation types.

---

### `#[bulkhead_priority(pool, priority)]`

**Purpose**: Assigns priority levels within bulkhead resource pools.

**Usage**:
```rust
#[service_method("POST /orders/express")]
#[require_auth]
#[bulkhead_priority("order_processing", priority = "high")]
pub async fn create_express_order(msg: Message) -> Result<RpcResponse, String> {
    let order: ExpressOrderRequest = msg.deserialize_payload()?;
    
    // High priority within order processing pool:
    // - Gets preferential resource allocation
    // - Can preempt lower priority operations
    
    let result = process_express_order(order).await?;
    Ok(RpcResponse::success(&result, 0)?)
}

#[service_method("POST /orders/batch")]
#[require_auth]  
#[bulkhead_priority("order_processing", priority = "low")]
pub async fn create_batch_orders(msg: Message) -> Result<RpcResponse, String> {
    let orders: BatchOrderRequest = msg.deserialize_payload()?;
    
    // Low priority - uses resources when available
    let results = process_batch_orders(orders).await?;
    Ok(RpcResponse::success(&results, 0)?)
}
```

**Priority Levels**: `high`, `normal`, `low`, `background`

**When to Use**: When different operations have different criticality levels.

---

## Rate Limiting & Throttling Macros

### `#[rate_limit(limit)]`

**Purpose**: Controls request rate to prevent system overload and ensure fair resource usage.

**Usage**:
```rust
#[service_method("POST /api/data/upload")]
#[require_auth]
#[rate_limit(requests = 10, per = "minute", by = "user")]
pub async fn upload_data(msg: Message) -> Result<RpcResponse, String> {
    let upload_data: DataUploadRequest = msg.deserialize_payload()?;
    
    // Rate limiting:
    // - 10 uploads per minute per user
    // - Prevents abuse of upload functionality
    // - Ensures fair resource allocation
    
    let result = process_data_upload(upload_data).await?;
    Ok(RpcResponse::success(&result, 0)?)
}

#[service_method("GET /public-api/search")]
#[rate_limit(requests = 1000, per = "hour", by = "api_key")]
pub async fn public_search(msg: Message) -> Result<RpcResponse, String> {
    let query: SearchQuery = msg.deserialize_payload()?;
    
    // API key based rate limiting
    let results = perform_public_search(query).await?;
    Ok(RpcResponse::success(&results, 0)?)
}
```

**Rate Limiting Options**:
- **by**: `user`, `ip`, `api_key`, `header:X-Client-ID`
- **per**: `second`, `minute`, `hour`, `day`
- **algorithm**: `token_bucket`, `sliding_window`, `fixed_window`

**When to Use**: For public APIs, resource-intensive operations, and abuse prevention.

---

### `#[throttle(max_concurrent)]`

**Purpose**: Limits concurrent execution to prevent resource exhaustion.

**Usage**:
```rust
#[service_method("POST /data/analysis")]
#[require_auth]
#[throttle(max_concurrent = 5)]
pub async fn analyze_large_dataset(msg: Message) -> Result<RpcResponse, String> {
    let analysis_request: AnalysisRequest = msg.deserialize_payload()?;
    
    // Maximum 5 concurrent analysis operations:
    // - Prevents memory exhaustion from too many large operations
    // - Queues additional requests
    // - Returns 503 Service Unavailable if queue is full
    
    let results = perform_intensive_analysis(analysis_request).await?;
    Ok(RpcResponse::success(&results, 0)?)
}
```

**When to Use**: For resource-intensive operations that could overwhelm the system.

---

## Graceful Degradation Macros

### `#[degrade_gracefully]`

**Purpose**: Provides reduced functionality when services are under stress or dependencies are unavailable.

**Usage**:
```rust
#[service_method("GET /product/:id")]
#[circuit_breaker("recommendation_service")]
#[degrade_gracefully]
pub async fn get_product_details(msg: Message) -> Result<RpcResponse, String> {
    let product_id = msg.get_path_param("id")?;
    
    // Always provide core product information
    let product = fetch_product_details(product_id).await?;
    
    // Enhanced features with graceful degradation:
    let mut response = ProductResponse {
        id: product.id,
        name: product.name,
        price: product.price,
        description: product.description,
        recommendations: None,
        reviews_summary: None,
        inventory_status: None,
    };
    
    // Try to add enhanced features, degrade gracefully on failure
    if let Ok(recommendations) = get_product_recommendations(product_id).await {
        response.recommendations = Some(recommendations);
    }
    
    if let Ok(reviews) = get_reviews_summary(product_id).await {
        response.reviews_summary = Some(reviews);
    }
    
    if let Ok(inventory) = check_inventory_status(product_id).await {
        response.inventory_status = Some(inventory);
    }
    
    Ok(RpcResponse::success(&response, 0)?)
}
```

**When to Use**: For user-facing services where partial functionality is better than complete failure.

---

### `#[adaptive_timeout]`

**Purpose**: Dynamically adjusts timeouts based on system load and response times.

**Usage**:
```rust
#[service_method("GET /dynamic-content")]
#[require_auth]
#[adaptive_timeout(
    base_timeout = "5s",
    max_timeout = "30s",
    adjustment_factor = 0.1
)]
pub async fn get_dynamic_content(msg: Message) -> Result<RpcResponse, String> {
    let request: ContentRequest = msg.deserialize_payload()?;
    
    // Timeout adapts based on:
    // - Recent response time trends
    // - Current system load
    // - Dependency performance
    // - Time of day patterns
    
    let content = generate_dynamic_content(request).await?;
    Ok(RpcResponse::success(&content, 0)?)
}
```

**When to Use**: For operations with variable execution times based on load or data size.

---

## Integration Examples

### Complete Resilience Stack
```rust
#[service_method("POST /critical-operation")]
#[require_auth]
#[validate]
#[bulkhead("critical_ops", capacity = 10)]
#[circuit_breaker("external_service", {
    failure_threshold = 3,
    recovery_timeout = "60s"
})]
#[retry({
    attempts = 3,
    strategy = "exponential",
    base_delay = "1s"
})]
#[timeout("30s")]
#[fallback("critical_operation_fallback")]
#[rate_limit(requests = 100, per = "hour", by = "user")]
#[idempotent]
#[metrics]
#[audit_log]
pub async fn critical_operation(msg: Message) -> Result<RpcResponse, String> {
    let request: CriticalOperationRequest = msg.deserialize_payload()?;
    
    // Complete resilience protection:
    // 1. Resource isolation with bulkhead
    // 2. Circuit breaker for external dependencies  
    // 3. Retry with exponential backoff
    // 4. Timeout protection
    // 5. Fallback functionality
    // 6. Rate limiting for abuse prevention
    // 7. Idempotency for safe retries
    // 8. Full observability
    
    let result = perform_critical_operation(request).await?;
    Ok(RpcResponse::success(&result, 0)?)
}

async fn critical_operation_fallback(msg: Message) -> Result<RpcResponse, String> {
    // Fallback: Return cached result or safe default
    Ok(RpcResponse::success(&json!({
        "status": "degraded",
        "message": "Operation completed with reduced functionality"
    }), 0)?)
}
```

### Microservice Integration Patterns
```rust
#[service_method("GET /user-profile/:id")]
#[require_auth]
#[timeout("10s")]
pub async fn get_user_profile(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    
    // Parallel calls with individual resilience
    let (profile_result, preferences_result, activity_result) = tokio::join!(
        get_basic_profile(user_id.clone()),
        get_user_preferences(user_id.clone()),  
        get_user_activity(user_id.clone())
    );
    
    // Combine results, handling partial failures gracefully
    let mut response = UserProfileResponse::default();
    
    match profile_result {
        Ok(profile) => response.basic_profile = Some(profile),
        Err(e) => tracing::warn!("Profile fetch failed: {}", e),
    }
    
    match preferences_result {
        Ok(preferences) => response.preferences = Some(preferences),
        Err(e) => tracing::warn!("Preferences fetch failed: {}", e),
    }
    
    match activity_result {
        Ok(activity) => response.recent_activity = Some(activity),
        Err(e) => tracing::warn!("Activity fetch failed: {}", e),
    }
    
    Ok(RpcResponse::success(&response, 0)?)
}

#[circuit_breaker("profile_service")]
#[retry(attempts = 2)]
#[timeout_cascade(percentage = 30)]
async fn get_basic_profile(user_id: String) -> Result<BasicProfile, String> {
    call_profile_service(user_id).await
}

#[circuit_breaker("preferences_service")]
#[fallback("default_preferences")]
#[timeout_cascade(percentage = 30)]
async fn get_user_preferences(user_id: String) -> Result<UserPreferences, String> {
    call_preferences_service(user_id).await
}

#[circuit_breaker("activity_service")]
#[fallback("empty_activity")]
#[timeout_cascade(percentage = 30)]
async fn get_user_activity(user_id: String) -> Result<UserActivity, String> {
    call_activity_service(user_id).await
}
```

### High-Throughput Service Protection
```rust
#[service_method("POST /api/events")]
#[rate_limit(requests = 10000, per = "minute", by = "api_key")]
#[throttle(max_concurrent = 100)]
#[bulkhead("event_processing", capacity = 200)]
#[timeout("5s")]
pub async fn ingest_events(msg: Message) -> Result<RpcResponse, String> {
    let events: EventBatch = msg.deserialize_payload()?;
    
    // High-throughput protection:
    // - Rate limiting per API key
    // - Concurrent request throttling
    // - Resource isolation
    // - Fast timeout to prevent queue buildup
    
    let results = process_event_batch(events).await?;
    Ok(RpcResponse::success(&results, 0)?)
}

#[service_method("GET /analytics/realtime")]
#[bulkhead_priority("analytics", priority = "low")]
#[adaptive_timeout(base_timeout = "2s", max_timeout = "10s")]
#[degrade_gracefully]
pub async fn get_realtime_analytics(msg: Message) -> Result<RpcResponse, String> {
    let query: AnalyticsQuery = msg.deserialize_payload()?;
    
    // Analytics with graceful degradation:
    // - Low priority to not impact core operations
    // - Adaptive timeouts based on load
    // - Graceful degradation for partial results
    
    let analytics = compute_realtime_analytics(query).await?;
    Ok(RpcResponse::success(&analytics, 0)?)
}
```

---

## Best Practices

### 1. Layer Resilience Patterns
```rust
// External service calls
#[circuit_breaker("service_name")]
#[retry(attempts = 3)]
#[timeout("10s")]
#[fallback("service_fallback")]

// Resource-intensive operations
#[bulkhead("heavy_ops")]
#[throttle(max_concurrent = 5)]
#[timeout("60s")]

// Public APIs
#[rate_limit(requests = 1000, per = "hour")]
#[throttle(max_concurrent = 50)]
#[timeout("30s")]
```

### 2. Configure Appropriate Timeouts
```rust
// Quick operations
#[timeout("2s")]

// Database queries
#[timeout("5s")]

// External API calls
#[timeout("10s")]

// Heavy processing
#[timeout("60s")]

// Batch operations
#[timeout("300s")]
```

### 3. Use Cascading Patterns
```rust
#[timeout("30s")]
pub async fn parent_operation() {
    // Child operations inherit reduced timeouts
    child_operation_1().await?; // ~10s timeout
    child_operation_2().await?; // ~10s timeout
    child_operation_3().await?; // ~10s timeout
}
```

### 4. Implement Proper Fallbacks
```rust
#[circuit_breaker("service")]
#[fallback("meaningful_fallback")]
pub async fn service_call() {
    // Primary implementation
}

async fn meaningful_fallback() -> Result<Response, Error> {
    // Return cached data, default values, or degraded functionality
    // Never just return an error - provide value when possible
}
```

---

This completes the Resilience Macros documentation, providing comprehensive fault tolerance and reliability patterns for production microservices.