//! gRPC插件桥接模块
//! 
//! 实现gRPC插件系统与A2A协议的桥接，支持不同框架的Agent通过gRPC插件接入

use crate::proto::{
    agent_x_plugin_client::AgentXPluginClient,
    A2aMessageRequest, A2aStreamChunk, InitializeRequest,
};
use agentx_a2a::{
    A2AMessage, A2AProtocolEngine, StreamManager, StreamChunk, StreamType,
    SecurityManager, A2AResult, A2AError,
};
use agentx_a2a::monitoring::MonitoringManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tonic::Request;

/// gRPC插件桥接器
pub struct PluginBridge {
    /// A2A协议引擎
    #[allow(dead_code)]
    a2a_engine: Arc<RwLock<A2AProtocolEngine>>,
    /// 流管理器
    #[allow(dead_code)]
    stream_manager: Arc<RwLock<StreamManager>>,
    /// 安全管理器
    #[allow(dead_code)]
    security_manager: Arc<RwLock<SecurityManager>>,
    /// 监控管理器
    monitoring_manager: Arc<RwLock<MonitoringManager>>,
    /// 插件客户端连接池
    plugin_clients: Arc<RwLock<HashMap<String, AgentXPluginClient<tonic::transport::Channel>>>>,
    /// 插件信息缓存
    plugin_info_cache: Arc<RwLock<HashMap<String, PluginInfo>>>,
    /// 消息路由表
    message_routes: Arc<RwLock<HashMap<String, String>>>, // agent_id -> plugin_id
}

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub framework: String,
    pub status: PluginStatus,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// 插件状态
#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Initializing,
    Running,
    Stopped,
    Error(String),
}

impl PluginBridge {
    /// 创建新的插件桥接器
    pub fn new(
        a2a_engine: Arc<RwLock<A2AProtocolEngine>>,
        stream_manager: Arc<RwLock<StreamManager>>,
        security_manager: Arc<RwLock<SecurityManager>>,
        monitoring_manager: Arc<RwLock<MonitoringManager>>,
    ) -> Self {
        Self {
            a2a_engine,
            stream_manager,
            security_manager,
            monitoring_manager,
            plugin_clients: Arc::new(RwLock::new(HashMap::new())),
            plugin_info_cache: Arc::new(RwLock::new(HashMap::new())),
            message_routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册插件
    pub async fn register_plugin(
        &self,
        plugin_id: String,
        endpoint: String,
        config: HashMap<String, String>,
    ) -> A2AResult<()> {
        println!("🔌 注册插件: {} (端点: {})", plugin_id, endpoint);
        
        // 创建gRPC客户端连接
        let client = AgentXPluginClient::connect(endpoint.clone()).await
            .map_err(|e| A2AError::internal(format!("连接插件失败: {}", e)))?;
        
        // 初始化插件
        let init_request = InitializeRequest {
            plugin_id: plugin_id.clone(),
            config: config.clone(),
            agentx_version: "0.1.0".to_string(),
            supported_protocols: vec!["a2a".to_string()],
        };
        
        let mut client_clone = client.clone();
        let response = client_clone.initialize(Request::new(init_request)).await
            .map_err(|e| A2AError::internal(format!("插件初始化失败: {}", e)))?;
        
        let init_response = response.into_inner();
        if !init_response.success {
            return Err(A2AError::internal(format!(
                "插件初始化失败: {}", 
                init_response.error_message
            )));
        }
        
        // 获取插件信息
        let plugin_info_proto = init_response.plugin_info
            .ok_or_else(|| A2AError::internal("插件信息缺失"))?;
        
        let plugin_info = PluginInfo {
            id: plugin_id.clone(),
            name: plugin_info_proto.name,
            version: plugin_info_proto.version,
            framework: plugin_info_proto.supported_frameworks.first()
                .unwrap_or(&"unknown".to_string()).clone(),
            status: PluginStatus::Running,
            capabilities: init_response.supported_features,
            metadata: plugin_info_proto.metadata,
        };
        
        // 存储插件信息
        self.plugin_clients.write().await.insert(plugin_id.clone(), client);
        self.plugin_info_cache.write().await.insert(plugin_id.clone(), plugin_info);
        
        // 记录监控指标
        self.monitoring_manager.write().await.increment_counter("plugins_registered", 1);
        
        println!("✅ 插件 {} 注册成功", plugin_id);
        Ok(())
    }
    
    /// 注销插件
    pub async fn unregister_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        println!("🔌 注销插件: {}", plugin_id);
        
        // 获取插件客户端
        let mut clients = self.plugin_clients.write().await;
        if let Some(mut client) = clients.remove(plugin_id) {
            // 通知插件关闭
            let _ = client.shutdown(Request::new(())).await;
        }
        
        // 清理插件信息
        self.plugin_info_cache.write().await.remove(plugin_id);
        
        // 清理路由表
        let mut routes = self.message_routes.write().await;
        routes.retain(|_, plugin| plugin != plugin_id);
        
        // 记录监控指标
        self.monitoring_manager.write().await.increment_counter("plugins_unregistered", 1);
        
        println!("✅ 插件 {} 注销成功", plugin_id);
        Ok(())
    }
    
    /// 路由A2A消息到插件
    pub async fn route_message_to_plugin(
        &self,
        message: A2AMessage,
        target_agent_id: &str,
    ) -> A2AResult<Option<A2AMessage>> {
        let start_time = std::time::Instant::now();
        
        // 查找目标Agent对应的插件
        let plugin_id = {
            let routes = self.message_routes.read().await;
            routes.get(target_agent_id).cloned()
        };
        
        let plugin_id = plugin_id.ok_or_else(|| {
            A2AError::agent_not_found(format!("Agent {} 未找到对应插件", target_agent_id))
        })?;
        
        // 获取插件客户端
        let client = {
            let clients = self.plugin_clients.read().await;
            clients.get(&plugin_id).cloned()
        };
        
        let mut client = client.ok_or_else(|| {
            A2AError::internal(format!("插件 {} 客户端未找到", plugin_id))
        })?;
        
        // 转换A2A消息为gRPC格式
        let grpc_request = self.convert_a2a_to_grpc(&message).await?;
        
        // 发送消息到插件
        let response = client.process_a2a_message(Request::new(grpc_request)).await
            .map_err(|e| A2AError::internal(format!("插件处理消息失败: {}", e)))?;
        
        let grpc_response = response.into_inner();
        
        // 记录处理时间
        let processing_time = start_time.elapsed();
        self.monitoring_manager.write().await.record_histogram(
            "plugin_message_processing_time",
            processing_time.as_secs_f64() * 1000.0,
            {
                let mut labels = HashMap::new();
                labels.insert("plugin_id".to_string(), plugin_id);
                labels
            }
        );
        
        // 转换响应
        if grpc_response.success {
            if let Some(response_msg) = grpc_response.response_message {
                let a2a_response = self.convert_grpc_to_a2a(&response_msg).await?;
                Ok(Some(a2a_response))
            } else {
                Ok(None)
            }
        } else {
            Err(A2AError::internal(format!(
                "插件处理失败: {}", 
                grpc_response.error_message
            )))
        }
    }
    
    /// 处理流式消息
    pub async fn handle_stream_message(
        &self,
        stream_chunk: StreamChunk,
        target_agent_id: &str,
    ) -> A2AResult<()> {
        // 查找目标插件
        let plugin_id = {
            let routes = self.message_routes.read().await;
            routes.get(target_agent_id).cloned()
        };
        
        let plugin_id = plugin_id.ok_or_else(|| {
            A2AError::agent_not_found(format!("Agent {} 未找到对应插件", target_agent_id))
        })?;
        
        // 获取插件客户端
        let client = {
            let clients = self.plugin_clients.read().await;
            clients.get(&plugin_id).cloned()
        };
        
        let mut client = client.ok_or_else(|| {
            A2AError::internal(format!("插件 {} 客户端未找到", plugin_id))
        })?;
        
        // 转换流块为gRPC格式
        let grpc_chunk = A2aStreamChunk {
            stream_id: stream_chunk.stream_id,
            sequence: stream_chunk.sequence,
            data: stream_chunk.data,
            is_final: stream_chunk.is_final,
            checksum: stream_chunk.checksum.unwrap_or_default(),
            stream_type: self.convert_stream_type(&StreamType::DataStream), // 简化处理
            metadata: stream_chunk.metadata.into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect(),
        };
        
        // 创建流请求
        let (tx, rx) = mpsc::channel(1);
        tx.send(grpc_chunk).await
            .map_err(|_| A2AError::internal("发送流数据失败"))?;
        drop(tx);
        
        let request_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        
        // 发送流到插件
        let _response = client.process_a2a_stream(Request::new(request_stream)).await
            .map_err(|e| A2AError::internal(format!("插件处理流失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 注册Agent到插件路由
    pub async fn register_agent_route(&self, agent_id: String, plugin_id: String) {
        self.message_routes.write().await.insert(agent_id, plugin_id);
    }
    
    /// 获取插件健康状态
    pub async fn check_plugin_health(&self, plugin_id: &str) -> A2AResult<bool> {
        let client = {
            let clients = self.plugin_clients.read().await;
            clients.get(plugin_id).cloned()
        };
        
        let mut client = client.ok_or_else(|| {
            A2AError::internal(format!("插件 {} 客户端未找到", plugin_id))
        })?;
        
        match client.health_check(Request::new(())).await {
            Ok(response) => {
                let health = response.into_inner();
                Ok(health.status == 1) // SERVING = 1
            },
            Err(_) => Ok(false),
        }
    }
    
    /// 获取所有插件信息
    pub async fn get_all_plugins(&self) -> Vec<PluginInfo> {
        self.plugin_info_cache.read().await.values().cloned().collect()
    }
    
    // 私有辅助方法
    
    async fn convert_a2a_to_grpc(&self, message: &A2AMessage) -> A2AResult<A2aMessageRequest> {
        Ok(A2aMessageRequest {
            message_id: message.message_id.clone(),
            from_agent: "unknown".to_string(), // A2AMessage没有from字段，使用默认值
            to_agent: "unknown".to_string(), // A2AMessage没有to字段，使用默认值
            message_type: 1, // REQUEST
            payload: None, // 简化处理
            metadata: HashMap::new(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            ttl_seconds: 300, // 5分钟TTL
        })
    }
    
    async fn convert_grpc_to_a2a(&self, grpc_msg: &A2aMessageRequest) -> A2AResult<A2AMessage> {
        use agentx_a2a::MessageRole;
        
        Ok(A2AMessage::new_text(
            MessageRole::Agent,
            format!("Response from plugin: {}", grpc_msg.message_id)
        ))
    }
    
    fn convert_stream_type(&self, stream_type: &StreamType) -> i32 {
        match stream_type {
            StreamType::DataStream => 1,
            StreamType::FileStream => 2,
            StreamType::EventStream => 3,
            StreamType::TaskStream => 4,
            StreamType::AudioStream => 5,
            StreamType::VideoStream => 6,
        }
    }
}
