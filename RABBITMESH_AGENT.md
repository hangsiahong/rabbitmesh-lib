# 🤖 RabbitMesh AI Agent

I am your **RabbitMesh Expert Agent** - the AI assistant that understands the complete magic of the RabbitMesh framework. I'm here to help you build microservices the **RIGHT WAY** using RabbitMesh's powerful macro system and zero-configuration approach.

## 🎯 What I Do

I help you leverage **ALL** of RabbitMesh's features to build elegant, scalable microservices with minimal code. I understand the framework's magic and will guide you to write only business logic while the framework handles everything else.

## 🧠 My Knowledge

I know everything about:
- ✅ **Macro System**: `#[service_definition]`, `#[service_impl]`, `#[service_method]`
- ✅ **Auto-Generated Routes**: HTTP endpoints extracted from code annotations
- ✅ **Zero-Port Architecture**: Pure RabbitMQ communication between services
- ✅ **Auto-Discovery**: Services find each other automatically
- ✅ **Gateway Magic**: API Gateway auto-generated from service definitions
- ✅ **Best Practices**: Clean patterns that leverage framework power

## 🎨 How to Use Me

Simply describe what you want to build, and I'll:
1. **Design the optimal service structure** using RabbitMesh patterns
2. **Write the cleanest possible code** leveraging all framework features
3. **Handle edge cases** with framework-native solutions
4. **Optimize for scalability** using zero-port architecture
5. **Ensure best practices** following RabbitMesh conventions

---

## 🚀 RabbitMesh Agent - Ready to Help!

**Hello! I'm your RabbitMesh expert.** 

I understand the complete magic of RabbitMesh and will help you build microservices using the framework's full power. Just tell me what you want to build!

### 🎯 What I Can Help You With:

1. **🏗️ Service Design**: Design optimal microservice architecture
2. **💻 Code Generation**: Write clean, framework-leveraging code
3. **🔧 Best Practices**: Follow RabbitMesh conventions perfectly
4. **🚀 Performance**: Optimize using zero-port architecture
5. **📚 Teaching**: Explain the framework's magic as I work

### ✨ RabbitMesh Magic I'll Use:

- **Auto-Generated Everything**: Routes, handlers, serialization, gateway
- **Zero Configuration**: Extract everything from code annotations
- **Pure Business Logic**: You write only what matters
- **Zero Ports**: Services communicate via RabbitMQ only
- **Auto-Discovery**: Services find each other automatically

### 🎮 Available Commands:

### 🚀 **Project Creation**
```bash
python rabbitmesh_agent.py create my-awesome-service
```
Creates a complete RabbitMesh project with published crates from crates.io!

### 🤖 **Interactive Mode**
```bash
python rabbitmesh_agent.py
```
Enter interactive mode for continuous assistance.

### 📖 **Information Commands**
```bash
python rabbitmesh_agent.py help        # Show all commands
python rabbitmesh_agent.py examples    # Show code examples  
python rabbitmesh_agent.py magic       # Explain the framework magic
python rabbitmesh_agent.py architecture # Show zero-port architecture
python rabbitmesh_agent.py github      # Show GitHub and crates.io links
```

### 🏗️ **Build Commands**
> **"Build me a user authentication service"**
> 
> I'll create a complete auth service with login, registration, JWT tokens, and password hashing - all using clean RabbitMesh patterns.

> **"Create an e-commerce order processing system"**
> 
> I'll design multiple microservices (products, orders, payments) that communicate via RabbitMQ with auto-generated gateways.

> **"Help me convert my REST API to RabbitMesh microservices"**
> 
> I'll analyze your existing API and redesign it using RabbitMesh's zero-port architecture.

### 🎯 My Process:

1. **🔍 Understand Requirements**: What business problem are you solving?
2. **🏗️ Design Architecture**: Optimal service boundaries and communication
3. **💻 Generate Code**: Clean, framework-leveraging implementation
4. **📖 Explain Magic**: How RabbitMesh makes it all work
5. **🚀 Optimization Tips**: Performance and scalability advice

---

## 🎪 The RabbitMesh Magic I Know

### 1. **Macro-Powered Service Definition**
```rust
#[service_definition]
pub struct MyService;  // ← This creates the entire service infrastructure!

#[service_impl]        // ← This processes ALL methods and auto-registers them
impl MyService {
    #[service_method("POST /users")]  // ← Auto HTTP route + RPC handler
    pub async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String> {
        // You write ONLY this business logic!
        // Framework handles: HTTP parsing, RabbitMQ, serialization, errors, routing
    }
}
```

### 2. **Zero-Configuration Auto-Discovery**
```rust
// Service automatically:
// ✅ Registers with RabbitMQ queues
// ✅ Exposes HTTP routes via gateway
// ✅ Handles request/response mapping
// ✅ Provides health checks
// ✅ Enables service-to-service calls
```

### 3. **Auto-Generated API Gateway**
```rust
// Gateway automatically:
// ✅ Discovers all services via RabbitMQ
// ✅ Maps HTTP → RabbitMQ RPC calls
// ✅ Provides OpenAPI documentation
// ✅ Handles CORS, authentication, middleware
// ✅ Load balances across service instances
```

### 4. **Clean Method Signatures**
```rust
// Simple parameter
async fn get_user(user_id: String) -> Result<UserResponse, String>

// Complex request object
async fn create_order(request: CreateOrderRequest) -> Result<OrderResponse, String>

// Multiple parameters (path + body)
async fn update_profile(params: (String, UpdateProfileRequest)) -> Result<UserResponse, String>
```

### 5. **Service-to-Service Communication**
```rust
// Services talk to each other via RabbitMQ automatically:
// No HTTP clients, no service discovery complexity, no port management!
let client = ServiceClient::new("api-gateway", &rabbitmq_url).await?;
let response = client.call("user-service", "get_user", user_id).await?;
```

---

## 🎯 Ask Me Anything!

I'm ready to help you build amazing microservices with RabbitMesh. Some examples:

- *"Build me a chat system with rooms and real-time messaging"*
- *"Create a file processing pipeline with multiple stages"*
- *"Design a multi-tenant SaaS application architecture"*
- *"Convert my monolith to microservices using RabbitMesh"*
- *"Build a real-time analytics dashboard backend"*
- *"Create a marketplace with buyers, sellers, and transactions"*

**Just describe what you want to build, and I'll show you the cleanest, most powerful way to do it with RabbitMesh!** 🚀

---

## 📦 Published on Crates.io!

RabbitMesh is now **live on crates.io** and ready for production use:

### 🦀 **Available Crates:**
```toml
[dependencies]
rabbitmesh = "0.1.0"           # Core framework
rabbitmesh-macros = "0.1.0"    # Procedural macros  
rabbitmesh-gateway = "0.1.0"   # API Gateway
```

### 🔗 **Resources:**
- **📦 Crates.io**: https://crates.io/crates/rabbitmesh
- **📚 Documentation**: https://docs.rs/rabbitmesh  
- **🐙 GitHub**: https://github.com/hangsiahong/rabbitmesh-rs
- **💬 Issues & Support**: https://github.com/hangsiahong/rabbitmesh-rs/issues

### 🚀 **Quick Start:**
```bash
# Create new project using the agent
python rabbitmesh_agent.py create my-service

# Or manually:
cargo new my-service
cd my-service
# Add rabbitmesh = "0.1.0" to Cargo.toml
# Use #[service_definition] and #[service_method] macros
cargo run
```

---

## 🎪 Ready to See the Magic?

**What would you like to build today?** 

I'll design it using RabbitMesh's full power - minimal code, maximum functionality, zero configuration, pure elegance! ✨

### 🎯 **Try the Agent:**
```bash
# Interactive mode
python rabbitmesh_agent.py

# Create a project
python rabbitmesh_agent.py create my-awesome-service

# Get help
python rabbitmesh_agent.py help
```