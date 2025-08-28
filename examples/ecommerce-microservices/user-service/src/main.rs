//! # User Service
//! 
//! Handles user registration, authentication, and user management.
//! This service exposes NO HTTP ports - it only connects to RabbitMQ.

use rabbitmesh::{MicroService, RpcResponse, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, debug};

/// User data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub name: String,
    pub created_at: String,
}

/// Request to create a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
}

/// Request to update user
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// In-memory user database (in production, use a real database)
type UserDatabase = Arc<RwLock<HashMap<u32, User>>>;

async fn setup_sample_data(db: &UserDatabase) {
    let mut users = db.write().await;
    users.insert(1, User {
        id: 1,
        email: "john@example.com".to_string(),
        name: "John Doe".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    users.insert(2, User {
        id: 2,
        email: "jane@example.com".to_string(),
        name: "Jane Smith".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    info!("✅ Sample users created");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting User Service");
    info!("🔥 Zero ports exposed - only RabbitMQ connection!");

    // Create in-memory database
    let db: UserDatabase = Arc::new(RwLock::new(HashMap::new()));
    setup_sample_data(&db).await;

    // Create microservice - NO PORTS EXPOSED!
    let service = MicroService::new_simple("user-service", "amqp://localhost:5672").await?;

    // Register get_user handler
    let db_clone = db.clone();
    service.register_function("get_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            debug!("Getting user");
            let user_id: u32 = msg.deserialize_payload()?;
            
            let users = db.read().await;
            match users.get(&user_id) {
                Some(user) => {
                    info!("Found user: {}", user.email);
                    Ok(RpcResponse::success(user.clone(), 10)?)
                }
                None => {
                    info!("User not found: {}", user_id);
                    Ok(RpcResponse::error(format!("User with ID {} not found", user_id)))
                }
            }
        }
    }).await;

    // Register create_user handler
    let db_clone = db.clone();
    service.register_function("create_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            info!("Creating user");
            let request: CreateUserRequest = msg.deserialize_payload()?;
            
            let mut users = db.write().await;
            
            // Check if email already exists
            for user in users.values() {
                if user.email == request.email {
                    return Ok(RpcResponse::error("Email already exists"));
                }
            }
            
            // Generate new ID
            let id = users.len() as u32 + 1;
            
            let user = User {
                id,
                email: request.email,
                name: request.name,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            
            users.insert(id, user.clone());
            info!("Created user: {} (ID: {})", user.email, user.id);
            
            Ok(RpcResponse::success(user, 20)?)
        }
    }).await;

    // Register list_users handler
    let db_clone = db.clone();
    service.register_function("list_users", move |_msg: Message| {
        let db = db_clone.clone();
        async move {
            debug!("Listing all users");
            
            let users = db.read().await;
            let user_list: Vec<User> = users.values().cloned().collect();
            
            info!("Listed {} users", user_list.len());
            Ok(RpcResponse::success(user_list, 5)?)
        }
    }).await;

    // Register ping handler for health checks
    service.register_function("ping", |_msg: Message| async move {
        Ok(RpcResponse::success("User service is healthy", 1)?)
    }).await;

    info!("✅ User service handlers registered");
    info!("🎯 Listening to RabbitMQ queue: rabbitmesh.user-service");
    info!("🔗 Available via API Gateway at:");
    info!("   GET  /api/v1/user-service/list_users");
    info!("   GET  /api/v1/user-service/get_user?user_id=1");
    info!("   POST /api/v1/user-service/create_user");
    
    // Start the service - runs forever, processing RabbitMQ messages
    service.start().await?;
    Ok(())
}