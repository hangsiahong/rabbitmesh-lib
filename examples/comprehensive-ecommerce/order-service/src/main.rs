mod config;
mod database;
mod handler;
mod model;
mod service;
mod utils;

use anyhow::Result;
use rabbitmesh::{MicroService, ServiceConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    config::Settings,
    database::OrderDatabase,
    handler::OrderHandler,
    service::OrderService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "order_service=debug,rabbitmesh=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Order Service...");

    // Load configuration
    let settings = Settings::new()?;
    tracing::info!("Configuration loaded");

    // Initialize database with MongoDB and Redis
    let db = OrderDatabase::new(&settings).await?;
    tracing::info!("Connected to MongoDB and Redis");

    // Initialize handler
    let handler = OrderHandler::new(db);
    
    // Initialize service
    let service = OrderService::new(handler);
    tracing::info!("Order service initialized");

    // Create service config and start microservice using the actual API
    let config = ServiceConfig::new(
        "order-service",
        &settings.rabbitmq.url
    );
    
    let microservice = MicroService::new(config).await?;
    
    // Register service method handlers via the service_impl macro
    OrderService::register_handlers(&microservice).await?;
    
    tracing::info!("Starting microservice on RabbitMQ: {}", settings.rabbitmq.url);
    microservice.start().await?;
    
    Ok(())
}