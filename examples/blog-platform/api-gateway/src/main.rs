mod config;

use config::*;
use rabbitmesh_gateway::create_auto_router;
use tracing::info;
use axum::Router;
use utoipa_swagger_ui::SwaggerUi;
use blog_common::openapi::get_openapi_spec;
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

    info!("🌐 Starting Blog Platform API Gateway");
    info!("📝 Auto-generating REST APIs for all blog microservices!");

    // Create auto-router that talks to microservices via RabbitMQ
    let api_router = create_auto_router(RABBITMQ_URL).await?;
    
    // Add OpenAPI documentation
    let app = Router::new()
        .merge(api_router)
        .merge(
            SwaggerUi::new("/docs")
                .url("/api-docs/openapi.json", get_openapi_spec())
        );
    
    // Bind to HTTP port (this is the ONLY port in our entire system!)
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", GATEWAY_PORT)).await?;
    
    info!("✅ Blog API Gateway running on http://0.0.0.0:{}", GATEWAY_PORT);
    info!("📋 Interactive API Documentation: http://localhost:{}/docs", GATEWAY_PORT);  
    info!("🔗 OpenAPI Spec: http://localhost:{}/api-docs/openapi.json", GATEWAY_PORT);
    info!("");
    info!("🚀 Zero-port microservices with service-to-service communication!");
    info!("💬 All microservices communicate via RabbitMQ - no direct HTTP calls!");
    
    // Start serving HTTP requests
    axum::serve(listener, app).await?;
    
    Ok(())
}