use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;
use rabbitmesh_gateway::create_auto_router;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,simple_todo=debug,rabbitmesh=debug")
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("🚀 Starting Todo API Gateway...");

    // Get configuration from environment
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());
    
    let http_port = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    info!("🐰 Connecting to RabbitMQ at: {}", rabbitmq_url);
    info!("🌐 Starting HTTP server on port: {}", http_port);

    // Create auto-generated router from service definitions
    let app = create_auto_router(&rabbitmq_url).await?;
    
    info!("✅ Auto-generated API Gateway created");
    info!("📖 Available endpoints auto-discovered from services");
    info!("🔍 Health Check: http://localhost:{}/health", http_port);
    info!("📊 Service Registry: http://localhost:{}/services", http_port);
    
    // Start the HTTP server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", http_port)).await?;
    info!("✅ Todo API Gateway started successfully");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}