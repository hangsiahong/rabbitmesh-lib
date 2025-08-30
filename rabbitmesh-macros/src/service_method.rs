use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, parse::Parse};

/// Arguments for #[service_method] attribute
struct ServiceMethodArgs {
    _route: Option<String>,
}

impl Parse for ServiceMethodArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(ServiceMethodArgs { _route: None });
        }
        
        let route_lit: LitStr = input.parse()?;
        Ok(ServiceMethodArgs {
            _route: Some(route_lit.value()),
        })
    }
}

/// Implementation of #[service_method] macro
/// Stores route information as metadata and preserves the original function
/// Route information is extracted by dynamic_discovery.rs for gateway generation
pub fn impl_service_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let _args = parse_macro_input!(args as ServiceMethodArgs);
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Note: Route information is now extracted by dynamic_discovery.rs
    // This macro just passes through the function with its attributes preserved
    
    // Pass through the original function unchanged
    let expanded = quote! {
        #input_fn
    };

    TokenStream::from(expanded)
}