//! Example showing how to use RabbitMesh macros for service definition
//!
//! This demonstrates the three core macros:
//! - #[service_definition]: Marks a struct as a RabbitMesh service
//! - #[service_impl]: Processes impl blocks to auto-register methods
//! - #[service_method]: Defines HTTP routes and RPC handlers

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

// The macros automatically generate:
// - Service registration methods (create_service, service_name, etc.)
// - Route discovery methods (get_routes)
// - RPC handler registration
// - HTTP route mapping
// - Service discovery integration

fn main() {
    println!("RabbitMesh Service Definition Example");
    println!("=====================================");
    println!();
    println!("The macros above automatically generate:");
    println!("✅ Service registration: GreetingService::create_service()");
    println!("✅ Service name: GreetingService::service_name()");  
    println!("✅ Route discovery: GreetingService::get_routes()");
    println!("✅ RPC handlers for: say_hello, get_greeting, health_check");
    println!("✅ HTTP routes: POST /hello, GET /greeting/:name, GET /health");
    println!("✅ RabbitMQ queues: rabbitmesh.GreetingService.*");
    println!("✅ JSON serialization/deserialization");
    println!("✅ Service discovery integration");
    println!();
    println!("All from just a few macro annotations! 🪄");
}