# 🚀 RabbitMesh Ecommerce Demo

Complete microservices ecommerce system showcasing the power of **message-driven architecture**.

## 🎯 What This Demonstrates

- **Zero-Port Microservices** - Services only connect to RabbitMQ (no HTTP ports)
- **Auto-Generated API Gateway** - REST endpoints created automatically
- **Inter-Service Communication** - Order service calls User & Product services via RabbitMQ
- **Horizontal Scaling** - Run multiple instances, RabbitMQ load balances automatically
- **Fault Tolerance** - Message persistence, retries, circuit breakers built-in

## 🏗️ Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐
│   Client    │────│ API Gateway │────│        Microservices           │
│ (HTTP/JSON) │    │  (Port 3000)│    │        (NO PORTS!)             │
└─────────────┘    └─────────────┘    │                                 │
                          │           │  ┌─────────────────────────┐  │
                          │           │  │      RabbitMQ           │  │
                          └───────────┼──│   Message Broker        │  │
                                      │  └─────────────────────────┘  │
                                      │      ↕       ↕       ↕        │
                                      │  ┌─────┐ ┌─────┐ ┌─────┐      │
                                      │  │User │ │Prod │ │Order│      │
                                      │  │Svc  │ │Svc  │ │Svc  │      │
                                      │  └─────┘ └─────┘ └─────┘      │
                                      └─────────────────────────────────┘
```

## 🚀 Quick Start

### 1. Start RabbitMQ
```bash
docker run -d --name rabbitmq \
  -p 5672:5672 -p 15672:15672 \
  rabbitmq:3-management
```

### 2. Run the Demo
```bash
cd examples/ecommerce
cargo run
```

### 3. Test the APIs

**Health Checks:**
```bash
curl http://localhost:3000/health
curl http://localhost:3000/health/user-service
```

**User Operations:**
```bash
# List users
curl http://localhost:3000/api/v1/user-service/list_users

# Get specific user
curl http://localhost:3000/api/v1/user-service/get_user/1

# Create new user
curl -X POST http://localhost:3000/api/v1/user-service/create_user \
  -H 'Content-Type: application/json' \
  -d '{"name":"Alice","email":"alice@example.com"}'
```

**Product Operations:**
```bash
# List all products
curl http://localhost:3000/api/v1/product-service/list_products

# Get specific product
curl http://localhost:3000/api/v1/product-service/get_product/1

# Get products by category
curl http://localhost:3000/api/v1/product-service/get_products_by_category/Electronics
```

**Order Operations (Inter-Service Communication):**
```bash
# Create order (calls User + Product services internally)
curl -X POST http://localhost:3000/api/v1/order-service/create_order \
  -H 'Content-Type: application/json' \
  -d '{"user_id":1,"items":[{"product_id":1,"quantity":1}]}'

# List all orders
curl http://localhost:3000/api/v1/order-service/list_orders

# Get orders for specific user
curl http://localhost:3000/api/v1/order-service/list_user_orders/1
```

## 🔥 Key Features Demonstrated

### 1. **Zero Port Management**
- ✅ User Service: No HTTP port, connects only to RabbitMQ
- ✅ Product Service: No HTTP port, connects only to RabbitMQ  
- ✅ Order Service: No HTTP port, connects only to RabbitMQ
- ✅ Only the Gateway exposes port 3000

### 2. **Inter-Service Communication**
When you create an order, watch the logs - you'll see:
1. Order service receives HTTP→RabbitMQ request
2. Order service calls User service via RabbitMQ to verify user exists
3. Order service calls Product service via RabbitMQ to get product details
4. All communication is asynchronous and fault-tolerant

### 3. **Automatic Load Balancing**
Run multiple instances of any service:
```bash
# Terminal 1: Run first instance
cargo run

# Terminal 2: Run second instance  
RUST_LOG=info cargo run
```
RabbitMQ automatically distributes requests between instances!

### 4. **Fault Tolerance**
- Stop any service → Requests queue up and wait
- Restart service → Processes queued requests automatically  
- Network issues → Built-in retries and circuit breakers

## 🎯 What Makes This Special

**Traditional Microservices Problems:**
- ❌ Each service needs its own port
- ❌ Complex service discovery (Consul, etcd, etc.)
- ❌ Manual load balancer configuration
- ❌ Network failures cause cascading failures
- ❌ Manual API gateway setup for each service

**RabbitMesh Solutions:**
- ✅ Services only connect to RabbitMQ - zero port management
- ✅ Automatic service discovery via queue routing
- ✅ Built-in load balancing via RabbitMQ queues
- ✅ Message persistence handles network failures gracefully  
- ✅ Auto-generated API gateway with zero configuration

## 🔧 Customization

### Add New Microservice
1. Create new service struct with `#[service_definition]`
2. Add methods with `#[service_method("HTTP route")]`
3. Register handlers and call `service.start()`
4. Gateway automatically exposes new service!

### Scale Horizontally
Just run more instances - they automatically join the mesh and share load.

### Monitor Services
- RabbitMQ Management UI: http://localhost:15672 (guest/guest)
- Service health: `curl http://localhost:3000/health/:service`

This demo shows how RabbitMesh **eliminates microservice complexity** while providing **enterprise-grade reliability**.