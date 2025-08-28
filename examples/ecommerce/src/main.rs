//! # RabbitMesh Ecommerce Demo
//!
//! Complete microservices ecommerce system demonstrating:
//! - Zero-port microservices (only RabbitMQ connections)
//! - Auto-generated REST API gateway
//! - Inter-service communication via message passing
//! - Horizontal scaling with automatic load balancing
//!
//! ## Architecture
//! 
//! ```
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────────────┐
//! │   Client    │────│ API Gateway │────│        Microservices           │
//! │ (HTTP/JSON) │    │  (Port 3000)│    │        (NO PORTS!)             │
//! └─────────────┘    └─────────────┘    │                                 │
//!                           │           │  ┌─────────────────────────┐  │
//!                           │           │  │      RabbitMQ           │  │
//!                           └───────────┼──│   Message Broker        │  │
//!                                       │  └─────────────────────────┘  │
//!                                       │      ↕       ↕       ↕        │
//!                                       │  ┌─────┐ ┌─────┐ ┌─────┐      │
//!                                       │  │User │ │Prod │ │Order│      │
//!                                       │  │Svc  │ │Svc  │ │Svc  │      │
//!                                       │  └─────┘ └─────┘ └─────┘      │
//!                                       └─────────────────────────────────┘
//! ```
//!
//! ## Running the Demo
//!
//! 1. Start RabbitMQ: `docker run -d -p 5672:5672 -p 15672:15672 rabbitmq:3-management`
//! 2. Run this demo: `cargo run`
//! 3. Test APIs: `curl http://localhost:3000/api/v1/user-service/list_users`

mod user_service;
mod product_service; 
mod order_service;
mod gateway;

use tracing_subscriber;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 RabbitMesh Ecommerce Demo");
    println!("============================");
    println!("");
    println!("This demo shows a complete microservices ecommerce system:");
    println!("• User Service - User registration and management");
    println!("• Product Service - Product catalog and inventory");
    println!("• Order Service - Order processing and fulfillment"); 
    println!("• API Gateway - Auto-generated REST APIs");
    println!("");
    println!("🔥 KEY FEATURES:");
    println!("✅ Zero port management - services only connect to RabbitMQ");
    println!("✅ Never blocks - every request spawns async task");
    println!("✅ Auto load balancing - RabbitMQ distributes requests");
    println!("✅ Fault tolerance - message persistence + retries");
    println!("✅ Inter-service calls - services call each other via RabbitMQ");
    println!("✅ Auto-generated API - REST endpoints created automatically");
    println!("");
    
    // Check if RabbitMQ is available
    println!("🐰 Checking RabbitMQ connection...");
    match rabbitmesh::ServiceClient::new("health-check", "amqp://localhost:5672").await {
        Ok(_) => println!("✅ RabbitMQ is running"),
        Err(_) => {
            println!("❌ RabbitMQ is not available!");
            println!("Please start RabbitMQ: docker run -d -p 5672:5672 -p 15672:15672 rabbitmq:3-management");
            std::process::exit(1);
        }
    }
    
    println!("");
    println!("🏁 Starting all services concurrently...");

    // Start all services concurrently (they all run forever)
    let user_service = tokio::spawn(async move {
        if let Err(e) = user_service::run().await {
            tracing::error!("User service error: {}", e);
        }
    });
    
    let product_service = tokio::spawn(async move {
        if let Err(e) = product_service::run().await {
            tracing::error!("Product service error: {}", e);
        }
    });
    
    let order_service = tokio::spawn(async move {
        if let Err(e) = order_service::run().await {
            tracing::error!("Order service error: {}", e);
        }
    });
    
    let gateway = tokio::spawn(async move {
        if let Err(e) = gateway::run().await {
            tracing::error!("Gateway error: {}", e);
        }
    });

    println!("🎯 All services starting up...");
    println!("");
    println!("Once all services are ready, try these API calls:");
    println!("");
    println!("# Health checks");
    println!("curl http://localhost:3000/health");
    println!("curl http://localhost:3000/health/user-service");
    println!("");
    println!("# User operations");
    println!("curl http://localhost:3000/api/v1/user-service/list_users");
    println!("curl http://localhost:3000/api/v1/user-service/get_user/1");
    println!("curl -X POST http://localhost:3000/api/v1/user-service/create_user \\");
    println!("  -H 'Content-Type: application/json' \\");  
    println!("  -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}'");
    println!("");
    println!("# Product operations");
    println!("curl http://localhost:3000/api/v1/product-service/list_products");
    println!("curl http://localhost:3000/api/v1/product-service/get_product/1");
    println!("curl http://localhost:3000/api/v1/product-service/get_products_by_category/Electronics");
    println!("");
    println!("# Order operations (demonstrates inter-service calls)");
    println!("curl http://localhost:3000/api/v1/order-service/list_orders");
    println!("curl -X POST http://localhost:3000/api/v1/order-service/create_order \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"user_id\":1,\"items\":[{{\"product_id\":1,\"quantity\":1}}]}}'");
    println!("");

    // Wait for any service to exit (shouldn't happen in normal operation)
    tokio::select! {
        result = user_service => {
            tracing::error!("User service exited: {:?}", result);
        }
        result = product_service => {
            tracing::error!("Product service exited: {:?}", result);
        }
        result = order_service => {
            tracing::error!("Order service exited: {:?}", result);
        }
        result = gateway => {
            tracing::error!("Gateway exited: {:?}", result);
        }
    }

    Ok(())
}