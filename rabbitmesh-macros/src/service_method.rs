use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, parse::Parse};

/// Arguments for #[service_method] attribute
struct ServiceMethodArgs {
    route: Option<String>,
}

impl Parse for ServiceMethodArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ServiceMethodArgs { route: None });
        }
        
        let route_lit: LitStr = input.parse()?;
        Ok(ServiceMethodArgs {
            route: Some(route_lit.value()),
        })
    }
}

/// Implementation of #[service_method] macro
/// Registers the endpoint and preserves the original function
pub fn impl_service_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ServiceMethodArgs);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // If route is provided, register the endpoint
    if let Some(route) = args.route {
        // Parse HTTP method and path from route (e.g., "POST /api/auth/login")
        let parts: Vec<&str> = route.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let http_method = parts[0].to_string();
            let http_path = parts[1].to_string();
            let method_name = input_fn.sig.ident.to_string();
            
            // Create endpoint registration (this will be collected by the gateway generator)
            let endpoint = crate::ServiceEndpoint {
                service_name: "unknown".to_string(), // Will be set by service_impl
                method_name,
                http_route: http_path,
                http_method,
                requires_auth: false, // Will be detected from other attributes
                required_roles: Vec::new(), // Will be detected from other attributes
                required_permissions: Vec::new(),
            };
            
            // Register the endpoint
            crate::register_service_endpoint(endpoint);
        }
    }
    
    // Pass through the original function
    let expanded = quote! {
        #input_fn
    };

    TokenStream::from(expanded)
}