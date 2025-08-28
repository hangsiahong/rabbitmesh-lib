//! # RabbitMesh - Message-Driven Microservices Framework
//!
//! RabbitMesh provides a complete framework for building microservices that communicate
//! over RabbitMQ instead of HTTP. This eliminates port management, provides automatic
//! load balancing, and offers built-in fault tolerance.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rabbitmesh::MicroService;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let service = MicroService::new_simple("user-service", "amqp://localhost:5672").await?;
//!     
//!     service.register_function("get_user", |msg| async move {
//!         let user_id: u32 = msg.deserialize_payload()?;
//!         Ok(rabbitmesh::message::RpcResponse::success(
//!             format!("User {}", user_id), 42
//!         )?)
//!     }).await;
//!     
//!     service.start().await?;
//!     Ok(())
//! }
//! ```

pub mod connection;
pub mod message;
pub mod rpc;
pub mod service;
pub mod client;
pub mod error;

pub use connection::{ConnectionManager, ConnectionConfig};
pub use message::{Message, MessageType, RpcRequest, RpcResponse};
pub use rpc::{RpcHandler, RpcFramework};
pub use service::{MicroService, ServiceConfig, ServiceClient as ServiceClientFromService};

// Convenient alias for the main service
pub use service::MicroService as Service;
pub use client::ServiceClient;
pub use error::{RabbitMeshError, Result};