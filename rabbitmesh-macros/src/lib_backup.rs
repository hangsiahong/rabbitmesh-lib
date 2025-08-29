//! # RabbitMesh Universal Macro Framework ✨
//!
//! **The most comprehensive procedural macro framework for microservices - supporting RBAC, ABAC, 
//! caching, validation, observability, workflows, and more across ALL project domains.**
//!
//! This crate provides both the foundational service macros AND the complete universal macro framework:
//!
//! ## 🏗️ Core Service Macros
//! - `#[service_definition]` - Marks a struct as a RabbitMesh service  
//! - `#[service_impl]` - Processes impl blocks to auto-register methods
//! - `#[service_method]` - Defines HTTP routes and RPC handlers
//!
//! ## 🌍 Universal Macro Framework
//! - **Authorization**: RBAC, ABAC, Hybrid patterns for all domains
//! - **Database**: Universal transaction management for SQL/NoSQL/Graph/TimeSeries
//! - **Caching**: Multi-level intelligent caching with domain optimizations
//! - **Validation**: Comprehensive input validation with security and compliance
//! - **Observability**: Metrics, tracing, logging, monitoring for all project types
//! - **Workflows**: State machines, sagas, approvals, event sourcing, CQRS
//!
//! ## 🪄 The Magic - From Simple Code to Enterprise-Grade Features
//!
//! **Before (Traditional Approach):** 200+ lines of boilerplate per method
//! **After (RabbitMesh):** Pure business logic with powerful macros
//!
//! ### 🔒 E-commerce with RBAC Authorization
//! ```rust,ignore
//! #[service_impl]
//! impl EcommerceService {
//!     #[service_method("POST /orders")]
//!     #[require_role("Customer")]           // RBAC authorization
//!     #[transactional]                     // Database transaction
//!     #[validate]                          // Input validation
//!     #[cached(ttl = 300)]                // Intelligent caching
//!     #[metrics(counter = "orders_total")] // Auto metrics
//!     pub async fn create_order(
//!         user: AuthContext,        // ← Auto-injected authenticated user
//!         db: DbContext,           // ← Auto-injected database context
//!         request: CreateOrderRequest // ← Auto-validated request
//!     ) -> Result<OrderResponse, String> {
//!         // PURE BUSINESS LOGIC ONLY!
//!         let order = Order::new(request, user.user_id);
//!         db.orders().save(&order).await?;
//!         Ok(OrderResponse::success(order))
//!     }
//! }
//! ```
//!
//! ### 🏥 Healthcare with HIPAA Compliance
//! ```rust,ignore
//! #[service_impl]
//! impl HealthcareService {
//!     #[service_method("POST /patients")]
//!     #[require_attributes(user.role = "Doctor" && user.department = resource.department)]
//!     #[validate_compliance(standards = ["hipaa", "gdpr"])]
//!     #[audit(include_user = true)]
//!     #[encrypt] // PHI data encryption
//!     pub async fn create_patient_record(
//!         user: AuthContext,
//!         request: PatientRequest // ← Auto-encrypted, HIPAA validated
//!     ) -> Result<PatientResponse, String> {
//!         // Healthcare business logic with automatic compliance
//!     }
//! }
//! ```
//!
//! ### 💰 Finance with Multi-Tier Approval Workflow
//! ```rust,ignore
//! #[service_impl]
//! impl FinanceService {
//!     #[service_method("POST /transfers")]
//!     #[require_role("Trader")]
//!     #[approval_required(
//!         threshold = "amount > 100000",
//!         approver_role = "Manager",
//!         auto_approve_if = "user.role = 'Director'"
//!     )]
//!     #[saga(compensate = "reverse_transfer")]
//!     #[rate_limit(requests = 10, per = "minute")]
//!     pub async fn large_transfer(
//!         user: AuthContext,
//!         request: TransferRequest
//!     ) -> Result<TransferResponse, String> {
//!         // Automatic approval workflow, saga coordination, rate limiting
//!     }
//! }
//! ```
//!
//! ### 🎮 Gaming with State Machine Workflow
//! ```rust,ignore
//! #[service_impl]
//! impl GamingService {
//!     #[service_method("POST /matches/:id/start")]
//!     #[require_auth]
//!     #[state_transition(from = "waiting", to = "in_progress")]
//!     #[event_sourced]
//!     #[metrics(histogram = "match_duration")]
//!     pub async fn start_match(
//!         user: AuthContext,
//!         match_data: MatchState, // ← Auto-loaded with state validation
//!         match_id: String
//!     ) -> Result<MatchResponse, String> {
//!         // State machine ensures valid transitions, events auto-stored
//!     }
//! }
//! ```
//!
//! **All This Generated Automatically:**
//! - ✅ HTTP routes + RabbitMQ RPC handlers
//! - ✅ RBAC/ABAC/Hybrid authorization for any domain
//! - ✅ Universal database transactions (SQL/NoSQL/Graph/etc.)
//! - ✅ Multi-level intelligent caching
//! - ✅ Comprehensive validation + security + compliance
//! - ✅ Complete observability (metrics, tracing, logging)
//! - ✅ Complex workflows (state machines, sagas, approvals)
//! - ✅ Domain-specific optimizations for 10+ industries
//!
//! ## 🎯 Supported Method Patterns
//!
//! The macros intelligently handle different method signatures:
//!
//! ```rust,ignore
//! # use rabbitmesh_macros::{service_definition, service_impl};
//! # use serde::{Deserialize, Serialize};
//! # #[derive(Deserialize)] pub struct CreateRequest { pub name: String }
//! # #[derive(Deserialize)] pub struct UpdateRequest { pub name: String }
//! # #[derive(Serialize)] pub struct Response { pub success: bool }
//! # #[service_definition] pub struct MyService;
//! #[service_impl]
//! impl MyService {
//!     // Simple path parameter
//!     #[service_method("GET /items/:id")]
//!     async fn get_item(id: String) -> Result<Response, String> {
//!         // id auto-extracted from URL
//!         Ok(Response { success: true })
//!     }
//!
//!     // Request body
//!     #[service_method("POST /items")]
//!     async fn create_item(request: CreateRequest) -> Result<Response, String> {
//!         // request auto-deserialized from JSON
//!         Ok(Response { success: true })
//!     }
//!
//!     // Path param + request body (tuple)
//!     #[service_method("PUT /items/:id")]
//!     async fn update_item(params: (String, UpdateRequest)) -> Result<Response, String> {
//!         let (id, request) = params;
//!         Ok(Response { success: true })
//!     }
//!
//!     // Multiple path parameters
//!     #[service_method("GET /users/:user_id/items/:item_id")]
//!     async fn get_user_item(params: (String, String)) -> Result<Response, String> {
//!         let (user_id, item_id) = params;
//!         Ok(Response { success: true })
//!     }
//! }
//! ```

extern crate proc_macro;

// Core service macros
mod service_definition;
mod service_method;
mod service_impl;
mod registry;

// Universal macro framework modules (not public in proc-macro crates)
mod universal;
mod authorization;
mod database;
mod caching;
mod validation;
mod observability;
mod workflow;

use proc_macro::TokenStream;

/// Marks a struct as a microservice definition.
/// 
/// This macro registers the service with the global service registry
/// and generates the necessary boilerplate for RPC handling.
#[proc_macro_attribute]
pub fn service_definition(args: TokenStream, input: TokenStream) -> TokenStream {
    service_definition::impl_service_definition(args, input)
}

/// Marks a method as a service endpoint with optional HTTP route information.
///
/// The route string is used by the API gateway to generate REST endpoints.
/// If no route is provided, only RPC calls will be supported.
///
/// ## Examples
///
/// ```rust,ignore
/// #[service_method("GET /users/:id")]
/// pub async fn get_user(user_id: u32) -> Result<User, String> { ... }
///
/// #[service_method("POST /users")]
/// pub async fn create_user(data: CreateUserRequest) -> Result<User, String> { ... }
///
/// #[service_method] // RPC only
/// pub async fn internal_cleanup() -> Result<(), String> { ... }
/// ```
#[proc_macro_attribute]
pub fn service_method(args: TokenStream, input: TokenStream) -> TokenStream {
    service_method::impl_service_method(args, input)
}

/// Processes an entire impl block and auto-generates RPC handler registration.
/// 
/// **This is the MOST POWERFUL macro** - it automatically discovers all #[service_method] 
/// functions and generates complete handler registration code plus universal macro processing.
///
/// ## Basic Service Example
/// 
/// ```rust,ignore
/// #[service_definition]
/// pub struct UserService;
///
/// #[service_impl]
/// impl UserService {
///     #[service_method("GET /users/:id")]
///     pub async fn get_user(user_id: u32) -> Result<User, String> { ... }
/// }
/// ```
///
/// ## Universal Macro Framework Example
/// 
/// ```rust,ignore
/// #[service_impl]
/// impl FinanceService {
///     #[service_method("POST /trades")]
///     #[require_role("Trader")]                    // RBAC authorization
///     #[require_attributes(amount <= user.limit)] // ABAC business rules
///     #[transactional(isolation = "serializable")] // Database transaction
///     #[cached(ttl = "30s", key = "trade_{symbol}")] // Intelligent caching
///     #[validate(schema = "trade_schema")]         // Input validation
///     #[rate_limit(requests = 100, per = "minute")] // Rate limiting
///     #[metrics(histogram = "trade_latency")]      // Observability
///     #[saga(compensate = "reverse_trade")]        // Distributed transactions
///     #[state_transition(from = "pending", to = "executed")] // State management
///     #[audit(compliance = ["sox", "mifid"])]      // Regulatory compliance
///     pub async fn execute_trade(
///         user: AuthContext,      // ← Auto-injected authenticated user
///         db: DbContext,          // ← Auto-injected database context  
///         cache: CacheContext,    // ← Auto-injected cache context
///         request: TradeRequest   // ← Auto-validated, rate-limited request
///     ) -> Result<TradeResponse, String> {
///         // PURE BUSINESS LOGIC - all cross-cutting concerns handled by macros!
///         let trade = Trade::new(request, user.trader_id);
///         db.trades().execute(&trade).await?;
///         Ok(TradeResponse::success(trade))
///     }
/// }
/// ```
///
/// **The macro framework supports 50+ different attributes across:**
/// - 🔐 **Authorization**: `#[require_auth]`, `#[require_role]`, `#[require_attributes]`, `#[require_ownership]`
/// - 💾 **Database**: `#[transactional]`, `#[read_only]`, `#[database]`, `#[auto_save]`, `#[versioned]`
/// - ⚡ **Caching**: `#[cached]`, `#[cache_invalidate]`, `#[cache_through]`, `#[multi_level_cache]`
/// - ✅ **Validation**: `#[validate]`, `#[sanitize]`, `#[validate_email]`, `#[validate_compliance]`
/// - 📊 **Observability**: `#[metrics]`, `#[trace]`, `#[log]`, `#[monitor]`, `#[alert]`, `#[health_check]`
/// - 🎭 **Workflows**: `#[saga]`, `#[state_machine]`, `#[approval_required]`, `#[event_sourced]`, `#[cqrs]`
/// - 🛡️ **Security**: `#[rate_limit]`, `#[encrypt]`, `#[audit_log]`, `#[pii_mask]`, `#[circuit_breaker]`
/// - 🔄 **Integration**: `#[webhook]`, `#[event_publish]`, `#[queue_message]`, `#[idempotent]`
///
/// Works across ALL domains: E-commerce, Finance, Healthcare, IoT, Gaming, Social Media, Logistics, and more!
#[proc_macro_attribute]
pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    service_impl::impl_service_impl(args, input)
}