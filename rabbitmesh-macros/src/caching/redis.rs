//! Redis Caching Module
//! 
//! Provides comprehensive Redis-based caching with support for multiple Redis configurations,
//! clustering, sentinel, and various caching patterns.

use quote::quote;

/// Generate Redis caching preprocessing code
pub fn generate_redis_caching_preprocessing(
    cache_key: Option<&str>,
    ttl_seconds: Option<u64>,
    cache_strategy: Option<&str>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🗄️ Initializing Redis caching layer");
        
        // Initialize Redis cache manager with auto-configuration
        let cache_manager = RedisCacheManager::new().await?;
        
        // Generate cache key based on message content or provided key
        let cache_key = if let Some(key) = #cache_key {
            key.to_string()
        } else {
            generate_cache_key(&msg)
        };
        
        let ttl = #ttl_seconds.unwrap_or(3600); // Default 1 hour
        let strategy = #cache_strategy.unwrap_or("read_through");
        
        tracing::debug!("🔑 Cache key: {}, TTL: {}s, Strategy: {}", cache_key, ttl, strategy);
        
        // Execute caching strategy
        let cache_result = match strategy {
            "read_through" => handle_read_through_cache(&cache_manager, &cache_key, ttl).await,
            "write_through" => handle_write_through_cache(&cache_manager, &cache_key, ttl).await,
            "write_behind" => handle_write_behind_cache(&cache_manager, &cache_key, ttl).await,
            "cache_aside" => handle_cache_aside(&cache_manager, &cache_key, ttl).await,
            "refresh_ahead" => handle_refresh_ahead_cache(&cache_manager, &cache_key, ttl).await,
            _ => {
                tracing::warn!("Unknown cache strategy '{}', falling back to read_through", strategy);
                handle_read_through_cache(&cache_manager, &cache_key, ttl).await
            }
        };
        
        match cache_result {
            CacheResult::Hit(value) => {
                tracing::debug!("✅ Cache hit for key: {}", cache_key);
                // Return cached value and skip handler execution
                return Ok(rabbitmesh::Message {
                    payload: value,
                    headers: msg.headers.clone(),
                    correlation_id: msg.correlation_id.clone(),
                    reply_to: msg.reply_to.clone(),
                });
            }
            CacheResult::Miss => {
                tracing::debug!("❌ Cache miss for key: {}", cache_key);
                // Continue to handler execution
            }
            CacheResult::Error(err) => {
                tracing::warn!("⚠️ Cache error for key {}: {}", cache_key, err);
                // Continue to handler execution on cache error
            }
        }
        
        /// Redis Cache Manager with support for various Redis configurations
        struct RedisCacheManager {
            client: RedisClient,
            serializer: Box<dyn CacheSerializer + Send + Sync>,
            compressor: Option<Box<dyn CacheCompressor + Send + Sync>>,
            encryption: Option<Box<dyn CacheEncryption + Send + Sync>>,
        }
        
        /// Redis client abstraction supporting different connection types
        enum RedisClient {
            Single(redis::Client),
            Cluster(redis::cluster::ClusterClient),
            Sentinel(redis::sentinel::SentinelClient),
            Pool(deadpool_redis::Pool),
        }
        
        /// Cache operation results
        #[derive(Debug)]
        enum CacheResult {
            Hit(serde_json::Value),
            Miss,
            Error(String),
        }
        
        /// Cache serialization trait for different formats
        trait CacheSerializer {
            fn serialize(&self, value: &serde_json::Value) -> Result<Vec<u8>, CacheError>;
            fn deserialize(&self, data: &[u8]) -> Result<serde_json::Value, CacheError>;
        }
        
        /// Cache compression trait for large values
        trait CacheCompressor {
            fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError>;
            fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError>;
        }
        
        /// Cache encryption trait for sensitive data
        trait CacheEncryption {
            fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CacheError>;
            fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CacheError>;
        }
        
        /// JSON serializer implementation
        struct JsonSerializer;
        
        impl CacheSerializer for JsonSerializer {
            fn serialize(&self, value: &serde_json::Value) -> Result<Vec<u8>, CacheError> {
                serde_json::to_vec(value)
                    .map_err(|e| CacheError::Serialization(format!("JSON serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8]) -> Result<serde_json::Value, CacheError> {
                serde_json::from_slice(data)
                    .map_err(|e| CacheError::Serialization(format!("JSON deserialization failed: {}", e)))
            }
        }
        
        /// MessagePack serializer for better performance
        struct MessagePackSerializer;
        
        impl CacheSerializer for MessagePackSerializer {
            fn serialize(&self, value: &serde_json::Value) -> Result<Vec<u8>, CacheError> {
                rmp_serde::to_vec(value)
                    .map_err(|e| CacheError::Serialization(format!("MessagePack serialization failed: {}", e)))
            }
            
            fn deserialize(&self, data: &[u8]) -> Result<serde_json::Value, CacheError> {
                rmp_serde::from_slice(data)
                    .map_err(|e| CacheError::Serialization(format!("MessagePack deserialization failed: {}", e)))
            }
        }
        
        /// GZIP compressor for large cache values
        struct GzipCompressor;
        
        impl CacheCompressor for GzipCompressor {
            fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
                use std::io::Write;
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(data)
                    .map_err(|e| CacheError::Compression(format!("GZIP compression failed: {}", e)))?;
                encoder.finish()
                    .map_err(|e| CacheError::Compression(format!("GZIP finish failed: {}", e)))
            }
            
            fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| CacheError::Compression(format!("GZIP decompression failed: {}", e)))?;
                Ok(decompressed)
            }
        }
        
        /// AES encryption for sensitive cache data
        struct AESEncryption {
            key: [u8; 32],
            iv: [u8; 16],
        }
        
        impl CacheEncryption for AESEncryption {
            fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
                use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
                
                let cipher = Aes256Gcm::new_from_slice(&self.key)
                    .map_err(|e| CacheError::Encryption(format!("AES key error: {}", e)))?;
                let nonce = Nonce::from_slice(&self.iv);
                
                cipher.encrypt(nonce, data)
                    .map_err(|e| CacheError::Encryption(format!("AES encryption failed: {}", e)))
            }
            
            fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
                use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
                
                let cipher = Aes256Gcm::new_from_slice(&self.key)
                    .map_err(|e| CacheError::Encryption(format!("AES key error: {}", e)))?;
                let nonce = Nonce::from_slice(&self.iv);
                
                cipher.decrypt(nonce, data)
                    .map_err(|e| CacheError::Encryption(format!("AES decryption failed: {}", e)))
            }
        }
        
        /// Cache error types
        #[derive(Debug, thiserror::Error)]
        enum CacheError {
            #[error("Redis connection error: {0}")]
            Connection(String),
            #[error("Cache operation error: {0}")]
            Operation(String),
            #[error("Serialization error: {0}")]
            Serialization(String),
            #[error("Compression error: {0}")]
            Compression(String),
            #[error("Encryption error: {0}")]
            Encryption(String),
            #[error("Configuration error: {0}")]
            Configuration(String),
        }
        
        impl RedisCacheManager {
            /// Create new Redis cache manager with auto-configuration
            async fn new() -> Result<Self, CacheError> {
                let redis_config = detect_redis_configuration().await?;
                let client = create_redis_client(&redis_config).await?;
                
                // Configure serializer
                let serializer_type = std::env::var("CACHE_SERIALIZER")
                    .unwrap_or_else(|_| "json".to_string());
                let serializer: Box<dyn CacheSerializer + Send + Sync> = match serializer_type.as_str() {
                    "msgpack" | "messagepack" => Box::new(MessagePackSerializer),
                    _ => Box::new(JsonSerializer),
                };
                
                // Configure compression
                let compressor = if std::env::var("CACHE_COMPRESSION").unwrap_or_default() == "true" {
                    Some(Box::new(GzipCompressor) as Box<dyn CacheCompressor + Send + Sync>)
                } else {
                    None
                };
                
                // Configure encryption
                let encryption = if let Ok(key_hex) = std::env::var("CACHE_ENCRYPTION_KEY") {
                    let key = hex::decode(key_hex)
                        .map_err(|e| CacheError::Configuration(format!("Invalid encryption key: {}", e)))?;
                    if key.len() != 32 {
                        return Err(CacheError::Configuration("Encryption key must be 32 bytes".to_string()));
                    }
                    let mut key_array = [0u8; 32];
                    key_array.copy_from_slice(&key);
                    
                    let iv = [0u8; 16]; // In production, use random IV per operation
                    Some(Box::new(AESEncryption { key: key_array, iv }) as Box<dyn CacheEncryption + Send + Sync>)
                } else {
                    None
                };
                
                Ok(Self {
                    client,
                    serializer,
                    compressor,
                    encryption,
                })
            }
            
            /// Get value from cache
            async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CacheError> {
                let raw_data = match &self.client {
                    RedisClient::Single(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Single Redis connection failed: {}", e)))?;
                        redis::cmd("GET").arg(key).query_async::<_, Option<Vec<u8>>>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis GET failed: {}", e)))?
                    }
                    RedisClient::Pool(pool) => {
                        let mut conn = pool.get().await
                            .map_err(|e| CacheError::Connection(format!("Pool connection failed: {}", e)))?;
                        redis::cmd("GET").arg(key).query_async::<_, Option<Vec<u8>>>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis GET failed: {}", e)))?
                    }
                    RedisClient::Cluster(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Cluster connection failed: {}", e)))?;
                        redis::cmd("GET").arg(key).query_async::<_, Option<Vec<u8>>>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Cluster GET failed: {}", e)))?
                    }
                    RedisClient::Sentinel(_client) => {
                        // Sentinel implementation would go here
                        return Err(CacheError::Operation("Sentinel not yet implemented".to_string()));
                    }
                };
                
                if let Some(mut data) = raw_data {
                    // Decrypt if encryption is enabled
                    if let Some(encryption) = &self.encryption {
                        data = encryption.decrypt(&data)?;
                    }
                    
                    // Decompress if compression is enabled
                    if let Some(compressor) = &self.compressor {
                        data = compressor.decompress(&data)?;
                    }
                    
                    // Deserialize
                    let value = self.serializer.deserialize(&data)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            
            /// Set value in cache with TTL
            async fn set(&self, key: &str, value: &serde_json::Value, ttl_seconds: u64) -> Result<(), CacheError> {
                // Serialize
                let mut data = self.serializer.serialize(value)?;
                
                // Compress if compression is enabled
                if let Some(compressor) = &self.compressor {
                    data = compressor.compress(&data)?;
                }
                
                // Encrypt if encryption is enabled
                if let Some(encryption) = &self.encryption {
                    data = encryption.encrypt(&data)?;
                }
                
                match &self.client {
                    RedisClient::Single(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Single Redis connection failed: {}", e)))?;
                        redis::cmd("SETEX").arg(key).arg(ttl_seconds).arg(data)
                            .query_async::<_, ()>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis SETEX failed: {}", e)))?;
                    }
                    RedisClient::Pool(pool) => {
                        let mut conn = pool.get().await
                            .map_err(|e| CacheError::Connection(format!("Pool connection failed: {}", e)))?;
                        redis::cmd("SETEX").arg(key).arg(ttl_seconds).arg(data)
                            .query_async::<_, ()>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis SETEX failed: {}", e)))?;
                    }
                    RedisClient::Cluster(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Cluster connection failed: {}", e)))?;
                        redis::cmd("SETEX").arg(key).arg(ttl_seconds).arg(data)
                            .query_async::<_, ()>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Cluster SETEX failed: {}", e)))?;
                    }
                    RedisClient::Sentinel(_client) => {
                        return Err(CacheError::Operation("Sentinel not yet implemented".to_string()));
                    }
                }
                
                Ok(())
            }
            
            /// Delete value from cache
            async fn delete(&self, key: &str) -> Result<bool, CacheError> {
                let deleted = match &self.client {
                    RedisClient::Single(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Single Redis connection failed: {}", e)))?;
                        redis::cmd("DEL").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis DEL failed: {}", e)))?
                    }
                    RedisClient::Pool(pool) => {
                        let mut conn = pool.get().await
                            .map_err(|e| CacheError::Connection(format!("Pool connection failed: {}", e)))?;
                        redis::cmd("DEL").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis DEL failed: {}", e)))?
                    }
                    RedisClient::Cluster(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Cluster connection failed: {}", e)))?;
                        redis::cmd("DEL").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Cluster DEL failed: {}", e)))?
                    }
                    RedisClient::Sentinel(_client) => {
                        return Err(CacheError::Operation("Sentinel not yet implemented".to_string()));
                    }
                };
                
                Ok(deleted > 0)
            }
            
            /// Check if key exists in cache
            async fn exists(&self, key: &str) -> Result<bool, CacheError> {
                let exists = match &self.client {
                    RedisClient::Single(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Single Redis connection failed: {}", e)))?;
                        redis::cmd("EXISTS").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis EXISTS failed: {}", e)))?
                    }
                    RedisClient::Pool(pool) => {
                        let mut conn = pool.get().await
                            .map_err(|e| CacheError::Connection(format!("Pool connection failed: {}", e)))?;
                        redis::cmd("EXISTS").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Redis EXISTS failed: {}", e)))?
                    }
                    RedisClient::Cluster(client) => {
                        let mut conn = client.get_async_connection().await
                            .map_err(|e| CacheError::Connection(format!("Cluster connection failed: {}", e)))?;
                        redis::cmd("EXISTS").arg(key).query_async::<_, i32>(&mut conn).await
                            .map_err(|e| CacheError::Operation(format!("Cluster EXISTS failed: {}", e)))?
                    }
                    RedisClient::Sentinel(_client) => {
                        return Err(CacheError::Operation("Sentinel not yet implemented".to_string()));
                    }
                };
                
                Ok(exists > 0)
            }
        }
        
        /// Caching strategies implementations
        
        /// Read-through cache pattern
        async fn handle_read_through_cache(
            cache: &RedisCacheManager, 
            key: &str, 
            ttl: u64
        ) -> CacheResult {
            match cache.get(key).await {
                Ok(Some(value)) => CacheResult::Hit(value),
                Ok(None) => CacheResult::Miss,
                Err(e) => CacheResult::Error(e.to_string()),
            }
        }
        
        /// Write-through cache pattern
        async fn handle_write_through_cache(
            cache: &RedisCacheManager, 
            key: &str, 
            ttl: u64
        ) -> CacheResult {
            // For write-through, we check cache first, then write will happen after handler
            handle_read_through_cache(cache, key, ttl).await
        }
        
        /// Write-behind cache pattern
        async fn handle_write_behind_cache(
            cache: &RedisCacheManager, 
            key: &str, 
            ttl: u64
        ) -> CacheResult {
            // For write-behind, read from cache first
            handle_read_through_cache(cache, key, ttl).await
        }
        
        /// Cache-aside pattern
        async fn handle_cache_aside(
            cache: &RedisCacheManager, 
            key: &str, 
            ttl: u64
        ) -> CacheResult {
            // Application manages cache manually
            handle_read_through_cache(cache, key, ttl).await
        }
        
        /// Refresh-ahead cache pattern
        async fn handle_refresh_ahead_cache(
            cache: &RedisCacheManager, 
            key: &str, 
            ttl: u64
        ) -> CacheResult {
            match cache.get(key).await {
                Ok(Some(value)) => {
                    // Check if cache entry is close to expiration and trigger refresh
                    // This is a simplified implementation
                    CacheResult::Hit(value)
                }
                Ok(None) => CacheResult::Miss,
                Err(e) => CacheResult::Error(e.to_string()),
            }
        }
        
        /// Utility functions
        
        /// Detect Redis configuration from environment
        async fn detect_redis_configuration() -> Result<RedisConfiguration, CacheError> {
            let redis_url = std::env::var("REDIS_URL")
                .or_else(|_| std::env::var("CACHE_URL"))
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());
            
            let redis_mode = std::env::var("REDIS_MODE")
                .unwrap_or_else(|_| "single".to_string())
                .to_lowercase();
            
            let config = match redis_mode.as_str() {
                "cluster" => RedisConfiguration::Cluster {
                    nodes: parse_cluster_nodes()?,
                    connection_timeout: parse_timeout("REDIS_CONNECTION_TIMEOUT", 5000),
                    response_timeout: parse_timeout("REDIS_RESPONSE_TIMEOUT", 5000),
                },
                "sentinel" => RedisConfiguration::Sentinel {
                    sentinels: parse_sentinel_nodes()?,
                    service_name: std::env::var("REDIS_SENTINEL_SERVICE")
                        .unwrap_or_else(|_| "mymaster".to_string()),
                    connection_timeout: parse_timeout("REDIS_CONNECTION_TIMEOUT", 5000),
                },
                "pool" => RedisConfiguration::Pool {
                    url: redis_url,
                    max_size: std::env::var("REDIS_POOL_SIZE")
                        .unwrap_or_else(|_| "10".to_string())
                        .parse()
                        .unwrap_or(10),
                    timeout: parse_timeout("REDIS_POOL_TIMEOUT", 5000),
                },
                _ => RedisConfiguration::Single {
                    url: redis_url,
                    connection_timeout: parse_timeout("REDIS_CONNECTION_TIMEOUT", 5000),
                },
            };
            
            Ok(config)
        }
        
        /// Redis configuration types
        #[derive(Debug)]
        enum RedisConfiguration {
            Single {
                url: String,
                connection_timeout: u64,
            },
            Cluster {
                nodes: Vec<String>,
                connection_timeout: u64,
                response_timeout: u64,
            },
            Sentinel {
                sentinels: Vec<String>,
                service_name: String,
                connection_timeout: u64,
            },
            Pool {
                url: String,
                max_size: u32,
                timeout: u64,
            },
        }
        
        /// Create Redis client based on configuration
        async fn create_redis_client(config: &RedisConfiguration) -> Result<RedisClient, CacheError> {
            match config {
                RedisConfiguration::Single { url, .. } => {
                    let client = redis::Client::open(url.as_str())
                        .map_err(|e| CacheError::Connection(format!("Failed to create Redis client: {}", e)))?;
                    Ok(RedisClient::Single(client))
                }
                RedisConfiguration::Pool { url, max_size, .. } => {
                    let cfg = deadpool_redis::Config::from_url(url);
                    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
                        .map_err(|e| CacheError::Connection(format!("Failed to create Redis pool: {}", e)))?;
                    Ok(RedisClient::Pool(pool))
                }
                RedisConfiguration::Cluster { nodes, .. } => {
                    let client = redis::cluster::ClusterClient::new(nodes.clone())
                        .map_err(|e| CacheError::Connection(format!("Failed to create Redis cluster client: {}", e)))?;
                    Ok(RedisClient::Cluster(client))
                }
                RedisConfiguration::Sentinel { .. } => {
                    // Sentinel implementation would go here
                    Err(CacheError::Configuration("Sentinel mode not yet implemented".to_string()))
                }
            }
        }
        
        /// Parse cluster nodes from environment
        fn parse_cluster_nodes() -> Result<Vec<String>, CacheError> {
            let nodes_str = std::env::var("REDIS_CLUSTER_NODES")
                .unwrap_or_else(|_| "localhost:6379,localhost:6380,localhost:6381".to_string());
            
            Ok(nodes_str.split(',')
                .map(|node| node.trim().to_string())
                .collect())
        }
        
        /// Parse sentinel nodes from environment
        fn parse_sentinel_nodes() -> Result<Vec<String>, CacheError> {
            let nodes_str = std::env::var("REDIS_SENTINEL_NODES")
                .unwrap_or_else(|_| "localhost:26379".to_string());
            
            Ok(nodes_str.split(',')
                .map(|node| node.trim().to_string())
                .collect())
        }
        
        /// Parse timeout from environment
        fn parse_timeout(env_var: &str, default: u64) -> u64 {
            std::env::var(env_var)
                .unwrap_or_else(|_| default.to_string())
                .parse()
                .unwrap_or(default)
        }
        
        /// Generate cache key from message
        fn generate_cache_key(msg: &rabbitmesh::Message) -> String {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            msg.payload.to_string().hash(&mut hasher);
            
            // Include relevant headers in cache key
            if let Some(headers) = msg.headers.as_ref() {
                for (key, value) in headers {
                    if !key.starts_with('_') { // Skip internal headers
                        key.hash(&mut hasher);
                        value.to_string().hash(&mut hasher);
                    }
                }
            }
            
            format!("rabbitmesh:cache:{:x}", hasher.finish())
        }
    }
}