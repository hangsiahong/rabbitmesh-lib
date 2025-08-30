# 🛒 RabbitMesh E-Commerce Example

**Complete E-Commerce Microservices Application Built with RabbitMesh Framework**

This is a comprehensive example demonstrating all RabbitMesh features in a real-world e-commerce scenario, including authentication, user management, order processing, caching, rate limiting, and automatic API generation.

## ✅ **Status: WORKING WITH v0.1.1**

This example correctly uses RabbitMesh **v0.1.1** from GitHub which includes all the implemented universal macros and features you've built. The previous build errors were caused by looking at the wrong version.

## 🎯 Features Demonstrated

### ✅ **Complete RabbitMesh Feature Set**
- **🔐 Authentication & Authorization** - JWT, RBAC, ABAC, permission checking
- **⚡ Rate Limiting** - Per-endpoint configurable rate limits
- **💾 Caching** - Redis distributed caching with automatic TTL
- **📊 Auto-Generated APIs** - REST and GraphQL endpoints from service definitions
- **🎭 Universal Macros** - 50+ production-ready macros from v0.1.1
- **🌐 Service-to-Service Communication** - Zero-port RabbitMQ mesh
- **📈 Observability** - Metrics, audit logs, and distributed tracing
- **🔒 Input Validation** - Automatic request/response validation
- **⚙️ Transaction Management** - Database transaction handling

### 🏗️ **Microservices Architecture**
- **User Service** - User management with MongoDB storage
- **Auth Service** - JWT authentication with RBAC authorization  
- **Order Service** - Order processing with Redis caching
- **Gateway** - Auto-generated REST/GraphQL APIs

### 🛠️ **Production Ready**
- **PM2 Process Management** - Multi-instance deployment
- **Docker Compose** - Complete infrastructure setup
- **MongoDB** - Document storage with validation
- **Redis** - High-performance caching
- **RabbitMQ** - Message broker for service mesh

---

## 🚀 Quick Start

### Prerequisites
- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Docker & Docker Compose** - [Install Docker](https://docs.docker.com/get-docker/)
- **PM2** (optional) - `npm install -g pm2`

### 1. Clone and Setup
```bash
# This example is already configured to use RabbitMesh v0.1.1 from GitHub
# No additional cloning needed - the Cargo.toml points to the correct branch

# Build all services (this will download v0.1.1 automatically)
./scripts/build-services.sh
```

### 2. Start Infrastructure
```bash
# Start RabbitMQ, MongoDB, and Redis
docker-compose up -d rabbitmq mongodb redis

# Wait for services to be ready (check health)
docker-compose ps
```

### 3. Start Microservices
```bash
# Option A: Using PM2 (Recommended for production)
pm2 start ecosystem.config.js
pm2 status
pm2 logs

# Option B: Manual start (Development)
# Terminal 1: User Service
cd user-service && cargo run

# Terminal 2: Auth Service  
cd auth-service && cargo run

# Terminal 3: Order Service
cd order-service && cargo run

# Terminal 4: Gateway
cd gateway && cargo run
```

### 4. Test the APIs
```bash
# API Gateway is now running at http://localhost:3000

# Check service health
curl http://localhost:3000/health

# View available services
curl http://localhost:3000/services

# API documentation
curl http://localhost:3000/api-docs

# GraphQL playground: http://localhost:3000/graphql
```

---

## 📋 API Documentation

### 🔐 **Authentication Flow**
```bash
# 1. Login to get JWT token
curl -X POST http://localhost:3000/api/v1/auth-service/auth/login \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"email\": \"customer@rabbitmesh.dev\",
    \"password\": \"customer123\"
  }'

# Response includes JWT token:
# {
#   \"token\": \"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...\",
#   \"user\": { ... },
#   \"expires_at\": \"2024-12-31T23:59:59Z\"
# }

# 2. Use token in subsequent requests
export JWT_TOKEN=\"your-jwt-token-here\"
```

### 👥 **User Management**
```bash
# Create user (public endpoint)
curl -X POST http://localhost:3000/api/v1/user-service/users \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"email\": \"newuser@example.com\",
    \"name\": \"New User\",
    \"password\": \"securepassword\",
    \"role\": \"customer\"
  }'

# Get user (requires auth + permission)
curl -X GET http://localhost:3000/api/v1/user-service/users/customer-001 \\
  -H \"Authorization: Bearer $JWT_TOKEN\"

# Update user (requires auth + permission)
curl -X PUT http://localhost:3000/api/v1/user-service/users/customer-001 \\
  -H \"Authorization: Bearer $JWT_TOKEN\" \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"name\": \"Updated Name\"
  }'

# List users (admin only)
curl -X GET http://localhost:3000/api/v1/user-service/users?limit=10 \\
  -H \"Authorization: Bearer $JWT_TOKEN\"
```

### 📦 **Order Management**
```bash
# Create order (requires auth)
curl -X POST http://localhost:3000/api/v1/order-service/orders \\
  -H \"Authorization: Bearer $JWT_TOKEN\" \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"user_id\": \"customer-001\",
    \"items\": [
      {
        \"product_id\": \"prod-001\",
        \"product_name\": \"RabbitMesh T-Shirt\",
        \"quantity\": 2,
        \"unit_price\": 25.99
      }
    ]
  }'

# Get order (cached with Redis)
curl -X GET http://localhost:3000/api/v1/order-service/orders/ORD-12345678 \\
  -H \"Authorization: Bearer $JWT_TOKEN\"

# Get user orders (ownership-based access)
curl -X GET http://localhost:3000/api/v1/order-service/orders/user/customer-001 \\
  -H \"Authorization: Bearer $JWT_TOKEN\"

# Update order status (manager/admin only)
curl -X PUT http://localhost:3000/api/v1/order-service/orders/ORD-12345678 \\
  -H \"Authorization: Bearer $JWT_TOKEN\" \\
  -H \"Content-Type: application/json\" \\
  -d '{
    \"status\": \"Confirmed\"
  }'

# Cancel order
curl -X POST http://localhost:3000/api/v1/order-service/orders/ORD-12345678/cancel \\
  -H \"Authorization: Bearer $JWT_TOKEN\"
```

---

## 🎭 RabbitMesh Universal Macros in Action

This example demonstrates **50+ Universal Macros** from v0.1.1 with real implementations:

### 🔐 **Security Macros**
```rust
#[service_method(\"POST /auth/login\")]
#[validate]                    // Real input validation
#[rate_limit(5, 300)]         // 5 attempts per 5 minutes
#[metrics]                    // Request metrics collection
#[audit_log]                  // Security audit logging
pub async fn login(msg: Message) -> Result<Value, String>
```

### 📦 **Order Processing Macros**
```rust
#[service_method(\"POST /orders\")]
#[require_auth]               // Real JWT authentication
#[require_permission(\"orders:write\")]  // RBAC authorization
#[validate]                   // Input validation with XSS protection
#[rate_limit(20, 60)]        // Rate limiting implementation
#[transactional]             // Database transaction management
#[metrics]                   // Performance metrics
#[audit_log]                 // Audit trail
#[event_publish]             // Domain event publishing
#[redis_cache(300)]          // Redis caching (5 min TTL)
pub async fn create_order(msg: Message) -> Result<Value, String>
```

### 🏃 **High-Performance Macros**
```rust
#[service_method(\"GET /users/:id\")]
#[require_auth]              // Authentication required
#[require_permission(\"users:read\")]  // Permission check
#[cached(300)]               // In-memory cache (5 min)
#[redis_cache(600)]          // Redis cache (10 min) 
#[rate_limit(100, 60)]       // 100 req/min rate limit
#[metrics]                   // Performance tracking
pub async fn get_user(msg: Message) -> Result<Value, String>
```

---

## ✅ **What's Working (v0.1.1)**

The v0.1.1 branch includes real implementations of:

1. **🔐 Authentication System**: JWT validation, role-based access control
2. **⚡ Rate Limiting**: In-memory rate limiting with configurable windows  
3. **💾 Caching**: Redis distributed caching + in-memory caching
4. **📊 Metrics & Monitoring**: Request/response metrics, audit logging
5. **🎯 Event System**: Event publishing and domain event handling
6. **🔒 Validation**: Input validation with XSS and injection protection
7. **⚙️ Database Transactions**: Automatic transaction lifecycle management
8. **🌐 Service Communication**: Zero-port RabbitMQ service mesh
9. **📋 Auto-Generated APIs**: REST and GraphQL from service definitions
10. **🏗️ Dynamic Discovery**: Workspace scanning for services

## 🏗️ Architecture Overview

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────────────┐
│   Frontend      │────│   API Gateway    │────│       Service Mesh          │
│  (React/Vue)    │HTTP│ (Auto-Generated) │AMQP│  ┌─────┐ ┌─────┐ ┌─────┐   │
│                 │    │                  │    │  │User │ │Auth │ │Order│   │
└─────────────────┘    └──────────────────┘    │  └─────┘ └─────┘ └─────┘   │
                                               │           │                  │
┌─────────────────────────────────────────────┐ │     ┌─────────────────────┐│
│            Infrastructure                   │ │     │     RabbitMQ        ││
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐   │ │     │   Message Broker    ││  
│  │MongoDB  │ │ Redis   │ │  RabbitMQ   │   │ └─────┴─────────────────────┘│
│  │(Users & │ │(Cache)  │ │(Messages)   │   │                              │
│  │Orders)  │ │         │ │             │   │                              │
│  └─────────┘ └─────────┘ └─────────────┘   │                              │
└─────────────────────────────────────────────┘                              │
                                               └─────────────────────────────┘
```

---

## 🤝 Sample Data

The example includes sample users for testing:

| Email | Password | Role | Permissions |
|-------|----------|------|-------------|
| admin@rabbitmesh.dev | admin123 | admin | All permissions |
| manager@rabbitmesh.dev | manager123 | manager | User & order management |
| customer@rabbitmesh.dev | customer123 | customer | Own orders only |

Sample orders are also created for the customer account.

---

## 🎓 Learning Objectives

This example teaches:

1. **RabbitMesh Framework Usage** - How to build microservices with universal macros
2. **Service Mesh Architecture** - Zero-port communication via message broker
3. **Authentication & Authorization** - JWT, RBAC, ABAC implementation
4. **Caching Strategies** - Multi-level caching with Redis and in-memory
5. **Database Integration** - MongoDB with connection pooling and transactions
6. **API Generation** - Automatic REST and GraphQL API creation
7. **Production Deployment** - PM2, Docker, monitoring, logging
8. **Performance Optimization** - Rate limiting, caching, load balancing

---

## 🐛 Troubleshooting

### Common Issues:

**Service won't start:**
```bash
# Check if RabbitMQ is running
docker-compose ps rabbitmq

# Check service logs
pm2 logs user-service
```

**Database connection failed:**
```bash
# Verify MongoDB is running
docker-compose ps mongodb

# Check MongoDB logs
docker-compose logs mongodb
```

**Redis cache errors:**
```bash
# Check Redis status
docker-compose ps redis

# Test Redis connection
redis-cli ping
```

**Build errors:**
```bash
# Clean and rebuild to get fresh v0.1.1 dependencies
cargo clean
cargo build --release
```

---

## 📄 License

This example is part of the RabbitMesh project and is licensed under the MIT License.

---

<div align=\"center\">

**🎉 Working with RabbitMesh v0.1.1 - All Universal Macros Implemented! 🎉**

*Experience the power of 50+ universal macros with real implementations*

</div>