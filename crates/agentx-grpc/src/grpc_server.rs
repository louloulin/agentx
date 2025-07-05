//! gRPC服务器实现
//! 
//! 实现AgentX的gRPC服务器，提供插件注册和A2A消息处理服务

use crate::plugin_manager::{PluginManager, PluginConfig};
use crate::proto::{
    agent_x_plugin_server::{AgentXPlugin, AgentXPluginServer},
    A2aMessageRequest, A2aMessageResponse, A2aStreamChunk, InitializeRequest, InitializeResponse,
    HealthCheckResponse, RegisterAgentRequest, RegisterAgentResponse, PluginInfoResponse,
    MetricsResponse, ListAgentsRequest, ListAgentsResponse, PluginInfo as ProtoPluginInfo,
    ProcessingStats, Metric, MetricType,
};
use agentx_a2a::{A2AResult, A2AError, A2AMessage, MessageRole};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

/// gRPC服务器
pub struct AgentXGrpcServer {
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
    /// 服务器配置
    config: ServerConfig,
    /// 连接的插件信息
    connected_plugins: Arc<RwLock<HashMap<String, ConnectedPlugin>>>,
}

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub request_timeout_seconds: u64,
    pub enable_reflection: bool,
}

/// 连接的插件信息
#[derive(Debug, Clone)]
pub struct ConnectedPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub framework: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

impl AgentXGrpcServer {
    /// 创建新的gRPC服务器
    pub fn new(plugin_manager: Arc<PluginManager>, config: ServerConfig) -> Self {
        Self {
            plugin_manager,
            config,
            connected_plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 启动gRPC服务器
    pub async fn start(&self) -> A2AResult<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        println!("🚀 启动AgentX gRPC服务器: {}", addr);
        
        let service = AgentXPluginServer::new(self.clone());
        
        tonic::transport::Server::builder()
            .max_concurrent_streams(Some(self.config.max_connections))
            .add_service(service)
            .serve(addr.parse().map_err(|e| A2AError::internal(format!("地址解析失败: {}", e)))?)
            .await
            .map_err(|e| A2AError::internal(format!("gRPC服务器启动失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 获取服务器统计信息
    pub async fn get_server_stats(&self) -> ServerStats {
        let connected_plugins = self.connected_plugins.read().await;
        
        ServerStats {
            connected_plugins_count: connected_plugins.len(),
            total_requests: 0, // 简化处理
            active_streams: 0,
            uptime_seconds: 0,
            memory_usage_bytes: 0,
        }
    }
}

/// 服务器统计信息
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub connected_plugins_count: usize,
    pub total_requests: u64,
    pub active_streams: u32,
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
}

#[tonic::async_trait]
impl AgentXPlugin for AgentXGrpcServer {
    type ProcessA2AStreamStream = tokio_stream::wrappers::ReceiverStream<Result<A2aStreamChunk, tonic::Status>>;
    /// 初始化插件
    async fn initialize(
        &self,
        request: Request<InitializeRequest>,
    ) -> Result<Response<InitializeResponse>, Status> {
        let req = request.into_inner();
        println!("🔌 插件初始化请求: {}", req.plugin_id);
        
        // 创建插件配置
        let plugin_config = PluginConfig {
            id: req.plugin_id.clone(),
            name: req.plugin_id.clone(), // 简化处理
            endpoint: "localhost:0".to_string(), // 由客户端连接，不需要端点
            framework: "unknown".to_string(),
            auto_restart: true,
            max_retries: 3,
            timeout_seconds: 30,
            config: req.config,
        };
        
        // 添加插件配置
        if let Err(e) = self.plugin_manager.add_plugin_config(plugin_config).await {
            return Ok(Response::new(InitializeResponse {
                success: false,
                error_message: format!("插件配置添加失败: {}", e),
                plugin_info: None,
                supported_features: vec![],
            }));
        }
        
        // 记录连接的插件
        let connected_plugin = ConnectedPlugin {
            id: req.plugin_id.clone(),
            name: req.plugin_id.clone(),
            version: "1.0.0".to_string(),
            framework: "unknown".to_string(),
            connected_at: chrono::Utc::now(),
            last_heartbeat: chrono::Utc::now(),
        };
        
        self.connected_plugins.write().await.insert(
            req.plugin_id.clone(),
            connected_plugin,
        );
        
        // 创建响应
        let plugin_info = ProtoPluginInfo {
            name: req.plugin_id.clone(),
            version: "1.0.0".to_string(),
            description: "AgentX Plugin".to_string(),
            author: "AgentX".to_string(),
            supported_frameworks: vec!["agentx".to_string()],
            metadata: HashMap::new(),
        };
        
        let response = InitializeResponse {
            success: true,
            error_message: String::new(),
            plugin_info: Some(plugin_info),
            supported_features: vec![
                "a2a_messaging".to_string(),
                "streaming".to_string(),
                "agent_management".to_string(),
            ],
        };
        
        println!("✅ 插件 {} 初始化成功", req.plugin_id);
        Ok(Response::new(response))
    }
    
    /// 关闭插件
    async fn shutdown(
        &self,
        _request: Request<()>,
    ) -> Result<Response<()>, Status> {
        println!("🛑 插件关闭请求");
        // 这里可以添加清理逻辑
        Ok(Response::new(()))
    }
    
    /// 健康检查
    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let response = HealthCheckResponse {
            status: 1, // SERVING
            message: "AgentX gRPC服务器运行正常".to_string(),
            details: HashMap::new(),
        };
        
        Ok(Response::new(response))
    }
    
    /// 处理A2A消息
    async fn process_a2a_message(
        &self,
        request: Request<A2aMessageRequest>,
    ) -> Result<Response<A2aMessageResponse>, Status> {
        let req = request.into_inner();
        let start_time = std::time::Instant::now();
        
        println!("📨 处理A2A消息: {} -> {}", req.from_agent, req.to_agent);
        
        // 转换为内部A2A消息格式
        let a2a_message = A2AMessage::new_text(
            MessageRole::Agent,
            format!("Message from {}: {}", req.from_agent, req.message_id)
        );
        
        // 处理消息（这里简化为直接返回成功）
        let processing_time = start_time.elapsed();
        
        // 创建处理统计
        let stats = ProcessingStats {
            start_time: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            end_time: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            processing_time_ms: processing_time.as_millis() as i64,
            memory_used_bytes: 0,
            counters: HashMap::new(),
        };
        
        // 创建响应消息
        let response_message = A2aMessageRequest {
            message_id: Uuid::new_v4().to_string(),
            from_agent: req.to_agent.clone(),
            to_agent: req.from_agent.clone(),
            message_type: 2, // RESPONSE
            payload: None,
            metadata: HashMap::new(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            ttl_seconds: 300,
        };
        
        let response = A2aMessageResponse {
            success: true,
            error_message: String::new(),
            response_message: Some(response_message),
            stats: Some(stats),
        };
        
        println!("✅ A2A消息处理完成 (耗时: {:.2}ms)", processing_time.as_secs_f64() * 1000.0);
        Ok(Response::new(response))
    }
    
    /// 处理A2A流
    async fn process_a2a_stream(
        &self,
        request: Request<Streaming<A2aStreamChunk>>,
    ) -> Result<Response<Self::ProcessA2AStreamStream>, Status> {
        println!("🌊 处理A2A流");
        
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // 处理输入流
        tokio::spawn(async move {
            let mut chunk_count = 0;
            
            while let Some(chunk_result) = stream.message().await.transpose() {
                match chunk_result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        println!("📦 接收流块 {} (序号: {})", chunk.stream_id, chunk.sequence);
                        
                        // 简单回显处理
                        let response_chunk = A2aStreamChunk {
                            stream_id: chunk.stream_id,
                            sequence: chunk.sequence,
                            data: format!("Processed: {}", String::from_utf8_lossy(&chunk.data)).into_bytes(),
                            is_final: chunk.is_final,
                            checksum: chunk.checksum,
                            stream_type: chunk.stream_type,
                            metadata: chunk.metadata,
                        };
                        
                        if tx.send(Ok(response_chunk)).await.is_err() {
                            break;
                        }
                        
                        if chunk.is_final {
                            println!("✅ 流处理完成，总块数: {}", chunk_count);
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("❌ 流处理错误: {}", e);
                        break;
                    }
                }
            }
        });
        
        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(output_stream))
    }
    
    /// 注册Agent
    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let req = request.into_inner();
        
        if let Some(agent_info) = req.agent_info {
            println!("👤 注册Agent: {} ({})", agent_info.name, agent_info.framework);
            
            let response = RegisterAgentResponse {
                success: true,
                error_message: String::new(),
                agent_id: agent_info.id.clone(),
                registration_token: Uuid::new_v4().to_string(),
            };
            
            println!("✅ Agent {} 注册成功", agent_info.name);
            Ok(Response::new(response))
        } else {
            Ok(Response::new(RegisterAgentResponse {
                success: false,
                error_message: "Agent信息缺失".to_string(),
                agent_id: String::new(),
                registration_token: String::new(),
            }))
        }
    }
    
    /// 注销Agent
    async fn unregister_agent(
        &self,
        request: Request<crate::proto::UnregisterAgentRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        println!("👤 注销Agent: {}", req.agent_id);
        Ok(Response::new(()))
    }
    
    /// 列出Agent
    async fn list_agents(
        &self,
        _request: Request<ListAgentsRequest>,
    ) -> Result<Response<ListAgentsResponse>, Status> {
        let response = ListAgentsResponse {
            agents: vec![], // 简化处理
            next_page_token: String::new(),
            total_count: 0,
        };
        
        Ok(Response::new(response))
    }
    
    /// 获取Agent能力
    async fn get_agent_capabilities(
        &self,
        request: Request<crate::proto::GetAgentCapabilitiesRequest>,
    ) -> Result<Response<crate::proto::GetAgentCapabilitiesResponse>, Status> {
        let req = request.into_inner();
        println!("🔍 查询Agent能力: {}", req.agent_id);
        
        let response = crate::proto::GetAgentCapabilitiesResponse {
            capabilities: vec![], // 简化处理
            metadata: HashMap::new(),
        };
        
        Ok(Response::new(response))
    }
    
    /// 获取插件信息
    async fn get_plugin_info(
        &self,
        _request: Request<()>,
    ) -> Result<Response<PluginInfoResponse>, Status> {
        let plugin_info = ProtoPluginInfo {
            name: "AgentX Core".to_string(),
            version: "0.1.0".to_string(),
            description: "AgentX核心gRPC服务器".to_string(),
            author: "AgentX Team".to_string(),
            supported_frameworks: vec![
                "langchain".to_string(),
                "autogen".to_string(),
                "mastra".to_string(),
            ],
            metadata: HashMap::new(),
        };
        
        let response = PluginInfoResponse {
            plugin_info: Some(plugin_info),
            supported_frameworks: vec![
                "langchain".to_string(),
                "autogen".to_string(),
                "mastra".to_string(),
            ],
            runtime_info: HashMap::new(),
        };
        
        Ok(Response::new(response))
    }
    
    /// 获取指标
    async fn get_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let stats = self.get_server_stats().await;
        
        let metrics = vec![
            Metric {
                name: "connected_plugins".to_string(),
                r#type: MetricType::Gauge as i32,
                value: stats.connected_plugins_count as f64,
                labels: HashMap::new(),
                timestamp: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp(),
                    nanos: 0,
                }),
            },
            Metric {
                name: "total_requests".to_string(),
                r#type: MetricType::Counter as i32,
                value: stats.total_requests as f64,
                labels: HashMap::new(),
                timestamp: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp(),
                    nanos: 0,
                }),
            },
        ];
        
        let response = MetricsResponse {
            metrics,
            collected_at: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
        };
        
        Ok(Response::new(response))
    }
}

impl Clone for AgentXGrpcServer {
    fn clone(&self) -> Self {
        Self {
            plugin_manager: self.plugin_manager.clone(),
            config: self.config.clone(),
            connected_plugins: self.connected_plugins.clone(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
            max_connections: 1000,
            request_timeout_seconds: 30,
            enable_reflection: true,
        }
    }
}
