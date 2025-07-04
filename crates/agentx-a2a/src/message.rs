//! A2A Message Format Implementation
//! 
//! This module implements the core A2A message format based on the official
//! A2A protocol specification from Google.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A2A Message - Core message format for agent-to-agent communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2AMessage {
    /// Unique message identifier
    pub id: String,
    
    /// Source agent identifier
    pub from: String,
    
    /// Target agent identifier  
    pub to: String,
    
    /// Message type/intent
    #[serde(rename = "type")]
    pub message_type: MessageType,
    
    /// Message payload
    pub payload: MessagePayload,
    
    /// Message metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Message timestamp (ISO 8601)
    pub timestamp: DateTime<Utc>,
    
    /// Conversation/session identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    
    /// A2A protocol version
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Message priority (0-10, higher is more urgent)
    #[serde(default)]
    pub priority: u8,
    
    /// Message expiration time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Message types supported by A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Request for action or information
    Request,
    
    /// Response to a request
    Response,
    
    /// Notification/event message
    Notification,
    
    /// Task delegation
    Delegation,
    
    /// Capability query
    CapabilityQuery,
    
    /// Capability response
    CapabilityResponse,
    
    /// Error message
    Error,
    
    /// Heartbeat/ping message
    Heartbeat,
}

/// Message payload containing the actual content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    /// Text content
    #[serde(rename = "text")]
    Text(TextPayload),
    
    /// Structured data
    #[serde(rename = "structured")]
    Structured(StructuredPayload),
    
    /// Tool/function call
    #[serde(rename = "tool_call")]
    ToolCall(ToolCallPayload),
    
    /// Tool/function result
    #[serde(rename = "tool_result")]
    ToolResult(ToolResultPayload),
    
    /// Capability information
    #[serde(rename = "capability")]
    Capability(CapabilityPayload),
    
    /// Error information
    #[serde(rename = "error")]
    Error(ErrorPayload),
    
    /// Binary data (base64 encoded)
    #[serde(rename = "binary")]
    Binary(BinaryPayload),
}

/// Text message payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPayload {
    /// Text content
    pub content: String,
    
    /// Content format (plain, markdown, html)
    #[serde(default = "default_format")]
    pub format: String,
    
    /// Language code (ISO 639-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Structured data payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredPayload {
    /// Schema identifier
    pub schema: String,
    
    /// Structured data
    pub data: serde_json::Value,
}

/// Tool/function call payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallPayload {
    /// Tool/function name
    pub name: String,
    
    /// Call identifier for correlation
    pub call_id: String,
    
    /// Function parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Tool/function result payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResultPayload {
    /// Call identifier for correlation
    pub call_id: String,
    
    /// Execution result
    pub result: serde_json::Value,
    
    /// Success indicator
    pub success: bool,
    
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Capability information payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityPayload {
    /// Capability name
    pub name: String,
    
    /// Capability description
    pub description: String,
    
    /// Input schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    
    /// Output schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Error information payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorPayload {
    /// Error code
    pub code: String,
    
    /// Error message
    pub message: String,
    
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Binary data payload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BinaryPayload {
    /// MIME type
    pub mime_type: String,
    
    /// Base64 encoded data
    pub data: String,
    
    /// Original filename
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl A2AMessage {
    /// Create a new A2A message
    pub fn new(
        from: String,
        to: String,
        message_type: MessageType,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            message_type,
            payload,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            conversation_id: None,
            version: A2A_VERSION.to_string(),
            priority: 5,
            expires_at: None,
        }
    }
    
    /// Create a text message
    pub fn text(from: String, to: String, content: String) -> Self {
        Self::new(
            from,
            to,
            MessageType::Request,
            MessagePayload::Text(TextPayload {
                content,
                format: "plain".to_string(),
                language: None,
            }),
        )
    }
    
    /// Create a response message
    pub fn response(original: &A2AMessage, payload: MessagePayload) -> Self {
        let mut response = Self::new(
            original.to.clone(),
            original.from.clone(),
            MessageType::Response,
            payload,
        );
        response.conversation_id = original.conversation_id.clone();
        response
    }
    
    /// Create an error response
    pub fn error_response(original: &A2AMessage, code: String, message: String) -> Self {
        Self::response(
            original,
            MessagePayload::Error(ErrorPayload {
                code,
                message,
                details: None,
            }),
        )
    }
    
    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Set conversation ID
    pub fn with_conversation_id(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_format() -> String {
    "plain".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_text_message() {
        let msg = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Hello, world!".to_string(),
        );
        
        assert_eq!(msg.from, "agent1");
        assert_eq!(msg.to, "agent2");
        assert_eq!(msg.message_type, MessageType::Request);
        
        if let MessagePayload::Text(text) = &msg.payload {
            assert_eq!(text.content, "Hello, world!");
            assert_eq!(text.format, "plain");
        } else {
            panic!("Expected text payload");
        }
    }
    
    #[test]
    fn test_create_response() {
        let original = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Hello".to_string(),
        ).with_conversation_id("conv123".to_string());
        
        let response = A2AMessage::response(
            &original,
            MessagePayload::Text(TextPayload {
                content: "Hi there!".to_string(),
                format: "plain".to_string(),
                language: None,
            }),
        );
        
        assert_eq!(response.from, "agent2");
        assert_eq!(response.to, "agent1");
        assert_eq!(response.message_type, MessageType::Response);
        assert_eq!(response.conversation_id, Some("conv123".to_string()));
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Test message".to_string(),
        );
        
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(msg, deserialized);
    }
    
    #[test]
    fn test_message_expiration() {
        let mut msg = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Test".to_string(),
        );
        
        // Not expired by default
        assert!(!msg.is_expired());
        
        // Set expiration in the past
        msg.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(msg.is_expired());
        
        // Set expiration in the future
        msg.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(!msg.is_expired());
    }
}
