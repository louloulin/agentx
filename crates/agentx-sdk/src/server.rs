//! 服务器模块
//! 
//! 提供插件服务器和框架服务器的实现

use crate::plugin::Plugin;
use crate::framework::Framework;
use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use agentx_grpc::ServerConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 插件服务器
pub struct PluginServer {
    config: ServerConfig,
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    is_running: bool,
}

impl PluginServer {
    /// 创建新的插件服务器
    pub async fn new(config: ServerConfig) -> A2AResult<Self> {
        Ok(Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            is_running: false,
        })
    }
    
    /// 启动服务器
    pub async fn start(&mut self) -> A2AResult<()> {
        if self.is_running {
            return Err(A2AError::internal("服务器已经在运行"));
        }
        
        tracing::info!("启动插件服务器: {}:{}", self.config.host, self.config.port);
        
        // 这里应该启动实际的gRPC服务器
        // 暂时只标记为运行状态
        self.is_running = true;
        
        tracing::info!("插件服务器启动成功");
        Ok(())
    }
    
    /// 停止服务器
    pub async fn stop(&mut self) -> A2AResult<()> {
        if !self.is_running {
            return Ok(());
        }
        
        tracing::info!("停止插件服务器");
        
        // 停止所有插件
        let mut plugins = self.plugins.write().await;
        for (id, plugin) in plugins.iter_mut() {
            tracing::info!("停止插件: {}", id);
            let _ = plugin.stop().await;
        }
        
        self.is_running = false;
        tracing::info!("插件服务器已停止");
        Ok(())
    }
    
    /// 注册插件
    pub async fn register_plugin(&self, id: String, plugin: Box<dyn Plugin>) -> A2AResult<()> {
        let mut plugins = self.plugins.write().await;
        
        if plugins.contains_key(&id) {
            return Err(A2AError::internal(format!("插件已存在: {}", id)));
        }
        
        plugins.insert(id.clone(), plugin);
        tracing::info!("插件注册成功: {}", id);
        
        Ok(())
    }
    
    /// 注销插件
    pub async fn unregister_plugin(&self, id: &str) -> A2AResult<()> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(mut plugin) = plugins.remove(id) {
            let _ = plugin.stop().await;
            tracing::info!("插件注销成功: {}", id);
            Ok(())
        } else {
            Err(A2AError::internal(format!("插件不存在: {}", id)))
        }
    }
    
    /// 处理消息
    pub async fn process_message(&self, plugin_id: &str, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.process_message(message).await
        } else {
            Err(A2AError::internal(format!("插件不存在: {}", plugin_id)))
        }
    }
    
    /// 列出所有插件
    pub async fn list_plugins(&self) -> A2AResult<Vec<PluginSummary>> {
        let plugins = self.plugins.read().await;
        
        let summaries = plugins
            .iter()
            .map(|(id, plugin)| PluginSummary {
                id: id.clone(),
                name: plugin.get_info().metadata.name.clone(),
                version: plugin.get_info().metadata.version.clone(),
                status: plugin.get_status(),
                framework: plugin.get_info().config.framework.clone(),
            })
            .collect();
        
        Ok(summaries)
    }
    
    /// 获取服务器状态
    pub async fn get_status(&self) -> ServerStatus {
        let plugins = self.plugins.read().await;
        
        ServerStatus {
            is_running: self.is_running,
            plugin_count: plugins.len(),
            host: self.config.host.clone(),
            port: self.config.port,
            max_connections: self.config.max_connections,
        }
    }
}

/// 框架服务器
pub struct FrameworkServer {
    plugin_server: PluginServer,
    frameworks: Arc<RwLock<HashMap<String, Framework>>>,
}

impl FrameworkServer {
    /// 创建新的框架服务器
    pub async fn new(config: ServerConfig) -> A2AResult<Self> {
        let plugin_server = PluginServer::new(config).await?;
        
        Ok(Self {
            plugin_server,
            frameworks: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 注册框架
    pub async fn register_framework(&self, id: String, framework: Framework) -> A2AResult<()> {
        let mut frameworks = self.frameworks.write().await;
        
        if frameworks.contains_key(&id) {
            return Err(A2AError::internal(format!("框架已存在: {}", id)));
        }
        
        frameworks.insert(id.clone(), framework);
        tracing::info!("框架注册成功: {}", id);
        
        Ok(())
    }
    
    /// 启动框架
    pub async fn start_framework(&self, id: &str) -> A2AResult<()> {
        let mut frameworks = self.frameworks.write().await;
        
        if let Some(framework) = frameworks.get_mut(id) {
            framework.start().await?;
            tracing::info!("框架启动成功: {}", id);
            Ok(())
        } else {
            Err(A2AError::internal(format!("框架不存在: {}", id)))
        }
    }
    
    /// 停止框架
    pub async fn stop_framework(&self, id: &str) -> A2AResult<()> {
        let mut frameworks = self.frameworks.write().await;
        
        if let Some(framework) = frameworks.get_mut(id) {
            framework.stop().await?;
            tracing::info!("框架停止成功: {}", id);
            Ok(())
        } else {
            Err(A2AError::internal(format!("框架不存在: {}", id)))
        }
    }
    
    /// 处理框架消息
    pub async fn process_framework_message(&self, framework_id: &str, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let mut frameworks = self.frameworks.write().await;
        
        if let Some(framework) = frameworks.get_mut(framework_id) {
            framework.process_message(message).await
        } else {
            Err(A2AError::internal(format!("框架不存在: {}", framework_id)))
        }
    }
}

/// Agent服务器
pub struct AgentServer {
    framework_server: FrameworkServer,
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl AgentServer {
    /// 创建新的Agent服务器
    pub async fn new(config: ServerConfig) -> A2AResult<Self> {
        let framework_server = FrameworkServer::new(config).await?;
        
        Ok(Self {
            framework_server,
            agents: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 注册Agent
    pub async fn register_agent(&self, agent_info: AgentInfo) -> A2AResult<()> {
        let mut agents = self.agents.write().await;
        
        if agents.contains_key(&agent_info.id) {
            return Err(A2AError::internal(format!("Agent已存在: {}", agent_info.id)));
        }
        
        agents.insert(agent_info.id.clone(), agent_info.clone());
        tracing::info!("Agent注册成功: {}", agent_info.id);
        
        Ok(())
    }
    
    /// 处理Agent消息
    pub async fn process_agent_message(&self, agent_id: &str, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let agents = self.agents.read().await;
        
        if let Some(agent_info) = agents.get(agent_id) {
            // 根据Agent的框架类型路由消息
            self.framework_server.process_framework_message(&agent_info.framework, message).await
        } else {
            Err(A2AError::internal(format!("Agent不存在: {}", agent_id)))
        }
    }
}

/// 服务器管理器
pub struct ServerManager {
    servers: HashMap<String, PluginServer>,
}

impl ServerManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
    
    /// 添加服务器
    pub async fn add_server(&mut self, name: String, config: ServerConfig) -> A2AResult<()> {
        let server = PluginServer::new(config).await?;
        self.servers.insert(name, server);
        Ok(())
    }
    
    /// 启动所有服务器
    pub async fn start_all(&mut self) -> A2AResult<()> {
        for (name, server) in self.servers.iter_mut() {
            tracing::info!("启动服务器: {}", name);
            server.start().await?;
        }
        Ok(())
    }
    
    /// 停止所有服务器
    pub async fn stop_all(&mut self) -> A2AResult<()> {
        for (name, server) in self.servers.iter_mut() {
            tracing::info!("停止服务器: {}", name);
            server.stop().await?;
        }
        Ok(())
    }
}

/// 服务注册表
pub struct ServiceRegistry {
    services: HashMap<String, ServiceInfo>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    /// 注册服务
    pub fn register_service(&mut self, info: ServiceInfo) {
        self.services.insert(info.id.clone(), info);
    }
    
    /// 发现服务
    pub fn discover_service(&self, service_type: &str) -> Vec<&ServiceInfo> {
        self.services
            .values()
            .filter(|info| info.service_type == service_type)
            .collect()
    }
}

/// 端点管理器
pub struct EndpointManager {
    endpoints: HashMap<String, EndpointInfo>,
}

impl EndpointManager {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }
    
    /// 注册端点
    pub fn register_endpoint(&mut self, info: EndpointInfo) {
        self.endpoints.insert(info.path.clone(), info);
    }
    
    /// 获取端点
    pub fn get_endpoint(&self, path: &str) -> Option<&EndpointInfo> {
        self.endpoints.get(path)
    }
}

/// 插件摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: crate::plugin::PluginStatus,
    pub framework: String,
}

/// 服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub is_running: bool,
    pub plugin_count: usize,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

/// Agent信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub framework: String,
    pub capabilities: Vec<String>,
    pub status: String,
}

/// 服务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub endpoint: String,
    pub version: String,
}

/// 端点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub description: String,
    pub handler: String,
}

impl Default for ServerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for EndpointManager {
    fn default() -> Self {
        Self::new()
    }
}
