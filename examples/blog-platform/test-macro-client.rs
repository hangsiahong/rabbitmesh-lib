use rabbitmesh::ServiceClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserRequest {
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let client = ServiceClient::new("test-client", "amqp://localhost:5672").await?;
    
    println!("🧪 Testing macro-generated UserService");
    
    // Test ping
    let response = client.call_with_timeout(
        "UserService-service",
        "ping", 
        (),
        Duration::from_secs(5)
    ).await?;
    
    let result: String = response.data()?;
    println!("✅ ping() -> {}", result);
    
    // Test get_user
    let response = client.call_with_timeout(
        "UserService-service",
        "get_user", 
        42u32,
        Duration::from_secs(5)
    ).await?;
    
    let user: User = response.data()?;
    println!("✅ get_user(42) -> {:?}", user);
    
    // Test create_user
    let response = client.call_with_timeout(
        "UserService-service",
        "create_user", 
        CreateUserRequest { name: "Alice".to_string() },
        Duration::from_secs(5)
    ).await?;
    
    let user: User = response.data()?;
    println!("✅ create_user(Alice) -> {:?}", user);
    
    println!("🎉 All macro service tests passed!");
    
    Ok(())
}