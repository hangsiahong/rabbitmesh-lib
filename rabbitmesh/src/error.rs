use thiserror::Error;

/// Result type alias for RabbitMesh operations
pub type Result<T> = std::result::Result<T, RabbitMeshError>;

/// Comprehensive error types for RabbitMesh framework
#[derive(Error, Debug)]
pub enum RabbitMeshError {
    /// AMQP connection errors
    #[error("AMQP connection error: {0}")]
    Connection(#[from] lapin::Error),

    /// Message serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// RPC timeout errors
    #[error("RPC call timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Service not found errors
    #[error("Service '{service_name}' not found")]
    ServiceNotFound { service_name: String },

    /// Method not found errors
    #[error("Method '{method_name}' not found in service '{service_name}'")]
    MethodNotFound {
        service_name: String,
        method_name: String,
    },

    /// Invalid message format
    #[error("Invalid message format: {reason}")]
    InvalidMessage { reason: String },

    /// Service handler errors
    #[error("Service handler error: {0}")]
    Handler(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal framework errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Network I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic errors from anyhow
    #[error(transparent)]
    Other(#[from] anyhow::Error),

    /// Tokio join errors
    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

impl RabbitMeshError {
    /// Create a new handler error
    pub fn handler_error<T: ToString>(message: T) -> Self {
        Self::Handler(message.to_string())
    }

    /// Create a new config error
    pub fn config_error<T: ToString>(message: T) -> Self {
        Self::Config(message.to_string())
    }

    /// Create a new internal error
    pub fn internal_error<T: ToString>(message: T) -> Self {
        Self::Internal(message.to_string())
    }

    /// Check if error is recoverable (should be retried)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Connection(_) => true,
            Self::Timeout { .. } => true,
            Self::Io(_) => true,
            _ => false,
        }
    }
}