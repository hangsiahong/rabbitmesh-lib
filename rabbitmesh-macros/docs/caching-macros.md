# Caching Macros

Multi-level caching with Redis, in-memory, distributed cache coordination, and intelligent cache invalidation for high-performance microservices.

## Overview

Caching macros provide sophisticated caching strategies that dramatically improve performance while maintaining data consistency. They support multiple cache levels, intelligent invalidation, and distributed coordination.

---

## Basic Caching Macros

### `#[cached(ttl)]`

**Purpose**: Caches method results with automatic expiration and key generation.

**Usage**:
```rust
#[service_method("GET /users/:id")]
#[require_auth]
#[cached(ttl = 300)] // Cache for 5 minutes
pub async fn get_user_profile(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    
    // This method will be cached automatically:
    // - Cache key: generated from method name + parameters
    // - TTL: 300 seconds
    // - Storage: Default cache backend (Redis/in-memory)
    
    let user = fetch_user_from_database(user_id).await?;
    Ok(RpcResponse::success(&user, 0)?)
}

#[service_method("GET /products")]
#[cached(ttl = "1h")] // Cache for 1 hour
pub async fn list_products(msg: Message) -> Result<RpcResponse, String> {
    let category = msg.get_query_param("category").unwrap_or_default();
    let limit = msg.get_query_param("limit").unwrap_or("10".to_string());
    
    // Cache key includes query parameters automatically
    let products = fetch_products_by_category(&category, limit.parse()?).await?;
    Ok(RpcResponse::success(&products, 0)?)
}
```

**TTL Formats**:
- **Seconds**: `300`, `"300s"`
- **Minutes**: `"5m"`, `"2.5m"`
- **Hours**: `"1h"`, `"24h"`
- **Days**: `"7d"`

**Cache Key Generation**:
- Method name + parameter hash
- Query parameters included
- User context (if authenticated)
- Custom key patterns supported

**When to Use**: For frequently accessed data with acceptable staleness.

---

### `#[cached(key, ttl)]`

**Purpose**: Custom cache key generation with flexible TTL configuration.

**Usage**:
```rust
#[service_method("GET /orders/:id")]
#[require_auth]
#[cached(
    key = "order:{order_id}:user:{user_id}",
    ttl = "15m"
)]
pub async fn get_order_details(msg: Message) -> Result<RpcResponse, String> {
    let order_id = msg.get_path_param("id")?;
    let user_id = msg.get_user_id()?;
    
    // Custom cache key: "order:123:user:456"
    // Allows for fine-grained cache control
    
    let order = fetch_order_with_details(order_id, user_id).await?;
    Ok(RpcResponse::success(&order, 0)?)
}

#[service_method("GET /analytics/dashboard")]
#[require_auth]
#[require_role("manager")]
#[cached(
    key = "dashboard:{user_id}:{date}",
    ttl = "30m",
    vary_by = ["user_role", "tenant_id"]
)]
pub async fn get_dashboard_data(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_user_id()?;
    let date = msg.get_query_param("date").unwrap_or_default();
    
    // Cache varies by user role and tenant
    let dashboard = generate_dashboard_data(user_id, &date).await?;
    Ok(RpcResponse::success(&dashboard, 0)?)
}
```

**Key Template Variables**:
- `{user_id}`, `{user_role}`, `{tenant_id}`
- `{method_name}`, `{service_name}`
- Path parameters: `{order_id}`, `{product_id}`
- Query parameters: `{category}`, `{limit}`
- Custom variables from request context

**When to Use**: When you need specific cache key patterns or cache segmentation.

---

## Multi-Level Caching Macros

### `#[cached_multi_level]`

**Purpose**: Implements hierarchical caching with L1 (in-memory), L2 (Redis), and L3 (database) levels.

**Usage**:
```rust
#[service_method("GET /products/:id")]
#[cached_multi_level(
    l1 = { ttl = "1m", max_size = 1000 },    // In-memory cache
    l2 = { ttl = "15m", backend = "redis" },  // Redis cache
    l3 = { ttl = "1h", backend = "database" } // Database cache
)]
pub async fn get_product_details(msg: Message) -> Result<RpcResponse, String> {
    let product_id = msg.get_path_param("id")?;
    
    // Cache lookup hierarchy:
    // 1. Check L1 (in-memory) - fastest
    // 2. Check L2 (Redis) - fast
    // 3. Check L3 (database cache) - medium
    // 4. Execute method - slowest
    // 
    // On cache hit at any level, populate higher levels
    
    let product = fetch_product_with_full_details(product_id).await?;
    Ok(RpcResponse::success(&product, 0)?)
}

#[service_method("GET /search")]
#[cached_multi_level(
    l1 = { ttl = "30s", max_size = 500 },
    l2 = { ttl = "5m" },
    write_through = true
)]
pub async fn search_products(msg: Message) -> Result<RpcResponse, String> {
    let query = msg.get_query_param("q").unwrap_or_default();
    let filters = msg.get_query_param("filters").unwrap_or_default();
    
    // Multi-level caching with write-through:
    // - Results written to all cache levels simultaneously
    // - Ensures consistency across cache levels
    
    let results = perform_product_search(&query, &filters).await?;
    Ok(RpcResponse::success(&results, 0)?)
}
```

**Multi-Level Features**:
- **Cache Promotion**: Cache hits promote data to higher levels
- **Write-Through**: Optional write-through to all levels
- **Write-Behind**: Asynchronous cache population
- **Cache Coherence**: Automatic invalidation across levels

**When to Use**: For frequently accessed data requiring optimal performance across different access patterns.

---

### `#[cached_distributed]`

**Purpose**: Distributed caching with coordination across multiple service instances.

**Usage**:
```rust
#[service_method("GET /global-config")]
#[cached_distributed(
    backend = "redis_cluster",
    ttl = "1h",
    consistency = "eventual",
    invalidation_channel = "config_updates"
)]
pub async fn get_global_configuration(msg: Message) -> Result<RpcResponse, String> {
    // Distributed caching across all service instances:
    // - Shared cache in Redis cluster
    // - Cache invalidation via pub/sub
    // - Eventual consistency model
    
    let config = load_global_configuration().await?;
    Ok(RpcResponse::success(&config, 0)?)
}

#[service_method("GET /feature-flags")]
#[cached_distributed(
    backend = "redis_cluster",
    ttl = "10m",
    consistency = "strong",
    partition_key = "tenant_id"
)]
pub async fn get_feature_flags(msg: Message) -> Result<RpcResponse, String> {
    let tenant_id = msg.get_tenant_id()?;
    
    // Strong consistency with partitioned caching:
    // - Cache partitioned by tenant
    // - Strong consistency guarantees
    // - Automatic partition management
    
    let flags = load_feature_flags_for_tenant(tenant_id).await?;
    Ok(RpcResponse::success(&flags, 0)?)
}
```

**Distributed Caching Features**:
- **Cluster Support**: Redis Cluster, Hazelcast, Apache Ignite
- **Consistency Models**: Strong, eventual, session consistency
- **Partitioning**: Automatic data partitioning
- **Replication**: Cross-datacenter replication
- **Invalidation**: Distributed cache invalidation

**When to Use**: For applications requiring shared cache state across multiple instances.

---

## Cache Invalidation Macros

### `#[cache_invalidate(pattern)]`

**Purpose**: Invalidates cache entries based on patterns when data is modified.

**Usage**:
```rust
#[service_method("PUT /users/:id")]
#[require_auth]
#[require_ownership]
#[validate]
#[cache_invalidate("user:{id}*")]
pub async fn update_user_profile(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let updates: UserProfileUpdate = msg.deserialize_payload()?;
    
    // Update user profile
    let updated_user = update_user_in_database(user_id, updates).await?;
    
    // Cache invalidation automatically triggered:
    // - Invalidates all keys matching "user:123*"
    // - Includes profile, preferences, settings, etc.
    
    Ok(RpcResponse::success(&updated_user, 0)?)
}

#[service_method("POST /orders")]
#[require_auth]
#[validate]
#[cache_invalidate([
    "user:{user_id}:orders*",
    "dashboard:{user_id}*",
    "analytics:orders*"
])]
pub async fn create_order(msg: Message) -> Result<RpcResponse, String> {
    let order_data: CreateOrderRequest = msg.deserialize_payload()?;
    let user_id = msg.get_user_id()?;
    
    let order = create_new_order(order_data).await?;
    
    // Multiple cache patterns invalidated:
    // - User's order history
    // - User's dashboard data  
    // - Global order analytics
    
    Ok(RpcResponse::success(&order, 0)?)
}
```

**Invalidation Patterns**:
- **Exact Match**: `"user:123"`
- **Prefix Match**: `"user:123*"`
- **Wildcard Match**: `"user:*:orders"`
- **Tag-Based**: `"tag:user_data,order_data"`

**When to Use**: For maintaining cache consistency when data is modified.

---

### `#[cache_tags(tags)]`

**Purpose**: Tags cache entries for intelligent invalidation by logical groupings.

**Usage**:
```rust
#[service_method("GET /users/:id")]
#[require_auth]
#[cached(ttl = "15m")]
#[cache_tags(["user_profile", "user_{id}"])]
pub async fn get_user_profile(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    
    // Cache tagged with:
    // - "user_profile" (generic tag)
    // - "user_123" (specific user tag)
    
    let profile = fetch_user_profile(user_id).await?;
    Ok(RpcResponse::success(&profile, 0)?)
}

#[service_method("GET /users/:id/orders")]
#[require_auth]
#[cached(ttl = "5m")]
#[cache_tags(["user_orders", "user_{id}", "orders"])]
pub async fn get_user_orders(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    
    let orders = fetch_user_orders(user_id).await?;
    Ok(RpcResponse::success(&orders, 0)?)
}

#[service_method("PUT /users/:id")]
#[require_auth]
#[validate]
#[cache_invalidate_tags(["user_{id}", "user_profile"])]
pub async fn update_user(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let updates: UserUpdate = msg.deserialize_payload()?;
    
    let updated_user = update_user_profile(user_id, updates).await?;
    
    // Invalidates all cache entries tagged with:
    // - "user_123" (all data for this user)
    // - "user_profile" (all user profile caches)
    
    Ok(RpcResponse::success(&updated_user, 0)?)
}
```

**Cache Tagging Benefits**:
- **Logical Grouping**: Group related cache entries
- **Batch Invalidation**: Invalidate multiple related entries
- **Flexible Invalidation**: Invalidate by different criteria
- **Dependency Tracking**: Track cache dependencies

**When to Use**: For complex cache invalidation scenarios with related data.

---

## Advanced Caching Strategies

### `#[cache_aside]`

**Purpose**: Implements cache-aside pattern with explicit cache management.

**Usage**:
```rust
#[service_method("GET /reports/:id")]
#[require_auth]
#[cache_aside(
    cache_key = "report:{id}",
    ttl = "2h",
    miss_handler = "generate_report_handler"
)]
pub async fn get_report(msg: Message) -> Result<RpcResponse, String> {
    let report_id = msg.get_path_param("id")?;
    
    // Cache-aside pattern:
    // 1. Check cache first
    // 2. If miss, call miss_handler
    // 3. Cache the result
    // 4. Return result
    
    Ok(RpcResponse::success(&"cached or generated report", 0)?)
}

async fn generate_report_handler(report_id: String) -> Result<Report, ReportError> {
    // Expensive report generation
    generate_complex_report(&report_id).await
}

#[service_method("POST /reports")]
#[require_auth]
#[validate]
#[cache_write_around("report:{id}")]
pub async fn create_report(msg: Message) -> Result<RpcResponse, String> {
    let report_data: CreateReportRequest = msg.deserialize_payload()?;
    
    // Write-around pattern:
    // - Write directly to database
    // - Don't populate cache (avoid cache pollution)
    // - Cache will be populated on first read
    
    let report = create_new_report(report_data).await?;
    Ok(RpcResponse::success(&report, 0)?)
}
```

**When to Use**: For expensive computations or when you need explicit control over cache behavior.

---

### `#[cache_write_through]`

**Purpose**: Implements write-through caching where writes go to both cache and storage.

**Usage**:
```rust
#[service_method("PUT /user-preferences/:id")]
#[require_auth]
#[require_ownership]
#[validate]
#[cache_write_through(
    cache_key = "preferences:{id}",
    ttl = "24h",
    backends = ["redis", "database"]
)]
pub async fn update_user_preferences(msg: Message) -> Result<RpcResponse, String> {
    let user_id = msg.get_path_param("id")?;
    let preferences: UserPreferences = msg.deserialize_payload()?;
    
    // Write-through pattern:
    // 1. Write to database
    // 2. Write to cache
    // 3. Both writes must succeed
    // 4. Strong consistency guaranteed
    
    let updated_preferences = update_preferences_in_db(&user_id, preferences).await?;
    
    Ok(RpcResponse::success(&updated_preferences, 0)?)
}

#[service_method("POST /user-settings")]
#[require_auth]
#[validate]
#[cache_write_behind(
    cache_key = "settings:{user_id}",
    ttl = "12h",
    write_delay = "30s"
)]
pub async fn update_user_settings(msg: Message) -> Result<RpcResponse, String> {
    let settings: UserSettings = msg.deserialize_payload()?;
    let user_id = msg.get_user_id()?;
    
    // Write-behind pattern:
    // 1. Write to cache immediately
    // 2. Return success to user
    // 3. Asynchronously write to database after delay
    // 4. Better performance, eventual consistency
    
    Ok(RpcResponse::success(&"Settings updated", 0)?)
}
```

**Write Patterns**:
- **Write-Through**: Synchronous writes to cache and storage
- **Write-Behind**: Asynchronous writes to storage
- **Write-Around**: Bypass cache on writes

**When to Use**: For frequently updated data requiring different consistency guarantees.

---

### `#[cache_read_through]`

**Purpose**: Implements read-through caching where cache misses automatically load from storage.

**Usage**:
```rust
#[service_method("GET /product-catalog")]
#[cache_read_through(
    cache_key = "catalog:{category}:{page}",
    ttl = "30m",
    loader = "load_product_catalog"
)]
pub async fn get_product_catalog(msg: Message) -> Result<RpcResponse, String> {
    let category = msg.get_query_param("category").unwrap_or_default();
    let page = msg.get_query_param("page").unwrap_or("1".to_string());
    
    // Read-through pattern:
    // 1. Check cache
    // 2. If miss, automatically call loader
    // 3. Cache the loaded data
    // 4. Return result
    // 5. Subsequent calls served from cache
    
    Ok(RpcResponse::success(&"catalog data", 0)?)
}

async fn load_product_catalog(category: String, page: String) -> Result<ProductCatalog, CatalogError> {
    // Load from database or external service
    fetch_catalog_from_database(&category, page.parse()?).await
}
```

**When to Use**: For simplifying cache miss handling and ensuring consistent data loading.

---

## Cache Warming & Preloading

### `#[cache_warm_up]`

**Purpose**: Preloads frequently accessed data into cache during service startup.

**Usage**:
```rust
#[service_impl]
impl ProductService {
    #[on_startup]
    #[cache_warm_up([
        "popular_products",
        "category_listings",
        "featured_items"
    ])]
    async fn warm_up_cache() -> Result<(), StartupError> {
        // Automatically executed during service startup
        // Preloads specified cache keys
        Ok(())
    }
    
    #[cache_preload("popular_products")]
    async fn preload_popular_products() -> Result<Vec<Product>, CacheError> {
        // Load most popular products
        fetch_popular_products(100).await
    }
    
    #[cache_preload("category_listings")]
    async fn preload_category_listings() -> Result<Vec<Category>, CacheError> {
        // Load all product categories
        fetch_all_categories().await
    }
    
    #[service_method("GET /products/popular")]
    #[cached(ttl = "1h")]
    pub async fn get_popular_products(msg: Message) -> Result<RpcResponse, String> {
        // This will be served from warmed cache
        let products = fetch_popular_products(50).await?;
        Ok(RpcResponse::success(&products, 0)?)
    }
}
```

**Cache Warming Strategies**:
- **Startup Warming**: Preload during service startup
- **Scheduled Warming**: Periodic cache refresh
- **Predictive Warming**: ML-based cache preloading
- **User-Triggered**: Warm cache based on user patterns

**When to Use**: For improving cold start performance and reducing cache miss penalties.

---

### `#[cache_refresh]`

**Purpose**: Automatically refreshes stale cache entries in the background.

**Usage**:
```rust
#[service_method("GET /stock-prices/:symbol")]
#[cached(ttl = "5m")]
#[cache_refresh(
    strategy = "background",
    refresh_ahead = "1m", // Refresh 1 minute before expiry
    max_staleness = "10m"
)]
pub async fn get_stock_price(msg: Message) -> Result<RpcResponse, String> {
    let symbol = msg.get_path_param("symbol")?;
    
    // Background refresh strategy:
    // - Serves from cache (even if slightly stale)
    // - Refreshes cache in background before expiry
    // - Ensures users always get fast responses
    
    let price = fetch_current_stock_price(&symbol).await?;
    Ok(RpcResponse::success(&price, 0)?)
}

#[service_method("GET /news-feed")]
#[cached(ttl = "2m")]
#[cache_refresh(
    strategy = "eager",
    refresh_frequency = "30s"
)]
pub async fn get_news_feed(msg: Message) -> Result<RpcResponse, String> {
    // Eager refresh strategy:
    // - Refreshes cache every 30 seconds
    // - Users always get fresh data
    // - Higher load but better data freshness
    
    let news = fetch_latest_news().await?;
    Ok(RpcResponse::success(&news, 0)?)
}
```

**Refresh Strategies**:
- **Background**: Refresh before expiry, serve stale if needed
- **Eager**: Refresh at regular intervals
- **On-Demand**: Refresh only when explicitly requested
- **Smart**: ML-based refresh timing

**When to Use**: For data that changes frequently but users expect fast responses.

---

## Integration Examples

### Complete E-commerce Caching Strategy
```rust
#[service_impl]
impl EcommerceService {
    // Product catalog with multi-level caching
    #[service_method("GET /products")]
    #[cached_multi_level(
        l1 = { ttl = "1m", max_size = 1000 },
        l2 = { ttl = "15m", backend = "redis" },
        l3 = { ttl = "1h", backend = "database" }
    )]
    #[cache_tags(["products", "catalog"])]
    pub async fn list_products(msg: Message) -> Result<RpcResponse, String> {
        let category = msg.get_query_param("category").unwrap_or_default();
        let sort = msg.get_query_param("sort").unwrap_or("popularity".to_string());
        
        let products = query_products(&category, &sort).await?;
        Ok(RpcResponse::success(&products, 0)?)
    }
    
    // Product details with cache warming
    #[service_method("GET /products/:id")]
    #[cached(
        key = "product:{id}:details",
        ttl = "30m"
    )]
    #[cache_refresh(
        strategy = "background",
        refresh_ahead = "5m"
    )]
    #[cache_tags(["product_{id}", "products"])]
    pub async fn get_product_details(msg: Message) -> Result<RpcResponse, String> {
        let product_id = msg.get_path_param("id")?;
        
        let product = fetch_product_with_inventory(&product_id).await?;
        Ok(RpcResponse::success(&product, 0)?)
    }
    
    // Shopping cart with write-through caching
    #[service_method("GET /cart")]
    #[require_auth]
    #[cached(
        key = "cart:{user_id}",
        ttl = "10m"
    )]
    #[cache_tags(["cart", "user_{user_id}"])]
    pub async fn get_shopping_cart(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        
        let cart = load_user_shopping_cart(&user_id).await?;
        Ok(RpcResponse::success(&cart, 0)?)
    }
    
    #[service_method("POST /cart/items")]
    #[require_auth]
    #[validate]
    #[cache_write_through(
        cache_key = "cart:{user_id}",
        ttl = "10m"
    )]
    #[cache_invalidate_tags(["user_{user_id}"])]
    pub async fn add_to_cart(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        let item: CartItem = msg.deserialize_payload()?;
        
        let updated_cart = add_item_to_cart(&user_id, item).await?;
        Ok(RpcResponse::success(&updated_cart, 0)?)
    }
    
    // User profile with distributed caching
    #[service_method("GET /profile")]
    #[require_auth]
    #[cached_distributed(
        backend = "redis_cluster",
        ttl = "1h",
        consistency = "eventual",
        partition_key = "user_id"
    )]
    #[cache_tags(["user_profile", "user_{user_id}"])]
    pub async fn get_user_profile(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        
        let profile = fetch_user_profile_with_preferences(&user_id).await?;
        Ok(RpcResponse::success(&profile, 0)?)
    }
    
    #[service_method("PUT /profile")]
    #[require_auth]
    #[validate]
    #[cache_invalidate_tags(["user_{user_id}", "user_profile"])]
    pub async fn update_profile(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_user_id()?;
        let updates: ProfileUpdate = msg.deserialize_payload()?;
        
        let updated_profile = update_user_profile(&user_id, updates).await?;
        Ok(RpcResponse::success(&updated_profile, 0)?)
    }
    
    // Search results with intelligent caching
    #[service_method("GET /search")]
    #[cached(
        key = "search:{query_hash}",
        ttl = "5m"
    )]
    #[cache_refresh(
        strategy = "smart",
        popularity_threshold = 10
    )]
    pub async fn search_products(msg: Message) -> Result<RpcResponse, String> {
        let query = msg.get_query_param("q").unwrap_or_default();
        let filters = msg.get_query_param("filters").unwrap_or_default();
        
        let results = perform_product_search(&query, &filters).await?;
        Ok(RpcResponse::success(&results, 0)?)
    }
    
    // Administrative data updates with cache invalidation
    #[service_method("PUT /admin/products/:id")]
    #[require_auth]
    #[require_role("admin")]
    #[validate]
    #[cache_invalidate([
        "product:{id}*",
        "search:*",
        "catalog:*"
    ])]
    #[cache_invalidate_tags(["products", "catalog", "product_{id}"])]
    pub async fn admin_update_product(msg: Message) -> Result<RpcResponse, String> {
        let product_id = msg.get_path_param("id")?;
        let updates: ProductUpdate = msg.deserialize_payload()?;
        
        let updated_product = update_product_admin(&product_id, updates).await?;
        
        // Warm cache with updated product
        self.cache_warm_product(&product_id).await?;
        
        Ok(RpcResponse::success(&updated_product, 0)?)
    }
    
    // Cache warming during startup
    #[on_startup]
    #[cache_warm_up([
        "popular_products",
        "featured_categories",
        "hot_deals",
        "trending_searches"
    ])]
    async fn warm_ecommerce_cache() -> Result<(), StartupError> {
        // Preload critical e-commerce data
        Ok(())
    }
    
    #[cache_preload("popular_products")]
    async fn preload_popular_products() -> Result<Vec<Product>, CacheError> {
        fetch_most_popular_products(50).await
    }
    
    #[cache_preload("featured_categories")]
    async fn preload_featured_categories() -> Result<Vec<Category>, CacheError> {
        fetch_featured_categories().await
    }
    
    #[cache_preload("hot_deals")]
    async fn preload_hot_deals() -> Result<Vec<Deal>, CacheError> {
        fetch_current_hot_deals().await
    }
}
```

### High-Performance API with Intelligent Caching
```rust
#[service_impl]
impl AnalyticsApiService {
    // Real-time data with short-term caching
    #[service_method("GET /metrics/realtime")]
    #[cached(ttl = "10s")]
    #[cache_refresh(
        strategy = "background",
        refresh_ahead = "2s"
    )]
    pub async fn get_realtime_metrics(msg: Message) -> Result<RpcResponse, String> {
        let metrics = collect_realtime_system_metrics().await?;
        Ok(RpcResponse::success(&metrics, 0)?)
    }
    
    // Historical data with long-term caching
    #[service_method("GET /analytics/historical")]
    #[cached_multi_level(
        l1 = { ttl = "5m", max_size = 100 },
        l2 = { ttl = "1h", backend = "redis" },
        l3 = { ttl = "24h", backend = "database" }
    )]
    #[cache_tags(["analytics", "historical"])]
    pub async fn get_historical_analytics(msg: Message) -> Result<RpcResponse, String> {
        let from_date = msg.get_query_param("from")?;
        let to_date = msg.get_query_param("to")?;
        let granularity = msg.get_query_param("granularity").unwrap_or("day".to_string());
        
        let analytics = compute_historical_analytics(&from_date, &to_date, &granularity).await?;
        Ok(RpcResponse::success(&analytics, 0)?)
    }
    
    // Heavy computation with cache-aside pattern
    #[service_method("GET /reports/:type")]
    #[require_auth]
    #[cache_aside(
        cache_key = "report:{type}:{params_hash}",
        ttl = "2h",
        miss_handler = "generate_complex_report"
    )]
    pub async fn get_complex_report(msg: Message) -> Result<RpcResponse, String> {
        let report_type = msg.get_path_param("type")?;
        // Complex report generation handled by cache-aside pattern
        Ok(RpcResponse::success(&"report data", 0)?)
    }
    
    async fn generate_complex_report(report_type: String, params: ReportParams) -> Result<Report, ReportError> {
        // Expensive computation
        compute_complex_analytics_report(&report_type, &params).await
    }
    
    // User-specific data with distributed caching
    #[service_method("GET /dashboard/:user_id")]
    #[require_auth]
    #[cached_distributed(
        backend = "redis_cluster",
        ttl = "15m",
        consistency = "eventual",
        partition_key = "user_id"
    )]
    pub async fn get_user_dashboard(msg: Message) -> Result<RpcResponse, String> {
        let user_id = msg.get_path_param("user_id")?;
        
        let dashboard = generate_user_specific_dashboard(&user_id).await?;
        Ok(RpcResponse::success(&dashboard, 0)?)
    }
}
```

---

## Best Practices

### 1. Choose Appropriate TTL Values
```rust
// Frequently changing data - short TTL
#[cached(ttl = "30s")]  // Real-time prices, live scores

// Moderately stable data - medium TTL  
#[cached(ttl = "5m")]   // User sessions, search results

// Stable data - long TTL
#[cached(ttl = "1h")]   // User profiles, product catalogs

// Static data - very long TTL
#[cached(ttl = "24h")]  // Configuration, reference data
```

### 2. Use Appropriate Cache Levels
```rust
// Frequently accessed, small data - In-memory (L1)
#[cached_multi_level(l1 = { ttl = "1m", max_size = 1000 })]

// Shared across instances - Redis (L2)  
#[cached_multi_level(l2 = { ttl = "15m", backend = "redis" })]

// Large, persistent data - Database cache (L3)
#[cached_multi_level(l3 = { ttl = "1h", backend = "database" })]
```

### 3. Implement Proper Cache Invalidation
```rust
// Pattern-based invalidation
#[cache_invalidate("user:{id}*")]

// Tag-based invalidation  
#[cache_invalidate_tags(["user_data", "profile"])]

// Multi-pattern invalidation
#[cache_invalidate([
    "user:{id}*",
    "dashboard:*",  
    "analytics:users*"
])]
```

### 4. Monitor Cache Performance
```rust
#[cached(ttl = "5m")]
#[metrics] // Track hit rates, response times
#[cache_monitor] // Monitor cache health
pub async fn monitored_endpoint() -> Result<Response, Error> {
    // Implementation with automatic monitoring
}
```

---

This completes the Caching Macros documentation, providing comprehensive caching strategies for building high-performance, scalable microservices with intelligent cache management.