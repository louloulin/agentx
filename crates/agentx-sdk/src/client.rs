//! 客户端模块
//! 
//! 提供连接到AgentX服务器的客户端实现

use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 客户端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// 服务器URL
    pub server_url: String,
    /// 连接超时时间（秒）
    pub timeout: u64,
    /// 是否启用TLS
    pub enable_tls: bool,
}

impl ClientConfig {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            timeout: 30,
            enable_tls: false,
        }
    }
}

/// 插件客户端
pub struct PluginClient {
    config: ClientConfig,
    http_client: reqwest::Client,
}

impl PluginClient {
    /// 创建新的插件客户端
    pub async fn new(config: ClientConfig) -> A2AResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| A2AError::internal(format!("创建HTTP客户端失败: {}", e)))?;
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// 发送消息
    pub async fn send_message(&self, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let url = format!("{}/api/v1/messages", self.config.server_url);
        
        let response = self.http_client
            .post(&url)
            .json(&message)
            .send()
            .await
            .map_err(|e| A2AError::internal(format!("发送消息失败: {}", e)))?;
        
        if response.status().is_success() {
            let response_message: Option<A2AMessage> = response
                .json()
                .await
                .map_err(|e| A2AError::internal(format!("解析响应失败: {}", e)))?;
            
            Ok(response_message)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(A2AError::internal(format!("服务器错误: {}", error_text)))
        }
    }
    
    /// 获取服务器状态
    pub async fn get_server_status(&self) -> A2AResult<ServerStatus> {
        let url = format!("{}/api/v1/status", self.config.server_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| A2AError::internal(format!("获取状态失败: {}", e)))?;
        
        if response.status().is_success() {
            let status: ServerStatus = response
                .json()
                .await
                .map_err(|e| A2AError::internal(format!("解析状态失败: {}", e)))?;
            
            Ok(status)
        } else {
            Err(A2AError::internal("获取服务器状态失败"))
        }
    }
    
    /// 列出可用的插件
    pub async fn list_plugins(&self) -> A2AResult<Vec<PluginInfo>> {
        let url = format!("{}/api/v1/plugins", self.config.server_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| A2AError::internal(format!("获取插件列表失败: {}", e)))?;
        
        if response.status().is_success() {
            let plugins: Vec<PluginInfo> = response
                .json()
                .await
                .map_err(|e| A2AError::internal(format!("解析插件列表失败: {}", e)))?;
            
            Ok(plugins)
        } else {
            Err(A2AError::internal("获取插件列表失败"))
        }
    }
    
    /// 获取插件信息
    pub async fn get_plugin_info(&self, plugin_id: &str) -> A2AResult<PluginInfo> {
        let url = format!("{}/api/v1/plugins/{}", self.config.server_url, plugin_id);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| A2AError::internal(format!("获取插件信息失败: {}", e)))?;
        
        if response.status().is_success() {
            let plugin_info: PluginInfo = response
                .json()
                .await
                .map_err(|e| A2AError::internal(format!("解析插件信息失败: {}", e)))?;
            
            Ok(plugin_info)
        } else {
            Err(A2AError::internal("获取插件信息失败"))
        }
    }
}

/// 框架客户端
pub struct FrameworkClient {
    plugin_client: PluginClient,
    framework_type: String,
}

impl FrameworkClient {
    pub fn new(plugin_client: PluginClient, framework_type: String) -> Self {
        Self {
            plugin_client,
            framework_type,
        }
    }
    
    /// 发送框架特定的消息
    pub async fn send_framework_message(&self, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        // 添加框架标识
        let mut enhanced_message = message;
        enhanced_message.metadata.insert("framework".to_string(), serde_json::Value::String(self.framework_type.clone()));
        
        self.plugin_client.send_message(enhanced_message).await
    }
    
    /// 获取框架特定的插件
    pub async fn list_framework_plugins(&self) -> A2AResult<Vec<PluginInfo>> {
        let all_plugins = self.plugin_client.list_plugins().await?;
        
        let framework_plugins = all_plugins
            .into_iter()
            .filter(|plugin| plugin.framework == self.framework_type)
            .collect();
        
        Ok(framework_plugins)
    }
}

/// Agent客户端
pub struct AgentClient {
    framework_client: FrameworkClient,
    agent_id: String,
}

impl AgentClient {
    pub fn new(framework_client: FrameworkClient, agent_id: String) -> Self {
        Self {
            framework_client,
            agent_id,
        }
    }
    
    /// 发送Agent消息
    pub async fn send_agent_message(&self, content: String) -> A2AResult<Option<A2AMessage>> {
        let message = A2AMessage::agent_message(content);
        self.framework_client.send_framework_message(message).await
    }
    
    /// 获取Agent信息
    pub async fn get_agent_info(&self) -> A2AResult<PluginInfo> {
        self.framework_client.plugin_client.get_plugin_info(&self.agent_id).await
    }
}

/// 连接管理器
pub struct ConnectionManager {
    clients: std::collections::HashMap<String, PluginClient>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            clients: std::collections::HashMap::new(),
        }
    }
    
    /// 添加客户端连接
    pub async fn add_connection(&mut self, name: String, config: ClientConfig) -> A2AResult<()> {
        let client = PluginClient::new(config).await?;
        self.clients.insert(name, client);
        Ok(())
    }
    
    /// 获取客户端
    pub fn get_client(&self, name: &str) -> Option<&PluginClient> {
        self.clients.get(name)
    }
    
    /// 移除连接
    pub fn remove_connection(&mut self, name: &str) -> Option<PluginClient> {
        self.clients.remove(name)
    }
    
    /// 列出所有连接
    pub fn list_connections(&self) -> Vec<&String> {
        self.clients.keys().collect()
    }
}

/// 请求构建器
pub struct RequestBuilder {
    message: A2AMessage,
}

impl RequestBuilder {
    pub fn new(content: String) -> Self {
        Self {
            message: A2AMessage::agent_message(content),
        }
    }
    
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.message.metadata.insert(key, value);
        self
    }
    
    pub fn with_role(mut self, role: agentx_a2a::MessageRole) -> Self {
        self.message.role = role;
        self
    }
    
    pub fn build(self) -> A2AMessage {
        self.message
    }
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub active_connections: u32,
    pub total_messages: u64,
}

/// 插件信息（客户端视图）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub framework: String,
    pub status: String,
    pub capabilities: Vec<String>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_config() {
        let config = ClientConfig::new("http://localhost:50051");
        assert_eq!(config.server_url, "http://localhost:50051");
        assert_eq!(config.timeout, 30);
        assert!(!config.enable_tls);
    }
    
    #[test]
    fn test_request_builder() {
        let request = RequestBuilder::new("Hello".to_string())
            .with_metadata("test".to_string(), serde_json::Value::String("value".to_string()))
            .build();
        
        assert!(request.metadata.contains_key("test"));
    }
    
    #[test]
    fn test_connection_manager() {
        let mut manager = ConnectionManager::new();
        assert_eq!(manager.list_connections().len(), 0);
        
        // 测试连接管理基本功能
        assert!(manager.get_client("test").is_none());
    }
}
