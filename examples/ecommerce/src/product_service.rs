use rabbitmesh::{MicroService, RpcResponse, Message};
use rabbitmesh_macros::{service_definition, service_method};
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

/// Request to update product
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub in_stock: Option<bool>,
}

/// Product search filters
#[derive(Debug, Deserialize)]
pub struct ProductFilters {
    pub category: Option<String>,
    pub in_stock: Option<bool>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

/// In-memory product database
type ProductDatabase = Arc<RwLock<HashMap<u32, Product>>>;

/// Product microservice definition  
#[service_definition]
pub struct ProductService;

impl ProductService {
    /// Get product by ID
    #[service_method("GET /products/:id")]
    pub async fn get_product(product_id: u32, db: ProductDatabase) -> Result<Product, String> {
        debug!("Getting product with ID: {}", product_id);
        
        let products = db.read().await;
        match products.get(&product_id) {
            Some(product) => {
                info!("Found product: {}", product.name);
                Ok(product.clone())
            }
            None => {
                info!("Product not found: {}", product_id);
                Err(format!("Product with ID {} not found", product_id))
            }
        }
    }

    /// Create a new product
    #[service_method("POST /products")]
    pub async fn create_product(data: CreateProductRequest, db: ProductDatabase) -> Result<Product, String> {
        info!("Creating product: {}", data.name);
        
        let mut products = db.write().await;
        
        // Generate new ID
        let id = products.len() as u32 + 1;
        
        let product = Product {
            id,
            name: data.name,
            description: data.description,
            price: data.price,
            category: data.category,
            in_stock: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        products.insert(id, product.clone());
        info!("Created product: {} (ID: {})", product.name, product.id);
        
        Ok(product)
    }

    /// Update product
    #[service_method("PUT /products/:id")]
    pub async fn update_product(product_id: u32, data: UpdateProductRequest, db: ProductDatabase) -> Result<Product, String> {
        info!("Updating product: {}", product_id);
        
        let mut products = db.write().await;
        
        match products.get_mut(&product_id) {
            Some(product) => {
                if let Some(name) = data.name {
                    product.name = name;
                }
                if let Some(description) = data.description {
                    product.description = description;
                }
                if let Some(price) = data.price {
                    product.price = price;
                }
                if let Some(category) = data.category {
                    product.category = category;
                }
                if let Some(in_stock) = data.in_stock {
                    product.in_stock = in_stock;
                }
                
                info!("Updated product: {}", product.name);
                Ok(product.clone())
            }
            None => Err(format!("Product with ID {} not found", product_id))
        }
    }

    /// Delete product
    #[service_method("DELETE /products/:id")]
    pub async fn delete_product(product_id: u32, db: ProductDatabase) -> Result<String, String> {
        info!("Deleting product: {}", product_id);
        
        let mut products = db.write().await;
        
        match products.remove(&product_id) {
            Some(product) => {
                info!("Deleted product: {}", product.name);
                Ok(format!("Product {} deleted successfully", product.name))
            }
            None => Err(format!("Product with ID {} not found", product_id))
        }
    }

    /// List products with optional filters
    #[service_method("GET /products")]
    pub async fn list_products(filters: Option<ProductFilters>, db: ProductDatabase) -> Result<Vec<Product>, String> {
        debug!("Listing products with filters: {:?}", filters);
        
        let products = db.read().await;
        let mut product_list: Vec<Product> = products.values().cloned().collect();
        
        // Apply filters if provided
        if let Some(filters) = filters {
            if let Some(category) = filters.category {
                product_list.retain(|p| p.category == category);
            }
            
            if let Some(in_stock) = filters.in_stock {
                product_list.retain(|p| p.in_stock == in_stock);
            }
            
            if let Some(min_price) = filters.min_price {
                product_list.retain(|p| p.price >= min_price);
            }
            
            if let Some(max_price) = filters.max_price {
                product_list.retain(|p| p.price <= max_price);
            }
        }
        
        info!("Listed {} products", product_list.len());
        Ok(product_list)
    }

    /// Get products by category
    #[service_method("GET /products/category/:category")]
    pub async fn get_products_by_category(category: String, db: ProductDatabase) -> Result<Vec<Product>, String> {
        debug!("Getting products by category: {}", category);
        
        let products = db.read().await;
        let category_products: Vec<Product> = products
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect();
        
        info!("Found {} products in category {}", category_products.len(), category);
        Ok(category_products)
    }

    /// Health check endpoint
    #[service_method]
    pub async fn ping() -> Result<String, String> {
        Ok("Product service is healthy".to_string())
    }
}

/// Run the product service
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting Product Service");

    // Create in-memory database
    let db: ProductDatabase = Arc::new(RwLock::new(HashMap::new()));
    
    // Add some sample data
    {
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
    }

    // Create microservice
    let service = ProductService::create_service("amqp://localhost:5672").await?;
    
    // Register handlers
    let db_clone = db.clone();
    service.register_function("get_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let product_id: u32 = msg.deserialize_payload()?;
            match ProductService::get_product(product_id, db).await {
                Ok(product) => Ok(RpcResponse::success(product, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("create_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let request: CreateProductRequest = msg.deserialize_payload()?;
            match ProductService::create_product(request, db).await {
                Ok(product) => Ok(RpcResponse::success(product, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("update_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            #[derive(Deserialize)]
            struct UpdateParams {
                product_id: u32,
                #[serde(flatten)]
                data: UpdateProductRequest,
            }
            
            let params: UpdateParams = msg.deserialize_payload()?;
            match ProductService::update_product(params.product_id, params.data, db).await {
                Ok(product) => Ok(RpcResponse::success(product, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("delete_product", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let product_id: u32 = msg.deserialize_payload()?;
            match ProductService::delete_product(product_id, db).await {
                Ok(result) => Ok(RpcResponse::success(result, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("list_products", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let filters: Option<ProductFilters> = msg.deserialize_payload().ok();
            match ProductService::list_products(filters, db).await {
                Ok(products) => Ok(RpcResponse::success(products, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("get_products_by_category", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let category: String = msg.deserialize_payload()?;
            match ProductService::get_products_by_category(category, db).await {
                Ok(products) => Ok(RpcResponse::success(products, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    service.register_function("ping", |_msg: Message| async move {
        match ProductService::ping().await {
            Ok(result) => Ok(RpcResponse::success(result, 0)?),
            Err(e) => Ok(RpcResponse::error(e)),
        }
    }).await;

    info!("✅ Product Service registered all handlers");
    
    // Start the service (this will run forever)
    service.start().await?;
    
    Ok(())
}