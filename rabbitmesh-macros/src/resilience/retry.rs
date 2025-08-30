//! Retry Logic Module
//! 
//! Provides comprehensive retry mechanisms with exponential backoff, jitter,
//! circuit breaker integration, and configurable retry policies.

use quote::quote;

/// Generate retry logic preprocessing code
pub fn generate_retry_preprocessing(
    max_retries: Option<u32>,
    initial_delay: Option<u64>,
    max_delay: Option<u64>,
    retry_strategy: Option<&str>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🔄 Initializing retry logic");
        
        // Create retry manager with configuration
        let retry_config = RetryConfig {
            max_retries: #max_retries.unwrap_or(3),
            initial_delay: std::time::Duration::from_millis(#initial_delay.unwrap_or(100)),
            max_delay: std::time::Duration::from_secs(#max_delay.unwrap_or(30)),
            strategy: match #retry_strategy.unwrap_or("exponential") {
                "linear" => RetryStrategy::Linear,
                "fixed" => RetryStrategy::Fixed,
                "exponential_jitter" => RetryStrategy::ExponentialWithJitter,
                "fibonacci" => RetryStrategy::Fibonacci,
                "custom" => RetryStrategy::Custom,
                _ => RetryStrategy::Exponential,
            },
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable_errors: get_retryable_errors(),
            non_retryable_errors: get_non_retryable_errors(),
            enable_circuit_breaker_integration: true,
            circuit_breaker_key: format!("{}:{}", 
                std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()),
                extract_method_name(&msg)
            ),
        };
        
        let retry_manager = RetryManager::new(retry_config);
        
        tracing::debug!("🔄 Retry config - Max: {}, Initial: {}ms, Strategy: {:?}", 
            retry_manager.config.max_retries, 
            retry_manager.config.initial_delay.as_millis(),
            retry_manager.config.strategy
        );
        
        /// Retry Manager for handling retry logic
        struct RetryManager {
            config: RetryConfig,
            metrics: Arc<RetryMetrics>,
        }
        
        /// Retry configuration
        #[derive(Debug, Clone)]
        struct RetryConfig {
            max_retries: u32,
            initial_delay: std::time::Duration,
            max_delay: std::time::Duration,
            strategy: RetryStrategy,
            backoff_multiplier: f64,
            jitter_factor: f64,
            retryable_errors: Vec<String>,
            non_retryable_errors: Vec<String>,
            enable_circuit_breaker_integration: bool,
            circuit_breaker_key: String,
        }
        
        /// Retry strategies
        #[derive(Debug, Clone, PartialEq)]
        enum RetryStrategy {
            Fixed,                    // Fixed delay between retries
            Linear,                   // Linear increase in delay
            Exponential,              // Exponential backoff
            ExponentialWithJitter,    // Exponential with randomized jitter
            Fibonacci,                // Fibonacci sequence delays
            Custom,                   // Custom strategy from environment
        }
        
        /// Retry context for tracking retry attempts
        #[derive(Debug)]
        struct RetryContext {
            attempt: u32,
            total_elapsed: std::time::Duration,
            last_error: Option<String>,
            retry_reason: RetryReason,
            start_time: std::time::Instant,
            delays: Vec<std::time::Duration>,
        }
        
        /// Reasons for retry
        #[derive(Debug, Clone, PartialEq)]
        enum RetryReason {
            TransientError,
            Timeout,
            NetworkError,
            ServiceUnavailable,
            RateLimited,
            CircuitBreakerOpen,
            Custom(String),
        }
        
        /// Retry decision
        #[derive(Debug)]
        enum RetryDecision {
            Retry {
                delay: std::time::Duration,
                reason: RetryReason,
            },
            Stop {
                reason: String,
            },
        }
        
        /// Retry outcome
        #[derive(Debug)]
        enum RetryOutcome<T> {
            Success(T),
            ExhaustedRetries {
                attempts: u32,
                last_error: String,
                total_elapsed: std::time::Duration,
            },
            NonRetryableError {
                error: String,
                attempt: u32,
            },
        }
        
        /// Retry metrics
        #[derive(Debug)]
        struct RetryMetrics {
            total_retries: AtomicU64,
            successful_retries: AtomicU64,
            exhausted_retries: AtomicU64,
            non_retryable_errors: AtomicU64,
            total_retry_time: AtomicU64,
            retry_counts_by_reason: Arc<RwLock<std::collections::HashMap<String, u64>>>,
        }
        
        impl RetryManager {
            /// Create new retry manager
            fn new(config: RetryConfig) -> Self {
                let metrics = Arc::new(RetryMetrics {
                    total_retries: AtomicU64::new(0),
                    successful_retries: AtomicU64::new(0),
                    exhausted_retries: AtomicU64::new(0),
                    non_retryable_errors: AtomicU64::new(0),
                    total_retry_time: AtomicU64::new(0),
                    retry_counts_by_reason: Arc::new(RwLock::new(std::collections::HashMap::new())),
                });
                
                Self { config, metrics }
            }
            
            /// Execute operation with retry logic
            async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> RetryOutcome<T>
            where
                F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send,
                E: std::fmt::Display + Send,
            {
                let mut context = RetryContext {
                    attempt: 0,
                    total_elapsed: std::time::Duration::from_secs(0),
                    last_error: None,
                    retry_reason: RetryReason::TransientError,
                    start_time: std::time::Instant::now(),
                    delays: Vec::new(),
                };
                
                loop {
                    context.attempt += 1;
                    let attempt_start = std::time::Instant::now();
                    
                    tracing::debug!("🔄 Retry attempt {} for operation", context.attempt);
                    
                    // Execute the operation
                    match operation().await {
                        Ok(result) => {
                            let total_time = context.start_time.elapsed();
                            
                            if context.attempt > 1 {
                                self.metrics.successful_retries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                self.metrics.total_retry_time.fetch_add(
                                    total_time.as_millis() as u64, 
                                    std::sync::atomic::Ordering::Relaxed
                                );
                                
                                tracing::info!("✅ Operation succeeded after {} attempts in {}ms", 
                                    context.attempt, total_time.as_millis());
                            }
                            
                            return RetryOutcome::Success(result);
                        }
                        Err(error) => {
                            let error_str = error.to_string();
                            context.last_error = Some(error_str.clone());
                            context.total_elapsed = context.start_time.elapsed();
                            
                            // Determine if error is retryable
                            let retry_reason = self.classify_error(&error_str);
                            context.retry_reason = retry_reason.clone();
                            
                            if !self.is_retryable_error(&error_str, &retry_reason) {
                                tracing::warn!("❌ Non-retryable error: {}", error_str);
                                self.metrics.non_retryable_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                return RetryOutcome::NonRetryableError {
                                    error: error_str,
                                    attempt: context.attempt,
                                };
                            }
                            
                            // Check if we should retry
                            let retry_decision = self.should_retry(&context).await;
                            
                            match retry_decision {
                                RetryDecision::Retry { delay, reason } => {
                                    context.delays.push(delay);
                                    self.metrics.total_retries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    
                                    // Update retry count by reason
                                    {
                                        let mut counts = self.metrics.retry_counts_by_reason.write().await;
                                        *counts.entry(format!("{:?}", reason)).or_insert(0) += 1;
                                    }
                                    
                                    tracing::warn!("⏳ Retrying in {}ms due to: {} (attempt {} of {})", 
                                        delay.as_millis(), error_str, context.attempt, self.config.max_retries + 1);
                                    
                                    // Apply backoff delay
                                    tokio::time::sleep(delay).await;
                                }
                                RetryDecision::Stop { reason } => {
                                    self.metrics.exhausted_retries.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    
                                    tracing::error!("❌ Retry exhausted: {} after {} attempts in {}ms", 
                                        reason, context.attempt, context.total_elapsed.as_millis());
                                    
                                    return RetryOutcome::ExhaustedRetries {
                                        attempts: context.attempt,
                                        last_error: error_str,
                                        total_elapsed: context.total_elapsed,
                                    };
                                }
                            }
                        }
                    }
                }
            }
            
            /// Determine if operation should be retried
            async fn should_retry(&self, context: &RetryContext) -> RetryDecision {
                // Check retry limit
                if context.attempt >= self.config.max_retries + 1 { // +1 because first attempt is not a retry
                    return RetryDecision::Stop {
                        reason: format!("Maximum retries ({}) exceeded", self.config.max_retries),
                    };
                }
                
                // Check circuit breaker if integration is enabled
                if self.config.enable_circuit_breaker_integration {
                    if let Ok(cb_manager) = super::circuit_breaker::CircuitBreakerManager::get_or_create(&self.config.circuit_breaker_key).await {
                        let cb_state = cb_manager.get_state().await;
                        if cb_state == super::circuit_breaker::CircuitBreakerState::Open {
                            return RetryDecision::Stop {
                                reason: "Circuit breaker is open".to_string(),
                            };
                        }
                    }
                }
                
                // Calculate delay based on strategy
                let delay = self.calculate_delay(context.attempt - 1, &context.delays);
                
                // Check if delay would exceed maximum
                if delay > self.config.max_delay {
                    return RetryDecision::Stop {
                        reason: format!("Calculated delay ({}ms) exceeds maximum ({}ms)", 
                            delay.as_millis(), self.config.max_delay.as_millis()),
                    };
                }
                
                RetryDecision::Retry {
                    delay,
                    reason: context.retry_reason.clone(),
                }
            }
            
            /// Calculate delay for next retry based on strategy
            fn calculate_delay(&self, retry_count: u32, previous_delays: &[std::time::Duration]) -> std::time::Duration {
                let base_delay = match self.config.strategy {
                    RetryStrategy::Fixed => {
                        self.config.initial_delay
                    }
                    RetryStrategy::Linear => {
                        self.config.initial_delay * (retry_count + 1)
                    }
                    RetryStrategy::Exponential => {
                        let delay_ms = self.config.initial_delay.as_millis() as f64 
                            * self.config.backoff_multiplier.powi(retry_count as i32);
                        std::time::Duration::from_millis(delay_ms as u64)
                    }
                    RetryStrategy::ExponentialWithJitter => {
                        let base_delay_ms = self.config.initial_delay.as_millis() as f64 
                            * self.config.backoff_multiplier.powi(retry_count as i32);
                        
                        // Add jitter
                        let jitter = base_delay_ms * self.config.jitter_factor * (rand::random::<f64>() - 0.5);
                        let final_delay_ms = (base_delay_ms + jitter).max(0.0);
                        
                        std::time::Duration::from_millis(final_delay_ms as u64)
                    }
                    RetryStrategy::Fibonacci => {
                        let fib_value = fibonacci(retry_count + 1) as u64;
                        self.config.initial_delay * fib_value
                    }
                    RetryStrategy::Custom => {
                        // Load custom delay calculation from environment
                        self.calculate_custom_delay(retry_count)
                    }
                };
                
                // Ensure delay doesn't exceed maximum
                std::cmp::min(base_delay, self.config.max_delay)
            }
            
            /// Calculate custom delay based on environment configuration
            fn calculate_custom_delay(&self, retry_count: u32) -> std::time::Duration {
                // Custom delay formula can be configured via environment
                // Format: "100,200,500,1000" (comma-separated delays in ms)
                let custom_delays = std::env::var("RETRY_CUSTOM_DELAYS")
                    .unwrap_or_else(|_| "100,200,500,1000,2000".to_string());
                
                let delays: Vec<u64> = custom_delays
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                
                if delays.is_empty() {
                    return self.config.initial_delay;
                }
                
                let delay_ms = if retry_count < delays.len() as u32 {
                    delays[retry_count as usize]
                } else {
                    *delays.last().unwrap() // Use last delay for subsequent retries
                };
                
                std::time::Duration::from_millis(delay_ms)
            }
            
            /// Classify error to determine retry reason
            fn classify_error(&self, error: &str) -> RetryReason {
                let error_lower = error.to_lowercase();
                
                if error_lower.contains("timeout") || error_lower.contains("timed out") {
                    RetryReason::Timeout
                } else if error_lower.contains("network") || error_lower.contains("connection") 
                    || error_lower.contains("dns") || error_lower.contains("resolve") {
                    RetryReason::NetworkError
                } else if error_lower.contains("unavailable") || error_lower.contains("503") {
                    RetryReason::ServiceUnavailable
                } else if error_lower.contains("rate limit") || error_lower.contains("429") 
                    || error_lower.contains("too many requests") {
                    RetryReason::RateLimited
                } else if error_lower.contains("circuit") && error_lower.contains("open") {
                    RetryReason::CircuitBreakerOpen
                } else if self.is_transient_error(error) {
                    RetryReason::TransientError
                } else {
                    RetryReason::Custom(error.to_string())
                }
            }
            
            /// Check if error is retryable
            fn is_retryable_error(&self, error: &str, reason: &RetryReason) -> bool {
                // Check non-retryable errors first
                for non_retryable in &self.config.non_retryable_errors {
                    if error.contains(non_retryable) {
                        return false;
                    }
                }
                
                // Check retryable errors
                for retryable in &self.config.retryable_errors {
                    if error.contains(retryable) {
                        return true;
                    }
                }
                
                // Default retryable reasons
                match reason {
                    RetryReason::TransientError | RetryReason::Timeout | 
                    RetryReason::NetworkError | RetryReason::ServiceUnavailable |
                    RetryReason::RateLimited => true,
                    RetryReason::CircuitBreakerOpen => false, // Let circuit breaker handle this
                    RetryReason::Custom(_) => self.is_transient_error(error),
                }
            }
            
            /// Check if error is considered transient
            fn is_transient_error(&self, error: &str) -> bool {
                let transient_indicators = [
                    "temporary", "transient", "temporary failure", "try again",
                    "server error", "5xx", "500", "502", "503", "504",
                    "internal error", "database lock", "deadlock",
                ];
                
                let error_lower = error.to_lowercase();
                transient_indicators.iter().any(|indicator| error_lower.contains(indicator))
            }
            
            /// Get retry metrics
            async fn get_metrics(&self) -> RetryMetricsSnapshot {
                let retry_counts_by_reason = {
                    let counts = self.metrics.retry_counts_by_reason.read().await;
                    counts.clone()
                };
                
                RetryMetricsSnapshot {
                    total_retries: self.metrics.total_retries.load(std::sync::atomic::Ordering::Relaxed),
                    successful_retries: self.metrics.successful_retries.load(std::sync::atomic::Ordering::Relaxed),
                    exhausted_retries: self.metrics.exhausted_retries.load(std::sync::atomic::Ordering::Relaxed),
                    non_retryable_errors: self.metrics.non_retryable_errors.load(std::sync::atomic::Ordering::Relaxed),
                    total_retry_time: std::time::Duration::from_millis(
                        self.metrics.total_retry_time.load(std::sync::atomic::Ordering::Relaxed)
                    ),
                    retry_counts_by_reason,
                }
            }
        }
        
        /// Snapshot of retry metrics
        #[derive(Debug, Clone)]
        struct RetryMetricsSnapshot {
            total_retries: u64,
            successful_retries: u64,
            exhausted_retries: u64,
            non_retryable_errors: u64,
            total_retry_time: std::time::Duration,
            retry_counts_by_reason: std::collections::HashMap<String, u64>,
        }
        
        /// Utility functions
        
        /// Calculate Fibonacci number
        fn fibonacci(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => {
                    let mut a = 0;
                    let mut b = 1;
                    for _ in 2..=n {
                        let c = a + b;
                        a = b;
                        b = c;
                    }
                    b
                }
            }
        }
        
        /// Get retryable errors from environment or defaults
        fn get_retryable_errors() -> Vec<String> {
            std::env::var("RETRY_RETRYABLE_ERRORS")
                .unwrap_or_else(|_| "timeout,network,unavailable,503,502,500".to_string())
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect()
        }
        
        /// Get non-retryable errors from environment or defaults
        fn get_non_retryable_errors() -> Vec<String> {
            std::env::var("RETRY_NON_RETRYABLE_ERRORS")
                .unwrap_or_else(|_| "unauthorized,forbidden,not found,400,401,403,404".to_string())
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .collect()
        }
        
        /// Extract method name from message
        fn extract_method_name(msg: &rabbitmesh::Message) -> String {
            if let Some(headers) = &msg.headers {
                if let Some(method) = headers.get("method").and_then(|v| v.as_str()) {
                    return method.to_string();
                }
            }
            
            msg.headers
                .as_ref()
                .and_then(|h| h.get("routing_key"))
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string()
        }
        
        // Set up retry execution wrapper
        let retry_wrapper = RetryWrapper {
            manager: retry_manager,
            original_msg: msg.clone(),
        };
        
        /// Retry wrapper for automatic retry handling
        struct RetryWrapper {
            manager: RetryManager,
            original_msg: rabbitmesh::Message,
        }
        
        impl RetryWrapper {
            /// Execute handler with retry logic
            async fn execute_handler<F, T>(&self, handler: F) -> Result<T, rabbitmesh::error::RabbitMeshError>
            where
                F: Fn(rabbitmesh::Message) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, rabbitmesh::error::RabbitMeshError>> + Send>> + Send + 'static,
                T: Send + 'static,
            {
                let msg = self.original_msg.clone();
                let outcome = self.manager.execute_with_retry(move || {
                    let msg_clone = msg.clone();
                    let handler_clone = &handler;
                    Box::pin(async move {
                        handler_clone(msg_clone).await
                    })
                }).await;
                
                match outcome {
                    RetryOutcome::Success(result) => Ok(result),
                    RetryOutcome::ExhaustedRetries { attempts, last_error, total_elapsed } => {
                        Err(rabbitmesh::error::RabbitMeshError::Handler(
                            format!("Operation failed after {} attempts over {}ms: {}", 
                                attempts, total_elapsed.as_millis(), last_error)
                        ))
                    }
                    RetryOutcome::NonRetryableError { error, attempt } => {
                        Err(rabbitmesh::error::RabbitMeshError::Handler(
                            format!("Non-retryable error on attempt {}: {}", attempt, error)
                        ))
                    }
                }
            }
        }
        
        // Store retry wrapper for use by the actual handler
        let _retry_context = RetryExecutionContext {
            wrapper: retry_wrapper,
        };
        
        /// Retry execution context for handler integration
        struct RetryExecutionContext {
            wrapper: RetryWrapper,
        }
    }
}