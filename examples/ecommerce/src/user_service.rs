use rabbitmesh::{MicroService, RpcResponse, Message};
use rabbitmesh_macros::{service_definition, service_method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, debug};
use uuid::Uuid;

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

/// User microservice definition
#[service_definition]
pub struct UserService;

impl UserService {
    /// Get user by ID
    #[service_method("GET /users/:id")]
    pub async fn get_user(user_id: u32, db: UserDatabase) -> Result<User, String> {
        debug!("Getting user with ID: {}", user_id);
        
        let users = db.read().await;
        match users.get(&user_id) {
            Some(user) => {
                info!("Found user: {}", user.email);
                Ok(user.clone())
            }
            None => {
                info!("User not found: {}", user_id);
                Err(format!("User with ID {} not found", user_id))
            }
        }
    }

    /// Create a new user
    #[service_method("POST /users")]
    pub async fn create_user(data: CreateUserRequest, db: UserDatabase) -> Result<User, String> {
        info!("Creating user: {}", data.email);
        
        let mut users = db.write().await;
        
        // Check if email already exists
        for user in users.values() {
            if user.email == data.email {
                return Err("Email already exists".to_string());
            }
        }
        
        // Generate new ID
        let id = users.len() as u32 + 1;
        
        let user = User {
            id,
            email: data.email,
            name: data.name,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        users.insert(id, user.clone());
        info!("Created user: {} (ID: {})", user.email, user.id);
        
        Ok(user)
    }

    /// Update user
    #[service_method("PUT /users/:id")]
    pub async fn update_user(user_id: u32, data: UpdateUserRequest, db: UserDatabase) -> Result<User, String> {
        info!("Updating user: {}", user_id);
        
        let mut users = db.write().await;
        
        match users.get_mut(&user_id) {
            Some(user) => {
                if let Some(name) = data.name {
                    user.name = name;
                }
                if let Some(email) = data.email {
                    user.email = email;
                }
                
                info!("Updated user: {}", user.email);
                Ok(user.clone())
            }
            None => Err(format!("User with ID {} not found", user_id))
        }
    }

    /// Delete user
    #[service_method("DELETE /users/:id")]
    pub async fn delete_user(user_id: u32, db: UserDatabase) -> Result<String, String> {
        info!("Deleting user: {}", user_id);
        
        let mut users = db.write().await;
        
        match users.remove(&user_id) {
            Some(user) => {
                info!("Deleted user: {}", user.email);
                Ok(format!("User {} deleted successfully", user.email))
            }
            None => Err(format!("User with ID {} not found", user_id))
        }
    }

    /// List all users
    #[service_method("GET /users")]
    pub async fn list_users(db: UserDatabase) -> Result<Vec<User>, String> {
        debug!("Listing all users");
        
        let users = db.read().await;
        let user_list: Vec<User> = users.values().cloned().collect();
        
        info!("Listed {} users", user_list.len());
        Ok(user_list)
    }

    /// Health check endpoint
    #[service_method]
    pub async fn ping() -> Result<String, String> {
        Ok("User service is healthy".to_string())
    }
}

/// Run the user service
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting User Service");

    // Create in-memory database
    let db: UserDatabase = Arc::new(RwLock::new(HashMap::new()));
    
    // Add some sample data
    {
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
    }

    // Create microservice
    let service = UserService::create_service("amqp://localhost:5672").await?;
    
    // Register handlers
    let db_clone = db.clone();
    service.register_function("get_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let user_id: u32 = msg.deserialize_payload()?;
            match UserService::get_user(user_id, db).await {
                Ok(user) => Ok(RpcResponse::success(user, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("create_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let request: CreateUserRequest = msg.deserialize_payload()?;
            match UserService::create_user(request, db).await {
                Ok(user) => Ok(RpcResponse::success(user, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("update_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            #[derive(Deserialize)]
            struct UpdateParams {
                user_id: u32,
                #[serde(flatten)]
                data: UpdateUserRequest,
            }
            
            let params: UpdateParams = msg.deserialize_payload()?;
            match UserService::update_user(params.user_id, params.data, db).await {
                Ok(user) => Ok(RpcResponse::success(user, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("delete_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let user_id: u32 = msg.deserialize_payload()?;
            match UserService::delete_user(user_id, db).await {
                Ok(result) => Ok(RpcResponse::success(result, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("list_users", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            match UserService::list_users(db).await {
                Ok(users) => Ok(RpcResponse::success(users, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    service.register_function("ping", |_msg: Message| async move {
        match UserService::ping().await {
            Ok(result) => Ok(RpcResponse::success(result, 0)?),
            Err(e) => Ok(RpcResponse::error(e)),
        }
    }).await;

    info!("✅ User Service registered all handlers");
    
    // Start the service (this will run forever)
    service.start().await?;
    
    Ok(())
}