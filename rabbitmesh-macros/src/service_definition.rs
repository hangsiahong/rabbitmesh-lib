use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

/// Implementation of #[service_definition] macro
pub fn impl_service_definition(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let struct_vis = &input.vis;
    
    // Generate service implementation with automatic handler registration
    let expanded = quote! {
        #struct_vis struct #struct_name;
        
        impl #struct_name {
            /// Get the service name (converts struct name to kebab-case)
            pub fn service_name() -> &'static str {
                // Simple static service names - avoiding const match for now
                const SERVICE_NAME: &str = concat!(stringify!(#struct_name), "-service");
                SERVICE_NAME
            }
            
            /// Create and configure a microservice instance
            pub async fn create_service(amqp_url: impl Into<String>) -> anyhow::Result<rabbitmesh::MicroService> {
                let service_name = Self::service_name();
                let service = rabbitmesh::MicroService::new_simple(service_name, amqp_url).await?;
                
                // Register all methods marked with #[service_method]
                Self::register_handlers(&service).await?;
                
                Ok(service)
            }
            
            // register_handlers() should be implemented in the impl block manually
            // TODO: Auto-generate this from #[service_method] macros
        }
    };

    TokenStream::from(expanded)
}