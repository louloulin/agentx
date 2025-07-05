//! gRPC客户端实现
//! 
//! 实现AgentX的gRPC客户端，用于连接和管理远程插件

use crate::proto::{
    agent_x_plugin_client::AgentXPluginClient,
    A2aMessageRequest, PluginInfoResponse,
};
use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::Request;
use tracing::{debug, error, info, warn};

/// gRPC客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 连接超时时间（秒）
    pub connect_timeout_seconds: u64,
    /// 请求超时时间（秒）
    pub request_timeout_seconds: u64,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 启用TLS
    pub enable_tls: bool,
    /// TLS证书路径
    pub tls_cert_path: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout_seconds: 10,
            request_timeout_seconds: 30,
            max_retries: 3,
            retry_interval_ms: 1000,
            enable_tls: false,
            tls_cert_path: None,
        }
    }
}

/// 连接状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    /// 未连接
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 连接失败
    Failed(String),
}

/// 插件连接信息
#[derive(Debug, Clone)]
pub struct PluginConnection {
    /// 插件ID
    pub plugin_id: String,
    /// 连接地址
    pub endpoint: String,
    /// 连接状态
    pub status: ConnectionStatus,
    /// 最后心跳时间
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// 连接统计
    pub stats: ConnectionStats,
}

/// 连接统计信息
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 连接建立时间
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// AgentX gRPC客户端
pub struct AgentXGrpcClient {
    /// 客户端配置
    config: ClientConfig,
    /// 活跃连接
    connections: Arc<RwLock<HashMap<String, PluginConnection>>>,
    /// gRPC客户端池
    client_pool: Arc<RwLock<HashMap<String, AgentXPluginClient<tonic::transport::Channel>>>>,
}

impl AgentXGrpcClient {
    /// 创建新的gRPC客户端
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            client_pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 连接到插件
    pub async fn connect_to_plugin(&self, plugin_id: String, endpoint: String) -> A2AResult<()> {
        info!("🔌 连接到插件: {} ({})", plugin_id, endpoint);

        // 更新连接状态为连接中
        {
            let mut connections = self.connections.write().await;
            connections.insert(plugin_id.clone(), PluginConnection {
                plugin_id: plugin_id.clone(),
                endpoint: endpoint.clone(),
                status: ConnectionStatus::Connecting,
                last_heartbeat: chrono::Utc::now(),
                stats: ConnectionStats::default(),
            });
        }

        // 建立gRPC连接
        let channel = match self.create_channel(&endpoint).await {
            Ok(channel) => channel,
            Err(e) => {
                error!("❌ 连接插件 {} 失败: {}", plugin_id, e);
                self.update_connection_status(&plugin_id, ConnectionStatus::Failed(e.to_string())).await;
                return Err(A2AError::internal(format!("连接插件失败: {}", e)));
            }
        };

        let client = AgentXPluginClient::new(channel);

        // 测试连接
        let mut test_client = client.clone();
        match test_client.health_check(Request::new(())).await {
            Ok(_) => {
                info!("✅ 插件 {} 连接成功", plugin_id);
                
                // 保存客户端
                self.client_pool.write().await.insert(plugin_id.clone(), client);
                
                // 更新连接状态
                self.update_connection_status(&plugin_id, ConnectionStatus::Connected).await;
                
                // 更新连接统计
                {
                    let mut connections = self.connections.write().await;
                    if let Some(conn) = connections.get_mut(&plugin_id) {
                        conn.stats.connected_at = Some(chrono::Utc::now());
                    }
                }
                
                Ok(())
            },
            Err(e) => {
                error!("❌ 插件 {} 健康检查失败: {}", plugin_id, e);
                self.update_connection_status(&plugin_id, ConnectionStatus::Failed(e.to_string())).await;
                Err(A2AError::internal(format!("插件健康检查失败: {}", e)))
            }
        }
    }

    /// 断开插件连接
    pub async fn disconnect_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        info!("🔌 断开插件连接: {}", plugin_id);

        // 移除客户端
        self.client_pool.write().await.remove(plugin_id);
        
        // 更新连接状态
        self.update_connection_status(plugin_id, ConnectionStatus::Disconnected).await;
        
        Ok(())
    }

    /// 发送A2A消息到插件
    pub async fn send_message_to_plugin(
        &self,
        plugin_id: &str,
        message: A2AMessage,
    ) -> A2AResult<A2AMessage> {
        debug!("📤 发送消息到插件: {}", plugin_id);

        let start_time = std::time::Instant::now();
        
        // 获取客户端
        let mut client = {
            let clients = self.client_pool.read().await;
            clients.get(plugin_id).cloned()
        }.ok_or_else(|| A2AError::internal(format!("插件 {} 未连接", plugin_id)))?;

        // 转换消息格式
        let request = self.convert_a2a_to_grpc_request(message)?;

        // 发送请求
        let response = match client.process_a2a_message(Request::new(request)).await {
            Ok(response) => response.into_inner(),
            Err(e) => {
                error!("❌ 插件 {} 消息处理失败: {}", plugin_id, e);
                self.update_connection_stats(plugin_id, false, start_time.elapsed().as_millis() as f64).await;
                return Err(A2AError::internal(format!("消息处理失败: {}", e)));
            }
        };

        // 更新统计信息
        self.update_connection_stats(plugin_id, true, start_time.elapsed().as_millis() as f64).await;

        // 转换响应
        if response.success {
            if let Some(response_msg) = response.response_message {
                self.convert_grpc_response_to_a2a(response_msg)
            } else {
                Err(A2AError::internal("插件响应消息为空"))
            }
        } else {
            Err(A2AError::internal(format!("插件处理失败: {}", response.error_message)))
        }
    }

    /// 获取插件信息
    pub async fn get_plugin_info(&self, plugin_id: &str) -> A2AResult<PluginInfoResponse> {
        debug!("📋 获取插件信息: {}", plugin_id);

        let mut client = {
            let clients = self.client_pool.read().await;
            clients.get(plugin_id).cloned()
        }.ok_or_else(|| A2AError::internal(format!("插件 {} 未连接", plugin_id)))?;

        match client.get_plugin_info(Request::new(())).await {
            Ok(response) => Ok(response.into_inner()),
            Err(e) => {
                error!("❌ 获取插件 {} 信息失败: {}", plugin_id, e);
                Err(A2AError::internal(format!("获取插件信息失败: {}", e)))
            }
        }
    }

    /// 检查插件健康状态
    pub async fn check_plugin_health(&self, plugin_id: &str) -> A2AResult<bool> {
        debug!("🏥 检查插件健康状态: {}", plugin_id);

        let mut client = {
            let clients = self.client_pool.read().await;
            clients.get(plugin_id).cloned()
        }.ok_or_else(|| A2AError::internal(format!("插件 {} 未连接", plugin_id)))?;

        match client.health_check(Request::new(())).await {
            Ok(response) => {
                let health = response.into_inner();
                let is_healthy = health.status == 1; // SERVING = 1
                
                // 更新心跳时间
                {
                    let mut connections = self.connections.write().await;
                    if let Some(conn) = connections.get_mut(plugin_id) {
                        conn.last_heartbeat = chrono::Utc::now();
                    }
                }
                
                Ok(is_healthy)
            },
            Err(e) => {
                warn!("⚠️ 插件 {} 健康检查失败: {}", plugin_id, e);
                Ok(false)
            }
        }
    }

    /// 获取所有连接状态
    pub async fn get_all_connections(&self) -> Vec<PluginConnection> {
        self.connections.read().await.values().cloned().collect()
    }

    /// 获取连接统计信息
    pub async fn get_connection_stats(&self, plugin_id: &str) -> Option<ConnectionStats> {
        let connections = self.connections.read().await;
        connections.get(plugin_id).map(|conn| conn.stats.clone())
    }

    // 私有辅助方法

    /// 创建gRPC通道
    async fn create_channel(&self, endpoint: &str) -> Result<tonic::transport::Channel, tonic::transport::Error> {
        let mut endpoint = tonic::transport::Endpoint::from_shared(endpoint.to_string())?;

        // 设置超时
        endpoint = endpoint
            .connect_timeout(std::time::Duration::from_secs(self.config.connect_timeout_seconds))
            .timeout(std::time::Duration::from_secs(self.config.request_timeout_seconds));

        // 暂时禁用TLS配置，因为需要额外的feature
        // TODO: 在需要TLS时启用tonic的tls feature
        if self.config.enable_tls {
            eprintln!("⚠️ TLS配置暂时不支持，需要启用tonic的tls feature");
        }

        endpoint.connect().await
    }

    /// 更新连接状态
    async fn update_connection_status(&self, plugin_id: &str, status: ConnectionStatus) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(plugin_id) {
            conn.status = status;
        }
    }

    /// 更新连接统计信息
    async fn update_connection_stats(&self, plugin_id: &str, success: bool, response_time_ms: f64) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(plugin_id) {
            conn.stats.total_requests += 1;
            if success {
                conn.stats.successful_requests += 1;
            } else {
                conn.stats.failed_requests += 1;
            }
            
            // 更新平均响应时间
            let total_time = conn.stats.avg_response_time_ms * (conn.stats.total_requests - 1) as f64 + response_time_ms;
            conn.stats.avg_response_time_ms = total_time / conn.stats.total_requests as f64;
        }
    }

    /// 转换A2A消息为gRPC请求
    fn convert_a2a_to_grpc_request(&self, message: A2AMessage) -> A2AResult<A2aMessageRequest> {
        use crate::converter::A2AConverter;
        A2AConverter::a2a_to_grpc_request(&message)
            .map_err(|e| A2AError::internal(format!("转换失败: {}", e)))
    }

    /// 转换gRPC响应为A2A消息
    fn convert_grpc_response_to_a2a(&self, response: A2aMessageRequest) -> A2AResult<A2AMessage> {
        use crate::converter::A2AConverter;
        A2AConverter::grpc_response_to_a2a(response)
            .map_err(|e| A2AError::internal(format!("转换失败: {}", e)))
    }
}
