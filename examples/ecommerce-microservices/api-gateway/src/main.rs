//! # API Gateway
//! 
//! Auto-generates REST API for all microservices.
//! This is the ONLY service that exposes an HTTP port!

use rabbitmesh_gateway::create_auto_router;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌐 Starting API Gateway");
    info!("🔥 This is the ONLY service with an HTTP port!");

    // Create auto-router that talks to microservices via RabbitMQ
    let app = create_auto_router("amqp://localhost:5672").await?;
    
    // Bind to HTTP port (this is the ONLY port in our entire system!)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await?;
    
    info!("✅ API Gateway running on http://0.0.0.0:3333");
    info!("📋 Auto-generated endpoints for all microservices:");
    info!("");
    info!("   🏥 Health Checks:");
    info!("     Gateway:        GET  /health");
    info!("     User Service:   GET  /health/user-service");
    info!("     Product Svc:    GET  /health/product-service");
    info!("");
    info!("   👤 User Service (via RabbitMQ):");
    info!("     List users:     GET  /api/v1/user-service/list_users");
    info!("     Get user:       GET  /api/v1/user-service/get_user?user_id=1");  
    info!("     Create user:    POST /api/v1/user-service/create_user");
    info!("                     {{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}");
    info!("");
    info!("   📦 Product Service (via RabbitMQ):");
    info!("     List products:  GET  /api/v1/product-service/list_products");
    info!("     Get product:    GET  /api/v1/product-service/get_product?product_id=1");
    info!("     By category:    GET  /api/v1/product-service/get_products_by_category?category=Electronics");
    info!("     Create product: POST /api/v1/product-service/create_product");
    info!("                     {{\"name\":\"Book\",\"description\":\"Programming book\",\"price\":29.99,\"category\":\"Books\"}}");
    info!("");
    info!("🎯 Example commands:");
    info!("   curl http://localhost:3333/health");
    info!("   curl http://localhost:3333/api/v1/user-service/list_users");
    info!("   curl http://localhost:3333/api/v1/product-service/list_products");
    info!("");
    info!("🚀 All requests: HTTP -> Gateway -> RabbitMQ -> Microservice");
    info!("   No direct HTTP calls to microservices!");
    
    // Start serving HTTP requests
    // This is where HTTP requests get converted to RabbitMQ messages!
    axum::serve(listener, app).await?;
    
    Ok(())
}