use anyhow::Result;
use chrono::Utc;
use mongodb::bson::doc;
use serde_json::Value;

use crate::{
    database::OrderDatabase,
    model::{Order, CreateOrderRequest, UpdateOrderRequest, OrderResponse, OrderSummary, OrderStatus},
    utils::{convert_create_items_to_order_items, calculate_order_total, generate_order_id, validate_order_status_transition, is_order_editable},
};

pub struct OrderHandler {
    db: OrderDatabase,
}

impl OrderHandler {
    pub fn new(db: OrderDatabase) -> Self {
        Self { db }
    }

    pub async fn create_order(&mut self, request: CreateOrderRequest) -> Result<OrderResponse> {
        let items = convert_create_items_to_order_items(request.items);
        let total_amount = calculate_order_total(&items);

        let order = Order {
            id: generate_order_id(),
            user_id: request.user_id.clone(),
            items,
            total_amount,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let collection = self.db.orders();
        collection.insert_one(&order).await?;

        // Cache the new order
        let order_json = serde_json::to_string(&order)?;
        self.db.cache_order(&order.id, &order_json).await?;

        // Invalidate user orders cache
        self.db.invalidate_user_orders_cache(&request.user_id).await?;

        Ok(order.into())
    }

    pub async fn get_order(&mut self, order_id: &str) -> Result<Option<OrderResponse>> {
        // Try cache first
        if let Some(cached_order) = self.db.get_cached_order(order_id).await? {
            if let Ok(order) = serde_json::from_str::<Order>(&cached_order) {
                return Ok(Some(order.into()));
            }
        }

        // Fetch from database
        let collection = self.db.orders();
        let filter = doc! { "id": order_id };
        
        if let Some(order) = collection.find_one(filter).await? {
            // Cache the order
            let order_json = serde_json::to_string(&order)?;
            self.db.cache_order(order_id, &order_json).await?;
            
            Ok(Some(order.into()))
        } else {
            Ok(None)
        }
    }

    pub async fn update_order(&mut self, order_id: &str, request: UpdateOrderRequest) -> Result<Option<OrderResponse>> {
        let collection = self.db.orders();
        let filter = doc! { "id": order_id };

        // Get current order to validate updates
        let current_order = collection.find_one(filter.clone()).await?;
        let mut current_order = match current_order {
            Some(order) => order,
            None => return Ok(None),
        };

        // Validate status transition if status is being updated
        if let Some(new_status) = &request.status {
            if !validate_order_status_transition(&current_order.status, new_status) {
                return Err(anyhow::anyhow!(
                    "Invalid status transition from {:?} to {:?}",
                    current_order.status,
                    new_status
                ));
            }
            current_order.status = new_status.clone();
        }

        // Update items if provided and order is editable
        if let Some(new_items) = request.items {
            if !is_order_editable(&current_order.status) {
                return Err(anyhow::anyhow!(
                    "Cannot modify items for order with status {:?}",
                    current_order.status
                ));
            }
            current_order.items = new_items;
            current_order.total_amount = calculate_order_total(&current_order.items);
        }

        current_order.updated_at = Utc::now();

        // Update in database
        let update_doc = doc! {
            "$set": {
                "status": mongodb::bson::to_bson(&current_order.status)?,
                "items": mongodb::bson::to_bson(&current_order.items)?,
                "total_amount": current_order.total_amount,
                "updated_at": mongodb::bson::to_bson(&current_order.updated_at)?
            }
        };

        collection.update_one(filter, update_doc).await?;

        // Invalidate caches
        self.db.invalidate_order_cache(order_id).await?;
        self.db.invalidate_user_orders_cache(&current_order.user_id).await?;

        Ok(Some(current_order.into()))
    }

    pub async fn delete_order(&mut self, order_id: &str) -> Result<bool> {
        let collection = self.db.orders();
        let filter = doc! { "id": order_id };

        // Get order to check if it can be deleted and get user_id
        if let Some(order) = collection.find_one(filter.clone()).await? {
            // Only allow deletion of pending orders
            if !matches!(order.status, OrderStatus::Pending) {
                return Err(anyhow::anyhow!(
                    "Cannot delete order with status {:?}",
                    order.status
                ));
            }

            let result = collection.delete_one(filter).await?;
            let deleted = result.deleted_count > 0;

            if deleted {
                // Invalidate caches
                self.db.invalidate_order_cache(order_id).await?;
                self.db.invalidate_user_orders_cache(&order.user_id).await?;
            }

            Ok(deleted)
        } else {
            Ok(false)
        }
    }

    pub async fn get_user_orders(&mut self, user_id: &str, limit: i64, skip: u64) -> Result<Vec<OrderSummary>> {
        let cache_key = format!("{}:{}:{}", user_id, limit, skip);
        
        // Try cache first
        if let Some(cached_orders) = self.db.get_cached_user_orders(&cache_key).await? {
            if let Ok(orders) = serde_json::from_str::<Vec<OrderSummary>>(&cached_orders) {
                return Ok(orders);
            }
        }

        // Fetch from database
        let collection = self.db.orders();
        let filter = doc! { "user_id": user_id };
        
        let mut cursor = collection
            .find(filter)
            .await?;

        let mut orders = Vec::new();
        while cursor.advance().await? {
            let order = cursor.deserialize_current()?;
            orders.push(order.into());
        }

        // Sort by creation date (newest first)
        orders.sort_by(|a: &Order, b: &Order| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let orders: Vec<OrderSummary> = orders
            .into_iter()
            .skip(skip as usize)
            .take(limit as usize)
            .map(|order| order.into())
            .collect();

        // Cache the result
        let orders_json = serde_json::to_string(&orders)?;
        self.db.cache_user_orders(&cache_key, &orders_json).await?;

        Ok(orders)
    }

    pub async fn get_orders_by_status(&mut self, status: OrderStatus, limit: i64, skip: u64) -> Result<Vec<OrderSummary>> {
        let collection = self.db.orders();
        let filter = doc! { "status": mongodb::bson::to_bson(&status)? };
        
        let mut cursor = collection
            .find(filter)
            .await?;

        let mut orders = Vec::new();
        while cursor.advance().await? {
            let order = cursor.deserialize_current()?;
            orders.push(order.into());
        }

        // Sort by creation date (newest first)
        orders.sort_by(|a: &Order, b: &Order| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let orders: Vec<OrderSummary> = orders
            .into_iter()
            .skip(skip as usize)
            .take(limit as usize)
            .map(|order| order.into())
            .collect();

        Ok(orders)
    }
}