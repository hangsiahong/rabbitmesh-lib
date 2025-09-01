use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, Method, HeaderMap},
    response::{Json, Response},
    routing::{get, post, put, delete, patch},
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
        
        // UNIVERSAL DYNAMIC ROUTER - Works with ANY service architecture!
        // Single catch-all route that handles ALL services dynamically
        .route("/api/v1/{*path}", get(universal_handler))
        .route("/api/v1/{*path}", post(universal_handler))
        .route("/api/v1/{*path}", put(universal_handler))
        .route("/api/v1/{*path}", delete(universal_handler))
        .route("/api/v1/{*path}", patch(universal_handler))
        
        // Service registry endpoints
        .route("/registry/services", get(list_services))
        .route("/registry/services/{service}", get(describe_service))
        
        // Client generator compatible endpoint
        .route("/api/services", get(list_services_for_client_generator))
        
        .with_state(state);

    info!("✅ Auto-router created, ready to proxy HTTP -> RabbitMQ -> Microservices");
    Ok(router)
}

/// UNIVERSAL HANDLER - The single handler that works with ANY service architecture!
/// 
/// This is the heart of the universal dynamic router. It:
/// 1. Extracts HTTP method from request
/// 2. Discovers all services dynamically from RabbitMQ
/// 3. Finds the matching service endpoint for ANY path pattern
/// 4. Calls the correct microservice method via RabbitMQ
/// 5. Returns the response
/// 
/// Works with ANY service patterns:
/// - POST /api/v1/auth-service/auth/login -> auth-service.login
/// - GET /api/v1/user-service/users -> user-service.list_users  
/// - POST /api/v1/order-service/orders -> order-service.create_order
/// - GET /api/v1/user-service/users/123 -> user-service.get_user(id: 123)
/// - ANY other pattern your project uses
async fn universal_handler(
    method: Method,
    State(state): State<GatewayState>,
    Path(path): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Option<Json<Value>>,
) -> Result<Json<Value>, GatewayError> {
    let full_path = format!("/api/v1/{}", path);
    let http_method = method.as_str();
    
    debug!("🚀 Universal handler: {} {}", http_method, full_path);
    
    // The magic happens here - dynamically route to the correct service with JWT forwarding
    match discover_and_route(&state, http_method, &full_path, query_params, headers, body).await {
        Ok(response) => Ok(Json(response)),
        Err(error) => {
            warn!("❌ Universal routing failed for {} {}: {:?}", http_method, full_path, error);
            Err(error)
        }
    }
}


/// Discovered service information for dynamic routing
#[derive(Debug, Clone)]
struct DiscoveredService {
    name: String,
    endpoints: Vec<ServiceEndpoint>,
}

/// Service endpoint information
#[derive(Debug, Clone)]  
struct ServiceEndpoint {
    handler: String,
    http_method: String,
    path_pattern: String,
    parameters: Vec<String>,
}

/// Core dynamic routing logic - the heart of the universal router
/// 
/// This function:
/// 1. Queries RabbitMQ to discover all available services
/// 2. Finds the service and method that matches the incoming request
/// 3. Maps the HTTP path to the correct RabbitMQ method call
/// 4. Handles parameters extraction for any path pattern
async fn discover_and_route(
    state: &GatewayState,
    http_method: &str,
    request_path: &str,
    query_params: HashMap<String, String>,
    headers: HeaderMap,
    body: Option<Json<Value>>,
) -> Result<Value, GatewayError> {
    
    // Step 1: Discover all available services dynamically
    let services = match discover_all_services(state).await {
        Ok(services) => services,
        Err(e) => {
            error!("Failed to discover services: {:?}", e);
            return Err(GatewayError::ServiceDiscoveryError { 
                error: "Failed to discover services".to_string() 
            });
        }
    };
    
    debug!("🔍 Discovered {} services for routing", services.len());
    
    // Step 2: Find matching service method for this request
    for service in services {
        if let Some(endpoint) = find_matching_endpoint(&service, http_method, request_path) {
            debug!("✅ Found match: {} {} -> {}.{}", http_method, request_path, service.name, endpoint.handler);
            
            // Step 3: Extract parameters from the path
            let path_params = extract_path_parameters(&endpoint.path_pattern, request_path);
            
            // Step 4: Prepare the complete parameter set
            let mut params = prepare_params(query_params, body);
            
            // Add path parameters to the request payload AND as special metadata fields
            for (key, value) in path_params.clone() {
                params.as_object_mut().unwrap().insert(key, serde_json::Value::String(value));
            }
            
            // Step 4.5: *** CRITICAL *** - Forward JWT authentication headers AND path params to microservice
            let params_with_auth = prepare_params_with_auth_headers_and_metadata(params, &headers, path_params);
            
            // Step 5: Call the service via RabbitMQ WITH JWT forwarding
            let response = state
                .service_client
                .call_with_timeout(&service.name, &endpoint.handler, params_with_auth, Duration::from_secs(30))
                .await?;
            
            // Step 6: Return the response
            match response {
                rabbitmesh::message::RpcResponse::Success { data, processing_time_ms } => {
                    debug!("✅ Dynamic route success: {}.{} ({}ms)", service.name, endpoint.handler, processing_time_ms);
                    return Ok(data);
                }
                rabbitmesh::message::RpcResponse::Error { error, code, details } => {
                    warn!("❌ Dynamic route error: {}.{} - {}", service.name, endpoint.handler, error);
                    return Err(GatewayError::ServiceError {
                        service: service.name.clone(),
                        method: endpoint.handler.clone(),
                        error,
                        code,
                        details,
                    });
                }
            }
        }
    }
    
    // No matching service found
    Err(GatewayError::NotFound { 
        path: request_path.to_string(),
        method: http_method.to_string() 
    })
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

/// *** CRITICAL FUNCTION *** - Forward HTTP auth headers AND path params to RabbitMQ microservices
/// 
/// This function handles two critical forwarding tasks:
/// 1. Forward JWT tokens from HTTP headers to microservice payload 
/// 2. Forward path parameters to both payload AND special metadata fields
fn prepare_params_with_auth_headers_and_metadata(mut params: Value, headers: &HeaderMap, path_params: HashMap<String, String>) -> Value {
    // Extract JWT token from Authorization header and forward it correctly
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_value) = auth_header.to_str() {
            // The JWT macro checks multiple sources for tokens:
            // 1. msg.metadata.get("authorization") - not accessible via current ServiceClient API
            // 2. msg.payload token fields - this is what we can use!
            if let Value::Object(ref mut params_map) = params {
                // Extract just the token part (remove "Bearer " prefix if present)
                let token = if auth_value.starts_with("Bearer ") {
                    &auth_value[7..]
                } else {
                    auth_value
                };
                
                // Add token to payload where JWT macro can find it
                params_map.insert("access_token".to_string(), Value::String(token.to_string()));
                params_map.insert("jwt".to_string(), Value::String(token.to_string()));
                params_map.insert("token".to_string(), Value::String(token.to_string()));
                
                debug!("🔐 Forwarding JWT token to microservice payload: {}...", 
                    if token.len() > 20 { 
                        &token[..20] 
                    } else { 
                        token 
                    }
                );
            }
        }
    }
    
    // Also forward other relevant auth headers
    let auth_headers = [
        "X-User-ID",
        "X-User-Role", 
        "X-User-Permissions",
        "X-Request-ID",
        "X-Forwarded-For"
    ];
    
    for header_name in auth_headers.iter() {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(value_str) = header_value.to_str() {
                if let Value::Object(ref mut params_map) = params {
                    let key = format!("_header_{}", header_name.to_lowercase().replace("-", "_"));
                    params_map.insert(key, Value::String(value_str.to_string()));
                }
            }
        }
    }
    
    // *** CRITICAL *** - Forward path parameters as special metadata fields
    // Since ServiceClient doesn't support setting msg.metadata directly,
    // we use special payload fields that the service can access
    if let Value::Object(ref mut params_map) = params {
        for (key, value) in path_params {
            // Add path params with special metadata prefix so services know these are metadata
            let metadata_key = format!("_metadata_{}", key);
            params_map.insert(metadata_key, Value::String(value.clone()));
            
            debug!("🔗 Forwarding path parameter to microservice: {} = {}", key, value);
        }
    }
    
    params
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
    
    // Dynamically discover services from RabbitMQ queues
    let discovered_service_names = discover_services_from_rabbitmq().await;
    
    for service_name in discovered_service_names {
        // Try to ping each service to see if it's available
        match state.service_client.call_with_timeout(
            &service_name,
            "ping",
            serde_json::json!({}),
            Duration::from_secs(2)
        ).await {
            Ok(_) => {
                discovered_services.push(serde_json::json!({
                    "name": service_name,
                    "status": "healthy",
                    "discovered_via": "rabbitmq_queue_discovery"
                }));
            }
            Err(_) => {
                // Service not responding, but queue exists
                discovered_services.push(serde_json::json!({
                    "name": service_name,
                    "status": "unreachable",
                    "discovered_via": "rabbitmq_queue_discovery"
                }));
            }
        }
    }
    
    Json(serde_json::json!({ 
        "services": discovered_services,
        "discovery_method": "rabbitmq_queue_discovery",
        "total_discovered": discovered_services.len()
    }))
}

/// Dynamically discover services by querying RabbitMQ management API
async fn discover_services_from_rabbitmq() -> Vec<String> {
    use reqwest;
    
    let mut services = Vec::new();
    
    // Query RabbitMQ management API for queues
    let client = reqwest::Client::new();
    match client
        .get("http://localhost:15672/api/queues")
        .basic_auth("guest", Some("guest"))
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(queues) = response.json::<Vec<serde_json::Value>>().await {
                for queue in queues {
                    if let Some(queue_name) = queue["name"].as_str() {
                        // Extract service names from queue names
                        // Format: rabbitmesh.{service-name} or rabbitmesh.{service-name}.responses
                        if queue_name.starts_with("rabbitmesh.") && !queue_name.ends_with(".responses") {
                            let service_name = queue_name
                                .strip_prefix("rabbitmesh.")
                                .unwrap_or(queue_name);
                            
                            // Avoid duplicates
                            if !services.contains(&service_name.to_string()) {
                                services.push(service_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to discover services from RabbitMQ: {}", e);
        }
    }
    
    services
}

/// List services in format expected by client generator
async fn list_services_for_client_generator(State(state): State<GatewayState>) -> Json<Value> {
    let mut endpoints = Vec::new();
    
    // Dynamically discover services from RabbitMQ queues
    let discovered_service_names = discover_services_from_rabbitmq().await;
    
    for service_name in discovered_service_names {
        // Get methods for each service
        let methods = discover_service_methods(&state, &service_name).await;
        
        // Convert each method to the endpoint format expected by client generator
        for method in methods {
            if let Some(method_obj) = method.as_object() {
                // Get route and convert path parameters from :param to {param} (generic)
                let original_path = method_obj.get("path").and_then(|p| p.as_str()).unwrap_or("/unknown");
                let gateway_path = format!("/api/v1/{}{}", service_name, convert_path_parameters(original_path));
                
                let endpoint = serde_json::json!({
                    "service": service_name,
                    "handler": method_obj.get("name").unwrap_or(&serde_json::Value::String("unknown".to_string())),
                    "method": method_obj.get("http_method").unwrap_or(&serde_json::Value::String("GET".to_string())),
                    "path": gateway_path,
                    "description": method_obj.get("description").unwrap_or(&serde_json::Value::String("Auto-generated method".to_string())),
                    "parameters": [],
                    "response_type": "any"
                });
                endpoints.push(endpoint);
            }
        }
    }
    
    Json(serde_json::json!({
        "endpoints": endpoints,
        "discovery_method": "dynamic_rabbitmq_discovery",
        "total_endpoints": endpoints.len()
    }))
}

/// Dynamically discover service methods by querying the service
async fn discover_service_methods(state: &GatewayState, service_name: &str) -> Vec<serde_json::Value> {
    // Try to call a schema or introspection method on the service
    match state.service_client.call_with_timeout(
        service_name,
        "schema",
        serde_json::json!({}),
        Duration::from_secs(3)
    ).await {
        Ok(rabbitmesh::message::RpcResponse::Success { data, .. }) => {
            // Service returned its schema - data is the JSON object directly
            if let Some(methods) = data.get("methods").and_then(|m| m.as_array()) {
                return methods.iter()
                    .filter_map(|method| {
                        if let (Some(name), Some(route)) = (
                            method.get("name").and_then(|n| n.as_str()),
                            method.get("route").and_then(|r| r.as_str())
                        ) {
                            Some(serde_json::json!({
                                "name": name,
                                "route": route,
                                "http_method": method.get("http_method").and_then(|h| h.as_str()).unwrap_or("GET"),
                                "path": method.get("path").and_then(|p| p.as_str()).unwrap_or("/unknown"),
                                "description": method.get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or(&format!("Method: {}", name))
                            }))
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        },
        Ok(rabbitmesh::message::RpcResponse::Error { error, .. }) => {
            tracing::warn!("Service {} schema call returned error: {}", service_name, error);
        },
        Err(e) => {
            tracing::debug!("Schema call failed for service {}: {}", service_name, e);
        }
    }

    // Fallback to basic ping method
    vec![
        serde_json::json!({
            "name": "ping",
            "route": "GET /ping",
            "http_method": "GET", 
            "path": "/ping",
            "description": "Health check endpoint"
        })
    ]
}

/// Generic function to convert path parameters from :param to {param}
/// Works with any parameter name, making it truly generic for any project
/// 
/// Examples:
/// - "/auth/login" -> "/auth/login" (no change)
/// - "/users/:id" -> "/users/{id}"
/// - "/orders/:order_id/items/:item_id" -> "/orders/{order_id}/items/{item_id}"
/// - "/products/:category/:subcategory/:sku" -> "/products/{category}/{subcategory}/{sku}"
fn convert_path_parameters(path: &str) -> String {
    use regex::Regex;
    
    // Create regex to match :parameter patterns
    let param_regex = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    
    // Replace all :param with {param}
    param_regex.replace_all(path, "{$1}").to_string()
}

/// Discover all available services by querying RabbitMQ management API
/// This is the key to making the router completely dynamic and generic
async fn discover_all_services(state: &GatewayState) -> Result<Vec<DiscoveredService>, Box<dyn std::error::Error + Send + Sync>> {
    // Query RabbitMQ management API to find all queues (services)
    let client = reqwest::Client::new();
    let rabbitmq_url = std::env::var("RABBITMQ_MANAGEMENT_URL")
        .unwrap_or_else(|_| "http://guest:guest@localhost:15672".to_string());
    
    let response = client
        .get(&format!("{}/api/queues", rabbitmq_url))
        .send()
        .await?;
    
    let queues: Vec<serde_json::Value> = response.json().await?;
    
    let mut services = Vec::new();
    
    // Find all service queues (not response queues)
    for queue in queues {
        if let Some(name) = queue.get("name").and_then(|n| n.as_str()) {
            if name.starts_with("rabbitmesh.") && !name.ends_with(".responses") {
                // Extract service name (e.g., "rabbitmesh.auth-service" -> "auth-service")
                if let Some(service_name) = name.strip_prefix("rabbitmesh.") {
                    // Get service schema to discover endpoints
                    match get_service_schema(state, service_name).await {
                        Ok(endpoints) => {
                            services.push(DiscoveredService {
                                name: service_name.to_string(),
                                endpoints,
                            });
                        }
                        Err(e) => {
                            debug!("Could not get schema for service {}: {:?}", service_name, e);
                            // Continue with other services
                        }
                    }
                }
            }
        }
    }
    
    Ok(services)
}

/// Get service schema by calling introspection methods on the actual services
/// This discovers the real endpoints by asking each service what methods it has
async fn get_service_schema(state: &GatewayState, service_name: &str) -> Result<Vec<ServiceEndpoint>, Box<dyn std::error::Error + Send + Sync>> {
    // First, let's try different common method names to get service info
    let introspection_methods = vec!["schema", "get_methods", "get_service_info", "describe", "introspect"];
    
    for method_name in introspection_methods {
        match state.service_client.call_with_timeout(
            service_name, 
            method_name, 
            serde_json::json!({}),
            Duration::from_secs(3)
        ).await {
            Ok(rabbitmesh::message::RpcResponse::Success { data, .. }) => {
                debug!("✅ Got schema from {}.{}", service_name, method_name);
                return parse_service_schema(service_name, &data);
            }
            Ok(rabbitmesh::message::RpcResponse::Error { .. }) => {
                debug!("Method {} not available on {}", method_name, service_name);
                continue;
            }
            Err(_) => {
                debug!("Failed to call {} on {}", method_name, service_name);
                continue;
            }
        }
    }
    
    // If no introspection method worked, use intelligent reverse-engineering
    // Parse the service source code or use inventory metadata if available
    debug!("🔍 No introspection available for {}, using intelligent discovery", service_name);
    discover_endpoints_intelligently(state, service_name).await
}

/// Parse service schema from response data
fn parse_service_schema(service_name: &str, data: &Value) -> Result<Vec<ServiceEndpoint>, Box<dyn std::error::Error + Send + Sync>> {
    let mut endpoints = Vec::new();
    
    // Try different schema formats
    if let Some(methods) = data.get("methods").and_then(|m| m.as_array()) {
        for method in methods {
            if let Some(endpoint) = parse_method_definition(service_name, method) {
                endpoints.push(endpoint);
            }
        }
    } else if let Some(endpoints_array) = data.get("endpoints").and_then(|e| e.as_array()) {
        for endpoint_data in endpoints_array {
            if let Some(endpoint) = parse_endpoint_definition(service_name, endpoint_data) {
                endpoints.push(endpoint);
            }
        }
    }
    
    Ok(endpoints)
}

/// Parse a method definition from service schema
fn parse_method_definition(service_name: &str, method: &Value) -> Option<ServiceEndpoint> {
    let name = method.get("name")?.as_str()?;
    let route = method.get("route")?.as_str()?;
    
    // Parse route like "POST /auth/login" or "GET /users/:id"
    let (http_method, path) = route.split_once(' ')?;
    
    // Convert :param to {param} and create full gateway path
    let normalized_path = convert_path_parameters(path);
    let full_path = format!("/api/v1/{}{}", service_name, normalized_path);
    
    Some(ServiceEndpoint {
        handler: name.to_string(),
        http_method: http_method.to_string(),
        path_pattern: full_path,
        parameters: extract_parameter_names(&normalized_path),
    })
}

/// Parse an endpoint definition from service schema
fn parse_endpoint_definition(service_name: &str, endpoint: &Value) -> Option<ServiceEndpoint> {
    let handler = endpoint.get("handler")?.as_str()?;
    let method = endpoint.get("method")?.as_str()?;
    let path = endpoint.get("path")?.as_str()?;
    
    let full_path = format!("/api/v1/{}{}", service_name, path);
    
    Some(ServiceEndpoint {
        handler: handler.to_string(),
        http_method: method.to_string(),
        path_pattern: full_path,
        parameters: extract_parameter_names(path),
    })
}

/// Intelligently discover endpoints by testing common service patterns
/// This is the fallback when introspection isn't available
async fn discover_endpoints_intelligently(
    state: &GatewayState, 
    service_name: &str
) -> Result<Vec<ServiceEndpoint>, Box<dyn std::error::Error + Send + Sync>> {
    let mut discovered_endpoints = Vec::new();
    
    // Test common method patterns by actually calling them
    let test_methods = vec![
        // Auth patterns
        ("login", "POST", "/auth/login"),
        ("validate_token", "POST", "/auth/validate"),  
        ("get_current_user", "GET", "/auth/me"),
        ("logout", "POST", "/auth/logout"),
        ("refresh_token", "POST", "/auth/refresh"),
        ("check_permission", "POST", "/auth/check-permission"),
        
        // CRUD patterns  
        ("list_users", "GET", "/users"),
        ("create_user", "POST", "/users"),
        ("get_user", "GET", "/users/{id}"),
        ("update_user", "PUT", "/users/{id}"),
        ("delete_user", "DELETE", "/users/{id}"),
        ("get_user_by_email", "GET", "/users/email/{email}"),
        
        // Order patterns
        ("list_orders", "GET", "/orders"),
        ("create_order", "POST", "/orders"), 
        ("get_order", "GET", "/orders/{id}"),
        ("update_order", "PUT", "/orders/{id}"),
        ("cancel_order", "DELETE", "/orders/{id}"),
        ("get_user_orders", "GET", "/orders/user/{user_id}"),
        
        // Generic patterns
        ("ping", "GET", "/ping"),
        ("health", "GET", "/health"),
        ("status", "GET", "/status"),
    ];
    
    for (method_name, http_method, path_template) in test_methods {
        // Test if this method exists by attempting to call it (with timeout)
        match state.service_client.call_with_timeout(
            service_name,
            method_name,
            serde_json::json!({"_probe": true}), // Special probe parameter
            Duration::from_millis(500) // Very short timeout for probing
        ).await {
            Ok(_) => {
                // Method exists! Add it to discovered endpoints
                let full_path = format!("/api/v1/{}{}", service_name, path_template);
                discovered_endpoints.push(ServiceEndpoint {
                    handler: method_name.to_string(),
                    http_method: http_method.to_string(),
                    path_pattern: full_path,
                    parameters: extract_parameter_names(path_template),
                });
                debug!("✅ Discovered method: {}.{}", service_name, method_name);
            }
            Err(e) => {
                // Method doesn't exist or failed - that's fine for discovery
                debug!("⚡ Method {}.{} not available: {:?}", service_name, method_name, e);
            }
        }
    }
    
    // If no methods were discovered, add at least a ping method
    if discovered_endpoints.is_empty() {
        discovered_endpoints.push(ServiceEndpoint {
            handler: "ping".to_string(),
            http_method: "GET".to_string(),
            path_pattern: format!("/api/v1/{}/ping", service_name),
            parameters: vec![],
        });
    }
    
    debug!("🔍 Intelligently discovered {} endpoints for {}", discovered_endpoints.len(), service_name);
    Ok(discovered_endpoints)
}

/// Find the matching endpoint for a given HTTP request
/// This implements the smart matching logic that handles any path pattern
fn find_matching_endpoint<'a>(service: &'a DiscoveredService, http_method: &str, request_path: &str) -> Option<&'a ServiceEndpoint> {
    for endpoint in &service.endpoints {
        if endpoint.http_method.eq_ignore_ascii_case(http_method) {
            if paths_match(&endpoint.path_pattern, request_path) {
                return Some(endpoint);
            }
        }
    }
    None
}

/// Check if a request path matches an endpoint pattern
/// Handles both exact matches and parameterized patterns
/// 
/// Examples:
/// - pattern: "/api/v1/auth-service/auth/login", path: "/api/v1/auth-service/auth/login" -> true
/// - pattern: "/api/v1/user-service/users/{id}", path: "/api/v1/user-service/users/123" -> true
/// - pattern: "/api/v1/order-service/orders", path: "/api/v1/order-service/orders" -> true
fn paths_match(pattern: &str, path: &str) -> bool {
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    let path_segments: Vec<&str> = path.split('/').collect();
    
    // Must have same number of segments
    if pattern_segments.len() != path_segments.len() {
        return false;
    }
    
    // Check each segment
    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        // If pattern segment is a parameter (starts with {), it matches any value
        if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            continue;
        }
        
        // Otherwise, must be exact match
        if pattern_seg != path_seg {
            return false;
        }
    }
    
    true
}

/// Extract parameter values from a request path based on a pattern
/// 
/// Example:
/// - pattern: "/api/v1/user-service/users/{id}", path: "/api/v1/user-service/users/123"
/// - returns: [("id", "123")]
fn extract_path_parameters(pattern: &str, path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    let pattern_segments: Vec<&str> = pattern.split('/').collect();
    let path_segments: Vec<&str> = path.split('/').collect();
    
    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // Extract parameter name from {param_name}
            let param_name = &pattern_seg[1..pattern_seg.len() - 1];
            params.insert(param_name.to_string(), path_seg.to_string());
        }
    }
    
    params
}

/// Extract parameter names from a path pattern
fn extract_parameter_names(path: &str) -> Vec<String> {
    use regex::Regex;
    
    let param_regex = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
    param_regex
        .captures_iter(path)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Convert service route format to gateway route format
/// e.g., "POST /users/:id" -> ("POST", "/api/v1/user-service/users/{id}")
fn convert_route_to_gateway_format(service_name: &str, route: &str) -> serde_json::Value {
    let parts: Vec<&str> = route.splitn(2, ' ').collect();
    if parts.len() == 2 {
        let method = parts[0];
        let path = convert_path_parameters(parts[1]);
        
        serde_json::json!({
            "http_method": method,
            "gateway_path": format!("/api/v1/{}{}", service_name, path)
        })
    } else {
        serde_json::json!({
            "http_method": "GET",
            "gateway_path": format!("/api/v1/{}/unknown", service_name)
        })
    }
}

/// Describe a specific service and its methods
async fn describe_service(
    State(state): State<GatewayState>,
    Path(service): Path<String>,
) -> Result<Json<Value>, GatewayError> {
    // First check the service registry
    if let Some(service_info) = state.service_registry.get_service(&service).await {
        return Ok(Json(serde_json::json!(service_info)));
    }
    
    // If not in registry, check if it's a dynamically discovered service
    let discovered_services = discover_services_from_rabbitmq().await;
    if discovered_services.contains(&service) {
        // Try to ping the service to check its status
        let status = match state.service_client.call_with_timeout(
            &service,
            "ping",
            serde_json::json!({}),
            Duration::from_secs(2)
        ).await {
            Ok(_) => "healthy",
            Err(_) => "unreachable",
        };
        
        // Try to get methods from the service
        let methods = discover_service_methods(&state, &service).await;
        
        return Ok(Json(serde_json::json!({
            "name": service,
            "status": status,
            "discovery_method": "rabbitmq_queue_discovery",
            "description": format!("Dynamically discovered service: {}", service),
            "methods": methods,
            "total_methods": methods.len()
        })));
    }
    
    Err(GatewayError::ServiceNotFound(service))
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
    ServiceDiscoveryError { error: String },
    NotFound { path: String, method: String },
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
            GatewayError::ServiceDiscoveryError { error } => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({
                        "error": format!("Service discovery failed: {}", error),
                        "code": "SERVICE_DISCOVERY_ERROR"
                    })
                )
            }
            GatewayError::NotFound { path, method } => {
                (
                    StatusCode::NOT_FOUND,
                    serde_json::json!({
                        "error": format!("No service endpoint found for {} {}", method, path),
                        "code": "ENDPOINT_NOT_FOUND",
                        "path": path,
                        "method": method
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