mod config;
mod models;
mod service;
mod utils;

use config::*;
use rabbitmesh::MicroService;
use service::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use time::macros::format_description;
use tracing_subscriber::fmt::time::UtcTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with better timestamp formatting
    let time_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let timer = UtcTime::new(time_format);
    
    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Auth Service");
    info!("🔐 Handles user authentication, registration & token validation");
    
    // Create databases
    let user_db: UserDatabase = Arc::new(RwLock::new(HashMap::new()));
    let token_store: TokenStore = Arc::new(RwLock::new(HashMap::new()));
    
    // Setup sample users
    setup_sample_users(&user_db).await?;
    
    // Create microservice
    let service = MicroService::new_simple(SERVICE_NAME, RABBITMQ_URL).await?;
    
    // Register handlers
    register_auth_handlers(&service, user_db, token_store).await?;
    
    info!("✅ Auth service handlers registered");
    info!("🎯 Listening to RabbitMQ queue: rabbitmesh.{}", SERVICE_NAME);
    info!("🔗 Available methods:");
    info!("   - register: Create new user account");
    info!("   - login: Authenticate user and get token");
    info!("   - validate_token: Validate token (used by other services)");
    info!("   - get_user: Get user profile by ID");
    info!("   - ping: Health check");
    
    // Start the service
    service.start().await?;
    Ok(())
}