mod models;
mod service;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;
use service::TodoService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,simple_todo=debug,rabbitmesh=debug")
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting Todo Service...");

    // Initialize the todo service storage
    TodoService::init().await?;
    info!("✅ Todo service storage initialized");

    // Get the RabbitMQ URL from environment or use default
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());

    info!("🐰 Connecting to RabbitMQ at: {}", rabbitmq_url);

    // Create and start the service using the macro-generated methods
    let service = TodoService::create_service(&rabbitmq_url).await?;
    
    info!("✅ Todo Service started successfully");
    info!("🎯 Service name: {}", TodoService::service_name());
    
    // Show auto-discovered routes
    let routes = TodoService::get_routes();
    info!("📊 Auto-discovered {} HTTP routes:", routes.len());
    for (route, method) in routes {
        info!("   {} -> {}", method, route);
    }
    
    info!("📞 Starting service listener...");
    service.start().await?;
    
    Ok(())
}