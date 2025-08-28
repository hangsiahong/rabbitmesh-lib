//! Simple RabbitMesh service example showing the macro-based approach
//!
//! This example demonstrates how to build a microservice using RabbitMesh's
//! declarative macro system. The service automatically gets HTTP routes,
//! RabbitMQ RPC handlers, and service discovery.

use rabbitmesh_macros::{service_definition, service_impl};
use serde::{Deserialize, Serialize};

// Request/Response models
#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ItemResponse {
    pub success: bool,
    pub message: String,
    pub item_id: Option<String>,
}

// Service definition - this creates the entire service infrastructure
#[service_definition]
pub struct ItemService;

// Service implementation - only business logic needed!
#[service_impl]
impl ItemService {
    /// Create a new item
    /// 
    /// Framework automatically:
    /// - Extracts HTTP route: POST /items
    /// - Registers RPC handler: create_item
    /// - Sets up RabbitMQ queue: rabbitmesh.ItemService.create_item
    /// - Handles JSON serialization/deserialization
    /// - Creates HTTP endpoint in auto-generated gateway
    #[service_method("POST /items")]
    pub async fn create_item(request: CreateItemRequest) -> Result<ItemResponse, String> {
        // JUST YOUR BUSINESS LOGIC!
        println!("Creating item: {} - {}", request.name, request.description);
        
        let item_id = uuid::Uuid::new_v4().to_string();
        
        Ok(ItemResponse {
            success: true,
            message: "Item created successfully".to_string(),
            item_id: Some(item_id),
        })
    }

    /// Get an item by ID
    /// 
    /// Framework automatically extracts 'item_id' from URL path
    #[service_method("GET /items/:id")]
    pub async fn get_item(item_id: String) -> Result<ItemResponse, String> {
        println!("Getting item: {}", item_id);
        
        Ok(ItemResponse {
            success: true,
            message: format!("Retrieved item {}", item_id),
            item_id: Some(item_id),
        })
    }

    /// List all items
    #[service_method("GET /items")]
    pub async fn list_items() -> Result<serde_json::Value, String> {
        println!("Listing all items");
        
        Ok(serde_json::json!({
            "success": true,
            "message": "Items retrieved successfully",
            "items": [],
            "total": 0
        }))
    }

    /// Update an item
    /// 
    /// Framework handles tuple parameters: (path_param, request_body)
    #[service_method("PUT /items/:id")]
    pub async fn update_item(params: (String, UpdateItemRequest)) -> Result<ItemResponse, String> {
        let (item_id, _request) = params;
        println!("Updating item: {}", item_id);
        
        Ok(ItemResponse {
            success: true,
            message: "Item updated successfully".to_string(),
            item_id: Some(item_id),
        })
    }

    /// Delete an item
    #[service_method("DELETE /items/:id")]
    pub async fn delete_item(item_id: String) -> Result<ItemResponse, String> {
        println!("Deleting item: {}", item_id);
        
        Ok(ItemResponse {
            success: true,
            message: "Item deleted successfully".to_string(),
            item_id: Some(item_id),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    println!("🚀 Starting ItemService...");

    // Get RabbitMQ URL
    let rabbitmq_url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string());

    // Create and start service using RabbitMesh magic
    let service = ItemService::create_service(&rabbitmq_url).await?;
    
    println!("✅ ItemService started successfully");
    println!("🎯 Service name: {}", ItemService::service_name());
    
    // Show the magic - auto-discovered routes
    let routes = ItemService::get_routes();
    println!("✨ Auto-discovered {} HTTP routes:", routes.len());
    for (route, method) in routes {
        println!("   {} -> {}", method, route);
    }
    
    println!("🪄 This demonstrates RabbitMesh magic:");
    println!("   ✅ Routes extracted from #[service_method] annotations");
    println!("   ✅ RPC handlers auto-registered for each method");
    println!("   ✅ HTTP → RabbitMQ mapping handled automatically");
    println!("   ✅ Service discovery enabled via RabbitMQ queues");
    println!("   ✅ JSON serialization/deserialization automatic");
    println!("   ✅ Zero configuration required!");

    println!("📞 Starting service listener...");
    
    // Start the service - this is where the magic happens!
    service.start().await?;
    
    Ok(())
}