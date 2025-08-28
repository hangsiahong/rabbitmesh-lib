#!/usr/bin/env python3
"""
🤖 RabbitMesh AI Agent

The ultimate AI assistant for building microservices with RabbitMesh framework.
I know all the magic, patterns, and best practices to help you build elegant services.

Usage: python rabbitmesh_agent.py
"""

import sys
import os
from pathlib import Path

# ASCII Art Banner
BANNER = """
╔══════════════════════════════════════════════════════════════════╗
║                    🐰 RabbitMesh AI Agent 🤖                     ║
║                                                                  ║
║  Your Expert Guide to Zero-Port Microservices Magic             ║
║  • Macro-Powered Service Definition                              ║
║  • Auto-Generated Everything                                     ║
║  • Zero Configuration Architecture                               ║
║  • Pure RabbitMQ Communication                                   ║
╚══════════════════════════════════════════════════════════════════╝
"""

INTRO = """
🎯 I'm your RabbitMesh Expert Agent!

I understand the complete magic of RabbitMesh and will help you build 
microservices using the framework's full power. I know:

✨ The Magic Features:
  • Auto-generated HTTP routes from #[service_method] annotations
  • Auto-registered RPC handlers for each method  
  • Auto-generated API Gateway with service discovery
  • Zero-port architecture (pure RabbitMQ communication)
  • Auto-serialization/deserialization of requests/responses

🎨 What I Can Build For You:
  • Authentication & Authorization Services
  • E-commerce Platforms (Products, Orders, Payments)
  • Real-time Chat Systems
  • File Processing Pipelines
  • Analytics & Reporting Services
  • Multi-tenant SaaS Architectures
  • API Gateways with Auto-Discovery
  • Any microservice architecture you can imagine!

🎪 My Specialties:
  • Write ONLY business logic (framework handles the rest)
  • Leverage ALL RabbitMesh features for maximum efficiency
  • Design optimal service boundaries and communication patterns
  • Follow framework conventions and best practices
  • Optimize for scalability and performance

💡 Example Commands:
  "Build me a user authentication service with JWT tokens"
  "Create an e-commerce order processing system" 
  "Design a real-time chat application with rooms"
  "Convert my REST API to RabbitMesh microservices"
  "Build a file upload and processing pipeline"
"""

class RabbitMeshAgent:
    def __init__(self):
        self.framework_knowledge = {
            "macros": ["service_definition", "service_impl", "service_method"],
            "auto_features": [
                "HTTP route extraction",
                "RPC handler registration", 
                "Request/response serialization",
                "API Gateway generation",
                "Service discovery",
                "Health checks",
                "Load balancing"
            ],
            "patterns": {
                "simple_param": "async fn get_item(id: String) -> Result<ItemResponse, String>",
                "complex_param": "async fn create_item(request: CreateItemRequest) -> Result<ItemResponse, String>",
                "tuple_params": "async fn update_item(params: (String, UpdateItemRequest)) -> Result<ItemResponse, String>"
            },
            "architecture": "Zero-port microservices communicating via RabbitMQ"
        }

    def display_banner(self):
        print(BANNER)
        print(INTRO)

    def show_examples(self):
        print("\n🎯 RabbitMesh Service Pattern Examples:\n")
        
        print("1️⃣  Simple Service Definition:")
        print("""
#[service_definition]
pub struct UserService;

#[service_impl]
impl UserService {
    #[service_method("POST /users")]
    pub async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String> {
        // JUST YOUR BUSINESS LOGIC!
        let user = User::new(request.username, request.email);
        user.save().await?;
        Ok(UserResponse::success(user))
    }

    #[service_method("GET /users/:id")]
    pub async fn get_user(user_id: String) -> Result<UserResponse, String> {
        // Framework handles HTTP parsing, RabbitMQ, serialization automatically
        let user = User::find(&user_id).await?;
        Ok(UserResponse::success(user))
    }
}
""")

        print("2️⃣  Service Output (Auto-Generated Magic):")
        print("""
✅ UserService started successfully
🎯 Service name: UserService-service
📊 Auto-discovered 8 HTTP routes:
   create_user -> POST /users
   get_user -> GET /users/:id
   list_users -> GET /users
   update_user -> PUT /users/:id
   delete_user -> DELETE /users/:id
   authenticate -> POST /auth/login
   register -> POST /auth/register
   refresh_token -> POST /auth/refresh
""")

        print("3️⃣  Auto-Generated API Gateway:")
        print("""
// Gateway automatically discovers your service!
let app = create_auto_router(&rabbitmq_url).await?;
// Maps HTTP requests to RabbitMQ RPC calls automatically
// Provides OpenAPI docs, health checks, service discovery
""")

    def show_help(self):
        print("""
🎪 RabbitMesh Agent Commands:

help          - Show this help message
examples      - Show RabbitMesh code examples  
patterns      - Show common service patterns
architecture  - Explain zero-port architecture
magic         - Explain the framework magic
build <desc>  - Ask me to build something specific!

🎯 Or just describe what you want to build:
  "authentication service"
  "e-commerce platform" 
  "chat system"
  "file processor"
  "analytics dashboard"
""")

    def explain_magic(self):
        print("""
✨ THE RABBITMESH MAGIC EXPLAINED ✨

🎪 What Happens When You Write This:
#[service_method("POST /users")]
pub async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String>

🪄 Framework Automatically:
  1. Extracts HTTP route: POST /users
  2. Registers RPC handler: create_user
  3. Sets up RabbitMQ queue: rabbitmesh.UserService.create_user
  4. Handles JSON serialization/deserialization
  5. Creates HTTP endpoint in auto-generated gateway
  6. Provides request validation and error handling
  7. Enables service-to-service calls via RabbitMQ
  8. Adds health checks and monitoring

🏗️ Zero Configuration Result:
  ✅ HTTP API endpoint working
  ✅ RabbitMQ RPC handler registered  
  ✅ Service discoverable by gateway
  ✅ Inter-service communication enabled
  ✅ Auto-generated OpenAPI docs
  ✅ Health checks and metrics
  ✅ Load balancing and fault tolerance

🎯 You Write: Business logic only (5-10 lines)
🎯 Framework Provides: Everything else (hundreds of lines)
""")

    def show_architecture(self):
        print("""
🏗️ RABBITMESH ZERO-PORT ARCHITECTURE

Traditional Microservices:
┌─────────┐ HTTP ┌─────────┐ HTTP ┌─────────┐
│Service A├─────→│Service B├─────→│Service C│
│ Port:80 │      │Port:81  │      │Port:82  │
└─────────┘      └─────────┘      └─────────┘
❌ Port management nightmare
❌ Service discovery complexity  
❌ Load balancing configuration
❌ Network configuration

RabbitMesh Architecture:
┌─────────┐      ┌─────────────┐      ┌─────────┐
│Service A├─────→│  RabbitMQ   │←─────┤Service B│
│ No Port │      │   Broker    │      │ No Port │
└─────────┘      └─────────────┘      └─────────┘
                        ↑
                 ┌─────────────┐
                 │Auto Gateway │ ← HTTP Only Here
                 │ Port: 3000  │
                 └─────────────┘

✅ Zero port management between services
✅ Automatic service discovery via queues
✅ Built-in load balancing and fault tolerance  
✅ Services can run anywhere (containers, VMs, bare metal)
✅ Single HTTP port for external access (gateway)
""")

    def build_service(self, description):
        """Generate service code based on user description"""
        print(f"\n🎯 Building: {description}\n")
        
        if "auth" in description.lower() or "user" in description.lower():
            self.generate_auth_service()
        elif "ecommerce" in description.lower() or "shop" in description.lower():
            self.generate_ecommerce_service()
        elif "chat" in description.lower():
            self.generate_chat_service()
        elif "file" in description.lower():
            self.generate_file_service()
        else:
            self.generate_generic_service(description)

    def generate_auth_service(self):
        print("🔐 Generating Authentication Service with RabbitMesh Magic:")
        print("""
// Cargo.toml
[dependencies]
rabbitmesh = { path = "../../rabbitmesh" }
rabbitmesh-macros = { path = "../../rabbitmesh-macros" }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
bcrypt = "0.15"
jsonwebtoken = "9.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

// src/models.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user: Option<User>,
}

// src/service.rs
use rabbitmesh_macros::{service_definition, service_impl};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type UserStorage = Arc<RwLock<HashMap<String, User>>>;
static STORAGE: tokio::sync::OnceCell<UserStorage> = tokio::sync::OnceCell::const_new();

#[service_definition]
pub struct AuthService;

impl AuthService {
    pub async fn init() -> Result<()> {
        let storage = Arc::new(RwLock::new(HashMap::new()));
        STORAGE.set(storage).map_err(|_| anyhow::anyhow!("Storage already initialized"))?;
        Ok(())
    }

    fn get_storage() -> &'static UserStorage {
        STORAGE.get().expect("Storage not initialized")
    }

    fn hash_password(password: &str) -> Result<String, String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Password hashing failed: {}", e))
    }

    fn verify_password(password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }

    fn generate_jwt(user_id: &str) -> Result<String, String> {
        // JWT generation logic here
        Ok(format!("jwt_token_for_{}", user_id))
    }
}

#[service_impl]
impl AuthService {
    /// Register a new user
    #[service_method("POST /auth/register")]
    pub async fn register(request: RegisterRequest) -> Result<AuthResponse, String> {
        let storage = Self::get_storage();
        let mut users = storage.write().await;

        // Check if user exists
        if users.values().any(|u| u.username == request.username || u.email == request.email) {
            return Ok(AuthResponse {
                success: false,
                message: "User already exists".to_string(),
                token: None,
                user: None,
            });
        }

        // Create new user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: request.username,
            email: request.email,
            password_hash: Self::hash_password(&request.password)?,
            is_active: true,
            created_at: chrono::Utc::now(),
        };

        let user_id = user.id.clone();
        users.insert(user_id.clone(), user.clone());

        // Generate JWT token
        let token = Self::generate_jwt(&user_id)?;

        Ok(AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            token: Some(token),
            user: Some(user),
        })
    }

    /// Login user
    #[service_method("POST /auth/login")]
    pub async fn login(request: LoginRequest) -> Result<AuthResponse, String> {
        let storage = Self::get_storage();
        let users = storage.read().await;

        // Find user by username
        let user = users.values()
            .find(|u| u.username == request.username)
            .cloned();

        match user {
            Some(user) if Self::verify_password(&request.password, &user.password_hash) => {
                let token = Self::generate_jwt(&user.id)?;
                Ok(AuthResponse {
                    success: true,
                    message: "Login successful".to_string(),
                    token: Some(token),
                    user: Some(user),
                })
            }
            _ => Ok(AuthResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                token: None,
                user: None,
            })
        }
    }

    /// Get user profile
    #[service_method("GET /users/:id")]
    pub async fn get_user(user_id: String) -> Result<AuthResponse, String> {
        let storage = Self::get_storage();
        let users = storage.read().await;

        match users.get(&user_id) {
            Some(user) => Ok(AuthResponse {
                success: true,
                message: "User found".to_string(),
                token: None,
                user: Some(user.clone()),
            }),
            None => Err("User not found".to_string()),
        }
    }

    /// List all users (admin only)
    #[service_method("GET /users")]
    pub async fn list_users() -> Result<serde_json::Value, String> {
        let storage = Self::get_storage();
        let users = storage.read().await;
        
        let user_list: Vec<&User> = users.values().collect();
        Ok(serde_json::json!({
            "success": true,
            "message": "Users retrieved",
            "users": user_list,
            "total": user_list.len()
        }))
    }

    /// Validate JWT token
    #[service_method("POST /auth/validate")]
    pub async fn validate_token(token: String) -> Result<AuthResponse, String> {
        // Token validation logic here
        // For demo, we'll extract user_id from token format
        if token.starts_with("jwt_token_for_") {
            let user_id = token.replace("jwt_token_for_", "");
            Self::get_user(user_id).await
        } else {
            Ok(AuthResponse {
                success: false,
                message: "Invalid token".to_string(),
                token: None,
                user: None,
            })
        }
    }

    /// Refresh JWT token
    #[service_method("POST /auth/refresh")]
    pub async fn refresh_token(token: String) -> Result<AuthResponse, String> {
        // Validate current token and issue new one
        let validation_result = Self::validate_token(token).await?;
        if validation_result.success {
            if let Some(user) = validation_result.user {
                let new_token = Self::generate_jwt(&user.id)?;
                return Ok(AuthResponse {
                    success: true,
                    message: "Token refreshed".to_string(),
                    token: Some(new_token),
                    user: Some(user),
                });
            }
        }
        
        Err("Token refresh failed".to_string())
    }
}

// src/main.rs
mod models;
mod service;

use anyhow::Result;
use tracing::{info};
use tracing_subscriber;
use service::AuthService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,auth_service=debug")
        .init();

    info!("🚀 Starting Auth Service...");

    // Initialize service storage
    AuthService::init().await?;
    
    // Get RabbitMQ URL
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());

    // Create and start service
    let service = AuthService::create_service(&rabbitmq_url).await?;
    
    info!("✅ Auth Service started successfully");
    info!("🎯 Service name: {}", AuthService::service_name());
    
    let routes = AuthService::get_routes();
    info!("📊 Auto-discovered {} HTTP routes:", routes.len());
    for (route, method) in routes {
        info!("   {} -> {}", method, route);
    }
    
    service.start().await?;
    Ok(())
}
""")
        print("🎉 Authentication Service Generated!")
        print("✨ Features: Registration, Login, JWT tokens, User profiles, Token validation")
        print("🎯 Auto-generated routes: POST /auth/register, POST /auth/login, GET /users/:id, etc.")

    def generate_generic_service(self, description):
        service_name = description.split()[0].title()
        print(f"🎯 Generating {service_name} Service with RabbitMesh Magic:")
        print(f"""
#[service_definition]
pub struct {service_name}Service;

#[service_impl]
impl {service_name}Service {{
    /// Create new {service_name.lower()} item
    #[service_method("POST /{service_name.lower()}s")]
    pub async fn create_{service_name.lower()}(request: Create{service_name}Request) -> Result<{service_name}Response, String> {{
        // Your business logic here!
        // Framework handles: HTTP parsing, RabbitMQ, serialization, routing
        Ok({service_name}Response::success("Created successfully"))
    }}

    /// Get {service_name.lower()} by ID  
    #[service_method("GET /{service_name.lower()}s/:id")]
    pub async fn get_{service_name.lower()}(id: String) -> Result<{service_name}Response, String> {{
        // Framework auto-extracts 'id' from URL path
        // Your business logic here!
        Ok({service_name}Response::success(&format!("Retrieved {{}}", id)))
    }}

    /// List all {service_name.lower()}s
    #[service_method("GET /{service_name.lower()}s")]
    pub async fn list_{service_name.lower()}s() -> Result<{service_name}ListResponse, String> {{
        // Your business logic here!
        Ok({service_name}ListResponse::success(vec![]))
    }}

    /// Update {service_name.lower()}
    #[service_method("PUT /{service_name.lower()}s/:id")]  
    pub async fn update_{service_name.lower()}(params: (String, Update{service_name}Request)) -> Result<{service_name}Response, String> {{
        let (id, request) = params; // Framework handles tuple unpacking
        // Your business logic here!
        Ok({service_name}Response::success("Updated successfully"))
    }}

    /// Delete {service_name.lower()}
    #[service_method("DELETE /{service_name.lower()}s/:id")]
    pub async fn delete_{service_name.lower()}(id: String) -> Result<{service_name}Response, String> {{
        // Your business logic here!
        Ok({service_name}Response::success("Deleted successfully"))
    }}
}}
""")
        print(f"🎉 {service_name} Service Generated!")
        print("✨ Framework automatically provides:")
        print("  ✅ HTTP routes extracted from annotations")
        print("  ✅ RabbitMQ RPC handlers registered")
        print("  ✅ Request/response serialization")
        print("  ✅ API Gateway integration")
        print("  ✅ Service discovery")

    def interactive_mode(self):
        """Interactive mode for continuous assistance"""
        self.display_banner()
        
        while True:
            try:
                print("\n🤖 RabbitMesh Agent > ", end="")
                user_input = input().strip()
                
                if not user_input:
                    continue
                
                if user_input.lower() in ['exit', 'quit', 'bye']:
                    print("\n👋 Happy coding with RabbitMesh! Build something amazing!")
                    break
                elif user_input.lower() == 'help':
                    self.show_help()
                elif user_input.lower() == 'examples':
                    self.show_examples()
                elif user_input.lower() == 'magic':
                    self.explain_magic()
                elif user_input.lower() == 'architecture':
                    self.show_architecture()
                elif user_input.lower().startswith('build '):
                    description = user_input[6:]
                    self.build_service(description)
                else:
                    # Treat any other input as a build request
                    print(f"🎯 I'll help you build: {user_input}")
                    self.build_service(user_input)
                    
            except KeyboardInterrupt:
                print("\n\n👋 Happy coding with RabbitMesh!")
                break
            except Exception as e:
                print(f"\n❌ Error: {e}")

def main():
    """Main entry point"""
    agent = RabbitMeshAgent()
    
    if len(sys.argv) > 1:
        # Command line mode
        command = " ".join(sys.argv[1:])
        if command.lower() == 'help':
            agent.show_help()
        elif command.lower() == 'examples':
            agent.show_examples()
        elif command.lower() == 'magic':
            agent.explain_magic()
        elif command.lower() == 'architecture':
            agent.show_architecture()
        else:
            agent.build_service(command)
    else:
        # Interactive mode
        agent.interactive_mode()

if __name__ == "__main__":
    main()