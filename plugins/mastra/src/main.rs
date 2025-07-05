//! AgentX Mastra插件主程序
//! 
//! 提供Mastra框架与AgentX A2A协议的桥接功能

use agentx_grpc::proto::agent_x_plugin_server::{AgentXPlugin, AgentXPluginServer};
use agentx_grpc::proto::*;
use agentx_sdk::{PluginConfig, PluginMetadata, PluginStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, error, warn, debug};

mod mastra_adapter;
mod typescript_bridge;
mod config;
mod error;
mod workflow_manager;
mod tool_manager;

use mastra_adapter::MastraAdapter;
use typescript_bridge::TypeScriptBridge;
use config::MastraConfig;
use error::{MastraError, MastraResult};
use workflow_manager::WorkflowManager;
use tool_manager::ToolManager;

/// Mastra插件服务
pub struct MastraPlugin {
    /// 插件配置
    config: Arc<RwLock<MastraConfig>>,
    /// Mastra适配器
    adapter: Arc<MastraAdapter>,
    /// TypeScript桥接器
    typescript_bridge: Arc<TypeScriptBridge>,
    /// 工作流管理器
    workflow_manager: Arc<WorkflowManager>,
    /// 工具管理器
    tool_manager: Arc<ToolManager>,
    /// 插件元数据
    metadata: PluginMetadata,
    /// 插件状态
    status: Arc<RwLock<PluginStatus>>,
    /// 注册的Agent
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl MastraPlugin {
    /// 创建新的Mastra插件实例
    pub async fn new(config: MastraConfig) -> MastraResult<Self> {
        info!("🚀 初始化Mastra插件...");
        
        // 创建TypeScript桥接器
        let typescript_bridge = Arc::new(TypeScriptBridge::new(&config).await?);
        
        // 创建工具管理器
        let tool_manager = Arc::new(ToolManager::new().await?);
        
        // 创建工作流管理器
        let workflow_manager = Arc::new(WorkflowManager::new().await?);
        
        // 创建Mastra适配器
        let adapter = Arc::new(MastraAdapter::new(
            typescript_bridge.clone(),
            workflow_manager.clone(),
            tool_manager.clone(),
        ).await?);
        
        // 创建插件元数据
        let metadata = PluginMetadata {
            name: "Mastra Plugin".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AgentX Mastra框架插件".to_string(),
            author: "AgentX Team".to_string(),
            supported_frameworks: vec!["mastra".to_string()],
            capabilities: vec![
                "workflow.execution".to_string(),
                "tool.integration".to_string(),
                "agent.orchestration".to_string(),
                "memory.management".to_string(),
                "eval.framework".to_string(),
                "typescript.runtime".to_string(),
                "api.integration".to_string(),
                "data.processing".to_string(),
                "real_time.communication".to_string(),
            ],
            metadata: HashMap::new(),
        };
        
        info!("✅ Mastra插件初始化完成");
        info!("   支持的能力: {:?}", metadata.capabilities);
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            adapter,
            typescript_bridge,
            workflow_manager,
            tool_manager,
            metadata,
            status: Arc::new(RwLock::new(PluginStatus::Initializing)),
            agents: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 启动插件服务器
    pub async fn serve(self, addr: std::net::SocketAddr) -> MastraResult<()> {
        info!("🌐 启动Mastra插件gRPC服务器在 {}", addr);
        
        // 更新状态为运行中
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Running;
        }
        
        let service = AgentXPluginServer::new(self);
        
        Server::builder()
            .add_service(service)
            .serve(addr)
            .await
            .map_err(|e| MastraError::ServerError(format!("gRPC服务器启动失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 验证Node.js环境和Mastra安装
    async fn verify_environment(&self) -> MastraResult<()> {
        info!("🔍 验证Node.js环境和Mastra安装...");
        
        // 检查Node.js版本
        let node_version = self.typescript_bridge.get_node_version().await?;
        info!("   Node.js版本: {}", node_version);
        
        // 检查Mastra版本
        let mastra_version = self.typescript_bridge.get_mastra_version().await?;
        info!("   Mastra版本: {}", mastra_version);
        
        // 验证必要的包
        let required_packages = vec![
            "@mastra/core",
            "@mastra/workflows",
            "@mastra/tools",
            "@mastra/memory",
            "@mastra/evals",
        ];
        
        for package in required_packages {
            if !self.typescript_bridge.check_package_installed(package).await? {
                return Err(MastraError::EnvironmentError(
                    format!("必需的npm包 '{}' 未安装", package)
                ));
            }
        }
        
        info!("✅ Node.js环境验证通过");
        Ok(())
    }
}

#[tonic::async_trait]
impl AgentXPlugin for MastraPlugin {
    /// 初始化插件
    async fn initialize(
        &self,
        request: Request<InitializeRequest>,
    ) -> Result<Response<InitializeResponse>, Status> {
        let req = request.into_inner();
        info!("🔧 初始化Mastra插件，插件ID: {}", req.plugin_id);
        
        // 验证环境
        if let Err(e) = self.verify_environment().await {
            error!("环境验证失败: {}", e);
            return Ok(Response::new(InitializeResponse {
                success: false,
                error_message: format!("环境验证失败: {}", e),
                plugin_info: None,
                supported_features: vec![],
            }));
        }
        
        // 更新配置
        {
            let mut config = self.config.write().await;
            for (key, value) in req.config {
                config.set_parameter(&key, &value);
            }
        }
        
        // 初始化适配器
        if let Err(e) = self.adapter.initialize().await {
            error!("适配器初始化失败: {}", e);
            return Ok(Response::new(InitializeResponse {
                success: false,
                error_message: format!("适配器初始化失败: {}", e),
                plugin_info: None,
                supported_features: vec![],
            }));
        }
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Ready;
        }
        
        info!("✅ Mastra插件初始化成功");
        
        Ok(Response::new(InitializeResponse {
            success: true,
            error_message: String::new(),
            plugin_info: Some(PluginInfo {
                name: self.metadata.name.clone(),
                version: self.metadata.version.clone(),
                description: self.metadata.description.clone(),
                author: self.metadata.author.clone(),
                supported_frameworks: self.metadata.supported_frameworks.clone(),
                metadata: HashMap::new(),
            }),
            supported_features: self.metadata.capabilities.clone(),
        }))
    }
    
    /// 关闭插件
    async fn shutdown(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        info!("🛑 关闭Mastra插件...");
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Stopping;
        }
        
        // 关闭工作流管理器
        if let Err(e) = self.workflow_manager.shutdown().await {
            warn!("工作流管理器关闭时出现警告: {}", e);
        }
        
        // 关闭工具管理器
        if let Err(e) = self.tool_manager.shutdown().await {
            warn!("工具管理器关闭时出现警告: {}", e);
        }
        
        // 关闭适配器
        if let Err(e) = self.adapter.shutdown().await {
            warn!("适配器关闭时出现警告: {}", e);
        }
        
        // 关闭TypeScript桥接器
        if let Err(e) = self.typescript_bridge.shutdown().await {
            warn!("TypeScript桥接器关闭时出现警告: {}", e);
        }
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Stopped;
        }
        
        info!("✅ Mastra插件已关闭");
        Ok(Response::new(()))
    }
    
    /// 健康检查
    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let status = self.status.read().await;
        
        let (health_status, message) = match *status {
            PluginStatus::Running | PluginStatus::Ready => {
                // 检查TypeScript桥接器健康状态
                match self.typescript_bridge.health_check().await {
                    Ok(_) => (
                        health_check_response::Status::Serving,
                        "Mastra插件运行正常".to_string()
                    ),
                    Err(e) => (
                        health_check_response::Status::NotServing,
                        format!("TypeScript桥接器健康检查失败: {}", e)
                    ),
                }
            }
            PluginStatus::Initializing => (
                health_check_response::Status::NotServing,
                "插件正在初始化".to_string()
            ),
            PluginStatus::Stopping | PluginStatus::Stopped => (
                health_check_response::Status::NotServing,
                "插件已停止".to_string()
            ),
            PluginStatus::Error => (
                health_check_response::Status::NotServing,
                "插件处于错误状态".to_string()
            ),
        };
        
        Ok(Response::new(HealthCheckResponse {
            status: health_status as i32,
            message,
            details: HashMap::new(),
        }))
    }
    
    /// 处理A2A消息
    async fn process_a2a_message(
        &self,
        request: Request<A2aMessageRequest>,
    ) -> Result<Response<A2aMessageResponse>, Status> {
        let req = request.into_inner();
        debug!("📨 处理A2A消息: {}", req.message_id);
        
        let start_time = std::time::Instant::now();
        
        // 通过适配器处理消息
        match self.adapter.process_message(req.clone()).await {
            Ok(response_message) => {
                let processing_time = start_time.elapsed();
                debug!("✅ 消息处理完成，耗时: {:?}", processing_time);
                
                Ok(Response::new(A2aMessageResponse {
                    success: true,
                    error_message: String::new(),
                    response_message: Some(response_message),
                    stats: Some(ProcessingStats {
                        start_time: None,
                        end_time: None,
                        processing_time_ms: processing_time.as_millis() as i64,
                        memory_used_bytes: 0, // TODO: 实现内存使用统计
                        counters: HashMap::new(),
                    }),
                }))
            }
            Err(e) => {
                error!("❌ 消息处理失败: {}", e);
                Ok(Response::new(A2aMessageResponse {
                    success: false,
                    error_message: e.to_string(),
                    response_message: None,
                    stats: None,
                }))
            }
        }
    }
    
    /// 处理A2A流式消息
    async fn process_a2a_stream(
        &self,
        request: Request<tonic::Streaming<A2aStreamChunk>>,
    ) -> Result<Response<Self::ProcessA2AStreamStream>, Status> {
        info!("🌊 开始处理A2A流式消息");
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // 启动流处理任务
        let adapter = self.adapter.clone();
        tokio::spawn(async move {
            if let Err(e) = adapter.process_stream(request.into_inner(), tx).await {
                error!("流处理失败: {}", e);
            }
        });
        
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx)
        )))
    }
    
    type ProcessA2AStreamStream = std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<A2aStreamChunk, Status>> + Send>>;
    
    /// 注册Agent
    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let req = request.into_inner();
        let agent_info = req.agent_info.ok_or_else(|| {
            Status::invalid_argument("缺少Agent信息")
        })?;
        
        info!("📝 注册Mastra Agent: {}", agent_info.name);
        
        // 通过适配器注册Agent
        match self.adapter.register_agent(agent_info.clone(), req.capabilities).await {
            Ok(agent_id) => {
                // 缓存Agent信息
                {
                    let mut agents = self.agents.write().await;
                    agents.insert(agent_id.clone(), agent_info);
                }
                
                info!("✅ Agent注册成功: {}", agent_id);
                
                Ok(Response::new(RegisterAgentResponse {
                    success: true,
                    error_message: String::new(),
                    agent_id,
                    registration_token: uuid::Uuid::new_v4().to_string(),
                }))
            }
            Err(e) => {
                error!("❌ Agent注册失败: {}", e);
                Ok(Response::new(RegisterAgentResponse {
                    success: false,
                    error_message: e.to_string(),
                    agent_id: String::new(),
                    registration_token: String::new(),
                }))
            }
        }
    }
    
    /// 注销Agent
    async fn unregister_agent(
        &self,
        request: Request<UnregisterAgentRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        info!("🗑️ 注销Agent: {}", req.agent_id);
        
        // 从缓存中移除
        {
            let mut agents = self.agents.write().await;
            agents.remove(&req.agent_id);
        }
        
        // 通过适配器注销Agent
        if let Err(e) = self.adapter.unregister_agent(&req.agent_id).await {
            warn!("Agent注销时出现警告: {}", e);
        }
        
        info!("✅ Agent注销完成: {}", req.agent_id);
        Ok(Response::new(()))
    }
    
    /// 列出Agent
    async fn list_agents(
        &self,
        _request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let agents = self.agents.read().await;
        let agent_list: Vec<AgentInfo> = agents.values().cloned().collect();
        
        Ok(Response::new(ListAgentsResponse {
            agents: agent_list,
            next_page_token: String::new(),
            total_count: agents.len() as i32,
        }))
    }
    
    /// 获取Agent能力
    async fn get_agent_capabilities(
        &self,
        request: Request<GetAgentCapabilitiesRequest>,
    ) -> Result<Response<GetAgentCapabilitiesResponse>, Status> {
        let req = request.into_inner();
        
        match self.adapter.get_agent_capabilities(&req.agent_id).await {
            Ok(capabilities) => Ok(Response::new(GetAgentCapabilitiesResponse {
                capabilities,
                metadata: HashMap::new(),
            })),
            Err(e) => Err(Status::not_found(format!("Agent不存在: {}", e))),
        }
    }
    
    /// 获取插件信息
    async fn get_plugin_info(
        &self,
        _request: Request<()>,
    ) -> Result<Response<PluginInfoResponse>, Status> {
        let config = self.config.read().await;
        
        Ok(Response::new(PluginInfoResponse {
            plugin_info: Some(PluginInfo {
                name: self.metadata.name.clone(),
                version: self.metadata.version.clone(),
                description: self.metadata.description.clone(),
                author: self.metadata.author.clone(),
                supported_frameworks: self.metadata.supported_frameworks.clone(),
                metadata: HashMap::new(),
            }),
            supported_frameworks: vec!["mastra".to_string()],
            runtime_info: config.get_runtime_info(),
        }))
    }
    
    /// 获取指标
    async fn get_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MetricsResponse>, Status> {
        // TODO: 实现详细的指标收集
        Ok(Response::new(MetricsResponse {
            metrics: vec![],
            collected_at: None,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,agentx_mastra_plugin=debug")
        .init();
    
    info!("🚀 启动AgentX Mastra插件");
    
    // 加载配置
    let config = MastraConfig::load().await?;
    info!("📋 配置加载完成: {:?}", config);
    
    // 创建插件实例
    let plugin = MastraPlugin::new(config.clone()).await?;
    
    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port).parse()?;
    info!("🌐 Mastra插件将在 {} 上提供服务", addr);
    
    plugin.serve(addr).await?;
    
    Ok(())
}
