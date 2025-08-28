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

    info!("🚀 Starting Post Service");
    info!("📝 Handles blog posts with auth service integration");
    
    // Create post database
    let post_db: PostDatabase = Arc::new(RwLock::new(HashMap::new()));
    
    // Setup sample posts
    setup_sample_posts(&post_db).await?;
    
    // Create microservice
    let service = MicroService::new_simple(SERVICE_NAME, RABBITMQ_URL).await?;
    
    // Create service client for inter-service communication
    let client = Arc::new(ServiceClient::new("post-client", RABBITMQ_URL).await?);
    
    info!("🔗 Connected to other services via RabbitMQ");
    
    // Register handlers
    register_post_handlers(&service, client, post_db).await?;
    
    info!("✅ Post service handlers registered");
    info!("🎯 Listening to RabbitMQ queue: rabbitmesh.{}", SERVICE_NAME);
    info!("🔗 Available methods:");
    info!("   - create_post: Create new blog post (requires auth)");
    info!("   - update_post: Update existing post (requires auth + ownership)");
    info!("   - list_posts: Get all published posts");
    info!("   - get_post: Get specific post by ID");
    info!("   - get_posts_by_author: Get posts by author ID");
    info!("   - ping: Health check");
    info!("");
    info!("🔐 This service communicates with:");
    info!("   - Auth Service: Token validation & user info");
    
    // Start the service
    service.start().await?;
    Ok(())
}