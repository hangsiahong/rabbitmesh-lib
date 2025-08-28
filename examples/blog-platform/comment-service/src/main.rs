mod config;
mod models;
mod service;
mod utils;

use config::*;
use rabbitmesh::{MicroService, ServiceClient};
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

    info!("🚀 Starting Comment Service");
    info!("💬 Handles blog comments with multi-service integration");
    
    // Create comment database
    let comment_db: CommentDatabase = Arc::new(RwLock::new(HashMap::new()));
    
    // Setup sample comments
    setup_sample_comments(&comment_db).await?;
    
    // Create microservice
    let service = MicroService::new_simple(SERVICE_NAME, RABBITMQ_URL).await?;
    
    // Create service client for inter-service communication
    let client = Arc::new(ServiceClient::new("comment-client", RABBITMQ_URL).await?);
    
    info!("🔗 Connected to other services via RabbitMQ");
    
    // Register handlers
    register_comment_handlers(&service, client, comment_db).await?;
    
    info!("✅ Comment service handlers registered");
    info!("🎯 Listening to RabbitMQ queue: rabbitmesh.{}", SERVICE_NAME);
    info!("🔗 Available methods:");
    info!("   - create_comment: Create comment (requires auth + post verification)");
    info!("   - update_comment: Update comment (requires auth + ownership)");
    info!("   - get_comments_by_post: Get all comments for a specific post");
    info!("   - list_comments: Get all approved comments");
    info!("   - get_comment: Get specific comment by ID");
    info!("   - ping: Health check");
    info!("");
    info!("🔐 This service communicates with:");
    info!("   - Auth Service: Token validation & user info");
    info!("   - Post Service: Post existence verification");
    info!("");
    info!("🌐 Multi-service workflow for creating comments:");
    info!("   1. Validate user token with Auth Service");
    info!("   2. Verify post exists with Post Service");
    info!("   3. Create comment in Comment Service database");
    
    // Start the service
    service.start().await?;
    Ok(())
}