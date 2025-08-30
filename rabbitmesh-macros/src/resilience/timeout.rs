//! Timeout Module
//! 
//! Provides configurable timeout handling for operations with adaptive timeouts,
//! cascading timeouts, and comprehensive monitoring.

use quote::quote;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};

/// Timeout configuration for different operation types
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Default timeout duration
    pub default_timeout: Duration,
    /// Per-operation timeout overrides
    pub operation_timeouts: HashMap<String, Duration>,
    /// Enable adaptive timeout adjustment
    pub adaptive_enabled: bool,
    /// Adaptive timeout percentile (e.g., 95th percentile)
    pub adaptive_percentile: f64,
    /// Maximum adaptive timeout multiplier
    pub max_adaptive_multiplier: f64,
    /// Minimum adaptive timeout
    pub min_adaptive_timeout: Duration,
    /// Maximum adaptive timeout
    pub max_adaptive_timeout: Duration,
    /// Enable cascading timeouts for dependent operations
    pub cascading_enabled: bool,
    /// Timeout cascade ratio (child timeout = parent timeout * ratio)
    pub cascade_ratio: f64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            operation_timeouts: HashMap::new(),
            adaptive_enabled: true,
            adaptive_percentile: 0.95,
            max_adaptive_multiplier: 3.0,
            min_adaptive_timeout: Duration::from_millis(100),
            max_adaptive_timeout: Duration::from_secs(300),
            cascading_enabled: true,
            cascade_ratio: 0.8,
        }
    }
}

/// Timeout manager for handling operation timeouts
pub struct TimeoutManager {
    config: TimeoutConfig,
    metrics: TimeoutMetrics,
    operation_history: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    adaptive_timeouts: Arc<RwLock<HashMap<String, Duration>>>,
}

/// Timeout metrics for monitoring
#[derive(Debug, Default)]
pub struct TimeoutMetrics {
    pub total_operations: AtomicU64,
    pub timed_out_operations: AtomicU64,
    pub total_timeout_time: AtomicU64,
    pub adaptive_adjustments: AtomicU32,
    pub cascading_timeouts: AtomicU32,
}

/// Error types for timeout operations
#[derive(Debug, thiserror::Error)]
pub enum TimeoutError {
    #[error("Operation timed out after {duration:?}")]
    OperationTimeout { duration: Duration },
    #[error("Cascading timeout: parent operation timed out")]
    CascadingTimeout,
    #[error("Invalid timeout configuration: {message}")]
    InvalidConfig { message: String },
    #[error("Timeout manager error: {source}")]
    ManagerError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl TimeoutManager {
    /// Create a new timeout manager
    pub fn new(config: TimeoutConfig) -> Result<Self, TimeoutError> {
        if config.adaptive_percentile < 0.0 || config.adaptive_percentile > 1.0 {
            return Err(TimeoutError::InvalidConfig {
                message: "adaptive_percentile must be between 0.0 and 1.0".to_string(),
            });
        }

        if config.cascade_ratio <= 0.0 || config.cascade_ratio >= 1.0 {
            return Err(TimeoutError::InvalidConfig {
                message: "cascade_ratio must be between 0.0 and 1.0".to_string(),
            });
        }

        Ok(Self {
            config,
            metrics: TimeoutMetrics::default(),
            operation_history: Arc::new(RwLock::new(HashMap::new())),
            adaptive_timeouts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get timeout for a specific operation
    pub fn get_timeout(&self, operation_name: &str) -> Duration {
        // Check for operation-specific timeout
        if let Some(&timeout) = self.config.operation_timeouts.get(operation_name) {
            return timeout;
        }

        // Check for adaptive timeout
        if self.config.adaptive_enabled {
            if let Ok(adaptive_timeouts) = self.adaptive_timeouts.read() {
                if let Some(&adaptive_timeout) = adaptive_timeouts.get(operation_name) {
                    return adaptive_timeout;
                }
            }
        }

        self.config.default_timeout
    }

    /// Get cascaded timeout for child operation
    pub fn get_cascaded_timeout(&self, parent_timeout: Duration) -> Duration {
        if !self.config.cascading_enabled {
            return self.config.default_timeout;
        }

        let cascaded = Duration::from_nanos(
            (parent_timeout.as_nanos() as f64 * self.config.cascade_ratio) as u64
        );

        // Ensure cascaded timeout is within bounds
        cascaded.max(self.config.min_adaptive_timeout)
               .min(self.config.max_adaptive_timeout)
    }

    /// Record operation completion for adaptive timeout calculation
    pub fn record_operation(&self, operation_name: &str, duration: Duration) {
        self.metrics.total_operations.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut history) = self.operation_history.write() {
            let op_history = history.entry(operation_name.to_string()).or_insert_with(Vec::new);
            op_history.push(duration);

            // Keep only recent history (last 100 operations)
            if op_history.len() > 100 {
                op_history.remove(0);
            }

            // Update adaptive timeout if enabled
            if self.config.adaptive_enabled && op_history.len() >= 10 {
                if let Some(adaptive_timeout) = self.calculate_adaptive_timeout(op_history) {
                    if let Ok(mut adaptive_timeouts) = self.adaptive_timeouts.write() {
                        adaptive_timeouts.insert(operation_name.to_string(), adaptive_timeout);
                        self.metrics.adaptive_adjustments.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    /// Record timeout occurrence
    pub fn record_timeout(&self, operation_name: &str, timeout_duration: Duration) {
        self.metrics.timed_out_operations.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_timeout_time.fetch_add(timeout_duration.as_millis() as u64, Ordering::Relaxed);

        // For adaptive timeouts, we still want to record this as a data point
        // but with a penalty to encourage longer timeouts
        let penalty_duration = Duration::from_nanos(
            (timeout_duration.as_nanos() as f64 * 1.5) as u64
        );
        self.record_operation(operation_name, penalty_duration);
    }

    /// Calculate adaptive timeout based on historical data
    fn calculate_adaptive_timeout(&self, history: &[Duration]) -> Option<Duration> {
        if history.is_empty() {
            return None;
        }

        let mut sorted_durations: Vec<Duration> = history.to_vec();
        sorted_durations.sort();

        let index = ((history.len() - 1) as f64 * self.config.adaptive_percentile) as usize;
        let percentile_duration = sorted_durations[index];

        // Apply multiplier and bounds
        let adaptive_duration = Duration::from_nanos(
            (percentile_duration.as_nanos() as f64 * self.config.max_adaptive_multiplier) as u64
        );

        Some(adaptive_duration
            .max(self.config.min_adaptive_timeout)
            .min(self.config.max_adaptive_timeout))
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> TimeoutMetrics {
        TimeoutMetrics {
            total_operations: AtomicU64::new(self.metrics.total_operations.load(Ordering::Relaxed)),
            timed_out_operations: AtomicU64::new(self.metrics.timed_out_operations.load(Ordering::Relaxed)),
            total_timeout_time: AtomicU64::new(self.metrics.total_timeout_time.load(Ordering::Relaxed)),
            adaptive_adjustments: AtomicU32::new(self.metrics.adaptive_adjustments.load(Ordering::Relaxed)),
            cascading_timeouts: AtomicU32::new(self.metrics.cascading_timeouts.load(Ordering::Relaxed)),
        }
    }

    /// Clear metrics
    pub fn clear_metrics(&self) {
        self.metrics.total_operations.store(0, Ordering::Relaxed);
        self.metrics.timed_out_operations.store(0, Ordering::Relaxed);
        self.metrics.total_timeout_time.store(0, Ordering::Relaxed);
        self.metrics.adaptive_adjustments.store(0, Ordering::Relaxed);
        self.metrics.cascading_timeouts.store(0, Ordering::Relaxed);
    }
}

/// Timeout context for tracking nested operations
#[derive(Debug, Clone)]
pub struct TimeoutContext {
    pub operation_name: String,
    pub start_time: Instant,
    pub timeout_duration: Duration,
    pub parent_context: Option<Box<TimeoutContext>>,
}

impl TimeoutContext {
    /// Create a new timeout context
    pub fn new(operation_name: String, timeout_duration: Duration) -> Self {
        Self {
            operation_name,
            start_time: Instant::now(),
            timeout_duration,
            parent_context: None,
        }
    }

    /// Create a cascaded timeout context
    pub fn with_parent(operation_name: String, timeout_duration: Duration, parent: TimeoutContext) -> Self {
        Self {
            operation_name,
            start_time: Instant::now(),
            timeout_duration,
            parent_context: Some(Box::new(parent)),
        }
    }

    /// Check if timeout has occurred
    pub fn is_timed_out(&self) -> bool {
        self.start_time.elapsed() >= self.timeout_duration
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> Option<Duration> {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.timeout_duration {
            None
        } else {
            Some(self.timeout_duration - elapsed)
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Generate timeout preprocessing code
pub fn generate_timeout_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        use std::time::{Duration, Instant};
        use std::sync::Arc;
        use std::env;
        use std::collections::HashMap;

        // Initialize timeout configuration from environment
        let timeout_config = {
            let default_timeout_ms: u64 = env::var("RABBITMESH_DEFAULT_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000);

            let adaptive_enabled: bool = env::var("RABBITMESH_ADAPTIVE_TIMEOUT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let cascading_enabled: bool = env::var("RABBITMESH_CASCADING_TIMEOUT_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true);

            let adaptive_percentile: f64 = env::var("RABBITMESH_ADAPTIVE_PERCENTILE")
                .unwrap_or_else(|_| "0.95".to_string())
                .parse()
                .unwrap_or(0.95);

            let max_adaptive_multiplier: f64 = env::var("RABBITMESH_MAX_ADAPTIVE_MULTIPLIER")
                .unwrap_or_else(|_| "3.0".to_string())
                .parse()
                .unwrap_or(3.0);

            let cascade_ratio: f64 = env::var("RABBITMESH_CASCADE_RATIO")
                .unwrap_or_else(|_| "0.8".to_string())
                .parse()
                .unwrap_or(0.8);

            rabbitmesh_macros::resilience::timeout::TimeoutConfig {
                default_timeout: Duration::from_millis(default_timeout_ms),
                operation_timeouts: HashMap::new(),
                adaptive_enabled,
                adaptive_percentile,
                max_adaptive_multiplier,
                min_adaptive_timeout: Duration::from_millis(100),
                max_adaptive_timeout: Duration::from_secs(300),
                cascading_enabled,
                cascade_ratio,
            }
        };

        // Initialize timeout manager
        let timeout_manager = Arc::new(
            rabbitmesh_macros::resilience::timeout::TimeoutManager::new(timeout_config)
                .map_err(|e| {
                    eprintln!("Failed to initialize timeout manager: {}", e);
                    e
                })?
        );

        // Create timeout context for current operation
        let operation_name = std::any::type_name::<T>().to_string();
        let timeout_duration = timeout_manager.get_timeout(&operation_name);
        let timeout_context = rabbitmesh_macros::resilience::timeout::TimeoutContext::new(
            operation_name.clone(),
            timeout_duration
        );

        // Store timeout manager and context for later use
        let _timeout_manager_ref = timeout_manager.clone();
        let _timeout_context_ref = timeout_context.clone();

        Ok(())
    }
}