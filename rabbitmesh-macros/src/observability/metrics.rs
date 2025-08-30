//! Metrics Collection Module
//! 
//! Provides comprehensive metrics collection with support for multiple backends
//! including Prometheus, StatsD, InfluxDB, and custom metrics systems.

use quote::quote;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, AtomicI64, AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};
use std::fmt;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Metrics collection backend
    pub backend: MetricsBackend,
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
            backend: MetricsBackend::Prometheus,
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
pub enum MetricsBackend {
    Prometheus,
    StatsD { endpoint: String },
    InfluxDB { endpoint: String, database: String },
    OpenTelemetry { endpoint: String },
    Custom { name: String, config: HashMap<String, String> },
    Multiple(Vec<MetricsBackend>),
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

/// Metrics collector for gathering system metrics
pub struct MetricsCollector {
    config: MetricsConfig,
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
    backends: Vec<Box<dyn MetricsBackend + Send + Sync>>,
    system_metrics: SystemMetrics,
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
    collection_sender: mpsc::UnboundedSender<MetricEvent>,
    collection_receiver: Arc<Mutex<mpsc::UnboundedReceiver<MetricEvent>>>,
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
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            backends: Vec::new(),
            system_metrics: SystemMetrics::default(),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_sender,
            collection_receiver: Arc::new(Mutex::new(collection_receiver)),
        };

        // Initialize backends based on configuration
        collector.initialize_backends()?;

        Ok(collector)
    }

    /// Initialize metrics backends
    fn initialize_backends(&mut self) -> Result<(), MetricsError> {
        match &self.config.backend {
            MetricsBackend::Prometheus => {
                let backend = PrometheusBackend::new();
                self.backends.push(Box::new(backend));
            }
            MetricsBackend::StatsD { endpoint } => {
                let backend = StatsDBackend::new(endpoint.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackend::InfluxDB { endpoint, database } => {
                let backend = InfluxDBBackend::new(endpoint.clone(), database.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackend::OpenTelemetry { endpoint } => {
                let backend = OpenTelemetryBackend::new(endpoint.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackend::Custom { name, config } => {
                let backend = CustomBackend::new(name.clone(), config.clone());
                self.backends.push(Box::new(backend));
            }
            MetricsBackend::Multiple(backends) => {
                for backend_config in backends {
                    // Recursively initialize multiple backends
                    // In a real implementation, this would handle the recursion properly
                    tracing::info!("Initializing multiple backend: {:?}", backend_config);
                }
            }
        }

        // Initialize all backends
        for backend in &mut self.backends {
            backend.initialize()?;
        }

        Ok(())
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
        // In a real implementation, this would collect actual system metrics
        // For now, simulate collection
        self.system_metrics.cpu_usage.store(75, Ordering::Relaxed);
        self.system_metrics.memory_usage.store(1024 * 1024 * 512, Ordering::Relaxed); // 512MB
        self.system_metrics.active_threads.store(10, Ordering::Relaxed);

        // Create system metrics
        let cpu_metric = Metric {
            name: "system_cpu_usage_percent".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(self.system_metrics.cpu_usage.load(Ordering::Relaxed) as f64),
            labels: HashMap::new(),
            help: Some("CPU usage percentage".to_string()),
            timestamp: SystemTime::now(),
        };

        let memory_metric = Metric {
            name: "system_memory_usage_bytes".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(self.system_metrics.memory_usage.load(Ordering::Relaxed) as f64),
            labels: HashMap::new(),
            help: Some("Memory usage in bytes".to_string()),
            timestamp: SystemTime::now(),
        };

        // Send to backends
        for backend in &self.backends {
            backend.send_metric(&cpu_metric)?;
            backend.send_metric(&memory_metric)?;
        }

        Ok(())
    }

    /// Process metric events
    async fn process_metric_events(&self) -> Result<(), MetricsError> {
        // In a real implementation, this would run in a background task
        // processing events from the channel
        tracing::debug!("Processing metric events");
        Ok(())
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

        // Cache metric
        if let Ok(mut metrics) = self.metrics.write() {
            if metrics.len() >= self.config.max_cache_size {
                self.evict_old_metrics(&mut metrics);
            }
            metrics.insert(metric.name.clone(), metric.clone());
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

    /// Evict old metrics from cache
    fn evict_old_metrics(&self, metrics: &mut HashMap<String, Metric>) {
        // Simple eviction - remove half the metrics
        // In a real implementation, this would use proper LRU or time-based eviction
        let to_remove = metrics.len() / 2;
        let keys: Vec<String> = metrics.keys().take(to_remove).cloned().collect();
        
        for key in keys {
            metrics.remove(&key);
        }
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> HashMap<String, Metric> {
        if let Ok(metrics) = self.metrics.read() {
            metrics.clone()
        } else {
            HashMap::new()
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
pub struct PrometheusBackend {
    endpoint: String,
}

impl PrometheusBackend {
    pub fn new() -> Self {
        Self {
            endpoint: "http://localhost:9090".to_string(),
        }
    }
}

impl MetricsBackend for PrometheusBackend {
    fn send_metric(&self, metric: &Metric) -> Result<(), MetricsError> {
        // In a real implementation, this would send to Prometheus
        tracing::debug!("Sending metric to Prometheus: {}", metric.name);
        Ok(())
    }

    fn send_batch(&self, metrics: &[Metric]) -> Result<(), MetricsError> {
        for metric in metrics {
            self.send_metric(metric)?;
        }
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
        if let Ok(metrics) = self.metrics.read() {
            writeln!(f, "  Cached Metrics: {}", metrics.len())?;
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
        Some("prometheus") => quote! { rabbitmesh_macros::observability::metrics::MetricsBackend::Prometheus },
        Some("statsd") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackend::StatsD { 
                endpoint: env::var("RABBITMESH_STATSD_ENDPOINT").unwrap_or_else(|_| "localhost:8125".to_string()) 
            }
        },
        Some("influxdb") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackend::InfluxDB { 
                endpoint: env::var("RABBITMESH_INFLUXDB_ENDPOINT").unwrap_or_else(|_| "http://localhost:8086".to_string()),
                database: env::var("RABBITMESH_INFLUXDB_DATABASE").unwrap_or_else(|_| "rabbitmesh".to_string())
            }
        },
        Some("opentelemetry") => quote! { 
            rabbitmesh_macros::observability::metrics::MetricsBackend::OpenTelemetry { 
                endpoint: env::var("RABBITMESH_OTEL_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string()) 
            }
        },
        _ => quote! { rabbitmesh_macros::observability::metrics::MetricsBackend::Prometheus },
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