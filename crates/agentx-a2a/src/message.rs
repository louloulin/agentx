//! A2A Protocol Message Implementation
//!
//! This module implements the Agent2Agent (A2A) protocol message format
//! based on the official A2A protocol specification v0.2.5 from Google.
//!
//! The A2A protocol enables communication and interoperability between
//! independent AI agent systems using JSON-RPC 2.0 over HTTP(S).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A2A Message - Core message format following A2A protocol specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2AMessage {
    /// Message role: "user" or "agent"
    pub role: MessageRole,

    /// Message parts (content)
    pub parts: Vec<MessagePart>,

    /// Unique message identifier
    #[serde(rename = "messageId")]
    pub message_id: String,

    /// Task identifier (if part of a task)
    #[serde(rename = "taskId", skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// Context identifier (logical grouping)
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,

    /// Message metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A2A Task - Fundamental unit of work in A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2ATask {
    /// Unique task identifier
    pub id: String,

    /// Context identifier (logical grouping)
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,

    /// Task status
    pub status: TaskStatus,

    /// Task artifacts (outputs)
    #[serde(default)]
    pub artifacts: Vec<Artifact>,

    /// Message history
    #[serde(default)]
    pub history: Vec<A2AMessage>,

    /// Task kind identifier
    pub kind: String,

    /// Task metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Message role in A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum MessageRole {
    /// User message
    User,
    /// Agent message
    Agent,
}

/// Task status in A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatus {
    /// Current task state
    pub state: TaskState,

    /// Status timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    /// Status message (for input-required state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Task state enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TaskState {
    /// Task has been submitted
    Submitted,
    /// Task is being processed
    Working,
    /// Task requires user input
    InputRequired,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was canceled
    Canceled,
}

/// Message part - smallest unit of content in A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum MessagePart {
    /// Text content part
    #[serde(rename = "text")]
    Text(TextPart),

    /// File content part
    #[serde(rename = "file")]
    File(FilePart),

    /// Structured data part
    #[serde(rename = "data")]
    Data(DataPart),
}

/// Text part containing plain text content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextPart {
    /// Text content
    pub text: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// File part containing file data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilePart {
    /// File data
    pub file: FileData,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Data part containing structured data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataPart {
    /// Structured data
    pub data: serde_json::Value,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// File data in A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FileData {
    /// File with bytes (base64 encoded)
    WithBytes(FileWithBytes),
    /// File with URI reference
    WithUri(FileWithUri),
}

/// File with embedded bytes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileWithBytes {
    /// File name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: String,

    /// Base64 encoded file content
    #[serde(rename = "bytes")]
    pub bytes: String,
}

/// File with URI reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileWithUri {
    /// File name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: String,

    /// URI to file content
    pub uri: String,
}

/// Artifact - output generated by agent as result of task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    /// Unique artifact identifier
    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    /// Artifact name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Artifact parts (content)
    pub parts: Vec<MessagePart>,

    /// Artifact metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// JSON-RPC 2.0 Request structure for A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request method name
    pub method: String,

    /// Request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,

    /// Request identifier
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Response structure for A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Response result (success case)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Response error (error case)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request identifier
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// A2A protocol constants
pub const A2A_VERSION: &str = "0.2.5";
pub const JSON_RPC_VERSION: &str = "2.0";

/// A2A protocol method names
pub mod methods {
    pub const SUBMIT_TASK: &str = "submitTask";
    pub const GET_TASK: &str = "getTask";
    pub const CANCEL_TASK: &str = "cancelTask";
    pub const SEND_MESSAGE: &str = "sendMessage";
    pub const GET_CAPABILITIES: &str = "getCapabilities";
}

impl A2AMessage {
    /// Create a new A2A message with text content
    pub fn new_text(role: MessageRole, text: String) -> Self {
        Self {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text,
                metadata: HashMap::new(),
            })],
            message_id: Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a user message
    pub fn user_message(text: String) -> Self {
        Self::new_text(MessageRole::User, text)
    }

    /// Create an agent message
    pub fn agent_message(text: String) -> Self {
        Self::new_text(MessageRole::Agent, text)
    }

    /// Create a message with file content
    pub fn new_file(role: MessageRole, file_data: FileData) -> Self {
        Self {
            role,
            parts: vec![MessagePart::File(FilePart {
                file: file_data,
                metadata: HashMap::new(),
            })],
            message_id: Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a message with structured data
    pub fn new_data(role: MessageRole, data: serde_json::Value) -> Self {
        Self {
            role,
            parts: vec![MessagePart::Data(DataPart {
                data,
                metadata: HashMap::new(),
            })],
            message_id: Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set task ID for this message
    pub fn with_task_id(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Set context ID for this message
    pub fn with_context_id(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get text content from message parts
    pub fn get_text_content(&self) -> Option<String> {
        for part in &self.parts {
            if let MessagePart::Text(text_part) = part {
                return Some(text_part.text.clone());
            }
        }
        None
    }
}

impl A2ATask {
    /// Create a new A2A task
    pub fn new(kind: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Submitted,
                timestamp: Some(Utc::now()),
                message: None,
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            kind,
            metadata: HashMap::new(),
        }
    }

    /// Add a message to the task history
    pub fn add_message(mut self, message: A2AMessage) -> Self {
        self.history.push(message);
        self
    }

    /// Update task status
    pub fn update_status(mut self, state: TaskState) -> Self {
        self.status = TaskStatus {
            state,
            timestamp: Some(Utc::now()),
            message: None,
        };
        self
    }

    /// Add an artifact to the task
    pub fn add_artifact(mut self, artifact: Artifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    /// Set context ID for this task
    pub fn with_context_id(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: String, params: Option<serde_json::Value>, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            method,
            params,
            id,
        }
    }

    /// Create a submitTask request
    pub fn submit_task(task: A2ATask, id: serde_json::Value) -> Self {
        Self::new(
            methods::SUBMIT_TASK.to_string(),
            Some(serde_json::to_value(task).unwrap()),
            id,
        )
    }

    /// Create a sendMessage request
    pub fn send_message(message: A2AMessage, id: serde_json::Value) -> Self {
        Self::new(
            methods::SEND_MESSAGE.to_string(),
            Some(serde_json::to_value(message).unwrap()),
            id,
        )
    }

    /// Create a getTask request
    pub fn get_task(task_id: String, id: serde_json::Value) -> Self {
        Self::new(
            methods::GET_TASK.to_string(),
            Some(serde_json::json!({"taskId": task_id})),
            id,
        )
    }
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(result: serde_json::Value, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(error: JsonRpcError, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String, data: Option<serde_json::Value>) -> Self {
        Self { code, message, data }
    }

    /// Parse error (-32700)
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error".to_string(), None)
    }

    /// Invalid request (-32600)
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid Request".to_string(), None)
    }

    /// Method not found (-32601)
    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found".to_string(), None)
    }

    /// Invalid params (-32602)
    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params".to_string(), None)
    }

    /// Internal error (-32603)
    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error".to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_message() {
        let msg = A2AMessage::user_message("Hello, world!".to_string());

        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.parts.len(), 1);

        if let MessagePart::Text(text_part) = &msg.parts[0] {
            assert_eq!(text_part.text, "Hello, world!");
        } else {
            panic!("Expected text part");
        }
    }

    #[test]
    fn test_create_agent_message() {
        let msg = A2AMessage::agent_message("Hello from agent!".to_string());

        assert_eq!(msg.role, MessageRole::Agent);
        assert_eq!(msg.get_text_content(), Some("Hello from agent!".to_string()));
    }

    #[test]
    fn test_create_task() {
        let task = A2ATask::new("text_generation".to_string())
            .with_context_id("ctx_123".to_string());

        assert_eq!(task.kind, "text_generation");
        assert_eq!(task.context_id, Some("ctx_123".to_string()));
        assert_eq!(task.status.state, TaskState::Submitted);
    }

    #[test]
    fn test_json_rpc_request() {
        let task = A2ATask::new("test_task".to_string());
        let request = JsonRpcRequest::submit_task(task, serde_json::Value::String("req_1".to_string()));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "submitTask");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_json_rpc_response() {
        let response = JsonRpcResponse::success(
            serde_json::json!({"status": "ok"}),
            serde_json::Value::String("req_1".to_string())
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_message_serialization() {
        let msg = A2AMessage::user_message("Test message".to_string())
            .with_task_id("task_123".to_string())
            .with_context_id("ctx_456".to_string());

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg, deserialized);
    }
}

