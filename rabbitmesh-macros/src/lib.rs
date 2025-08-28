//! # RabbitMesh Macros - The Magic Behind Zero-Port Microservices ✨
//!
//! **Procedural macros that transform simple Rust structs into complete microservices with automatic route generation and RabbitMQ RPC integration.**
//!
//! This crate provides the three core macros that make RabbitMesh's zero-configuration philosophy possible:
//! - `#[service_definition]` - Marks a struct as a RabbitMesh service  
//! - `#[service_impl]` - Processes impl blocks to auto-register methods
//! - `#[service_method]` - Defines HTTP routes and RPC handlers
//!
//! ## 🪄 The Magic
//!
//! Write this simple code:
//!
//! ```rust,ignore
//! use rabbitmesh_macros::{service_definition, service_impl};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! pub struct CreateUserRequest {
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! #[derive(Serialize)]
//! pub struct UserResponse {
//!     pub success: bool,
//!     pub message: String,
//!     pub user_id: Option<String>,
//! }
//!
//! #[service_definition]
//! pub struct UserService;
//!
//! #[service_impl]
//! impl UserService {
//!     #[service_method("POST /users")]
//!     pub async fn create_user(request: CreateUserRequest) -> Result<UserResponse, String> {
//!         // JUST YOUR BUSINESS LOGIC!
//!         println!("Creating user: {}", request.name);
//!         Ok(UserResponse {
//!             success: true,
//!             message: "User created successfully".to_string(),
//!             user_id: Some("user123".to_string()),
//!         })
//!     }
//!
//!     #[service_method("GET /users/:id")]
//!     pub async fn get_user(user_id: String) -> Result<UserResponse, String> {
//!         // Framework auto-extracts user_id from URL path
//!         println!("Getting user: {}", user_id);
//!         Ok(UserResponse {
//!             success: true,
//!             message: format!("Retrieved user {}", user_id),
//!             user_id: Some(user_id),
//!         })
//!     }
//! }
//! ```
//!
//! **And get all this automatically generated:**
//! - ✅ HTTP routes: `POST /users`, `GET /users/:id`
//! - ✅ RabbitMQ RPC handlers: `rabbitmesh.UserService.create_user`, `rabbitmesh.UserService.get_user`
//! - ✅ Service registration methods: `UserService::create_service()`, `UserService::service_name()`
//! - ✅ Route discovery: `UserService::get_routes()`
//! - ✅ JSON serialization/deserialization 
//! - ✅ API Gateway integration
//! - ✅ Service discovery
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

mod service_definition;
mod service_method;
mod service_impl;
mod registry;

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
/// This is the advanced version that automatically discovers all #[service_method] 
/// functions and generates the complete handler registration code.
///
/// ## Example
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
#[proc_macro_attribute]
pub fn service_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    service_impl::impl_service_impl(args, input)
}