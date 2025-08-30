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

use crate::{
    config::Settings,
    database::UserDatabase,
    handler::UserHandler,
    service::UserService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "user_service=debug,rabbitmesh=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting User Service...");

    // Load configuration
    let settings = Settings::new()?;
    tracing::info!("Configuration loaded");

    // Initialize database
    let db = UserDatabase::new(&settings).await?;
    tracing::info!("Connected to MongoDB");

    // Initialize handler
    let handler = UserHandler::new(db);
    
    // Initialize service
    let service = UserService::new(handler);
    tracing::info!("User service initialized");

    // Create service config and start microservice using the actual API
    let config = ServiceConfig::new(
        "user-service",
        &settings.rabbitmq.url
    );
    
    let microservice = MicroService::new(config).await?;
    
    // TODO: Register service method handlers here
    // The handlers will be registered via the service_method macros
    
    tracing::info!("Starting microservice on RabbitMQ: {}", settings.rabbitmq.url);
    microservice.start().await?;
    
    Ok(())
}