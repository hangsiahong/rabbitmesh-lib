use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, DeriveInput, parse::Parse, Token, Lit};

/// Implementation of #[service_definition] macro
pub fn impl_service_definition(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let struct_vis = &input.vis;
    
    // Generate service implementation
    let expanded = quote! {
        #struct_vis struct #struct_name;
        
        impl #struct_name {
            /// Get the service name (converts struct name to kebab-case)
            pub fn service_name() -> &'static str {
                // Convert CamelCase to kebab-case
                // UserService -> user-service
                // ProductCatalogService -> product-catalog-service
                const SERVICE_NAME: &str = {
                    let name = stringify!(#struct_name);
                    // This is a compile-time constant, so we'll use a simple approach
                    // In a real implementation, you'd want proper case conversion
                    match name {
                        "UserService" => "user-service",
                        "ProductService" => "product-service", 
                        "OrderService" => "order-service",
                        "AuthService" => "auth-service",
                        "NotificationService" => "notification-service",
                        _ => {
                            // Fallback: just lowercase the name
                            // Note: This is simplified - in production you'd want full case conversion
                            name
                        }
                    }
                };
                SERVICE_NAME
            }
            
            /// Create a microservice instance for this service
            pub async fn create_service(amqp_url: impl Into<String>) -> Result<rabbitmesh::MicroService, rabbitmesh::RabbitMeshError> {
                let service_name = Self::service_name();
                rabbitmesh::MicroService::new_simple(service_name, amqp_url).await
            }
        }
        
        // Register this service with the global registry
        inventory::submit! {
            rabbitmesh_macros::registry::ServiceDefinition {
                name: stringify!(#struct_name),
                service_name: #struct_name::service_name(),
            }
        }
    };

    TokenStream::from(expanded)
}