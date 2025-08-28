use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, FnArg, Pat, Type, LitStr};

/// Arguments for method attributes
#[derive(Debug)]
struct MethodInfo {
    name: String,
    route: Option<String>,
    params: Vec<(String, Type)>,
    return_type: Type,
}

/// Implementation of #[service_impl] macro that processes entire impl blocks
pub fn impl_service_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemImpl);
    
    if let Type::Path(type_path) = &*input.self_ty {
        let struct_name = &type_path.path.segments.last().unwrap().ident;
        
        let mut methods = Vec::new();
        let mut new_items = Vec::new();
        
        // Process each method in the impl block
        for item in &input.items {
            if let ImplItem::Fn(method) = item {
                let method_name = method.sig.ident.to_string();
                
                // Check if this method has #[service_method] attribute
                let mut route = None;
                let mut is_service_method = false;
                
                for attr in &method.attrs {
                    if attr.path().is_ident("service_method") {
                        is_service_method = true;
                        
                        // Parse route if provided
                        if let Ok(lit) = attr.parse_args::<LitStr>() {
                            route = Some(lit.value());
                        }
                        break;
                    }
                }
                
                if is_service_method {
                    // Extract parameter information
                    let mut params = Vec::new();
                    for input in &method.sig.inputs {
                        if let FnArg::Typed(pat_type) = input {
                            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                                params.push((
                                    pat_ident.ident.to_string(),
                                    (*pat_type.ty).clone()
                                ));
                            }
                        }
                    }
                    
                    // Extract return type
                    let return_type = if let syn::ReturnType::Type(_, ty) = &method.sig.output {
                        (**ty).clone()
                    } else {
                        syn::parse_quote!(())
                    };
                    
                    methods.push(MethodInfo {
                        name: method_name,
                        route,
                        params,
                        return_type,
                    });
                }
                
                // Remove #[service_method] attribute and add the method
                let mut new_method = method.clone();
                new_method.attrs.retain(|attr| !attr.path().is_ident("service_method"));
                new_items.push(ImplItem::Fn(new_method));
            } else {
                new_items.push(item.clone());
            }
        }
        
        // Generate handler registration code
        let handler_registrations = methods.iter().map(|method| {
            let method_name_str = &method.name;
            let method_ident = syn::Ident::new(&method.name, proc_macro2::Span::call_site());
            
            if method.params.is_empty() {
                // No parameters
                quote! {
                    service.register_function(#method_name_str, move |_msg: rabbitmesh::Message| {
                        async move {
                            Ok(match Self::#method_ident().await {
                                Ok(result) => rabbitmesh::RpcResponse::success(result, 0).unwrap(),
                                Err(err) => rabbitmesh::RpcResponse::error(&err),
                            })
                        }
                    }).await;
                }
            } else if method.params.len() == 1 {
                // Single parameter
                quote! {
                    service.register_function(#method_name_str, move |msg: rabbitmesh::Message| {
                        async move {
                            Ok(match msg.deserialize_payload() {
                                Ok(param) => {
                                    match Self::#method_ident(param).await {
                                        Ok(result) => rabbitmesh::RpcResponse::success(result, 0).unwrap(),
                                        Err(err) => rabbitmesh::RpcResponse::error(&err),
                                    }
                                }
                                Err(e) => rabbitmesh::RpcResponse::error(&format!("Deserialization error: {}", e)),
                            })
                        }
                    }).await;
                }
            } else {
                // Multiple parameters - expect tuple or struct
                quote! {
                    service.register_function(#method_name_str, move |msg: rabbitmesh::Message| {
                        async move {
                            Ok(match msg.deserialize_payload() {
                                Ok(params) => {
                                    match Self::#method_ident(params).await {
                                        Ok(result) => rabbitmesh::RpcResponse::success(result, 0).unwrap(),
                                        Err(err) => rabbitmesh::RpcResponse::error(&err),
                                    }
                                }
                                Err(e) => rabbitmesh::RpcResponse::error(&format!("Deserialization error: {}", e)),
                            })
                        }
                    }).await;
                }
            }
        });
        
        // Generate route information for API gateway
        let routes = methods.iter().filter_map(|method| {
            method.route.as_ref().map(|route| {
                let method_name = &method.name;
                quote! {
                    (#route, #method_name)
                }
            })
        });
        
        // Add the auto-generated register_handlers method
        let register_handlers_method = quote! {
            /// Auto-generated handler registration - DO NOT MODIFY
            pub async fn register_handlers(service: &rabbitmesh::MicroService) -> anyhow::Result<()> {
                use rabbitmesh::{Message, RpcResponse};
                
                tracing::info!("🔧 Auto-registering service methods...");
                
                #(#handler_registrations)*
                
                tracing::info!("✅ All service methods registered automatically");
                Ok(())
            }
        };
        
        // Add route information method
        let routes_method = quote! {
            /// Auto-generated route information for API gateway
            pub fn get_routes() -> Vec<(&'static str, &'static str)> {
                vec![#(#routes),*]
            }
        };
        
        // Add the generated methods to the impl block
        let register_handlers_item: ImplItem = syn::parse2(register_handlers_method).unwrap();
        let routes_item: ImplItem = syn::parse2(routes_method).unwrap();
        
        new_items.push(register_handlers_item);
        new_items.push(routes_item);
        
        // Generate the complete impl block
        let generics = &input.generics;
        let self_ty = &input.self_ty;
        let trait_ = &input.trait_;
        let attrs = &input.attrs;
        
        let expanded = if let Some((bang, path, for_token)) = trait_ {
            quote! {
                #(#attrs)*
                impl #generics #bang #path #for_token #self_ty {
                    #(#new_items)*
                }
            }
        } else {
            quote! {
                #(#attrs)*
                impl #generics #self_ty {
                    #(#new_items)*
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        // Return error if not a proper type
        quote! {
            compile_error!("service_impl can only be applied to impl blocks for named types");
        }.into()
    }
}