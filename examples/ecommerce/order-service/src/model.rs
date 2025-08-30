use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: String,
    pub items: Vec<CreateOrderItem>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<OrderStatus>,
    pub items: Option<Vec<OrderItem>>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderSummary {
    pub id: String,
    pub user_id: String,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub item_count: usize,
    pub created_at: DateTime<Utc>,
}

impl From<Order> for OrderResponse {
    fn from(order: Order) -> Self {
        Self {
            id: order.id,
            user_id: order.user_id,
            items: order.items,
            total_amount: order.total_amount,
            status: order.status,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

impl From<Order> for OrderSummary {
    fn from(order: Order) -> Self {
        Self {
            id: order.id,
            user_id: order.user_id,
            total_amount: order.total_amount,
            status: order.status,
            item_count: order.items.len(),
            created_at: order.created_at,
        }
    }
}