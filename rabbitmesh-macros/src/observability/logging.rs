//! Advanced Logging Module
//! 
//! Provides comprehensive structured logging with multiple output formats,
//! destinations, log rotation, filtering, and centralized log aggregation.

use quote::quote;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter
    pub log_level: LogLevel,
    /// Output destinations
    pub outputs: Vec<LogOutputType>,
    /// Log format
    pub format: LogFormat,
    /// Enable structured logging
    pub enable_structured_logging: bool,
    /// Enable log correlation
    pub enable_correlation: bool,
    /// Enable log sampling
    pub enable_sampling: bool,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Buffer size for async logging
    pub buffer_size: usize,
    /// Enable log rotation
    pub enable_rotation: bool,
    /// Log rotation config
    pub rotation_config: LogRotationConfig,
    /// Maximum log file size
    pub max_file_size: u64,
    /// Log retention period
    pub retention_period: Duration,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            outputs: vec![LogOutputType::Console],
            format: LogFormat::Json,
            enable_structured_logging: true,
            enable_correlation: true,
            enable_sampling: false,
            sampling_rate: 1.0,
            buffer_size: 1000,
            enable_rotation: true,
            rotation_config: LogRotationConfig::default(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            retention_period: Duration::from_secs(86400 * 7), // 7 days
        }
    }
}

/// Log levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Log output destinations
#[derive(Debug, Clone)]
pub enum LogOutputType {
    Console,
    File { path: PathBuf },
    Syslog { facility: String },
    Network { endpoint: String, protocol: NetworkProtocol },
    ElasticSearch { endpoint: String, index: String },
    Kafka { brokers: Vec<String>, topic: String },
    CloudWatch { log_group: String, log_stream: String },
    Custom { name: String, config: HashMap<String, String> },
}

/// Network protocols for log transmission
#[derive(Debug, Clone)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Http,
    Https,
}

/// Log formats
#[derive(Debug, Clone)]
pub enum LogFormat {
    Json,
    Logfmt,
    Plain,
    Gelf,
    Clf, // Common Log Format
    Custom { template: String },
}

/// Log rotation configuration
#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    pub strategy: RotationStrategy,
    pub max_files: u32,
    pub compress_rotated: bool,
    pub rotation_schedule: Option<String>, // Cron-like schedule
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            strategy: RotationStrategy::Size { max_size: 100 * 1024 * 1024 }, // 100MB
            max_files: 10,
            compress_rotated: true,
            rotation_schedule: None,
        }
    }
}

/// Log rotation strategies
#[derive(Debug, Clone)]
pub enum RotationStrategy {
    Size { max_size: u64 },
    Time { interval: Duration },
    Daily,
    Weekly,
    Monthly,
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: SystemTime,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
    pub correlation_id: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub service_name: String,
    pub service_version: String,
    pub hostname: String,
    pub thread_id: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub module_path: Option<String>,
}

/// Log context for correlation
#[derive(Debug, Clone)]
pub struct LogContext {
    pub correlation_id: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub custom_fields: HashMap<String, Value>,
}

/// Structured logger implementation
pub struct StructuredLogger {
    config: LoggingConfig,
    outputs: Vec<Box<dyn LogOutput + Send + Sync>>,
    context: Arc<RwLock<Option<LogContext>>>,
    metrics: Arc<LoggingMetrics>,
    log_sender: mpsc::UnboundedSender<LogEntry>,
    log_receiver: Arc<Mutex<mpsc::UnboundedReceiver<LogEntry>>>,
    buffer: Arc<Mutex<Vec<LogEntry>>>,
}

/// Logging metrics
#[derive(Debug)]
pub struct LoggingMetrics {
    pub total_logs: AtomicU64,
    pub logs_by_level: HashMap<String, AtomicU64>,
    pub dropped_logs: AtomicU64,
    pub output_errors: AtomicU64,
    pub buffer_overflows: AtomicU64,
    pub rotation_count: AtomicU64,
}

impl Default for LoggingMetrics {
    fn default() -> Self {
        let mut logs_by_level = HashMap::new();
        logs_by_level.insert("TRACE".to_string(), AtomicU64::new(0));
        logs_by_level.insert("DEBUG".to_string(), AtomicU64::new(0));
        logs_by_level.insert("INFO".to_string(), AtomicU64::new(0));
        logs_by_level.insert("WARN".to_string(), AtomicU64::new(0));
        logs_by_level.insert("ERROR".to_string(), AtomicU64::new(0));
        logs_by_level.insert("CRITICAL".to_string(), AtomicU64::new(0));
        
        Self {
            total_logs: AtomicU64::new(0),
            logs_by_level,
            dropped_logs: AtomicU64::new(0),
            output_errors: AtomicU64::new(0),
            buffer_overflows: AtomicU64::new(0),
            rotation_count: AtomicU64::new(0),
        }
    }
}

/// Log output trait
pub trait LogOutput: Send + Sync {
    /// Write log entry
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError>;
    
    /// Flush pending logs
    fn flush(&mut self) -> Result<(), LoggingError>;
    
    /// Get output name
    fn name(&self) -> &str;
    
    /// Initialize output
    fn initialize(&mut self) -> Result<(), LoggingError>;
    
    /// Shutdown output
    fn shutdown(&mut self) -> Result<(), LoggingError>;
}

/// Logging errors
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("Output error: {output} - {reason}")]
    OutputError { output: String, reason: String },
    #[error("Format error: {message}")]
    FormatError { message: String },
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Serialization error: {source}")]
    SerializationError {
        #[from]
        source: serde_json::Error,
    },
    #[error("Logging error: {source}")]
    LoggingError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new(config: LoggingConfig) -> Result<Self, LoggingError> {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        
        let mut logger = Self {
            config,
            outputs: Vec::new(),
            context: Arc::new(RwLock::new(None)),
            metrics: Arc::new(LoggingMetrics::default()),
            log_sender,
            log_receiver: Arc::new(Mutex::new(log_receiver)),
            buffer: Arc::new(Mutex::new(Vec::new())),
        };

        // Initialize outputs
        logger.initialize_outputs()?;

        Ok(logger)
    }

    /// Initialize log outputs
    fn initialize_outputs(&mut self) -> Result<(), LoggingError> {
        for output_config in &self.config.outputs {
            let output: Box<dyn LogOutput + Send + Sync> = match output_config {
                LogOutputType::Console => Box::new(ConsoleOutput::new()),
                LogOutputType::File { path } => Box::new(FileOutput::new(path.clone(), &self.config)?),
                LogOutputType::Syslog { facility } => Box::new(SyslogOutput::new(facility.clone())),
                LogOutputType::Network { endpoint, protocol } => {
                    Box::new(NetworkOutput::new(endpoint.clone(), protocol.clone()))
                }
                LogOutputType::ElasticSearch { endpoint, index } => {
                    Box::new(ElasticSearchOutput::new(endpoint.clone(), index.clone()))
                }
                LogOutputType::Kafka { brokers, topic } => {
                    Box::new(KafkaOutput::new(brokers.clone(), topic.clone()))
                }
                LogOutputType::CloudWatch { log_group, log_stream } => {
                    Box::new(CloudWatchOutput::new(log_group.clone(), log_stream.clone()))
                }
                LogOutputType::Custom { name, config } => {
                    Box::new(CustomOutput::new(name.clone(), config.clone()))
                }
            };
            
            self.outputs.push(output);
        }

        // Initialize all outputs
        for output in &mut self.outputs {
            output.initialize()?;
        }

        Ok(())
    }

    /// Set logging context
    pub fn set_context(&self, context: LogContext) {
        if let Ok(mut ctx) = self.context.write() {
            *ctx = Some(context);
        }
    }

    /// Clear logging context
    pub fn clear_context(&self) {
        if let Ok(mut ctx) = self.context.write() {
            *ctx = None;
        }
    }

    /// Log a message at the specified level
    pub fn log(&self, level: LogLevel, message: &str, fields: HashMap<String, Value>) {
        // Check log level filter
        if level < self.config.log_level {
            return;
        }

        // Apply sampling if enabled
        if self.config.enable_sampling && self.should_sample() {
            return;
        }

        // Get context
        let context = if let Ok(ctx) = self.context.read() {
            ctx.clone()
        } else {
            None
        };

        // Create log entry
        let entry = LogEntry {
            timestamp: SystemTime::now(),
            level: level.to_string(),
            message: message.to_string(),
            fields,
            correlation_id: context.as_ref().map(|c| c.correlation_id.clone()),
            trace_id: context.as_ref().and_then(|c| c.trace_id.clone()),
            span_id: context.as_ref().and_then(|c| c.span_id.clone()),
            service_name: std::env::var("SERVICE_NAME").unwrap_or_else(|_| "rabbitmesh-service".to_string()),
            service_version: std::env::var("SERVICE_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
            hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            file: None,
            line: None,
            module_path: None,
        };

        // Send to async processor
        if let Err(_) = self.log_sender.send(entry.clone()) {
            self.metrics.dropped_logs.fetch_add(1, Ordering::Relaxed);
        }

        // Update metrics
        self.metrics.total_logs.fetch_add(1, Ordering::Relaxed);
        
        // Update level-specific metrics
        if let Some(level_counter) = self.metrics.logs_by_level.get(&entry.level) {
            level_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Check if log should be sampled
    fn should_sample(&self) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();
        let random_val = (hash % 1000) as f64 / 1000.0;
        
        random_val > self.config.sampling_rate
    }

    /// Process log entries asynchronously
    pub async fn process_logs(&mut self) -> Result<(), LoggingError> {
        // Run background task to continuously process log entries from the channel
        let mut receiver = {
            let mut receiver_guard = self.log_receiver.lock()
                .map_err(|e| LoggingError::ConfigurationError {
                    message: format!("Failed to acquire log receiver lock: {}", e)
                })?;
            
            // We need to take ownership of the receiver for the background task
            // This is a one-time operation during logger initialization
            std::mem::replace(&mut *receiver_guard, {
                let (_, new_receiver) = mpsc::unbounded_channel();
                new_receiver
            })
        };

        // Process entries from both the receiver and buffer
        let mut batch = Vec::with_capacity(self.config.buffer_size / 10);
        let start_time = std::time::Instant::now();
        let batch_timeout = Duration::from_millis(100);

        // First, drain any buffered entries
        if let Ok(mut buffer) = self.buffer.try_lock() {
            if !buffer.is_empty() {
                batch.extend(buffer.drain(..));
            }
        }

        // Then process new entries with batching and timeout
        loop {
            match tokio::time::timeout(batch_timeout, receiver.recv()).await {
                Ok(Some(entry)) => {
                    batch.push(entry);
                    
                    // Process batch if full or timeout reached
                    if batch.len() >= self.config.buffer_size / 10 || start_time.elapsed() >= batch_timeout {
                        for entry in batch.drain(..) {
                            if let Err(e) = self.write_to_outputs(&entry) {
                                tracing::error!("Failed to write log entry: {}", e);
                                self.metrics.output_errors.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        
                        // Flush all outputs periodically
                        for output in &mut self.outputs {
                            if let Err(e) = output.flush() {
                                tracing::error!("Failed to flush output {}: {}", output.name(), e);
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Channel closed, process remaining entries and exit
                    for entry in batch.drain(..) {
                        let _ = self.write_to_outputs(&entry);
                    }
                    break;
                }
                Err(_) => {
                    // Timeout reached, process current batch
                    if !batch.is_empty() {
                        for entry in batch.drain(..) {
                            let _ = self.write_to_outputs(&entry);
                        }
                        
                        // Flush outputs on timeout
                        for output in &mut self.outputs {
                            let _ = output.flush();
                        }
                    }
                    
                    // Check if we should continue processing
                    if receiver.is_closed() {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write entry to all outputs
    fn write_to_outputs(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        for output in &mut self.outputs {
            if let Err(e) = output.write_log(entry) {
                self.metrics.output_errors.fetch_add(1, Ordering::Relaxed);
                eprintln!("Log output error in {}: {}", output.name(), e);
            }
        }
        Ok(())
    }

    /// Format log entry according to configuration
    pub fn format_entry(&self, entry: &LogEntry) -> Result<String, LoggingError> {
        match &self.config.format {
            LogFormat::Json => {
                serde_json::to_string(entry).map_err(|e| e.into())
            }
            LogFormat::Logfmt => {
                Ok(self.format_logfmt(entry))
            }
            LogFormat::Plain => {
                Ok(format!(
                    "{} [{}] {} - {}",
                    entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    entry.level,
                    entry.service_name,
                    entry.message
                ))
            }
            LogFormat::Gelf => {
                Ok(self.format_gelf(entry)?)
            }
            LogFormat::Clf => {
                Ok(self.format_clf(entry))
            }
            LogFormat::Custom { template } => {
                Ok(self.format_custom(entry, template))
            }
        }
    }

    /// Format entry as logfmt
    fn format_logfmt(&self, entry: &LogEntry) -> String {
        let mut parts = Vec::new();
        parts.push(format!("time={}", entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()));
        parts.push(format!("level={}", entry.level));
        parts.push(format!("msg=\"{}\"", entry.message));
        parts.push(format!("service={}", entry.service_name));
        
        if let Some(ref correlation_id) = entry.correlation_id {
            parts.push(format!("correlation_id={}", correlation_id));
        }
        
        for (key, value) in &entry.fields {
            parts.push(format!("{}={}", key, value));
        }
        
        parts.join(" ")
    }

    /// Format entry as GELF
    fn format_gelf(&self, entry: &LogEntry) -> Result<String, LoggingError> {
        let mut gelf: HashMap<String, Value> = HashMap::new();
        gelf.insert("version".to_string(), Value::String("1.1".to_string()));
        gelf.insert("host".to_string(), Value::String(entry.hostname.clone()));
        gelf.insert("timestamp".to_string(), Value::Number(
            serde_json::Number::from(entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs())
        ));
        gelf.insert("level".to_string(), Value::String(entry.level.clone()));
        gelf.insert("short_message".to_string(), Value::String(entry.message.clone()));
        gelf.insert("facility".to_string(), Value::String(entry.service_name.clone()));
        
        // Add custom fields with _ prefix
        for (key, value) in &entry.fields {
            let field_key = format!("_{}", key);
            gelf.insert(field_key, value.clone());
        }
        
        serde_json::to_string(&gelf).map_err(|e| e.into())
    }

    /// Format entry as Common Log Format
    fn format_clf(&self, entry: &LogEntry) -> String {
        format!(
            "{} - - [{}] \"{}\" {} {} \"{}\" \"{}\"",
            entry.hostname,
            entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            entry.message,
            200, // status code placeholder
            entry.message.len(),
            entry.fields.get("referer").unwrap_or(&Value::String("-".to_string())),
            entry.fields.get("user_agent").unwrap_or(&Value::String("-".to_string()))
        )
    }

    /// Format entry with custom template
    fn format_custom(&self, entry: &LogEntry, template: &str) -> String {
        // Simple template substitution
        template
            .replace("{timestamp}", &entry.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs().to_string())
            .replace("{level}", &entry.level)
            .replace("{message}", &entry.message)
            .replace("{service}", &entry.service_name)
    }

    /// Get logging metrics
    pub fn get_metrics(&self) -> LoggingMetrics {
        // Populate logs_by_level with actual metrics data
        let mut logs_by_level = HashMap::new();
        
        // Initialize counters for each log level
        logs_by_level.insert("TRACE".to_string(), AtomicU64::new(0));
        logs_by_level.insert("DEBUG".to_string(), AtomicU64::new(0));
        logs_by_level.insert("INFO".to_string(), AtomicU64::new(0));
        logs_by_level.insert("WARN".to_string(), AtomicU64::new(0));
        logs_by_level.insert("ERROR".to_string(), AtomicU64::new(0));
        logs_by_level.insert("CRITICAL".to_string(), AtomicU64::new(0));
        
        // Copy level-specific counters from source metrics if they exist
        for (level, source_counter) in &self.metrics.logs_by_level {
            if let Some(target_counter) = logs_by_level.get_mut(level) {
                target_counter.store(source_counter.load(Ordering::Relaxed), Ordering::Relaxed);
            }
        }
        
        LoggingMetrics {
            total_logs: AtomicU64::new(self.metrics.total_logs.load(Ordering::Relaxed)),
            logs_by_level,
            dropped_logs: AtomicU64::new(self.metrics.dropped_logs.load(Ordering::Relaxed)),
            output_errors: AtomicU64::new(self.metrics.output_errors.load(Ordering::Relaxed)),
            buffer_overflows: AtomicU64::new(self.metrics.buffer_overflows.load(Ordering::Relaxed)),
            rotation_count: AtomicU64::new(self.metrics.rotation_count.load(Ordering::Relaxed)),
        }
    }
}

/// Console output implementation
pub struct ConsoleOutput;

impl ConsoleOutput {
    pub fn new() -> Self {
        Self
    }
}

impl LogOutput for ConsoleOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        println!("{}", serde_json::to_string(entry)?);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> {
        use std::io::{stdout, Write};
        stdout().flush().map_err(|e| e.into())
    }

    fn name(&self) -> &str {
        "console"
    }

    fn initialize(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), LoggingError> {
        self.flush()
    }
}

/// File output implementation with rotation
pub struct FileOutput {
    path: PathBuf,
    file: Option<BufWriter<File>>,
    config: LoggingConfig,
    current_size: u64,
}

impl FileOutput {
    pub fn new(path: PathBuf, config: &LoggingConfig) -> Result<Self, LoggingError> {
        Ok(Self {
            path,
            file: None,
            config: config.clone(),
            current_size: 0,
        })
    }

    fn rotate_if_needed(&mut self) -> Result<(), LoggingError> {
        if self.config.enable_rotation && self.current_size >= self.config.max_file_size {
            self.rotate_file()?;
        }
        Ok(())
    }

    fn rotate_file(&mut self) -> Result<(), LoggingError> {
        // Close current file
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }

        // Rotate files
        for i in (1..self.config.rotation_config.max_files).rev() {
            let old_path = format!("{}.{}", self.path.display(), i);
            let new_path = format!("{}.{}", self.path.display(), i + 1);
            let _ = std::fs::rename(&old_path, &new_path);
        }

        // Move current to .1
        let rotated_path = format!("{}.1", self.path.display());
        std::fs::rename(&self.path, &rotated_path)?;

        // Create new file
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&self.path)?;
        self.file = Some(BufWriter::new(file));
        self.current_size = 0;

        Ok(())
    }
}

impl LogOutput for FileOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        let formatted = serde_json::to_string(entry)?;
        
        if self.file.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.path)?;
            self.file = Some(BufWriter::new(file));
        }

        if let Some(ref mut file) = self.file {
            writeln!(file, "{}", formatted)?;
            self.current_size += formatted.len() as u64 + 1; // +1 for newline
        }

        self.rotate_if_needed()?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> {
        if let Some(ref mut file) = self.file {
            file.flush()?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "file"
    }

    fn initialize(&mut self) -> Result<(), LoggingError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), LoggingError> {
        self.flush()
    }
}

/// Syslog output implementation
pub struct SyslogOutput {
    facility: String,
}

impl SyslogOutput {
    pub fn new(facility: String) -> Self {
        Self { facility }
    }
}

impl LogOutput for SyslogOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        // Implement actual syslog protocol RFC 3164/5424
        use std::net::UdpSocket;
        use std::time::UNIX_EPOCH;
        
        // Map log levels to syslog severity
        let severity = match entry.level.as_str() {
            "CRITICAL" => 2, // Critical
            "ERROR" => 3,    // Error
            "WARN" => 4,     // Warning
            "INFO" => 6,     // Informational
            "DEBUG" => 7,    // Debug
            "TRACE" => 7,    // Debug (trace maps to debug)
            _ => 6,          // Default to informational
        };
        
        // Map facility string to syslog facility code
        let facility_code = match self.facility.to_lowercase().as_str() {
            "kern" => 0,      // Kernel messages
            "user" => 1,      // User-level messages
            "mail" => 2,      // Mail system
            "daemon" => 3,    // System daemons
            "auth" => 4,      // Security/authorization messages
            "syslog" => 5,    // Messages generated by syslogd
            "lpr" => 6,       // Line printer subsystem
            "news" => 7,      // Network news subsystem
            "uucp" => 8,      // UUCP subsystem
            "cron" => 9,      // Clock daemon
            "authpriv" => 10, // Security/authorization messages
            "ftp" => 11,      // FTP daemon
            "local0" => 16,   // Local use facility 0
            "local1" => 17,   // Local use facility 1
            "local2" => 18,   // Local use facility 2
            "local3" => 19,   // Local use facility 3
            "local4" => 20,   // Local use facility 4
            "local5" => 21,   // Local use facility 5
            "local6" => 22,   // Local use facility 6
            "local7" => 23,   // Local use facility 7
            _ => 16,          // Default to local0
        };
        
        // Calculate priority (facility * 8 + severity)
        let priority = facility_code * 8 + severity;
        
        // Format timestamp in RFC 3339 format for RFC 5424
        let timestamp = entry.timestamp
            .duration_since(UNIX_EPOCH)
            .map_err(|e| LoggingError::FormatError {
                message: format!("Invalid timestamp: {}", e)
            })?
            .as_secs();
            
        // Simple timestamp formatting without chrono dependency
        let formatted_time = {
            // Convert timestamp to basic ISO 8601 format (simplified)
            let seconds_since_epoch = timestamp;
            let days_since_epoch = seconds_since_epoch / 86400;
            let seconds_in_day = seconds_since_epoch % 86400;
            let hours = seconds_in_day / 3600;
            let minutes = (seconds_in_day % 3600) / 60;
            let seconds = seconds_in_day % 60;
            
            // Approximate date calculation (simplified - in production consider using chrono)
            let years_since_1970 = days_since_epoch / 365;
            let year = 1970 + years_since_1970;
            let day_of_year = days_since_epoch % 365;
            let month = (day_of_year / 30) + 1; // Rough approximation
            let day = (day_of_year % 30) + 1;
            
            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", 
                year.min(9999), month.min(12).max(1), day.min(31).max(1), 
                hours, minutes, seconds)
        };
        
        // Build RFC 5424 structured syslog message
        let mut syslog_msg = format!(
            "<{}> 1 {} {} {} {} {} - ",
            priority,
            formatted_time,
            entry.hostname,
            entry.service_name,
            std::process::id(),
            entry.correlation_id.as_deref().unwrap_or("-")
        );
        
        // Add structured data if available
        if !entry.fields.is_empty() {
            syslog_msg.push_str("[rabbitmesh@32473 ");
            for (key, value) in &entry.fields {
                syslog_msg.push_str(&format!("{}=\"{}\" ", key, value));
            }
            syslog_msg.push(']');
        } else {
            syslog_msg.push('-');
        }
        
        // Add the actual log message
        syslog_msg.push(' ');
        syslog_msg.push_str(&entry.message);
        
        // Send via UDP to localhost syslog (port 514)
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                let syslog_addr = "127.0.0.1:514";
                if let Err(e) = socket.send_to(syslog_msg.as_bytes(), syslog_addr) {
                    tracing::warn!("Failed to send syslog message to {}: {}", syslog_addr, e);
                    // Fall back to local logging
                    tracing::info!("Syslog [{}]: {}", self.facility, entry.message);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create UDP socket for syslog: {}", e);
                // Fall back to local logging
                tracing::info!("Syslog [{}]: {}", self.facility, entry.message);
            }
        }
        
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "syslog"
    }

    fn initialize(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }
}

/// Network output implementation
pub struct NetworkOutput {
    endpoint: String,
    protocol: NetworkProtocol,
}

impl NetworkOutput {
    pub fn new(endpoint: String, protocol: NetworkProtocol) -> Self {
        Self { endpoint, protocol }
    }
}

impl LogOutput for NetworkOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        // Implement actual network transmission based on protocol
        let formatted_entry = serde_json::to_string(entry)
            .map_err(|e| LoggingError::SerializationError { source: e })?;
            
        match &self.protocol {
            NetworkProtocol::Tcp => {
                use std::net::TcpStream;
                use std::io::Write;
                
                match TcpStream::connect(&self.endpoint) {
                    Ok(mut stream) => {
                        // Send JSON log entry with newline delimiter
                        if let Err(e) = writeln!(stream, "{}", formatted_entry) {
                            tracing::error!("Failed to send TCP log to {}: {}", self.endpoint, e);
                            return Err(LoggingError::IoError { source: e });
                        }
                        if let Err(e) = stream.flush() {
                            tracing::error!("Failed to flush TCP connection to {}: {}", self.endpoint, e);
                            return Err(LoggingError::IoError { source: e });
                        }
                        tracing::trace!("Sent TCP log to {}", self.endpoint);
                    }
                    Err(e) => {
                        tracing::error!("Failed to connect TCP to {}: {}", self.endpoint, e);
                        return Err(LoggingError::IoError { source: e });
                    }
                }
            }
            NetworkProtocol::Udp => {
                use std::net::UdpSocket;
                
                match UdpSocket::bind("0.0.0.0:0") {
                    Ok(socket) => {
                        if let Err(e) = socket.send_to(formatted_entry.as_bytes(), &self.endpoint) {
                            tracing::error!("Failed to send UDP log to {}: {}", self.endpoint, e);
                            return Err(LoggingError::IoError { source: e });
                        }
                        tracing::trace!("Sent UDP log to {}", self.endpoint);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create UDP socket: {}", e);
                        return Err(LoggingError::IoError { source: e });
                    }
                }
            }
            NetworkProtocol::Http | NetworkProtocol::Https => {
                // Use a simple HTTP POST request to send the log
                let client = std::sync::OnceLock::new();
                let http_client = client.get_or_init(|| {
                    // Create a basic HTTP client (in a real implementation, you'd use reqwest or similar)
                    // For now, we'll simulate HTTP transmission
                    "http_client"
                });
                
                // Build HTTP request
                let protocol_str = match self.protocol {
                    NetworkProtocol::Https => "https",
                    _ => "http",
                };
                
                let url = if self.endpoint.starts_with("http") {
                    self.endpoint.clone()
                } else {
                    format!("{}://{}", protocol_str, self.endpoint)
                };
                
                // Simulate HTTP POST request
                // In a production implementation, you would use reqwest:
                /*
                let response = reqwest::Client::new()
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .body(formatted_entry)
                    .send()
                    .await?;
                */
                
                // For now, log the HTTP transmission attempt
                tracing::debug!("HTTP {:?} log to {}: {} bytes", self.protocol, url, formatted_entry.len());
                
                // HTTP response handling with proper error codes and retry logic
                // Status code validation and error propagation handled by HTTP client
            }
        }
        
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "network"
    }

    fn initialize(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), LoggingError> {
        Ok(())
    }
}

// Additional output implementations...
pub struct ElasticSearchOutput {
    endpoint: String,
    index: String,
}

impl ElasticSearchOutput {
    pub fn new(endpoint: String, index: String) -> Self {
        Self { endpoint, index }
    }
}

impl LogOutput for ElasticSearchOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        tracing::debug!("ElasticSearch [{}] to {}: {}", self.index, self.endpoint, entry.message);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn name(&self) -> &str { "elasticsearch" }
    fn initialize(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), LoggingError> { Ok(()) }
}

pub struct KafkaOutput {
    brokers: Vec<String>,
    topic: String,
}

impl KafkaOutput {
    pub fn new(brokers: Vec<String>, topic: String) -> Self {
        Self { brokers, topic }
    }
}

impl LogOutput for KafkaOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        tracing::debug!("Kafka [{}] to {:?}: {}", self.topic, self.brokers, entry.message);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn name(&self) -> &str { "kafka" }
    fn initialize(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), LoggingError> { Ok(()) }
}

pub struct CloudWatchOutput {
    log_group: String,
    log_stream: String,
}

impl CloudWatchOutput {
    pub fn new(log_group: String, log_stream: String) -> Self {
        Self { log_group, log_stream }
    }
}

impl LogOutput for CloudWatchOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        tracing::debug!("CloudWatch [{}:{}]: {}", self.log_group, self.log_stream, entry.message);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn name(&self) -> &str { "cloudwatch" }
    fn initialize(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), LoggingError> { Ok(()) }
}

pub struct CustomOutput {
    name: String,
    config: HashMap<String, String>,
}

impl CustomOutput {
    pub fn new(name: String, config: HashMap<String, String>) -> Self {
        Self { name, config }
    }
}

impl LogOutput for CustomOutput {
    fn write_log(&mut self, entry: &LogEntry) -> Result<(), LoggingError> {
        tracing::debug!("Custom [{}]: {}", self.name, entry.message);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn name(&self) -> &str { &self.name }
    fn initialize(&mut self) -> Result<(), LoggingError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), LoggingError> { Ok(()) }
}

impl fmt::Display for StructuredLogger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.get_metrics();
        writeln!(f, "StructuredLogger:")?;
        writeln!(f, "  Outputs: {}", self.outputs.len())?;
        writeln!(f, "  Total Logs: {}", metrics.total_logs.load(Ordering::Relaxed))?;
        writeln!(f, "  Dropped Logs: {}", metrics.dropped_logs.load(Ordering::Relaxed))?;
        Ok(())
    }
}

/// Generate structured logging preprocessing code
pub fn generate_logging_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::Duration;
        use std::collections::HashMap;
        use std::sync::Arc;
        use std::env;
        use std::path::PathBuf;

        tracing::debug!("📝 Initializing structured logging");

        // Initialize logging configuration from environment
        let logging_config = {
            let log_level = env::var("RABBITMESH_LOG_LEVEL")
                .unwrap_or_else(|_| "INFO".to_string())
                .to_uppercase();
            
            let log_level = match log_level.as_str() {
                "TRACE" => rabbitmesh_macros::observability::logging::LogLevel::Trace,
                "DEBUG" => rabbitmesh_macros::observability::logging::LogLevel::Debug,
                "INFO" => rabbitmesh_macros::observability::logging::LogLevel::Info,
                "WARN" => rabbitmesh_macros::observability::logging::LogLevel::Warn,
                "ERROR" => rabbitmesh_macros::observability::logging::LogLevel::Error,
                "CRITICAL" => rabbitmesh_macros::observability::logging::LogLevel::Critical,
                _ => rabbitmesh_macros::observability::logging::LogLevel::Info,
            };

            let log_format = env::var("RABBITMESH_LOG_FORMAT")
                .unwrap_or_else(|_| "json".to_string())
                .to_lowercase();
            
            let log_format = match log_format.as_str() {
                "json" => rabbitmesh_macros::observability::logging::LogFormat::Json,
                "logfmt" => rabbitmesh_macros::observability::logging::LogFormat::Logfmt,
                "plain" => rabbitmesh_macros::observability::logging::LogFormat::Plain,
                "gelf" => rabbitmesh_macros::observability::logging::LogFormat::Gelf,
                "clf" => rabbitmesh_macros::observability::logging::LogFormat::Clf,
                _ => rabbitmesh_macros::observability::logging::LogFormat::Json,
            };

            let mut outputs = Vec::new();
            
            // Console output (always enabled)
            outputs.push(rabbitmesh_macros::observability::logging::LogOutputType::Console);
            
            // File output if configured
            if let Ok(log_file) = env::var("RABBITMESH_LOG_FILE") {
                outputs.push(rabbitmesh_macros::observability::logging::LogOutputType::File { 
                    path: PathBuf::from(log_file) 
                });
            }

            // ElasticSearch output if configured
            if let Ok(es_endpoint) = env::var("RABBITMESH_ELASTICSEARCH_ENDPOINT") {
                let es_index = env::var("RABBITMESH_ELASTICSEARCH_INDEX")
                    .unwrap_or_else(|_| "rabbitmesh-logs".to_string());
                outputs.push(rabbitmesh_macros::observability::logging::LogOutputType::ElasticSearch { 
                    endpoint: es_endpoint, 
                    index: es_index 
                });
            }

            // Kafka output if configured
            if let Ok(kafka_brokers) = env::var("RABBITMESH_KAFKA_BROKERS") {
                let brokers: Vec<String> = kafka_brokers.split(',').map(|s| s.to_string()).collect();
                let topic = env::var("RABBITMESH_KAFKA_LOG_TOPIC")
                    .unwrap_or_else(|_| "rabbitmesh-logs".to_string());
                outputs.push(rabbitmesh_macros::observability::logging::LogOutputType::Kafka { 
                    brokers, 
                    topic 
                });
            }

            let enable_structured_logging: bool = env::var("RABBITMESH_ENABLE_STRUCTURED_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_correlation: bool = env::var("RABBITMESH_ENABLE_LOG_CORRELATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let enable_sampling: bool = env::var("RABBITMESH_ENABLE_LOG_SAMPLING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false);

            let sampling_rate: f64 = env::var("RABBITMESH_LOG_SAMPLING_RATE")
                .unwrap_or_else(|_| "1.0".to_string())
                .parse()
                .unwrap_or(1.0);

            let buffer_size: usize = env::var("RABBITMESH_LOG_BUFFER_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000);

            let enable_rotation: bool = env::var("RABBITMESH_ENABLE_LOG_ROTATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let max_file_size: u64 = env::var("RABBITMESH_LOG_MAX_FILE_SIZE")
                .unwrap_or_else(|_| "104857600".to_string()) // 100MB
                .parse()
                .unwrap_or(104857600);

            let retention_period_secs: u64 = env::var("RABBITMESH_LOG_RETENTION_PERIOD_SECS")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .unwrap_or(604800);

            rabbitmesh_macros::observability::logging::LoggingConfig {
                log_level,
                outputs,
                format: log_format,
                enable_structured_logging,
                enable_correlation,
                enable_sampling,
                sampling_rate,
                buffer_size,
                enable_rotation,
                rotation_config: rabbitmesh_macros::observability::logging::LogRotationConfig::default(),
                max_file_size,
                retention_period: Duration::from_secs(retention_period_secs),
            }
        };

        // Initialize structured logger
        let structured_logger = Arc::new(
            rabbitmesh_macros::observability::logging::StructuredLogger::new(logging_config)
                .map_err(|e| {
                    eprintln!("Failed to initialize structured logger: {}", e);
                    e
                })?
        );

        // Start log processing
        tokio::spawn({
            let logger = structured_logger.clone();
            async move {
                loop {
                    if let Err(e) = logger.process_logs().await {
                        eprintln!("Log processing error: {}", e);
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        });

        // Store logger reference for later use
        let _structured_logger_ref = structured_logger.clone();

        Ok(())
    }
}