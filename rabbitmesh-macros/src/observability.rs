//! Universal Observability and Monitoring Macros for RabbitMesh
//! 
//! This module provides comprehensive observability and monitoring macros that work
//! across ALL project types and operational environments:
//!
//! **Project Types Supported:**
//! - 🛒 E-commerce (order flows, payment processing, inventory tracking)
//! - 💰 Finance (trading metrics, transaction monitoring, risk analytics)
//! - 🏥 Healthcare (patient flow, system uptime, compliance monitoring)
//! - 🏭 IoT (sensor metrics, device health, data pipeline monitoring)
//! - 🎮 Gaming (player metrics, match performance, server health)
//! - 📱 Social Media (engagement metrics, content moderation, user activity)
//! - 🏢 Enterprise (business metrics, system performance, user behavior)
//! - 🎓 Education (learning analytics, system usage, performance tracking)
//! - 📦 Logistics (delivery tracking, route optimization, fleet monitoring)
//! - 🌍 GIS/Mapping (location analytics, spatial queries, map rendering)
//!
//! **Observability Features:**
//! - Metrics (Counters, Gauges, Histograms, Summaries)
//! - Distributed Tracing (OpenTelemetry, Jaeger, Zipkin)
//! - Structured Logging (JSON, Splunk, ELK Stack)
//! - Health Checks (Liveness, Readiness, Custom checks)
//! - Alerting (Prometheus, PagerDuty, Slack, Email)
//! - Profiling (CPU, Memory, Performance bottlenecks)

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Attribute, LitStr, LitInt, LitBool};
use std::collections::HashMap;
use crate::universal::{MacroAttribute, MacroValue, MacroContext};

/// Universal observability and monitoring processor
pub struct ObservabilityProcessor;

impl ObservabilityProcessor {
    /// Generate observability logic for any macro attribute
    pub fn generate_observability_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        match attr.name.as_str() {
            "metrics" => Self::generate_metrics_logic(attr, context),
            "trace" => Self::generate_tracing_logic(attr, context),
            "log" => Self::generate_logging_logic(attr, context),
            "monitor" => Self::generate_monitoring_logic(attr, context),
            "alert" => Self::generate_alerting_logic(attr, context),
            "health_check" => Self::generate_health_check_logic(attr, context),
            "profile" => Self::generate_profiling_logic(attr, context),
            "benchmark" => Self::generate_benchmarking_logic(attr, context),
            "audit" => Self::generate_audit_logic(attr, context),
            "dashboard" => Self::generate_dashboard_logic(attr, context),
            "sla" => Self::generate_sla_monitoring(attr, context),
            "anomaly_detection" => Self::generate_anomaly_detection(attr, context),
            _ => quote! {}
        }
    }

    /// Generate metrics collection logic
    fn generate_metrics_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let counter_name = Self::extract_string_arg(attr, "counter");
        let histogram_name = Self::extract_string_arg(attr, "histogram");
        let gauge_name = Self::extract_string_arg(attr, "gauge");
        let labels = Self::extract_metric_labels(attr);
        let domain_metrics = Self::generate_domain_metrics(context, attr);
        
        quote! {
            // Universal metrics collection
            debug!("📊 Initializing metrics collection");
            
            let metrics_start_time = std::time::Instant::now();
            let method_name = #context.method_name;
            let service_name = #context.service_name;
            
            // Domain-specific metrics setup
            #domain_metrics
            
            // Counter metrics
            if let Some(counter) = #counter_name {
                let counter_labels = vec![#(#labels),*];
                rabbitmesh_metrics::increment_counter(&counter, &counter_labels).await;
                debug!("📈 Incremented counter: {}", counter);
            }
            
            // Gauge metrics (will be updated based on method result)
            if let Some(gauge) = #gauge_name {
                let gauge_labels = vec![#(#labels),*];
                rabbitmesh_metrics::set_gauge(&gauge, 1.0, &gauge_labels).await;
                debug!("📏 Set gauge: {}", gauge);
            }
            
            // Histogram metrics setup (duration will be recorded after method execution)
            let histogram_name = #histogram_name;
        }
    }

    /// Generate distributed tracing logic
    fn generate_tracing_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let span_name = Self::extract_string_arg(attr, "span_name")
            .unwrap_or_else(|| format!("{}_{}", context.service_name, context.method_name));
        let include_fields = Self::extract_trace_fields(attr);
        let trace_level = Self::extract_string_arg(attr, "level").unwrap_or_else(|| "info".to_string());
        let domain_tracing = Self::generate_domain_tracing(context);
        
        quote! {
            // Universal distributed tracing
            debug!("🔍 Starting distributed tracing");
            
            // Create root span for this method
            let span = tracing::span!(
                tracing::Level::INFO,
                #span_name,
                service = #context.service_name,
                method = #context.method_name,
                #(#include_fields),*
            );
            let _guard = span.enter();
            
            // Extract trace context from message
            let trace_context = rabbitmesh_tracing::extract_trace_context(&msg).await;
            
            #domain_tracing
            
            // Inject trace context for downstream services
            rabbitmesh_tracing::inject_trace_context(&mut msg, &trace_context).await;
            
            debug!("🔍 Tracing context established");
        }
    }

    /// Generate structured logging logic
    fn generate_logging_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let log_level = Self::extract_string_arg(attr, "level").unwrap_or_else(|| "info".to_string());
        let include_request = Self::extract_bool_arg(attr, "include_request").unwrap_or(true);
        let include_response = Self::extract_bool_arg(attr, "include_response").unwrap_or(true);
        let sensitive_fields = Self::extract_sensitive_fields(attr);
        let domain_logging = Self::generate_domain_logging(context);
        
        quote! {
            // Universal structured logging
            debug!("📝 Setting up structured logging");
            
            let log_context = rabbitmesh_logging::LogContext::new()
                .with_service(#context.service_name)
                .with_method(#context.method_name)
                .with_trace_id(&msg.get_trace_id())
                .with_request_id(&msg.get_request_id());
            
            #domain_logging
            
            // Log request (with sensitive data filtering)
            if #include_request {
                let filtered_request = rabbitmesh_logging::filter_sensitive_data(
                    &msg, 
                    &vec![#(#sensitive_fields),*]
                ).await;
                
                match #log_level.as_str() {
                    "debug" => debug!(?filtered_request, "📥 Request received"),
                    "info" => info!(?filtered_request, "📥 Request received"),
                    "warn" => warn!(?filtered_request, "📥 Request received"),
                    "error" => error!(?filtered_request, "📥 Request received"),
                    _ => info!(?filtered_request, "📥 Request received"),
                }
            }
            
            // Response logging will be handled after method execution
            let should_log_response = #include_response;
        }
    }

    /// Generate monitoring and health checks
    fn generate_monitoring_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let check_dependencies = Self::extract_bool_arg(attr, "check_dependencies").unwrap_or(true);
        let check_resources = Self::extract_bool_arg(attr, "check_resources").unwrap_or(true);
        let timeout_ms = Self::extract_int_arg(attr, "timeout").unwrap_or(5000);
        let domain_monitoring = Self::generate_domain_monitoring(context);
        
        quote! {
            // Universal monitoring and health checks
            debug!("🏥 Starting health monitoring");
            
            let health_check_start = std::time::Instant::now();
            let mut health_status = rabbitmesh_monitoring::HealthStatus::new();
            
            #domain_monitoring
            
            // Check service dependencies
            if #check_dependencies {
                let dependency_health = rabbitmesh_monitoring::check_dependencies(#timeout_ms).await?;
                health_status.add_dependency_checks(dependency_health);
            }
            
            // Check system resources
            if #check_resources {
                let resource_health = rabbitmesh_monitoring::check_system_resources().await?;
                health_status.add_resource_checks(resource_health);
            }
            
            // Record health check duration
            let health_check_duration = health_check_start.elapsed();
            rabbitmesh_metrics::record_histogram(
                "health_check_duration_ms",
                health_check_duration.as_millis() as f64,
                &[("service", #context.service_name)]
            ).await;
            
            debug!("🏥 Health monitoring completed in {:?}", health_check_duration);
        }
    }

    /// Generate alerting logic
    fn generate_alerting_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let alert_channels = Self::extract_alert_channels(attr);
        let severity_level = Self::extract_string_arg(attr, "severity").unwrap_or_else(|| "medium".to_string());
        let alert_conditions = Self::extract_alert_conditions(attr);
        let domain_alerts = Self::generate_domain_alerts(context);
        
        quote! {
            // Universal alerting system
            debug!("🚨 Setting up alerting");
            
            let alert_manager = rabbitmesh_alerting::AlertManager::new()
                .with_service(#context.service_name)
                .with_method(#context.method_name)
                .with_severity(#severity_level);
            
            #domain_alerts
            
            // Configure alert channels
            let alert_channels = vec![#(#alert_channels),*];
            for channel in alert_channels {
                alert_manager.add_channel(channel).await;
            }
            
            // Set up alert conditions
            let alert_conditions = vec![#(#alert_conditions),*];
            for condition in alert_conditions {
                alert_manager.add_condition(condition).await;
            }
            
            debug!("🚨 Alert configuration completed");
        }
    }

    /// Generate health check logic
    fn generate_health_check_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let check_type = Self::extract_string_arg(attr, "type").unwrap_or_else(|| "readiness".to_string());
        let timeout_seconds = Self::extract_int_arg(attr, "timeout").unwrap_or(30);
        let custom_checks = Self::extract_custom_health_checks(attr);
        let domain_health_checks = Self::generate_domain_health_checks(context);
        
        quote! {
            // Universal health checks
            debug!("💓 Performing {} health check", #check_type);
            
            let health_checker = rabbitmesh_health::HealthChecker::new()
                .with_timeout(#timeout_seconds)
                .with_service(#context.service_name);
            
            #domain_health_checks
            
            // Perform health check based on type
            let health_result = match #check_type.as_str() {
                "liveness" => health_checker.check_liveness().await?,
                "readiness" => health_checker.check_readiness().await?,
                "startup" => health_checker.check_startup().await?,
                "custom" => {
                    let custom_checks = vec![#(#custom_checks),*];
                    health_checker.check_custom(custom_checks).await?
                },
                _ => health_checker.check_readiness().await?
            };
            
            // Record health check metrics
            rabbitmesh_metrics::set_gauge(
                &format!("{}_health_status", #check_type),
                if health_result.is_healthy() { 1.0 } else { 0.0 },
                &[("service", #context.service_name)]
            ).await;
            
            debug!("💓 Health check result: {:?}", health_result);
        }
    }

    /// Generate profiling logic
    fn generate_profiling_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let profile_cpu = Self::extract_bool_arg(attr, "cpu").unwrap_or(true);
        let profile_memory = Self::extract_bool_arg(attr, "memory").unwrap_or(true);
        let sample_rate = Self::extract_float_arg(attr, "sample_rate").unwrap_or(0.1);
        let profile_duration = Self::extract_int_arg(attr, "duration").unwrap_or(60);
        
        quote! {
            // Universal profiling
            debug!("🔬 Starting performance profiling");
            
            let profiler = rabbitmesh_profiling::Profiler::new()
                .with_sample_rate(#sample_rate)
                .with_duration(std::time::Duration::from_secs(#profile_duration as u64));
            
            // CPU profiling
            if #profile_cpu {
                profiler.start_cpu_profiling().await?;
                debug!("🔬 CPU profiling started");
            }
            
            // Memory profiling  
            if #profile_memory {
                profiler.start_memory_profiling().await?;
                debug!("🔬 Memory profiling started");
            }
            
            // Method-specific profiling will be collected after execution
            let profiling_enabled = #profile_cpu || #profile_memory;
        }
    }

    /// Generate benchmarking logic
    fn generate_benchmarking_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let benchmark_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| format!("{}_{}_benchmark", context.service_name, context.method_name));
        let iterations = Self::extract_int_arg(attr, "iterations").unwrap_or(1000);
        let warmup_iterations = Self::extract_int_arg(attr, "warmup").unwrap_or(100);
        
        quote! {
            // Universal benchmarking
            debug!("⚡ Setting up benchmark: {}", #benchmark_name);
            
            let benchmark = rabbitmesh_benchmarking::Benchmark::new(#benchmark_name)
                .with_iterations(#iterations as usize)
                .with_warmup(#warmup_iterations as usize);
            
            // Benchmark will run multiple iterations of the method
            // Results will be collected and reported after execution
            let benchmark_enabled = true;
        }
    }

    /// Generate audit logging
    fn generate_audit_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let audit_fields = Self::extract_audit_fields(attr);
        let include_user = Self::extract_bool_arg(attr, "include_user").unwrap_or(true);
        let include_timestamp = Self::extract_bool_arg(attr, "include_timestamp").unwrap_or(true);
        let audit_level = Self::extract_string_arg(attr, "level").unwrap_or_else(|| "info".to_string());
        let domain_audit = Self::generate_domain_audit(context);
        
        quote! {
            // Universal audit logging
            debug!("📋 Setting up audit logging");
            
            let mut audit_log = rabbitmesh_audit::AuditLog::new()
                .with_service(#context.service_name)
                .with_method(#context.method_name)
                .with_level(#audit_level);
            
            #domain_audit
            
            // Include user information
            if #include_user {
                if let Some(user_id) = msg.get_header("user_id") {
                    audit_log = audit_log.with_user_id(user_id);
                }
                if let Some(session_id) = msg.get_header("session_id") {
                    audit_log = audit_log.with_session_id(session_id);
                }
            }
            
            // Include timestamp
            if #include_timestamp {
                audit_log = audit_log.with_timestamp(chrono::Utc::now());
            }
            
            // Add custom audit fields
            let audit_fields = vec![#(#audit_fields),*];
            for (key, value) in audit_fields {
                audit_log = audit_log.with_field(key, value);
            }
            
            debug!("📋 Audit logging configured");
        }
    }

    /// Generate dashboard integration
    fn generate_dashboard_logic(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let dashboard_name = Self::extract_string_arg(attr, "name")
            .unwrap_or_else(|| format!("{}_dashboard", context.service_name));
        let dashboard_type = Self::extract_string_arg(attr, "type").unwrap_or_else(|| "grafana".to_string());
        let widgets = Self::extract_dashboard_widgets(attr);
        
        quote! {
            // Universal dashboard integration
            debug!("📊 Setting up dashboard integration: {}", #dashboard_name);
            
            let dashboard_client = rabbitmesh_dashboard::DashboardClient::new(#dashboard_type)
                .with_name(#dashboard_name)
                .with_service(#context.service_name);
            
            // Configure dashboard widgets
            let widgets = vec![#(#widgets),*];
            for widget in widgets {
                dashboard_client.add_widget(widget).await?;
            }
            
            // Real-time metrics will be pushed to dashboard during method execution
            let dashboard_integration_enabled = true;
            
            debug!("📊 Dashboard integration configured");
        }
    }

    /// Generate SLA monitoring
    fn generate_sla_monitoring(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let sla_target = Self::extract_float_arg(attr, "target").unwrap_or(99.9);
        let response_time_limit = Self::extract_int_arg(attr, "response_time_ms").unwrap_or(1000);
        let error_rate_limit = Self::extract_float_arg(attr, "error_rate_limit").unwrap_or(0.1);
        
        quote! {
            // SLA monitoring
            debug!("📈 Setting up SLA monitoring (target: {}%)", #sla_target);
            
            let sla_monitor = rabbitmesh_sla::SLAMonitor::new()
                .with_target(#sla_target)
                .with_response_time_limit(#response_time_limit as u64)
                .with_error_rate_limit(#error_rate_limit)
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            // Track SLA metrics during method execution
            let sla_tracking_enabled = true;
        }
    }

    /// Generate anomaly detection
    fn generate_anomaly_detection(attr: &MacroAttribute, context: &MacroContext) -> TokenStream {
        let detection_algorithm = Self::extract_string_arg(attr, "algorithm").unwrap_or_else(|| "statistical".to_string());
        let sensitivity = Self::extract_float_arg(attr, "sensitivity").unwrap_or(0.8);
        let window_size = Self::extract_int_arg(attr, "window_size").unwrap_or(100);
        
        quote! {
            // Anomaly detection
            debug!("🔍 Setting up anomaly detection (algorithm: {})", #detection_algorithm);
            
            let anomaly_detector = rabbitmesh_anomaly::AnomalyDetector::new()
                .with_algorithm(#detection_algorithm)
                .with_sensitivity(#sensitivity)
                .with_window_size(#window_size as usize)
                .with_service(#context.service_name)
                .with_method(#context.method_name);
            
            // Real-time anomaly detection during method execution
            let anomaly_detection_enabled = true;
        }
    }

    // Domain-specific observability generators

    /// Generate domain-specific metrics
    fn generate_domain_metrics(context: &MacroContext, attr: &MacroAttribute) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let method_name = &context.method_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce specific metrics
                if #method_name.contains("order") {
                    rabbitmesh_metrics::increment_counter("ecommerce_orders_total", &[("service", #context.service_name)]).await;
                    rabbitmesh_metrics::increment_counter("ecommerce_revenue_tracking", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("product") {
                    rabbitmesh_metrics::increment_counter("ecommerce_product_views", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("cart") {
                    rabbitmesh_metrics::increment_counter("ecommerce_cart_operations", &[("service", #context.service_name)]).await;
                }
            },
            "finance" => quote! {
                // Finance specific metrics
                if #method_name.contains("trade") || #method_name.contains("order") {
                    rabbitmesh_metrics::increment_counter("finance_trades_total", &[("service", #context.service_name)]).await;
                    rabbitmesh_metrics::record_histogram("finance_trade_value", 0.0, &[("service", #context.service_name)]).await;
                } else if #method_name.contains("portfolio") {
                    rabbitmesh_metrics::set_gauge("finance_portfolio_value", 0.0, &[("service", #context.service_name)]).await;
                } else if #method_name.contains("risk") {
                    rabbitmesh_metrics::record_histogram("finance_risk_score", 0.0, &[("service", #context.service_name)]).await;
                }
            },
            "healthcare" => quote! {
                // Healthcare specific metrics
                if #method_name.contains("patient") {
                    rabbitmesh_metrics::increment_counter("healthcare_patient_interactions", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("appointment") {
                    rabbitmesh_metrics::increment_counter("healthcare_appointments_scheduled", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("diagnosis") {
                    rabbitmesh_metrics::increment_counter("healthcare_diagnoses_created", &[("service", #context.service_name)]).await;
                }
            },
            "iot" => quote! {
                // IoT specific metrics
                if #method_name.contains("sensor") {
                    rabbitmesh_metrics::increment_counter("iot_sensor_readings", &[("service", #context.service_name)]).await;
                    rabbitmesh_metrics::set_gauge("iot_active_sensors", 0.0, &[("service", #context.service_name)]).await;
                } else if #method_name.contains("device") {
                    rabbitmesh_metrics::increment_counter("iot_device_communications", &[("service", #context.service_name)]).await;
                }
            },
            "gaming" => quote! {
                // Gaming specific metrics
                if #method_name.contains("player") {
                    rabbitmesh_metrics::increment_counter("gaming_player_actions", &[("service", #context.service_name)]).await;
                    rabbitmesh_metrics::set_gauge("gaming_active_players", 0.0, &[("service", #context.service_name)]).await;
                } else if #method_name.contains("match") {
                    rabbitmesh_metrics::increment_counter("gaming_matches_started", &[("service", #context.service_name)]).await;
                    rabbitmesh_metrics::record_histogram("gaming_match_duration", 0.0, &[("service", #context.service_name)]).await;
                }
            },
            "social" => quote! {
                // Social media specific metrics
                if #method_name.contains("post") {
                    rabbitmesh_metrics::increment_counter("social_posts_created", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("like") || #method_name.contains("reaction") {
                    rabbitmesh_metrics::increment_counter("social_engagement_total", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("feed") {
                    rabbitmesh_metrics::increment_counter("social_feed_requests", &[("service", #context.service_name)]).await;
                }
            },
            "logistics" => quote! {
                // Logistics specific metrics
                if #method_name.contains("shipment") || #method_name.contains("delivery") {
                    rabbitmesh_metrics::increment_counter("logistics_shipments_processed", &[("service", #context.service_name)]).await;
                } else if #method_name.contains("route") {
                    rabbitmesh_metrics::record_histogram("logistics_route_distance", 0.0, &[("service", #context.service_name)]).await;
                } else if #method_name.contains("tracking") {
                    rabbitmesh_metrics::increment_counter("logistics_tracking_updates", &[("service", #context.service_name)]).await;
                }
            },
            _ => quote! {
                // Generic metrics
                rabbitmesh_metrics::increment_counter("method_calls_total", &[("service", #context.service_name), ("method", #method_name)]).await;
            }
        }
    }

    /// Generate domain-specific tracing
    fn generate_domain_tracing(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce tracing context
                if let Some(user_id) = msg.get_header("user_id") {
                    tracing::Span::current().record("customer_id", &user_id);
                }
                if let Some(order_id) = msg.get_path_param("order_id") {
                    tracing::Span::current().record("order_id", &order_id);
                }
                if let Some(product_id) = msg.get_path_param("product_id") {
                    tracing::Span::current().record("product_id", &product_id);
                }
            },
            "finance" => quote! {
                // Finance tracing context
                if let Some(account_id) = msg.get_header("account_id") {
                    tracing::Span::current().record("account_id", &account_id);
                }
                if let Some(portfolio_id) = msg.get_path_param("portfolio_id") {
                    tracing::Span::current().record("portfolio_id", &portfolio_id);
                }
                if let Some(trade_id) = msg.get_path_param("trade_id") {
                    tracing::Span::current().record("trade_id", &trade_id);
                }
            },
            "healthcare" => quote! {
                // Healthcare tracing context (with privacy protection)
                if let Some(patient_hash) = msg.get_header("patient_hash") {
                    tracing::Span::current().record("patient_ref", &patient_hash);
                }
                if let Some(appointment_id) = msg.get_path_param("appointment_id") {
                    tracing::Span::current().record("appointment_id", &appointment_id);
                }
            },
            "iot" => quote! {
                // IoT tracing context
                if let Some(device_id) = msg.get_header("device_id") {
                    tracing::Span::current().record("device_id", &device_id);
                }
                if let Some(sensor_type) = msg.get_header("sensor_type") {
                    tracing::Span::current().record("sensor_type", &sensor_type);
                }
            },
            "gaming" => quote! {
                // Gaming tracing context
                if let Some(player_id) = msg.get_header("player_id") {
                    tracing::Span::current().record("player_id", &player_id);
                }
                if let Some(match_id) = msg.get_path_param("match_id") {
                    tracing::Span::current().record("match_id", &match_id);
                }
                if let Some(game_session) = msg.get_header("game_session") {
                    tracing::Span::current().record("game_session", &game_session);
                }
            },
            _ => quote! {
                // Generic tracing context
                if let Some(request_id) = msg.get_header("request_id") {
                    tracing::Span::current().record("request_id", &request_id);
                }
            }
        }
    }

    /// Generate domain-specific logging
    fn generate_domain_logging(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "healthcare" => quote! {
                // Healthcare logging with HIPAA compliance
                log_context = log_context
                    .with_compliance("HIPAA")
                    .with_data_classification("PHI");
            },
            "finance" => quote! {
                // Finance logging with compliance
                log_context = log_context
                    .with_compliance("SOX,PCI-DSS")
                    .with_data_classification("Financial");
            },
            _ => quote! {
                // Standard logging configuration
                log_context = log_context.with_compliance("GDPR");
            }
        }
    }

    /// Generate domain-specific monitoring
    fn generate_domain_monitoring(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        let service_name = &context.service_name;
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce specific monitoring
                health_status.add_check("payment_gateway", 
                    rabbitmesh_monitoring::check_external_service("payment_gateway").await?);
                health_status.add_check("inventory_database",
                    rabbitmesh_monitoring::check_database_connection("inventory").await?);
                health_status.add_check("product_search",
                    rabbitmesh_monitoring::check_search_service("products").await?);
            },
            "finance" => quote! {
                // Finance specific monitoring
                health_status.add_check("market_data_feed",
                    rabbitmesh_monitoring::check_market_data_connection().await?);
                health_status.add_check("trading_engine",
                    rabbitmesh_monitoring::check_trading_engine_health().await?);
                health_status.add_check("risk_calculator",
                    rabbitmesh_monitoring::check_risk_service().await?);
            },
            "healthcare" => quote! {
                // Healthcare specific monitoring
                health_status.add_check("patient_database",
                    rabbitmesh_monitoring::check_database_connection("patients").await?);
                health_status.add_check("imaging_system",
                    rabbitmesh_monitoring::check_external_service("imaging").await?);
                health_status.add_check("lab_integration",
                    rabbitmesh_monitoring::check_external_service("lab_system").await?);
            },
            "iot" => quote! {
                // IoT specific monitoring
                health_status.add_check("device_connectivity",
                    rabbitmesh_monitoring::check_device_connectivity().await?);
                health_status.add_check("time_series_database",
                    rabbitmesh_monitoring::check_database_connection("timeseries").await?);
                health_status.add_check("message_broker",
                    rabbitmesh_monitoring::check_message_broker_health().await?);
            },
            "gaming" => quote! {
                // Gaming specific monitoring
                health_status.add_check("game_servers",
                    rabbitmesh_monitoring::check_game_servers().await?);
                health_status.add_check("leaderboard_service",
                    rabbitmesh_monitoring::check_external_service("leaderboard").await?);
                health_status.add_check("matchmaking_service",
                    rabbitmesh_monitoring::check_external_service("matchmaking").await?);
            },
            _ => quote! {
                // Generic monitoring
                health_status.add_check("database",
                    rabbitmesh_monitoring::check_primary_database().await?);
                health_status.add_check("cache",
                    rabbitmesh_monitoring::check_cache_connection().await?);
            }
        }
    }

    /// Generate domain-specific alerts
    fn generate_domain_alerts(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                // E-commerce specific alerts
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("order_failure_rate")
                    .threshold(0.05)  // 5% failure rate
                    .action("notify_operations")
                ).await;
                
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("payment_processing_time")
                    .threshold(5000.0) // 5 seconds
                    .action("scale_payment_service")
                ).await;
            },
            "finance" => quote! {
                // Finance specific alerts
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("trading_latency")
                    .threshold(100.0) // 100ms
                    .severity("critical")
                    .action("notify_trading_desk")
                ).await;
                
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("risk_limit_breach")
                    .threshold(1.0)
                    .severity("critical")
                    .action("halt_trading")
                ).await;
            },
            "healthcare" => quote! {
                // Healthcare specific alerts
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("patient_data_access_anomaly")
                    .threshold(1.0)
                    .severity("high")
                    .action("notify_compliance_team")
                ).await;
            },
            _ => quote! {
                // Generic alerts
                alert_manager.add_condition(rabbitmesh_alerting::AlertCondition::new()
                    .metric("error_rate")
                    .threshold(0.01) // 1% error rate
                    .action("notify_team")
                ).await;
            }
        }
    }

    /// Generate domain-specific health checks
    fn generate_domain_health_checks(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "ecommerce" => quote! {
                health_checker.add_dependency("payment-gateway")
                    .add_dependency("inventory-service")
                    .add_dependency("product-catalog")
                    .add_dependency("user-service");
            },
            "finance" => quote! {
                health_checker.add_dependency("market-data-service")
                    .add_dependency("trading-engine")
                    .add_dependency("risk-service")
                    .add_dependency("account-service");
            },
            "healthcare" => quote! {
                health_checker.add_dependency("patient-service")
                    .add_dependency("imaging-service")
                    .add_dependency("lab-service")
                    .add_dependency("scheduling-service");
            },
            "iot" => quote! {
                health_checker.add_dependency("device-registry")
                    .add_dependency("telemetry-service")
                    .add_dependency("time-series-db")
                    .add_dependency("alert-service");
            },
            "gaming" => quote! {
                health_checker.add_dependency("player-service")
                    .add_dependency("matchmaking-service")
                    .add_dependency("leaderboard-service")
                    .add_dependency("game-state-service");
            },
            _ => quote! {
                health_checker.add_dependency("database")
                    .add_dependency("cache")
                    .add_dependency("message-queue");
            }
        }
    }

    /// Generate domain-specific audit logging
    fn generate_domain_audit(context: &MacroContext) -> TokenStream {
        let domain = Self::infer_domain_from_context(context);
        
        match domain.as_str() {
            "healthcare" => quote! {
                // Healthcare audit logging (HIPAA compliance)
                audit_log = audit_log
                    .with_compliance_standard("HIPAA")
                    .with_data_classification("PHI")
                    .with_retention_policy("7_years");
            },
            "finance" => quote! {
                // Finance audit logging (SOX/PCI compliance)
                audit_log = audit_log
                    .with_compliance_standard("SOX,PCI-DSS")
                    .with_data_classification("Financial")
                    .with_retention_policy("7_years");
            },
            _ => quote! {
                // Standard audit logging
                audit_log = audit_log
                    .with_compliance_standard("GDPR")
                    .with_retention_policy("3_years");
            }
        }
    }

    // Helper methods for extracting configuration

    /// Extract metric labels
    fn extract_metric_labels(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut labels = Vec::new();
        
        if let Some(MacroValue::Array(label_array)) = attr.args.get("labels") {
            for label in label_array {
                if let MacroValue::String(label_str) = label {
                    let parts: Vec<&str> = label_str.split('.').collect();
                    if parts.len() == 2 {
                        let source = parts[0];
                        let field = parts[1];
                        labels.push(quote! {
                            (#field, msg.get_field(#source, #field).unwrap_or_else(|| "unknown".to_string()))
                        });
                    }
                }
            }
        }
        
        labels
    }

    /// Extract trace fields
    fn extract_trace_fields(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut fields = Vec::new();
        
        if let Some(MacroValue::Array(field_array)) = attr.args.get("include") {
            for field in field_array {
                if let MacroValue::String(field_str) = field {
                    let field_ident = format_ident!("{}", field_str.replace(".", "_"));
                    fields.push(quote! {
                        #field_ident = tracing::field::Empty
                    });
                }
            }
        }
        
        fields
    }

    /// Extract sensitive fields for logging
    fn extract_sensitive_fields(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut fields = vec![
            quote! { "password".to_string() },
            quote! { "credit_card".to_string() },
            quote! { "ssn".to_string() },
            quote! { "api_key".to_string() },
        ];
        
        if let Some(MacroValue::Array(field_array)) = attr.args.get("sensitive_fields") {
            for field in field_array {
                if let MacroValue::String(field_str) = field {
                    fields.push(quote! { #field_str.to_string() });
                }
            }
        }
        
        fields
    }

    /// Extract alert channels
    fn extract_alert_channels(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut channels = Vec::new();
        
        if let Some(MacroValue::Array(channel_array)) = attr.args.get("channels") {
            for channel in channel_array {
                if let MacroValue::String(channel_str) = channel {
                    channels.push(quote! {
                        rabbitmesh_alerting::AlertChannel::from_string(#channel_str)
                    });
                }
            }
        }
        
        channels
    }

    /// Extract alert conditions
    fn extract_alert_conditions(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut conditions = Vec::new();
        
        if let Some(MacroValue::Array(condition_array)) = attr.args.get("conditions") {
            for condition in condition_array {
                if let MacroValue::String(condition_str) = condition {
                    conditions.push(quote! {
                        rabbitmesh_alerting::AlertCondition::from_expression(#condition_str)
                    });
                }
            }
        }
        
        conditions
    }

    /// Extract custom health checks
    fn extract_custom_health_checks(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut checks = Vec::new();
        
        if let Some(MacroValue::Array(check_array)) = attr.args.get("custom_checks") {
            for check in check_array {
                if let MacroValue::String(check_str) = check {
                    let check_ident = format_ident!("{}", check_str);
                    checks.push(quote! {
                        Box::new(#check_ident) as Box<dyn rabbitmesh_health::HealthCheck>
                    });
                }
            }
        }
        
        checks
    }

    /// Extract audit fields
    fn extract_audit_fields(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut fields = Vec::new();
        
        if let Some(MacroValue::Array(field_array)) = attr.args.get("fields") {
            for field in field_array {
                if let MacroValue::String(field_str) = field {
                    fields.push(quote! {
                        (#field_str.to_string(), msg.get_field_value(#field_str).unwrap_or_else(|| "unknown".to_string()))
                    });
                }
            }
        }
        
        fields
    }

    /// Extract dashboard widgets
    fn extract_dashboard_widgets(attr: &MacroAttribute) -> Vec<TokenStream> {
        let mut widgets = Vec::new();
        
        if let Some(MacroValue::Array(widget_array)) = attr.args.get("widgets") {
            for widget in widget_array {
                if let MacroValue::String(widget_str) = widget {
                    widgets.push(quote! {
                        rabbitmesh_dashboard::Widget::from_config(#widget_str)
                    });
                }
            }
        }
        
        widgets
    }

    /// Extract string argument
    fn extract_string_arg(attr: &MacroAttribute, key: &str) -> Option<String> {
        if let Some(MacroValue::String(value)) = attr.args.get(key) {
            Some(value.clone())
        } else {
            None
        }
    }

    /// Extract integer argument
    fn extract_int_arg(attr: &MacroAttribute, key: &str) -> Option<i64> {
        if let Some(MacroValue::Integer(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Extract boolean argument
    fn extract_bool_arg(attr: &MacroAttribute, key: &str) -> Option<bool> {
        if let Some(MacroValue::Boolean(value)) = attr.args.get(key) {
            Some(*value)
        } else {
            None
        }
    }

    /// Extract float argument
    fn extract_float_arg(attr: &MacroAttribute, key: &str) -> Option<f64> {
        if let Some(MacroValue::Integer(value)) = attr.args.get(key) {
            Some(*value as f64)
        } else {
            None
        }
    }

    /// Infer domain from context
    fn infer_domain_from_context(context: &MacroContext) -> String {
        let service_name = context.service_name.to_lowercase();
        let method_name = context.method_name.to_lowercase();
        
        // Check service name patterns
        if service_name.contains("ecommerce") || service_name.contains("shop") || service_name.contains("product") || service_name.contains("cart") {
            return "ecommerce".to_string();
        }
        if service_name.contains("finance") || service_name.contains("trading") || service_name.contains("bank") || service_name.contains("payment") {
            return "finance".to_string();
        }
        if service_name.contains("health") || service_name.contains("medical") || service_name.contains("patient") {
            return "healthcare".to_string();
        }
        if service_name.contains("iot") || service_name.contains("sensor") || service_name.contains("device") {
            return "iot".to_string();
        }
        if service_name.contains("game") || service_name.contains("gaming") || service_name.contains("player") {
            return "gaming".to_string();
        }
        if service_name.contains("social") || service_name.contains("media") || service_name.contains("feed") {
            return "social".to_string();
        }
        if service_name.contains("logistics") || service_name.contains("shipping") || service_name.contains("delivery") {
            return "logistics".to_string();
        }
        
        // Check method name patterns
        if method_name.contains("product") || method_name.contains("cart") || method_name.contains("order") {
            return "ecommerce".to_string();
        }
        if method_name.contains("trade") || method_name.contains("portfolio") || method_name.contains("account") {
            return "finance".to_string();
        }
        if method_name.contains("patient") || method_name.contains("medical") || method_name.contains("health") {
            return "healthcare".to_string();
        }
        if method_name.contains("sensor") || method_name.contains("device") || method_name.contains("telemetry") {
            return "iot".to_string();
        }
        if method_name.contains("player") || method_name.contains("game") || method_name.contains("score") {
            return "gaming".to_string();
        }
        if method_name.contains("post") || method_name.contains("feed") || method_name.contains("social") {
            return "social".to_string();
        }
        if method_name.contains("ship") || method_name.contains("deliver") || method_name.contains("route") {
            return "logistics".to_string();
        }
        
        "generic".to_string()
    }
}