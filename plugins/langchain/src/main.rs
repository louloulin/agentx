//! AgentX LangChain插件主程序
//! 
//! 提供LangChain框架与AgentX A2A协议的桥接功能

use agentx_grpc::proto::agent_x_plugin_server::{AgentXPlugin, AgentXPluginServer};
use agentx_grpc::proto::*;
use agentx_sdk::{PluginConfig, PluginMetadata, PluginStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, error, warn, debug};

mod langchain_adapter;
mod python_bridge;
mod config;
mod error;

use langchain_adapter::LangChainAdapter;
use python_bridge::PythonBridge;
use config::LangChainConfig;
use error::{LangChainError, LangChainResult};

/// LangChain插件服务
pub struct LangChainPlugin {
    /// 插件配置
    config: Arc<RwLock<LangChainConfig>>,
    /// LangChain适配器
    adapter: Arc<LangChainAdapter>,
    /// Python桥接器
    python_bridge: Arc<PythonBridge>,
    /// 插件元数据
    metadata: PluginMetadata,
    /// 插件状态
    status: Arc<RwLock<PluginStatus>>,
    /// 注册的Agent
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl LangChainPlugin {
    /// 创建新的LangChain插件实例
    pub async fn new(config: LangChainConfig) -> LangChainResult<Self> {
        info!("🚀 初始化LangChain插件...");
        
        // 创建Python桥接器
        let python_bridge = Arc::new(PythonBridge::new(&config).await?);
        
        // 创建LangChain适配器
        let adapter = Arc::new(LangChainAdapter::new(python_bridge.clone()).await?);
        
        // 创建插件元数据
        let metadata = PluginMetadata {
            name: "LangChain Plugin".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "AgentX LangChain框架插件".to_string(),
            author: "AgentX Team".to_string(),
            supported_frameworks: vec!["langchain".to_string()],
            capabilities: vec![
                "text.chat".to_string(),
                "text.completion".to_string(),
                "tool.calling".to_string(),
                "chain.execution".to_string(),
                "agent.reasoning".to_string(),
                "memory.conversation".to_string(),
                "retrieval.qa".to_string(),
            ],
            metadata: HashMap::new(),
        };
        
        info!("✅ LangChain插件初始化完成");
        info!("   支持的能力: {:?}", metadata.capabilities);
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            adapter,
            python_bridge,
            metadata,
            status: Arc::new(RwLock::new(PluginStatus::Initializing)),
            agents: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 启动插件服务器
    pub async fn serve(self, addr: std::net::SocketAddr) -> LangChainResult<()> {
        info!("🌐 启动LangChain插件gRPC服务器在 {}", addr);
        
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
            .map_err(|e| LangChainError::ServerError(format!("gRPC服务器启动失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 验证Python环境和LangChain安装
    async fn verify_environment(&self) -> LangChainResult<()> {
        info!("🔍 验证Python环境和LangChain安装...");
        
        // 检查Python版本
        let python_version = self.python_bridge.get_python_version().await?;
        info!("   Python版本: {}", python_version);
        
        // 检查LangChain版本
        let langchain_version = self.python_bridge.get_langchain_version().await?;
        info!("   LangChain版本: {}", langchain_version);
        
        // 验证必要的包
        let required_packages = vec![
            "langchain",
            "langchain-core", 
            "langchain-community",
            "openai",
            "anthropic",
        ];
        
        for package in required_packages {
            if !self.python_bridge.check_package_installed(package).await? {
                return Err(LangChainError::EnvironmentError(
                    format!("必需的Python包 '{}' 未安装", package)
                ));
            }
        }
        
        info!("✅ Python环境验证通过");
        Ok(())
    }
}

#[tonic::async_trait]
impl AgentXPlugin for LangChainPlugin {
    /// 初始化插件
    async fn initialize(
        &self,
        request: Request<InitializeRequest>,
    ) -> Result<Response<InitializeResponse>, Status> {
        let req = request.into_inner();
        info!("🔧 初始化LangChain插件，插件ID: {}", req.plugin_id);
        
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
        
        info!("✅ LangChain插件初始化成功");
        
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
        info!("🛑 关闭LangChain插件...");
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Stopping;
        }
        
        // 关闭适配器
        if let Err(e) = self.adapter.shutdown().await {
            warn!("适配器关闭时出现警告: {}", e);
        }
        
        // 关闭Python桥接器
        if let Err(e) = self.python_bridge.shutdown().await {
            warn!("Python桥接器关闭时出现警告: {}", e);
        }
        
        // 更新状态
        {
            let mut status = self.status.write().await;
            *status = PluginStatus::Stopped;
        }
        
        info!("✅ LangChain插件已关闭");
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
                // 检查Python桥接器健康状态
                match self.python_bridge.health_check().await {
                    Ok(_) => (
                        health_check_response::Status::Serving,
                        "LangChain插件运行正常".to_string()
                    ),
                    Err(e) => (
                        health_check_response::Status::NotServing,
                        format!("Python桥接器健康检查失败: {}", e)
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
        
        // TODO: 实现流式消息处理
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // 启动流处理任务
        let adapter = self.adapter.clone();
        tokio::spawn(async move {
            // 流处理逻辑
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
        
        info!("📝 注册LangChain Agent: {}", agent_info.name);
        
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
            supported_frameworks: vec!["langchain".to_string()],
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
        .with_env_filter("info,agentx_langchain_plugin=debug")
        .init();
    
    info!("🚀 启动AgentX LangChain插件");
    
    // 加载配置
    let config = LangChainConfig::load().await?;
    info!("📋 配置加载完成: {:?}", config);
    
    // 创建插件实例
    let plugin = LangChainPlugin::new(config.clone()).await?;
    
    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port).parse()?;
    info!("🌐 LangChain插件将在 {} 上提供服务", addr);
    
    plugin.serve(addr).await?;
    
    Ok(())
}
