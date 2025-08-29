use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, FnArg, Pat, Type, LitStr, Attribute};
use crate::universal::{UniversalMacroProcessor, MacroContext, MethodParam, ParamSource};

/// Arguments for method attributes
#[derive(Debug)]
struct MethodInfo {
    name: String,
    route: Option<String>,
    params: Vec<(String, Type)>,
    return_type: Type,
}

/// Check if a parameter is injected by macros
fn is_injected_param(param_name: &str) -> bool {
    matches!(param_name, "user" | "auth_context" | "db" | "db_context" | "cache" | "cache_context" | "service_context")
}

/// Determine parameter source based on parameter name and type
fn determine_param_source(param_name: &str) -> ParamSource {
    match param_name {
        "user" | "auth_context" => ParamSource::Injected("AuthContext".to_string()),
        "db" | "db_context" => ParamSource::Injected("DbContext".to_string()),
        "cache" | "cache_context" => ParamSource::Injected("CacheContext".to_string()),
        "service_context" => ParamSource::Context,
        param if param.ends_with("_id") => ParamSource::Path,
        _ => ParamSource::Body,
    }
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
                    
                    // Create macro context for universal processing
                    let macro_context = MacroContext {
                        service_name: struct_name.to_string(),
                        method_name: method_name.clone(),
                        method_params: params.iter().map(|(name, ty)| MethodParam {
                            name: name.clone(),
                            param_type: quote!(#ty).to_string(),
                            is_injected: is_injected_param(name),
                            source: determine_param_source(name),
                        }).collect(),
                        return_type: quote!(#return_type).to_string(),
                        attributes: UniversalMacroProcessor::parse_attributes(&method.attrs),
                        http_route: route.clone(),
                    };
                    
                    methods.push((MethodInfo {
                        name: method_name,
                        route,
                        params,
                        return_type,
                    }, macro_context));
                }
                
                // Remove universal macro attributes but keep #[service_method] for now
                let mut new_method = method.clone();
                new_method.attrs.retain(|attr| {
                    // Keep service_method but remove universal macro attributes
                    if attr.path().is_ident("service_method") {
                        return true;
                    }
                    
                    // Remove all universal macro attributes
                    let attr_name = attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
                    !matches!(attr_name.as_str(),
                        // Authorization macros
                        "require_auth" | "require_role" | "require_permission" | "require_ownership" |
                        "require_attributes" | "require_2fa" | "require_any_role" | "optional_auth" |
                        
                        // Database macros
                        "transactional" | "read_only" | "isolated" | "database" | "collection" |
                        "auto_save" | "soft_delete" | "versioned" | "audited" |
                        
                        // Caching macros
                        "cached" | "cache_invalidate" | "cache_through" | "cache_aside" |
                        "cache_key" | "cache_ttl" | "cache_tags" | "multi_level_cache" |
                        
                        // Validation macros
                        "validate" | "sanitize" | "transform" | "constrain" | "custom_validate" |
                        "validate_email" | "validate_phone" | "validate_range" |
                        
                        // Rate limiting macros
                        "rate_limit" | "throttle" | "circuit_breaker" | "timeout" | "retry" |
                        "bulkhead" | "backpressure" |
                        
                        // Observability macros
                        "metrics" | "trace" | "log" | "monitor" | "alert" | "health_check" |
                        "profile" | "benchmark" |
                        
                        // Workflow macros
                        "state_machine" | "saga" | "workflow" | "approval_required" |
                        "event_sourced" | "cqrs" | "projection" |
                        
                        // Integration macros
                        "webhook" | "event_publish" | "event_subscribe" | "queue_message" |
                        "external_api" | "idempotent" | "compensate" |
                        
                        // Security macros
                        "encrypt" | "decrypt" | "sign" | "verify" | "hash" | "audit_log" |
                        "pii_mask" | "gdpr_compliant" | "hipaa_compliant" |
                        
                        // Performance macros
                        "async_pool" | "batch_process" | "parallel" | "streaming" |
                        "lazy_load" | "prefetch" | "compress" |
                        
                        // Testing macros
                        "mock" | "stub" | "test_data" | "load_test" | "chaos_test" |
                        "a_b_test" | "feature_flag"
                    )
                });
                
                new_items.push(ImplItem::Fn(new_method));
            } else {
                new_items.push(item.clone());
            }
        }
        
        // Generate handler registration code with universal macro processing
        let handler_registrations = methods.iter().map(|(method, context)| {
            let method_name_str = &method.name;
            let method_ident = syn::Ident::new(&method.name, proc_macro2::Span::call_site());
            
            // Generate universal wrapper using the macro framework
            let universal_wrapper = UniversalMacroProcessor::generate_universal_wrapper(context);
            
            // For now, generate simple handler registration
            // The universal wrapper will handle all the macro processing
            quote! {
                service.register_function(#method_name_str, #universal_wrapper).await;
            }
        });
        
        // Generate route information for API gateway
        let routes = methods.iter().filter_map(|(method, _)| {
            method.route.as_ref().map(|route| {
                let method_name = &method.name;
                quote! {
                    (#route, #method_name)
                }
            })
        });
        
        // Add the auto-generated register_handlers method
        let register_handlers_method = quote! {
            /// Auto-generated handler registration with Universal Macro Framework
            /// Supports RBAC, ABAC, Hybrid authorization, database transactions, 
            /// caching, validation, observability, workflows, and more!
            pub async fn register_handlers(service: &rabbitmesh::MicroService) -> anyhow::Result<()> {
                use rabbitmesh::{Message, RpcResponse};
                use tracing::{info, warn, error, debug, trace};
                
                info!("🌟 Registering service methods with Universal Macro Framework...");
                info!("🔐 Authorization: RBAC, ABAC, Hybrid patterns supported");
                info!("💾 Database: Universal transactions for SQL/NoSQL/Graph/TimeSeries");
                info!("⚡ Caching: Multi-level intelligent caching with domain optimizations");
                info!("✅ Validation: Comprehensive input validation + security + compliance");
                info!("📊 Observability: Complete metrics, tracing, logging, monitoring");
                info!("🎭 Workflows: State machines, sagas, approvals, event sourcing, CQRS");
                
                #(#handler_registrations)*
                
                info!("✨ All service methods registered with enterprise-grade features!");
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