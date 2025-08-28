use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, ItemFn, Lit, LitStr, 
    parse::Parse, Token, punctuated::Punctuated,
};

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
pub fn impl_service_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ServiceMethodArgs);
    let input = parse_macro_input!(input as ItemFn);
    
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;
    
    // Generate method metadata for registry
    let route_info = if let Some(route) = args.route {
        quote! { Some(#route) }
    } else {
        quote! { None }
    };
    
    // Generate the actual function with metadata
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }
        
        // Register this method with the global registry
        inventory::submit! {
            rabbitmesh_macros::registry::MethodDefinition {
                method_name: stringify!(#fn_name),
                http_route: #route_info,
                function_name: stringify!(#fn_name),
            }
        }
    };

    TokenStream::from(expanded)
}