mod config;
mod handler;
mod model;
mod service;
mod utils;

use anyhow::Result;
use rabbitmesh::{MicroService, ServiceConfig, ServiceClient};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

use crate::{
    config::Settings,
    handler::AuthHandler,
    service::AuthService,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=debug,rabbitmesh=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Auth Service...");

    // Load configuration
    let settings = Settings::new()?;
    let rabbitmq_url = settings.rabbitmq.url.clone();
    tracing::info!("Configuration loaded");

    // Initialize service client for inter-service communication
    let service_client = ServiceClient::new("auth-service", &rabbitmq_url).await?;
    tracing::info!("Connected to RabbitMQ for service communication");

    // Initialize handler
    let handler = AuthHandler::new(settings, service_client);
    
    // Initialize service
    let service = AuthService::new(handler);
    tracing::info!("Auth service initialized");

    // Create service config and start microservice using the actual API
    let config = ServiceConfig::new(
        "auth-service",
        &rabbitmq_url
    );
    
    let microservice = MicroService::new(config).await?;
    
    // Register service method handlers via the service_impl macro
    AuthService::register_handlers(&microservice).await?;
    
    tracing::info!("Starting microservice on RabbitMQ: {}", rabbitmq_url);
    microservice.start().await?;
    
    Ok(())
}