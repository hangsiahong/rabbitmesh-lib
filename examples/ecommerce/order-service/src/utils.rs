use crate::model::{CreateOrderItem, OrderItem, OrderStatus};
use uuid::Uuid;

pub fn calculate_order_total(items: &[OrderItem]) -> f64 {
    items.iter().map(|item| item.total_price).sum()
}

pub fn convert_create_items_to_order_items(create_items: Vec<CreateOrderItem>) -> Vec<OrderItem> {
    create_items
        .into_iter()
        .map(|item| {
            let total_price = item.unit_price * item.quantity as f64;
            OrderItem {
                product_id: item.product_id,
                product_name: item.product_name,
                quantity: item.quantity,
                unit_price: item.unit_price,
                total_price,
            }
        })
        .collect()
}

pub fn generate_order_id() -> String {
    format!("ORD-{}", Uuid::new_v4().to_string().replace('-', "").to_uppercase()[..8].to_string())
}

pub fn validate_order_status_transition(from: &OrderStatus, to: &OrderStatus) -> bool {
    use OrderStatus::*;
    
    match (from, to) {
        // From Pending
        (Pending, Confirmed) => true,
        (Pending, Cancelled) => true,
        
        // From Confirmed
        (Confirmed, Processing) => true,
        (Confirmed, Cancelled) => true,
        
        // From Processing
        (Processing, Shipped) => true,
        (Processing, Cancelled) => true,
        
        // From Shipped
        (Shipped, Delivered) => true,
        
        // Can't transition from terminal states
        (Delivered, _) => false,
        (Cancelled, _) => false,
        
        // Any other transitions are invalid
        _ => false,
    }
}

pub fn is_order_editable(status: &OrderStatus) -> bool {
    matches!(status, OrderStatus::Pending)
}

pub fn calculate_order_metrics(items: &[OrderItem]) -> OrderMetrics {
    let total_quantity: u32 = items.iter().map(|item| item.quantity).sum();
    let total_amount = calculate_order_total(items);
    let unique_products = items.len();
    let average_item_price = if unique_products > 0 {
        total_amount / unique_products as f64
    } else {
        0.0
    };

    OrderMetrics {
        total_quantity,
        total_amount,
        unique_products,
        average_item_price,
    }
}

#[derive(Debug)]
pub struct OrderMetrics {
    pub total_quantity: u32,
    pub total_amount: f64,
    pub unique_products: usize,
    pub average_item_price: f64,
}