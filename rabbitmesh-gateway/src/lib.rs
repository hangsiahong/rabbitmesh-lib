//! # RabbitMesh Gateway
//!
//! Auto-generating API gateway that exposes microservices as REST and GraphQL APIs.
//! The gateway communicates with microservices via RabbitMQ, not HTTP ports.
//!
//! ## How It Works
//!
//! ```
//! Frontend/Client -> HTTP -> Gateway -> RabbitMQ -> Microservice
//!                            ↑                        ↓
//!                         Converts                Processes  
//!                      HTTP to AMQP              business logic
//!                            ↓                        ↑
//!                    HTTP Response <- RabbitMQ <- RPC Response
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use rabbitmesh_gateway::create_auto_router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = create_auto_router("amqp://localhost:5672").await.unwrap();
//!     
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

pub mod router;
pub mod registry;

pub use router::{create_auto_router, GatewayState};
pub use registry::{ServiceRegistry, ServiceInfo, MethodInfo, ParameterInfo};