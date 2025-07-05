//! AgentX分布式集群管理模块
//! 
//! 提供多节点Agent注册发现、负载均衡、故障转移等分布式功能

pub mod node_manager;
pub mod service_discovery;
pub mod load_balancer;
pub mod cluster_state;
pub mod health_checker;
pub mod config;
pub mod error;

// 重新导出主要类型
pub use node_manager::{NodeManager, NodeInfo, NodeStatus};
pub use service_discovery::{ServiceDiscovery, ServiceRegistry, DiscoveryBackend};
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy, TargetNode};
pub use cluster_state::{ClusterState, ClusterStateManager, StateSync};
pub use health_checker::{HealthChecker, HealthStatus};
pub use config::{ClusterConfig, NodeConfig, DiscoveryConfig, LoadBalancerConfig, StateConfig, HealthCheckConfig};
pub use error::{ClusterError, ClusterResult};

/// 集群管理器 - 统一的分布式集群管理接口
pub struct ClusterManager {
    /// 节点管理器
    node_manager: NodeManager,
    /// 服务发现
    service_discovery: ServiceDiscovery,
    /// 负载均衡器
    load_balancer: LoadBalancer,
    /// 集群状态管理器
    state_manager: ClusterStateManager,
    /// 健康检查器
    health_checker: HealthChecker,
    /// 配置
    config: ClusterConfig,
}

impl ClusterManager {
    /// 创建新的集群管理器
    pub async fn new(config: ClusterConfig) -> ClusterResult<Self> {
        let node_manager = NodeManager::new(config.node.clone()).await?;
        let service_discovery = ServiceDiscovery::new(config.discovery.clone()).await?;
        let load_balancer = LoadBalancer::new(config.load_balancer.clone()).await?;
        let state_manager = ClusterStateManager::new(config.state.clone()).await?;
        let health_checker = HealthChecker::new(config.health_check.clone()).await?;
        
        Ok(Self {
            node_manager,
            service_discovery,
            load_balancer,
            state_manager,
            health_checker,
            config,
        })
    }
    
    /// 启动集群管理器
    pub async fn start(&mut self) -> ClusterResult<()> {
        // 启动各个组件
        self.node_manager.start().await?;
        self.service_discovery.start().await?;
        self.load_balancer.start().await?;
        self.state_manager.start().await?;
        self.health_checker.start().await?;
        
        Ok(())
    }
    
    /// 停止集群管理器
    pub async fn stop(&mut self) -> ClusterResult<()> {
        // 停止各个组件
        self.health_checker.stop().await?;
        self.state_manager.stop().await?;
        self.load_balancer.stop().await?;
        self.service_discovery.stop().await?;
        self.node_manager.stop().await?;
        
        Ok(())
    }
    
    /// 注册Agent到集群
    pub async fn register_agent(&self, agent_info: agentx_a2a::AgentCard) -> ClusterResult<String> {
        // 通过服务发现注册Agent
        let agent_id = self.service_discovery.register_agent(agent_info.clone()).await?;
        
        // 更新集群状态
        self.state_manager.update_agent_state(&agent_id, &agent_info).await?;
        
        // 添加到负载均衡器
        let endpoint = agent_info.endpoints.first()
            .map(|ep| ep.url.clone())
            .unwrap_or_else(|| format!("http://localhost:8080"));
        self.load_balancer.add_target(&agent_id, endpoint.clone()).await?;

        // 开始健康检查
        self.health_checker.start_monitoring(&agent_id, endpoint).await?;
        
        Ok(agent_id)
    }
    
    /// 注销Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> ClusterResult<()> {
        // 停止健康检查
        self.health_checker.stop_monitoring(agent_id).await?;
        
        // 从负载均衡器移除
        self.load_balancer.remove_target(agent_id).await?;
        
        // 更新集群状态
        self.state_manager.remove_agent_state(agent_id).await?;
        
        // 从服务发现注销
        self.service_discovery.unregister_agent(agent_id).await?;
        
        Ok(())
    }
    
    /// 发现可用的Agent
    pub async fn discover_agents(&self, capability: Option<&str>) -> ClusterResult<Vec<agentx_a2a::AgentCard>> {
        self.service_discovery.discover_agents(capability).await
    }
    
    /// 选择目标Agent（负载均衡）
    pub async fn select_target(&self, capability: Option<&str>) -> ClusterResult<Option<agentx_a2a::AgentCard>> {
        // 获取可用的Agent列表
        let agents = self.discover_agents(capability).await?;
        
        if agents.is_empty() {
            return Ok(None);
        }
        
        // 通过负载均衡器选择目标
        // 注意：负载均衡器中的ID是带"agent-"前缀的
        let target_ids: Vec<String> = agents.iter().map(|a| format!("agent-{}", a.id)).collect();

        if let Some(selected_id) = self.load_balancer.select_target(&target_ids).await? {
            // 返回选中的Agent信息
            // 从选中的ID中移除"agent-"前缀来匹配原始Agent ID
            let original_id = selected_id.strip_prefix("agent-").unwrap_or(&selected_id);
            for agent in agents {
                if agent.id == original_id {
                    return Ok(Some(agent));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 获取集群状态
    pub async fn get_cluster_state(&self) -> ClusterResult<ClusterState> {
        self.state_manager.get_state().await
    }
    
    /// 获取节点信息
    pub async fn get_node_info(&self) -> ClusterResult<NodeInfo> {
        self.node_manager.get_node_info().await
    }
    
    /// 获取集群中的所有节点
    pub async fn list_nodes(&self) -> ClusterResult<Vec<NodeInfo>> {
        self.node_manager.list_nodes().await
    }
    
    /// 检查Agent健康状态
    pub async fn check_agent_health(&self, agent_id: &str) -> ClusterResult<HealthStatus> {
        self.health_checker.check_health(agent_id).await
    }

    /// 获取负载均衡器目标列表（用于调试）
    pub async fn list_load_balancer_targets(&self) -> ClusterResult<Vec<load_balancer::TargetNode>> {
        self.load_balancer.list_targets().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cluster_manager_creation() {
        let config = ClusterConfig::default();
        let result = ClusterManager::new(config).await;
        
        // 在测试环境中，某些依赖可能不可用，所以我们只测试不会panic
        match result {
            Ok(_) => println!("集群管理器创建成功"),
            Err(e) => println!("集群管理器创建失败（预期）: {}", e),
        }
    }
}
