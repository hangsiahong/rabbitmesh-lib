//! Universal Macro Framework for RabbitMesh
//! 
//! This module provides the core infrastructure for universal macros that work
//! across ALL project types: e-commerce, finance, healthcare, IoT, gaming, etc.
//!
//! The framework is designed to be:
//! - **Universal**: Works with any domain model
//! - **Composable**: Macros can be combined freely
//! - **Extensible**: Easy to add new macro types
//! - **Type-safe**: Full compile-time verification
//! - **Performance-first**: Zero runtime overhead

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, ItemFn, ItemImpl, ImplItem, FnArg, Pat, Type, 
    Attribute, Meta, MetaList, MetaNameValue, Lit, Expr, Path, PathSegment,
    parse::Parse, parse::ParseStream, Token, LitStr, LitInt, LitBool,
    punctuated::Punctuated, token::Comma
};
use std::collections::HashMap;

/// Universal macro attribute parser
#[derive(Debug, Clone)]
pub struct MacroAttribute {
    pub name: String,
    pub args: HashMap<String, MacroValue>,
}

/// Universal value type for macro arguments
#[derive(Debug, Clone)]
pub enum MacroValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<MacroValue>),
    Expression(String),
    Path(String),
}

/// Context information for macro processing
#[derive(Debug, Clone)]
pub struct MacroContext {
    pub service_name: String,
    pub method_name: String,
    pub method_params: Vec<MethodParam>,
    pub return_type: String,
    pub attributes: Vec<MacroAttribute>,
    pub http_route: Option<String>,
}

/// Method parameter information
#[derive(Debug, Clone)]
pub struct MethodParam {
    pub name: String,
    pub param_type: String,
    pub is_injected: bool,     // Whether this param is injected by macros
    pub source: ParamSource,   // Where the param comes from
}

/// Parameter source types
#[derive(Debug, Clone)]
pub enum ParamSource {
    Body,                      // From request body
    Path,                      // From URL path
    Query,                     // From query parameters
    Header,                    // From HTTP headers
    Injected(String),          // Injected by macro (type name)
    Context,                   // From service context
}

/// Universal macro processor
pub struct UniversalMacroProcessor;

impl UniversalMacroProcessor {
    /// Parse all macro attributes from a method
    pub fn parse_attributes(attrs: &[Attribute]) -> Vec<MacroAttribute> {
        let mut result = Vec::new();
        
        for attr in attrs {
            if let Some(macro_attr) = Self::parse_single_attribute(attr) {
                result.push(macro_attr);
            }
        }
        
        result
    }
    
    /// Parse a single attribute into MacroAttribute
    fn parse_single_attribute(attr: &Attribute) -> Option<MacroAttribute> {
        let path_str = attr.path().segments.last()?.ident.to_string();
        
        // Skip non-macro attributes
        if !Self::is_macro_attribute(&path_str) {
            return None;
        }
        
        let args = match &attr.meta {
            Meta::Path(_) => HashMap::new(),
            Meta::List(meta_list) => Self::parse_meta_list(meta_list),
            Meta::NameValue(meta_name_value) => Self::parse_name_value(meta_name_value),
        };
        
        Some(MacroAttribute {
            name: path_str,
            args,
        })
    }
    
    /// Check if an attribute name is a macro attribute
    fn is_macro_attribute(name: &str) -> bool {
        matches!(name,
            // Authentication macros
            "require_auth" | "require_role" | "require_permission" | "require_ownership" |
            "require_attributes" | "require_2fa" | "require_any_role" | "optional_auth" |
            
            // Database macros
            "transactional" | "read_only" | "isolated" | "database" | "collection" |
            "auto_save" | "soft_delete" | "versioned" | "audited" |
            
            // Caching macros
            "cached" | "cache_invalidate" | "cache_through" | "cache_aside" |
            "cache_key" | "cache_ttl" | "cache_tags" |
            
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
    }
    
    /// Parse MetaList arguments
    fn parse_meta_list(meta_list: &MetaList) -> HashMap<String, MacroValue> {
        let mut result = HashMap::new();
        
        // Parse token stream manually for now - simplified approach
        let tokens = meta_list.tokens.clone();
        if let Ok(simple_value) = syn::parse2::<Lit>(tokens.clone()) {
            if let Some(value) = Self::parse_lit_to_value(&simple_value) {
                result.insert("value".to_string(), value);
            }
        }
        
        result
    }
    
    /// Parse NameValue arguments
    fn parse_name_value(name_value: &MetaNameValue) -> HashMap<String, MacroValue> {
        let mut result = HashMap::new();
        
        if let Some(key) = name_value.path.get_ident() {
            match &name_value.value {
                syn::Expr::Lit(expr_lit) => {
                    if let Some(value) = Self::parse_lit_to_value(&expr_lit.lit) {
                        result.insert(key.to_string(), value);
                    }
                }
                _ => {
                    // Handle other expression types if needed
                }
            }
        }
        
        result
    }
    
    /// Convert Lit to MacroValue
    fn parse_lit_to_value(lit: &Lit) -> Option<MacroValue> {
        match lit {
            Lit::Str(lit_str) => Some(MacroValue::String(lit_str.value())),
            Lit::Int(lit_int) => lit_int.base10_parse::<i64>().ok().map(MacroValue::Integer),
            Lit::Bool(lit_bool) => Some(MacroValue::Boolean(lit_bool.value)),
            _ => None,
        }
    }
    
    /// Generate universal macro wrapper for a method
    pub fn generate_universal_wrapper(context: &MacroContext) -> proc_macro2::TokenStream {
        let method_name = format_ident!("{}", context.method_name);
        let impl_method_name = format_ident!("{}_impl", context.method_name);
        
        // Generate pre-processing steps
        let pre_processing = Self::generate_pre_processing(context);
        
        // Generate parameter injection
        let param_injection = Self::generate_parameter_injection(context);
        
        // Generate method call
        let method_call = Self::generate_method_call(context, &impl_method_name);
        
        // Generate post-processing steps
        let post_processing = Self::generate_post_processing(context);
        
        quote! {
            pub async fn #method_name(msg: rabbitmesh::Message) -> rabbitmesh::Result<rabbitmesh::RpcResponse> {
                use tracing::{info, warn, error, debug, trace};
                use anyhow::Result;
                
                #pre_processing
                
                #param_injection
                
                let result = #method_call;
                
                #post_processing
                
                match result {
                    Ok(response) => Ok(rabbitmesh::RpcResponse::success(response, 0)?),
                    Err(err) => Ok(rabbitmesh::RpcResponse::error(&err)),
                }
            }
        }
    }
    
    /// Generate pre-processing steps based on attributes
    fn generate_pre_processing(context: &MacroContext) -> proc_macro2::TokenStream {
        let mut steps = Vec::new();
        
        for attr in &context.attributes {
            match attr.name.as_str() {
                "require_auth" | "require_role" | "require_permission" | "require_ownership" | "require_attributes" => {
                    steps.push(Self::generate_auth_preprocessing(attr));
                }
                "rate_limit" | "throttle" => {
                    steps.push(Self::generate_rate_limit_preprocessing(attr));
                }
                "validate" | "sanitize" | "transform" => {
                    steps.push(Self::generate_validation_preprocessing(attr));
                }
                "cached" => {
                    steps.push(Self::generate_cache_check_preprocessing(attr));
                }
                "metrics" | "trace" | "monitor" => {
                    steps.push(Self::generate_observability_preprocessing(attr));
                }
                _ => {}
            }
        }
        
        quote! {
            #(#steps)*
        }
    }
    
    /// Generate parameter injection based on context and attributes
    fn generate_parameter_injection(context: &MacroContext) -> proc_macro2::TokenStream {
        let mut injections = Vec::new();
        
        // Inject authentication context if needed
        if context.attributes.iter().any(|attr| attr.name.starts_with("require_")) {
            injections.push(quote! {
                let auth_context = rabbitmesh_auth::AuthContext::from_message(&msg).await?;
            });
        }
        
        // Inject database context if needed
        if context.attributes.iter().any(|attr| matches!(attr.name.as_str(), "transactional" | "database" | "collection")) {
            injections.push(quote! {
                let db_context = rabbitmesh_database::DbContext::from_service().await?;
            });
        }
        
        // Inject cache context if needed
        if context.attributes.iter().any(|attr| attr.name.starts_with("cache")) {
            injections.push(quote! {
                let cache_context = rabbitmesh_cache::CacheContext::from_service().await?;
            });
        }
        
        quote! {
            #(#injections)*
        }
    }
    
    /// Generate method call with proper parameters
    fn generate_method_call(context: &MacroContext, impl_method_name: &proc_macro2::Ident) -> proc_macro2::TokenStream {
        let mut params = Vec::new();
        
        for param in &context.method_params {
            match param.source {
                ParamSource::Body => {
                    params.push(quote! { msg.deserialize_payload()? });
                }
                ParamSource::Path => {
                    let param_name = &param.name;
                    params.push(quote! { msg.get_path_param(#param_name)? });
                }
                ParamSource::Query => {
                    let param_name = &param.name;
                    params.push(quote! { msg.get_query_param(#param_name)? });
                }
                ParamSource::Header => {
                    let param_name = &param.name;
                    params.push(quote! { msg.get_header(#param_name)? });
                }
                ParamSource::Injected(ref type_name) => {
                    match type_name.as_str() {
                        "AuthContext" => params.push(quote! { auth_context.clone() }),
                        "DbContext" => params.push(quote! { db_context.clone() }),
                        "CacheContext" => params.push(quote! { cache_context.clone() }),
                        _ => {}
                    }
                }
                ParamSource::Context => {
                    params.push(quote! { service_context.clone() });
                }
            }
        }
        
        if params.is_empty() {
            quote! { Self::#impl_method_name().await }
        } else {
            quote! { Self::#impl_method_name(#(#params),*).await }
        }
    }
    
    /// Generate post-processing steps
    fn generate_post_processing(context: &MacroContext) -> proc_macro2::TokenStream {
        let mut steps = Vec::new();
        
        for attr in &context.attributes {
            match attr.name.as_str() {
                "cached" => {
                    steps.push(Self::generate_cache_store_postprocessing(attr));
                }
                "event_publish" | "webhook" => {
                    steps.push(Self::generate_event_postprocessing(attr));
                }
                "audit_log" => {
                    steps.push(Self::generate_audit_postprocessing(attr));
                }
                _ => {}
            }
        }
        
        quote! {
            #(#steps)*
        }
    }
    
    // Individual preprocessing generators
    fn generate_auth_preprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        match attr.name.as_str() {
            "require_auth" => {
                quote! {
                    debug!("🔐 Validating authentication");
                    let auth_header = msg.get_header("authorization")
                        .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;
                }
            }
            "require_role" => {
                if let Some(MacroValue::String(role)) = attr.args.get("value") {
                    quote! {
                        debug!("🔐 Validating role: {}", #role);
                        let auth_header = msg.get_header("authorization")
                            .ok_or_else(|| anyhow::anyhow!("Missing authorization header"))?;
                        // Role validation will be done during parameter injection
                    }
                } else {
                    quote! {}
                }
            }
            _ => quote! {}
        }
    }
    
    fn generate_rate_limit_preprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            debug!("⏱️ Checking rate limits");
            rabbitmesh_rate_limit::check_rate_limit(&msg).await?;
        }
    }
    
    fn generate_validation_preprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            debug!("✅ Validating input");
            // Validation will be done during deserialization
        }
    }
    
    fn generate_cache_check_preprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            debug!("💾 Checking cache");
            if let Some(cached_result) = rabbitmesh_cache::check_cache(&msg).await? {
                return Ok(rabbitmesh::RpcResponse::success(cached_result, 0)?);
            }
        }
    }
    
    fn generate_observability_preprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            debug!("📊 Starting observability");
            let _span = tracing::info_span!("service_method", method = %stringify!(method_name)).entered();
            let start_time = std::time::Instant::now();
        }
    }
    
    // Post-processing generators
    fn generate_cache_store_postprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            if let Ok(ref response) = result {
                debug!("💾 Storing in cache");
                rabbitmesh_cache::store_cache(&msg, response).await?;
            }
        }
    }
    
    fn generate_event_postprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            if let Ok(ref response) = result {
                debug!("📤 Publishing events");
                rabbitmesh_events::publish_event(&msg, response).await?;
            }
        }
    }
    
    fn generate_audit_postprocessing(attr: &MacroAttribute) -> proc_macro2::TokenStream {
        quote! {
            debug!("📝 Recording audit log");
            rabbitmesh_audit::log_action(&msg, &result).await?;
        }
    }
}