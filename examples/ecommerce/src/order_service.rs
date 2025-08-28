use rabbitmesh::{MicroService, RpcResponse, Message, ServiceClient};
use rabbitmesh_macros::{service_definition, service_method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, debug, warn};

/// Order data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Order item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: u32,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

/// Request to create a new order
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: u32,
    pub items: Vec<CreateOrderItem>,
}

/// Order item in create request
#[derive(Debug, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: u32,
    pub quantity: u32,
}

/// Request to update order status
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
}

/// User info from user service
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub name: String,
}

/// Product info from product service
#[derive(Debug, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub in_stock: bool,
}

/// In-memory order database
type OrderDatabase = Arc<RwLock<HashMap<u32, Order>>>;

/// Order microservice definition
#[service_definition]
pub struct OrderService;

impl OrderService {
    /// Get order by ID
    #[service_method("GET /orders/:id")]
    pub async fn get_order(order_id: u32, db: OrderDatabase) -> Result<Order, String> {
        debug!("Getting order with ID: {}", order_id);
        
        let orders = db.read().await;
        match orders.get(&order_id) {
            Some(order) => {
                info!("Found order: {} (User: {})", order.id, order.user_id);
                Ok(order.clone())
            }
            None => {
                info!("Order not found: {}", order_id);
                Err(format!("Order with ID {} not found", order_id))
            }
        }
    }

    /// Create a new order (calls user service and product service)
    #[service_method("POST /orders")]
    pub async fn create_order(
        data: CreateOrderRequest, 
        db: OrderDatabase,
        client: Arc<ServiceClient>
    ) -> Result<Order, String> {
        info!("Creating order for user: {}", data.user_id);
        
        // Verify user exists by calling user service
        let user_response = client
            .call("user-service", "get_user", data.user_id)
            .await
            .map_err(|e| format!("Failed to verify user: {}", e))?;
            
        let _user: User = user_response.data()
            .map_err(|e| format!("User not found: {}", e))?;
        
        // Process order items and verify products
        let mut order_items = Vec::new();
        let mut total_amount = 0.0;
        
        for create_item in data.items {
            // Get product details
            let product_response = client
                .call("product-service", "get_product", create_item.product_id)
                .await
                .map_err(|e| format!("Failed to get product {}: {}", create_item.product_id, e))?;
                
            let product: Product = product_response.data()
                .map_err(|e| format!("Product {} not found: {}", create_item.product_id, e))?;
            
            // Check if product is in stock
            if !product.in_stock {
                return Err(format!("Product {} is out of stock", product.name));
            }
            
            let item_total = product.price * create_item.quantity as f64;
            total_amount += item_total;
            
            order_items.push(OrderItem {
                product_id: create_item.product_id,
                quantity: create_item.quantity,
                unit_price: product.price,
                total_price: item_total,
            });
        }
        
        // Create order
        let mut orders = db.write().await;
        let id = orders.len() as u32 + 1;
        
        let order = Order {
            id,
            user_id: data.user_id,
            items: order_items,
            total_amount,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        orders.insert(id, order.clone());
        info!("Created order: {} (Total: ${:.2})", order.id, order.total_amount);
        
        Ok(order)
    }

    /// Update order status
    #[service_method("PUT /orders/:id/status")]
    pub async fn update_order_status(
        order_id: u32, 
        data: UpdateOrderStatusRequest, 
        db: OrderDatabase
    ) -> Result<Order, String> {
        info!("Updating order {} status to {:?}", order_id, data.status);
        
        let mut orders = db.write().await;
        
        match orders.get_mut(&order_id) {
            Some(order) => {
                order.status = data.status;
                order.updated_at = chrono::Utc::now().to_rfc3339();
                
                info!("Updated order {} status", order.id);
                Ok(order.clone())
            }
            None => Err(format!("Order with ID {} not found", order_id))
        }
    }

    /// Cancel order
    #[service_method("DELETE /orders/:id")]
    pub async fn cancel_order(order_id: u32, db: OrderDatabase) -> Result<String, String> {
        info!("Cancelling order: {}", order_id);
        
        let mut orders = db.write().await;
        
        match orders.get_mut(&order_id) {
            Some(order) => {
                match order.status {
                    OrderStatus::Shipped | OrderStatus::Delivered => {
                        return Err("Cannot cancel shipped or delivered order".to_string());
                    }
                    _ => {
                        order.status = OrderStatus::Cancelled;
                        order.updated_at = chrono::Utc::now().to_rfc3339();
                        
                        info!("Cancelled order: {}", order.id);
                        Ok(format!("Order {} cancelled successfully", order.id))
                    }
                }
            }
            None => Err(format!("Order with ID {} not found", order_id))
        }
    }

    /// List orders for a user
    #[service_method("GET /users/:user_id/orders")]
    pub async fn list_user_orders(user_id: u32, db: OrderDatabase) -> Result<Vec<Order>, String> {
        debug!("Listing orders for user: {}", user_id);
        
        let orders = db.read().await;
        let user_orders: Vec<Order> = orders
            .values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        
        info!("Found {} orders for user {}", user_orders.len(), user_id);
        Ok(user_orders)
    }

    /// List all orders (admin endpoint)
    #[service_method("GET /orders")]
    pub async fn list_orders(db: OrderDatabase) -> Result<Vec<Order>, String> {
        debug!("Listing all orders");
        
        let orders = db.read().await;
        let order_list: Vec<Order> = orders.values().cloned().collect();
        
        info!("Listed {} orders", order_list.len());
        Ok(order_list)
    }

    /// Health check endpoint
    #[service_method]
    pub async fn ping() -> Result<String, String> {
        Ok("Order service is healthy".to_string())
    }
}

/// Run the order service
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting Order Service");

    // Create in-memory database
    let db: OrderDatabase = Arc::new(RwLock::new(HashMap::new()));
    
    // Create client for calling other services
    let client = Arc::new(ServiceClient::new("order-service-client", "amqp://localhost:5672").await?);
    
    // Create microservice
    let service = OrderService::create_service("amqp://localhost:5672").await?;
    
    // Register handlers
    let db_clone = db.clone();
    service.register_function("get_order", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let order_id: u32 = msg.deserialize_payload()?;
            match OrderService::get_order(order_id, db).await {
                Ok(order) => Ok(RpcResponse::success(order, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    let client_clone = client.clone();
    service.register_function("create_order", move |msg: Message| {
        let db = db_clone.clone();
        let client = client_clone.clone();
        async move {
            let request: CreateOrderRequest = msg.deserialize_payload()?;
            match OrderService::create_order(request, db, client).await {
                Ok(order) => Ok(RpcResponse::success(order, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("update_order_status", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            #[derive(Deserialize)]
            struct UpdateParams {
                order_id: u32,
                #[serde(flatten)]
                data: UpdateOrderStatusRequest,
            }
            
            let params: UpdateParams = msg.deserialize_payload()?;
            match OrderService::update_order_status(params.order_id, params.data, db).await {
                Ok(order) => Ok(RpcResponse::success(order, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("cancel_order", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let order_id: u32 = msg.deserialize_payload()?;
            match OrderService::cancel_order(order_id, db).await {
                Ok(result) => Ok(RpcResponse::success(result, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("list_user_orders", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            let user_id: u32 = msg.deserialize_payload()?;
            match OrderService::list_user_orders(user_id, db).await {
                Ok(orders) => Ok(RpcResponse::success(orders, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    let db_clone = db.clone();
    service.register_function("list_orders", move |msg: Message| {
        let db = db_clone.clone();
        async move {
            match OrderService::list_orders(db).await {
                Ok(orders) => Ok(RpcResponse::success(orders, 0)?),
                Err(e) => Ok(RpcResponse::error(e)),
            }
        }
    }).await;

    service.register_function("ping", |_msg: Message| async move {
        match OrderService::ping().await {
            Ok(result) => Ok(RpcResponse::success(result, 0)?),
            Err(e) => Ok(RpcResponse::error(e)),
        }
    }).await;

    info!("✅ Order Service registered all handlers");
    
    // Start the service (this will run forever)
    service.start().await?;
    
    Ok(())
}