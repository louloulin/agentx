//! 消息格式转换器
//! 
//! 提供不同AI框架间的消息格式转换功能

use crate::framework::FrameworkType;
use agentx_a2a::{A2AMessage, A2AResult, A2AError, MessageRole, MessagePart, TextPart};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, warn};

/// 消息转换器
pub struct MessageConverter {
    /// 转换规则缓存
    conversion_rules: HashMap<(FrameworkType, FrameworkType), ConversionRule>,
    /// 转换统计
    stats: ConversionStats,
}

/// 转换规则
#[derive(Debug, Clone)]
pub struct ConversionRule {
    /// 源框架类型
    pub source_framework: FrameworkType,
    /// 目标框架类型
    pub target_framework: FrameworkType,
    /// 字段映射规则
    pub field_mappings: HashMap<String, String>,
    /// 自定义转换函数
    pub custom_converter: Option<fn(&Value) -> A2AResult<Value>>,
}

/// 转换统计信息
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// 总转换次数
    pub total_conversions: u64,
    /// 成功转换次数
    pub successful_conversions: u64,
    /// 失败转换次数
    pub failed_conversions: u64,
    /// 按框架类型分组的转换次数
    pub conversions_by_framework: HashMap<FrameworkType, u64>,
}

/// LangChain消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangChainMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub additional_kwargs: HashMap<String, Value>,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

/// AutoGen消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoGenMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

/// Mastra消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MastraMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub context: HashMap<String, Value>,
    #[serde(default)]
    pub tools: Vec<String>,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolFunction,
    #[serde(rename = "type")]
    pub call_type: String,
}

/// 工具函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

impl MessageConverter {
    /// 创建新的消息转换器
    pub fn new() -> Self {
        let mut converter = Self {
            conversion_rules: HashMap::new(),
            stats: ConversionStats::default(),
        };
        
        // 初始化默认转换规则
        converter.initialize_default_rules();
        converter
    }

    /// 将A2A消息转换为指定框架格式
    pub fn convert_from_a2a(
        &mut self,
        message: &A2AMessage,
        target_framework: FrameworkType,
    ) -> A2AResult<Value> {
        debug!("转换A2A消息到 {:?} 格式", target_framework);
        
        self.stats.total_conversions += 1;
        *self.stats.conversions_by_framework.entry(target_framework.clone()).or_insert(0) += 1;

        let result = match target_framework {
            FrameworkType::LangChain => self.convert_to_langchain(message),
            FrameworkType::AutoGen => self.convert_to_autogen(message),
            FrameworkType::Mastra => self.convert_to_mastra(message),
            FrameworkType::CrewAI => self.convert_to_crewai(message),
            FrameworkType::SemanticKernel => self.convert_to_semantic_kernel(message),
            FrameworkType::LangGraph => self.convert_to_langgraph(message),
            FrameworkType::Custom(ref name) => self.convert_to_custom(message, name),
        };

        match result {
            Ok(value) => {
                self.stats.successful_conversions += 1;
                Ok(value)
            }
            Err(e) => {
                self.stats.failed_conversions += 1;
                Err(e)
            }
        }
    }

    /// 将框架消息转换为A2A格式
    pub fn convert_to_a2a(
        &mut self,
        message: Value,
        source_framework: FrameworkType,
    ) -> A2AResult<A2AMessage> {
        debug!("转换 {:?} 消息到A2A格式", source_framework);
        
        self.stats.total_conversions += 1;
        *self.stats.conversions_by_framework.entry(source_framework.clone()).or_insert(0) += 1;

        let result = match source_framework {
            FrameworkType::LangChain => self.convert_from_langchain(message),
            FrameworkType::AutoGen => self.convert_from_autogen(message),
            FrameworkType::Mastra => self.convert_from_mastra(message),
            FrameworkType::CrewAI => self.convert_from_crewai(message),
            FrameworkType::SemanticKernel => self.convert_from_semantic_kernel(message),
            FrameworkType::LangGraph => self.convert_from_langgraph(message),
            FrameworkType::Custom(ref name) => self.convert_from_custom(message, name),
        };

        match result {
            Ok(msg) => {
                self.stats.successful_conversions += 1;
                Ok(msg)
            }
            Err(e) => {
                self.stats.failed_conversions += 1;
                Err(e)
            }
        }
    }

    /// 框架间直接转换
    pub fn convert_between_frameworks(
        &mut self,
        message: Value,
        source_framework: FrameworkType,
        target_framework: FrameworkType,
    ) -> A2AResult<Value> {
        debug!("直接转换 {:?} 到 {:?}", source_framework, target_framework);
        
        // 先转换为A2A格式，再转换为目标格式
        let a2a_message = self.convert_to_a2a(message, source_framework)?;
        self.convert_from_a2a(&a2a_message, target_framework)
    }

    /// 获取转换统计信息
    pub fn get_stats(&self) -> &ConversionStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = ConversionStats::default();
    }

    // 私有方法 - 转换为各框架格式

    fn convert_to_langchain(&self, message: &A2AMessage) -> A2AResult<Value> {
        let role = match message.role {
            MessageRole::User => "human",
            MessageRole::Agent => "assistant",
        };

        // 提取文本内容
        let content = self.extract_text_content(message);

        let langchain_msg = LangChainMessage {
            role: role.to_string(),
            content,
            additional_kwargs: message.metadata.clone(),
            tool_calls: vec![], // TODO: 转换工具调用
        };

        serde_json::to_value(langchain_msg)
            .map_err(|e| A2AError::internal(format!("LangChain消息序列化失败: {}", e)))
    }

    fn convert_to_autogen(&self, message: &A2AMessage) -> A2AResult<Value> {
        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Agent => "assistant",
        };

        let content = self.extract_text_content(message);

        let autogen_msg = AutoGenMessage {
            role: role.to_string(),
            content,
            name: message.metadata.get("agent_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            metadata: message.metadata.clone(),
        };

        serde_json::to_value(autogen_msg)
            .map_err(|e| A2AError::internal(format!("AutoGen消息序列化失败: {}", e)))
    }

    fn convert_to_mastra(&self, message: &A2AMessage) -> A2AResult<Value> {
        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Agent => "assistant",
        };

        let content = self.extract_text_content(message);

        let mastra_msg = MastraMessage {
            role: role.to_string(),
            content,
            timestamp: chrono::Utc::now().to_rfc3339(), // A2A消息没有timestamp字段
            context: message.metadata.clone(),
            tools: vec![], // TODO: 提取工具信息
        };

        serde_json::to_value(mastra_msg)
            .map_err(|e| A2AError::internal(format!("Mastra消息序列化失败: {}", e)))
    }

    fn convert_to_crewai(&self, message: &A2AMessage) -> A2AResult<Value> {
        // CrewAI使用类似LangChain的格式
        self.convert_to_langchain(message)
    }

    fn convert_to_semantic_kernel(&self, message: &A2AMessage) -> A2AResult<Value> {
        let role = match message.role {
            MessageRole::User => "user",
            MessageRole::Agent => "assistant",
        };

        let content = self.extract_text_content(message);

        Ok(json!({
            "role": role,
            "content": content,
            "metadata": message.metadata
        }))
    }

    fn convert_to_langgraph(&self, message: &A2AMessage) -> A2AResult<Value> {
        // LangGraph使用LangChain兼容格式
        self.convert_to_langchain(message)
    }

    fn convert_to_custom(&self, message: &A2AMessage, framework_name: &str) -> A2AResult<Value> {
        warn!("使用通用转换器处理自定义框架: {}", framework_name);

        let content = self.extract_text_content(message);

        Ok(json!({
            "framework": framework_name,
            "role": message.role,
            "content": content,
            "metadata": message.metadata,
            "timestamp": chrono::Utc::now()
        }))
    }

    // 私有方法 - 从各框架格式转换

    fn convert_from_langchain(&self, value: Value) -> A2AResult<A2AMessage> {
        let msg: LangChainMessage = serde_json::from_value(value)
            .map_err(|e| A2AError::internal(format!("LangChain消息解析失败: {}", e)))?;

        let role = match msg.role.as_str() {
            "human" | "user" => MessageRole::User,
            "assistant" | "ai" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: msg.content,
                metadata: std::collections::HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata: msg.additional_kwargs,
        })
    }

    fn convert_from_autogen(&self, value: Value) -> A2AResult<A2AMessage> {
        let msg: AutoGenMessage = serde_json::from_value(value)
            .map_err(|e| A2AError::internal(format!("AutoGen消息解析失败: {}", e)))?;

        let role = match msg.role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        let mut metadata = msg.metadata;
        if let Some(name) = msg.name {
            metadata.insert("agent_name".to_string(), json!(name));
        }

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: msg.content,
                metadata: std::collections::HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata,
        })
    }

    fn convert_from_mastra(&self, value: Value) -> A2AResult<A2AMessage> {
        let msg: MastraMessage = serde_json::from_value(value)
            .map_err(|e| A2AError::internal(format!("Mastra消息解析失败: {}", e)))?;

        let role = match msg.role.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: msg.content,
                metadata: std::collections::HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata: msg.context,
        })
    }

    fn convert_from_crewai(&self, value: Value) -> A2AResult<A2AMessage> {
        // CrewAI使用类似LangChain的格式
        self.convert_from_langchain(value)
    }

    fn convert_from_semantic_kernel(&self, value: Value) -> A2AResult<A2AMessage> {
        let role_str = value.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user");

        let role = match role_str {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        let content = value.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let metadata = value.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: content,
                metadata: std::collections::HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata,
        })
    }

    fn convert_from_langgraph(&self, value: Value) -> A2AResult<A2AMessage> {
        // LangGraph使用LangChain兼容格式
        self.convert_from_langchain(value)
    }

    fn convert_from_custom(&self, value: Value, framework_name: &str) -> A2AResult<A2AMessage> {
        warn!("使用通用转换器处理自定义框架: {}", framework_name);
        
        let role_str = value.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user");

        let role = match role_str {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        let content = value.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let metadata = value.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: content,
                metadata: std::collections::HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata,
        })
    }

    fn initialize_default_rules(&mut self) {
        // 这里可以添加默认的转换规则
        // 目前使用硬编码的转换逻辑
    }

    /// 从A2A消息中提取文本内容
    fn extract_text_content(&self, message: &A2AMessage) -> String {
        message.parts.iter()
            .filter_map(|part| {
                if let MessagePart::Text(text_part) = part {
                    Some(text_part.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl Default for MessageConverter {
    fn default() -> Self {
        Self::new()
    }
}
