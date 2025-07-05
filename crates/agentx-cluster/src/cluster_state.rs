//! 集群状态管理
//! 
//! 管理集群的全局状态和状态同步

use crate::config::StateConfig;
use crate::error::{ClusterError, ClusterResult};
use agentx_a2a::AgentCard;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// 集群状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    /// 集群ID
    pub cluster_id: String,
    /// 集群名称
    pub cluster_name: String,
    /// 集群版本
    pub version: String,
    /// 节点数量
    pub node_count: usize,
    /// Agent数量
    pub agent_count: usize,
    /// 集群状态
    pub status: ClusterStatus,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 元数据
    pub metadata: std::collections::HashMap<String, String>,
}

/// 集群状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterStatus {
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 降级运行
    Degraded,
    /// 维护中
    Maintenance,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// Agent状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateInfo {
    /// Agent信息
    pub agent_info: AgentCard,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// 最后心跳时间
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    /// 统计信息
    pub stats: AgentStats,
}

/// Agent统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    /// 处理的消息数
    pub messages_processed: u64,
    /// 错误数
    pub errors: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: u64,
    /// 最后活动时间
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for AgentStats {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            errors: 0,
            avg_response_time: 0,
            last_activity: None,
        }
    }
}

/// 状态同步接口
#[async_trait::async_trait]
pub trait StateSync: Send + Sync {
    /// 同步状态到其他节点
    async fn sync_state(&self, state: &ClusterState) -> ClusterResult<()>;
    
    /// 从其他节点获取状态
    async fn fetch_state(&self) -> ClusterResult<Option<ClusterState>>;
    
    /// 监听状态变化
    async fn watch_state_changes(&self) -> ClusterResult<tokio::sync::mpsc::Receiver<ClusterState>>;
}

/// 内存状态同步实现（用于测试）
pub struct MemoryStateSync {
    state: Arc<RwLock<Option<ClusterState>>>,
}

impl MemoryStateSync {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl StateSync for MemoryStateSync {
    async fn sync_state(&self, state: &ClusterState) -> ClusterResult<()> {
        let mut stored_state = self.state.write().await;
        *stored_state = Some(state.clone());
        
        debug!("同步集群状态: {}", state.cluster_id);
        Ok(())
    }
    
    async fn fetch_state(&self) -> ClusterResult<Option<ClusterState>> {
        let state = self.state.read().await;
        Ok(state.clone())
    }
    
    async fn watch_state_changes(&self) -> ClusterResult<tokio::sync::mpsc::Receiver<ClusterState>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // TODO: 实现状态变化监听
        // 这里只是一个占位实现
        
        Ok(rx)
    }
}

/// 集群状态管理器
pub struct ClusterStateManager {
    /// 当前集群状态
    cluster_state: Arc<RwLock<ClusterState>>,
    /// Agent状态
    agent_states: Arc<DashMap<String, AgentStateInfo>>,
    /// 状态同步器
    state_sync: Box<dyn StateSync>,
    /// 配置
    config: StateConfig,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl ClusterStateManager {
    /// 创建新的集群状态管理器
    pub async fn new(config: StateConfig) -> ClusterResult<Self> {
        let cluster_state = ClusterState {
            cluster_id: config.cluster_id.clone(),
            cluster_name: config.cluster_name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            node_count: 1,
            agent_count: 0,
            status: ClusterStatus::Initializing,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: config.metadata.clone(),
        };
        
        // 创建状态同步器
        let state_sync: Box<dyn StateSync> = match config.sync_backend.as_str() {
            "memory" => Box::new(MemoryStateSync::new()),
            _ => {
                return Err(ClusterError::UnsupportedBackend(
                    format!("不支持的状态同步后端: {}", config.sync_backend)
                ));
            }
        };
        
        info!("🗂️ 创建集群状态管理器，集群ID: {}", cluster_state.cluster_id);
        
        Ok(Self {
            cluster_state: Arc::new(RwLock::new(cluster_state)),
            agent_states: Arc::new(DashMap::new()),
            state_sync,
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// 启动状态管理器
    pub async fn start(&mut self) -> ClusterResult<()> {
        info!("🚀 启动集群状态管理器");
        
        // 更新集群状态
        {
            let mut state = self.cluster_state.write().await;
            state.status = ClusterStatus::Running;
            state.updated_at = chrono::Utc::now();
        }
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动状态同步任务
        self.start_sync_task().await?;
        
        // 启动统计更新任务
        self.start_stats_task().await?;
        
        info!("✅ 集群状态管理器启动成功");
        Ok(())
    }
    
    /// 停止状态管理器
    pub async fn stop(&mut self) -> ClusterResult<()> {
        info!("🛑 停止集群状态管理器");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        // 更新集群状态
        {
            let mut state = self.cluster_state.write().await;
            state.status = ClusterStatus::Stopped;
            state.updated_at = chrono::Utc::now();
        }
        
        info!("✅ 集群状态管理器已停止");
        Ok(())
    }
    
    /// 获取集群状态
    pub async fn get_state(&self) -> ClusterResult<ClusterState> {
        let state = self.cluster_state.read().await;
        Ok(state.clone())
    }
    
    /// 更新Agent状态
    pub async fn update_agent_state(&self, agent_id: &str, agent_info: &AgentCard) -> ClusterResult<()> {
        let now = chrono::Utc::now();
        
        if let Some(mut state_info) = self.agent_states.get_mut(agent_id) {
            // 更新现有Agent状态
            state_info.agent_info = agent_info.clone();
            state_info.updated_at = now;
            state_info.last_heartbeat = Some(now);
        } else {
            // 添加新Agent状态
            let state_info = AgentStateInfo {
                agent_info: agent_info.clone(),
                registered_at: now,
                updated_at: now,
                last_heartbeat: Some(now),
                stats: AgentStats::default(),
            };
            
            self.agent_states.insert(agent_id.to_string(), state_info);
            
            // 更新集群Agent数量
            {
                let mut cluster_state = self.cluster_state.write().await;
                cluster_state.agent_count = self.agent_states.len();
                cluster_state.updated_at = now;
            }
        }
        
        debug!("更新Agent状态: {}", agent_id);
        Ok(())
    }
    
    /// 移除Agent状态
    pub async fn remove_agent_state(&self, agent_id: &str) -> ClusterResult<()> {
        self.agent_states.remove(agent_id);
        
        // 更新集群Agent数量
        {
            let mut cluster_state = self.cluster_state.write().await;
            cluster_state.agent_count = self.agent_states.len();
            cluster_state.updated_at = chrono::Utc::now();
        }
        
        debug!("移除Agent状态: {}", agent_id);
        Ok(())
    }
    
    /// 获取Agent状态
    pub async fn get_agent_state(&self, agent_id: &str) -> ClusterResult<Option<AgentStateInfo>> {
        if let Some(state_info) = self.agent_states.get(agent_id) {
            Ok(Some(state_info.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// 列出所有Agent状态
    pub async fn list_agent_states(&self) -> ClusterResult<Vec<AgentStateInfo>> {
        let states: Vec<AgentStateInfo> = self.agent_states.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        Ok(states)
    }
    
    /// 更新Agent统计信息
    pub async fn update_agent_stats(&self, agent_id: &str, stats: AgentStats) -> ClusterResult<()> {
        if let Some(mut state_info) = self.agent_states.get_mut(agent_id) {
            state_info.stats = stats;
            state_info.updated_at = chrono::Utc::now();
            
            debug!("更新Agent统计: {}", agent_id);
        }
        
        Ok(())
    }
    
    /// 更新集群状态
    pub async fn update_cluster_status(&self, status: ClusterStatus) -> ClusterResult<()> {
        {
            let mut cluster_state = self.cluster_state.write().await;
            cluster_state.status = status;
            cluster_state.updated_at = chrono::Utc::now();
        }
        
        // 同步状态到其他节点
        let state = self.get_state().await?;
        self.state_sync.sync_state(&state).await?;
        
        debug!("更新集群状态: {:?}", state.status);
        Ok(())
    }
    
    /// 启动状态同步任务
    async fn start_sync_task(&self) -> ClusterResult<()> {
        let cluster_state = self.cluster_state.clone();
        let running = self.running.clone();
        let sync_interval = self.config.sync_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(sync_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 执行状态同步
                debug!("🔄 执行集群状态同步");
                
                // TODO: 实现具体的状态同步逻辑
                // 1. 收集本地状态
                // 2. 同步到其他节点
                // 3. 从其他节点获取状态
                // 4. 合并状态信息
            }
        });
        
        Ok(())
    }
    
    /// 启动统计更新任务
    async fn start_stats_task(&self) -> ClusterResult<()> {
        let agent_states = self.agent_states.clone();
        let cluster_state = self.cluster_state.clone();
        let running = self.running.clone();
        let stats_interval = self.config.stats_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(stats_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 更新统计信息
                debug!("📊 更新集群统计信息");
                
                // 清理过期的Agent状态
                let now = chrono::Utc::now();
                let mut expired_agents = Vec::new();
                
                for entry in agent_states.iter() {
                    let state_info = entry.value();
                    if let Some(last_heartbeat) = state_info.last_heartbeat {
                        let duration = now.signed_duration_since(last_heartbeat);
                        
                        // 如果超过5分钟没有心跳，认为Agent已离线
                        if duration.num_seconds() > 300 {
                            expired_agents.push(entry.key().clone());
                        }
                    }
                }
                
                for agent_id in expired_agents {
                    agent_states.remove(&agent_id);
                    warn!("移除过期Agent状态: {}", agent_id);
                }
                
                // 更新集群统计
                {
                    let mut state = cluster_state.write().await;
                    state.agent_count = agent_states.len();
                    state.updated_at = now;
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    fn create_test_agent(id: &str) -> AgentCard {
        use agentx_a2a::{AgentStatus, TrustLevel, InteractionModality};

        AgentCard {
            id: id.to_string(),
            name: format!("Test Agent {}", id),
            description: "Test agent for cluster state".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            status: AgentStatus::Online,
            supported_versions: vec!["1.0.0".to_string()],
            tags: vec!["test".to_string()],
            interaction_modalities: vec![InteractionModality::Text],
            ux_capabilities: None,
            trust_level: TrustLevel::Trusted,
            supported_task_types: vec!["test".to_string()],
        }
    }
    
    #[tokio::test]
    async fn test_cluster_state_manager_creation() {
        let config = StateConfig::default();
        let state_manager = ClusterStateManager::new(config).await.unwrap();
        
        let state = state_manager.get_state().await.unwrap();
        assert_eq!(state.status, ClusterStatus::Initializing);
        assert_eq!(state.agent_count, 0);
    }
    
    #[tokio::test]
    async fn test_agent_state_management() {
        let config = StateConfig::default();
        let state_manager = ClusterStateManager::new(config).await.unwrap();
        
        let agent = create_test_agent("test1");
        
        // 更新Agent状态
        state_manager.update_agent_state("test1", &agent).await.unwrap();
        
        // 验证Agent状态
        let agent_state = state_manager.get_agent_state("test1").await.unwrap();
        assert!(agent_state.is_some());
        assert_eq!(agent_state.unwrap().agent_info.id, "test1");
        
        // 验证集群状态更新
        let cluster_state = state_manager.get_state().await.unwrap();
        assert_eq!(cluster_state.agent_count, 1);
        
        // 列出所有Agent状态
        let agent_states = state_manager.list_agent_states().await.unwrap();
        assert_eq!(agent_states.len(), 1);
        
        // 移除Agent状态
        state_manager.remove_agent_state("test1").await.unwrap();
        
        let agent_state = state_manager.get_agent_state("test1").await.unwrap();
        assert!(agent_state.is_none());
        
        let cluster_state = state_manager.get_state().await.unwrap();
        assert_eq!(cluster_state.agent_count, 0);
    }
    
    #[tokio::test]
    async fn test_cluster_status_update() {
        let config = StateConfig::default();
        let state_manager = ClusterStateManager::new(config).await.unwrap();
        
        // 更新集群状态
        state_manager.update_cluster_status(ClusterStatus::Running).await.unwrap();
        
        let state = state_manager.get_state().await.unwrap();
        assert_eq!(state.status, ClusterStatus::Running);
    }
    
    #[tokio::test]
    async fn test_agent_stats_update() {
        let config = StateConfig::default();
        let state_manager = ClusterStateManager::new(config).await.unwrap();
        
        let agent = create_test_agent("test1");
        state_manager.update_agent_state("test1", &agent).await.unwrap();
        
        // 更新统计信息
        let stats = AgentStats {
            messages_processed: 100,
            errors: 5,
            avg_response_time: 50,
            last_activity: Some(chrono::Utc::now()),
        };
        
        state_manager.update_agent_stats("test1", stats.clone()).await.unwrap();
        
        let agent_state = state_manager.get_agent_state("test1").await.unwrap();
        assert!(agent_state.is_some());
        assert_eq!(agent_state.unwrap().stats.messages_processed, 100);
    }
}
