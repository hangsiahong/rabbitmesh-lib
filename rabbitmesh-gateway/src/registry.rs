use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Service registry for tracking available services and their methods
#[derive(Debug)]
pub struct ServiceRegistry {
    services: RwLock<HashMap<String, ServiceInfo>>,
}

/// Information about a registered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub methods: Vec<MethodInfo>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Information about a service method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    pub name: String,
    pub http_route: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
}

/// Parameter information for a method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
}

impl ServiceRegistry {
    /// Create new service registry
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Register a service
    pub async fn register_service(&self, service_info: ServiceInfo) {
        let mut services = self.services.write().await;
        services.insert(service_info.name.clone(), service_info);
    }

    /// Get service information
    pub async fn get_service(&self, name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Remove a service
    pub async fn unregister_service(&self, name: &str) {
        let mut services = self.services.write().await;
        services.remove(name);
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}