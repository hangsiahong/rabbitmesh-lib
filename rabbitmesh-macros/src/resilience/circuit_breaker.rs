//! Circuit Breaker Module
//! 
//! Provides comprehensive circuit breaker implementation with multiple states,
//! configurable failure thresholds, and automatic recovery mechanisms.

use quote::quote;

/// Generate circuit breaker preprocessing code
pub fn generate_circuit_breaker_preprocessing(
    failure_threshold: Option<u32>,
    recovery_timeout: Option<u64>,
    half_open_max_calls: Option<u32>
) -> proc_macro2::TokenStream {
    quote! {
        tracing::debug!("🔌 Initializing circuit breaker protection");
        
        // Get or create circuit breaker for this service/method
        let service_name = std::env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string());
        let method_name = extract_method_name(&msg);
        let circuit_breaker_key = format!("{}:{}", service_name, method_name);
        
        let circuit_breaker = CircuitBreakerManager::get_or_create(&circuit_breaker_key).await?;
        
        let failure_threshold = #failure_threshold.unwrap_or(5);
        let recovery_timeout = #recovery_timeout.unwrap_or(60);
        let half_open_max = #half_open_max_calls.unwrap_or(3);
        
        tracing::debug!("⚡ Circuit breaker {} - Threshold: {}, Recovery: {}s, Half-open: {}", 
            circuit_breaker_key, failure_threshold, recovery_timeout, half_open_max);
        
        // Check circuit breaker state before execution
        let circuit_state = circuit_breaker.get_state().await;
        
        match circuit_state {
            CircuitBreakerState::Open => {
                let time_remaining = circuit_breaker.time_until_half_open().await;
                tracing::warn!("🚫 Circuit breaker {} is OPEN - blocking request ({}s remaining)", 
                    circuit_breaker_key, time_remaining.as_secs());
                
                return Err(rabbitmesh::error::RabbitMeshError::Handler(
                    format!("Circuit breaker is open. Service temporarily unavailable. Retry in {}s", 
                        time_remaining.as_secs())
                ));
            }
            CircuitBreakerState::HalfOpen => {
                if !circuit_breaker.can_attempt_call().await {
                    tracing::warn!("🔶 Circuit breaker {} is HALF-OPEN - max calls reached", circuit_breaker_key);
                    return Err(rabbitmesh::error::RabbitMeshError::Handler(
                        "Circuit breaker is half-open and at capacity. Try again later".to_string()
                    ));
                }
                tracing::debug!("🔶 Circuit breaker {} is HALF-OPEN - allowing test call", circuit_breaker_key);
            }
            CircuitBreakerState::Closed => {
                tracing::debug!("✅ Circuit breaker {} is CLOSED - allowing call", circuit_breaker_key);
            }
        }
        
        // Record call attempt
        circuit_breaker.record_call_start().await;
        
        /// Circuit Breaker Manager for managing multiple circuit breakers
        struct CircuitBreakerManager;
        
        /// Circuit breaker states
        #[derive(Debug, Clone, PartialEq)]
        enum CircuitBreakerState {
            Closed,    // Normal operation - calls allowed
            Open,      // Failure threshold exceeded - calls blocked
            HalfOpen,  // Recovery testing - limited calls allowed
        }
        
        /// Circuit breaker implementation with configurable policies
        struct CircuitBreaker {
            key: String,
            state: Arc<RwLock<CircuitBreakerInternalState>>,
            config: CircuitBreakerConfig,
            metrics: Arc<CircuitBreakerMetrics>,
            state_change_listeners: Vec<Arc<dyn CircuitBreakerStateListener + Send + Sync>>,
        }
        
        /// Internal circuit breaker state
        #[derive(Debug)]
        struct CircuitBreakerInternalState {
            current_state: CircuitBreakerState,
            failure_count: u32,
            success_count: u32,
            last_failure_time: Option<std::time::Instant>,
            last_success_time: Option<std::time::Instant>,
            state_changed_time: std::time::Instant,
            half_open_calls: u32,
            consecutive_successes: u32,
        }
        
        /// Circuit breaker configuration
        #[derive(Debug, Clone)]
        struct CircuitBreakerConfig {
            failure_threshold: u32,
            recovery_timeout: std::time::Duration,
            half_open_max_calls: u32,
            success_threshold: u32,
            timeout: std::time::Duration,
            slow_call_threshold: std::time::Duration,
            slow_call_rate_threshold: f64,
            minimum_number_of_calls: u32,
            sliding_window_size: u32,
            window_type: SlidingWindowType,
        }
        
        /// Sliding window types for failure tracking
        #[derive(Debug, Clone, PartialEq)]
        enum SlidingWindowType {
            CountBased,  // Track last N calls
            TimeBased,   // Track calls in last N seconds
        }
        
        /// Circuit breaker metrics
        struct CircuitBreakerMetrics {
            total_calls: AtomicU64,
            successful_calls: AtomicU64,
            failed_calls: AtomicU64,
            slow_calls: AtomicU64,
            state_transitions: AtomicU64,
            time_in_open_state: AtomicU64,
            time_in_half_open_state: AtomicU64,
            current_failure_rate: AtomicU32, // as percentage * 100
            current_slow_call_rate: AtomicU32, // as percentage * 100
        }
        
        /// State change listener trait
        trait CircuitBreakerStateListener {
            fn on_state_change(&self, key: &str, from: CircuitBreakerState, to: CircuitBreakerState);
            fn on_call_success(&self, key: &str, duration: std::time::Duration);
            fn on_call_failure(&self, key: &str, error: &str, duration: std::time::Duration);
        }
        
        /// Call outcome for circuit breaker decision making
        #[derive(Debug)]
        enum CallOutcome {
            Success,
            Failure(String),
            Timeout,
            SlowCall,
        }
        
        /// Thread-safe global circuit breaker registry
        static CIRCUIT_BREAKER_REGISTRY: once_cell::sync::OnceCell<Arc<RwLock<std::collections::HashMap<String, Arc<CircuitBreaker>>>>> 
            = once_cell::sync::OnceCell::new();
        
        impl CircuitBreakerManager {
            /// Get or create circuit breaker for a specific key
            async fn get_or_create(key: &str) -> Result<Arc<CircuitBreaker>, CircuitBreakerError> {
                let registry = CIRCUIT_BREAKER_REGISTRY.get_or_init(|| {
                    Arc::new(RwLock::new(std::collections::HashMap::new()))
                });
                
                // Try to get existing circuit breaker
                {
                    let registry_read = registry.read().await;
                    if let Some(cb) = registry_read.get(key) {
                        return Ok(cb.clone());
                    }
                }
                
                // Create new circuit breaker
                let config = CircuitBreakerConfig::from_env(key);
                let circuit_breaker = Arc::new(CircuitBreaker::new(key.to_string(), config)?);
                
                // Store in registry
                {
                    let mut registry_write = registry.write().await;
                    registry_write.insert(key.to_string(), circuit_breaker.clone());
                }
                
                tracing::info!("🔌 Created new circuit breaker: {}", key);
                Ok(circuit_breaker)
            }
            
            /// Get all circuit breaker states for monitoring
            async fn get_all_states() -> std::collections::HashMap<String, CircuitBreakerState> {
                let registry = CIRCUIT_BREAKER_REGISTRY.get_or_init(|| {
                    Arc::new(RwLock::new(std::collections::HashMap::new()))
                });
                
                let registry_read = registry.read().await;
                let mut states = std::collections::HashMap::new();
                
                for (key, cb) in registry_read.iter() {
                    let state = cb.get_state().await;
                    states.insert(key.clone(), state);
                }
                
                states
            }
        }
        
        impl CircuitBreaker {
            /// Create new circuit breaker
            fn new(key: String, config: CircuitBreakerConfig) -> Result<Self, CircuitBreakerError> {
                let state = Arc::new(RwLock::new(CircuitBreakerInternalState {
                    current_state: CircuitBreakerState::Closed,
                    failure_count: 0,
                    success_count: 0,
                    last_failure_time: None,
                    last_success_time: None,
                    state_changed_time: std::time::Instant::now(),
                    half_open_calls: 0,
                    consecutive_successes: 0,
                }));
                
                let metrics = Arc::new(CircuitBreakerMetrics {
                    total_calls: AtomicU64::new(0),
                    successful_calls: AtomicU64::new(0),
                    failed_calls: AtomicU64::new(0),
                    slow_calls: AtomicU64::new(0),
                    state_transitions: AtomicU64::new(0),
                    time_in_open_state: AtomicU64::new(0),
                    time_in_half_open_state: AtomicU64::new(0),
                    current_failure_rate: AtomicU32::new(0),
                    current_slow_call_rate: AtomicU32::new(0),
                });
                
                Ok(Self {
                    key,
                    state,
                    config,
                    metrics,
                    state_change_listeners: Vec::new(),
                })
            }
            
            /// Get current circuit breaker state
            async fn get_state(&self) -> CircuitBreakerState {
                let state = self.state.read().await;
                
                // Check if we should transition from Open to HalfOpen
                if state.current_state == CircuitBreakerState::Open {
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() >= self.config.recovery_timeout {
                            drop(state);
                            return self.transition_to_half_open().await;
                        }
                    }
                }
                
                state.current_state.clone()
            }
            
            /// Check if a call can be attempted (for half-open state)
            async fn can_attempt_call(&self) -> bool {
                let state = self.state.read().await;
                match state.current_state {
                    CircuitBreakerState::Closed => true,
                    CircuitBreakerState::HalfOpen => state.half_open_calls < self.config.half_open_max_calls,
                    CircuitBreakerState::Open => false,
                }
            }
            
            /// Record the start of a call
            async fn record_call_start(&self) {
                self.metrics.total_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                let mut state = self.state.write().await;
                if state.current_state == CircuitBreakerState::HalfOpen {
                    state.half_open_calls += 1;
                }
            }
            
            /// Record the outcome of a call
            async fn record_call_outcome(&self, outcome: CallOutcome, duration: std::time::Duration) {
                match outcome {
                    CallOutcome::Success => {
                        self.metrics.successful_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        self.handle_success(duration).await;
                    }
                    CallOutcome::Failure(error) => {
                        self.metrics.failed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        self.handle_failure(&error, duration).await;
                    }
                    CallOutcome::Timeout => {
                        self.metrics.failed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        self.handle_failure("Timeout", duration).await;
                    }
                    CallOutcome::SlowCall => {
                        self.metrics.slow_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        // Slow calls might be treated as failures depending on configuration
                        if duration > self.config.slow_call_threshold {
                            self.handle_slow_call(duration).await;
                        }
                    }
                }
                
                self.update_metrics().await;
            }
            
            /// Handle successful call
            async fn handle_success(&self, duration: std::time::Duration) {
                let mut state = self.state.write().await;
                state.last_success_time = Some(std::time::Instant::now());
                state.success_count += 1;
                
                match state.current_state {
                    CircuitBreakerState::HalfOpen => {
                        state.consecutive_successes += 1;
                        if state.consecutive_successes >= self.config.success_threshold {
                            // Transition back to closed
                            self.notify_state_change(&state.current_state, &CircuitBreakerState::Closed);
                            state.current_state = CircuitBreakerState::Closed;
                            state.failure_count = 0;
                            state.half_open_calls = 0;
                            state.consecutive_successes = 0;
                            state.state_changed_time = std::time::Instant::now();
                            self.metrics.state_transitions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            tracing::info!("🔌 Circuit breaker {} transitioned to CLOSED", self.key);
                        }
                    }
                    CircuitBreakerState::Closed => {
                        // Reset failure count on success
                        if state.failure_count > 0 {
                            state.failure_count = 0;
                        }
                    }
                    CircuitBreakerState::Open => {
                        // Should not reach here, but handle gracefully
                        tracing::warn!("🔌 Circuit breaker {} received success in OPEN state", self.key);
                    }
                }
                
                // Notify listeners
                for listener in &self.state_change_listeners {
                    listener.on_call_success(&self.key, duration);
                }
            }
            
            /// Handle failed call
            async fn handle_failure(&self, error: &str, duration: std::time::Duration) {
                let mut state = self.state.write().await;
                state.last_failure_time = Some(std::time::Instant::now());
                state.failure_count += 1;
                
                match state.current_state {
                    CircuitBreakerState::Closed => {
                        if state.failure_count >= self.config.failure_threshold {
                            // Transition to open
                            self.notify_state_change(&state.current_state, &CircuitBreakerState::Open);
                            state.current_state = CircuitBreakerState::Open;
                            state.state_changed_time = std::time::Instant::now();
                            self.metrics.state_transitions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            tracing::warn!("🔌 Circuit breaker {} transitioned to OPEN after {} failures", 
                                self.key, state.failure_count);
                        }
                    }
                    CircuitBreakerState::HalfOpen => {
                        // Immediate transition back to open on failure
                        self.notify_state_change(&state.current_state, &CircuitBreakerState::Open);
                        state.current_state = CircuitBreakerState::Open;
                        state.half_open_calls = 0;
                        state.consecutive_successes = 0;
                        state.state_changed_time = std::time::Instant::now();
                        self.metrics.state_transitions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        tracing::warn!("🔌 Circuit breaker {} transitioned from HALF-OPEN back to OPEN", self.key);
                    }
                    CircuitBreakerState::Open => {
                        // Should not reach here, but handle gracefully
                        tracing::warn!("🔌 Circuit breaker {} received failure in OPEN state", self.key);
                    }
                }
                
                // Notify listeners
                for listener in &self.state_change_listeners {
                    listener.on_call_failure(&self.key, error, duration);
                }
            }
            
            /// Handle slow call
            async fn handle_slow_call(&self, duration: std::time::Duration) {
                // Slow calls can be treated as failures depending on configuration
                let slow_call_rate = self.calculate_slow_call_rate().await;
                if slow_call_rate > self.config.slow_call_rate_threshold {
                    self.handle_failure("Slow call rate exceeded", duration).await;
                }
            }
            
            /// Transition from Open to HalfOpen
            async fn transition_to_half_open(&self) -> CircuitBreakerState {
                let mut state = self.state.write().await;
                if state.current_state == CircuitBreakerState::Open {
                    let old_state = state.current_state.clone();
                    state.current_state = CircuitBreakerState::HalfOpen;
                    state.half_open_calls = 0;
                    state.consecutive_successes = 0;
                    state.state_changed_time = std::time::Instant::now();
                    self.metrics.state_transitions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    
                    self.notify_state_change(&old_state, &state.current_state);
                    tracing::info!("🔌 Circuit breaker {} transitioned to HALF-OPEN", self.key);
                }
                state.current_state.clone()
            }
            
            /// Calculate time until circuit breaker can transition to half-open
            async fn time_until_half_open(&self) -> std::time::Duration {
                let state = self.state.read().await;
                if let Some(last_failure) = state.last_failure_time {
                    let elapsed = last_failure.elapsed();
                    if elapsed < self.config.recovery_timeout {
                        self.config.recovery_timeout - elapsed
                    } else {
                        std::time::Duration::from_secs(0)
                    }
                } else {
                    std::time::Duration::from_secs(0)
                }
            }
            
            /// Calculate current failure rate
            async fn calculate_failure_rate(&self) -> f64 {
                let total_calls = self.metrics.total_calls.load(std::sync::atomic::Ordering::Relaxed);
                let failed_calls = self.metrics.failed_calls.load(std::sync::atomic::Ordering::Relaxed);
                
                if total_calls == 0 {
                    0.0
                } else {
                    (failed_calls as f64 / total_calls as f64) * 100.0
                }
            }
            
            /// Calculate current slow call rate
            async fn calculate_slow_call_rate(&self) -> f64 {
                let total_calls = self.metrics.total_calls.load(std::sync::atomic::Ordering::Relaxed);
                let slow_calls = self.metrics.slow_calls.load(std::sync::atomic::Ordering::Relaxed);
                
                if total_calls == 0 {
                    0.0
                } else {
                    (slow_calls as f64 / total_calls as f64) * 100.0
                }
            }
            
            /// Update metrics
            async fn update_metrics(&self) {
                let failure_rate = self.calculate_failure_rate().await;
                let slow_call_rate = self.calculate_slow_call_rate().await;
                
                self.metrics.current_failure_rate.store(
                    (failure_rate * 100.0) as u32, 
                    std::sync::atomic::Ordering::Relaxed
                );
                
                self.metrics.current_slow_call_rate.store(
                    (slow_call_rate * 100.0) as u32, 
                    std::sync::atomic::Ordering::Relaxed
                );
            }
            
            /// Notify state change listeners
            fn notify_state_change(&self, from: &CircuitBreakerState, to: &CircuitBreakerState) {
                for listener in &self.state_change_listeners {
                    listener.on_state_change(&self.key, from.clone(), to.clone());
                }
            }
            
            /// Get circuit breaker statistics
            async fn get_metrics(&self) -> CircuitBreakerMetrics {
                CircuitBreakerMetrics {
                    total_calls: AtomicU64::new(self.metrics.total_calls.load(std::sync::atomic::Ordering::Relaxed)),
                    successful_calls: AtomicU64::new(self.metrics.successful_calls.load(std::sync::atomic::Ordering::Relaxed)),
                    failed_calls: AtomicU64::new(self.metrics.failed_calls.load(std::sync::atomic::Ordering::Relaxed)),
                    slow_calls: AtomicU64::new(self.metrics.slow_calls.load(std::sync::atomic::Ordering::Relaxed)),
                    state_transitions: AtomicU64::new(self.metrics.state_transitions.load(std::sync::atomic::Ordering::Relaxed)),
                    time_in_open_state: AtomicU64::new(self.metrics.time_in_open_state.load(std::sync::atomic::Ordering::Relaxed)),
                    time_in_half_open_state: AtomicU64::new(self.metrics.time_in_half_open_state.load(std::sync::atomic::Ordering::Relaxed)),
                    current_failure_rate: AtomicU32::new(self.metrics.current_failure_rate.load(std::sync::atomic::Ordering::Relaxed)),
                    current_slow_call_rate: AtomicU32::new(self.metrics.current_slow_call_rate.load(std::sync::atomic::Ordering::Relaxed)),
                }
            }
        }
        
        impl CircuitBreakerConfig {
            /// Create configuration from environment variables
            fn from_env(key: &str) -> Self {
                let prefix = format!("CB_{}_", key.replace([':', '-', '.'], "_").to_uppercase());
                
                Self {
                    failure_threshold: Self::env_or_default(&format!("{}FAILURE_THRESHOLD", prefix), #failure_threshold.unwrap_or(5)),
                    recovery_timeout: std::time::Duration::from_secs(
                        Self::env_or_default(&format!("{}RECOVERY_TIMEOUT", prefix), #recovery_timeout.unwrap_or(60))
                    ),
                    half_open_max_calls: Self::env_or_default(&format!("{}HALF_OPEN_MAX_CALLS", prefix), #half_open_max_calls.unwrap_or(3)),
                    success_threshold: Self::env_or_default(&format!("{}SUCCESS_THRESHOLD", prefix), 3),
                    timeout: std::time::Duration::from_secs(Self::env_or_default(&format!("{}TIMEOUT", prefix), 30)),
                    slow_call_threshold: std::time::Duration::from_millis(
                        Self::env_or_default(&format!("{}SLOW_CALL_THRESHOLD_MS", prefix), 5000)
                    ),
                    slow_call_rate_threshold: Self::env_or_default(&format!("{}SLOW_CALL_RATE_THRESHOLD", prefix), 80.0),
                    minimum_number_of_calls: Self::env_or_default(&format!("{}MIN_CALLS", prefix), 10),
                    sliding_window_size: Self::env_or_default(&format!("{}SLIDING_WINDOW_SIZE", prefix), 100),
                    window_type: if Self::env_or_default(&format!("{}WINDOW_TYPE", prefix), "count".to_string()) == "time" {
                        SlidingWindowType::TimeBased
                    } else {
                        SlidingWindowType::CountBased
                    },
                }
            }
            
            /// Get environment variable or default value
            fn env_or_default<T: std::str::FromStr>(env_var: &str, default: T) -> T {
                std::env::var(env_var)
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(default)
            }
        }
        
        /// Circuit breaker error types
        #[derive(Debug, thiserror::Error)]
        enum CircuitBreakerError {
            #[error("Configuration error: {0}")]
            Configuration(String),
            #[error("State error: {0}")]
            State(String),
            #[error("Metrics error: {0}")]
            Metrics(String),
        }
        
        /// Utility functions
        
        /// Extract method name from message for circuit breaker key
        fn extract_method_name(msg: &rabbitmesh::Message) -> String {
            // Try to extract method name from headers or payload
            if let Some(headers) = &msg.headers {
                if let Some(method) = headers.get("method").and_then(|v| v.as_str()) {
                    return method.to_string();
                }
            }
            
            // Fallback: extract from routing key or use default
            msg.headers
                .as_ref()
                .and_then(|h| h.get("routing_key"))
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string()
        }
        
        // Set up automatic call outcome recording based on handler result
        let circuit_breaker_guard = CircuitBreakerGuard {
            circuit_breaker: circuit_breaker.clone(),
            start_time: std::time::Instant::now(),
        };
        
        /// RAII guard for automatic circuit breaker outcome recording
        struct CircuitBreakerGuard {
            circuit_breaker: Arc<CircuitBreaker>,
            start_time: std::time::Instant,
        }
        
        impl Drop for CircuitBreakerGuard {
            fn drop(&mut self) {
                let duration = self.start_time.elapsed();
                let circuit_breaker = self.circuit_breaker.clone();
                
                tokio::spawn(async move {
                    // In a real implementation, we would determine the outcome based on the handler result
                    // For now, we assume success if we reach this point without panicking
                    circuit_breaker.record_call_outcome(CallOutcome::Success, duration).await;
                });
            }
        }
    }
}