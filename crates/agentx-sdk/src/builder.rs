//! 构建器模块
//! 
//! 提供各种组件的构建器模式实现

use crate::plugin::{Plugin, PluginConfig, PluginInfo, PluginMetadata, PluginStats, PluginCapability, PluginStatus};
use crate::framework::{Framework, FrameworkType, FrameworkConfig, FrameworkAdapter, LangChainAdapter, AutoGenAdapter, MastraAdapter};
use crate::adapter::{AgentConfig, AgentInfo};
use crate::client::ClientConfig;
use crate::server::PluginServer;
use agentx_a2a::{A2AResult, A2AError};
use agentx_grpc::ServerConfig;
use std::collections::HashMap;

/// 插件构建器
pub struct PluginBuilder {
    framework: Option<String>,
    config: Option<PluginConfig>,
    metadata: Option<PluginMetadata>,
    capabilities: Vec<PluginCapability>,
}

impl PluginBuilder {
    pub fn new() -> Self {
        Self {
            framework: None,
            config: None,
            metadata: None,
            capabilities: Vec::new(),
        }
    }
    
    pub fn framework(mut self, framework: &str) -> Self {
        self.framework = Some(framework.to_string());
        self
    }
    
    pub fn config(mut self, config: PluginConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub fn metadata(mut self, metadata: PluginMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    pub fn capability(mut self, capability: PluginCapability) -> Self {
        self.capabilities.push(capability);
        self
    }
    
    pub async fn build(self) -> A2AResult<Box<dyn Plugin>> {
        let framework = self.framework.ok_or_else(|| A2AError::internal("Framework not specified"))?;
        let config = self.config.unwrap_or_else(|| PluginConfig::default_for_framework(&framework).unwrap());
        let metadata = self.metadata.unwrap_or_else(|| {
            PluginMetadata::new(&format!("{}_plugin", framework), "1.0.0", &format!("{} plugin", framework), "AgentX")
        });
        
        let capabilities = if self.capabilities.is_empty() {
            vec![PluginCapability::TextProcessing, PluginCapability::ToolCalling]
        } else {
            self.capabilities
        };
        
        let plugin_info = PluginInfo {
            metadata,
            status: PluginStatus::Uninitialized,
            capabilities,
            config: config.clone(),
            stats: PluginStats::default(),
        };
        
        let plugin = GenericPlugin::new(plugin_info);
        Ok(Box::new(plugin))
    }
}

/// 框架构建器
pub struct FrameworkBuilder {
    framework_type: Option<FrameworkType>,
    config: Option<FrameworkConfig>,
}

impl FrameworkBuilder {
    pub fn new() -> Self {
        Self {
            framework_type: None,
            config: None,
        }
    }
    
    pub fn framework_type(mut self, framework_type: FrameworkType) -> Self {
        self.framework_type = Some(framework_type);
        self
    }
    
    pub fn config(mut self, config: FrameworkConfig) -> Self {
        self.config = Some(config);
        self
    }
    
    pub async fn build(self) -> A2AResult<Framework> {
        let framework_type = self.framework_type.ok_or_else(|| A2AError::internal("Framework type not specified"))?;
        let mut config = self.config.unwrap_or_default();
        config.framework_type = framework_type.clone();
        
        let adapter: Box<dyn FrameworkAdapter> = match framework_type {
            FrameworkType::LangChain => Box::new(LangChainAdapter::new(config.clone())),
            FrameworkType::AutoGen => Box::new(AutoGenAdapter::new(config.clone())),
            FrameworkType::Mastra => Box::new(MastraAdapter::new(config.clone())),
            _ => return Err(A2AError::internal("Unsupported framework type")),
        };
        
        Ok(Framework::new(framework_type, config, adapter))
    }
}

/// 适配器构建器
pub struct AdapterBuilder {
    agent_config: Option<AgentConfig>,
    agent_info: Option<AgentInfo>,
}

impl AdapterBuilder {
    pub fn new() -> Self {
        Self {
            agent_config: None,
            agent_info: None,
        }
    }
    
    pub fn agent_config(mut self, config: AgentConfig) -> Self {
        self.agent_config = Some(config);
        self
    }
    
    pub fn agent_info(mut self, info: AgentInfo) -> Self {
        self.agent_info = Some(info);
        self
    }
    
    pub fn build(self) -> A2AResult<(AgentConfig, AgentInfo)> {
        let config = self.agent_config.unwrap_or_default();
        let info = self.agent_info.unwrap_or_else(|| AgentInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name: config.name.clone(),
            description: "Generic agent".to_string(),
            capabilities: vec!["text_processing".to_string()],
            status: "active".to_string(),
        });
        
        Ok((config, info))
    }
}

/// 配置构建器
pub struct ConfigBuilder {
    framework: Option<String>,
    runtime_path: Option<String>,
    working_directory: Option<String>,
    environment_variables: HashMap<String, String>,
    custom_config: HashMap<String, serde_json::Value>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            framework: None,
            runtime_path: None,
            working_directory: None,
            environment_variables: HashMap::new(),
            custom_config: HashMap::new(),
        }
    }
    
    pub fn framework(mut self, framework: &str) -> Self {
        self.framework = Some(framework.to_string());
        self
    }
    
    pub fn runtime_path(mut self, path: &str) -> Self {
        self.runtime_path = Some(path.to_string());
        self
    }
    
    pub fn working_directory(mut self, dir: &str) -> Self {
        self.working_directory = Some(dir.to_string());
        self
    }
    
    pub fn env_var(mut self, key: &str, value: &str) -> Self {
        self.environment_variables.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn custom(mut self, key: &str, value: serde_json::Value) -> Self {
        self.custom_config.insert(key.to_string(), value);
        self
    }
    
    pub fn build(self) -> A2AResult<PluginConfig> {
        let framework = self.framework.unwrap_or_else(|| "custom".to_string());
        
        Ok(PluginConfig {
            framework,
            framework_version: None,
            bind_address: "127.0.0.1:0".to_string(),
            server_address: "127.0.0.1:50051".to_string(),
            max_connections: 100,
            request_timeout: 30,
            enable_tls: false,
            custom: self.custom_config,
        })
    }
}

/// 服务器构建器
pub struct ServerBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<u32>,
    enable_reflection: Option<bool>,
    request_timeout: Option<u64>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            port: None,
            max_connections: None,
            enable_reflection: None,
            request_timeout: None,
        }
    }
    
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }
    
    pub fn enable_reflection(mut self, enable: bool) -> Self {
        self.enable_reflection = Some(enable);
        self
    }
    
    pub fn request_timeout(mut self, timeout: u64) -> Self {
        self.request_timeout = Some(timeout);
        self
    }
    
    pub async fn build(self) -> A2AResult<PluginServer> {
        let config = ServerConfig {
            host: self.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            port: self.port.unwrap_or(50051),
            max_connections: self.max_connections.unwrap_or(1000),
            enable_reflection: self.enable_reflection.unwrap_or(true),
            request_timeout_seconds: self.request_timeout.unwrap_or(30),
        };
        
        PluginServer::new(config).await
    }
}

/// 客户端构建器
pub struct ClientBuilder {
    server_url: Option<String>,
    timeout: Option<u64>,
    enable_tls: Option<bool>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            server_url: None,
            timeout: None,
            enable_tls: None,
        }
    }
    
    pub fn server_url(mut self, url: &str) -> Self {
        self.server_url = Some(url.to_string());
        self
    }
    
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.enable_tls = Some(enable);
        self
    }
    
    pub fn build(self) -> A2AResult<ClientConfig> {
        let server_url = self.server_url.ok_or_else(|| A2AError::internal("Server URL not specified"))?;
        
        Ok(ClientConfig {
            server_url,
            timeout: self.timeout.unwrap_or(30),
            enable_tls: self.enable_tls.unwrap_or(false),
        })
    }
}

/// 通用插件实现
struct GenericPlugin {
    info: PluginInfo,
}

impl GenericPlugin {
    fn new(info: PluginInfo) -> Self {
        Self { info }
    }
}

#[async_trait::async_trait]
impl crate::plugin::PluginLifecycle for GenericPlugin {
    async fn initialize(&mut self) -> A2AResult<()> {
        self.info.status = PluginStatus::Initializing;
        // 初始化逻辑
        self.info.status = PluginStatus::Running;
        Ok(())
    }
    
    async fn start(&mut self) -> A2AResult<()> {
        self.info.status = PluginStatus::Running;
        Ok(())
    }
    
    async fn stop(&mut self) -> A2AResult<()> {
        self.info.status = PluginStatus::Stopped;
        Ok(())
    }
    
    async fn pause(&mut self) -> A2AResult<()> {
        self.info.status = PluginStatus::Paused;
        Ok(())
    }
    
    async fn resume(&mut self) -> A2AResult<()> {
        self.info.status = PluginStatus::Running;
        Ok(())
    }
    
    async fn health_check(&self) -> A2AResult<bool> {
        Ok(matches!(self.info.status, PluginStatus::Running))
    }
}

#[async_trait::async_trait]
impl Plugin for GenericPlugin {
    fn get_info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn get_status(&self) -> PluginStatus {
        self.info.status.clone()
    }
    
    async fn process_message(&mut self, message: agentx_a2a::A2AMessage) -> A2AResult<Option<agentx_a2a::A2AMessage>> {
        // 简单的回显处理
        let response = agentx_a2a::A2AMessage::agent_message(
            format!("Processed: {}", message.message_id)
        );
        Ok(Some(response))
    }
    
    async fn send_message(&mut self, _message: agentx_a2a::A2AMessage) -> A2AResult<()> {
        // 发送消息逻辑
        Ok(())
    }
    
    async fn register_event_handler(&mut self, _handler: Box<dyn crate::plugin::PluginEventHandler>) -> A2AResult<()> {
        // 注册事件处理器
        Ok(())
    }
    
    fn get_capabilities(&self) -> &[PluginCapability] {
        &self.info.capabilities
    }
    
    async fn update_config(&mut self, config: PluginConfig) -> A2AResult<()> {
        self.info.config = config;
        Ok(())
    }
    
    fn get_stats(&self) -> &PluginStats {
        &self.info.stats
    }
}

impl Default for PluginBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FrameworkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AdapterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
