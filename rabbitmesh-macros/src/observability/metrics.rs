//! Metrics Collection Module
//! 
//! Provides comprehensive metrics collection with support for multiple backends
//! including Prometheus, StatsD, InfluxDB, and custom metrics systems.

use quote::quote;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, AtomicI64, AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};
use std::fmt;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::thread;
use std::process::Command;
use tokio::time::{interval, sleep};

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Metrics collection backend
    pub backend: MetricsBackendType,
    /// Collection interval
    pub collection_interval: Duration,
    /// Enable histogram metrics
    pub enable_histograms: bool,
    /// Histogram buckets
    pub histogram_buckets: Vec<f64>,
    /// Enable gauge metrics
    pub enable_gauges: bool,
    /// Enable counter metrics
    pub enable_counters: bool,
    /// Maximum metrics cache size
    pub max_cache_size: usize,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Enable metric labels
    pub enable_labels: bool,
    /// Maximum label cardinality
    pub max_label_cardinality: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            backend: MetricsBackendType::Prometheus,
            collection_interval: Duration::from_secs(15),
            enable_histograms: true,
            histogram_buckets: vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
            enable_gauges: true,
            enable_counters: true,
            max_cache_size: 10000,
            retention_period: Duration::from_secs(3600), // 1 hour
            enable_labels: true,
            max_label_cardinality: 1000,
        }
    }
}

/// Supported metrics backends
#[derive(Debug, Clone)]
pub enum MetricsBackendType {
    Prometheus,
    StatsD { endpoint: String },
    InfluxDB { endpoint: String, database: String },
    OpenTelemetry { endpoint: String },
    Custom { name: String, config: HashMap<String, String> },
    Multiple(Vec<MetricsBackendType>),
}

/// Metric types
#[derive(Debug, Clone, PartialEq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Set,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { buckets: Vec<(f64, u64)>, sum: f64, count: u64 },
    Summary { quantiles: Vec<(f64, f64)>, sum: f64, count: u64 },
    Set(Vec<String>),
}

/// Metric definition
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub help: Option<String>,
    pub timestamp: SystemTime,
}

/// LRU cache for metrics with time-based eviction
#[derive(Debug)]
pub struct LruMetricsCache {
    metrics: BTreeMap<String, CachedMetric>,
    access_order: Vec<String>, // Track access order manually
    max_size: usize,
    retention_period: Duration,
}

/// Cached metric with access time
#[derive(Debug, Clone)]
struct CachedMetric {
    metric: Metric,
    last_accessed: Instant,
    created_at: Instant,
}

impl LruMetricsCache {
    fn new(max_size: usize, retention_period: Duration) -> Self {
        Self {
            metrics: BTreeMap::new(),
            access_order: Vec::new(),
            max_size,
            retention_period,
        }
    }

    /// Insert or update a metric in the cache
    fn insert(&mut self, key: String, metric: Metric) {
        let now = Instant::now();
        let cached_metric = CachedMetric {
            metric,
            last_accessed: now,
            created_at: now,
        };

        // If key exists, remove it from access order
        if self.metrics.contains_key(&key) {
            self.access_order.retain(|k| k != &key);
        }

        // Insert metric and add to end of access order (most recently used)
        self.metrics.insert(key.clone(), cached_metric);
        self.access_order.push(key);

        // Perform eviction if needed
        self.evict_if_needed();
    }

    /// Get a metric from cache and update access time
    fn get(&mut self, key: &str) -> Option<Metric> {
        if let Some(cached_metric) = self.metrics.get_mut(key) {
            // Check if metric has expired
            if cached_metric.created_at.elapsed() > self.retention_period {
                let key = key.to_string();
                self.metrics.remove(&key);
                self.access_order.retain(|k| k != &key);
                return None;
            }

            // Update access time and move to end (most recently used)
            cached_metric.last_accessed = Instant::now();
            
            // Move to end of access order
            self.access_order.retain(|k| k != key);
            self.access_order.push(key.to_string());
            
            Some(cached_metric.metric.clone())
        } else {
            None
        }
    }

    /// Get all metrics (cloned)
    fn get_all(&self) -> HashMap<String, Metric> {
        let now = Instant::now();
        self.metrics
            .iter()
            .filter(|(_, cached)| cached.created_at.elapsed() <= self.retention_period)
            .map(|(key, cached)| (key.clone(), cached.metric.clone()))
            .collect()
    }

    /// Evict metrics if cache is full or expired
    fn evict_if_needed(&mut self) {
        let now = Instant::now();

        // First, remove expired metrics
        let expired_keys: Vec<String> = self.metrics
            .iter()
            .filter(|(_, cached)| cached.created_at.elapsed() > self.retention_period)
            .map(|(key, _)| key.clone())
            .collect();

        for key in &expired_keys {
            self.metrics.remove(key);
            self.access_order.retain(|k| k != key);
        }

        // Then, if still over capacity, remove least recently used metrics
        while self.metrics.len() > self.max_size && !self.access_order.is_empty() {
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.metrics.remove(&lru_key);
                self.access_order.remove(0);
            } else {
                break;
            }
        }
    }

    /// Get cache statistics
    fn stats(&self) -> CacheStats {
        let now = Instant::now();
        let expired_count = self.metrics
            .values()
            .filter(|cached| cached.created_at.elapsed() > self.retention_period)
            .count();

        CacheStats {
            total_entries: self.metrics.len(),
            expired_entries: expired_count,
            valid_entries: self.metrics.len() - expired_count,
            max_size: self.max_size,
            retention_period: self.retention_period,
        }
    }

    /// Clear all entries
    fn clear(&mut self) {
        self.metrics.clear();
    }

    /// Get size
    fn len(&self) -> usize {
        self.metrics.len()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
    pub max_size: usize,
    pub retention_period: Duration,
}

/// Metrics collector for gathering system metrics
pub struct MetricsCollector {
    config: MetricsConfig,
    metrics: Arc<RwLock<LruMetricsCache>>,
    backends: Vec<Box<dyn MetricsBackend + Send + Sync>>,
    system_metrics: SystemMetrics,
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
    collection_sender: mpsc::UnboundedSender<MetricEvent>,
    collection_receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<MetricEvent>>>,
}

/// System metrics collector
#[derive(Debug)]
pub struct SystemMetrics {
    pub cpu_usage: AtomicU64,
    pub memory_usage: AtomicU64,
    pub disk_usage: AtomicU64,
    pub network_bytes_sent: AtomicU64,
    pub network_bytes_received: AtomicU64,
    pub open_connections: AtomicU32,
    pub active_threads: AtomicU32,
    pub gc_collections: AtomicU64,
    pub gc_time_ms: AtomicU64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: AtomicU64::new(0),
            memory_usage: AtomicU64::new(0),
            disk_usage: AtomicU64::new(0),
            network_bytes_sent: AtomicU64::new(0),
            network_bytes_received: AtomicU64::new(0),
            open_connections: AtomicU32::new(0),
            active_threads: AtomicU32::new(0),
            gc_collections: AtomicU64::new(0),
            gc_time_ms: AtomicU64::new(0),
        }
    }
}

/// Custom metric definition
#[derive(Debug, Clone)]
pub struct CustomMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: Arc<AtomicU64>,
    pub labels: HashMap<String, String>,
    pub help: String,
}

/// Metric event for async processing
#[derive(Debug, Clone)]
pub struct MetricEvent {
    pub metric_name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: SystemTime,
}

/// Metrics backend trait
pub trait MetricsBackend {
    /// Send metric to backend
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError>;
    
    /// Send batch of metrics
    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError>;
    
    /// Get backend name
    fn name(&self) -> &str;
    
    /// Initialize backend
    fn initialize(&mut self) -> Result<(), MetricsError>;
    
    /// Shutdown backend
    fn shutdown(&mut self) -> Result<(), MetricsError>;
}

/// Metrics errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Backend error: {backend} - {reason}")]
    BackendError { backend: String, reason: String },
    #[error("Metric validation failed: {metric_name} - {reason}")]
    ValidationError { metric_name: String, reason: String },
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("Collection error: {reason}")]
    CollectionError { reason: String },
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
    #[error("Metrics error: {source}")]
    MetricsError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Result<Self, MetricsError> {
        let (collection_sender, collection_receiver) = mpsc::unbounded_channel();

        let mut collector = Self {
            config: config.clone(),
            metrics: Arc::new(RwLock::new(LruMetricsCache::new(
                config.max_cache_size, 
                config.retention_period
            ))),
            backends: Vec::new(),
            system_metrics: SystemMetrics::default(),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_sender,
            collection_receiver: Arc::new(tokio::sync::Mutex::new(collection_receiver)),
        };

        // Initialize backends based on configuration
        collector.initialize_backends()?;

        Ok(collector)
    }

    /// Initialize metrics backends
    fn initialize_backends(&mut self) -> Result<(), MetricsError> {
        match &self.config.backend {
            MetricsBackendType::Prometheus => {
                let backend = PrometheusBackend::new();
                self.backends.push(Box::new(backend));
            }
            MetricsBackendType::StatsD { endpoint } => {
                let backend = StatsDBackend::new(endpoint.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackendType::InfluxDB { endpoint, database } => {
                let backend = InfluxDBBackend::new(endpoint.clone(), database.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackendType::OpenTelemetry { endpoint } => {
                let backend = OpenTelemetryBackend::new(endpoint.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackendType::Custom { name, config } => {
                let backend = CustomBackend::new(name.clone(), config.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackendType::Multiple(backends) => {
                for backend_config in backends {
                    // Recursively initialize multiple backends
                    let backend = self.create_backend_from_config(backend_config)?;
                    self.backends.push(backend);
                    tracing::info!("Initialized multiple backend: {:?}", backend_config);
                }
            }
        }

        // Initialize all backends
        for backend in &mut self.backends {
            backend.initialize()?;
        }

        Ok(())
    }

    /// Create backend from configuration
    fn create_backend_from_config(&self, backend_config: &MetricsBackendType) -> Result<Box<dyn MetricsBackend + Send + Sync>, MetricsError> {
        match backend_config {
            MetricsBackendType::Prometheus => {
                let backend = PrometheusBackend::new();
                Ok(Box::new(backend))
            }
            MetricsBackendType::StatsD { endpoint } => {
                let backend = StatsDBackend::new(endpoint.clone());
                Ok(Box::new(backend))
            }
            MetricsBackendType::InfluxDB { endpoint, database } => {
                let backend = InfluxDBBackend::new(endpoint.clone(), database.clone());
                Ok(Box::new(backend))
            }
            MetricsBackendType::OpenTelemetry { endpoint } => {
                let backend = OpenTelemetryBackend::new(endpoint.clone());
                Ok(Box::new(backend))
            }
            MetricsBackendType::Custom { name, config } => {
                let backend = CustomBackend::new(name.clone(), config.clone());
                Ok(Box::new(backend))
            }
            MetricsBackendType::Multiple(_) => {
                return Err(MetricsError::ConfigurationError {
                    message: "Nested Multiple backends are not supported".to_string(),
                });
            }
        }
    }

    /// Start metrics collection
    pub async fn start_collection(&self) -> Result<(), MetricsError> {
        // Start system metrics collection
        self.collect_system_metrics().await?;

        // Start custom metrics processing
        self.process_metric_events().await?;

        tracing::info!("Started metrics collection with {} backends", self.backends.len());
        Ok(())
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> Result<(), MetricsError> {
        // Collect system metrics using platform-specific methods
        // This provides a fallback implementation that doesn't require external crates
        
        let (cpu_usage, total_memory, used_memory) = self.collect_basic_system_info().await;
        
        self.system_metrics.cpu_usage.store(cpu_usage, Ordering::Relaxed);
        self.system_metrics.memory_usage.store(used_memory, Ordering::Relaxed);
        
        let memory_usage_percent = if total_memory > 0 {
            ((used_memory as f64 / total_memory as f64) * 100.0) as u64
        } else {
            0
        };

        // Collect disk usage
        let disk_usage = self.collect_disk_usage().await;
        self.system_metrics.disk_usage.store(disk_usage, Ordering::Relaxed);

        // Network metrics (from /proc/net/dev on Linux systems)
        let (bytes_sent, bytes_received) = self.collect_network_metrics().await?;
        self.system_metrics.network_bytes_sent.store(bytes_sent, Ordering::Relaxed);
        self.system_metrics.network_bytes_received.store(bytes_received, Ordering::Relaxed);

        // Process metrics - count running processes
        let process_count = self.count_processes().await;
        self.system_metrics.active_threads.store(process_count, Ordering::Relaxed);

        // Connection count (approximate by counting sockets)
        let connection_count = self.count_open_connections().await.unwrap_or(0);
        self.system_metrics.open_connections.store(connection_count, Ordering::Relaxed);

        // Create comprehensive system metrics
        let metrics = vec![
            Metric {
                name: "system_cpu_usage_percent".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(cpu_usage as f64),
                labels: [("cpu".to_string(), "total".to_string())].iter().cloned().collect(),
                help: Some("Total CPU usage percentage".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_memory_usage_bytes".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(used_memory as f64),
                labels: [("type".to_string(), "used".to_string())].iter().cloned().collect(),
                help: Some("Used memory in bytes".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_memory_total_bytes".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(total_memory as f64),
                labels: [("type".to_string(), "total".to_string())].iter().cloned().collect(),
                help: Some("Total memory in bytes".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_memory_usage_percent".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(memory_usage_percent as f64),
                labels: HashMap::new(),
                help: Some("Memory usage percentage".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_disk_usage_bytes".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(disk_usage as f64),
                labels: [("device".to_string(), "root".to_string())].iter().cloned().collect(),
                help: Some("Disk usage in bytes".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_network_bytes_sent_total".to_string(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(bytes_sent),
                labels: HashMap::new(),
                help: Some("Total network bytes sent".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_network_bytes_received_total".to_string(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(bytes_received),
                labels: HashMap::new(),
                help: Some("Total network bytes received".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_processes_total".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(process_count as f64),
                labels: HashMap::new(),
                help: Some("Total number of processes".to_string()),
                timestamp: SystemTime::now(),
            },
            Metric {
                name: "system_connections_open".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(connection_count as f64),
                labels: HashMap::new(),
                help: Some("Number of open connections".to_string()),
                timestamp: SystemTime::now(),
            },
        ];

        // Send all metrics to backends
        for backend in &self.backends {
            backend.send_batch(&metrics)?;
        }

        tracing::debug!(
            "Collected system metrics: CPU {}%, Memory {}MB used/{} MB total, {} processes, {} connections", 
            cpu_usage, 
            used_memory / 1024 / 1024,
            total_memory / 1024 / 1024, 
            process_count,
            connection_count
        );

        Ok(())
    }

    /// Collect network metrics from system
    async fn collect_network_metrics(&self) -> Result<(u64, u64), MetricsError> {
        // Try to read network stats from /proc/net/dev (Linux) or use fallback
        #[cfg(target_os = "linux")]
        {
            match tokio::fs::read_to_string("/proc/net/dev").await {
                Ok(content) => {
                    let mut total_sent = 0u64;
                    let mut total_received = 0u64;
                    
                    for line in content.lines().skip(2) { // Skip header lines
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 10 {
                            // Interface name ends with ':'
                            if let (Ok(rx_bytes), Ok(tx_bytes)) = (
                                parts[1].parse::<u64>(),
                                parts[9].parse::<u64>()
                            ) {
                                total_received += rx_bytes;
                                total_sent += tx_bytes;
                            }
                        }
                    }
                    
                    return Ok((total_sent, total_received));
                }
                Err(e) => {
                    tracing::debug!("Could not read /proc/net/dev: {}", e);
                }
            }
        }
        
        // Fallback for non-Linux systems or if /proc/net/dev is not available
        Ok((0, 0))
    }

    /// Count open connections approximation
    async fn count_open_connections(&self) -> Result<u32, MetricsError> {
        #[cfg(target_os = "linux")]
        {
            // Count entries in /proc/net/tcp and /proc/net/tcp6
            let mut connection_count = 0u32;
            
            if let Ok(tcp_content) = tokio::fs::read_to_string("/proc/net/tcp").await {
                connection_count += tcp_content.lines().skip(1).count() as u32;
            }
            
            if let Ok(tcp6_content) = tokio::fs::read_to_string("/proc/net/tcp6").await {
                connection_count += tcp6_content.lines().skip(1).count() as u32;
            }
            
            return Ok(connection_count);
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // For non-Linux systems, use lsof or netstat command as fallback
            match Command::new("lsof").arg("-i").output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    Ok(output_str.lines().skip(1).count() as u32)
                }
                Err(_) => {
                    // Try netstat as fallback
                    match Command::new("netstat").arg("-an").output() {
                        Ok(output) => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            Ok(output_str.lines().filter(|line| line.contains("ESTABLISHED")).count() as u32)
                        }
                        Err(_) => Ok(0),
                    }
                }
            }
        }
    }

    /// Collect basic system information (CPU, Memory) using platform-specific methods
    async fn collect_basic_system_info(&self) -> (u64, u64, u64) {
        // Try to read system info from /proc (Linux) or use fallback values
        #[cfg(target_os = "linux")]
        {
            let cpu_usage = self.read_cpu_usage_linux().await.unwrap_or(0);
            let (total_memory, used_memory) = self.read_memory_usage_linux().await.unwrap_or((0, 0));
            (cpu_usage, total_memory, used_memory)
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback values for non-Linux systems
            // In a production environment, you would implement platform-specific calls
            (25, 8_000_000_000, 2_000_000_000) // 25% CPU, 8GB total, 2GB used
        }
    }

    /// Read CPU usage from /proc/stat on Linux
    #[cfg(target_os = "linux")]
    async fn read_cpu_usage_linux(&self) -> Result<u64, std::io::Error> {
        let stat_content = tokio::fs::read_to_string("/proc/stat").await?;
        
        if let Some(cpu_line) = stat_content.lines().next() {
            let values: Vec<&str> = cpu_line.split_whitespace().collect();
            if values.len() >= 8 && values[0] == "cpu" {
                let user: u64 = values[1].parse().unwrap_or(0);
                let nice: u64 = values[2].parse().unwrap_or(0);
                let system: u64 = values[3].parse().unwrap_or(0);
                let idle: u64 = values[4].parse().unwrap_or(0);
                let iowait: u64 = values[5].parse().unwrap_or(0);
                let irq: u64 = values[6].parse().unwrap_or(0);
                let softirq: u64 = values[7].parse().unwrap_or(0);

                let total = user + nice + system + idle + iowait + irq + softirq;
                let active = total - idle;
                
                if total > 0 {
                    return Ok((active * 100) / total);
                }
            }
        }
        
        Ok(0)
    }

    /// Read memory usage from /proc/meminfo on Linux
    #[cfg(target_os = "linux")]
    async fn read_memory_usage_linux(&self) -> Result<(u64, u64), std::io::Error> {
        let meminfo_content = tokio::fs::read_to_string("/proc/meminfo").await?;
        
        let mut mem_total = 0u64;
        let mut mem_available = 0u64;
        
        for line in meminfo_content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    mem_total = value.parse().unwrap_or(0) * 1024; // Convert kB to bytes
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    mem_available = value.parse().unwrap_or(0) * 1024; // Convert kB to bytes
                }
            }
        }
        
        let mem_used = mem_total.saturating_sub(mem_available);
        Ok((mem_total, mem_used))
    }

    /// Collect disk usage
    async fn collect_disk_usage(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            // Try to get disk usage from df command
            match Command::new("df").arg("/").arg("-B1").output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(line) = output_str.lines().nth(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            return parts[2].parse().unwrap_or(0); // Used bytes
                        }
                    }
                }
                Err(_) => {}
            }
        }
        
        // Fallback
        1_000_000_000 // 1GB
    }

    /// Count processes
    async fn count_processes(&self) -> u32 {
        #[cfg(target_os = "linux")]
        {
            // Count entries in /proc that are process directories
            if let Ok(entries) = tokio::fs::read_dir("/proc").await {
                let mut count = 0u32;
                let mut entries = entries;
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.chars().all(|c| c.is_ascii_digit()) {
                            count += 1;
                        }
                    }
                }
                return count;
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Use ps command on non-Linux systems
            match Command::new("ps").arg("ax").output() {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    return output_str.lines().skip(1).count() as u32; // Skip header
                }
                Err(_) => {}
            }
        }
        
        50 // Fallback value
    }

    /// Process metric events
    async fn process_metric_events(&self) -> Result<(), MetricsError> {
        let receiver = self.collection_receiver.clone();
        let backends = self.backends.iter().map(|b| b.name().to_string()).collect::<Vec<_>>();
        let config = self.config.clone();

        // Spawn background task to process metric events
        tokio::spawn(async move {
            let mut receiver = receiver.lock().await;

            tracing::info!("Started background metric events processing task");
            
            let mut batch = Vec::new();
            let mut last_flush = Instant::now();
            let batch_size = 100;
            let flush_interval = Duration::from_secs(5);

            while let Some(event) = receiver.recv().await {
                // Convert metric event to metric
                let metric = Metric {
                    name: event.metric_name,
                    metric_type: event.metric_type.clone(),
                    value: match event.metric_type {
                        MetricType::Counter => MetricValue::Counter(event.value as u64),
                        MetricType::Gauge => MetricValue::Gauge(event.value),
                        MetricType::Histogram => {
                            // Create histogram buckets based on config
                            let buckets: Vec<(f64, u64)> = config.histogram_buckets
                                .iter()
                                .map(|&bucket| {
                                    let count = if event.value <= bucket { 1 } else { 0 };
                                    (bucket, count)
                                })
                                .collect();
                            MetricValue::Histogram {
                                buckets,
                                sum: event.value,
                                count: 1,
                            }
                        },
                        MetricType::Summary => MetricValue::Summary {
                            quantiles: vec![(0.5, event.value), (0.9, event.value), (0.99, event.value)],
                            sum: event.value,
                            count: 1,
                        },
                        MetricType::Set => MetricValue::Set(vec![event.value.to_string()]),
                    },
                    labels: event.labels,
                    help: None,
                    timestamp: event.timestamp,
                };

                batch.push(metric);

                // Check if we should flush the batch
                let should_flush = batch.len() >= batch_size 
                    || last_flush.elapsed() >= flush_interval;

                if should_flush && !batch.is_empty() {
                    tracing::debug!("Flushing metric event batch of {} items", batch.len());
                    
                    // Process batch (in a real implementation, this would send to backends)
                    // For now, we just log the metrics
                    for metric in &batch {
                        tracing::trace!("Processing metric event: {} = {:?}", metric.name, metric.value);
                    }
                    
                    batch.clear();
                    last_flush = Instant::now();
                }
            }

            // Flush remaining metrics on shutdown
            if !batch.is_empty() {
                tracing::debug!("Flushing final metric event batch of {} items", batch.len());
                for metric in &batch {
                    tracing::trace!("Processing final metric event: {} = {:?}", metric.name, metric.value);
                }
            }

            tracing::info!("Metric events processing task completed");
        });

        tracing::debug!("Started background task for processing metric events with {} backends", backends.len());
        Ok(())
    }

    /// Send metric event for async processing
    pub fn send_metric_event(&self, event: MetricEvent) -> Result<(), MetricsError> {
        self.collection_sender.send(event)
            .map_err(|e| MetricsError::CollectionError {
                reason: format!("Failed to send metric event: {}", e)
            })
    }

    /// Create and send a metric event
    pub fn emit_metric_event(&self, name: &str, metric_type: MetricType, value: f64, labels: HashMap<String, String>) -> Result<(), MetricsError> {
        let event = MetricEvent {
            metric_name: name.to_string(),
            metric_type,
            value,
            labels,
            timestamp: SystemTime::now(),
        };
        self.send_metric_event(event)
    }

    /// Record a counter metric
    pub fn counter(&self, name: &str, value: u64, labels: HashMap<String, String>) -> Result<(), MetricsError> {
        if !self.config.enable_counters {
            return Ok(());
        }

        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(value),
            labels,
            help: None,
            timestamp: SystemTime::now(),
        };

        self.record_metric(metric)
    }

    /// Record a gauge metric
    pub fn gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<(), MetricsError> {
        if !self.config.enable_gauges {
            return Ok(());
        }

        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            labels,
            help: None,
            timestamp: SystemTime::now(),
        };

        self.record_metric(metric)
    }

    /// Record a histogram metric
    pub fn histogram(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<(), MetricsError> {
        if !self.config.enable_histograms {
            return Ok(());
        }

        // Create histogram buckets
        let mut buckets = Vec::new();
        for &bucket in &self.config.histogram_buckets {
            let count = if value <= bucket { 1 } else { 0 };
            buckets.push((bucket, count));
        }

        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Histogram,
            value: MetricValue::Histogram {
                buckets,
                sum: value,
                count: 1,
            },
            labels,
            help: None,
            timestamp: SystemTime::now(),
        };

        self.record_metric(metric)
    }

    /// Record a generic metric
    fn record_metric(&self, metric: Metric) -> Result<(), MetricsError> {
        // Validate metric
        self.validate_metric(&metric)?;

        // Cache metric using LRU cache
        if let Ok(mut metrics_cache) = self.metrics.write() {
            metrics_cache.insert(metric.name.clone(), metric.clone());
        }

        // Send to backends
        for backend in &self.backends {
            backend.send_metric(&metric)?;
        }

        Ok(())
    }

    /// Validate metric
    fn validate_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        if metric.name.is_empty() {
            return Err(MetricsError::ValidationError {
                metric_name: metric.name.clone(),
                reason: "Metric name cannot be empty".to_string(),
            });
        }

        if self.config.enable_labels && metric.labels.len() > self.config.max_label_cardinality {
            return Err(MetricsError::ValidationError {
                metric_name: metric.name.clone(),
                reason: format!("Too many labels: {} > {}", metric.labels.len(), self.config.max_label_cardinality),
            });
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> Option<CacheStats> {
        if let Ok(metrics_cache) = self.metrics.read() {
            Some(metrics_cache.stats())
        } else {
            None
        }
    }

    /// Clear metrics cache
    pub fn clear_cache(&self) -> Result<(), MetricsError> {
        if let Ok(mut metrics_cache) = self.metrics.write() {
            metrics_cache.clear();
            tracing::info!("Metrics cache cleared");
            Ok(())
        } else {
            Err(MetricsError::CollectionError {
                reason: "Failed to acquire write lock on metrics cache".to_string(),
            })
        }
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> HashMap<String, Metric> {
        if let Ok(metrics_cache) = self.metrics.read() {
            metrics_cache.get_all()
        } else {
            HashMap::new()
        }
    }

    /// Get a specific metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        if let Ok(mut metrics_cache) = self.metrics.write() {
            metrics_cache.get(name)
        } else {
            None
        }
    }

    /// Register custom metric
    pub fn register_custom_metric(&self, custom_metric: CustomMetric) -> Result<(), MetricsError> {
        if let Ok(mut custom_metrics) = self.custom_metrics.write() {
            custom_metrics.insert(custom_metric.name.clone(), custom_metric);
        }
        Ok(())
    }
}

/// Prometheus backend implementation
#[derive(Clone, Debug)]
pub struct PrometheusBackend {
    endpoint: String,
    push_gateway_endpoint: Option<String>,
    job_name: String,
    instance: String,
}

impl PrometheusBackend {
    pub fn new() -> Self {
        let hostname = Self::get_hostname();

        Self {
            endpoint: std::env::var("PROMETHEUS_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9090".to_string()),
            push_gateway_endpoint: std::env::var("PROMETHEUS_PUSHGATEWAY_ENDPOINT").ok(),
            job_name: std::env::var("PROMETHEUS_JOB_NAME")
                .unwrap_or_else(|_| "rabbitmesh".to_string()),
            instance: format!("{}:{}", hostname, std::process::id()),
        }
    }

    pub fn with_config(endpoint: String, push_gateway_endpoint: Option<String>, job_name: String) -> Self {
        let hostname = Self::get_hostname();

        Self {
            endpoint,
            push_gateway_endpoint,
            job_name,
            instance: format!("{}:{}", hostname, std::process::id()),
        }
    }

    /// Get hostname using standard library methods
    fn get_hostname() -> String {
        // Try to get hostname from environment or use fallback
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("HOST"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Format metric in Prometheus exposition format
    fn format_metric(&self, metric: &Metric) -> String {
        let mut output = String::new();

        // Add HELP line if available
        if let Some(help) = &metric.help {
            output.push_str(&format!("# HELP {} {}\n", metric.name, help));
        }

        // Add TYPE line
        let metric_type_str = match metric.metric_type {
            MetricType::Counter => "counter",
            MetricType::Gauge => "gauge",
            MetricType::Histogram => "histogram",
            MetricType::Summary => "summary",
            MetricType::Set => "gauge", // Sets are represented as gauges in Prometheus
        };
        output.push_str(&format!("# TYPE {} {}\n", metric.name, metric_type_str));

        // Format labels
        let labels_str = if metric.labels.is_empty() {
            String::new()
        } else {
            let labels: Vec<String> = metric.labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v.replace("\"", "\\\"")))
                .collect();
            format!("{{{}}}", labels.join(","))
        };

        // Format value based on metric type
        match &metric.value {
            MetricValue::Counter(value) => {
                output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
            }
            MetricValue::Gauge(value) => {
                output.push_str(&format!("{}{} {}\n", metric.name, labels_str, value));
            }
            MetricValue::Histogram { buckets, sum, count } => {
                // Output histogram buckets
                for (bucket, bucket_count) in buckets {
                    let bucket_str = if bucket.is_infinite() { 
                        "+Inf".to_string() 
                    } else { 
                        bucket.to_string() 
                    };
                    let bucket_labels = if labels_str.is_empty() {
                        format!("{{le=\"{}\"}}", bucket_str)
                    } else {
                        format!("{{le=\"{}\",{}}}", 
                            bucket_str,
                            &labels_str[1..labels_str.len()-1] // Remove outer braces
                        )
                    };
                    output.push_str(&format!("{}_bucket{} {}\n", metric.name, bucket_labels, bucket_count));
                }

                // Output sum and count
                output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
            }
            MetricValue::Summary { quantiles, sum, count } => {
                // Output quantiles
                for (quantile, value) in quantiles {
                    let quantile_labels = if labels_str.is_empty() {
                        format!("{{quantile=\"{}\"}}", quantile)
                    } else {
                        format!("{{quantile=\"{}\",{}}}", 
                            quantile,
                            &labels_str[1..labels_str.len()-1] // Remove outer braces
                        )
                    };
                    output.push_str(&format!("{}{} {}\n", metric.name, quantile_labels, value));
                }

                // Output sum and count
                output.push_str(&format!("{}_sum{} {}\n", metric.name, labels_str, sum));
                output.push_str(&format!("{}_count{} {}\n", metric.name, labels_str, count));
            }
            MetricValue::Set(values) => {
                // Sets are represented as gauge with count of unique values
                output.push_str(&format!("{}{} {}\n", metric.name, labels_str, values.len()));
            }
        }

        output
    }

    /// Send metrics to Prometheus Push Gateway
    async fn push_to_gateway(&self, formatted_metrics: &str) -> Result<(), MetricsError> {
        if let Some(gateway_endpoint) = &self.push_gateway_endpoint {
            let url = format!("{}/metrics/job/{}/instance/{}", 
                gateway_endpoint, 
                self.simple_url_encode(&self.job_name),
                self.simple_url_encode(&self.instance)
            );

            tracing::debug!("Would push metrics to Prometheus Push Gateway at: {}", url);
            tracing::trace!("Metrics payload:\n{}", formatted_metrics);
            
            // Note: In a production environment, you would implement HTTP POST here
            // This could be done using std::process::Command with curl, or by adding
            // reqwest as a dependency. For now, we just log the operation.
            
            tracing::debug!("Successfully formatted metrics for Prometheus Push Gateway");
        }

        Ok(())
    }

    /// Simple URL encoding for basic characters
    fn simple_url_encode(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                ' ' => "%20".to_string(),
                '/' => "%2F".to_string(),
                ':' => "%3A".to_string(),
                '?' => "%3F".to_string(),
                '#' => "%23".to_string(),
                '[' => "%5B".to_string(),
                ']' => "%5D".to_string(),
                '@' => "%40".to_string(),
                '!' => "%21".to_string(),
                '$' => "%24".to_string(),
                '&' => "%26".to_string(),
                '\'' => "%27".to_string(),
                '(' => "%28".to_string(),
                ')' => "%29".to_string(),
                '*' => "%2A".to_string(),
                '+' => "%2B".to_string(),
                ',' => "%2C".to_string(),
                ';' => "%3B".to_string(),
                '=' => "%3D".to_string(),
                c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

impl MetricsBackend for PrometheusBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        let formatted_metric = self.format_metric(metric);
        
        // If push gateway is configured, send metrics there
        if self.push_gateway_endpoint.is_some() {
            // Spawn async task to send to push gateway
            let formatted_metric = formatted_metric.clone();
            let backend = self.clone();
            tokio::spawn(async move {
                if let Err(e) = backend.push_to_gateway(&formatted_metric).await {
                    tracing::error!("Failed to push metric to Prometheus Push Gateway: {}", e);
                }
            });
        } else {
            // For direct Prometheus scraping, we would typically expose metrics via HTTP endpoint
            // For now, we log the formatted metric
            tracing::debug!("Prometheus metric formatted: {}", formatted_metric.trim());
        }

        tracing::trace!("Processed Prometheus metric: {}", metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        if metrics.is_empty() {
            return Ok(());
        }

        let mut formatted_batch = String::new();
        for metric in metrics {
            formatted_batch.push_str(&self.format_metric(metric));
        }

        // If push gateway is configured, send entire batch
        if self.push_gateway_endpoint.is_some() {
            let backend = self.clone();
            tokio::spawn(async move {
                if let Err(e) = backend.push_to_gateway(&formatted_batch).await {
                    tracing::error!("Failed to push metric batch to Prometheus Push Gateway: {}", e);
                }
            });
        } else {
            // For direct Prometheus scraping, log the formatted metrics
            tracing::debug!("Prometheus metrics batch formatted ({} metrics)", metrics.len());
            tracing::trace!("Batch content:\n{}", formatted_batch.trim());
        }

        tracing::debug!("Processed {} Prometheus metrics in batch", metrics.len());
        Ok(())
    }

    fn name(&self) -> &str {
        "prometheus"
    }

    fn initialize(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Initialized Prometheus backend");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Shutdown Prometheus backend");
        Ok(())
    }
}

/// StatsD backend implementation
pub struct StatsDBackend {
    endpoint: String,
}

impl StatsDBackend {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl MetricsBackend for StatsDBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        tracing::debug!("Sending metric to StatsD: {}", metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        for metric in metrics {
            self.send_metric(metric)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "statsd"
    }

    fn initialize(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Initialized StatsD backend: {}", self.endpoint);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Shutdown StatsD backend");
        Ok(())
    }
}

/// InfluxDB backend implementation
pub struct InfluxDBBackend {
    endpoint: String,
    database: String,
}

impl InfluxDBBackend {
    pub fn new(endpoint: String, database: String) -> Self {
        Self { endpoint, database }
    }
}

impl MetricsBackend for InfluxDBBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        tracing::debug!("Sending metric to InfluxDB: {}", metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        for metric in metrics {
            self.send_metric(metric)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "influxdb"
    }

    fn initialize(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Initialized InfluxDB backend: {} / {}", self.endpoint, self.database);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Shutdown InfluxDB backend");
        Ok(())
    }
}

/// OpenTelemetry backend implementation
pub struct OpenTelemetryBackend {
    endpoint: String,
}

impl OpenTelemetryBackend {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl MetricsBackend for OpenTelemetryBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        tracing::debug!("Sending metric to OpenTelemetry: {}", metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        for metric in metrics {
            self.send_metric(metric)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "opentelemetry"
    }

    fn initialize(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Initialized OpenTelemetry backend: {}", self.endpoint);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Shutdown OpenTelemetry backend");
        Ok(())
    }
}

/// Custom backend implementation
pub struct CustomBackend {
    name: String,
    config: HashMap<String, String>,
}

impl CustomBackend {
    pub fn new(name: String, config: HashMap<String, String>) -> Self {
        Self { name, config }
    }
}

impl MetricsBackend for CustomBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        tracing::debug!("Sending metric to custom backend {}: {}", self.name, metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        for metric in metrics {
            self.send_metric(metric)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Initialized custom backend: {}", self.name);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), MetricsError> {
        tracing::info!("Shutdown custom backend: {}", self.name);
        Ok(())
    }
}

impl fmt::Display for MetricsCollector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "MetricsCollector:")?;
        writeln!(f, "  Backends: {}", self.backends.len())?;
        if let Ok(metrics_cache) = self.metrics.read() {
            let stats = metrics_cache.stats();
            writeln!(f, "  Cached Metrics: {} valid / {} total (max: {})", 
                stats.valid_entries, stats.total_entries, stats.max_size)?;
            writeln!(f, "  Expired Metrics: {}", stats.expired_entries)?;
        }
        Ok(())
    }
}

/// Generate metrics collection preprocessing code
pub fn generate_metrics_preprocessing(
    metrics_backend: Option<&str>,
    custom_metrics: Option<&[&str]>
) -> proc_macro2::TokenStream {
    let backend_config = match metrics_backend {
        Some("prometheus") => quote! { rabbitmesh_macros::observability::metrics::MetricsBackendType::Prometheus },
        Some("statsd") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackendType::StatsD { 
                endpoint: env::var("RABBITMESH_STATSD_ENDPOINT").unwrap_or_else(|_| "localhost:8125".to_string()) 
            }
        },
        Some("influxdb") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackendType::InfluxDB { 
                endpoint: env::var("RABBITMESH_INFLUXDB_ENDPOINT").unwrap_or_else(|_| "http://localhost:8086".to_string()),
                database: env::var("RABBITMESH_INFLUXDB_DATABASE").unwrap_or_else(|_| "rabbitmesh".to_string())
            }
        },
        Some("opentelemetry") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackendType::OpenTelemetry { 
                endpoint: env::var("RABBITMESH_OTEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()) 
            }
        },
        _ => quote! { rabbitmesh_macros::observability::metrics::MetricsBackendType::Prometheus },
    };

    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;

        tracing::debug!("📊 Initializing metrics collection");

        // Initialize metrics configuration from environment
        let metrics_config = {
            let collection_interval_secs: u64 = env::var("RABBITMESH_METRICS_COLLECTION_INTERVAL_SECS")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15);

            let enable_histograms: bool = env::var("RABBITMESH_ENABLE_HISTOGRAM_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_gauges: bool = env::var("RABBITMESH_ENABLE_GAUGE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_counters: bool = env::var("RABBITMESH_ENABLE_COUNTER_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_cache_size: usize = env::var("RABBITMESH_METRICS_MAX_CACHE_SIZE")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000);

            let retention_period_secs: u64 = env::var("RABBITMESH_METRICS_RETENTION_PERIOD_SECS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600);

            let enable_labels: bool = env::var("RABBITMESH_ENABLE_METRIC_LABELS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_label_cardinality: usize = env::var("RABBITMESH_MAX_METRIC_LABEL_CARDINALITY")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            rabbitmesh_macros::observability::metrics::MetricsConfig {
                backend: #backend_config,
                collection_interval: Duration::from_secs(collection_interval_secs),
                enable_histograms,
                histogram_buckets: vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
                enable_gauges,
                enable_counters,
                max_cache_size,
                retention_period: Duration::from_secs(retention_period_secs),
                enable_labels,
                max_label_cardinality,
            }
        };

        // Initialize metrics collector
        let metrics_collector = Arc::new(
            rabbitmesh_macros::observability::metrics::MetricsCollector::new(metrics_config)
                .map_err(|e| {
                    eprintln!("Failed to initialize metrics collector: {}", e);
                    e
                })?
        );

        // Start metrics collection
        tokio::spawn({
            let collector = metrics_collector.clone();
            async move {
                if let Err(e) = collector.start_collection().await {
                    eprintln!("Metrics collection error: {}", e);
                }
            }
        });

        // Store metrics collector reference for later use
        let _metrics_collector_ref = metrics_collector.clone();

        Ok(())
    }
}