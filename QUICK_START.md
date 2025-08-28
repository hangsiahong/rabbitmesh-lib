# 🚀 RabbitMesh Quick Start Guide

Get up and running with RabbitMesh microservices in minutes!

## 🎯 What is RabbitMesh?

RabbitMesh is a **zero-port microservices framework** that uses RabbitMQ for service communication instead of HTTP. Services are defined with simple Rust macros, and everything else is auto-generated.

### ✨ The Magic
- **Write only business logic** - Framework handles routing, serialization, RPC calls
- **Zero configuration** - Routes extracted from code annotations  
- **Auto-generated gateway** - HTTP API created automatically from service definitions
- **Zero ports** - Services communicate via RabbitMQ only
- **Auto-discovery** - Services find each other automatically

## 🏃‍♂️ Quick Start

### 1. **Use the AI Agent** (Recommended)
```bash
# Interactive mode - ask for anything!
python rabbitmesh_agent.py

# Command line mode
python rabbitmesh_agent.py "build me an authentication service"
python rabbitmesh_agent.py help
python rabbitmesh_agent.py examples
```

### 2. **Try the Working Example**
```bash
# Start RabbitMQ
docker run -d --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3.12-management

# Run the Todo Service
cd examples/simple-todo
cargo run --bin todo-service

# In another terminal - start gateway (when implemented)
cargo run --bin todo-gateway
```

### 3. **Create Your Own Service**

```rust
// Cargo.toml
[dependencies]
rabbitmesh = { path = "../../rabbitmesh" }
rabbitmesh-macros = { path = "../../rabbitmesh-macros" }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"

// src/main.rs
use rabbitmesh_macros::{service_definition, service_impl};

#[service_definition]
pub struct MyService;

#[service_impl]
impl MyService {
    #[service_method("POST /items")]
    pub async fn create_item(request: CreateItemRequest) -> Result<ItemResponse, String> {
        // JUST YOUR BUSINESS LOGIC!
        // Framework handles everything else
        Ok(ItemResponse::success("Item created"))
    }

    #[service_method("GET /items/:id")]
    pub async fn get_item(id: String) -> Result<ItemResponse, String> {
        // Clean, simple method signature
        Ok(ItemResponse::success(&format!("Item {}", id)))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize service
    let rabbitmq_url = "amqp://guest:guest@localhost:5672/%2f";
    let service = MyService::create_service(&rabbitmq_url).await?;
    
    // Start service - that's it!
    service.start().await?;
    Ok(())
}
```

## 🎪 Framework Features

### Auto-Generated Routes
```rust
#[service_method("GET /users/:id")]     // ✅ Path parameters
#[service_method("POST /users")]        // ✅ Request body
#[service_method("PUT /users/:id")]     // ✅ Path + body (tuple)
#[service_method("DELETE /users/:id")]  // ✅ Simple operations
```

### Clean Method Signatures  
```rust
// Simple parameter
async fn get_user(user_id: String) -> Result<UserResponse, String>

// Complex request
async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String>  

// Multiple parameters (path param + request body)
async fn update_user(params: (String, UpdateUserRequest)) -> Result<UserResponse, String>
```

### Zero Configuration Output
```
✅ MyService started successfully  
🎯 Service name: MyService-service
📊 Auto-discovered 5 HTTP routes:
   create_item -> POST /items
   get_item -> GET /items/:id
   list_items -> GET /items
   update_item -> PUT /items/:id
   delete_item -> DELETE /items/:id
```

## 🏗️ Architecture Benefits

### Traditional Microservices
```
Service A:8080 ←HTTP→ Service B:8081 ←HTTP→ Service C:8082
```
❌ Port management  
❌ Service discovery  
❌ Load balancing config  
❌ Network complexity  

### RabbitMesh
```
Service A ←RabbitMQ→ Service B ←RabbitMQ→ Service C
         ↘            ↓            ↙
           Auto-Generated Gateway:3000 ←HTTP← Clients
```
✅ Zero port management  
✅ Automatic service discovery  
✅ Built-in load balancing  
✅ Services run anywhere  

## 🤖 Get Help from the AI Agent

The **RabbitMesh Agent** knows everything about the framework:

```bash
# Ask for anything!
python rabbitmesh_agent.py "authentication service with JWT"
python rabbitmesh_agent.py "e-commerce platform with orders and payments"
python rabbitmesh_agent.py "real-time chat system"

# Learn the framework
python rabbitmesh_agent.py magic        # Explain the magic
python rabbitmesh_agent.py examples     # Show code examples  
python rabbitmesh_agent.py architecture # Explain zero-port architecture
```

## 📚 Learn More

- **Simple Todo Example**: `examples/simple-todo/` - Perfect starting point
- **Agent Documentation**: `RABBITMESH_AGENT.md` - AI assistant guide
- **Examples Summary**: `EXAMPLES_SUMMARY.md` - What works and what doesn't

## 🎯 Next Steps

1. **Try the agent**: `python rabbitmesh_agent.py`
2. **Build something**: Ask the agent to generate code for your use case  
3. **Run the example**: Test the Todo service
4. **Build your service**: Use the patterns you learned

**Welcome to the future of microservices!** 🚀