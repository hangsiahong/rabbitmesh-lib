# 🚀 RabbitMesh Ecommerce Microservices Demo

This demonstrates the **correct microservices architecture** where each service is a separate binary:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐
│   Client    │────│ API Gateway │────│        Microservices           │
│             │HTTP│  (Port 3000)│AMQP│        (NO PORTS!)             │
└─────────────┘    └─────────────┘    │                                 │
                          │           │  ┌─────────────────────────┐  │
                          │           │  │      RabbitMQ           │  │
                          └───────────┼──│   Message Broker        │  │
                                      │  └─────────────────────────┘  │
                                      │      ↕       ↕       ↕        │
                                      │  ┌─────┐ ┌─────┐ ┌─────┐      │
                                      │  │User │ │Prod │ │Order│      │
                                      │  │Svc  │ │Svc  │ │Svc  │      │
                                      │  │:0   │ │:0   │ │:0   │      │
                                      │  └─────┘ └─────┘ └─────┘      │
                                      └─────────────────────────────────┘
```

## 📦 Services

### 🚀 Each Service = Separate Binary

- **`user-service/`** - User management microservice (NO HTTP PORT)
- **`product-service/`** - Product catalog microservice (NO HTTP PORT)  
- **`api-gateway/`** - HTTP API gateway (Port 3000 ONLY)

## 🏁 Running the Demo

### 1. Start RabbitMQ
```bash
docker run -d -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

### 2. Start Each Service Separately

**Terminal 1 - User Service:**
```bash
cd user-service
cargo run
```

**Terminal 2 - Product Service:**
```bash
cd product-service
cargo run
```

**Terminal 3 - API Gateway:**
```bash
cd api-gateway
cargo run
```

## 🌟 Test the APIs

```bash
# Health checks
curl http://localhost:3000/health
curl http://localhost:3000/health/user-service
curl http://localhost:3000/health/product-service

# User operations (gateway calls user-service via RabbitMQ)
curl http://localhost:3000/api/v1/user-service/list_users
curl http://localhost:3000/api/v1/user-service/get_user?user_id=1

curl -X POST http://localhost:3000/api/v1/user-service/create_user \
  -H 'Content-Type: application/json' \
  -d '{"name":"Alice","email":"alice@example.com"}'

# Product operations (gateway calls product-service via RabbitMQ)
curl http://localhost:3000/api/v1/product-service/list_products  
curl http://localhost:3000/api/v1/product-service/get_product?product_id=1
curl "http://localhost:3000/api/v1/product-service/get_products_by_category?category=Electronics"

curl -X POST http://localhost:3000/api/v1/product-service/create_product \
  -H 'Content-Type: application/json' \
  -d '{"name":"Book","description":"Programming book","price":29.99,"category":"Books"}'
```

## 🔥 Key Architecture Benefits

### ✅ **Zero Port Management**
- User Service: Listens to `rabbitmesh.user-service` queue
- Product Service: Listens to `rabbitmesh.product-service` queue
- API Gateway: Only HTTP port (3000) in entire system

### ✅ **True Microservices**
- Each service is independent binary
- Services can be deployed separately
- Scale services independently
- Different teams can own different services

### ✅ **Message-Driven Communication**
```
HTTP Request -> Gateway -> RabbitMQ Message -> Service -> RabbitMQ Response -> Gateway -> HTTP Response
```

### ✅ **Auto Load Balancing**
Run multiple instances of any service:
```bash
# Terminal 4 - Second user service instance
cd user-service
cargo run
```
RabbitMQ automatically distributes load between instances!

### ✅ **Fault Tolerance**
- Stop any service → Requests queue up and wait
- Restart service → Processes queued requests automatically
- Network issues → Built-in retries and circuit breakers

## 🎯 Production Deployment

Each service can be deployed independently:

```dockerfile
# user-service/Dockerfile
FROM rust:1.70 as builder
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/user-service /usr/local/bin/
CMD ["user-service"]
```

Deploy anywhere:
- Docker containers
- Kubernetes pods  
- Bare metal servers
- Cloud functions

Services automatically discover each other via RabbitMQ queues!

## 🚀 This is Revolutionary

**Traditional microservices:** Complex service discovery, port management, load balancing

**RabbitMesh microservices:** Just connect to RabbitMQ - everything else is automatic!

Each service is a simple binary that processes messages. That's it.

**The future of microservices is message-driven!** 🔥