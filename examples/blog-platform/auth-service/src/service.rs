use crate::models::*;
use crate::utils::*;
use blog_common::ValidateTokenRequest;
use rabbitmesh::{Message, MicroService, RpcResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub type UserDatabase = Arc<RwLock<HashMap<String, User>>>;
pub type TokenStore = Arc<RwLock<HashMap<String, String>>>;

pub async fn setup_sample_users(db: &UserDatabase) -> Result<(), Box<dyn std::error::Error>> {
    let mut users = db.write().await;
    
    let user1_id = generate_user_id();
    users.insert(user1_id.clone(), User {
        id: user1_id.clone(),
        username: "john_doe".to_string(),
        email: "john@example.com".to_string(),
        password_hash: hash_password("password123")?,
        created_at: chrono::Utc::now().to_rfc3339(),
        is_active: true,
    });

    let user2_id = generate_user_id();
    users.insert(user2_id.clone(), User {
        id: user2_id.clone(),
        username: "jane_smith".to_string(),
        email: "jane@example.com".to_string(),
        password_hash: hash_password("password456")?,
        created_at: chrono::Utc::now().to_rfc3339(),
        is_active: true,
    });

    info!("✅ Sample users created");
    Ok(())
}

pub async fn register_auth_handlers(
    service: &MicroService,
    db: UserDatabase,
    tokens: TokenStore,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Register user handler
    let db_clone = db.clone();
    let tokens_clone = tokens.clone();
    service.register_function("register", move |msg: Message| {
        let db = db_clone.clone();
        let tokens = tokens_clone.clone();
        async move {
            let request: RegisterRequest = msg.deserialize_payload()?;
            info!("Registering user: {}", request.username);
            
            let mut users = db.write().await;
            
            // Check if username already exists
            for user in users.values() {
                if user.username == request.username {
                    return Ok(RpcResponse::error("Username already exists".to_string()));
                }
            }
            
            let user_id = generate_user_id();
            let password_hash = match hash_password(&request.password) {
                Ok(hash) => hash,
                Err(e) => {
                    error!("Password hashing failed: {}", e);
                    return Ok(RpcResponse::error(format!("Password hashing failed: {}", e)));
                }
            };
            
            let user = User {
                id: user_id.clone(),
                username: request.username.clone(),
                email: request.email,
                password_hash,
                created_at: chrono::Utc::now().to_rfc3339(),
                is_active: true,
            };
            
            users.insert(user_id.clone(), user.clone());
            
            // Generate token
            let token = generate_token(&user_id);
            let mut token_store = tokens.write().await;
            token_store.insert(token.clone(), user_id.clone());
            
            let response = AuthResponse {
                user_id: user.id,
                username: user.username,
                email: user.email,
                token,
            };
            
            info!("User registered successfully: {}", request.username);
            Ok(RpcResponse::success(response, 15)?)
        }
    }).await;

    // Login handler
    let db_clone = db.clone();
    let tokens_clone = tokens.clone();
    service.register_function("login", move |msg: Message| {
        let db = db_clone.clone();
        let tokens = tokens_clone.clone();
        async move {
            let request: LoginRequest = msg.deserialize_payload()?;
            info!("Login attempt for: {}", request.username);
            
            let users = db.read().await;
            
            // Find user by username
            let user = users.values()
                .find(|u| u.username == request.username && u.is_active)
                .cloned();
            
            match user {
                Some(user) => {
                    match verify_password(&request.password, &user.password_hash) {
                        Ok(true) => {
                            let token = generate_token(&user.id);
                            let mut token_store = tokens.write().await;
                            token_store.insert(token.clone(), user.id.clone());
                            
                            let response = AuthResponse {
                                user_id: user.id,
                                username: user.username,
                                email: user.email,
                                token,
                            };
                            
                            info!("Login successful for: {}", request.username);
                            Ok(RpcResponse::success(response, 10)?)
                        }
                        Ok(false) => {
                            info!("Invalid password for: {}", request.username);
                            Ok(RpcResponse::error("Invalid credentials".to_string()))
                        }
                        Err(e) => {
                            error!("Password verification error: {}", e);
                            Ok(RpcResponse::error("Authentication failed".to_string()))
                        }
                    }
                }
                None => {
                    info!("User not found: {}", request.username);
                    Ok(RpcResponse::error("Invalid credentials".to_string()))
                }
            }
        }
    }).await;

    // Validate token handler (for other services)
    let db_clone = db.clone();
    let tokens_clone = tokens.clone();
    service.register_function("validate_token", move |msg: Message| {
        let db = db_clone.clone();
        let tokens = tokens_clone.clone();
        async move {
            let request: ValidateTokenRequest = msg.deserialize_payload()?;
            
            let tokens_store = tokens.read().await;
            if let Some(user_id) = tokens_store.get(&request.token) {
                let users = db.read().await;
                if let Some(user) = users.get(user_id) {
                    if user.is_active {
                        let profile = UserProfile {
                            id: user.id.clone(),
                            username: user.username.clone(),
                            email: user.email.clone(),
                            created_at: user.created_at.clone(),
                            is_active: user.is_active,
                        };
                        
                        return Ok(RpcResponse::success(profile, 5)?);
                    }
                }
            }
            
            Ok(RpcResponse::error("Invalid or expired token".to_string()))
        }
    }).await;

    // Get user profile handler
    let db_clone = db.clone();
    service.register_function("get_user", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let user_id: String = msg.deserialize_payload()?;
            
            let users = db.read().await;
            if let Some(user) = users.get(&user_id) {
                let profile = UserProfile {
                    id: user.id.clone(),
                    username: user.username.clone(),
                    email: user.email.clone(),
                    created_at: user.created_at.clone(),
                    is_active: user.is_active,
                };
                
                Ok(RpcResponse::success(profile, 5)?)
            } else {
                Ok(RpcResponse::error("User not found".to_string()))
            }
        }
    }).await;

    // Health check
    service.register_function("ping", |_msg: Message| async move {
        Ok(RpcResponse::success("Auth service is healthy", 1)?)
    }).await;

    Ok(())
}