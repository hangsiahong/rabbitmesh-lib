//! Circuit Breaker Module
//! 
//! Provides comprehensive circuit breaker implementation with multiple states,
//! configurable failure thresholds, and automatic recovery mechanisms.

use quote::quote;
use std::sync::{Arc, RwLock, atomic::{AtomicU64, AtomicU32, Ordering}};

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
        #[derive(Debug, Clone)]
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
            async fn record_call_outcome(&self, outcome: CallOutcome, duration: std::time::Duration) -> Result<(), CircuitBreakerError> {
                tracing::debug!("🔌 Recording call outcome: {:?} in {:?}", outcome, duration);
                
                match outcome {
                    CallOutcome::Success => {
                        self.metrics.successful_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if let Err(e) = self.handle_success(duration).await {
                            tracing::error!("🔌 Failed to handle success: {}", e);
                            return Err(e);
                        }
                        tracing::debug!("🔌 Successfully recorded success outcome");
                    }
                    CallOutcome::Failure(ref error) => {
                        self.metrics.failed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if let Err(e) = self.handle_failure(error, duration).await {
                            tracing::error!("🔌 Failed to handle failure: {}", e);
                            return Err(e);
                        }
                        tracing::debug!("🔌 Successfully recorded failure outcome: {}", error);
                    }
                    CallOutcome::Timeout => {
                        self.metrics.failed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if let Err(e) = self.handle_failure("Timeout", duration).await {
                            tracing::error!("🔌 Failed to handle timeout: {}", e);
                            return Err(e);
                        }
                        tracing::debug!("🔌 Successfully recorded timeout outcome");
                    }
                    CallOutcome::SlowCall => {
                        self.metrics.slow_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        // Slow calls might be treated as failures depending on configuration
                        if duration > self.config.slow_call_threshold {
                            if let Err(e) = self.handle_slow_call(duration).await {
                                tracing::error!("🔌 Failed to handle slow call: {}", e);
                                return Err(e);
                            }
                        }
                        tracing::debug!("🔌 Successfully recorded slow call outcome");
                    }
                }
                
                if let Err(e) = self.update_metrics().await {
                    tracing::error!("🔌 Failed to update metrics: {}", e);
                    return Err(e);
                }
                
                Ok(())
            }
            
            /// Handle successful call
            async fn handle_success(&self, duration: std::time::Duration) -> Result<(), CircuitBreakerError> {
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
                        return Err(CircuitBreakerError::State(
                            format!("Unexpected success in OPEN state for circuit breaker: {}", self.key)
                        ));
                    }
                }
                
                // Notify listeners
                for listener in &self.state_change_listeners {
                    listener.on_call_success(&self.key, duration);
                }
                
                Ok(())
            }
            
            /// Handle failed call
            async fn handle_failure(&self, error: &str, duration: std::time::Duration) -> Result<(), CircuitBreakerError> {
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
                
                Ok(())
            }
            
            /// Handle slow call
            async fn handle_slow_call(&self, duration: std::time::Duration) -> Result<(), CircuitBreakerError> {
                // Slow calls can be treated as failures depending on configuration
                let slow_call_rate = self.calculate_slow_call_rate().await;
                if slow_call_rate > self.config.slow_call_rate_threshold {
                    tracing::warn!("🔌 Slow call rate threshold exceeded: {:.2}% > {:.2}%", 
                        slow_call_rate, self.config.slow_call_rate_threshold);
                    self.handle_failure("Slow call rate exceeded", duration).await?;
                } else {
                    tracing::debug!("🔌 Slow call registered but within threshold: {:.2}% <= {:.2}%", 
                        slow_call_rate, self.config.slow_call_rate_threshold);
                }
                Ok(())
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
            async fn update_metrics(&self) -> Result<(), CircuitBreakerError> {
                let failure_rate = self.calculate_failure_rate().await;
                let slow_call_rate = self.calculate_slow_call_rate().await;
                
                // Validate rates are within expected bounds
                if failure_rate < 0.0 || failure_rate > 100.0 {
                    return Err(CircuitBreakerError::Metrics(
                        format!("Invalid failure rate: {:.2}%", failure_rate)
                    ));
                }
                
                if slow_call_rate < 0.0 || slow_call_rate > 100.0 {
                    return Err(CircuitBreakerError::Metrics(
                        format!("Invalid slow call rate: {:.2}%", slow_call_rate)
                    ));
                }
                
                self.metrics.current_failure_rate.store(
                    (failure_rate * 100.0) as u32, 
                    std::sync::atomic::Ordering::Relaxed
                );
                
                self.metrics.current_slow_call_rate.store(
                    (slow_call_rate * 100.0) as u32, 
                    std::sync::atomic::Ordering::Relaxed
                );
                
                tracing::trace!("🔌 Updated metrics - Failure rate: {:.2}%, Slow call rate: {:.2}%", 
                    failure_rate, slow_call_rate);
                
                Ok(())
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
        
        // Set up circuit breaker outcome tracking with thread-local result capture
        let _circuit_breaker_guard = CircuitBreakerGuard::new(circuit_breaker.clone());
        
        /// Thread-local storage for capturing handler results
        thread_local! {
            static HANDLER_RESULT: std::cell::RefCell<Option<HandlerResultOutcome>> = std::cell::RefCell::new(None);
        }
        
        /// Handler result outcome for circuit breaker analysis
        #[derive(Debug, Clone)]
        enum HandlerResultOutcome {
            Success(serde_json::Value),
            Error(String),
            Timeout,
            Panic(String),
        }
        
        /// RAII guard for automatic circuit breaker outcome recording with comprehensive result analysis
        struct CircuitBreakerGuard {
            circuit_breaker: Arc<CircuitBreaker>,
            start_time: std::time::Instant,
            timeout_threshold: std::time::Duration,
            slow_call_threshold: std::time::Duration,
        }
        
        impl CircuitBreakerGuard {
            fn new(circuit_breaker: Arc<CircuitBreaker>) -> Self {
                let config = &circuit_breaker.config;
                Self {
                    circuit_breaker,
                    start_time: std::time::Instant::now(),
                    timeout_threshold: config.timeout,
                    slow_call_threshold: config.slow_call_threshold,
                }
            }
            
            /// Record successful handler execution
            fn record_success(&self, result: serde_json::Value) {
                HANDLER_RESULT.with(|r| {
                    *r.borrow_mut() = Some(HandlerResultOutcome::Success(result));
                });
            }
            
            /// Record handler error
            fn record_error(&self, error: String) {
                HANDLER_RESULT.with(|r| {
                    *r.borrow_mut() = Some(HandlerResultOutcome::Error(error));
                });
            }
            
            /// Record handler timeout
            fn record_timeout(&self) {
                HANDLER_RESULT.with(|r| {
                    *r.borrow_mut() = Some(HandlerResultOutcome::Timeout);
                });
            }
            
            /// Record handler panic
            fn record_panic(&self, panic_info: String) {
                HANDLER_RESULT.with(|r| {
                    *r.borrow_mut() = Some(HandlerResultOutcome::Panic(panic_info));
                });
            }
        }
        
        impl Drop for CircuitBreakerGuard {
            fn drop(&mut self) {
                let duration = self.start_time.elapsed();
                let circuit_breaker = self.circuit_breaker.clone();
                let timeout_threshold = self.timeout_threshold;
                let slow_call_threshold = self.slow_call_threshold;
                
                // Capture the current thread's result before spawning async task
                let handler_outcome = HANDLER_RESULT.with(|r| r.borrow_mut().take());
                
                tokio::spawn(async move {
                    let outcome = match handler_outcome {
                        Some(HandlerResultOutcome::Success(result)) => {
                            // Analyze success result for potential issues
                            let outcome = analyze_success_result(result, duration, slow_call_threshold).await;
                            tracing::debug!("🔌 Circuit breaker outcome: Success - Duration: {:?}", duration);
                            outcome
                        }
                        Some(HandlerResultOutcome::Error(error)) => {
                            // Categorize error types
                            let outcome = categorize_error(&error, duration).await;
                            tracing::warn!("🔌 Circuit breaker outcome: Error - {} - Duration: {:?}", error, duration);
                            outcome
                        }
                        Some(HandlerResultOutcome::Timeout) => {
                            tracing::warn!("🔌 Circuit breaker outcome: Timeout - Duration: {:?}", duration);
                            CallOutcome::Timeout
                        }
                        Some(HandlerResultOutcome::Panic(panic_info)) => {
                            tracing::error!("🔌 Circuit breaker outcome: Panic - {} - Duration: {:?}", panic_info, duration);
                            CallOutcome::Failure(format!("Handler panic: {}", panic_info))
                        }
                        None => {
                            // No explicit result captured - analyze based on execution characteristics
                            analyze_execution_characteristics(duration, timeout_threshold, slow_call_threshold).await
                        }
                    };
                    
                    // Record the determined outcome
                    if let Err(e) = circuit_breaker.record_call_outcome(outcome, duration).await {
                        tracing::error!("🔌 Failed to record circuit breaker outcome: {}", e);
                    }
                });
            }
        }
        
        /// Analyze successful result for potential circuit breaker concerns
        async fn analyze_success_result(
            result: serde_json::Value, 
            duration: std::time::Duration,
            slow_call_threshold: std::time::Duration
        ) -> CallOutcome {
            // Check if call was slow
            if duration > slow_call_threshold {
                tracing::warn!("🐌 Slow call detected: {:?} > {:?}", duration, slow_call_threshold);
                return CallOutcome::SlowCall;
            }
            
            // Analyze result content for potential issues
            if let Some(obj) = result.as_object() {
                // Check for error indicators in successful responses
                if obj.contains_key("error") || obj.contains_key("errors") {
                    if let Some(error_msg) = obj.get("error").and_then(|e| e.as_str()) {
                        tracing::warn!("🔌 Success response contains error indicator: {}", error_msg);
                        return CallOutcome::Failure(format!("Response error: {}", error_msg));
                    }
                }
                
                // Check for warning indicators that might suggest degraded service
                if obj.contains_key("warnings") || obj.contains_key("partial_failure") {
                    tracing::warn!("🔌 Success response contains warnings or partial failures");
                    // Treat warnings as slow calls rather than failures
                    return CallOutcome::SlowCall;
                }
                
                // Check for status codes in response
                if let Some(status) = obj.get("status").and_then(|s| s.as_u64()) {
                    match status {
                        200..=299 => {} // Success range
                        400..=499 => {
                            tracing::warn!("🔌 Client error status in response: {}", status);
                            return CallOutcome::Failure(format!("Client error status: {}", status));
                        }
                        500..=599 => {
                            tracing::warn!("🔌 Server error status in response: {}", status);
                            return CallOutcome::Failure(format!("Server error status: {}", status));
                        }
                        _ => {
                            tracing::warn!("🔌 Unexpected status code in response: {}", status);
                        }
                    }
                }
            }
            
            CallOutcome::Success
        }
        
        /// Categorize error types for circuit breaker decision making
        async fn categorize_error(error: &str, duration: std::time::Duration) -> CallOutcome {
            let error_lower = error.to_lowercase();
            
            // Timeout-related errors
            if error_lower.contains("timeout") || error_lower.contains("timed out") || 
               error_lower.contains("deadline exceeded") || error_lower.contains("request timeout") {
                tracing::warn!("🔌 Timeout error detected: {}", error);
                return CallOutcome::Timeout;
            }
            
            // Network/connection errors that should be treated as failures
            if error_lower.contains("connection") || error_lower.contains("network") ||
               error_lower.contains("dns") || error_lower.contains("unreachable") ||
               error_lower.contains("refused") || error_lower.contains("reset") {
                tracing::warn!("🔌 Network/Connection error: {}", error);
                return CallOutcome::Failure(error.to_string());
            }
            
            // Resource exhaustion errors
            if error_lower.contains("out of memory") || error_lower.contains("resource exhausted") ||
               error_lower.contains("too many") || error_lower.contains("quota exceeded") ||
               error_lower.contains("rate limit") || error_lower.contains("throttled") {
                tracing::warn!("🔌 Resource exhaustion error: {}", error);
                return CallOutcome::Failure(error.to_string());
            }
            
            // Service unavailable errors
            if error_lower.contains("service unavailable") || error_lower.contains("server error") ||
               error_lower.contains("internal server error") || error_lower.contains("bad gateway") ||
               error_lower.contains("gateway timeout") || error_lower.contains("service down") {
                tracing::warn!("🔌 Service unavailable error: {}", error);
                return CallOutcome::Failure(error.to_string());
            }
            
            // Authentication/authorization errors (usually don't indicate service health issues)
            if error_lower.contains("unauthorized") || error_lower.contains("forbidden") ||
               error_lower.contains("authentication") || error_lower.contains("permission denied") ||
               error_lower.contains("access denied") || error_lower.contains("invalid token") {
                tracing::debug!("🔌 Auth error (not counting as failure): {}", error);
                return CallOutcome::Success; // Don't count auth errors as service failures
            }
            
            // Validation/client errors (usually don't indicate service health issues)
            if error_lower.contains("validation") || error_lower.contains("invalid input") ||
               error_lower.contains("bad request") || error_lower.contains("malformed") ||
               error_lower.contains("parse error") || error_lower.contains("schema") {
                tracing::debug!("🔌 Client/Validation error (not counting as failure): {}", error);
                return CallOutcome::Success; // Don't count client errors as service failures
            }
            
            // Check for slow execution even with errors
            let slow_call_threshold = std::time::Duration::from_millis(5000); // Default threshold
            if duration > slow_call_threshold {
                tracing::warn!("🔌 Slow error response: {} - Duration: {:?}", error, duration);
                return CallOutcome::SlowCall;
            }
            
            // Default: treat as service failure
            tracing::warn!("🔌 General service error: {}", error);
            CallOutcome::Failure(error.to_string())
        }
        
        /// Analyze execution characteristics when no explicit result is available
        async fn analyze_execution_characteristics(
            duration: std::time::Duration,
            timeout_threshold: std::time::Duration,
            slow_call_threshold: std::time::Duration
        ) -> CallOutcome {
            // Check for timeout
            if duration >= timeout_threshold {
                tracing::warn!("🔌 Execution timeout detected: {:?} >= {:?}", duration, timeout_threshold);
                return CallOutcome::Timeout;
            }
            
            // Check for slow call
            if duration > slow_call_threshold {
                tracing::warn!("🔌 Slow execution detected: {:?} > {:?}", duration, slow_call_threshold);
                return CallOutcome::SlowCall;
            }
            
            // If we reach here without explicit error, assume success
            tracing::debug!("🔌 No explicit result captured, assuming success - Duration: {:?}", duration);
            CallOutcome::Success
        }
    }
}

/// Generate circuit breaker postprocessing code for integration with service methods
pub fn generate_circuit_breaker_postprocessing() -> proc_macro2::TokenStream {
    quote! {
        // Circuit breaker postprocessing is handled automatically by the CircuitBreakerGuard's Drop implementation
        // The guard captures the handler result and determines the appropriate outcome
        
        // Additional postprocessing for result capture can be added here
        tracing::debug!("🔌 Circuit breaker postprocessing: Guard active, outcome will be determined on drop");
        
        // The HANDLER_RESULT thread-local storage will be automatically read by the guard's Drop implementation
        // No additional cleanup needed here as the RAII pattern handles everything
        
        /// Helper function to record circuit breaker outcomes manually if needed
        fn record_circuit_breaker_success(result: serde_json::Value) {
            HANDLER_RESULT.with(|r| {
                *r.borrow_mut() = Some(HandlerResultOutcome::Success(result));
            });
            tracing::debug!("🔌 Manually recorded circuit breaker success outcome");
        }
        
        /// Helper function to record circuit breaker errors manually if needed  
        fn record_circuit_breaker_error(error: String) {
            HANDLER_RESULT.with(|r| {
                *r.borrow_mut() = Some(HandlerResultOutcome::Error(error));
            });
            tracing::debug!("🔌 Manually recorded circuit breaker error outcome");
        }
        
        /// Helper function to record circuit breaker timeouts manually if needed
        fn record_circuit_breaker_timeout() {
            HANDLER_RESULT.with(|r| {
                *r.borrow_mut() = Some(HandlerResultOutcome::Timeout);
            });
            tracing::debug!("🔌 Manually recorded circuit breaker timeout outcome");
        }
    }
}