//! 协议兼容层
//! 
//! 提供与MCP、OpenAI Assistant API等主流协议的兼容性支持

use agentx_a2a::{A2AMessage, A2AResult, A2AError, MessageRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// MCP (Model Context Protocol) 兼容层
pub mod mcp {
    use super::*;
    
    /// MCP消息格式
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpMessage {
        pub jsonrpc: String,
        pub id: Option<String>,
        pub method: String,
        pub params: Option<serde_json::Value>,
        pub result: Option<serde_json::Value>,
        pub error: Option<McpError>,
    }
    
    /// MCP错误格式
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpError {
        pub code: i32,
        pub message: String,
        pub data: Option<serde_json::Value>,
    }
    
    /// MCP工具定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpTool {
        pub name: String,
        pub description: String,
        pub input_schema: serde_json::Value,
    }
    
    /// MCP资源定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct McpResource {
        pub uri: String,
        pub name: String,
        pub description: Option<String>,
        pub mime_type: Option<String>,
    }
    
    /// MCP协议适配器
    pub struct McpAdapter {
        tools: HashMap<String, McpTool>,
        resources: HashMap<String, McpResource>,
    }
    
    impl McpAdapter {
        pub fn new() -> Self {
            Self {
                tools: HashMap::new(),
                resources: HashMap::new(),
            }
        }
        
        /// 注册MCP工具
        pub fn register_tool(&mut self, tool: McpTool) {
            self.tools.insert(tool.name.clone(), tool);
        }
        
        /// 注册MCP资源
        pub fn register_resource(&mut self, resource: McpResource) {
            self.resources.insert(resource.uri.clone(), resource);
        }
        
        /// 将MCP消息转换为A2A消息
        pub fn mcp_to_a2a(&self, mcp_msg: McpMessage) -> A2AResult<A2AMessage> {
            let mut a2a_msg = match mcp_msg.method.as_str() {
                "tools/call" => {
                    let content = format!("MCP工具调用: {}", 
                        mcp_msg.params.unwrap_or_default().to_string());
                    A2AMessage::user_message(content)
                },
                "resources/read" => {
                    let content = format!("MCP资源读取: {}", 
                        mcp_msg.params.unwrap_or_default().to_string());
                    A2AMessage::user_message(content)
                },
                "completion/complete" => {
                    let content = format!("MCP补全请求: {}", 
                        mcp_msg.params.unwrap_or_default().to_string());
                    A2AMessage::user_message(content)
                },
                _ => {
                    let content = format!("MCP通用消息: {}", mcp_msg.method);
                    A2AMessage::new_text(MessageRole::Agent, content)
                }
            };
            
            // 添加MCP特定的元数据
            a2a_msg.metadata.insert("mcp_method".to_string(), 
                serde_json::Value::String(mcp_msg.method));
            if let Some(id) = mcp_msg.id {
                a2a_msg.metadata.insert("mcp_id".to_string(), 
                    serde_json::Value::String(id));
            }
            a2a_msg.metadata.insert("protocol".to_string(), 
                serde_json::Value::String("mcp".to_string()));
            
            Ok(a2a_msg)
        }
        
        /// 将A2A消息转换为MCP消息
        pub fn a2a_to_mcp(&self, a2a_msg: &A2AMessage) -> A2AResult<McpMessage> {
            let method = a2a_msg.metadata.get("mcp_method")
                .and_then(|v| v.as_str())
                .unwrap_or("notification")
                .to_string();
            
            let id = a2a_msg.metadata.get("mcp_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // 提取消息内容
            let content = if let Some(text_part) = a2a_msg.parts.first() {
                match text_part {
                    agentx_a2a::MessagePart::Text(text) => text.text.clone(),
                    _ => "非文本消息".to_string(),
                }
            } else {
                "空消息".to_string()
            };
            
            let mcp_msg = McpMessage {
                jsonrpc: "2.0".to_string(),
                id,
                method,
                params: Some(serde_json::json!({
                    "content": content,
                    "metadata": a2a_msg.metadata
                })),
                result: None,
                error: None,
            };
            
            Ok(mcp_msg)
        }
        
        /// 列出可用工具
        pub fn list_tools(&self) -> Vec<&McpTool> {
            self.tools.values().collect()
        }
        
        /// 列出可用资源
        pub fn list_resources(&self) -> Vec<&McpResource> {
            self.resources.values().collect()
        }
    }
    
    impl Default for McpAdapter {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// OpenAI Assistant API 兼容层
pub mod openai {
    use super::*;
    
    /// OpenAI Assistant消息格式
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenAIMessage {
        pub role: String,
        pub content: String,
        pub name: Option<String>,
        pub function_call: Option<FunctionCall>,
        pub tool_calls: Option<Vec<ToolCall>>,
    }
    
    /// 函数调用
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FunctionCall {
        pub name: String,
        pub arguments: String,
    }
    
    /// 工具调用
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ToolCall {
        pub id: String,
        pub r#type: String,
        pub function: FunctionCall,
    }
    
    /// OpenAI Assistant定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenAIAssistant {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub model: String,
        pub instructions: Option<String>,
        pub tools: Vec<OpenAITool>,
    }
    
    /// OpenAI工具定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenAITool {
        pub r#type: String,
        pub function: OpenAIFunction,
    }
    
    /// OpenAI函数定义
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OpenAIFunction {
        pub name: String,
        pub description: String,
        pub parameters: serde_json::Value,
    }
    
    /// OpenAI协议适配器
    pub struct OpenAIAdapter {
        assistants: HashMap<String, OpenAIAssistant>,
    }
    
    impl OpenAIAdapter {
        pub fn new() -> Self {
            Self {
                assistants: HashMap::new(),
            }
        }
        
        /// 注册Assistant
        pub fn register_assistant(&mut self, assistant: OpenAIAssistant) {
            self.assistants.insert(assistant.id.clone(), assistant);
        }
        
        /// 将OpenAI消息转换为A2A消息
        pub fn openai_to_a2a(&self, openai_msg: OpenAIMessage) -> A2AResult<A2AMessage> {
            let role = match openai_msg.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Agent,
                "system" => MessageRole::Agent, // 将system映射为Agent
                _ => MessageRole::User,
            };
            
            let mut a2a_msg = A2AMessage::new_text(role, openai_msg.content);
            
            // 添加OpenAI特定的元数据
            a2a_msg.metadata.insert("protocol".to_string(), 
                serde_json::Value::String("openai".to_string()));
            a2a_msg.metadata.insert("original_role".to_string(), 
                serde_json::Value::String(openai_msg.role));
            
            if let Some(name) = openai_msg.name {
                a2a_msg.metadata.insert("name".to_string(), 
                    serde_json::Value::String(name));
            }
            
            if let Some(function_call) = openai_msg.function_call {
                a2a_msg.metadata.insert("function_call".to_string(), 
                    serde_json::to_value(function_call)?);
            }
            
            if let Some(tool_calls) = openai_msg.tool_calls {
                a2a_msg.metadata.insert("tool_calls".to_string(), 
                    serde_json::to_value(tool_calls)?);
            }
            
            Ok(a2a_msg)
        }
        
        /// 将A2A消息转换为OpenAI消息
        pub fn a2a_to_openai(&self, a2a_msg: &A2AMessage) -> A2AResult<OpenAIMessage> {
            let role = match a2a_msg.role {
                MessageRole::User => "user",
                MessageRole::Agent => "assistant",
            }.to_string();

            let content = if let Some(text_part) = a2a_msg.parts.first() {
                match text_part {
                    agentx_a2a::MessagePart::Text(text) => text.text.clone(),
                    _ => "非文本消息".to_string(),
                }
            } else {
                "空消息".to_string()
            };
            
            let name = a2a_msg.metadata.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            let function_call = a2a_msg.metadata.get("function_call")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            let tool_calls = a2a_msg.metadata.get("tool_calls")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            let openai_msg = OpenAIMessage {
                role,
                content,
                name,
                function_call,
                tool_calls,
            };
            
            Ok(openai_msg)
        }
        
        /// 列出可用的Assistant
        pub fn list_assistants(&self) -> Vec<&OpenAIAssistant> {
            self.assistants.values().collect()
        }
    }
    
    impl Default for OpenAIAdapter {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// 协议兼容管理器
pub struct ProtocolCompatManager {
    mcp_adapter: mcp::McpAdapter,
    openai_adapter: openai::OpenAIAdapter,
}

impl ProtocolCompatManager {
    pub fn new() -> Self {
        Self {
            mcp_adapter: mcp::McpAdapter::new(),
            openai_adapter: openai::OpenAIAdapter::new(),
        }
    }
    
    /// 获取MCP适配器
    pub fn mcp(&mut self) -> &mut mcp::McpAdapter {
        &mut self.mcp_adapter
    }
    
    /// 获取OpenAI适配器
    pub fn openai(&mut self) -> &mut openai::OpenAIAdapter {
        &mut self.openai_adapter
    }
    
    /// 自动检测协议类型并转换为A2A消息
    pub async fn auto_convert_to_a2a(&self, data: serde_json::Value) -> A2AResult<A2AMessage> {
        // 尝试检测协议类型
        if data.get("jsonrpc").is_some() {
            // 可能是MCP消息
            let mcp_msg: mcp::McpMessage = serde_json::from_value(data)?;
            self.mcp_adapter.mcp_to_a2a(mcp_msg)
        } else if data.get("role").is_some() && data.get("content").is_some() {
            // 可能是OpenAI消息
            let openai_msg: openai::OpenAIMessage = serde_json::from_value(data)?;
            self.openai_adapter.openai_to_a2a(openai_msg)
        } else {
            // 默认作为A2A消息处理
            let a2a_msg: A2AMessage = serde_json::from_value(data)?;
            Ok(a2a_msg)
        }
    }
}

impl Default for ProtocolCompatManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_adapter() {
        let mut adapter = mcp::McpAdapter::new();
        
        // 注册工具
        let tool = mcp::McpTool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };
        adapter.register_tool(tool);
        
        // 测试MCP到A2A转换
        let mcp_msg = mcp::McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some("1".to_string()),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({"tool": "test_tool"})),
            result: None,
            error: None,
        };
        
        let a2a_msg = adapter.mcp_to_a2a(mcp_msg).unwrap();
        assert_eq!(a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap(), "mcp");
        
        // 测试A2A到MCP转换
        let mcp_result = adapter.a2a_to_mcp(&a2a_msg).unwrap();
        assert_eq!(mcp_result.jsonrpc, "2.0");
    }
    
    #[tokio::test]
    async fn test_openai_adapter() {
        let mut adapter = openai::OpenAIAdapter::new();
        
        // 注册Assistant
        let assistant = openai::OpenAIAssistant {
            id: "asst_123".to_string(),
            name: "Test Assistant".to_string(),
            description: Some("Test assistant".to_string()),
            model: "gpt-4".to_string(),
            instructions: Some("You are a helpful assistant".to_string()),
            tools: vec![],
        };
        adapter.register_assistant(assistant);
        
        // 测试OpenAI到A2A转换
        let openai_msg = openai::OpenAIMessage {
            role: "user".to_string(),
            content: "Hello, assistant!".to_string(),
            name: None,
            function_call: None,
            tool_calls: None,
        };
        
        let a2a_msg = adapter.openai_to_a2a(openai_msg).unwrap();
        assert_eq!(a2a_msg.role, MessageRole::User);
        assert_eq!(a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap(), "openai");
        
        // 测试A2A到OpenAI转换
        let openai_result = adapter.a2a_to_openai(&a2a_msg).unwrap();
        assert_eq!(openai_result.role, "user");
        assert_eq!(openai_result.content, "Hello, assistant!");
    }
    
    #[tokio::test]
    async fn test_protocol_compat_manager() {
        let manager = ProtocolCompatManager::new();
        
        // 测试MCP消息自动检测
        let mcp_data = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "tools/call",
            "params": {"tool": "test"}
        });
        
        let result = manager.auto_convert_to_a2a(mcp_data).await.unwrap();
        assert_eq!(result.metadata.get("protocol").unwrap().as_str().unwrap(), "mcp");
        
        // 测试OpenAI消息自动检测
        let openai_data = serde_json::json!({
            "role": "user",
            "content": "Hello!"
        });
        
        let result = manager.auto_convert_to_a2a(openai_data).await.unwrap();
        assert_eq!(result.metadata.get("protocol").unwrap().as_str().unwrap(), "openai");
    }
}
