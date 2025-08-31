# RabbitMesh Macros

A comprehensive collection of procedural macros for building robust, production-ready microservices with RabbitMQ-based messaging, event sourcing, CQRS patterns, and advanced observability.

## 🚀 Overview

RabbitMesh Macros provides 50+ powerful procedural macros that automatically generate boilerplate code and implement complex patterns for microservice architectures. These macros handle everything from service method decoration to advanced workflow orchestration, authentication, caching, and resilience patterns.

## 📖 Documentation Structure

This documentation is organized into focused sections for easy navigation:

### Core Service Macros
- [**Service Macros**](docs/service-macros.md) - Core service implementation, method routing, and lifecycle management
- [**Authentication Macros**](docs/auth-macros.md) - JWT validation, RBAC, ABAC, and security patterns

### Data & State Management
- [**Validation Macros**](docs/validation-macros.md) - Input validation, schema validation, and data integrity
- [**Event Sourcing Macros**](docs/event-sourcing-macros.md) - CQRS, aggregates, projections, and event stores
- [**Caching Macros**](docs/caching-macros.md) - Distributed caching, cache strategies, and cache invalidation
- [**Database Macros**](docs/database-macros.md) - Transaction management, connection pooling, and ORM patterns

### Operations & Reliability
- [**Observability Macros**](docs/observability-macros.md) - Metrics, logging, tracing, and monitoring
- [**Resilience Macros**](docs/resilience-macros.md) - Circuit breakers, retries, timeouts, and bulkheads
- [**Workflow Macros**](docs/workflow-macros.md) - State machines, orchestration, and choreography

## 🎯 Quick Start

Add RabbitMesh Macros to your `Cargo.toml`:

```toml
[dependencies]
rabbitmesh-macros = "0.2.0"
rabbitmesh = "0.2.0"
```

### Basic Service Example

```rust
use rabbitmesh_macros::{service_impl, service_method, require_auth, validate, metrics, cached};
use rabbitmesh::{MicroService, Message, RpcResponse};

#[service_impl]
impl UserService {
    #[service_method("GET /users/:id")]
    #[require_auth]
    #[validate]
    #[cached(ttl = 300)]
    #[metrics]
    pub async fn get_user(msg: Message) -> Result<RpcResponse, String> {
        // Your service logic here
        let user_id = msg.get_path_param("id")?;
        
        // Fetch user from database
        let user = fetch_user_by_id(&user_id).await?;
        
        RpcResponse::success(&user, 0).map_err(|e| e.to_string())
    }
}
```

## 🏗️ Macro Categories

### 1. Service Foundation Macros
Create and manage microservice implementations with automatic routing, lifecycle management, and method decoration.

### 2. Authentication & Authorization
Comprehensive security macros for JWT token validation, role-based access control (RBAC), attribute-based access control (ABAC), and permission management.

### 3. Data Validation & Processing
Input validation, schema validation, data transformation, and integrity checking macros.

### 4. Event Sourcing & CQRS
Complete event sourcing implementation with command/query separation, aggregate management, event stores, and read model projections.

### 5. Caching Strategies
Multi-level caching with Redis, in-memory, distributed cache coordination, and intelligent cache invalidation.

### 6. Observability & Monitoring
Comprehensive observability stack with Prometheus metrics, structured logging, distributed tracing, and custom monitoring.

### 7. Resilience Patterns
Production-ready resilience patterns including circuit breakers, retries with backoff, timeouts, bulkheads, and graceful degradation.

### 8. Workflow & State Management
Advanced workflow orchestration with state machines, saga patterns, choreography, and long-running process management.

### 9. Database Operations
Transaction management, connection pooling, query optimization, and database migration support.

## 🌟 Key Features

- **Zero Boilerplate**: Automatically generate complex implementations
- **Production Ready**: Battle-tested patterns and error handling
- **Composable**: Mix and match macros for your specific needs
- **Type Safe**: Full Rust type safety with compile-time guarantees
- **Performance Optimized**: Minimal runtime overhead
- **Async First**: Built for modern async Rust applications
- **Observability Native**: Built-in metrics, logging, and tracing
- **Cloud Native**: Kubernetes-ready with health checks and graceful shutdown

## 🔧 Architecture

RabbitMesh Macros are built using:
- **Procedural Macros**: Code generation at compile time
- **Token Streams**: AST manipulation for clean code generation  
- **Attribute Parsing**: Flexible macro configuration
- **Quote**: Template-based code generation
- **Syn**: Rust syntax tree parsing

## 📊 Macro Statistics

- **50+ Macros**: Comprehensive coverage of microservice patterns
- **9 Categories**: Well-organized functionality groups
- **Type Safe**: Full compile-time validation
- **Zero Runtime Cost**: All code generated at compile time
- **Async Native**: Built for tokio and async-std

## 🚀 Getting Started

1. **Choose Your Pattern**: Browse the documentation sections above
2. **Import Macros**: Add the macros you need to your service
3. **Configure**: Use attributes to customize behavior
4. **Build**: Let the macros generate your implementation
5. **Deploy**: Your service is ready for production

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](../CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🆘 Support

- **Documentation**: Check the category-specific docs above
- **Issues**: [GitHub Issues](https://github.com/hangsiahong/rabbitmesh-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hangsiahong/rabbitmesh-rs/discussions)

---

*Built with ❤️ for the Rust microservices community*