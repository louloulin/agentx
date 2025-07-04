//! A2A Protocol Error Types
//! 
//! This module defines error types for the A2A protocol implementation.

use thiserror::Error;

/// A2A Protocol errors
#[derive(Error, Debug)]
pub enum A2AError {
    /// Message serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    
    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    /// Capability not found
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
    
    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    /// Authorization error
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    /// Network communication error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    /// Timeout error
    #[error("Operation timed out")]
    Timeout,
    
    /// Message expired
    #[error("Message has expired")]
    MessageExpired,
    
    /// Invalid endpoint
    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),
    
    /// Protocol version mismatch
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for A2A operations
pub type A2AResult<T> = Result<T, A2AError>;

impl A2AError {
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create a configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
    
    /// Create an invalid message error
    pub fn invalid_message(msg: impl Into<String>) -> Self {
        Self::InvalidMessage(msg.into())
    }
    
    /// Create an agent not found error
    pub fn agent_not_found(agent_id: impl Into<String>) -> Self {
        Self::AgentNotFound(agent_id.into())
    }
    
    /// Create a capability not found error
    pub fn capability_not_found(capability: impl Into<String>) -> Self {
        Self::CapabilityNotFound(capability.into())
    }
    
    /// Create an authentication error
    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }
    
    /// Create an authorization error
    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::Authorization(msg.into())
    }
    
    /// Create an invalid endpoint error
    pub fn invalid_endpoint(endpoint: impl Into<String>) -> Self {
        Self::InvalidEndpoint(endpoint.into())
    }
    
    /// Create a version mismatch error
    pub fn version_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
    
    /// Create a service unavailable error
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            A2AError::Network(_) |
            A2AError::Timeout |
            A2AError::RateLimitExceeded |
            A2AError::ServiceUnavailable(_)
        )
    }
    
    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            A2AError::Serialization(_) => "SERIALIZATION_ERROR",
            A2AError::InvalidMessage(_) => "INVALID_MESSAGE",
            A2AError::AgentNotFound(_) => "AGENT_NOT_FOUND",
            A2AError::CapabilityNotFound(_) => "CAPABILITY_NOT_FOUND",
            A2AError::Authentication(_) => "AUTHENTICATION_FAILED",
            A2AError::Authorization(_) => "AUTHORIZATION_FAILED",
            A2AError::Network(_) => "NETWORK_ERROR",
            A2AError::Timeout => "TIMEOUT",
            A2AError::MessageExpired => "MESSAGE_EXPIRED",
            A2AError::InvalidEndpoint(_) => "INVALID_ENDPOINT",
            A2AError::VersionMismatch { .. } => "VERSION_MISMATCH",
            A2AError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            A2AError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            A2AError::Internal(_) => "INTERNAL_ERROR",
            A2AError::Configuration(_) => "CONFIGURATION_ERROR",
            A2AError::Validation(_) => "VALIDATION_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = A2AError::agent_not_found("agent123");
        assert_eq!(error.to_string(), "Agent not found: agent123");
        assert_eq!(error.error_code(), "AGENT_NOT_FOUND");
        assert!(!error.is_retryable());
    }
    
    #[test]
    fn test_retryable_errors() {
        assert!(A2AError::Timeout.is_retryable());
        assert!(A2AError::RateLimitExceeded.is_retryable());
        assert!(!A2AError::AgentNotFound("test".to_string()).is_retryable());
    }
    
    #[test]
    fn test_version_mismatch() {
        let error = A2AError::version_mismatch("1.0", "2.0");
        assert!(error.to_string().contains("expected 1.0, got 2.0"));
    }
}
