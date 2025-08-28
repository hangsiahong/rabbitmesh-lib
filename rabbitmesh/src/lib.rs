//! # RabbitMesh - Zero-Port Microservices Framework
//!
//! **Build elegant microservices with simple macros - write only business logic!**
//!
//! RabbitMesh revolutionizes microservices by eliminating port management through pure RabbitMQ 
//! communication. Services are defined with clean Rust macros, and everything else is auto-generated.
//!
//! ## ✨ The Magic
//!
//! ```rust,no_run
//! use rabbitmesh_macros::{service_definition, service_impl};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! pub struct CreateUserRequest {
//!     pub username: String,
//!     pub email: String,
//! }
//!
//! #[derive(Serialize)]
//! pub struct UserResponse {
//!     pub success: bool,
//!     pub message: String,
//! }
//!
//! // Define your service with macros
//! #[service_definition]
//! pub struct UserService;
//!
//! // Implement business logic only - framework handles everything else!
//! #[service_impl]
//! impl UserService {
//!     #[service_method("POST /users")]
//!     pub async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String> {
//!         // JUST YOUR BUSINESS LOGIC!
//!         // Framework automatically handles:
//!         // ✅ HTTP route extraction (POST /users)  
//!         // ✅ RabbitMQ RPC registration
//!         // ✅ JSON serialization/deserialization
//!         // ✅ API Gateway integration
//!         // ✅ Service discovery
//!         
//!         println!("Creating user: {}", request.username);
//!         Ok(UserResponse {
//!             success: true,
//!             message: "User created successfully!".to_string(),
//!         })
//!     }
//!
//!     #[service_method("GET /users/:id")]
//!     pub async fn get_user(user_id: String) -> Result<UserResponse, String> {
//!         // Framework auto-extracts 'user_id' from URL path
//!         println!("Getting user: {}", user_id);
//!         Ok(UserResponse {
//!             success: true,
//!             message: format!("Retrieved user {}", user_id),
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize service (for stateful services)
//!     // UserService::init().await?;
//!     
//!     // Create and start service - that's it!
//!     let service = UserService::create_service("amqp://guest:guest@localhost:5672/%2f").await?;
//!     
//!     println!("🚀 UserService started!");
//!     println!("🎯 Service name: {}", UserService::service_name());
//!     
//!     // Show the magic - auto-discovered routes
//!     let routes = UserService::get_routes();
//!     println!("✨ Auto-discovered {} routes:", routes.len());
//!     for (route, method) in routes {
//!         println!("   {} -> {}", method, route);
//!     }
//!     
//!     service.start().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## 🪄 What RabbitMesh Does Automatically
//!
//! When you write `#[service_method("POST /users")]`, the framework automatically:
//!
//! - ✅ **Extracts HTTP route** from annotation (`POST /users`)
//! - ✅ **Registers RPC handler** for the method (`create_user`) 
//! - ✅ **Sets up RabbitMQ queues** (`rabbitmesh.UserService.create_user`)
//! - ✅ **Handles serialization** (JSON request/response)
//! - ✅ **Creates gateway endpoint** (HTTP API automatically available)
//! - ✅ **Enables service discovery** (other services can call this method)
//! - ✅ **Provides health checks** and monitoring
//! - ✅ **Implements fault tolerance** (retries, circuit breakers)
//!
//! **You write**: 5-10 lines of business logic  
//! **Framework provides**: Everything else (hundreds of lines worth!)
//!
//! ## 🏗️ Zero-Port Architecture
//!
//! **Traditional Microservices:**
//! ```text
//! Service A:8080 ←HTTP→ Service B:8081 ←HTTP→ Service C:8082
//! ❌ Port management hell
//! ❌ Service discovery complexity  
//! ❌ Load balancing configuration
//! ```
//!
//! **RabbitMesh:**
//! ```text
//! Service A ←RabbitMQ→ Service B ←RabbitMQ→ Service C
//!          ↘           ↓           ↙
//!            Auto-Generated Gateway:3000 ←HTTP← Clients
//! ✅ Zero port management between services
//! ✅ Automatic service discovery
//! ✅ Built-in load balancing and fault tolerance
//! ```
//!
//! ## 🎨 Method Signature Patterns
//!
//! RabbitMesh supports clean, intuitive method signatures:
//!
//! ```rust,no_run
//! # use rabbitmesh_macros::{service_definition, service_impl};
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Deserialize)] pub struct CreateUserRequest { pub name: String }
//! # #[derive(Deserialize)] pub struct UpdateUserRequest { pub name: String }
//! # #[derive(Serialize)] pub struct UserResponse { pub success: bool }
//! # #[service_definition] pub struct UserService;
//! #[service_impl]
//! impl UserService {
//!     // Simple parameter
//!     #[service_method("GET /users/:id")]
//!     async fn get_user(user_id: String) -> Result<UserResponse, String> {
//!         // user_id auto-extracted from URL path
//!         Ok(UserResponse { success: true })
//!     }
//!
//!     // Request body
//!     #[service_method("POST /users")]
//!     async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String> {
//!         // request auto-deserialized from JSON
//!         Ok(UserResponse { success: true })
//!     }
//!
//!     // Path parameter + request body (tuple)
//!     #[service_method("PUT /users/:id")]
//!     async fn update_user(params: (String, UpdateUserRequest)) -> Result<UserResponse, String> {
//!         let (user_id, request) = params;
//!         Ok(UserResponse { success: true })
//!     }
//!
//!     // Multiple path parameters
//!     #[service_method("GET /users/:user_id/posts/:post_id")]
//!     async fn get_user_post(params: (String, String)) -> Result<UserResponse, String> {
//!         let (user_id, post_id) = params;
//!         Ok(UserResponse { success: true })
//!     }
//! }
//! ```
//!
//! ## 🚀 Service-to-Service Communication
//!
//! Services communicate via RabbitMQ automatically:
//!
//! ```rust,no_run
//! use rabbitmesh::ServiceClient;
//!
//! // From API Gateway or another service
//! let client = ServiceClient::new("api-gateway", "amqp://localhost:5672").await?;
//! let response = client.call("user-service", "get_user", "user123").await?;
//! ```
//!
//! ## 🎯 Key Features
//!
//! - **🪄 Zero Configuration**: Routes extracted from code annotations
//! - **🚀 Auto-Generated Gateway**: HTTP API created from service definitions
//! - **🔌 Zero Ports**: Services communicate only via RabbitMQ  
//! - **🔍 Auto-Discovery**: Services find each other automatically
//! - **⚡ High Performance**: Built on Tokio async runtime
//! - **🛡️ Fault Tolerance**: Built-in retry mechanisms and circuit breakers
//! - **📊 Observability**: Structured logging and health checks
//! - **🎨 Clean Code**: Write only business logic, framework handles the rest
//!
//! ## 📚 Examples
//!
//! See the `examples/` directory for complete working examples:
//! - **Simple Todo Service**: Basic CRUD operations with auto-generated routes
//! - **Authentication Service**: JWT tokens, user management, password hashing
//! - **Blog Platform**: Multi-service architecture with auto-discovery
//!
//! ## 🤖 AI Assistant
//!
//! Get help building services with our AI agent:
//! ```bash
//! python rabbitmesh_agent.py "build me an authentication service"
//! ```
//!
//! ## 🧪 Requirements
//!
//! - **Rust**: 1.70+
//! - **RabbitMQ**: 3.8+ 
//! - **Dependencies**: `rabbitmesh-macros`, `serde`, `tokio`
//!
//! ---
//!
//! **Ready to build the future of microservices?** 🚀
//!
//! *Zero ports, maximum power, pure elegance.*

pub mod connection;
pub mod message;
pub mod rpc;
pub mod service;
pub mod client;
pub mod error;

pub use connection::{ConnectionManager, ConnectionConfig};
pub use message::{Message, MessageType, RpcRequest, RpcResponse};
pub use rpc::{RpcHandler, RpcFramework};
pub use service::{MicroService, ServiceConfig, ServiceClient as ServiceClientFromService};

// Convenient alias for the main service
pub use service::MicroService as Service;
pub use client::ServiceClient;
pub use error::{RabbitMeshError, Result};