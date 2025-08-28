/// Global service registry using inventory crate
/// 
/// This allows the proc macros to register services at compile time
/// and the gateway to discover them at runtime.

/// Service definition registered by #[service_definition]
#[derive(Debug)]
pub struct ServiceDefinition {
    pub name: &'static str,
    pub service_name: &'static str,
}

/// Method definition registered by #[service_method] 
#[derive(Debug)]
pub struct MethodDefinition {
    pub method_name: &'static str,
    pub http_route: Option<&'static str>,
    pub function_name: &'static str,
}

inventory::collect!(ServiceDefinition);
inventory::collect!(MethodDefinition);

/// Get all registered services
pub fn get_registered_services() -> Vec<&'static ServiceDefinition> {
    inventory::iter::<ServiceDefinition>.into_iter().collect()
}

/// Get all registered methods
pub fn get_registered_methods() -> Vec<&'static MethodDefinition> {
    inventory::iter::<MethodDefinition>.into_iter().collect()
}