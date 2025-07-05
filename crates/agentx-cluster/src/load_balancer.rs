//! 负载均衡器
//! 
//! 提供多种负载均衡策略和目标选择算法

use crate::config::LoadBalancerConfig;
use crate::error::ClusterResult;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// 负载均衡策略
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接
    LeastConnections,
    /// 加权轮询
    WeightedRoundRobin,
    /// 一致性哈希
    ConsistentHash,
    /// 最少响应时间
    LeastResponseTime,
}

/// 目标节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetNode {
    /// 节点ID
    pub id: String,
    /// 节点地址
    pub endpoint: String,
    /// 权重
    pub weight: u32,
    /// 当前连接数
    pub connections: u32,
    /// 平均响应时间（毫秒）
    pub avg_response_time: u64,
    /// 是否健康
    pub healthy: bool,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 负载均衡器
pub struct LoadBalancer {
    /// 目标节点
    targets: Arc<DashMap<String, TargetNode>>,
    /// 负载均衡策略
    strategy: LoadBalancingStrategy,
    /// 轮询计数器
    round_robin_counter: Arc<AtomicUsize>,
    /// 配置
    config: LoadBalancerConfig,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl LoadBalancer {
    /// 创建新的负载均衡器
    pub async fn new(config: LoadBalancerConfig) -> ClusterResult<Self> {
        info!("⚖️ 创建负载均衡器，策略: {:?}", config.strategy);
        
        Ok(Self {
            targets: Arc::new(DashMap::new()),
            strategy: config.strategy.clone(),
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// 启动负载均衡器
    pub async fn start(&mut self) -> ClusterResult<()> {
        info!("🚀 启动负载均衡器");
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动统计更新任务
        self.start_stats_update_task().await?;
        
        info!("✅ 负载均衡器启动成功");
        Ok(())
    }
    
    /// 停止负载均衡器
    pub async fn stop(&mut self) -> ClusterResult<()> {
        info!("🛑 停止负载均衡器");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        info!("✅ 负载均衡器已停止");
        Ok(())
    }
    
    /// 添加目标节点
    pub async fn add_target(&self, node_id: &str, endpoint: String) -> ClusterResult<()> {
        let target = TargetNode {
            id: node_id.to_string(),
            endpoint,
            weight: 1,
            connections: 0,
            avg_response_time: 0,
            healthy: true,
            updated_at: chrono::Utc::now(),
        };
        
        self.targets.insert(node_id.to_string(), target);
        
        debug!("添加负载均衡目标: {}", node_id);
        Ok(())
    }
    
    /// 移除目标节点
    pub async fn remove_target(&self, node_id: &str) -> ClusterResult<()> {
        self.targets.remove(node_id);
        
        debug!("移除负载均衡目标: {}", node_id);
        Ok(())
    }
    
    /// 更新目标节点权重
    pub async fn update_target_weight(&self, node_id: &str, weight: u32) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(node_id) {
            target.weight = weight;
            target.updated_at = chrono::Utc::now();
            
            debug!("更新目标权重: {} -> {}", node_id, weight);
        }
        
        Ok(())
    }
    
    /// 更新目标节点健康状态
    pub async fn update_target_health(&self, node_id: &str, healthy: bool) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(node_id) {
            target.healthy = healthy;
            target.updated_at = chrono::Utc::now();
            
            debug!("更新目标健康状态: {} -> {}", node_id, healthy);
        }
        
        Ok(())
    }
    
    /// 更新目标节点连接数
    pub async fn update_target_connections(&self, node_id: &str, connections: u32) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(node_id) {
            target.connections = connections;
            target.updated_at = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    /// 更新目标节点响应时间
    pub async fn update_target_response_time(&self, node_id: &str, response_time: u64) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(node_id) {
            // 使用指数移动平均计算平均响应时间
            if target.avg_response_time == 0 {
                target.avg_response_time = response_time;
            } else {
                target.avg_response_time = (target.avg_response_time * 7 + response_time) / 8;
            }
            target.updated_at = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    /// 选择目标节点
    pub async fn select_target(&self, candidates: &[String]) -> ClusterResult<Option<String>> {
        // 过滤健康的候选节点
        let healthy_targets: Vec<TargetNode> = candidates.iter()
            .filter_map(|id| self.targets.get(id))
            .filter(|target| target.healthy)
            .map(|target| target.clone())
            .collect();
        
        if healthy_targets.is_empty() {
            return Ok(None);
        }
        
        let selected = match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(&healthy_targets).await
            }
            LoadBalancingStrategy::Random => {
                self.select_random(&healthy_targets).await
            }
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_targets).await
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_targets).await
            }
            LoadBalancingStrategy::ConsistentHash => {
                // 对于一致性哈希，需要额外的键参数，这里使用第一个候选者
                self.select_consistent_hash(&healthy_targets, &candidates[0]).await
            }
            LoadBalancingStrategy::LeastResponseTime => {
                self.select_least_response_time(&healthy_targets).await
            }
        };
        
        if let Some(ref target_id) = selected {
            debug!("选择目标节点: {} (策略: {:?})", target_id, self.strategy);
        }
        
        Ok(selected)
    }
    
    /// 轮询选择
    async fn select_round_robin(&self, targets: &[TargetNode]) -> Option<String> {
        if targets.is_empty() {
            return None;
        }
        
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % targets.len();
        Some(targets[index].id.clone())
    }
    
    /// 随机选择
    async fn select_random(&self, targets: &[TargetNode]) -> Option<String> {
        if targets.is_empty() {
            return None;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..targets.len());
        Some(targets[index].id.clone())
    }
    
    /// 最少连接选择
    async fn select_least_connections(&self, targets: &[TargetNode]) -> Option<String> {
        targets.iter()
            .min_by_key(|target| target.connections)
            .map(|target| target.id.clone())
    }
    
    /// 加权轮询选择
    async fn select_weighted_round_robin(&self, targets: &[TargetNode]) -> Option<String> {
        if targets.is_empty() {
            return None;
        }
        
        // 计算总权重
        let total_weight: u32 = targets.iter().map(|t| t.weight).sum();
        if total_weight == 0 {
            return self.select_round_robin(targets).await;
        }
        
        // 生成随机数
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_weight);
        
        // 根据权重选择
        for target in targets {
            if random_weight < target.weight {
                return Some(target.id.clone());
            }
            random_weight -= target.weight;
        }
        
        // 备用方案
        Some(targets[0].id.clone())
    }
    
    /// 一致性哈希选择
    async fn select_consistent_hash(&self, targets: &[TargetNode], key: &str) -> Option<String> {
        if targets.is_empty() {
            return None;
        }
        
        // 简单的哈希实现
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        
        let index = (hash as usize) % targets.len();
        Some(targets[index].id.clone())
    }
    
    /// 最少响应时间选择
    async fn select_least_response_time(&self, targets: &[TargetNode]) -> Option<String> {
        targets.iter()
            .min_by_key(|target| target.avg_response_time)
            .map(|target| target.id.clone())
    }
    
    /// 获取所有目标节点
    pub async fn list_targets(&self) -> ClusterResult<Vec<TargetNode>> {
        let targets: Vec<TargetNode> = self.targets.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        Ok(targets)
    }
    
    /// 获取目标节点信息
    pub async fn get_target(&self, node_id: &str) -> ClusterResult<Option<TargetNode>> {
        if let Some(target) = self.targets.get(node_id) {
            Ok(Some(target.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// 启动统计更新任务
    async fn start_stats_update_task(&self) -> ClusterResult<()> {
        let targets = self.targets.clone();
        let running = self.running.clone();
        let update_interval = self.config.stats_update_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            
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
                debug!("📊 更新负载均衡统计信息");
                
                // TODO: 实现具体的统计更新逻辑
                // 1. 收集各节点的连接数
                // 2. 收集各节点的响应时间
                // 3. 更新健康状态
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[tokio::test]
    async fn test_load_balancer_creation() {
        let config = LoadBalancerConfig::default();
        let load_balancer = LoadBalancer::new(config).await.unwrap();
        
        assert_eq!(load_balancer.strategy, LoadBalancingStrategy::RoundRobin);
    }
    
    #[tokio::test]
    async fn test_target_management() {
        let config = LoadBalancerConfig::default();
        let load_balancer = LoadBalancer::new(config).await.unwrap();
        
        // 添加目标
        load_balancer.add_target("node1", "http://localhost:8001".to_string()).await.unwrap();
        load_balancer.add_target("node2", "http://localhost:8002".to_string()).await.unwrap();
        
        // 列出目标
        let targets = load_balancer.list_targets().await.unwrap();
        assert_eq!(targets.len(), 2);
        
        // 获取目标
        let target = load_balancer.get_target("node1").await.unwrap();
        assert!(target.is_some());
        assert_eq!(target.unwrap().id, "node1");
        
        // 更新权重
        load_balancer.update_target_weight("node1", 5).await.unwrap();
        let target = load_balancer.get_target("node1").await.unwrap();
        assert_eq!(target.unwrap().weight, 5);
        
        // 更新健康状态
        load_balancer.update_target_health("node1", false).await.unwrap();
        let target = load_balancer.get_target("node1").await.unwrap();
        assert!(!target.unwrap().healthy);
        
        // 移除目标
        load_balancer.remove_target("node1").await.unwrap();
        let target = load_balancer.get_target("node1").await.unwrap();
        assert!(target.is_none());
    }
    
    #[tokio::test]
    async fn test_round_robin_selection() {
        let config = LoadBalancerConfig {
            strategy: LoadBalancingStrategy::RoundRobin,
            ..Default::default()
        };
        let load_balancer = LoadBalancer::new(config).await.unwrap();
        
        // 添加目标
        load_balancer.add_target("node1", "http://localhost:8001".to_string()).await.unwrap();
        load_balancer.add_target("node2", "http://localhost:8002".to_string()).await.unwrap();
        load_balancer.add_target("node3", "http://localhost:8003".to_string()).await.unwrap();
        
        let candidates = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
        
        // 测试轮询选择
        let mut selections = Vec::new();
        for _ in 0..6 {
            let selected = load_balancer.select_target(&candidates).await.unwrap();
            selections.push(selected.unwrap());
        }
        
        // 验证轮询模式
        assert_eq!(selections[0], "node1");
        assert_eq!(selections[1], "node2");
        assert_eq!(selections[2], "node3");
        assert_eq!(selections[3], "node1");
        assert_eq!(selections[4], "node2");
        assert_eq!(selections[5], "node3");
    }
    
    #[tokio::test]
    async fn test_least_connections_selection() {
        let config = LoadBalancerConfig {
            strategy: LoadBalancingStrategy::LeastConnections,
            ..Default::default()
        };
        let load_balancer = LoadBalancer::new(config).await.unwrap();
        
        // 添加目标
        load_balancer.add_target("node1", "http://localhost:8001".to_string()).await.unwrap();
        load_balancer.add_target("node2", "http://localhost:8002".to_string()).await.unwrap();
        
        // 设置不同的连接数
        load_balancer.update_target_connections("node1", 5).await.unwrap();
        load_balancer.update_target_connections("node2", 2).await.unwrap();
        
        let candidates = vec!["node1".to_string(), "node2".to_string()];
        
        // 应该选择连接数较少的node2
        let selected = load_balancer.select_target(&candidates).await.unwrap();
        assert_eq!(selected.unwrap(), "node2");
    }
    
    #[tokio::test]
    async fn test_health_filtering() {
        let config = LoadBalancerConfig::default();
        let load_balancer = LoadBalancer::new(config).await.unwrap();
        
        // 添加目标
        load_balancer.add_target("node1", "http://localhost:8001".to_string()).await.unwrap();
        load_balancer.add_target("node2", "http://localhost:8002".to_string()).await.unwrap();
        
        // 设置node1为不健康
        load_balancer.update_target_health("node1", false).await.unwrap();
        
        let candidates = vec!["node1".to_string(), "node2".to_string()];
        
        // 应该只选择健康的node2
        let selected = load_balancer.select_target(&candidates).await.unwrap();
        assert_eq!(selected.unwrap(), "node2");
        
        // 如果所有节点都不健康，应该返回None
        load_balancer.update_target_health("node2", false).await.unwrap();
        let selected = load_balancer.select_target(&candidates).await.unwrap();
        assert!(selected.is_none());
    }
}
