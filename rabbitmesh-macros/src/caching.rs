//! Universal Caching and Performance Macros for RabbitMesh
//! 
//! This module provides comprehensive caching and performance macros that work
//! across ALL project types and database systems:
//!
//! **Project Types Supported:**
//! - 🛒 E-commerce (product catalogs, user carts, pricing)
//! - 💰 Finance (trading data, portfolios, market feeds)
//! - 🏥 Healthcare (patient records, medical imaging, lab results)
//! - 🏭 IoT (sensor data, device states, telemetry)
//! - 🎮 Gaming (player profiles, leaderboards, game states)
//! - 📱 Social Media (feeds, notifications, user profiles)
//! - 🏢 Enterprise (dashboards, reports, user permissions)
//! - 🎓 Education (course content, student progress, grades)
//! - 📦 Logistics (shipments, inventory, routes)
//! - 🌍 GIS/Mapping (location data, routes, spatial queries)
//!
//! **Cache Types Supported:**
//! - In-Memory (Redis, Hazelcast, Apache Ignite)
//! - Distributed (Redis Cluster, Memcached, Coherence)
//! - CDN (CloudFlare, AWS CloudFront, Akamai)
//! - Application-Level (LRU, LFU, FIFO)
//! - Database-Level (Query result caching, Connection pooling)

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Attribute, LitInt, LitStr, LitBool};
use std::collections::HashMap;
use crate::universal::{MacroAttribute, MacroValue, MacroContext};

/// Universal caching and performance processor
pub struct CachingProcessor;

impl CachingProcessor {
    /// Generate caching logic for any macro attribute
    pub fn generate_caching_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        match attr.name.as_str() {
            "cached" => Self::generate_cached_logic(attr, context),
            "cache_invalidate" => Self::generate_cache_invalidation(attr, context),
            "cache_through" => Self::generate_cache_through(attr, context),
            "cache_aside" => Self::generate_cache_aside(attr, context),
            "cache_key" => Self::generate_cache_key_logic(attr, context),
            "cache_ttl" => Self::generate_cache_ttl_logic(attr, context), 
            "cache_tags" => Self::generate_cache_tags_logic(attr, context),
            "multi_level_cache" => Self::generate_multi_level_cache(attr, context),
            "cache_warm_up" => Self::generate_cache_warm_up(attr, context),
            "cache_partition" => Self::generate_cache_partition(attr, context),
            _ => quote! {}
        }
    }

    /// Generate main cached logic with universal support
    fn generate_cached_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let cache_key = Self::extract_cache_key(attr, context);
        let ttl = Self::extract_ttl(attr).unwrap_or(3600); // Default 1 hour
        let cache_type = Self::extract_cache_type(attr);
        let domain_optimization = Self::generate_domain_cache_optimization(context, attr);
        
        quote! {
            // Universal cache check - works with any cache backend
            let cache_key = #cache_key;
            debug!("💾 Checking cache with key: {}", cache_key);
            
            #domain_optimization
            
            // Try to get from cache first
            if let Some(cached_value) = rabbitmesh_cache::get_universal_cache(&cache_key, #cache_type).await? {
                debug!("✅ Cache hit for key: {}", cache_key);
                return Ok(rabbitmesh::RpcResponse::success(cached_value, 0)?);
            }
            
            debug!("❌ Cache miss for key: {}", cache_key);
        }
    }

    /// Generate cache invalidation logic
    fn generate_cache_invalidation(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let invalidation_patterns = Self::extract_invalidation_patterns(attr);
        let domain_patterns = Self::generate_domain_invalidation_patterns(context);
        
        quote! {
            // Universal cache invalidation
            debug!("🗑️ Invalidating cache patterns");
            
            #domain_patterns
            
            let invalidation_patterns = vec![#(#invalidation_patterns),*];
            for pattern in invalidation_patterns {
                rabbitmesh_cache::invalidate_pattern(&pattern).await?;
                debug!("🗑️ Invalidated pattern: {}", pattern);
            }
        }
    }

    /// Generate cache-through pattern
    fn generate_cache_through(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let cache_key = Self::extract_cache_key(attr, context);
        let ttl = Self::extract_ttl(attr).unwrap_or(3600);
        
        quote! {
            // Cache-through pattern - always update cache with result
            let cache_key = #cache_key;
            
            // This will be called after method execution
            let cache_result = |result: &Result<_, String>| async move {
                if let Ok(success_result) = result {
                    debug!("💾 Storing result in cache (cache-through): {}", cache_key);
                    rabbitmesh_cache::set_universal_cache(&cache_key, success_result, #ttl).await?;
                }
                Ok(())
            };
        }
    }

    /// Generate cache-aside pattern  
    fn generate_cache_aside(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let cache_key = Self::extract_cache_key(attr, context);
        let ttl = Self::extract_ttl(attr).unwrap_or(3600);
        
        quote! {
            // Cache-aside pattern - application manages cache
            let cache_key = #cache_key;
            
            // Check cache first
            if let Some(cached_value) = rabbitmesh_cache::get_universal_cache(&cache_key, "application").await? {
                debug!("✅ Cache-aside hit: {}", cache_key);
                return Ok(rabbitmesh::RpcResponse::success(cached_value, 0)?);
            }
            
            // Will cache result after method execution if successful
            let should_cache_result = true;
        }
    }

    /// Generate multi-level caching
    fn generate_multi_level_cache(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let cache_key = Self::extract_cache_key(attr, context);
        let l1_ttl = Self::extract_arg_as_int(attr, "l1_ttl").unwrap_or(300); // 5 min
        let l2_ttl = Self::extract_arg_as_int(attr, "l2_ttl").unwrap_or(3600); // 1 hour
        let l3_ttl = Self::extract_arg_as_int(attr, "l3_ttl").unwrap_or(86400); // 1 day
        
        quote! {
            // Multi-level cache hierarchy
            let cache_key = #cache_key;
            debug!("💾 Checking multi-level cache: {}", cache_key);
            
            // Level 1: In-memory cache (fastest)
            if let Some(cached_value) = rabbitmesh_cache::get_l1_cache(&cache_key).await? {
                debug!("🚀 L1 Cache hit: {}", cache_key);
                return Ok(rabbitmesh::RpcResponse::success(cached_value, 0)?);
            }
            
            // Level 2: Distributed cache (Redis)
            if let Some(cached_value) = rabbitmesh_cache::get_l2_cache(&cache_key).await? {
                debug!("⚡ L2 Cache hit: {}", cache_key);
                
                // Promote to L1 cache
                rabbitmesh_cache::set_l1_cache(&cache_key, &cached_value, #l1_ttl).await?;
                return Ok(rabbitmesh::RpcResponse::success(cached_value, 0)?);
            }
            
            // Level 3: Persistent cache (Database/File)
            if let Some(cached_value) = rabbitmesh_cache::get_l3_cache(&cache_key).await? {
                debug!("💿 L3 Cache hit: {}", cache_key);
                
                // Promote to L2 and L1
                rabbitmesh_cache::set_l2_cache(&cache_key, &cached_value, #l2_ttl).await?;
                rabbitmesh_cache::set_l1_cache(&cache_key, &cached_value, #l1_ttl).await?;
                return Ok(rabbitmesh::RpcResponse::success(cached_value, 0)?);
            }
            
            debug!("❌ All cache levels missed: {}", cache_key);
        }
    }

    /// Generate cache warming logic
    fn generate_cache_warm_up(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let warm_up_strategy = Self::extract_arg_as_string(attr, "strategy").unwrap_or_else(|| "lazy".to_string());
        let domain_warm_up = Self::generate_domain_cache_warm_up(context);
        
        quote! {
            // Cache warm-up strategy
            debug!("🔥 Initiating cache warm-up strategy: {}", #warm_up_strategy);
            
            #domain_warm_up
            
            match #warm_up_strategy.as_str() {
                "eager" => {
                    // Pre-populate cache with commonly accessed data
                    rabbitmesh_cache::warm_up_eager_cache(service_context).await?;
                },
                "lazy" => {
                    // Warm up cache as requests come in
                    rabbitmesh_cache::warm_up_lazy_cache().await?;
                },
                "scheduled" => {
                    // Background cache warming
                    rabbitmesh_cache::schedule_cache_warm_up().await?;
                },
                _ => {}
            }
        }
    }

    /// Generate cache key logic
    fn generate_cache_key_logic(_attr: &MacroAttribute, _context: &MacroContext) -> TokenStream {
        quote! {
            // Cache key configuration handled in main caching logic
            debug!("🔑 Cache key configuration processed");
        }
    }

    /// Generate cache TTL logic
    fn generate_cache_ttl_logic(_attr: &MacroAttribute, _context: &MacroContext) -> TokenStream {
        quote! {
            // Cache TTL configuration handled in main caching logic
            debug!("⏰ Cache TTL configuration processed");
        }
    }

    /// Generate cache tags logic
    fn generate_cache_tags_logic(_attr: &MacroAttribute, _context: &MacroContext) -> TokenStream {
        quote! {
            // Cache tags configuration handled in main caching logic  
            debug!("🏷️ Cache tags configuration processed");
        }
    }

    /// Generate cache partitioning logic
    fn generate_cache_partition(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let partition_key = Self::extract_arg_as_string(attr, "partition_key").unwrap_or_else(|| "user_id".to_string());
        let partition_count = Self::extract_arg_as_int(attr, "partitions").unwrap_or(16);
        
        quote! {
            // Cache partitioning for better distribution
            let partition_key_value = msg.get_header(#partition_key)
                .or_else(|| msg.get_query_param(#partition_key))
                .unwrap_or_else(|| "default".to_string());
                
            let partition_id = rabbitmesh_cache::calculate_partition(&partition_key_value, #partition_count);
            debug!("🗂️ Using cache partition: {} (key: {})", partition_id, partition_key_value);
        }
    }

    /// Generate domain-specific cache optimizations
    fn generate_domain_cache_optimization(context: &MacroContext, attr: &MacroAttribute) -> TokenStream {
        let service_name = &context.service_name;
        let method_name = &context.method_name;
        
        // Try to infer domain from service name or method patterns
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce specific cache optimizations
                if #method_name.contains("product") || #method_name.contains("catalog") {
                    // Product catalogs benefit from longer TTL and CDN caching
                    let cache_config = rabbitmesh_cache::EcommerceCacheConfig::product_catalog();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("cart") || #method_name.contains("session") {
                    // Cart data needs shorter TTL and user-specific partitioning
                    let cache_config = rabbitmesh_cache::EcommerceCacheConfig::user_session();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("price") || #method_name.contains("inventory") {
                    // Pricing data needs real-time invalidation
                    let cache_config = rabbitmesh_cache::EcommerceCacheConfig::dynamic_pricing();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "finance" => quote! {
                // Finance specific cache optimizations
                if #method_name.contains("market") || #method_name.contains("quote") {
                    // Market data needs very short TTL (seconds)
                    let cache_config = rabbitmesh_cache::FinanceCacheConfig::market_data();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("portfolio") || #method_name.contains("account") {
                    // Portfolio data can be cached longer with user partitioning
                    let cache_config = rabbitmesh_cache::FinanceCacheConfig::user_portfolio();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("trade") || #method_name.contains("order") {
                    // Trading data needs high-frequency caching
                    let cache_config = rabbitmesh_cache::FinanceCacheConfig::trading_data();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "healthcare" => quote! {
                // Healthcare specific cache optimizations
                if #method_name.contains("patient") || #method_name.contains("medical") {
                    // Patient data needs secure caching with encryption
                    let cache_config = rabbitmesh_cache::HealthcareCacheConfig::patient_data();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("schedule") || #method_name.contains("appointment") {
                    // Scheduling data benefits from time-based invalidation
                    let cache_config = rabbitmesh_cache::HealthcareCacheConfig::scheduling();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "iot" => quote! {
                // IoT specific cache optimizations
                if #method_name.contains("sensor") || #method_name.contains("telemetry") {
                    // Sensor data benefits from time-series caching
                    let cache_config = rabbitmesh_cache::IoTCacheConfig::sensor_data();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("device") || #method_name.contains("status") {
                    // Device status needs real-time caching
                    let cache_config = rabbitmesh_cache::IoTCacheConfig::device_status();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "gaming" => quote! {
                // Gaming specific cache optimizations
                if #method_name.contains("player") || #method_name.contains("profile") {
                    // Player profiles benefit from session-based caching
                    let cache_config = rabbitmesh_cache::GamingCacheConfig::player_profile();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("leaderboard") || #method_name.contains("score") {
                    // Leaderboards need frequent updates but can be cached briefly
                    let cache_config = rabbitmesh_cache::GamingCacheConfig::leaderboard();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("match") || #method_name.contains("game_state") {
                    // Game state needs real-time caching
                    let cache_config = rabbitmesh_cache::GamingCacheConfig::game_state();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "social" => quote! {
                // Social media specific cache optimizations
                if #method_name.contains("feed") || #method_name.contains("timeline") {
                    // Social feeds benefit from personalized caching
                    let cache_config = rabbitmesh_cache::SocialCacheConfig::user_feed();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("notification") {
                    // Notifications need real-time caching
                    let cache_config = rabbitmesh_cache::SocialCacheConfig::notifications();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            "logistics" => quote! {
                // Logistics specific cache optimizations
                if #method_name.contains("route") || #method_name.contains("delivery") {
                    // Route data benefits from geographic caching
                    let cache_config = rabbitmesh_cache::LogisticsCacheConfig::route_optimization();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                } else if #method_name.contains("inventory") || #method_name.contains("warehouse") {
                    // Inventory data needs location-based caching
                    let cache_config = rabbitmesh_cache::LogisticsCacheConfig::inventory_tracking();
                    rabbitmesh_cache::apply_cache_config(cache_config).await?;
                }
            },
            _ => quote! {
                // Generic cache optimization
                debug!("🔧 Applying generic cache optimization");
            }
        }
    }

    /// Generate domain-specific cache invalidation patterns
    fn generate_domain_invalidation_patterns(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                let mut domain_invalidation_patterns = Vec::new();
                
                if #method_name.contains("update_product") || #method_name.contains("modify_product") {
                    domain_invalidation_patterns.extend(vec![
                        "product_catalog:*".to_string(),
                        "product_search:*".to_string(),
                        "category_products:*".to_string()
                    ]);
                }
                
                if #method_name.contains("update_inventory") {
                    domain_invalidation_patterns.extend(vec![
                        "inventory:*".to_string(),
                        "product_availability:*".to_string()
                    ]);
                }
                
                if #method_name.contains("update_price") {
                    domain_invalidation_patterns.extend(vec![
                        "pricing:*".to_string(),
                        "product_details:*".to_string(),
                        "cart_totals:*".to_string()
                    ]);
                }
                
                for pattern in domain_invalidation_patterns {
                    rabbitmesh_cache::invalidate_pattern(&pattern).await?;
                }
            },
            "finance" => quote! {
                let mut domain_invalidation_patterns = Vec::new();
                
                if #method_name.contains("execute_trade") || #method_name.contains("place_order") {
                    domain_invalidation_patterns.extend(vec![
                        "portfolio:*".to_string(),
                        "account_balance:*".to_string(),
                        "trading_history:*".to_string()
                    ]);
                }
                
                if #method_name.contains("market_update") {
                    domain_invalidation_patterns.extend(vec![
                        "market_data:*".to_string(),
                        "quotes:*".to_string(),
                        "charts:*".to_string()
                    ]);
                }
                
                for pattern in domain_invalidation_patterns {
                    rabbitmesh_cache::invalidate_pattern(&pattern).await?;
                }
            },
            "gaming" => quote! {
                let mut domain_invalidation_patterns = Vec::new();
                
                if #method_name.contains("update_score") || #method_name.contains("complete_match") {
                    domain_invalidation_patterns.extend(vec![
                        "leaderboard:*".to_string(),
                        "player_stats:*".to_string(),
                        "rankings:*".to_string()
                    ]);
                }
                
                if #method_name.contains("update_profile") {
                    domain_invalidation_patterns.extend(vec![
                        "player_profile:*".to_string(),
                        "friend_lists:*".to_string()
                    ]);
                }
                
                for pattern in domain_invalidation_patterns {
                    rabbitmesh_cache::invalidate_pattern(&pattern).await?;
                }
            },
            _ => quote! {
                debug!("🔧 Using generic invalidation patterns");
            }
        }
    }

    /// Generate domain-specific cache warm-up strategies
    fn generate_domain_cache_warm_up(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce cache warm-up
                tokio::spawn(async {
                    // Pre-cache popular products
                    rabbitmesh_cache::warm_up_popular_products().await?;
                    
                    // Pre-cache category pages
                    rabbitmesh_cache::warm_up_category_pages().await?;
                    
                    // Pre-cache search results for common queries
                    rabbitmesh_cache::warm_up_search_results().await?;
                    
                    Ok::<(), anyhow::Error>(())
                });
            },
            "finance" => quote! {
                // Finance cache warm-up
                tokio::spawn(async {
                    // Pre-cache market data
                    rabbitmesh_cache::warm_up_market_data().await?;
                    
                    // Pre-cache popular stock quotes
                    rabbitmesh_cache::warm_up_stock_quotes().await?;
                    
                    // Pre-cache economic indicators
                    rabbitmesh_cache::warm_up_economic_data().await?;
                    
                    Ok::<(), anyhow::Error>(())
                });
            },
            "social" => quote! {
                // Social media cache warm-up
                tokio::spawn(async {
                    // Pre-cache trending topics
                    rabbitmesh_cache::warm_up_trending_topics().await?;
                    
                    // Pre-cache popular users' profiles
                    rabbitmesh_cache::warm_up_popular_profiles().await?;
                    
                    // Pre-cache recent posts from followed users
                    rabbitmesh_cache::warm_up_social_feeds().await?;
                    
                    Ok::<(), anyhow::Error>(())
                });
            },
            _ => quote! {
                debug!("🔧 Generic cache warm-up strategy");
            }
        }
    }

    // Helper methods for extracting cache configuration

    /// Extract cache key from attribute
    fn extract_cache_key(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        if let Some(MacroValue::String(key_template)) = attr.args.get("key") {
            // Dynamic key generation based on template
            Self::generate_dynamic_cache_key(key_template, context)
        } else {
            // Default key generation
            let service = &context.service_name;
            let method = &context.method_name;
            quote! {
                format!("{}:{}:{}", #service, #method, 
                    rabbitmesh_cache::generate_param_hash(&msg))
            }
        }
    }

    /// Generate dynamic cache key from template
    fn generate_dynamic_cache_key(template: &str, context: &MacroContext) -> TokenStream {
        // Replace template variables with actual values
        // e.g., "user_{user_id}" -> format!("user_{}", user_id)
        let mut key_parts = Vec::new();
        let mut current_part = String::new();
        let mut in_variable = false;
        let mut variable_name = String::new();

        for c in template.chars() {
            match c {
                '{' => {
                    if !current_part.is_empty() {
                        key_parts.push(quote! { #current_part });
                        current_part.clear();
                    }
                    in_variable = true;
                    variable_name.clear();
                }
                '}' => {
                    if in_variable {
                        // Generate code to extract the variable value
                        key_parts.push(quote! {
                            msg.get_path_param(#variable_name)
                                .or_else(|| msg.get_query_param(#variable_name))
                                .or_else(|| msg.get_header(#variable_name))
                                .unwrap_or_else(|| "unknown".to_string())
                        });
                        in_variable = false;
                    }
                }
                _ => {
                    if in_variable {
                        variable_name.push(c);
                    } else {
                        current_part.push(c);
                    }
                }
            }
        }

        if !current_part.is_empty() {
            key_parts.push(quote! { #current_part });
        }

        quote! {
            format!("{}", vec![#(#key_parts),*].join(""))
        }
    }

    /// Extract TTL from attribute
    fn extract_ttl(attr: &MacroAttribute) -> Option<i64> {
        match attr.args.get("ttl") {
            Some(MacroValue::Integer(ttl)) => Some(*ttl),
            Some(MacroValue::String(ttl_str)) => {
                // Parse time strings like "5m", "1h", "30s"
                Self::parse_duration_string(ttl_str)
            }
            _ => None
        }
    }

    /// Parse duration strings into seconds
    fn parse_duration_string(duration: &str) -> Option<i64> {
        let duration = duration.trim().to_lowercase();
        
        if let Some(num_part) = duration.chars().take_while(|c| c.is_numeric()).collect::<String>().parse::<i64>().ok() {
            let suffix = duration.chars().skip_while(|c| c.is_numeric()).collect::<String>();
            
            match suffix.as_str() {
                "s" | "sec" | "second" | "seconds" => Some(num_part),
                "m" | "min" | "minute" | "minutes" => Some(num_part * 60),
                "h" | "hour" | "hours" => Some(num_part * 3600),
                "d" | "day" | "days" => Some(num_part * 86400),
                _ => Some(num_part) // Default to seconds
            }
        } else {
            None
        }
    }

    /// Extract cache type from attribute
    fn extract_cache_type(attr: &MacroAttribute) -> TokenStream {
        if let Some(MacroValue::String(cache_type)) = attr.args.get("type") {
            quote! { #cache_type }
        } else {
            quote! { "redis" }
        }
    }

    /// Extract invalidation patterns
    fn extract_invalidation_patterns(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut patterns = Vec::new();
        
        if let Some(MacroValue::Array(pattern_array)) = attr.args.get("invalidate_on") {
            for pattern in pattern_array {
                if let MacroValue::String(pattern_str) = pattern {
                    patterns.push(quote! { #pattern_str.to_string() });
                }
            }
        }
        
        patterns
    }

    /// Extract argument as string
    fn extract_arg_as_string(attr: &MacroAttribute, key: &str) -> Option<String> {
        if let Some(MacroValue::String(value)) = attr.args.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    /// Extract argument as integer
    fn extract_arg_as_int(attr: &MacroAttribute, key: &str) -> Option<i64> {
        if let Some(MacroValue::Integer(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Infer domain from context
    fn infer_domain_from_context(context: &MacroContext) -> String {
        let service_name = context.service_name.to_lowercase();
        let method_name = context.method_name.to_lowercase();
        
        // Check service name patterns
        if service_name.contains("ecommerce") || service_name.contains("shop") || service_name.contains("product") || service_name.contains("cart") {
            return "ecommerce".to_string();
        }
        if service_name.contains("finance") || service_name.contains("trading") || service_name.contains("bank") || service_name.contains("payment") {
            return "finance".to_string();
        }
        if service_name.contains("health") || service_name.contains("medical") || service_name.contains("patient") {
            return "healthcare".to_string();
        }
        if service_name.contains("iot") || service_name.contains("sensor") || service_name.contains("device") {
            return "iot".to_string();
        }
        if service_name.contains("game") || service_name.contains("gaming") || service_name.contains("player") {
            return "gaming".to_string();
        }
        if service_name.contains("social") || service_name.contains("media") || service_name.contains("feed") {
            return "social".to_string();
        }
        if service_name.contains("logistics") || service_name.contains("shipping") || service_name.contains("delivery") {
            return "logistics".to_string();
        }
        
        // Check method name patterns
        if method_name.contains("product") || method_name.contains("cart") || method_name.contains("order") {
            return "ecommerce".to_string();
        }
        if method_name.contains("trade") || method_name.contains("portfolio") || method_name.contains("account") {
            return "finance".to_string();
        }
        if method_name.contains("patient") || method_name.contains("medical") || method_name.contains("health") {
            return "healthcare".to_string();
        }
        if method_name.contains("sensor") || method_name.contains("device") || method_name.contains("telemetry") {
            return "iot".to_string();
        }
        if method_name.contains("player") || method_name.contains("game") || method_name.contains("score") {
            return "gaming".to_string();
        }
        if method_name.contains("post") || method_name.contains("feed") || method_name.contains("social") {
            return "social".to_string();
        }
        if method_name.contains("ship") || method_name.contains("deliver") || method_name.contains("route") {
            return "logistics".to_string();
        }
        
        "generic".to_string()
    }
}

/// Cache configuration traits for domain-specific optimizations
pub trait DomainCacheConfig {
    fn get_ttl(&self) -> i64;
    fn get_cache_type(&self) -> &str;
    fn get_partitioning_strategy(&self) -> &str;
    fn get_invalidation_patterns(&self) -> Vec<String>;
}

/// Performance optimization macros
impl CachingProcessor {
    /// Generate performance monitoring
    pub fn generate_performance_monitoring(_attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let method_name = &context.method_name;
        quote! {
            let perf_start = std::time::Instant::now();
            let method_name = #method_name;
            
            // This will be used after method execution
            let record_performance = |duration: std::time::Duration, success: bool| async move {
                rabbitmesh_performance::record_method_performance(
                    method_name,
                    duration,
                    success
                ).await;
            };
        }
    }

    /// Generate batch processing optimization
    pub fn generate_batch_processing(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let batch_size = Self::extract_arg_as_int(attr, "batch_size").unwrap_or(100);
        let timeout = Self::extract_arg_as_int(attr, "timeout").unwrap_or(5000);
        
        quote! {
            // Batch processing optimization
            debug!("📦 Initiating batch processing (size: {}, timeout: {}ms)", #batch_size, #timeout);
            
            let batch_processor = rabbitmesh_performance::BatchProcessor::new(#batch_size, #timeout);
            
            // Check if this request can be batched
            if let Some(batch_result) = batch_processor.try_batch_request(&msg).await? {
                debug!("📦 Request processed in batch");
                return Ok(rabbitmesh::RpcResponse::success(batch_result, 0)?);
            }
            
            // Process individually but collect for future batching
            batch_processor.collect_for_batching(&msg).await?;
        }
    }

    /// Generate async pool optimization
    pub fn generate_async_pool(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let pool_size = Self::extract_arg_as_int(attr, "pool_size").unwrap_or(10);
        
        quote! {
            // Async pool optimization
            debug!("🏊‍♂️ Using async pool (size: {})", #pool_size);
            
            let pool_handle = rabbitmesh_performance::AsyncPool::get_or_create(#pool_size).await;
            
            // Execute method in pool
            let pool_result = pool_handle.execute(async move {
                // Method will be executed here
            }).await?;
        }
    }
}