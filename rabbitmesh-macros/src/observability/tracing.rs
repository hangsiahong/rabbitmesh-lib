//! Distributed Tracing Module
//! 
//! Provides comprehensive distributed tracing support for microservices with multiple
//! backend implementations, span management, context propagation, and performance analysis.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::{SystemTime, Duration};

/// Main tracing macro for instrumenting functions with distributed tracing
pub fn traced(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let config = TracingConfig::default();
    generate_tracing_code(&input_fn, &config)
}

/// Configuration for tracing behavior
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub span_name: Option<String>,
    pub service_name: Option<String>, 
    pub operation_name: Option<String>,
    pub level: TraceLevel,
    pub sample_rate: f64,
    pub include_args: bool,
    pub include_result: bool,
    pub timeout: Option<Duration>,
    pub backend: TracingBackend,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            span_name: None,
            service_name: None,
            operation_name: None,
            level: TraceLevel::Info,
            sample_rate: 1.0,
            include_args: true,
            include_result: true,
            timeout: Some(Duration::from_secs(30)),
            backend: TracingBackend::OpenTelemetry,
        }
    }
}

/// Trace level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TraceLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Supported tracing backends
#[derive(Debug, Clone, PartialEq)]
pub enum TracingBackend {
    OpenTelemetry,
    Jaeger,
    Zipkin,
    DataDog,
    NewRelic,
    Console,
    File(String),
    Custom(String),
}

/// Span context for distributed tracing
#[derive(Debug, Clone)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_flags: u8,
    pub trace_state: BTreeMap<String, String>,
    pub baggage: BTreeMap<String, String>,
}

impl SpanContext {
    pub fn new() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let hash = hasher.finish();
        
        Self {
            trace_id: format!("{:032x}", hash as u128),
            span_id: format!("{:016x}", hash),
            parent_span_id: None,
            trace_flags: 1, // Sampled
            trace_state: BTreeMap::new(),
            baggage: BTreeMap::new(),
        }
    }
    
    pub fn child(&self) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.span_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        Self {
            trace_id: self.trace_id.clone(),
            span_id: format!("{:016x}", hash),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
            baggage: self.baggage.clone(),
        }
    }
    
    pub fn set_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }
    
    pub fn get_baggage(&self, key: &str) -> Option<&String> {
        self.baggage.get(key)
    }
}

/// Individual span in a trace
#[derive(Debug, Clone)]
pub struct Span {
    pub context: SpanContext,
    pub operation_name: String,
    pub service_name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub status: SpanStatus,
    pub tags: BTreeMap<String, SpanValue>,
    pub logs: Vec<SpanLog>,
    pub references: Vec<SpanReference>,
}

impl Span {
    pub fn new(operation_name: String, service_name: String) -> Self {
        Self {
            context: SpanContext::new(),
            operation_name,
            service_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Ok,
            tags: BTreeMap::new(),
            logs: Vec::new(),
            references: Vec::new(),
        }
    }
    
    pub fn with_parent(operation_name: String, service_name: String, parent: &SpanContext) -> Self {
        Self {
            context: parent.child(),
            operation_name,
            service_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Ok,
            tags: BTreeMap::new(),
            logs: Vec::new(),
            references: vec![SpanReference {
                ref_type: SpanRefType::ChildOf,
                trace_id: parent.trace_id.clone(),
                span_id: parent.span_id.clone(),
            }],
        }
    }
    
    pub fn set_tag(&mut self, key: String, value: SpanValue) {
        self.tags.insert(key, value);
    }
    
    pub fn log(&mut self, message: String, fields: BTreeMap<String, SpanValue>) {
        self.logs.push(SpanLog {
            timestamp: SystemTime::now(),
            message,
            fields,
        });
    }
    
    pub fn finish(&mut self) {
        self.end_time = Some(SystemTime::now());
        if let Ok(duration) = self.end_time.unwrap().duration_since(self.start_time) {
            self.duration = Some(duration);
        }
    }
    
    pub fn set_error(&mut self, error: String) {
        self.status = SpanStatus::Error;
        self.set_tag("error".to_string(), SpanValue::Bool(true));
        self.set_tag("error.message".to_string(), SpanValue::String(error));
    }
}

/// Span status enumeration
#[derive(Debug, Clone)]
pub enum SpanStatus {
    Ok,
    Cancelled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
    Error,
}

/// Span value types for tags and log fields
#[derive(Debug, Clone)]
pub enum SpanValue {
    String(String),
    Bool(bool),
    Int64(i64),
    Float64(f64),
    Binary(Vec<u8>),
}

/// Span log entry
#[derive(Debug, Clone)]
pub struct SpanLog {
    pub timestamp: SystemTime,
    pub message: String,
    pub fields: BTreeMap<String, SpanValue>,
}

/// Span reference types
#[derive(Debug, Clone)]
pub enum SpanRefType {
    ChildOf,
    FollowsFrom,
}

/// Span reference
#[derive(Debug, Clone)]
pub struct SpanReference {
    pub ref_type: SpanRefType,
    pub trace_id: String,
    pub span_id: String,
}

/// Tracer interface for different backends
pub trait Tracer: Send + Sync {
    fn start_span(&self, operation_name: &str, parent: Option<&SpanContext>) -> Box<dyn ActiveSpan>;
    fn extract_context(&self, carrier: &dyn TextMapReader) -> Option<SpanContext>;
    fn inject_context(&self, context: &SpanContext, carrier: &mut dyn TextMapWriter);
    fn flush(&self) -> Result<(), TracingError>;
    fn close(&self) -> Result<(), TracingError>;
}

/// Active span interface
pub trait ActiveSpan: Send + Sync {
    fn context(&self) -> &SpanContext;
    fn set_tag(&mut self, key: &str, value: SpanValue);
    fn log(&mut self, message: &str, fields: BTreeMap<String, SpanValue>);
    fn set_operation_name(&mut self, name: &str);
    fn finish(&mut self);
    fn set_error(&mut self, error: &str);
}

/// Text map reader for context propagation
pub trait TextMapReader {
    fn get(&self, key: &str) -> Option<&str>;
    fn keys(&self) -> Vec<&str>;
}

/// Text map writer for context propagation
pub trait TextMapWriter {
    fn set(&mut self, key: &str, value: &str);
}

/// HTTP headers implementation of TextMapReader/Writer
pub struct HttpHeaders {
    headers: HashMap<String, String>,
}

impl HttpHeaders {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }
    
    pub fn from_map(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }
}

impl TextMapReader for HttpHeaders {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }
    
    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|s| s.as_str()).collect()
    }
}

impl TextMapWriter for HttpHeaders {
    fn set(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }
}

/// OpenTelemetry tracer implementation
pub struct OpenTelemetryTracer {
    service_name: String,
    endpoint: String,
    headers: HashMap<String, String>,
    spans: Arc<RwLock<Vec<Span>>>,
    sample_rate: f64,
}

impl OpenTelemetryTracer {
    pub fn new(service_name: String, endpoint: String) -> Self {
        let spans = Arc::new(RwLock::new(Vec::new()));
        
        Self {
            service_name,
            endpoint,
            headers: HashMap::new(),
            spans,
            sample_rate: 1.0,
        }
    }
    
    fn send_batch(spans: &[Span], endpoint: &str) {
        // Implementation would send spans to OpenTelemetry collector
        println!("📊 Sending {} spans to OpenTelemetry at {}", spans.len(), endpoint);
    }
}

impl Tracer for OpenTelemetryTracer {
    fn start_span(&self, operation_name: &str, parent: Option<&SpanContext>) -> Box<dyn ActiveSpan> {
        let span = match parent {
            Some(parent_ctx) => Span::with_parent(
                operation_name.to_string(),
                self.service_name.clone(),
                parent_ctx,
            ),
            None => Span::new(operation_name.to_string(), self.service_name.clone()),
        };
        
        Box::new(OpenTelemetrySpan {
            span,
        })
    }
    
    fn extract_context(&self, carrier: &dyn TextMapReader) -> Option<SpanContext> {
        // Extract W3C Trace Context headers
        let trace_parent = carrier.get("traceparent")?;
        let parts: Vec<&str> = trace_parent.split('-').collect();
        
        if parts.len() >= 4 {
            let trace_id = parts[1].to_string();
            let span_id = parts[2].to_string();
            let trace_flags = u8::from_str_radix(parts[3], 16).unwrap_or(0);
            
            let mut context = SpanContext {
                trace_id,
                span_id,
                parent_span_id: None,
                trace_flags,
                trace_state: BTreeMap::new(),
                baggage: BTreeMap::new(),
            };
            
            // Parse trace state
            if let Some(trace_state) = carrier.get("tracestate") {
                for pair in trace_state.split(',') {
                    if let Some((key, value)) = pair.split_once('=') {
                        context.trace_state.insert(key.to_string(), value.to_string());
                    }
                }
            }
            
            // Parse baggage
            if let Some(baggage) = carrier.get("baggage") {
                for pair in baggage.split(',') {
                    if let Some((key, value)) = pair.split_once('=') {
                        context.baggage.insert(key.to_string(), value.to_string());
                    }
                }
            }
            
            Some(context)
        } else {
            None
        }
    }
    
    fn inject_context(&self, context: &SpanContext, carrier: &mut dyn TextMapWriter) {
        // Inject W3C Trace Context headers
        let traceparent = format!(
            "00-{}-{}-{:02x}",
            context.trace_id,
            context.span_id,
            context.trace_flags
        );
        carrier.set("traceparent", &traceparent);
        
        // Inject trace state
        if !context.trace_state.is_empty() {
            let trace_state: String = context
                .trace_state
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            carrier.set("tracestate", &trace_state);
        }
        
        // Inject baggage
        if !context.baggage.is_empty() {
            let baggage: String = context
                .baggage
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            carrier.set("baggage", &baggage);
        }
    }
    
    fn flush(&self) -> Result<(), TracingError> {
        Ok(())
    }
    
    fn close(&self) -> Result<(), TracingError> {
        Ok(())
    }
}

/// OpenTelemetry active span implementation
pub struct OpenTelemetrySpan {
    span: Span,
}

impl ActiveSpan for OpenTelemetrySpan {
    fn context(&self) -> &SpanContext {
        &self.span.context
    }
    
    fn set_tag(&mut self, key: &str, value: SpanValue) {
        self.span.set_tag(key.to_string(), value);
    }
    
    fn log(&mut self, message: &str, fields: BTreeMap<String, SpanValue>) {
        self.span.log(message.to_string(), fields);
    }
    
    fn set_operation_name(&mut self, name: &str) {
        self.span.operation_name = name.to_string();
    }
    
    fn finish(&mut self) {
        self.span.finish();
    }
    
    fn set_error(&mut self, error: &str) {
        self.span.set_error(error.to_string());
    }
}

/// Jaeger tracer implementation
pub struct JaegerTracer {
    service_name: String,
    agent_endpoint: String,
    spans: Arc<RwLock<Vec<Span>>>,
}

impl JaegerTracer {
    pub fn new(service_name: String, agent_endpoint: String) -> Self {
        let spans = Arc::new(RwLock::new(Vec::new()));
        
        Self {
            service_name,
            agent_endpoint,
            spans,
        }
    }
    
    fn send_span_to_jaeger(span: &Span, endpoint: &str) {
        println!("📊 Sending span {} to Jaeger at {}", span.operation_name, endpoint);
    }
}

impl Tracer for JaegerTracer {
    fn start_span(&self, operation_name: &str, parent: Option<&SpanContext>) -> Box<dyn ActiveSpan> {
        let span = match parent {
            Some(parent_ctx) => Span::with_parent(
                operation_name.to_string(),
                self.service_name.clone(),
                parent_ctx,
            ),
            None => Span::new(operation_name.to_string(), self.service_name.clone()),
        };
        
        Box::new(JaegerSpan {
            span,
        })
    }
    
    fn extract_context(&self, carrier: &dyn TextMapReader) -> Option<SpanContext> {
        // Extract Jaeger headers (uber-trace-id)
        let uber_trace = carrier.get("uber-trace-id")?;
        let parts: Vec<&str> = uber_trace.split(':').collect();
        
        if parts.len() >= 4 {
            Some(SpanContext {
                trace_id: parts[0].to_string(),
                span_id: parts[1].to_string(),
                parent_span_id: if parts[2] == "0" { None } else { Some(parts[2].to_string()) },
                trace_flags: parts[3].parse().unwrap_or(0),
                trace_state: BTreeMap::new(),
                baggage: BTreeMap::new(),
            })
        } else {
            None
        }
    }
    
    fn inject_context(&self, context: &SpanContext, carrier: &mut dyn TextMapWriter) {
        let uber_trace = format!(
            "{}:{}:{}:{}",
            context.trace_id,
            context.span_id,
            context.parent_span_id.as_deref().unwrap_or("0"),
            context.trace_flags
        );
        carrier.set("uber-trace-id", &uber_trace);
    }
    
    fn flush(&self) -> Result<(), TracingError> {
        Ok(())
    }
    
    fn close(&self) -> Result<(), TracingError> {
        Ok(())
    }
}

/// Jaeger active span implementation
pub struct JaegerSpan {
    span: Span,
}

impl ActiveSpan for JaegerSpan {
    fn context(&self) -> &SpanContext {
        &self.span.context
    }
    
    fn set_tag(&mut self, key: &str, value: SpanValue) {
        self.span.set_tag(key.to_string(), value);
    }
    
    fn log(&mut self, message: &str, fields: BTreeMap<String, SpanValue>) {
        self.span.log(message.to_string(), fields);
    }
    
    fn set_operation_name(&mut self, name: &str) {
        self.span.operation_name = name.to_string();
    }
    
    fn finish(&mut self) {
        self.span.finish();
    }
    
    fn set_error(&mut self, error: &str) {
        self.span.set_error(error.to_string());
    }
}

/// Global tracer registry
pub struct TracerRegistry {
    tracers: RwLock<HashMap<String, Box<dyn Tracer>>>,
    default_tracer: RwLock<Option<String>>,
    sampling_rules: RwLock<Vec<SamplingRule>>,
    span_processors: RwLock<Vec<Box<dyn SpanProcessor>>>,
}

impl TracerRegistry {
    pub fn new() -> Self {
        Self {
            tracers: RwLock::new(HashMap::new()),
            default_tracer: RwLock::new(None),
            sampling_rules: RwLock::new(Vec::new()),
            span_processors: RwLock::new(Vec::new()),
        }
    }
    
    pub fn register_tracer(&self, name: String, tracer: Box<dyn Tracer>) {
        let mut tracers = self.tracers.write().unwrap();
        tracers.insert(name.clone(), tracer);
        
        // Set as default if none exists
        let mut default = self.default_tracer.write().unwrap();
        if default.is_none() {
            *default = Some(name);
        }
    }
    
    pub fn add_sampling_rule(&self, rule: SamplingRule) {
        let mut rules = self.sampling_rules.write().unwrap();
        rules.push(rule);
    }
    
    pub fn should_sample(&self, span: &Span) -> bool {
        let rules = self.sampling_rules.read().unwrap();
        for rule in rules.iter() {
            if rule.matches(span) {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                span.operation_name.hash(&mut hasher);
                let hash = hasher.finish();
                let random_val = (hash % 1000) as f64 / 1000.0;
                
                return rule.sample_rate > random_val;
            }
        }
        true // Default to sampling
    }
}

/// Sampling rule for trace sampling decisions
#[derive(Debug, Clone)]
pub struct SamplingRule {
    pub service_name: Option<String>,
    pub operation_name: Option<String>,
    pub sample_rate: f64,
    pub max_per_second: Option<u64>,
}

impl SamplingRule {
    pub fn matches(&self, span: &Span) -> bool {
        if let Some(service) = &self.service_name {
            if &span.service_name != service {
                return false;
            }
        }
        
        if let Some(operation) = &self.operation_name {
            if &span.operation_name != operation {
                return false;
            }
        }
        
        true
    }
}

/// Span processor for custom span handling
pub trait SpanProcessor: Send + Sync {
    fn on_start(&self, span: &mut Span);
    fn on_end(&self, span: &Span);
    fn force_flush(&self) -> Result<(), TracingError>;
    fn shutdown(&self) -> Result<(), TracingError>;
}

/// Metrics collecting span processor
pub struct MetricsSpanProcessor {
    span_count: AtomicU64,
    error_count: AtomicU64,
    duration_sum: AtomicU64,
}

impl MetricsSpanProcessor {
    pub fn new() -> Self {
        Self {
            span_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            duration_sum: AtomicU64::new(0),
        }
    }
    
    pub fn get_metrics(&self) -> SpanMetrics {
        SpanMetrics {
            span_count: self.span_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            avg_duration_ms: {
                let count = self.span_count.load(Ordering::Relaxed);
                if count > 0 {
                    self.duration_sum.load(Ordering::Relaxed) / count
                } else {
                    0
                }
            },
        }
    }
}

impl SpanProcessor for MetricsSpanProcessor {
    fn on_start(&self, _span: &mut Span) {
        self.span_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn on_end(&self, span: &Span) {
        if matches!(span.status, SpanStatus::Error) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
        
        if let Some(duration) = span.duration {
            self.duration_sum.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        }
    }
    
    fn force_flush(&self) -> Result<(), TracingError> {
        Ok(())
    }
    
    fn shutdown(&self) -> Result<(), TracingError> {
        Ok(())
    }
}

/// Span metrics collected by MetricsSpanProcessor
#[derive(Debug, Clone)]
pub struct SpanMetrics {
    pub span_count: u64,
    pub error_count: u64,
    pub avg_duration_ms: u64,
}

/// Tracing errors
#[derive(Debug, Clone)]
pub enum TracingError {
    BackendUnavailable,
    InvalidContext,
    SerializationError(String),
    NetworkError(String),
    ConfigurationError(String),
}

impl std::fmt::Display for TracingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracingError::BackendUnavailable => write!(f, "Tracing backend unavailable"),
            TracingError::InvalidContext => write!(f, "Invalid span context"),
            TracingError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            TracingError::NetworkError(e) => write!(f, "Network error: {}", e),
            TracingError::ConfigurationError(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl std::error::Error for TracingError {}

/// Generate the tracing instrumentation code
fn generate_tracing_code(input_fn: &ItemFn, config: &TracingConfig) -> TokenStream {
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_block = &input_fn.block;
    
    let fn_name_str = fn_name.to_string();
    let span_name = config.span_name.as_ref()
        .unwrap_or(&fn_name_str);
    let service_name_str = "SERVICE_NAME";
    let operation_name_str = format!("{}::{}", service_name_str, fn_name);
    let operation_name = config.operation_name.as_ref()
        .unwrap_or(&operation_name_str);
    
    let sample_rate = config.sample_rate;
    let include_result = config.include_result;
    
    let expanded = quote! {
        #fn_vis #fn_sig {
            use std::collections::BTreeMap;
            use std::env;
            
            // Get service name from environment
            let service_name = env::var(#service_name_str)
                .unwrap_or_else(|_| "unknown-service".to_string());
            
            // Check sampling decision
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            #span_name.hash(&mut hasher);
            let hash = hasher.finish();
            let random_val = (hash % 1000) as f64 / 1000.0;
            let should_sample = random_val < #sample_rate;
            
            if should_sample {
                let span_name = #span_name;
                let operation_name = #operation_name;
                
                println!("🔍 Starting trace span: {} ({})", span_name, operation_name);
                
                // Record start time
                let start_time = std::time::Instant::now();
                
                // Execute the function
                let result = (|| #fn_block)();
                
                // Record execution metrics
                let execution_time = start_time.elapsed();
                
                match &result {
                    Ok(_) => {
                        if #include_result {
                            println!("✅ Trace span completed: {} ({:.2}ms)", 
                                   span_name, execution_time.as_secs_f64() * 1000.0);
                        }
                    }
                    Err(e) => {
                        println!("❌ Trace span failed: {} - {:?} ({:.2}ms)", 
                               span_name, e, execution_time.as_secs_f64() * 1000.0);
                    }
                }
                
                result
            } else {
                // Not sampled, execute without tracing
                (|| #fn_block)()
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Create a new OpenTelemetry tracer
pub fn create_opentelemetry_tracer(service_name: String, endpoint: String) -> OpenTelemetryTracer {
    OpenTelemetryTracer::new(service_name, endpoint)
}

/// Create a new Jaeger tracer
pub fn create_jaeger_tracer(service_name: String, agent_endpoint: String) -> JaegerTracer {
    JaegerTracer::new(service_name, agent_endpoint)
}

/// Initialize global tracing with environment-based configuration
pub fn init_global_tracing() -> Result<(), TracingError> {
    let registry = TracerRegistry::new();
    
    let service_name = std::env::var("SERVICE_NAME")
        .unwrap_or_else(|_| "unknown-service".to_string());
    
    match std::env::var("TRACING_BACKEND").as_deref() {
        Ok("opentelemetry") | Ok("otel") => {
            let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string());
            let tracer = Box::new(OpenTelemetryTracer::new(service_name, endpoint));
            registry.register_tracer("opentelemetry".to_string(), tracer);
        }
        Ok("jaeger") => {
            let endpoint = std::env::var("JAEGER_ENDPOINT")
                .unwrap_or_else(|_| "localhost:14268".to_string());
            let tracer = Box::new(JaegerTracer::new(service_name, endpoint));
            registry.register_tracer("jaeger".to_string(), tracer);
        }
        _ => {
            // Default to console/noop tracer for development
            return Err(TracingError::ConfigurationError("No valid tracing backend configured".to_string()));
        }
    }
    
    Ok(())
}

/// Generate distributed tracing preprocessing code
pub fn generate_tracing_preprocessing() -> proc_macro2::TokenStream {
    quote! {
        // Initialize distributed tracing system
        if let Ok(backend) = std::env::var("TRACING_BACKEND") {
            match rabbitmesh_macros::observability::tracing::init_global_tracing() {
                Ok(_) => println!("🔍 Distributed tracing initialized with backend: {}", backend),
                Err(e) => eprintln!("⚠️ Failed to initialize tracing: {}", e),
            }
        }
        
        Ok(())
    }
}