//! # RabbitMesh Macros
//!
//! Procedural macros for auto-generating microservice definitions and API endpoints.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rabbitmesh_macros::{service_definition, service_method};
//!
//! #[service_definition]
//! pub struct UserService;
//!
//! impl UserService {
//!     #[service_method("GET /users/:id")]
//!     pub async fn get_user(user_id: u32) -> Result<User, String> {
//!         // Business logic
//!         Ok(User { id: user_id, name: "John".to_string() })
//!     }
//! }
//! ```

extern crate proc_macro;

mod service_definition;
mod service_method;
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