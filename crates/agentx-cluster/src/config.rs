//! 集群配置模块
//! 
//! 定义集群管理的各种配置选项

use crate::node_manager::NodeRole;
use crate::service_discovery::DiscoveryBackend;
use crate::load_balancer::LoadBalancingStrategy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// 集群配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// 节点配置
    pub node: NodeConfig,
    /// 服务发现配置
    pub discovery: DiscoveryConfig,
    /// 负载均衡配置
    pub load_balancer: LoadBalancerConfig,
    /// 集群状态配置
    pub state: StateConfig,
    /// 健康检查配置
    pub health_check: HealthCheckConfig,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            discovery: DiscoveryConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
            state: StateConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// 节点ID（可选，自动生成）
    pub node_id: Option<String>,
    /// 节点名称
    pub node_name: String,
    /// 绑定地址
    pub bind_address: SocketAddr,
    /// 节点角色
    pub role: NodeRole,
    /// 心跳间隔
    pub heartbeat_interval: Duration,
    /// 节点发现间隔
    pub discovery_interval: Duration,
    /// 节点元数据
    pub metadata: HashMap<String, String>,
    /// 节点能力
    pub capabilities: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: None,
            node_name: "agentx-node".to_string(),
            bind_address: "0.0.0.0:8080".parse().unwrap(),
            role: NodeRole::Worker,
            heartbeat_interval: Duration::from_secs(30),
            discovery_interval: Duration::from_secs(60),
            metadata: HashMap::new(),
            capabilities: vec![
                "agent.hosting".to_string(),
                "message.routing".to_string(),
            ],
        }
    }
}

/// 服务发现配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// 后端类型
    pub backend: DiscoveryBackend,
    /// TTL（生存时间）
    pub ttl_seconds: u64,
    /// 清理间隔
    pub cleanup_interval: Duration,
    /// Consul配置
    pub consul: Option<ConsulConfig>,
    /// etcd配置
    pub etcd: Option<EtcdConfig>,
    /// Kubernetes配置
    pub kubernetes: Option<KubernetesConfig>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            backend: DiscoveryBackend::Memory,
            ttl_seconds: 300,
            cleanup_interval: Duration::from_secs(60),
            consul: None,
            etcd: None,
            kubernetes: None,
        }
    }
}

/// Consul配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsulConfig {
    /// Consul地址
    pub address: String,
    /// 数据中心
    pub datacenter: Option<String>,
    /// Token
    pub token: Option<String>,
    /// 命名空间
    pub namespace: Option<String>,
}

/// etcd配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtcdConfig {
    /// etcd端点
    pub endpoints: Vec<String>,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 证书路径
    pub cert_path: Option<String>,
    /// 密钥路径
    pub key_path: Option<String>,
    /// CA证书路径
    pub ca_path: Option<String>,
}

/// Kubernetes配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// 命名空间
    pub namespace: String,
    /// 标签选择器
    pub label_selector: Option<String>,
    /// 字段选择器
    pub field_selector: Option<String>,
    /// kubeconfig路径
    pub kubeconfig_path: Option<String>,
}

/// 负载均衡配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// 负载均衡策略
    pub strategy: LoadBalancingStrategy,
    /// 统计更新间隔
    pub stats_update_interval: Duration,
    /// 健康检查间隔
    pub health_check_interval: Duration,
    /// 连接超时
    pub connection_timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            stats_update_interval: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
            connection_timeout: Duration::from_secs(5),
            max_retries: 3,
        }
    }
}

/// 集群状态配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// 集群ID
    pub cluster_id: String,
    /// 集群名称
    pub cluster_name: String,
    /// 状态同步后端
    pub sync_backend: String,
    /// 同步间隔
    pub sync_interval: Duration,
    /// 统计间隔
    pub stats_interval: Duration,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            cluster_id: uuid::Uuid::new_v4().to_string(),
            cluster_name: "agentx-cluster".to_string(),
            sync_backend: "memory".to_string(),
            sync_interval: Duration::from_secs(30),
            stats_interval: Duration::from_secs(60),
            metadata: HashMap::new(),
        }
    }
}

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// 检查间隔
    pub check_interval: Duration,
    /// 默认检查间隔
    pub default_interval: Duration,
    /// 默认超时时间
    pub default_timeout: Duration,
    /// 默认重试次数
    pub default_retries: u32,
    /// 失败阈值
    pub failure_threshold: u32,
    /// 成功阈值
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(10),
            default_interval: Duration::from_secs(30),
            default_timeout: Duration::from_secs(5),
            default_retries: 3,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

impl ClusterConfig {
    /// 从文件加载配置
    pub async fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub async fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
    
    /// 从环境变量加载配置
    pub fn load_from_env(&mut self) {
        // 节点配置
        if let Ok(node_id) = std::env::var("AGENTX_NODE_ID") {
            self.node.node_id = Some(node_id);
        }
        if let Ok(node_name) = std::env::var("AGENTX_NODE_NAME") {
            self.node.node_name = node_name;
        }
        if let Ok(bind_address) = std::env::var("AGENTX_BIND_ADDRESS") {
            if let Ok(addr) = bind_address.parse() {
                self.node.bind_address = addr;
            }
        }
        
        // 服务发现配置
        if let Ok(discovery_backend) = std::env::var("AGENTX_DISCOVERY_BACKEND") {
            match discovery_backend.as_str() {
                "consul" => self.discovery.backend = DiscoveryBackend::Consul,
                "etcd" => self.discovery.backend = DiscoveryBackend::Etcd,
                "kubernetes" => self.discovery.backend = DiscoveryBackend::Kubernetes,
                _ => self.discovery.backend = DiscoveryBackend::Memory,
            }
        }
        
        // 负载均衡配置
        if let Ok(lb_strategy) = std::env::var("AGENTX_LB_STRATEGY") {
            match lb_strategy.as_str() {
                "random" => self.load_balancer.strategy = LoadBalancingStrategy::Random,
                "least_connections" => self.load_balancer.strategy = LoadBalancingStrategy::LeastConnections,
                "weighted_round_robin" => self.load_balancer.strategy = LoadBalancingStrategy::WeightedRoundRobin,
                "consistent_hash" => self.load_balancer.strategy = LoadBalancingStrategy::ConsistentHash,
                "least_response_time" => self.load_balancer.strategy = LoadBalancingStrategy::LeastResponseTime,
                _ => self.load_balancer.strategy = LoadBalancingStrategy::RoundRobin,
            }
        }
        
        // 集群状态配置
        if let Ok(cluster_id) = std::env::var("AGENTX_CLUSTER_ID") {
            self.state.cluster_id = cluster_id;
        }
        if let Ok(cluster_name) = std::env::var("AGENTX_CLUSTER_NAME") {
            self.state.cluster_name = cluster_name;
        }
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证节点配置
        if self.node.node_name.is_empty() {
            return Err("节点名称不能为空".to_string());
        }
        
        if self.node.heartbeat_interval.as_secs() == 0 {
            return Err("心跳间隔必须大于0".to_string());
        }
        
        // 验证服务发现配置
        if self.discovery.ttl_seconds == 0 {
            return Err("TTL必须大于0".to_string());
        }
        
        // 验证负载均衡配置
        if self.load_balancer.connection_timeout.as_secs() == 0 {
            return Err("连接超时必须大于0".to_string());
        }
        
        // 验证集群状态配置
        if self.state.cluster_id.is_empty() {
            return Err("集群ID不能为空".to_string());
        }
        
        if self.state.cluster_name.is_empty() {
            return Err("集群名称不能为空".to_string());
        }
        
        // 验证健康检查配置
        if self.health_check.default_timeout.as_secs() == 0 {
            return Err("健康检查超时必须大于0".to_string());
        }
        
        Ok(())
    }
    
    /// 获取运行时信息
    pub fn get_runtime_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("node_name".to_string(), self.node.node_name.clone());
        info.insert("bind_address".to_string(), self.node.bind_address.to_string());
        info.insert("node_role".to_string(), format!("{:?}", self.node.role));
        info.insert("discovery_backend".to_string(), format!("{:?}", self.discovery.backend));
        info.insert("lb_strategy".to_string(), format!("{:?}", self.load_balancer.strategy));
        info.insert("cluster_id".to_string(), self.state.cluster_id.clone());
        info.insert("cluster_name".to_string(), self.state.cluster_name.clone());
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ClusterConfig::default();
        
        assert_eq!(config.node.node_name, "agentx-node");
        assert_eq!(config.discovery.backend, DiscoveryBackend::Memory);
        assert_eq!(config.load_balancer.strategy, LoadBalancingStrategy::RoundRobin);
        assert!(!config.state.cluster_id.is_empty());
        assert_eq!(config.state.cluster_name, "agentx-cluster");
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = ClusterConfig::default();
        
        // 有效配置应该通过验证
        assert!(config.validate().is_ok());
        
        // 无效的节点名称
        config.node.node_name = String::new();
        assert!(config.validate().is_err());
        
        // 恢复有效配置
        config.node.node_name = "test-node".to_string();
        assert!(config.validate().is_ok());
        
        // 无效的TTL
        config.discovery.ttl_seconds = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_env_config_loading() {
        let mut config = ClusterConfig::default();
        
        // 设置环境变量
        std::env::set_var("AGENTX_NODE_NAME", "test-node-env");
        std::env::set_var("AGENTX_CLUSTER_NAME", "test-cluster-env");
        std::env::set_var("AGENTX_DISCOVERY_BACKEND", "consul");
        std::env::set_var("AGENTX_LB_STRATEGY", "random");
        
        // 从环境变量加载
        config.load_from_env();
        
        assert_eq!(config.node.node_name, "test-node-env");
        assert_eq!(config.state.cluster_name, "test-cluster-env");
        assert_eq!(config.discovery.backend, DiscoveryBackend::Consul);
        assert_eq!(config.load_balancer.strategy, LoadBalancingStrategy::Random);
        
        // 清理环境变量
        std::env::remove_var("AGENTX_NODE_NAME");
        std::env::remove_var("AGENTX_CLUSTER_NAME");
        std::env::remove_var("AGENTX_DISCOVERY_BACKEND");
        std::env::remove_var("AGENTX_LB_STRATEGY");
    }
    
    #[test]
    fn test_runtime_info() {
        let config = ClusterConfig::default();
        let info = config.get_runtime_info();
        
        assert!(info.contains_key("node_name"));
        assert!(info.contains_key("bind_address"));
        assert!(info.contains_key("node_role"));
        assert!(info.contains_key("discovery_backend"));
        assert!(info.contains_key("lb_strategy"));
        assert!(info.contains_key("cluster_id"));
        assert!(info.contains_key("cluster_name"));
    }
}
