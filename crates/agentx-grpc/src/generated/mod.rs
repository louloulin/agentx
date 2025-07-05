//! 生成的gRPC代码模块
//! 
//! 包含从protobuf定义生成的Rust代码

// 由于这些文件是在构建时生成的，我们需要包含它们
// 实际的生成代码将在构建时创建

pub mod agentx {
    pub mod a2a {
        pub mod v1 {
            // A2A服务相关的生成代码将在这里
            // 这些将由tonic-build在构建时生成
            
            // 临时的类型定义，实际的将被生成的代码替换
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct A2AMessage {
                pub message_id: String,
                pub conversation_id: String,
                pub role: i32,
                pub parts: Vec<MessagePart>,
                pub metadata: std::collections::HashMap<String, serde_json::Value>,
                pub timestamp: chrono::DateTime<chrono::Utc>,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct MessagePart {
                pub content: MessagePartContent,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub enum MessagePartContent {
                Text(String),
                File(FileData),
                Data(serde_json::Value),
                ToolCall(ToolCall),
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct FileData {
                pub name: Option<String>,
                pub mime_type: String,
                pub data: FileDataContent,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub enum FileDataContent {
                Bytes(String), // base64编码
                Uri(String),
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct ToolCall {
                pub tool_call_id: String,
                pub function_name: String,
                pub arguments: serde_json::Value,
            }
        }
    }
    
    pub mod plugin {
        pub mod v1 {
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct PluginInfo {
                pub id: String,
                pub name: String,
                pub version: String,
                pub description: String,
                pub plugin_type: i32,
                pub status: i32,
                pub capabilities: Vec<String>,
                pub supported_frameworks: Vec<String>,
                pub metadata: std::collections::HashMap<String, serde_json::Value>,
                pub created_at: chrono::DateTime<chrono::Utc>,
                pub updated_at: chrono::DateTime<chrono::Utc>,
            }
        }
    }
    
    pub mod registry {
        pub mod v1 {
            use serde::{Deserialize, Serialize};
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct AgentCard {
                pub id: String,
                pub name: String,
                pub description: String,
                pub version: String,
                pub capabilities: Vec<Capability>,
                pub endpoints: Vec<Endpoint>,
                pub metadata: std::collections::HashMap<String, serde_json::Value>,
                pub created_at: chrono::DateTime<chrono::Utc>,
                pub updated_at: chrono::DateTime<chrono::Utc>,
                pub status: i32,
                pub trust_level: i32,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct Capability {
                pub name: String,
                pub description: String,
                pub capability_type: i32,
                pub available: bool,
                pub input_schema: Option<serde_json::Value>,
                pub output_schema: Option<serde_json::Value>,
                pub metadata: std::collections::HashMap<String, serde_json::Value>,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct Endpoint {
                pub endpoint_type: String,
                pub url: String,
                pub protocol: Option<String>,
                pub auth: Option<AuthInfo>,
                pub metadata: std::collections::HashMap<String, serde_json::Value>,
            }
            
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct AuthInfo {
                pub auth_type: String,
                pub parameters: std::collections::HashMap<String, serde_json::Value>,
            }
        }
    }
}
