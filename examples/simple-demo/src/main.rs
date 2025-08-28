//! # Simple RabbitMesh Demo
//!
//! This demonstrates the core RabbitMesh functionality:
//! - User microservice with zero port exposure
//! - Gateway that auto-generates REST API
//! - All communication via RabbitMQ

use serde::{Deserialize, Serialize};
use tracing::{info, error};
use rabbitmesh::{MicroService, RpcResponse, Message};
use rabbitmesh_gateway::create_auto_router;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

async fn run_user_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting User Service");

    let service = MicroService::new_simple("user-service", "amqp://localhost:5672").await?;
    
    // Register get_user handler
    service.register_function("get_user", |msg: Message| async move {
        info!("User service handling get_user request");
        let user_id: u32 = msg.deserialize_payload()?;
        
        let user = User {
            id: user_id,
            name: format!("User {}", user_id),
            email: format!("user{}@example.com", user_id),
        };
        
        Ok(RpcResponse::success(user, 10)?)
    }).await;

    // Register create_user handler
    service.register_function("create_user", |msg: Message| async move {
        info!("User service handling create_user request");
        let request: CreateUserRequest = msg.deserialize_payload()?;
        
        let user = User {
            id: 1,
            name: request.name,
            email: request.email,
        };
        
        Ok(RpcResponse::success(user, 20)?)
    }).await;

    // Register list_users handler
    service.register_function("list_users", |_msg: Message| async move {
        info!("User service handling list_users request");
        
        let users = vec![
            User {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
            },
        ];
        
        Ok(RpcResponse::success(users, 5)?)
    }).await;

    // Register ping handler for health checks
    service.register_function("ping", |_msg: Message| async move {
        info!("User service handling ping");
        Ok(RpcResponse::success("User service is healthy", 1)?)
    }).await;

    info!("✅ User service handlers registered");
    service.start().await?;
    Ok(())
}

async fn run_gateway() -> Result<(), Box<dyn std::error::Error>> {
    info!("🌐 Starting API Gateway");
    
    let app = create_auto_router("amqp://localhost:5672").await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    
    info!("✅ API Gateway running on http://0.0.0.0:3001");
    info!("📋 Try these endpoints:");
    info!("   Health:      curl http://localhost:3001/health");
    info!("   List users:  curl http://localhost:3001/api/v1/user-service/list_users");
    info!("   Get user:    curl http://localhost:3001/api/v1/user-service/get_user?user_id=1");
    info!("   Create user: curl -X POST http://localhost:3001/api/v1/user-service/create_user \\");
    info!("                     -H 'Content-Type: application/json' \\");
    info!("                     -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}'");
    
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 RabbitMesh Simple Demo");
    println!("=========================");
    println!();
    println!("This demonstrates:");
    println!("✅ Zero-port microservice (user-service connects only to RabbitMQ)");
    println!("✅ Auto-generated REST API (gateway creates endpoints automatically)");
    println!("✅ Message-driven architecture (HTTP -> RabbitMQ -> Service)");
    println!();

    // Check RabbitMQ connection
    match rabbitmesh::ServiceClient::new("health-check", "amqp://localhost:5672").await {
        Ok(_) => println!("✅ RabbitMQ is running"),
        Err(_) => {
            println!("❌ RabbitMQ not available!");
            println!("Start with: docker run -d -p 5672:5672 -p 15672:15672 rabbitmq:3-management");
            return Ok(());
        }
    }
    
    println!();
    println!("🏁 Starting services...");

    // Start both services concurrently
    let user_service = tokio::spawn(async move {
        if let Err(e) = run_user_service().await {
            error!("User service error: {}", e);
        }
    });

    let gateway = tokio::spawn(async move {
        if let Err(e) = run_gateway().await {
            error!("Gateway error: {}", e);
        }
    });

    // Wait for either to complete (they should run forever)
    tokio::select! {
        _ = user_service => {
            error!("User service exited unexpectedly");
        }
        _ = gateway => {
            error!("Gateway exited unexpectedly");
        }
    }

    Ok(())
}