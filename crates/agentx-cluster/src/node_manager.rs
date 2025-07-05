//! 节点管理器
//! 
//! 管理集群中的节点信息、状态和生命周期

use crate::config::NodeConfig;
use crate::error::{ClusterError, ClusterResult};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// 节点状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
    /// 不可达
    Unreachable,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点地址
    pub address: SocketAddr,
    /// 节点状态
    pub status: NodeStatus,
    /// 节点角色
    pub role: NodeRole,
    /// 节点元数据
    pub metadata: std::collections::HashMap<String, String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 最后心跳时间
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// 节点版本
    pub version: String,
    /// 支持的功能
    pub capabilities: Vec<String>,
}

/// 节点角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// 主节点
    Master,
    /// 工作节点
    Worker,
    /// 边缘节点
    Edge,
}

/// 节点管理器
pub struct NodeManager {
    /// 当前节点信息
    current_node: Arc<RwLock<NodeInfo>>,
    /// 集群中的所有节点
    cluster_nodes: Arc<DashMap<String, NodeInfo>>,
    /// 配置
    config: NodeConfig,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl NodeManager {
    /// 创建新的节点管理器
    pub async fn new(config: NodeConfig) -> ClusterResult<Self> {
        let node_id = config.node_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let current_node = NodeInfo {
            id: node_id.clone(),
            name: config.node_name.clone(),
            address: config.bind_address,
            status: NodeStatus::Initializing,
            role: config.role.clone(),
            metadata: config.metadata.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_heartbeat: None,
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: config.capabilities.clone(),
        };
        
        info!("🏗️ 创建节点管理器，节点ID: {}", node_id);
        
        Ok(Self {
            current_node: Arc::new(RwLock::new(current_node)),
            cluster_nodes: Arc::new(DashMap::new()),
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// 启动节点管理器
    pub async fn start(&mut self) -> ClusterResult<()> {
        info!("🚀 启动节点管理器");
        
        // 更新节点状态为运行中
        {
            let mut node = self.current_node.write().await;
            node.status = NodeStatus::Running;
            node.updated_at = chrono::Utc::now();
            node.last_heartbeat = Some(chrono::Utc::now());
        }
        
        // 设置运行状态
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动心跳任务
        self.start_heartbeat_task().await?;
        
        // 启动节点发现任务
        self.start_node_discovery_task().await?;
        
        info!("✅ 节点管理器启动成功");
        Ok(())
    }
    
    /// 停止节点管理器
    pub async fn stop(&mut self) -> ClusterResult<()> {
        info!("🛑 停止节点管理器");
        
        // 设置停止状态
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        // 更新节点状态
        {
            let mut node = self.current_node.write().await;
            node.status = NodeStatus::Stopped;
            node.updated_at = chrono::Utc::now();
        }
        
        info!("✅ 节点管理器已停止");
        Ok(())
    }
    
    /// 获取当前节点信息
    pub async fn get_node_info(&self) -> ClusterResult<NodeInfo> {
        let node = self.current_node.read().await;
        Ok(node.clone())
    }
    
    /// 更新当前节点状态
    pub async fn update_node_status(&self, status: NodeStatus) -> ClusterResult<()> {
        let mut node = self.current_node.write().await;
        node.status = status;
        node.updated_at = chrono::Utc::now();
        
        debug!("更新节点状态: {:?}", node.status);
        Ok(())
    }
    
    /// 添加集群节点
    pub async fn add_cluster_node(&self, node_info: NodeInfo) -> ClusterResult<()> {
        let node_id = node_info.id.clone();
        self.cluster_nodes.insert(node_id.clone(), node_info);
        
        debug!("添加集群节点: {}", node_id);
        Ok(())
    }
    
    /// 移除集群节点
    pub async fn remove_cluster_node(&self, node_id: &str) -> ClusterResult<()> {
        self.cluster_nodes.remove(node_id);
        
        debug!("移除集群节点: {}", node_id);
        Ok(())
    }
    
    /// 列出所有集群节点
    pub async fn list_nodes(&self) -> ClusterResult<Vec<NodeInfo>> {
        let mut nodes = Vec::new();
        
        // 添加当前节点
        let current_node = self.current_node.read().await;
        nodes.push(current_node.clone());
        
        // 添加集群中的其他节点
        for entry in self.cluster_nodes.iter() {
            nodes.push(entry.value().clone());
        }
        
        Ok(nodes)
    }
    
    /// 获取指定节点信息
    pub async fn get_cluster_node(&self, node_id: &str) -> ClusterResult<Option<NodeInfo>> {
        // 检查是否是当前节点
        {
            let current_node = self.current_node.read().await;
            if current_node.id == node_id {
                return Ok(Some(current_node.clone()));
            }
        }
        
        // 检查集群节点
        if let Some(node) = self.cluster_nodes.get(node_id) {
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// 更新节点心跳
    pub async fn update_heartbeat(&self, node_id: &str) -> ClusterResult<()> {
        let now = chrono::Utc::now();
        
        // 检查是否是当前节点
        {
            let mut current_node = self.current_node.write().await;
            if current_node.id == node_id {
                current_node.last_heartbeat = Some(now);
                current_node.updated_at = now;
                return Ok(());
            }
        }
        
        // 更新集群节点心跳
        if let Some(mut node) = self.cluster_nodes.get_mut(node_id) {
            node.last_heartbeat = Some(now);
            node.updated_at = now;
        }
        
        Ok(())
    }
    
    /// 检查节点是否健康
    pub async fn is_node_healthy(&self, node_id: &str, timeout_seconds: u64) -> ClusterResult<bool> {
        if let Some(node) = self.get_cluster_node(node_id).await? {
            if let Some(last_heartbeat) = node.last_heartbeat {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last_heartbeat);
                
                return Ok(duration.num_seconds() < timeout_seconds as i64);
            }
        }
        
        Ok(false)
    }
    
    /// 启动心跳任务
    async fn start_heartbeat_task(&self) -> ClusterResult<()> {
        let current_node = self.current_node.clone();
        let running = self.running.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 更新心跳时间
                {
                    let mut node = current_node.write().await;
                    node.last_heartbeat = Some(chrono::Utc::now());
                    node.updated_at = chrono::Utc::now();
                }
                
                debug!("💓 节点心跳更新");
            }
        });
        
        Ok(())
    }
    
    /// 启动节点发现任务
    async fn start_node_discovery_task(&self) -> ClusterResult<()> {
        let cluster_nodes = self.cluster_nodes.clone();
        let running = self.running.clone();
        let discovery_interval = self.config.discovery_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(discovery_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 执行节点发现逻辑
                // TODO: 实现具体的节点发现机制
                debug!("🔍 执行节点发现");
                
                // 清理不健康的节点
                let mut to_remove = Vec::new();
                for entry in cluster_nodes.iter() {
                    let node = entry.value();
                    if let Some(last_heartbeat) = node.last_heartbeat {
                        let now = chrono::Utc::now();
                        let duration = now.signed_duration_since(last_heartbeat);
                        
                        // 如果超过5分钟没有心跳，认为节点不健康
                        if duration.num_seconds() > 300 {
                            to_remove.push(node.id.clone());
                        }
                    }
                }
                
                for node_id in to_remove {
                    cluster_nodes.remove(&node_id);
                    warn!("移除不健康的节点: {}", node_id);
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_node_manager_creation() {
        let config = NodeConfig::default();
        let node_manager = NodeManager::new(config).await.unwrap();
        
        let node_info = node_manager.get_node_info().await.unwrap();
        assert_eq!(node_info.status, NodeStatus::Initializing);
        assert!(!node_info.id.is_empty());
    }
    
    #[tokio::test]
    async fn test_node_status_update() {
        let config = NodeConfig::default();
        let node_manager = NodeManager::new(config).await.unwrap();
        
        // 更新状态
        node_manager.update_node_status(NodeStatus::Running).await.unwrap();
        
        let node_info = node_manager.get_node_info().await.unwrap();
        assert_eq!(node_info.status, NodeStatus::Running);
    }
    
    #[tokio::test]
    async fn test_cluster_node_management() {
        let config = NodeConfig::default();
        let node_manager = NodeManager::new(config).await.unwrap();
        
        // 创建测试节点
        let test_node = NodeInfo {
            id: "test-node".to_string(),
            name: "Test Node".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            status: NodeStatus::Running,
            role: NodeRole::Worker,
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_heartbeat: Some(chrono::Utc::now()),
            version: "0.1.0".to_string(),
            capabilities: vec!["test".to_string()],
        };
        
        // 添加节点
        node_manager.add_cluster_node(test_node.clone()).await.unwrap();
        
        // 验证节点存在
        let retrieved_node = node_manager.get_cluster_node("test-node").await.unwrap();
        assert!(retrieved_node.is_some());
        assert_eq!(retrieved_node.unwrap().id, "test-node");
        
        // 列出所有节点
        let nodes = node_manager.list_nodes().await.unwrap();
        assert_eq!(nodes.len(), 2); // 当前节点 + 测试节点
        
        // 移除节点
        node_manager.remove_cluster_node("test-node").await.unwrap();
        
        let retrieved_node = node_manager.get_cluster_node("test-node").await.unwrap();
        assert!(retrieved_node.is_none());
    }
    
    #[tokio::test]
    async fn test_heartbeat_update() {
        let config = NodeConfig::default();
        let node_manager = NodeManager::new(config).await.unwrap();
        
        let node_info = node_manager.get_node_info().await.unwrap();
        let node_id = node_info.id.clone();
        
        // 更新心跳
        node_manager.update_heartbeat(&node_id).await.unwrap();
        
        let updated_node = node_manager.get_node_info().await.unwrap();
        assert!(updated_node.last_heartbeat.is_some());
    }
    
    #[tokio::test]
    async fn test_node_health_check() {
        let config = NodeConfig::default();
        let node_manager = NodeManager::new(config).await.unwrap();
        
        let node_info = node_manager.get_node_info().await.unwrap();
        let node_id = node_info.id.clone();
        
        // 更新心跳
        node_manager.update_heartbeat(&node_id).await.unwrap();
        
        // 检查健康状态（30秒超时）
        let is_healthy = node_manager.is_node_healthy(&node_id, 30).await.unwrap();
        assert!(is_healthy);
        
        // 检查不存在的节点
        let is_healthy = node_manager.is_node_healthy("nonexistent", 30).await.unwrap();
        assert!(!is_healthy);
    }
}
