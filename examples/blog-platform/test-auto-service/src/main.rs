use rabbitmesh_macros::{service_definition, service_method, service_impl};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub id: u32,
    pub name: String,
}

#[service_definition]
pub struct UserService;

#[service_impl]
impl UserService {
    #[service_method("GET /users/:id")]
    pub async fn get_user(user_id: u32) -> Result<User, String> {
        info!("Getting user with ID: {}", user_id);
        Ok(User { id: user_id, name: "John Doe".to_string() })
    }
    
    #[service_method("POST /users")]
    pub async fn create_user(data: CreateUserRequest) -> Result<User, String> {
        info!("Creating user with name: {}", data.name);
        Ok(User { id: 1, name: data.name })
    }

    #[service_method("PUT /users/:id")]
    pub async fn update_user(data: UpdateUserRequest) -> Result<User, String> {
        info!("Updating user {} with name: {}", data.id, data.name);
        Ok(User { id: data.id, name: data.name })
    }

    #[service_method]
    pub async fn ping() -> Result<String, String> {
        Ok("pong".to_string())
    }

    #[service_method]
    pub async fn health_check() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "UserService"
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Fully Automated Test Service");
    info!("Service name: {}", UserService::service_name());
    
    // Show auto-discovered routes
    let routes = UserService::get_routes();
    info!("📊 Auto-discovered {} HTTP routes:", routes.len());
    for (route, method) in routes {
        info!("   {} -> {}", route, method);
    }
    
    let service = UserService::create_service("amqp://localhost:5672").await?;
    
    info!("✅ Service created with auto-generated handlers");
    info!("🎯 Listening on RabbitMQ queue: rabbitmesh.{}", UserService::service_name());
    info!("🔥 Zero boilerplate - pure business logic!");
    
    service.start().await?;
    Ok(())
}