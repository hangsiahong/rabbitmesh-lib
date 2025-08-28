# 🚀 Blog Platform - Service-to-Service Communication Demo

This demonstrates **advanced microservices architecture** with **service-to-service communication** using RabbitMesh.

## 🏗️ Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐
│   Client    │────│ API Gateway │────│         Microservices          │
│             │HTTP│  (Port 3334)│AMQP│         (NO PORTS!)            │
└─────────────┘    └─────────────┘    │                                 │
                          │           │  ┌─────────────────────────┐  │
                          │           │  │      RabbitMQ           │  │
                          └───────────┼──│   Message Broker        │  │
                                      │  └─────────────────────────┘  │
                                      │           ↕ ↕ ↕               │
                                      │  ┌─────┐ ┌─────┐ ┌─────────┐  │
                                      │  │Auth │ │Post │ │Comment  │  │
                                      │  │Svc  │ │Svc  │ │Service  │  │
                                      │  │:0   │ │:0   │ │:0       │  │
                                      │  └──┬──┘ └──┬──┘ └──┬──────┘  │
                                      │     └────────┼──────┘         │
                                      │              │                │
                                      └──────────────┼────────────────┘
                                                     ↓
                                          Service-to-Service 
                                           Communication
```

## 📦 Services with Clean File Structure

### 🔐 Auth Service (`auth-service/`)
```
auth-service/src/
├── main.rs     # Binary entry point
├── service.rs  # RabbitMesh handlers & business logic  
├── models.rs   # User, AuthResponse, etc.
├── utils.rs    # Password hashing, token generation
└── config.rs   # Service configuration
```

**Handles:** User registration, login, token validation

### 📝 Post Service (`post-service/`)
```
post-service/src/
├── main.rs     # Binary entry point  
├── service.rs  # Post handlers + auth integration
├── models.rs   # Post, CreatePostRequest, etc.
├── utils.rs    # Auth service communication
└── config.rs   # Service configuration
```

**Handles:** Blog posts, **calls Auth Service** for user verification

### 💬 Comment Service (`comment-service/`)
```
comment-service/src/
├── main.rs     # Binary entry point
├── service.rs  # Comment handlers + multi-service integration  
├── models.rs   # Comment, CreateCommentRequest, etc.
├── utils.rs    # Auth + Post service communication
└── config.rs   # Service configuration
```

**Handles:** Comments, **calls Auth Service** AND **Post Service**

### 🌐 API Gateway (`api-gateway/`)
Auto-generates REST endpoints for all services.

## 🔗 Service-to-Service Communication Flow

### Creating a Comment (Multi-Service Workflow):
```
1. Client → Gateway → Comment Service
2. Comment Service → Auth Service (validate token)
3. Comment Service → Post Service (verify post exists)  
4. Comment Service → Response → Gateway → Client
```

**Zero direct HTTP calls between services - all via RabbitMQ!**

## 🏁 Running the Demo

### Quick Start with Makefile (Recommended)

```bash
# See all available commands
make help

# Start everything (RabbitMQ + all services)  
make start

# Check service status
make status

# View service logs
make logs

# Stop everything
make stop

# Test API endpoints
make test-api

# Full development setup
make dev
```

### Manual Setup (Alternative)

<details>
<summary>Click to expand manual setup instructions</summary>

#### 1. Start RabbitMQ
```bash
docker run -d -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

#### 2. Start Services (4 terminals)

**Terminal 1 - Auth Service:**
```bash
cd auth-service && cargo run
```

**Terminal 2 - Post Service:**
```bash
cd post-service && cargo run  
```

**Terminal 3 - Comment Service:**
```bash
cd comment-service && cargo run
```

**Terminal 4 - API Gateway:**
```bash
cd api-gateway && cargo run
```

</details>

## 🌟 Test Service-to-Service Communication

### Quick API Test
```bash
# Run automated API tests
make test-api
```

### Manual API Testing

### 1. Register & Login (Get Auth Token)
```bash
# Register new user
curl -X POST http://localhost:3334/api/v1/auth-service/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@blog.com","password":"secret123"}'

# Login to get token
curl -X POST http://localhost:3334/api/v1/auth-service/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"secret123"}'
```

### 2. Create Blog Post (Post Service → Auth Service)
```bash
# Use token from login response
curl -X POST http://localhost:3334/api/v1/post-service/create_post \
  -H 'Content-Type: application/json' \
  -d '{"title":"My First Post","content":"Hello RabbitMesh!","token":"<YOUR_TOKEN>","tags":["intro","tech"]}'
```

### 3. Create Comment (Comment Service → Auth + Post Services)
```bash
# Get a post ID first
curl http://localhost:3334/api/v1/post-service/list_posts

# Create comment (triggers multi-service calls!)
curl -X POST http://localhost:3334/api/v1/comment-service/create_comment \
  -H 'Content-Type: application/json' \
  -d '{"post_id":"<POST_ID>","content":"Great post!","token":"<YOUR_TOKEN>"}'
```

### 4. View Results
```bash
# List all posts
curl http://localhost:3334/api/v1/post-service/list_posts

# List all comments  
curl http://localhost:3334/api/v1/comment-service/list_comments

# Get comments for specific post
curl -X POST http://localhost:3334/api/v1/comment-service/get_comments_by_post \
  -H 'Content-Type: application/json' \
  -d '{"post_id":"<POST_ID>"}'
```

## 🔥 Key Features Demonstrated

### ✅ **Clean File Organization**
- `main.rs`: Binary entry point
- `service.rs`: Business logic & RabbitMesh handlers
- `models.rs`: Data structures & types
- `utils.rs`: Helper functions & inter-service calls
- `config.rs`: Configuration constants

### ✅ **Service-to-Service Communication**
- Comment Service validates tokens with Auth Service
- Comment Service verifies posts with Post Service
- All communication via RabbitMQ messages (zero HTTP!)

### ✅ **Authentication Flow**
- JWT-style token generation & validation
- Password hashing with bcrypt
- Token-based authorization across services

### ✅ **Fault Tolerance**  
- Services can start in any order
- Built-in retries and timeouts for inter-service calls
- Graceful error handling with detailed messages

### ✅ **Zero Port Management**
- Only API Gateway exposes HTTP port (3334)
- All services communicate via RabbitMQ queues
- Easy to scale and deploy independently

## 🚀 This is the Future of Microservices!

**Traditional:** Complex service mesh, HTTP calls, port management, load balancers

**RabbitMesh:** Simple message-driven architecture, automatic load balancing, zero configuration

Each service is a clean, focused binary that processes messages. That's it! 🔥