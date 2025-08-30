use rabbitmesh_macros::{
    audit_log, cached, event_publish, metrics, rate_limit, redis_cache,
    require_auth, require_ownership, require_permission,
    service_impl, service_method, transactional, validate
};
use rabbitmesh::Message;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    handler::OrderHandler,
    model::{CreateOrderRequest, UpdateOrderRequest, OrderStatus},
};

pub struct OrderService {
    handler: Arc<Mutex<OrderHandler>>,
}

#[service_impl]
impl OrderService {
    pub fn new(handler: OrderHandler) -> Self {
        Self {
            handler: Arc::new(Mutex::new(handler)),
        }
    }

    #[service_method("POST /orders")]
    #[require_auth]
    #[require_permission("orders:write")]
    #[validate]
    #[rate_limit(20, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    #[event_publish]
    pub async fn create_order(msg: Message) -> Result<Value, String> {
        let request: CreateOrderRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        // TODO: Get handler from service registry or global state
        // For now, implement the business logic directly
        let order_id = uuid::Uuid::new_v4().to_string();
        
        // Convert CreateOrderItem to OrderItem and calculate totals
        let mut total_amount = 0.0;
        let items: Vec<crate::model::OrderItem> = request.items.into_iter().map(|item| {
            let total_price = item.unit_price * item.quantity as f64;
            total_amount += total_price;
            crate::model::OrderItem {
                product_id: item.product_id,
                product_name: item.product_name,
                quantity: item.quantity,
                unit_price: item.unit_price,
                total_price,
            }
        }).collect();
        
        let order = crate::model::Order {
            id: order_id,
            user_id: request.user_id.clone(),
            items,
            total_amount,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        tracing::info!("Created order: {} for user: {}", order.id, request.user_id);
        Ok(serde_json::to_value(order).unwrap())
    }

    #[service_method("GET /orders/:id")]
    #[require_auth]
    #[require_permission("orders:read")]
    #[redis_cache(300)]
    #[rate_limit(100, 60)]
    #[metrics]
    pub async fn get_order(msg: Message) -> Result<Value, String> {
        let order_id = msg.metadata.get("order_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("Order ID not found in request")?;

        tracing::info!("Getting order: {}", order_id);
        
        // TODO: Query from real database
        // For now, return constructed response based on the ID
        let order = crate::model::Order {
            id: order_id.clone(),
            user_id: "user-123".to_string(),
            items: vec![],
            total_amount: 99.99,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(order).unwrap())
    }

    #[service_method("PUT /orders/:id")]
    #[require_auth]
    #[require_permission("orders:write")]
    #[validate]
    #[rate_limit(30, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    #[event_publish]
    pub async fn update_order(msg: Message) -> Result<Value, String> {
        let order_id = msg.metadata.get("order_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("Order ID not found in request")?;

        let request: UpdateOrderRequest = msg.deserialize_payload()
            .map_err(|e| format!("Invalid request format: {}", e))?;

        tracing::info!("Updating order: {}", order_id);

        // TODO: Update in real database
        // For now, return updated order
        let order = crate::model::Order {
            id: order_id.clone(),
            user_id: "user-123".to_string(),
            items: request.items.unwrap_or_else(|| vec![]),
            total_amount: 149.99,
            status: request.status.unwrap_or(OrderStatus::Processing),
            created_at: chrono::Utc::now() - chrono::Duration::hours(1),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(order).unwrap())
    }

    #[service_method("DELETE /orders/:id")]
    #[require_auth]
    #[require_permission("orders:delete")]
    #[rate_limit(10, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    pub async fn delete_order(msg: Message) -> Result<Value, String> {
        let order_id = msg.metadata.get("order_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("Order ID not found in request")?;

        tracing::info!("Deleting order: {}", order_id);

        // TODO: Delete from real database
        // For now, always return success
        Ok(serde_json::json!({
            "message": "Order deleted successfully",
            "order_id": order_id,
            "deleted_at": chrono::Utc::now().to_rfc3339()
        }))
    }

    #[service_method("GET /orders/user/:user_id")]
    #[require_auth]
    #[require_permission("orders:read")]
    #[require_ownership(resource = "user")]
    #[redis_cache(60)]
    #[rate_limit(50, 60)]
    #[metrics]
    pub async fn get_user_orders(msg: Message) -> Result<Value, String> {
        let user_id = msg.metadata.get("user_id")
            .ok_or("User ID not found in request")?;

        let limit = msg.metadata.get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(20);
        
        let skip = msg.metadata.get("skip")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        tracing::info!("Getting orders for user: {} with limit: {}, skip: {}", user_id, limit, skip);

        // TODO: Query from real database
        // For now, return sample orders for the user
        let orders = vec![
            crate::model::Order {
                id: "order-1".to_string(),
                user_id: user_id.clone(),
                items: vec![],
                total_amount: 59.99,
                status: OrderStatus::Delivered,
                created_at: chrono::Utc::now() - chrono::Duration::days(5),
                updated_at: chrono::Utc::now() - chrono::Duration::days(3),
            },
            crate::model::Order {
                id: "order-2".to_string(),
                user_id: user_id.clone(),
                items: vec![],
                total_amount: 129.99,
                status: OrderStatus::Processing,
                created_at: chrono::Utc::now() - chrono::Duration::days(2),
                updated_at: chrono::Utc::now(),
            }
        ];

        Ok(serde_json::to_value(orders).unwrap())
    }

    #[service_method("GET /orders/status/:status")]
    #[require_auth]
    #[require_permission("orders:read")]
    #[cached(120)]
    #[rate_limit(100, 60)]
    #[metrics]
    pub async fn get_orders_by_status(msg: Message) -> Result<Value, String> {
        let status_str = msg.metadata.get("status")
            .ok_or("Status not provided in request")?;

        let status = match status_str.to_lowercase().as_str() {
            "pending" => OrderStatus::Pending,
            "confirmed" => OrderStatus::Confirmed,
            "processing" => OrderStatus::Processing,
            "shipped" => OrderStatus::Shipped,
            "delivered" => OrderStatus::Delivered,
            "cancelled" => OrderStatus::Cancelled,
            _ => return Err("Invalid order status".to_string()),
        };

        let limit = msg.metadata.get("limit")
            .and_then(|l| l.parse::<i64>().ok())
            .unwrap_or(50);
        
        let skip = msg.metadata.get("skip")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        tracing::info!("Getting orders with status: {:?}, limit: {}, skip: {}", status, limit, skip);

        // TODO: Query from real database
        // For now, return sample orders with the requested status
        let orders = vec![
            crate::model::Order {
                id: "order-status-1".to_string(),
                user_id: "user-456".to_string(),
                items: vec![],
                total_amount: 79.99,
                status: status.clone(),
                created_at: chrono::Utc::now() - chrono::Duration::days(1),
                updated_at: chrono::Utc::now(),
            }
        ];

        Ok(serde_json::to_value(orders).unwrap())
    }

    #[service_method("POST /orders/:id/confirm")]
    #[require_auth]
    #[require_permission("orders:write")]
    #[rate_limit(50, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    #[event_publish]
    pub async fn confirm_order(msg: Message) -> Result<Value, String> {
        let order_id = msg.metadata.get("order_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("Order ID not found in request")?;

        tracing::info!("Confirming order: {}", order_id);

        // TODO: Update in real database
        // For now, return confirmed order
        let order = crate::model::Order {
            id: order_id.clone(),
            user_id: "user-123".to_string(),
            items: vec![],
            total_amount: 149.99,
            status: OrderStatus::Confirmed,
            created_at: chrono::Utc::now() - chrono::Duration::hours(2),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(order).unwrap())
    }

    #[service_method("POST /orders/:id/cancel")]
    #[require_auth]
    #[require_permission("orders:write")]
    #[rate_limit(20, 60)]
    #[transactional]
    #[metrics]
    #[audit_log]
    #[event_publish]
    pub async fn cancel_order(msg: Message) -> Result<Value, String> {
        let order_id = msg.metadata.get("order_id")
            .or_else(|| msg.metadata.get("id"))
            .ok_or("Order ID not found in request")?;

        tracing::info!("Cancelling order: {}", order_id);

        // TODO: Update in real database
        // For now, return cancelled order
        let order = crate::model::Order {
            id: order_id.clone(),
            user_id: "user-123".to_string(),
            items: vec![],
            total_amount: 149.99,
            status: OrderStatus::Cancelled,
            created_at: chrono::Utc::now() - chrono::Duration::hours(2),
            updated_at: chrono::Utc::now(),
        };

        Ok(serde_json::to_value(order).unwrap())
    }
}