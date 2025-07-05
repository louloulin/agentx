//! 适配器模块
//! 
//! 提供Agent、消息、工具和工作流的适配器接口

use agentx_a2a::{A2AMessage, A2AResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent适配器接口
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// 处理消息
    async fn process_message(&mut self, message: A2AMessage) -> A2AResult<Option<A2AMessage>>;
    
    /// 获取Agent信息
    async fn get_agent_info(&self) -> A2AResult<AgentInfo>;
    
    /// 设置Agent配置
    async fn configure(&mut self, config: AgentConfig) -> A2AResult<()>;
}

/// Agent信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub status: String,
}

/// Agent配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: Option<String>,
    pub tools: Vec<String>,
    pub custom: HashMap<String, serde_json::Value>,
}

/// Agent包装器
pub struct AgentWrapper {
    adapter: Box<dyn AgentAdapter>,
    info: AgentInfo,
    config: AgentConfig,
}

impl AgentWrapper {
    pub fn new(adapter: Box<dyn AgentAdapter>, info: AgentInfo, config: AgentConfig) -> Self {
        Self { adapter, info, config }
    }
    
    pub async fn process(&mut self, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        self.adapter.process_message(message).await
    }
    
    pub fn get_info(&self) -> &AgentInfo {
        &self.info
    }
    
    pub fn get_config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Agent代理
pub struct AgentProxy {
    agents: HashMap<String, AgentWrapper>,
}

impl AgentProxy {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
    
    pub fn register_agent(&mut self, id: String, agent: AgentWrapper) {
        self.agents.insert(id, agent);
    }
    
    pub async fn route_message(&mut self, agent_id: &str, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.process(message).await
        } else {
            Err(agentx_a2a::A2AError::internal(format!("Agent not found: {}", agent_id)))
        }
    }
    
    pub fn list_agents(&self) -> Vec<&AgentInfo> {
        self.agents.values().map(|a| a.get_info()).collect()
    }
}

/// Agent注册表
pub struct AgentRegistry {
    agents: HashMap<String, AgentInfo>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, info: AgentInfo) {
        self.agents.insert(info.id.clone(), info);
    }
    
    pub fn unregister(&mut self, id: &str) -> Option<AgentInfo> {
        self.agents.remove(id)
    }
    
    pub fn get(&self, id: &str) -> Option<&AgentInfo> {
        self.agents.get(id)
    }
    
    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }
    
    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|info| info.capabilities.contains(&capability.to_string()))
            .collect()
    }
}

/// 消息适配器
#[async_trait]
pub trait MessageAdapter: Send + Sync {
    /// 转换消息格式
    async fn convert_message(&self, message: A2AMessage, target_format: &str) -> A2AResult<serde_json::Value>;
    
    /// 验证消息
    async fn validate_message(&self, message: &A2AMessage) -> A2AResult<bool>;
    
    /// 增强消息
    async fn enhance_message(&self, message: A2AMessage) -> A2AResult<A2AMessage>;
}

/// 工具适配器
#[async_trait]
pub trait ToolAdapter: Send + Sync {
    /// 执行工具
    async fn execute_tool(&self, tool_name: &str, args: serde_json::Value) -> A2AResult<serde_json::Value>;
    
    /// 获取工具列表
    async fn list_tools(&self) -> A2AResult<Vec<ToolInfo>>;
    
    /// 获取工具信息
    async fn get_tool_info(&self, tool_name: &str) -> A2AResult<ToolInfo>;
}

/// 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub returns: serde_json::Value,
}

/// 工作流适配器
#[async_trait]
pub trait WorkflowAdapter: Send + Sync {
    /// 执行工作流
    async fn execute_workflow(&self, workflow_id: &str, inputs: serde_json::Value) -> A2AResult<serde_json::Value>;
    
    /// 获取工作流列表
    async fn list_workflows(&self) -> A2AResult<Vec<WorkflowInfo>>;
    
    /// 创建工作流
    async fn create_workflow(&self, definition: WorkflowDefinition) -> A2AResult<String>;
}

/// 工作流信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub inputs: serde_json::Value,
    pub outputs: serde_json::Value,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: String,
    pub config: serde_json::Value,
    pub dependencies: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "default_agent".to_string(),
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            system_prompt: None,
            tools: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentProxy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_registry() {
        let mut registry = AgentRegistry::new();
        
        let info = AgentInfo {
            id: "test_agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            capabilities: vec!["text_processing".to_string()],
            status: "active".to_string(),
        };
        
        registry.register(info.clone());
        
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get("test_agent").is_some());
        
        let found = registry.find_by_capability("text_processing");
        assert_eq!(found.len(), 1);
    }
    
    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.name, "default_agent");
        assert_eq!(config.model, "gpt-3.5-turbo");
        assert_eq!(config.temperature, 0.7);
    }
}
