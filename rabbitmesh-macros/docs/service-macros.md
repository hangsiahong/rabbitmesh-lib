# Service Macros

Core macros for implementing microservices with automatic routing, lifecycle management, and method decoration.

## Overview

Service macros form the foundation of RabbitMesh applications, providing the core infrastructure for creating robust, scalable microservices with minimal boilerplate code.

---

## Core Service Macros

### `#[service_impl]`

**Purpose**: Transforms a regular impl block into a microservice implementation with automatic service registration and lifecycle management.

**Usage**:
```rust
use rabbitmesh_macros::service_impl;

#[service_impl]
impl UserService {
    // Service methods go here
}
```

**Generated Code**:
- Service trait implementation
- Automatic service registration
- Lifecycle management hooks
- Error handling infrastructure
- Message routing setup

**When to Use**: Apply to every service implementation block in your microservice.

---

### `#[service_method(route)]`

**Purpose**: Declares a method as a service endpoint with automatic HTTP-style routing.

**Syntax**: `#[service_method("METHOD /path/:param")]`

**Examples**:
```rust
#[service_method("GET /users/:id")]
pub async fn get_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    // Implementation
}

#[service_method("POST /users")]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}

#[service_method("PUT /users/:id/profile")]
pub async fn update_profile(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}

#[service_method("DELETE /users/:id")]
pub async fn delete_user(msg: Message) -> Result<RpcResponse, String> {
    // Implementation
}
```

**Route Patterns**:
- **Static paths**: `/users`, `/health`, `/api/v1/status`
- **Path parameters**: `/users/:id`, `/orders/:order_id/items/:item_id`
- **Wildcards**: `/files/*path` (captures remaining path)
- **Query parameters**: Handled via `msg.get_query_param()`

**Generated Code**:
- Route registration with service registry
- Path parameter extraction
- HTTP method validation
- Request/response serialization
- Error handling and status codes

**When to Use**: On every public method that should be accessible as a service endpoint.

---

### `#[async_handler]`

**Purpose**: Optimizes async method handling with proper futures management and cancellation support.

**Usage**:
```rust
#[service_method("GET /long-running")]
#[async_handler]
pub async fn long_running_operation(msg: Message) -> Result<RpcResponse, String> {
    // Long-running async operation
    tokio::time::sleep(Duration::from_secs(30)).await;
    Ok(RpcResponse::success(&"completed", 0)?)
}
```

**Features**:
- **Cancellation Support**: Proper handling of cancelled requests
- **Timeout Management**: Integration with timeout patterns
- **Resource Cleanup**: Automatic cleanup of async resources
- **Memory Optimization**: Optimized memory usage for long-running operations

**When to Use**: For methods that perform long-running async operations, I/O-heavy tasks, or operations that may be cancelled.

---

### `#[rpc_method]`

**Purpose**: Creates RPC-style method calls with automatic serialization and type safety.

**Usage**:
```rust
#[rpc_method]
pub async fn calculate_price(
    product_id: String,
    quantity: u32,
    discount: Option<f64>
) -> Result<PriceCalculation, ServiceError> {
    // Implementation
}
```

**Features**:
- **Type Safety**: Automatic serialization/deserialization
- **Parameter Validation**: Built-in parameter validation
- **Error Mapping**: Automatic error type conversion
- **Schema Generation**: Auto-generates OpenAPI schemas

**Generated Code**:
```rust
pub async fn calculate_price_handler(msg: Message) -> Result<RpcResponse, String> {
    let params: CalculatePriceParams = msg.deserialize_payload()?;
    let result = Self::calculate_price(params.product_id, params.quantity, params.discount).await?;
    RpcResponse::success(&result, 0).map_err(|e| e.to_string())
}
```

**When to Use**: For type-safe RPC-style methods with complex parameters.

---

### `#[event_handler(event_type)]`

**Purpose**: Creates event handlers for processing domain events with automatic deserialization.

**Usage**:
```rust
#[event_handler("user.created")]
pub async fn handle_user_created(event: UserCreatedEvent) -> Result<(), EventError> {
    // Send welcome email
    email_service.send_welcome_email(&event.email).await?;
    Ok(())
}

#[event_handler("order.completed")]
pub async fn handle_order_completed(event: OrderCompletedEvent) -> Result<(), EventError> {
    // Update inventory
    inventory_service.update_stock(&event.items).await?;
    Ok(())
}
```

**Features**:
- **Event Routing**: Automatic routing based on event type
- **Deserialization**: Type-safe event deserialization
- **Error Handling**: Built-in error handling and retries
- **Dead Letter Queues**: Automatic DLQ handling for failed events

**When to Use**: For implementing event-driven architecture and domain event processing.

---

### `#[message_handler(queue)]`

**Purpose**: Creates handlers for processing messages from specific queues.

**Usage**:
```rust
#[message_handler("user.notifications")]
pub async fn handle_notification(msg: NotificationMessage) -> Result<(), MessageError> {
    // Process notification
    notification_service.send(&msg).await?;
    Ok(())
}

#[message_handler("background.tasks")]
pub async fn handle_background_task(msg: BackgroundTask) -> Result<(), MessageError> {
    match msg.task_type {
        TaskType::DataExport => export_data(&msg.parameters).await?,
        TaskType::Cleanup => cleanup_old_data(&msg.parameters).await?,
        _ => return Err(MessageError::UnsupportedTaskType),
    }
    Ok(())
}
```

**When to Use**: For queue-based message processing and background job handling.

---

## Service Lifecycle Macros

### `#[on_startup]`

**Purpose**: Executes initialization logic when the service starts up.

**Usage**:
```rust
#[service_impl]
impl UserService {
    #[on_startup]
    async fn initialize() -> Result<(), StartupError> {
        // Database migrations
        run_migrations().await?;
        
        // Cache warming
        warm_user_cache().await?;
        
        // External service health checks
        verify_dependencies().await?;
        
        Ok(())
    }
}
```

**When to Use**: For database migrations, cache warming, dependency verification, and other startup tasks.

---

### `#[on_shutdown]`

**Purpose**: Executes cleanup logic during graceful shutdown.

**Usage**:
```rust
#[service_impl]
impl UserService {
    #[on_shutdown]
    async fn cleanup() -> Result<(), ShutdownError> {
        // Close database connections
        close_db_pool().await?;
        
        // Flush metrics
        flush_metrics().await?;
        
        // Complete ongoing operations
        wait_for_completion().await?;
        
        Ok(())
    }
}
```

**When to Use**: For graceful shutdown, resource cleanup, and ensuring data consistency.

---

### `#[health_check]`

**Purpose**: Implements health check endpoints for monitoring and orchestration.

**Usage**:
```rust
#[service_impl]
impl UserService {
    #[health_check]
    async fn health() -> HealthStatus {
        let mut status = HealthStatus::new();
        
        // Check database connectivity
        status.add_check("database", check_database().await);
        
        // Check external dependencies
        status.add_check("auth_service", check_auth_service().await);
        
        // Check cache availability
        status.add_check("redis", check_redis().await);
        
        status
    }
    
    #[health_check(path = "/ready")]
    async fn readiness() -> HealthStatus {
        // Readiness-specific checks
        let mut status = HealthStatus::new();
        status.add_check("initialized", self.is_initialized());
        status
    }
}
```

**Generated Endpoints**:
- `GET /health` - Liveness check
- `GET /ready` - Readiness check (if specified)
- `GET /health/detailed` - Detailed health information

**When to Use**: Essential for Kubernetes deployments, load balancers, and monitoring systems.

---

## Advanced Service Macros

### `#[service_context]`

**Purpose**: Provides dependency injection and shared service context.

**Usage**:
```rust
#[service_context]
struct UserServiceContext {
    database: Arc<DatabasePool>,
    cache: Arc<RedisCache>,
    email_service: Arc<EmailService>,
    metrics: Arc<MetricsCollector>,
}

#[service_impl]
impl UserService {
    #[service_method("POST /users")]
    pub async fn create_user(
        msg: Message,
        ctx: &UserServiceContext
    ) -> Result<RpcResponse, String> {
        // Use injected dependencies
        let user = ctx.database.create_user(user_data).await?;
        ctx.cache.set_user(user.id, &user).await?;
        ctx.email_service.send_welcome(&user.email).await?;
        ctx.metrics.increment("users_created");
        
        Ok(RpcResponse::success(&user, 0)?)
    }
}
```

**When to Use**: For dependency injection, shared resources, and complex service configurations.

---

### `#[middleware(name)]`

**Purpose**: Applies middleware to service methods for cross-cutting concerns.

**Usage**:
```rust
#[service_impl]
impl UserService {
    #[service_method("GET /users/:id")]
    #[middleware("request_logging")]
    #[middleware("rate_limiting")]
    pub async fn get_user(msg: Message) -> Result<RpcResponse, String> {
        // Method implementation
    }
}
```

**Common Middleware**:
- **request_logging**: Structured request/response logging
- **rate_limiting**: Request rate limiting
- **cors**: Cross-Origin Resource Sharing
- **compression**: Response compression
- **security_headers**: Security headers injection

**When to Use**: For implementing cross-cutting concerns like logging, security, and rate limiting.

---

## Best Practices

### 1. Method Organization
```rust
#[service_impl]
impl UserService {
    // Lifecycle methods first
    #[on_startup]
    async fn startup() -> Result<(), StartupError> { /* */ }
    
    #[health_check]
    async fn health() -> HealthStatus { /* */ }
    
    // Public API methods
    #[service_method("GET /users/:id")]
    pub async fn get_user(msg: Message) -> Result<RpcResponse, String> { /* */ }
    
    // Event handlers
    #[event_handler("user.updated")]
    async fn handle_user_updated(event: UserUpdatedEvent) -> Result<(), EventError> { /* */ }
    
    // Private helper methods (no macros)
    async fn validate_user_data(data: &UserData) -> Result<(), ValidationError> { /* */ }
}
```

### 2. Error Handling
```rust
#[service_method("POST /users")]
pub async fn create_user(msg: Message) -> Result<RpcResponse, String> {
    let user_data: CreateUserRequest = msg.deserialize_payload()
        .map_err(|e| format!("Invalid request: {}", e))?;
    
    let user = create_user_in_db(user_data).await
        .map_err(|e| format!("Database error: {}", e))?;
    
    RpcResponse::success(&user, 0)
        .map_err(|e| format!("Response serialization error: {}", e))
}
```

### 3. Resource Management
```rust
#[service_impl]
impl UserService {
    #[on_startup]
    async fn initialize() -> Result<(), StartupError> {
        // Initialize resources that need cleanup
        initialize_connection_pools().await?;
        Ok(())
    }
    
    #[on_shutdown]
    async fn cleanup() -> Result<(), ShutdownError> {
        // Ensure proper cleanup
        close_all_connections().await?;
        Ok(())
    }
}
```

---

## Performance Considerations

1. **Async Efficiency**: Use `#[async_handler]` for I/O-intensive operations
2. **Resource Pooling**: Initialize expensive resources in `#[on_startup]`
3. **Graceful Shutdown**: Implement proper cleanup in `#[on_shutdown]`
4. **Health Checks**: Keep health checks lightweight and fast
5. **Error Handling**: Use efficient error handling patterns

---

## Integration Examples

### Complete Service Implementation
```rust
use rabbitmesh_macros::*;

#[service_context]
struct UserServiceContext {
    database: Arc<DatabasePool>,
    cache: Arc<RedisCache>,
}

#[service_impl]
impl UserService {
    #[on_startup]
    async fn startup() -> Result<(), StartupError> {
        run_migrations().await?;
        Ok(())
    }
    
    #[health_check]
    async fn health() -> HealthStatus {
        HealthStatus::healthy()
    }
    
    #[service_method("GET /users/:id")]
    #[require_auth]
    #[cached(ttl = 300)]
    pub async fn get_user(
        msg: Message,
        ctx: &UserServiceContext
    ) -> Result<RpcResponse, String> {
        let user_id = msg.get_path_param("id")?;
        let user = ctx.database.get_user(&user_id).await?;
        Ok(RpcResponse::success(&user, 0)?)
    }
    
    #[event_handler("user.created")]
    async fn handle_user_created(event: UserCreatedEvent) -> Result<(), EventError> {
        send_welcome_email(&event.email).await?;
        Ok(())
    }
}
```

---

This completes the Service Macros documentation. These macros form the foundation of every RabbitMesh microservice, providing the essential infrastructure for building robust, scalable services.