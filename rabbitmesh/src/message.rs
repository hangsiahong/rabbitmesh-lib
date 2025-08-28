use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::error::{RabbitMeshError, Result};

/// Message types for service communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum MessageType {
    /// RPC request message
    Request,
    /// RPC response message  
    Response,
    /// One-way event message (fire and forget)
    Event,
    /// Health check ping
    Ping,
    /// Health check pong
    Pong,
}

/// Core message structure for all service communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: Uuid,
    /// Message type
    pub message_type: MessageType,
    /// Source service name
    pub from: String,
    /// Target service name (optional for broadcasts)
    pub to: Option<String>,
    /// Method name being called
    pub method: String,
    /// Message payload as JSON
    pub payload: serde_json::Value,
    /// Request correlation ID (for matching requests/responses)
    pub correlation_id: Option<Uuid>,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    /// Message metadata (headers, tracing info, etc.)
    pub metadata: HashMap<String, String>,
    /// Retry attempt number (0 for first attempt)
    pub retry_count: u32,
}

impl Message {
    /// Create a new RPC request message
    pub fn new_request(
        from: impl Into<String>,
        to: impl Into<String>,
        method: impl Into<String>,
        payload: impl Serialize,
    ) -> Result<Self> {
        let id = Uuid::new_v4();
        Ok(Self {
            id,
            message_type: MessageType::Request,
            from: from.into(),
            to: Some(to.into()),
            method: method.into(),
            payload: serde_json::to_value(payload)?,
            correlation_id: Some(id), // Use message ID as correlation ID for requests
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            retry_count: 0,
        })
    }

    /// Create a response message for a request
    pub fn new_response(
        request: &Message,
        from: impl Into<String>,
        payload: impl Serialize,
    ) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Response,
            from: from.into(),
            to: Some(request.from.clone()),
            method: request.method.clone(),
            payload: serde_json::to_value(payload)?,
            correlation_id: request.correlation_id,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            retry_count: 0,
        })
    }

    /// Create a one-way event message
    pub fn new_event(
        from: impl Into<String>,
        method: impl Into<String>,
        payload: impl Serialize,
    ) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Event,
            from: from.into(),
            to: None, // Events can be broadcast
            method: method.into(),
            payload: serde_json::to_value(payload)?,
            correlation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            retry_count: 0,
        })
    }

    /// Create a ping message for health checks
    pub fn new_ping(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Ping,
            from: from.into(),
            to: Some(to.into()),
            method: "ping".to_string(),
            payload: serde_json::Value::Null,
            correlation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            retry_count: 0,
        }
    }

    /// Create a pong response to a ping
    pub fn new_pong(ping: &Message, from: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Pong,
            from: from.into(),
            to: Some(ping.from.clone()),
            method: "pong".to_string(),
            payload: serde_json::Value::Null,
            correlation_id: ping.correlation_id,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            retry_count: 0,
        }
    }

    /// Serialize message to bytes for transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Deserialize payload to specific type
    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        Ok(serde_json::from_value(self.payload.clone())?)
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        matches!(self.message_type, MessageType::Request)
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        matches!(self.message_type, MessageType::Response)
    }

    /// Check if this is an event message
    pub fn is_event(&self) -> bool {
        matches!(self.message_type, MessageType::Event)
    }

    /// Create a retry version of this message
    pub fn with_retry(mut self) -> Self {
        self.retry_count += 1;
        self.timestamp = Utc::now();
        self
    }

    /// Get age of message in milliseconds
    pub fn age_ms(&self) -> i64 {
        (Utc::now() - self.timestamp).num_milliseconds()
    }
}

/// Simplified RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Service name to call
    pub service: String,
    /// Method name to call
    pub method: String,
    /// Request parameters
    pub params: serde_json::Value,
    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

impl RpcRequest {
    /// Create a new RPC request
    pub fn new(
        service: impl Into<String>,
        method: impl Into<String>,
        params: impl Serialize,
    ) -> Result<Self> {
        Ok(Self {
            service: service.into(),
            method: method.into(),
            params: serde_json::to_value(params)?,
            timeout_ms: None,
        })
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Convert to Message
    pub fn into_message(self, from: impl Into<String>) -> Result<Message> {
        Message::new_request(from, self.service, self.method, self.params)
    }
}

/// Simplified RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum RpcResponse {
    /// Successful response
    Success { 
        data: serde_json::Value,
        /// Processing time in milliseconds
        processing_time_ms: u64,
    },
    /// Error response
    Error { 
        error: String,
        /// Error code for structured error handling
        code: Option<String>,
        /// Additional error details
        details: Option<serde_json::Value>,
    },
}

impl RpcResponse {
    /// Create a successful response
    pub fn success(data: impl Serialize, processing_time_ms: u64) -> Result<Self> {
        Ok(Self::Success {
            data: serde_json::to_value(data)?,
            processing_time_ms,
        })
    }

    /// Create an error response
    pub fn error(error: impl Into<String>) -> Self {
        Self::Error {
            error: error.into(),
            code: None,
            details: None,
        }
    }

    /// Create an error response with code and details
    pub fn error_detailed(
        error: impl Into<String>,
        code: impl Into<String>,
        details: impl Serialize,
    ) -> Result<Self> {
        Ok(Self::Error {
            error: error.into(),
            code: Some(code.into()),
            details: Some(serde_json::to_value(details)?),
        })
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if response is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Extract success data
    pub fn data<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        match self {
            Self::Success { data, .. } => Ok(serde_json::from_value(data.clone())?),
            Self::Error { error, .. } => Err(RabbitMeshError::handler_error(error)),
        }
    }

    /// Convert to Message for transmission
    pub fn into_message(self, request: &Message, from: impl Into<String>) -> Result<Message> {
        Message::new_response(request, from, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message::new_request("service-a", "service-b", "get_user", 123).unwrap();
        let bytes = message.to_bytes().unwrap();
        let deserialized = Message::from_bytes(&bytes).unwrap();
        
        assert_eq!(message.id, deserialized.id);
        assert_eq!(message.message_type, deserialized.message_type);
        assert_eq!(message.from, deserialized.from);
        assert_eq!(message.to, deserialized.to);
        assert_eq!(message.method, deserialized.method);
    }

    #[test]
    fn test_rpc_response() {
        let success = RpcResponse::success("data", 42).unwrap();
        assert!(success.is_success());
        assert!(!success.is_error());

        let error = RpcResponse::error("Something went wrong");
        assert!(!error.is_success());
        assert!(error.is_error());
    }

    #[test]
    fn test_message_metadata() {
        let message = Message::new_request("a", "b", "method", ()).unwrap()
            .with_metadata("trace-id", "abc-123")
            .with_metadata("user-id", "user-456");

        assert_eq!(message.get_metadata("trace-id"), Some(&"abc-123".to_string()));
        assert_eq!(message.get_metadata("user-id"), Some(&"user-456".to_string()));
        assert_eq!(message.get_metadata("nonexistent"), None);
    }
}