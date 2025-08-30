mod config;

use anyhow::Result;
use axum::{
    http::{header, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use rabbitmesh_gateway::create_auto_router;
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gateway=debug,rabbitmesh_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting RabbitMesh Gateway...");

    // Load configuration
    let settings = Settings::new()?;
    tracing::info!("Configuration loaded");

    // Create auto-generated router from service definitions
    let auto_router = create_auto_router(&settings.rabbitmq.url).await?;
    tracing::info!("Auto-generated router created from service definitions");


    // Combine auto-generated and custom routes
    let app = Router::new()
        .merge(auto_router)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                        .expose_headers([header::CONTENT_TYPE])
                )
        );

    tracing::info!("Gateway configured with the following features:");
    tracing::info!("  ✅ Auto-generated REST APIs from service definitions");
    tracing::info!("  ✅ Auto-generated GraphQL endpoint at /graphql");
    tracing::info!("  ✅ Service-to-service communication via RabbitMQ");
    tracing::info!("  ✅ Built-in authentication and authorization");
    tracing::info!("  ✅ Rate limiting and caching");
    tracing::info!("  ✅ Request/response validation");
    tracing::info!("  ✅ Metrics and audit logging");
    tracing::info!("  ✅ CORS support for web applications");

    // Start the server
    let listener = tokio::net::TcpListener::bind(&settings.server.bind_address).await?;
    
    tracing::info!("🚀 Gateway started on {}", settings.server.bind_address);
    tracing::info!("📋 API Documentation: http://{}/api-docs", settings.server.bind_address);
    tracing::info!("🔍 Health Check: http://{}/health", settings.server.bind_address);
    tracing::info!("📊 GraphQL Playground: http://{}/graphql", settings.server.bind_address);
    tracing::info!("🎯 Available Services: http://{}/services", settings.server.bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
