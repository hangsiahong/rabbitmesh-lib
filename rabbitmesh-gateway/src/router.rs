use axum::{
    extract::{Path, Query, State},
    http::{StatusCode},
    response::{Json, Response},
    routing::{get, post, put, delete},
    Router,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

use rabbitmesh::{ServiceClient, RabbitMeshError};
use crate::registry::ServiceRegistry;

/// Gateway state containing the RabbitMQ client and service registry
#[derive(Debug, Clone)]
pub struct GatewayState {
    /// Client for calling microservices via RabbitMQ
    pub service_client: Arc<ServiceClient>,
    /// Registry of available services and their methods
    pub service_registry: Arc<ServiceRegistry>,
}

/// Create the auto-router that handles all REST API calls
/// 
/// ## How it works:
/// 1. Frontend sends HTTP request to gateway
/// 2. Gateway parses the request and extracts service/method  
/// 3. Gateway calls microservice via RabbitMQ (NO PORTS!)
/// 4. Microservice processes request and responds via RabbitMQ
/// 5. Gateway returns HTTP response to frontend
/// 
/// Example flow:
/// ```
/// Frontend -> GET /api/v1/user-service/users/123
///          -> Gateway extracts: service="user-service", method="get_user", params={id: 123}
///          -> Gateway calls service via RabbitMQ
///          -> user-service processes request 
///          -> user-service responds via RabbitMQ
///          -> Gateway returns JSON to frontend
/// ```
pub async fn create_auto_router(amqp_url: impl Into<String>) -> Result<Router, RabbitMeshError> {
    info!("🌐 Creating auto-router gateway");
    
    // Create service client that talks to microservices via RabbitMQ
    let service_client = Arc::new(
        ServiceClient::new("api-gateway", amqp_url).await?
    );
    
    let service_registry = Arc::new(ServiceRegistry::new());
    
    let state = GatewayState {
        service_client,
        service_registry,
    };

    let router = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/health/{service}", get(service_health_check))
        
        // Auto-generated REST API routes
        // Pattern: /api/v1/{service}/{method}/{params...}
        .route("/api/v1/{service}/{method}", post(handle_rpc_call))
        .route("/api/v1/{service}/{method}", get(handle_rpc_call))
        .route("/api/v1/{service}/{method}", put(handle_rpc_call))
        .route("/api/v1/{service}/{method}", delete(handle_rpc_call))
        
        // Parameterized routes: /api/v1/user-service/users/123
        .route("/api/v1/{service}/{method}/{param}", get(handle_rpc_call_with_param))
        .route("/api/v1/{service}/{method}/{param}", post(handle_rpc_call_with_param))
        .route("/api/v1/{service}/{method}/{param}", put(handle_rpc_call_with_param))
        .route("/api/v1/{service}/{method}/{param}", delete(handle_rpc_call_with_param))
        
        // Service registry endpoints
        .route("/registry/services", get(list_services))
        .route("/registry/services/{service}", get(describe_service))
        
        .with_state(state);

    info!("✅ Auto-router created, ready to proxy HTTP -> RabbitMQ -> Microservices");
    Ok(router)
}

/// Handle RPC call without path parameters
/// 
/// Examples:
/// - POST /api/v1/user-service/create_user  
/// - GET /api/v1/product-service/list_products
async fn handle_rpc_call(
    State(state): State<GatewayState>,
    Path((service, method)): Path<(String, String)>,
    Query(query_params): Query<HashMap<String, String>>,
    body: Option<Json<Value>>,
) -> Result<Json<Value>, GatewayError> {
    debug!("🔄 RPC call: {}.{}", service, method);
    
    // Prepare parameters from query params and body
    let params = prepare_params(query_params, body);
    
    // In a real implementation, you'd extract headers here
    let params_with_meta = params;
    
    // Call microservice via RabbitMQ (THIS IS THE MAGIC!)
    let response = state
        .service_client
        .call_with_timeout(&service, &method, params_with_meta, Duration::from_secs(30))
        .await?;
    
    // Convert RPC response to HTTP response
    match response {
        rabbitmesh::message::RpcResponse::Success { data, processing_time_ms } => {
            debug!("✅ RPC success: {}.{} ({}ms)", service, method, processing_time_ms);
            Ok(Json(data))
        }
        rabbitmesh::message::RpcResponse::Error { error, code, details } => {
            warn!("❌ RPC error: {}.{} - {}", service, method, error);
            Err(GatewayError::ServiceError {
                service: service.clone(),
                method: method.clone(),
                error,
                code,
                details,
            })
        }
    }
}

/// Handle RPC call with single path parameter
/// 
/// Examples:
/// - GET /api/v1/user-service/get_user/123 -> get_user(user_id: 123)
/// - DELETE /api/v1/order-service/cancel_order/abc-456 -> cancel_order(order_id: "abc-456")
async fn handle_rpc_call_with_param(
    State(state): State<GatewayState>,
    Path((service, method, param)): Path<(String, String, String)>,
    Query(query_params): Query<HashMap<String, String>>,
    body: Option<Json<Value>>,
) -> Result<Json<Value>, GatewayError> {
    debug!("🔄 RPC call with param: {}.{}({})", service, method, param);
    
    // Add path parameter to query params
    let mut all_params = query_params;
    
    // Try to parse param as different types
    let param_value = if let Ok(num) = param.parse::<i64>() {
        Value::Number(serde_json::Number::from(num))
    } else if let Ok(num) = param.parse::<f64>() {
        Value::Number(serde_json::Number::from_f64(num).unwrap_or_else(|| serde_json::Number::from(0)))
    } else if param == "true" || param == "false" {
        Value::Bool(param == "true")
    } else {
        Value::String(param)
    };
    
    // Common parameter names based on method patterns
    let param_key = match method.as_str() {
        m if m.contains("user") => "user_id",
        m if m.contains("order") => "order_id", 
        m if m.contains("product") => "product_id",
        _ => "id", // Default fallback
    };
    
    all_params.insert(param_key.to_string(), param_value.to_string());
    
    let params = prepare_params(all_params, body);
    let params_with_meta = params;
    
    // Call microservice via RabbitMQ
    let response = state
        .service_client
        .call_with_timeout(&service, &method, params_with_meta, Duration::from_secs(30))
        .await?;
    
    match response {
        rabbitmesh::message::RpcResponse::Success { data, processing_time_ms } => {
            debug!("✅ RPC success: {}.{} ({}ms)", service, method, processing_time_ms);
            Ok(Json(data))
        }
        rabbitmesh::message::RpcResponse::Error { error, code, details } => {
            warn!("❌ RPC error: {}.{} - {}", service, method, error);
            Err(GatewayError::ServiceError {
                service,
                method,
                error,
                code,
                details,
            })
        }
    }
}

/// Prepare parameters from query params and request body
fn prepare_params(
    query_params: HashMap<String, String>,
    body: Option<Json<Value>>,
) -> Value {
    let mut params = serde_json::Map::new();
    
    // Add query parameters
    for (key, value) in query_params {
        // Try to parse value as JSON, fallback to string
        let parsed_value = serde_json::from_str(&value)
            .unwrap_or_else(|_| Value::String(value));
        params.insert(key, parsed_value);
    }
    
    // Add body parameters
    if let Some(Json(body_value)) = body {
        if let Value::Object(body_map) = body_value {
            for (key, value) in body_map {
                params.insert(key, value);
            }
        }
    }
    
    Value::Object(params)
}


/// Health check for the gateway itself
async fn health_check(State(state): State<GatewayState>) -> Result<Json<Value>, GatewayError> {
    let is_healthy = state.service_client.is_healthy().await;
    let stats = state.service_client.get_stats().await;
    
    Ok(Json(serde_json::json!({
        "status": if is_healthy { "healthy" } else { "unhealthy" },
        "gateway": "rabbitmesh-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "connection": stats.connection_stats,
        "rpc": stats.rpc_stats
    })))
}

/// Health check for specific service
async fn service_health_check(
    State(state): State<GatewayState>,
    Path(service): Path<String>,
) -> Result<Json<Value>, GatewayError> {
    // Try to ping the service
    match state.service_client.call_with_timeout(
        &service,
        "ping",
        serde_json::json!({}),
        Duration::from_secs(5)
    ).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "service": service,
            "status": "healthy"
        }))),
        Err(_) => Ok(Json(serde_json::json!({
            "service": service,
            "status": "unhealthy"
        })))
    }
}

/// List all registered services by discovering them via RabbitMQ
async fn list_services(State(state): State<GatewayState>) -> Json<Value> {
    let mut discovered_services = Vec::new();
    
    // Known service names to discover
    let known_services = vec!["auth-service", "todo-service", "notification-service"];
    
    for service_name in known_services {
        // Try to ping each service to see if it's available
        match state.service_client.call_with_timeout(
            service_name,
            "ping",
            serde_json::json!({}),
            Duration::from_secs(2)
        ).await {
            Ok(_) => {
                discovered_services.push(serde_json::json!({
                    "name": service_name,
                    "status": "healthy",
                    "discovered_via": "rabbitmq_ping"
                }));
            }
            Err(_) => {
                // Service not responding, but we can still list it as known
                discovered_services.push(serde_json::json!({
                    "name": service_name,
                    "status": "unreachable",
                    "discovered_via": "rabbitmq_ping"
                }));
            }
        }
    }
    
    Json(serde_json::json!({ 
        "services": discovered_services,
        "discovery_method": "rabbitmq_service_ping",
        "total_discovered": discovered_services.len()
    }))
}

/// Describe a specific service and its methods
async fn describe_service(
    State(state): State<GatewayState>,
    Path(service): Path<String>,
) -> Result<Json<Value>, GatewayError> {
    match state.service_registry.get_service(&service).await {
        Some(service_info) => Ok(Json(serde_json::json!(service_info))),
        None => Err(GatewayError::ServiceNotFound(service)),
    }
}

/// Gateway-specific errors
#[derive(Debug)]
pub enum GatewayError {
    ServiceError {
        service: String,
        method: String,
        error: String,
        code: Option<String>,
        details: Option<Value>,
    },
    ServiceNotFound(String),
    RabbitMeshError(RabbitMeshError),
}

impl From<RabbitMeshError> for GatewayError {
    fn from(err: RabbitMeshError) -> Self {
        Self::RabbitMeshError(err)
    }
}

impl axum::response::IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            GatewayError::ServiceError { service, method, error, code, details } => {
                let status = match code.as_deref() {
                    Some("NOT_FOUND") => StatusCode::NOT_FOUND,
                    Some("UNAUTHORIZED") => StatusCode::UNAUTHORIZED,
                    Some("FORBIDDEN") => StatusCode::FORBIDDEN,
                    Some("BAD_REQUEST") => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                let body = serde_json::json!({
                    "error": error,
                    "service": service,
                    "method": method,
                    "code": code,
                    "details": details
                });
                
                (status, body)
            }
            GatewayError::ServiceNotFound(service) => {
                (
                    StatusCode::NOT_FOUND,
                    serde_json::json!({
                        "error": format!("Service '{}' not found", service),
                        "code": "SERVICE_NOT_FOUND"
                    })
                )
            }
            GatewayError::RabbitMeshError(err) => {
                error!("Gateway error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({
                        "error": "Internal gateway error",
                        "code": "GATEWAY_ERROR"
                    })
                )
            }
        };

        (status, Json(error_message)).into_response()
    }
}