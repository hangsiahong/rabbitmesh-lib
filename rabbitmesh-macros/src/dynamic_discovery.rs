//! Dynamic Service Discovery for Universal Gateway Generation
//! 
//! This module implements TRUE universal service discovery that works with ANY project type.
//! NO HARDCODING - discovers services dynamically from the workspace.

use std::fs;
use std::path::Path;
use syn::{ItemImpl, Item, ImplItem, Attribute, parse_file};
use quote::quote;
use proc_macro2::TokenStream;
use serde::{Serialize, Deserialize};

/// Discovered service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    pub name: String,
    pub methods: Vec<DiscoveredMethod>,
}

/// Discovered method information  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMethod {
    pub name: String,
    pub http_method: String,
    pub path: String,
    pub parameters: Vec<DiscoveredParameter>,
    pub macros: Vec<String>,
}

/// Discovered parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredParameter {
    pub name: String,
    pub param_type: String,
    pub required: bool,
}

/// Dynamic service discovery engine
pub struct ServiceDiscovery;

impl ServiceDiscovery {
    /// Discover all services in the current workspace
    pub fn discover_workspace_services() -> Vec<DiscoveredService> {
        let mut services = Vec::new();
        
        // 1. Find workspace root
        let workspace_root = Self::find_workspace_root();
        
        // 2. Parse workspace Cargo.toml to find all members
        let workspace_members = Self::parse_workspace_members(&workspace_root);
        
        // 3. Scan each workspace member for services
        for member in &workspace_members {
            let member_path = workspace_root.join(&member);
            if let Some(discovered) = Self::discover_services_in_crate(&member_path) {
                services.extend(discovered);
            }
        }
        
        // 4. If no workspace members found, scan current directory
        if services.is_empty() {
            if let Some(discovered) = Self::discover_services_in_crate(&std::env::current_dir().unwrap()) {
                services.extend(discovered);
            }
        }
        
        tracing::info!("🔍 Discovered {} services dynamically", services.len());
        for service in &services {
            tracing::info!("  📋 Service: {} ({} methods)", service.name, service.methods.len());
        }
        
        services
    }
    
    /// Find the workspace root directory
    fn find_workspace_root() -> std::path::PathBuf {
        let mut current_dir = std::env::current_dir().unwrap();
        
        // Walk up the directory tree looking for workspace Cargo.toml
        loop {
            let cargo_toml = current_dir.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if it's a workspace
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    if content.contains("[workspace]") {
                        return current_dir;
                    }
                }
                // If it's a regular Cargo.toml, check parent
                if let Some(parent) = current_dir.parent() {
                    current_dir = parent.to_path_buf();
                    continue;
                }
            }
            
            // Fallback to current directory
            return std::env::current_dir().unwrap();
        }
    }
    
    /// Parse workspace members from Cargo.toml using proper TOML parsing
    fn parse_workspace_members(workspace_root: &Path) -> Vec<String> {
        let cargo_toml = workspace_root.join("Cargo.toml");
        
        if let Ok(content) = fs::read_to_string(&cargo_toml) {
            // Use proper TOML parsing instead of manual string manipulation
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                if let Some(workspace) = toml_value.get("workspace") {
                    if let Some(members) = workspace.get("members") {
                        if let Some(members_array) = members.as_array() {
                            return members_array.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect();
                        }
                    }
                }
            }
        }
        
        Vec::new()
    }
    
    /// Discover services in a single crate
    fn discover_services_in_crate(crate_path: &Path) -> Option<Vec<DiscoveredService>> {
        let src_path = crate_path.join("src");
        if !src_path.exists() {
            return None;
        }
        
        let mut services = Vec::new();
        
        // Recursively scan all .rs files in src/
        Self::scan_rust_files(&src_path, &mut services);
        
        if services.is_empty() {
            None
        } else {
            Some(services)
        }
    }
    
    /// Recursively scan Rust files for service definitions
    fn scan_rust_files(dir: &Path, services: &mut Vec<DiscoveredService>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    Self::scan_rust_files(&path, services);
                } else if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        Self::parse_rust_file(&path, services);
                    }
                }
            }
        }
    }
    
    /// Parse a single Rust file for service definitions
    fn parse_rust_file(file_path: &Path, services: &mut Vec<DiscoveredService>) {
        if let Ok(content) = fs::read_to_string(file_path) {
            if let Ok(file) = parse_file(&content) {
                for item in file.items {
                    if let Item::Impl(impl_item) = item {
                        Self::process_impl_block(impl_item, services);
                    }
                }
            }
        }
    }
    
    /// Process an impl block looking for service_impl attribute
    fn process_impl_block(impl_item: ItemImpl, services: &mut Vec<DiscoveredService>) {
        // Check if this impl has #[service_impl] attribute
        let has_service_impl = impl_item.attrs.iter().any(|attr| {
            attr.path().is_ident("service_impl")
        });
        
        if !has_service_impl {
            return;
        }
        
        // Extract service name from the type
        let service_name = match &*impl_item.self_ty {
            syn::Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    "UnknownService".to_string()
                }
            },
            _ => "UnknownService".to_string(),
        };
        
        let mut methods = Vec::new();
        
        // Process each method in the impl block
        for item in &impl_item.items {
            if let ImplItem::Fn(method) = item {
                if let Some(discovered_method) = Self::process_service_method(method) {
                    methods.push(discovered_method);
                }
            }
        }
        
        if !methods.is_empty() {
            services.push(DiscoveredService {
                name: Self::service_name_to_kebab_case(&service_name),
                methods,
            });
        }
    }
    
    /// Process a method looking for service_method attribute
    fn process_service_method(method: &syn::ImplItemFn) -> Option<DiscoveredMethod> {
        // Check for #[service_method] attribute
        let service_method_attr = method.attrs.iter().find(|attr| {
            attr.path().is_ident("service_method")
        })?;
        
        let method_name = method.sig.ident.to_string();
        
        // Parse HTTP route from attribute
        let (http_method, path) = Self::parse_service_method_attr(service_method_attr)
            .unwrap_or(("POST".to_string(), format!("/{}", method_name)));
        
        // Extract macro attributes
        let macros = Self::extract_macro_attributes(&method.attrs);
        
        // Extract parameters (simplified)
        let parameters = Self::extract_parameters(&method.sig);
        
        Some(DiscoveredMethod {
            name: method_name,
            http_method,
            path,
            parameters,
            macros,
        })
    }
    
    /// Parse service_method attribute to extract HTTP method and path
    fn parse_service_method_attr(attr: &Attribute) -> Option<(String, String)> {
        // Try to parse the attribute arguments
        if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
            let route_str = lit_str.value();
            let parts: Vec<&str> = route_str.splitn(2, ' ').collect();
            
            if parts.len() == 2 {
                Some((parts[0].to_uppercase(), parts[1].to_string()))
            } else {
                Some(("POST".to_string(), route_str))
            }
        } else {
            None
        }
    }
    
    /// Extract macro attributes from method
    fn extract_macro_attributes(attrs: &[Attribute]) -> Vec<String> {
        let universal_macros = vec![
            "require_auth", "require_role", "require_permission", "validate", "rate_limit",
            "metrics", "audit_log", "cached", "transactional", "event_publish", "streaming",
            "batch_process", "require_ownership", "jwt_auth", "bearer_auth", "api_key_auth",
            "oauth", "rbac", "abac", "authorize", "admin_only", "user_only", "owner_only",
            "sanitize", "escape", "csrf_protect", "xss_protect", "throttle", "circuit_breaker",
            "timeout", "retry", "fallback", "redis_cache", "memory_cache", "read_only",
            "read_write", "trace", "monitor", "log", "prometheus", "websocket", "real_time",
            "async_process", "workflow", "state_machine", "saga", "event_sourcing", "cqrs"
        ];
        
        attrs.iter()
            .filter_map(|attr| {
                let path_str = attr.path().get_ident()?.to_string();
                if universal_macros.contains(&path_str.as_str()) {
                    Some(path_str)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Extract parameters from method signature (simplified)
    fn extract_parameters(sig: &syn::Signature) -> Vec<DiscoveredParameter> {
        let mut parameters = Vec::new();
        
        for input in &sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = pat_ident.ident.to_string();
                    let param_type = format!("{}", quote!(#pat_type.ty));
                    
                    parameters.push(DiscoveredParameter {
                        name: param_name,
                        param_type,
                        required: true, // Simplified - could be enhanced
                    });
                }
            }
        }
        
        parameters
    }
    
    /// Convert service name to kebab-case
    fn service_name_to_kebab_case(name: &str) -> String {
        // Remove "Service" suffix if present
        let name = name.strip_suffix("Service").unwrap_or(name);
        
        // Convert PascalCase to kebab-case
        let mut result = String::new();
        let mut chars = name.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap());
        }
        
        format!("{}-service", result)
    }
    
    /// Generate dynamic gateway router from discovered services
    pub fn generate_dynamic_gateway(services: Vec<DiscoveredService>) -> TokenStream {
        if services.is_empty() {
            return Self::generate_fallback_gateway();
        }
        
        let mut routes = Vec::new();
        let mut handlers = Vec::new();
        let mut service_list = Vec::new();
        
        for service in &services {
            for method in &service.methods {
                let handler_name = format!("{}_{}", service.name.replace("-", "_"), method.name);
                let handler_ident = syn::Ident::new(&handler_name, proc_macro2::Span::call_site());
                
                let route_path = format!("/api/v1/{}/{}", service.name, method.name);
                let http_method = method.http_method.to_uppercase();
                
                // Generate route registration
                let route_registration = match http_method.as_str() {
                    "GET" => quote! { .route(#route_path, axum::routing::get(#handler_ident)) },
                    "POST" => quote! { .route(#route_path, axum::routing::post(#handler_ident)) },
                    "PUT" => quote! { .route(#route_path, axum::routing::put(#handler_ident)) },
                    "DELETE" => quote! { .route(#route_path, axum::routing::delete(#handler_ident)) },
                    "PATCH" => quote! { .route(#route_path, axum::routing::patch(#handler_ident)) },
                    _ => quote! { .route(#route_path, axum::routing::post(#handler_ident)) },
                };
                routes.push(route_registration);
                
                // Generate handler function
                let service_name = &service.name;
                let method_name = &method.name;
                let handler_func = quote! {
                    async fn #handler_ident(
                        axum::extract::Query(query_params): axum::extract::Query<std::collections::HashMap<String, String>>,
                        body: Option<axum::Json<serde_json::Value>>
                    ) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
                        handle_dynamic_service_call(#service_name, #method_name, query_params, body).await
                    }
                };
                handlers.push(handler_func);
                
                // Add to service list for /api/services endpoint
                service_list.push(quote! {
                    serde_json::json!({
                        "method": #http_method,
                        "path": #route_path,
                        "service": #service_name,
                        "handler": #method_name
                    })
                });
            }
        }
        
        let services_count = services.len();
        let total_endpoints = service_list.len();
        
        quote! {
            pub struct AutoGeneratedGateway;
            
            impl AutoGeneratedGateway {
                pub async fn create_router() -> anyhow::Result<axum::Router> {
                    tracing::info!("🚀 Creating DYNAMIC gateway with {} discovered services", #services_count);
                    
                    let app = axum::Router::new()
                        .route("/api/services", axum::routing::get(list_discovered_services))
                        .route("/health", axum::routing::get(health_check))
                        #(#routes)*;
                    
                    tracing::info!("✅ Dynamic gateway created with {} endpoints from {} services", #total_endpoints, #services_count);
                    Ok(app)
                }
            }
            
            #(#handlers)*
            
            async fn list_discovered_services() -> axum::Json<serde_json::Value> {
                axum::Json(serde_json::json!({
                    "message": "Services dynamically discovered from workspace",
                    "services_count": #services_count,
                    "total_endpoints": #total_endpoints,
                    "endpoints": [#(#service_list),*]
                }))
            }
            
            async fn health_check() -> axum::Json<serde_json::Value> {
                axum::Json(serde_json::json!({
                    "status": "healthy",
                    "service": "dynamic-rabbitmesh-gateway",
                    "message": "Universal RabbitMesh Gateway - Dynamically Generated"
                }))
            }
            
            use std::sync::Arc;
            use tokio::sync::OnceCell;
            
            // Global singleton service client to avoid multiple consumers
            static GATEWAY_CLIENT: OnceCell<Arc<rabbitmesh::ServiceClient>> = OnceCell::const_new();
            
            async fn get_or_create_client() -> Result<Arc<rabbitmesh::ServiceClient>, axum::http::StatusCode> {
                GATEWAY_CLIENT.get_or_init(|| async {
                    match rabbitmesh::ServiceClient::new("dynamic-gateway", "amqp://guest:guest@localhost:5672/%2f").await {
                        Ok(client) => Arc::new(client),
                        Err(e) => {
                            tracing::error!("Failed to create RabbitMQ client: {}", e);
                            panic!("Cannot initialize RabbitMQ client");
                        }
                    }
                }).await;
                
                Ok(GATEWAY_CLIENT.get().unwrap().clone())
            }
            
            async fn handle_dynamic_service_call(
                service_name: &str,
                method_name: &str,
                query_params: std::collections::HashMap<String, String>,
                body: Option<axum::Json<serde_json::Value>>
            ) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
                use tokio::time::Duration;
                
                // Get the singleton service client
                let client = get_or_create_client().await?;
                
                // Prepare parameters
                let mut params = serde_json::Map::new();
                
                // Add query parameters
                for (key, value) in query_params {
                    let parsed_value = serde_json::from_str(&value)
                        .unwrap_or_else(|_| serde_json::Value::String(value));
                    params.insert(key, parsed_value);
                }
                
                // Add body parameters
                if let Some(axum::Json(body_value)) = body {
                    if let serde_json::Value::Object(body_map) = body_value {
                        for (key, value) in body_map {
                            params.insert(key, value);
                        }
                    }
                }
                
                let params_value = serde_json::Value::Object(params);
                
                tracing::debug!("📞 Calling service: {} method: {}", service_name, method_name);
                
                // Call service via RabbitMQ
                match client.call_with_timeout(service_name, method_name, params_value, Duration::from_secs(30)).await {
                    Ok(rabbitmesh::message::RpcResponse::Success { data, .. }) => {
                        tracing::debug!("✅ Service call successful: {}.{}", service_name, method_name);
                        Ok(axum::Json(data))
                    },
                    Ok(rabbitmesh::message::RpcResponse::Error { error, .. }) => {
                        tracing::warn!("❌ Service returned error: {}", error);
                        Err(axum::http::StatusCode::BAD_REQUEST)
                    },
                    Err(e) => {
                        tracing::error!("❌ RPC call failed: {}.{} - {}", service_name, method_name, e);
                        Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
        }
    }
    
    /// Generate fallback gateway when no services are discovered
    fn generate_fallback_gateway() -> TokenStream {
        quote! {
            pub struct AutoGeneratedGateway;
            
            impl AutoGeneratedGateway {
                pub async fn create_router() -> anyhow::Result<axum::Router> {
                    tracing::warn!("⚠️  No services discovered in workspace");
                    tracing::info!("💡 Add #[service_impl] and #[service_method] annotations to your services");
                    
                    let app = axum::Router::new()
                        .route("/api/services", axum::routing::get(no_services_found))
                        .route("/health", axum::routing::get(health_check));
                    
                    tracing::info!("🔧 Fallback gateway created - add service annotations to generate routes");
                    Ok(app)
                }
            }
            
            async fn no_services_found() -> axum::Json<serde_json::Value> {
                axum::Json(serde_json::json!({
                    "message": "No services found. Add #[service_impl] and #[service_method] annotations to your services.",
                    "services_count": 0,
                    "total_endpoints": 0,
                    "endpoints": [],
                    "help": "Example: #[service_impl] impl MyService { #[service_method(\"POST /endpoint\")] async fn my_method() {} }"
                }))
            }
            
            async fn health_check() -> axum::Json<serde_json::Value> {
                axum::Json(serde_json::json!({
                    "status": "healthy",
                    "service": "fallback-rabbitmesh-gateway",
                    "message": "Universal RabbitMesh Gateway - Awaiting Service Discovery"
                }))
            }
        }
    }
}