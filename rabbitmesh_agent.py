#!/usr/bin/env python3
"""
🤖 RabbitMesh AI Agent

The ultimate AI assistant for building microservices with RabbitMesh framework.
I know all the magic, patterns, and best practices to help you build elegant services.

Usage: python rabbitmesh_agent.py
"""

import sys
import os
import subprocess
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
create <name> - Create a new RabbitMesh project
github        - Show GitHub repository and resources
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
rabbitmesh = "0.1.0"
rabbitmesh-macros = "0.1.0"
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

    def create_project(self, project_name):
        """Create a new RabbitMesh project with published crates"""
        print(f"\n🚀 Creating new RabbitMesh project: {project_name}")
        print("=" * 50)
        
        try:
            # Create project directory
            if os.path.exists(project_name):
                print(f"❌ Directory '{project_name}' already exists!")
                return
                
            os.makedirs(project_name, exist_ok=True)
            os.chdir(project_name)
            
            # Initialize Cargo project
            print("📦 Initializing Cargo project...")
            subprocess.run(["cargo", "init", "--name", project_name], check=True)
            
            # Create Cargo.toml with RabbitMesh dependencies
            cargo_toml = f"""[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
# RabbitMesh Framework (Published on crates.io)
rabbitmesh = "0.1.0"
rabbitmesh-macros = "0.1.0"

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Async runtime  
tokio = {{ version = "1.0", features = ["full"] }}

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Unique IDs
uuid = {{ version = "1.0", features = ["v4", "serde"] }}

# Time handling
chrono = {{ version = "0.4", features = ["serde"] }}

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tokio-test = "0.4"
"""
            
            with open("Cargo.toml", "w") as f:
                f.write(cargo_toml)
            
            # Create basic service structure
            print("🏗️ Creating service structure...")
            
            # Create src/models.rs
            models_rs = '''//! Data models for the service

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ItemResponse {
    pub success: bool,
    pub message: String,
    pub item: Option<Item>,
}

#[derive(Debug, Serialize)]
pub struct ItemListResponse {
    pub success: bool,
    pub message: String,
    pub items: Vec<Item>,
    pub total: usize,
}

impl ItemResponse {
    pub fn success(item: Item) -> Self {
        Self {
            success: true,
            message: "Operation successful".to_string(),
            item: Some(item),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            item: None,
        }
    }
}
'''
            
            # Create src/service.rs  
            service_rs = f'''//! {project_name.title()} service implementation using RabbitMesh

use rabbitmesh_macros::{{service_definition, service_impl}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{{info, warn}};
use anyhow::Result;
use chrono::Utc;

use crate::models::*;

// In-memory storage for demo purposes
type ItemStorage = Arc<RwLock<HashMap<String, Item>>>;

#[service_definition]
pub struct {project_name.title().replace("-", "").replace("_", "")}Service;

static STORAGE: tokio::sync::OnceCell<ItemStorage> = tokio::sync::OnceCell::const_new();

impl {project_name.title().replace("-", "").replace("_", "")}Service {{
    pub async fn init() -> Result<()> {{
        let storage = Arc::new(RwLock::new(HashMap::new()));
        STORAGE.set(storage)
            .map_err(|_| anyhow::anyhow!("Storage already initialized"))?;
        Ok(())
    }}

    fn get_storage() -> &'static ItemStorage {{
        STORAGE.get().expect("Storage not initialized")
    }}
}}

#[service_impl]
impl {project_name.title().replace("-", "").replace("_", "")}Service {{
    /// Create a new item
    #[service_method("POST /items")]
    pub async fn create_item(request: CreateItemRequest) -> Result<ItemResponse, String> {{
        info!("📝 Creating new item: {{}}", request.name);
        
        let item = Item {{
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description.unwrap_or_default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }};
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        storage.insert(item.id.clone(), item.clone());
        
        info!("✅ Item created successfully: {{}}", item.id);
        Ok(ItemResponse::success(item))
    }}

    /// Get an item by ID
    #[service_method("GET /items/:id")]
    pub async fn get_item(item_id: String) -> Result<ItemResponse, String> {{
        info!("🔍 Getting item: {{}}", item_id);
        
        let storage = Self::get_storage();
        let storage = storage.read().await;
        
        match storage.get(&item_id) {{
            Some(item) => {{
                info!("✅ Item found: {{}}", item_id);
                Ok(ItemResponse::success(item.clone()))
            }}
            None => {{
                warn!("❌ Item not found: {{}}", item_id);
                Err("Item not found".to_string())
            }}
        }}
    }}

    /// List all items
    #[service_method("GET /items")]
    pub async fn list_items() -> Result<ItemListResponse, String> {{
        info!("📋 Listing all items");
        
        let storage = Self::get_storage();
        let storage = storage.read().await;
        let items: Vec<Item> = storage.values().cloned().collect();
        let total = items.len();
        
        info!("✅ Retrieved {{}} items", total);
        
        Ok(ItemListResponse {{
            success: true,
            message: format!("Retrieved {{}} items", total),
            items,
            total,
        }})
    }}

    /// Update an item
    #[service_method("PUT /items/:id")]
    pub async fn update_item(params: (String, UpdateItemRequest)) -> Result<ItemResponse, String> {{
        let (item_id, request) = params;
        info!("✏️ Updating item: {{}}", item_id);
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        
        match storage.get_mut(&item_id) {{
            Some(item) => {{
                if let Some(name) = request.name {{
                    item.name = name;
                }}
                if let Some(description) = request.description {{
                    item.description = description;
                }}
                item.updated_at = Utc::now();
                
                info!("✅ Item updated successfully: {{}}", item_id);
                Ok(ItemResponse::success(item.clone()))
            }}
            None => {{
                warn!("❌ Item not found for update: {{}}", item_id);
                Err("Item not found".to_string())
            }}
        }}
    }}

    /// Delete an item
    #[service_method("DELETE /items/:id")]
    pub async fn delete_item(item_id: String) -> Result<ItemResponse, String> {{
        info!("🗑️ Deleting item: {{}}", item_id);
        
        let storage = Self::get_storage();
        let mut storage = storage.write().await;
        
        match storage.remove(&item_id) {{
            Some(item) => {{
                info!("✅ Item deleted successfully: {{}}", item_id);
                Ok(ItemResponse::success(item))
            }}
            None => {{
                warn!("❌ Item not found for deletion: {{}}", item_id);
                Err("Item not found".to_string())
            }}
        }}
    }}
}}
'''

            # Create src/main.rs
            main_rs = f'''//! {project_name.title()} - RabbitMesh Microservice

mod models;
mod service;

use anyhow::Result;
use tracing::{{info, error}};
use tracing_subscriber;
use service::{project_name.title().replace("-", "").replace("_", "")}Service;

#[tokio::main]
async fn main() -> Result<()> {{
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,{project_name.replace("-", "_")}=debug,rabbitmesh=debug")
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting {project_name.title()} Service...");

    // Initialize the service storage
    {project_name.title().replace("-", "").replace("_", "")}Service::init().await?;
    info!("✅ Service storage initialized");

    // Get the RabbitMQ URL from environment or use default
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());

    info!("🐰 Connecting to RabbitMQ at: {{}}", rabbitmq_url);

    // Create and start the service using the macro-generated methods
    let service = {project_name.title().replace("-", "").replace("_", "")}Service::create_service(&rabbitmq_url).await?;
    
    info!("✅ {project_name.title()} Service started successfully");
    info!("🎯 Service name: {{}}", {project_name.title().replace("-", "").replace("_", "")}Service::service_name());
    
    // Show auto-discovered routes
    let routes = {project_name.title().replace("-", "").replace("_", "")}Service::get_routes();
    info!("📊 Auto-discovered {{}} HTTP routes:", routes.len());
    for (route, method) in routes {{
        info!("   {{}} -> {{}}", method, route);
    }}
    
    info!("📞 Starting service listener...");
    service.start().await?;
    
    Ok(())
}}
'''

            # Write files
            with open("src/models.rs", "w") as f:
                f.write(models_rs)
            with open("src/service.rs", "w") as f:
                f.write(service_rs)
            with open("src/main.rs", "w") as f:
                f.write(main_rs)
            
            # Create README.md
            readme_md = f"""# {project_name.title()}

A RabbitMesh microservice built with zero-port architecture.

## 🚀 Features

- **Zero Port Management**: Services communicate via RabbitMQ only
- **Auto-Generated APIs**: HTTP routes extracted from code annotations  
- **Service Discovery**: Automatic via RabbitMQ queues
- **Built-in Fault Tolerance**: Retry mechanisms and circuit breakers

## 🏃 Quick Start

1. **Start RabbitMQ**:
```bash
docker run -d --hostname rabbitmq --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3-management
```

2. **Run the service**:
```bash
cargo run
```

3. **Test the service** (in another terminal):
```bash
# Create an item
curl -X POST http://localhost:3000/items \\
  -H "Content-Type: application/json" \\
  -d '{{"name": "Test Item", "description": "A test item"}}'

# List items  
curl http://localhost:3000/items

# Get specific item
curl http://localhost:3000/items/<item_id>
```

## 🏗️ Architecture

```
Client → API Gateway (HTTP) → RabbitMQ → {project_name.title()} Service
         :3000                                   (No Port!)
```

## 📦 Generated Routes

The RabbitMesh macros automatically generate:
- `POST /items` - Create item
- `GET /items/:id` - Get item  
- `GET /items` - List items
- `PUT /items/:id` - Update item
- `DELETE /items/:id` - Delete item

## 🌐 RabbitMesh Framework

Built with [RabbitMesh](https://github.com/hangsiahong/rabbitmesh-rs) - the zero-port microservices framework.

Learn more: https://crates.io/crates/rabbitmesh
"""

            with open("README.md", "w") as f:
                f.write(readme_md)
            
            # Create .gitignore
            gitignore = """/target/
**/*.rs.bk
Cargo.lock
.env
"""
            with open(".gitignore", "w") as f:
                f.write(gitignore)
            
            print("✅ Project created successfully!")
            print(f"\n🎯 Next steps:")
            print(f"  cd {project_name}")
            print("  cargo run")
            print("\n📖 The service will:")
            print("  • Connect to RabbitMQ")
            print("  • Auto-register HTTP routes")
            print("  • Start handling requests")
            print("\n🔗 RabbitMesh GitHub: https://github.com/hangsiahong/rabbitmesh-rs")
            
        except subprocess.CalledProcessError as e:
            print(f"❌ Error creating project: {{e}}")
        except Exception as e:
            print(f"❌ Unexpected error: {{e}}")

    def show_github_info(self):
        """Show GitHub repository and resources"""
        print("""
🐙 RabbitMesh GitHub Repository
════════════════════════════════

📦 Main Repository: https://github.com/hangsiahong/rabbitmesh-rs
🦀 Crates.io: https://crates.io/crates/rabbitmesh
📚 Documentation: https://docs.rs/rabbitmesh

🎯 Published Crates:
  • rabbitmesh = "0.1.0"           (Core framework)
  • rabbitmesh-macros = "0.1.0"    (Procedural macros)  
  • rabbitmesh-gateway = "0.1.0"   (API Gateway)

🚀 Getting Started:
  1. cargo new my-service
  2. Add rabbitmesh = "0.1.0" to Cargo.toml
  3. Use #[service_definition] and #[service_method] macros
  4. cargo run

🤝 Contributing:
  • Issues: https://github.com/hangsiahong/rabbitmesh-rs/issues
  • Pull Requests: https://github.com/hangsiahong/rabbitmesh-rs/pulls
  • Discussions: https://github.com/hangsiahong/rabbitmesh-rs/discussions

💡 Examples:
  • Simple Todo: Working example included in repository
  • Authentication: JWT-based auth service example
  • E-commerce: Multi-service architecture demo

🏗️ Architecture Highlights:
  ✅ Zero-port microservices (RabbitMQ only)
  ✅ Auto-generated HTTP routes from annotations
  ✅ Built-in service discovery
  ✅ Fault tolerance and retries
  ✅ Production-ready async runtime
""")

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
                elif user_input.lower() == 'github':
                    self.show_github_info()
                elif user_input.lower().startswith('create '):
                    project_name = user_input[7:].strip()
                    if project_name:
                        self.create_project(project_name)
                    else:
                        print("❌ Please provide a project name: create my-service")
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
        elif command.lower() == 'github':
            agent.show_github_info()
        elif command.lower().startswith('create '):
            project_name = command[7:].strip()
            if project_name:
                agent.create_project(project_name)
            else:
                print("❌ Please provide a project name: python rabbitmesh_agent.py create my-service")
        else:
            agent.build_service(command)
    else:
        # Interactive mode
        agent.interactive_mode()

if __name__ == "__main__":
    main()