//! RabbitMesh Macros - Syntax Example  
//! 
//! This file demonstrates RabbitMesh macro syntax for documentation purposes.
//! It cannot be compiled within the macros crate due to circular dependencies.
//!
//! For working examples, see:
//! - `examples/simple-todo/` - Complete working service with full dependencies
//! - `rabbitmesh/examples/simple_service.rs` - Basic working example

// Example syntax (for documentation only):
/*

use rabbitmesh_macros::{service_definition, service_impl};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct HelloResponse {
    pub message: String,
    pub timestamp: String,
}

// Step 1: Define your service struct
#[service_definition]
pub struct GreetingService;

// Step 2: Implement methods with automatic registration
#[service_impl]
impl GreetingService {
    /// Say hello - becomes HTTP POST /hello and RPC handler 
    #[service_method("POST /hello")]
    pub async fn say_hello(request: HelloRequest) -> Result<HelloResponse, String> {
        Ok(HelloResponse {
            message: format!("Hello, {}!", request.name),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Get greeting - becomes HTTP GET /greeting/:name
    #[service_method("GET /greeting/:name")]
    pub async fn get_greeting(name: String) -> Result<HelloResponse, String> {
        Ok(HelloResponse {
            message: format!("Greetings, {}!", name),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Health check - becomes HTTP GET /health
    #[service_method("GET /health")]
    pub async fn health_check() -> Result<String, String> {
        Ok("Service is healthy!".to_string())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create and start service using generated methods
    let service = GreetingService::create_service("amqp://localhost:5672").await?;
    
    println!("🚀 GreetingService started!");
    println!("✨ Auto-discovered routes:");
    for (route, method) in GreetingService::get_routes() {
        println!("   {} -> {}", method, route);
    }
    
    service.start().await?;
    Ok(())
}

*/

fn main() {
    // This example shows macro syntax only.
    // For working examples, run:
    //   cargo run --example simple_service (in rabbitmesh crate)
    //   cargo run --bin todo-service (in examples/simple-todo)
    println!("📖 This is a syntax documentation example.");
    println!("🚀 See working examples in:");
    println!("   - examples/simple-todo/ (complete service)");
    println!("   - rabbitmesh/examples/ (basic examples)");
}

// The macros automatically generate:
// - Service registration methods (create_service, service_name, etc.)
// - Route discovery methods (get_routes)
// - RPC handler registration
// - HTTP route mapping
// - Service discovery integration

