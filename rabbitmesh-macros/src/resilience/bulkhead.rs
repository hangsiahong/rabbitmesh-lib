//! Bulkhead Pattern Module
//! 
//! Provides comprehensive resource isolation using the bulkhead pattern
//! to prevent cascading failures and ensure system resilience.

use quote::quote;

/// Generate bulkhead preprocessing code
pub fn generate_bulkhead_preprocessing(
    pool_name: Option<&str>,
    pool_size: Option<u32>,
    timeout_seconds: Option<u64>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🛡️ Initializing bulkhead resource isolation");
        
        // Create bulkhead manager with configuration
        let pool_name = #pool_name.unwrap_or("default");
        let pool_size = #pool_size.unwrap_or(10);
        let timeout_seconds = #timeout_seconds.unwrap_or(30);
        
        let bulkhead_manager = BulkheadManager::get_or_create().await?;
        let resource_pool = bulkhead_manager.get_or_create_pool(pool_name, pool_size).await?;
        
        tracing::debug!("🛡️ Bulkhead {} initialized - Pool size: {}, Timeout: {}s", 
            pool_name, pool_size, timeout_seconds);
        
        // Acquire resource from isolated pool
        let resource_permit = match resource_pool
            .acquire_resource(std::time::Duration::from_secs(timeout_seconds))
            .await 
        {
            Ok(permit) => {
                tracing::debug!("✅ Resource acquired from bulkhead pool '{}'", pool_name);
                permit
            }
            Err(BulkheadError::Timeout) => {
                tracing::warn!("⏰ Bulkhead resource acquisition timeout for pool '{}'", pool_name);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Resource pool '{}' timeout - all resources busy", pool_name)
                ));
            }
            Err(BulkheadError::PoolExhausted) => {
                tracing::warn!("🚫 Bulkhead pool '{}' exhausted", pool_name);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Resource pool '{}' exhausted", pool_name)
                ));
            }
            Err(e) => {
                tracing::error!("❌ Bulkhead error for pool '{}': {}", pool_name, e);
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Bulkhead error: {}", e)
                ));
            }
        };
        
        /// Bulkhead Manager for resource isolation
        struct BulkheadManager {
            pools: Arc<RwLock<std::collections::HashMap<String, Arc<ResourcePool>>>>,
            metrics: Arc<BulkheadMetrics>,
        }
        
        /// Resource Pool for isolated execution
        struct ResourcePool {
            name: String,
            semaphore: Arc<tokio::sync::Semaphore>,
            max_size: u32,
            active_count: Arc<AtomicU32>,
            queue_count: Arc<AtomicU32>,
            total_requests: Arc<AtomicU64>,
            timeout_count: Arc<AtomicU64>,
            rejection_count: Arc<AtomicU64>,
            created_at: std::time::Instant,
            pool_type: PoolType,
        }
        
        /// Types of resource pools
        #[derive(Debug, Clone, PartialEq)]
        enum PoolType {
            ThreadPool,      // CPU-bound tasks
            IOPool,          // I/O-bound tasks  
            DatabasePool,    // Database connections
            ExternalAPIPool, // External API calls
            Custom(String),  // Custom resource type
        }
        
        /// Resource permit for RAII resource management
        struct ResourcePermit {
            pool_name: String,
            permit: tokio::sync::SemaphorePermit<'static>,
            acquired_at: std::time::Instant,
            pool_active_count: Arc<AtomicU32>,
            pool_queue_count: Arc<AtomicU32>,
        }
        
        /// Bulkhead configuration
        #[derive(Debug, Clone)]
        struct BulkheadConfig {
            pool_name: String,
            max_pool_size: u32,
            queue_timeout: std::time::Duration,
            pool_type: PoolType,
            rejection_policy: RejectionPolicy,
            health_check_interval: std::time::Duration,
        }
        
        /// Policies for handling resource exhaustion
        #[derive(Debug, Clone, PartialEq)]
        enum RejectionPolicy {
            Abort,           // Immediately reject new requests
            CallerBlocks,    // Block caller until resource available
            DiscardOldest,   // Remove oldest queued request
            DiscardNewest,   // Reject newest request
        }
        
        /// Bulkhead metrics for monitoring
        struct BulkheadMetrics {
            pool_utilization: Arc<RwLock<std::collections::HashMap<String, f64>>>,
            queue_lengths: Arc<RwLock<std::collections::HashMap<String, u32>>>,
            timeout_rates: Arc<RwLock<std::collections::HashMap<String, f64>>>,
            throughput_metrics: Arc<RwLock<std::collections::HashMap<String, ThroughputMetric>>>,
        }
        
        /// Throughput metrics per pool
        #[derive(Debug, Clone)]
        struct ThroughputMetric {
            requests_per_second: f64,
            average_duration: std::time::Duration,
            last_updated: std::time::Instant,
            total_completed: u64,
        }
        
        /// Bulkhead errors
        #[derive(Debug, thiserror::Error)]
        enum BulkheadError {
            #[error("Resource acquisition timeout")]
            Timeout,
            #[error("Resource pool exhausted")]
            PoolExhausted,
            #[error("Pool not found: {pool_name}")]
            PoolNotFound { pool_name: String },
            #[error("Configuration error: {0}")]
            Configuration(String),
            #[error("Resource error: {0}")]
            Resource(String),
        }
        
        /// Pool health status
        #[derive(Debug, Clone)]
        struct PoolHealth {
            pool_name: String,
            is_healthy: bool,
            utilization_percent: f64,
            queue_depth: u32,
            average_wait_time: std::time::Duration,
            timeout_rate: f64,
            last_checked: std::time::Instant,
        }
        
        /// Thread-safe global bulkhead manager
        static BULKHEAD_MANAGER: once_cell::sync::OnceCell<Arc<BulkheadManager>> = once_cell::sync::OnceCell::new();
        
        impl BulkheadManager {
            /// Get or create singleton bulkhead manager
            async fn get_or_create() -> Result<Arc<Self>, BulkheadError> {
                if let Some(manager) = BULKHEAD_MANAGER.get() {
                    return Ok(manager.clone());
                }
                
                let manager = Arc::new(Self {
                    pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    metrics: Arc::new(BulkheadMetrics::new()),
                });
                
                // Start background health monitoring
                let manager_clone = manager.clone();
                tokio::spawn(async move {
                    manager_clone.health_monitor_loop().await;
                });
                
                BULKHEAD_MANAGER.set(manager.clone())
                    .map_err(|_| BulkheadError::Configuration("Failed to set global bulkhead manager".to_string()))?;
                
                Ok(manager)
            }
            
            /// Get or create resource pool
            async fn get_or_create_pool(&self, name: &str, size: u32) -> Result<Arc<ResourcePool>, BulkheadError> {
                // Try to get existing pool
                {
                    let pools = self.pools.read().await;
                    if let Some(pool) = pools.get(name) {
                        return Ok(pool.clone());
                    }
                }
                
                // Create new pool
                let pool_type = self.determine_pool_type(name);
                let pool = Arc::new(ResourcePool::new(name, size, pool_type)?);
                
                // Store in registry
                {
                    let mut pools = self.pools.write().await;
                    pools.insert(name.to_string(), pool.clone());
                }
                
                tracing::info!("🛡️ Created new bulkhead pool '{}' with {} resources", name, size);
                Ok(pool)
            }
            
            /// Determine pool type based on name
            fn determine_pool_type(&self, name: &str) -> PoolType {
                let name_lower = name.to_lowercase();
                
                if name_lower.contains("db") || name_lower.contains("database") {
                    PoolType::DatabasePool
                } else if name_lower.contains("api") || name_lower.contains("http") || name_lower.contains("external") {
                    PoolType::ExternalAPIPool
                } else if name_lower.contains("io") || name_lower.contains("file") || name_lower.contains("disk") {
                    PoolType::IOPool
                } else if name_lower.contains("cpu") || name_lower.contains("compute") || name_lower.contains("thread") {
                    PoolType::ThreadPool
                } else {
                    PoolType::Custom(name.to_string())
                }
            }
            
            /// Get pool health status
            async fn get_pool_health(&self, pool_name: &str) -> Result<PoolHealth, BulkheadError> {
                let pools = self.pools.read().await;
                let pool = pools.get(pool_name)
                    .ok_or_else(|| BulkheadError::PoolNotFound { pool_name: pool_name.to_string() })?;
                
                let utilization = pool.get_utilization();
                let queue_depth = pool.queue_count.load(std::sync::atomic::Ordering::Relaxed);
                let timeout_rate = pool.get_timeout_rate();
                
                Ok(PoolHealth {
                    pool_name: pool_name.to_string(),
                    is_healthy: utilization < 0.9 && timeout_rate < 0.1, // 90% util, 10% timeout threshold
                    utilization_percent: utilization * 100.0,
                    queue_depth,
                    average_wait_time: std::time::Duration::from_millis(10), // Would calculate actual wait time
                    timeout_rate,
                    last_checked: std::time::Instant::now(),
                })
            }
            
            /// Health monitoring background task
            async fn health_monitor_loop(&self) {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                
                loop {
                    interval.tick().await;
                    
                    let pools = {
                        let pools_read = self.pools.read().await;
                        pools_read.keys().cloned().collect::<Vec<_>>()
                    };
                    
                    for pool_name in pools {
                        if let Ok(health) = self.get_pool_health(&pool_name).await {
                            if !health.is_healthy {
                                tracing::warn!("🚨 Pool '{}' unhealthy - Util: {:.1}%, Timeouts: {:.1}%", 
                                    pool_name, health.utilization_percent, health.timeout_rate * 100.0);
                            } else {
                                tracing::debug!("✅ Pool '{}' healthy - Util: {:.1}%", 
                                    pool_name, health.utilization_percent);
                            }
                        }
                    }
                }
            }
        }
        
        impl ResourcePool {
            fn new(name: &str, max_size: u32, pool_type: PoolType) -> Result<Self, BulkheadError> {
                if max_size == 0 {
                    return Err(BulkheadError::Configuration("Pool size must be greater than 0".to_string()));
                }
                
                Ok(Self {
                    name: name.to_string(),
                    semaphore: Arc::new(tokio::sync::Semaphore::new(max_size as usize)),
                    max_size,
                    active_count: Arc::new(AtomicU32::new(0)),
                    queue_count: Arc::new(AtomicU32::new(0)),
                    total_requests: Arc::new(AtomicU64::new(0)),
                    timeout_count: Arc::new(AtomicU64::new(0)),
                    rejection_count: Arc::new(AtomicU64::new(0)),
                    created_at: std::time::Instant::now(),
                    pool_type,
                })
            }
            
            /// Acquire resource from pool with timeout
            async fn acquire_resource(&self, timeout: std::time::Duration) -> Result<ResourcePermit, BulkheadError> {
                self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.queue_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                tracing::debug!("🔄 Acquiring resource from pool '{}' (queue: {})", 
                    self.name, self.queue_count.load(std::sync::atomic::Ordering::Relaxed));
                
                let acquire_result = tokio::time::timeout(
                    timeout,
                    self.semaphore.clone().acquire_owned()
                ).await;
                
                self.queue_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                
                match acquire_result {
                    Ok(Ok(permit)) => {
                        self.active_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        
                        tracing::debug!("✅ Resource acquired from pool '{}' (active: {}/{})", 
                            self.name, 
                            self.active_count.load(std::sync::atomic::Ordering::Relaxed),
                            self.max_size);
                        
                        Ok(ResourcePermit {
                            pool_name: self.name.clone(),
                            permit,
                            acquired_at: std::time::Instant::now(),
                            pool_active_count: self.active_count.clone(),
                            pool_queue_count: self.queue_count.clone(),
                        })
                    }
                    Ok(Err(_)) => {
                        self.rejection_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        Err(BulkheadError::PoolExhausted)
                    }
                    Err(_) => {
                        self.timeout_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tracing::warn!("⏰ Resource acquisition timeout for pool '{}'", self.name);
                        Err(BulkheadError::Timeout)
                    }
                }
            }
            
            /// Get current pool utilization (0.0 to 1.0)
            fn get_utilization(&self) -> f64 {
                let active = self.active_count.load(std::sync::atomic::Ordering::Relaxed) as f64;
                let max = self.max_size as f64;
                active / max
            }
            
            /// Get timeout rate (0.0 to 1.0)
            fn get_timeout_rate(&self) -> f64 {
                let timeouts = self.timeout_count.load(std::sync::atomic::Ordering::Relaxed) as f64;
                let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed) as f64;
                
                if total > 0.0 {
                    timeouts / total
                } else {
                    0.0
                }
            }
            
            /// Get pool statistics
            fn get_statistics(&self) -> PoolStatistics {
                PoolStatistics {
                    pool_name: self.name.clone(),
                    pool_type: self.pool_type.clone(),
                    max_size: self.max_size,
                    active_count: self.active_count.load(std::sync::atomic::Ordering::Relaxed),
                    queue_count: self.queue_count.load(std::sync::atomic::Ordering::Relaxed),
                    total_requests: self.total_requests.load(std::sync::atomic::Ordering::Relaxed),
                    timeout_count: self.timeout_count.load(std::sync::atomic::Ordering::Relaxed),
                    rejection_count: self.rejection_count.load(std::sync::atomic::Ordering::Relaxed),
                    utilization: self.get_utilization(),
                    timeout_rate: self.get_timeout_rate(),
                    uptime: self.created_at.elapsed(),
                }
            }
        }
        
        impl Drop for ResourcePermit {
            fn drop(&mut self) {
                self.pool_active_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                let usage_duration = self.acquired_at.elapsed();
                
                tracing::debug!("🔓 Resource released from pool '{}' after {}ms", 
                    self.pool_name, usage_duration.as_millis());
                
                // Note: The semaphore permit is automatically returned when dropped
            }
        }
        
        impl BulkheadMetrics {
            fn new() -> Self {
                Self {
                    pool_utilization: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    queue_lengths: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    timeout_rates: Arc::new(RwLock::new(std::collections::HashMap::new())),
                    throughput_metrics: Arc::new(RwLock::new(std::collections::HashMap::new())),
                }
            }
        }
        
        /// Pool statistics for monitoring
        #[derive(Debug, Clone)]
        struct PoolStatistics {
            pool_name: String,
            pool_type: PoolType,
            max_size: u32,
            active_count: u32,
            queue_count: u32,
            total_requests: u64,
            timeout_count: u64,
            rejection_count: u64,
            utilization: f64,
            timeout_rate: f64,
            uptime: std::time::Duration,
        }
        
        /// Bulkhead configuration from environment
        fn get_bulkhead_config(pool_name: &str) -> BulkheadConfig {
            let env_prefix = format!("BULKHEAD_{}_", pool_name.to_uppercase());
            
            BulkheadConfig {
                pool_name: pool_name.to_string(),
                max_pool_size: std::env::var(format!("{}SIZE", env_prefix))
                    .unwrap_or_else(|_| pool_size.to_string())
                    .parse()
                    .unwrap_or(pool_size),
                queue_timeout: std::time::Duration::from_secs(
                    std::env::var(format!("{}TIMEOUT", env_prefix))
                        .unwrap_or_else(|_| timeout_seconds.to_string())
                        .parse()
                        .unwrap_or(timeout_seconds)
                ),
                pool_type: PoolType::Custom(pool_name.to_string()),
                rejection_policy: match std::env::var(format!("{}REJECT_POLICY", env_prefix))
                    .unwrap_or_else(|_| "abort".to_string())
                    .to_lowercase()
                    .as_str()
                {
                    "caller_blocks" => RejectionPolicy::CallerBlocks,
                    "discard_oldest" => RejectionPolicy::DiscardOldest,
                    "discard_newest" => RejectionPolicy::DiscardNewest,
                    _ => RejectionPolicy::Abort,
                },
                health_check_interval: std::time::Duration::from_secs(30),
            }
        }
        
        // Set up resource permit guard for automatic cleanup
        let _resource_guard = BulkheadResourceGuard {
            permit: resource_permit,
        };
        
        /// RAII guard for bulkhead resource management
        struct BulkheadResourceGuard {
            permit: ResourcePermit,
        }
        
        impl Drop for BulkheadResourceGuard {
            fn drop(&mut self) {
                // Resource is automatically released when ResourcePermit is dropped
                tracing::debug!("🛡️ Bulkhead resource guard dropped for pool '{}'", self.permit.pool_name);
            }
        }
    }
}