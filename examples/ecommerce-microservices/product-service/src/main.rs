//! # Product Service
//! 
//! Manages product catalog, inventory, and product information.
//! This service exposes NO HTTP ports - it only connects to RabbitMQ.

use rabbitmesh::{MicroService, RpcResponse, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, debug};

/// Product data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub in_stock: bool,
    pub created_at: String,
}

/// Request to create a new product
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
}

/// In-memory product database
type ProductDatabase = Arc<RwLock<HashMap<u32, Product>>>;

async fn setup_sample_data(db: &ProductDatabase) {
    let mut products = db.write().await;
    products.insert(1, Product {
        id: 1,
        name: "Laptop".to_string(),
        description: "High-performance laptop for developers".to_string(),
        price: 1299.99,
        category: "Electronics".to_string(),
        in_stock: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    products.insert(2, Product {
        id: 2,
        name: "Wireless Mouse".to_string(),
        description: "Ergonomic wireless mouse".to_string(),
        price: 49.99,
        category: "Electronics".to_string(),
        in_stock: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    products.insert(3, Product {
        id: 3,
        name: "Coffee Mug".to_string(),
        description: "Ceramic coffee mug for developers".to_string(),
        price: 19.99,
        category: "Home".to_string(),
        in_stock: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    });
    info!("✅ Sample products created");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Product Service");
    info!("🔥 Zero ports exposed - only RabbitMQ connection!");

    // Create in-memory database
    let db: ProductDatabase = Arc::new(RwLock::new(HashMap::new()));
    setup_sample_data(&db).await;

    // Create microservice - NO PORTS EXPOSED!
    let service = MicroService::new_simple("product-service", "amqp://localhost:5672").await?;

    // Register get_product handler
    let db_clone = db.clone();
    service.register_function("get_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            debug!("Getting product");
            let product_id: u32 = msg.deserialize_payload()?;
            
            let products = db.read().await;
            match products.get(&product_id) {
                Some(product) => {
                    info!("Found product: {}", product.name);
                    Ok(RpcResponse::success(product.clone(), 10)?)
                }
                None => {
                    info!("Product not found: {}", product_id);
                    Ok(RpcResponse::error(format!("Product with ID {} not found", product_id)))
                }
            }
        }
    }).await;

    // Register create_product handler
    let db_clone = db.clone();
    service.register_function("create_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            info!("Creating product");
            let request: CreateProductRequest = msg.deserialize_payload()?;
            
            let mut products = db.write().await;
            
            // Generate new ID
            let id = products.len() as u32 + 1;
            
            let product = Product {
                id,
                name: request.name,
                description: request.description,
                price: request.price,
                category: request.category,
                in_stock: true,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            
            products.insert(id, product.clone());
            info!("Created product: {} (ID: {})", product.name, product.id);
            
            Ok(RpcResponse::success(product, 20)?)
        }
    }).await;

    // Register list_products handler
    let db_clone = db.clone();
    service.register_function("list_products", move |_msg: Message| {
        let db = db_clone.clone();
        async move {
            debug!("Listing all products");
            
            let products = db.read().await;
            let product_list: Vec<Product> = products.values().cloned().collect();
            
            info!("Listed {} products", product_list.len());
            Ok(RpcResponse::success(product_list, 5)?)
        }
    }).await;

    // Register get_products_by_category handler
    let db_clone = db.clone();
    service.register_function("get_products_by_category", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            debug!("Getting products by category");
            let category: String = msg.deserialize_payload()?;
            
            let products = db.read().await;
            let category_products: Vec<Product> = products
                .values()
                .filter(|p| p.category == category)
                .cloned()
                .collect();
            
            info!("Found {} products in category {}", category_products.len(), category);
            Ok(RpcResponse::success(category_products, 15)?)
        }
    }).await;

    // Register ping handler for health checks
    service.register_function("ping", |_msg: Message| async move {
        Ok(RpcResponse::success("Product service is healthy", 1)?)
    }).await;

    info!("✅ Product service handlers registered");
    info!("🎯 Listening to RabbitMQ queue: rabbitmesh.product-service");
    info!("🔗 Available via API Gateway at:");
    info!("   GET  /api/v1/product-service/list_products");
    info!("   GET  /api/v1/product-service/get_product?product_id=1");
    info!("   GET  /api/v1/product-service/get_products_by_category?category=Electronics");
    info!("   POST /api/v1/product-service/create_product");
    
    // Start the service - runs forever, processing RabbitMQ messages
    service.start().await?;
    Ok(())
}