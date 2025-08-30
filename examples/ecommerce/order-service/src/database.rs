use anyhow::Result;
use mongodb::{Client, Collection, Database};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;

use crate::{config::Settings, model::Order};

pub struct OrderDatabase {
    db: Database,
    redis: MultiplexedConnection,
    cache_ttl: u64,
}

impl OrderDatabase {
    pub async fn new(settings: &Settings) -> Result<Self> {
        // MongoDB connection
        let client = Client::with_uri_str(&settings.database.url).await?;
        let db = client.database(&settings.database.name);
        
        // Redis connection
        let redis_client = redis::Client::open(settings.redis.url.clone())?;
        let redis = redis_client.get_multiplexed_async_connection().await?;
        
        Ok(Self { 
            db,
            redis,
            cache_ttl: settings.redis.cache_ttl_seconds,
        })
    }

    pub fn orders(&self) -> Collection<Order> {
        self.db.collection("orders")
    }

    pub async fn get_cached_order(&mut self, order_id: &str) -> Result<Option<String>> {
        let cache_key = format!("order:{}", order_id);
        let result: Option<String> = self.redis.get(&cache_key).await?;
        Ok(result)
    }

    pub async fn cache_order(&mut self, order_id: &str, order_json: &str) -> Result<()> {
        let cache_key = format!("order:{}", order_id);
        let _: () = self.redis.set_ex(&cache_key, order_json, self.cache_ttl).await?;
        Ok(())
    }

    pub async fn invalidate_order_cache(&mut self, order_id: &str) -> Result<()> {
        let cache_key = format!("order:{}", order_id);
        let _: () = self.redis.del(&cache_key).await?;
        Ok(())
    }

    pub async fn get_cached_user_orders(&mut self, user_id: &str) -> Result<Option<String>> {
        let cache_key = format!("user_orders:{}", user_id);
        let result: Option<String> = self.redis.get(&cache_key).await?;
        Ok(result)
    }

    pub async fn cache_user_orders(&mut self, user_id: &str, orders_json: &str) -> Result<()> {
        let cache_key = format!("user_orders:{}", user_id);
        let _: () = self.redis.set_ex(&cache_key, orders_json, self.cache_ttl).await?;
        Ok(())
    }

    pub async fn invalidate_user_orders_cache(&mut self, user_id: &str) -> Result<()> {
        let cache_key = format!("user_orders:{}", user_id);
        let _: () = self.redis.del(&cache_key).await?;
        Ok(())
    }
}