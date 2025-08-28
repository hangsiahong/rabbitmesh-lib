use rabbitmesh_gateway::create_auto_router;
use tracing::info;

/// Run the API Gateway
/// 
/// This gateway automatically exposes all microservices as REST APIs
/// without any manual configuration. It talks to microservices via RabbitMQ.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("🌐 Starting API Gateway");

    // Create auto-router that talks to microservices via RabbitMQ
    let app = create_auto_router("amqp://localhost:5672").await?;
    
    // Bind to HTTP port (this is the ONLY port in our entire system!)
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    
    info!("✅ API Gateway running on http://0.0.0.0:3000");
    info!("📋 Available endpoints:");
    info!("   Health Check:     GET  /health");
    info!("   Service Health:   GET  /health/:service");
    info!("   User Service:");
    info!("     List users:     GET  /api/v1/user-service/list_users");
    info!("     Get user:       GET  /api/v1/user-service/get_user/:id");  
    info!("     Create user:    POST /api/v1/user-service/create_user");
    info!("     Update user:    PUT  /api/v1/user-service/update_user/:id");
    info!("     Delete user:    DELETE /api/v1/user-service/delete_user/:id");
    info!("   Product Service:");
    info!("     List products:  GET  /api/v1/product-service/list_products");
    info!("     Get product:    GET  /api/v1/product-service/get_product/:id");
    info!("     Create product: POST /api/v1/product-service/create_product");
    info!("     Update product: PUT  /api/v1/product-service/update_product/:id");  
    info!("     Delete product: DELETE /api/v1/product-service/delete_product/:id");
    info!("     By category:    GET  /api/v1/product-service/get_products_by_category/:category");
    info!("   Order Service:");
    info!("     List orders:    GET  /api/v1/order-service/list_orders");
    info!("     Get order:      GET  /api/v1/order-service/get_order/:id");
    info!("     Create order:   POST /api/v1/order-service/create_order");
    info!("     Update status:  PUT  /api/v1/order-service/update_order_status/:id");
    info!("     Cancel order:   DELETE /api/v1/order-service/cancel_order/:id");
    info!("     User orders:    GET  /api/v1/order-service/list_user_orders/:user_id");
    info!("");
    info!("🎯 Example API calls:");
    info!("   curl http://localhost:3000/health");
    info!("   curl http://localhost:3000/api/v1/user-service/list_users");
    info!("   curl http://localhost:3000/api/v1/user-service/get_user/1");
    info!("   curl -X POST http://localhost:3000/api/v1/user-service/create_user \\");
    info!("        -H 'Content-Type: application/json' \\");
    info!("        -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}'");
    info!("");

    // Start serving HTTP requests
    // This is where HTTP requests get converted to RabbitMQ messages!
    axum::serve(listener, app).await?;
    
    Ok(())
}