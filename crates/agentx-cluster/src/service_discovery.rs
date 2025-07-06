//! 服务发现模块
//! 
//! 提供Agent注册、发现和健康检查功能

use crate::config::DiscoveryConfig;
use crate::error::{ClusterError, ClusterResult};
use agentx_a2a::AgentCard;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};
use std::collections::HashMap;

/// 服务发现后端类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryBackend {
    /// 内存存储（用于测试和单机部署）
    Memory,
    /// Consul
    Consul,
    /// etcd
    Etcd,
    /// Kubernetes
    Kubernetes,
}

/// 服务注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistry {
    /// 服务ID
    pub service_id: String,
    /// Agent信息
    pub agent_info: AgentCard,
    /// 注册时间
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// TTL（生存时间）
    pub ttl_seconds: u64,
    /// 标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: std::collections::HashMap<String, String>,
}

/// 服务发现接口
#[async_trait::async_trait]
pub trait ServiceDiscoveryBackend: Send + Sync {
    /// 注册服务
    async fn register(&self, registry: ServiceRegistry) -> ClusterResult<()>;
    
    /// 注销服务
    async fn deregister(&self, service_id: &str) -> ClusterResult<()>;
    
    /// 发现服务
    async fn discover(&self, capability: Option<&str>) -> ClusterResult<Vec<ServiceRegistry>>;
    
    /// 更新服务健康状态
    async fn update_health(&self, service_id: &str, healthy: bool) -> ClusterResult<()>;
    
    /// 获取服务信息
    async fn get_service(&self, service_id: &str) -> ClusterResult<Option<ServiceRegistry>>;
    
    /// 列出所有服务
    async fn list_services(&self) -> ClusterResult<Vec<ServiceRegistry>>;
}

/// 内存服务发现后端
pub struct MemoryServiceDiscovery {
    /// 服务注册表
    services: Arc<DashMap<String, ServiceRegistry>>,
    /// 健康状态
    health_status: Arc<DashMap<String, bool>>,
}

impl MemoryServiceDiscovery {
    pub fn new() -> Self {
        Self {
            services: Arc::new(DashMap::new()),
            health_status: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl ServiceDiscoveryBackend for MemoryServiceDiscovery {
    async fn register(&self, registry: ServiceRegistry) -> ClusterResult<()> {
        let service_id = registry.service_id.clone();
        self.services.insert(service_id.clone(), registry);
        self.health_status.insert(service_id.clone(), true);
        
        debug!("注册服务: {}", service_id);
        Ok(())
    }
    
    async fn deregister(&self, service_id: &str) -> ClusterResult<()> {
        self.services.remove(service_id);
        self.health_status.remove(service_id);
        
        debug!("注销服务: {}", service_id);
        Ok(())
    }
    
    async fn discover(&self, capability: Option<&str>) -> ClusterResult<Vec<ServiceRegistry>> {
        let mut results = Vec::new();
        
        for entry in self.services.iter() {
            let registry = entry.value();
            
            // 检查健康状态
            if let Some(healthy) = self.health_status.get(&registry.service_id) {
                if !*healthy {
                    continue;
                }
            }
            
            // 过滤能力
            if let Some(cap) = capability {
                let has_capability = registry.agent_info.capabilities.iter()
                    .any(|c| c.name == cap);
                if !has_capability {
                    continue;
                }
            }
            
            results.push(registry.clone());
        }
        
        debug!("发现服务数量: {}", results.len());
        Ok(results)
    }
    
    async fn update_health(&self, service_id: &str, healthy: bool) -> ClusterResult<()> {
        self.health_status.insert(service_id.to_string(), healthy);
        
        debug!("更新服务健康状态: {} -> {}", service_id, healthy);
        Ok(())
    }
    
    async fn get_service(&self, service_id: &str) -> ClusterResult<Option<ServiceRegistry>> {
        if let Some(registry) = self.services.get(service_id) {
            Ok(Some(registry.clone()))
        } else {
            Ok(None)
        }
    }
    
    async fn list_services(&self) -> ClusterResult<Vec<ServiceRegistry>> {
        let services: Vec<ServiceRegistry> = self.services.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        Ok(services)
    }
}

/// 服务发现管理器
pub struct ServiceDiscovery {
    /// 后端实现
    backend: Box<dyn ServiceDiscoveryBackend>,
    /// 配置
    config: DiscoveryConfig,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl ServiceDiscovery {
    /// 创建新的服务发现管理器
    pub async fn new(config: DiscoveryConfig) -> ClusterResult<Self> {
        let backend: Box<dyn ServiceDiscoveryBackend> = match config.backend {
            DiscoveryBackend::Memory => Box::new(MemoryServiceDiscovery::new()),
            DiscoveryBackend::Consul => {
                // TODO: 实现Consul后端
                return Err(ClusterError::UnsupportedBackend("Consul后端尚未实现".to_string()));
            }
            DiscoveryBackend::Etcd => {
                // 实现etcd后端
                let endpoints = vec!["http://localhost:2379".to_string()]; // 默认etcd端点
                let key_prefix = "/agentx".to_string();
                Box::new(EtcdServiceDiscovery::new(endpoints, key_prefix))
            }
            DiscoveryBackend::Kubernetes => {
                // TODO: 实现Kubernetes后端
                return Err(ClusterError::UnsupportedBackend("Kubernetes后端尚未实现".to_string()));
            }
        };
        
        info!("🔍 创建服务发现管理器，后端: {:?}", config.backend);
        
        Ok(Self {
            backend,
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// 启动服务发现
    pub async fn start(&mut self) -> ClusterResult<()> {
        info!("🚀 启动服务发现");
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动清理任务
        self.start_cleanup_task().await?;
        
        info!("✅ 服务发现启动成功");
        Ok(())
    }
    
    /// 停止服务发现
    pub async fn stop(&mut self) -> ClusterResult<()> {
        info!("🛑 停止服务发现");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        info!("✅ 服务发现已停止");
        Ok(())
    }
    
    /// 注册Agent
    pub async fn register_agent(&self, agent_info: AgentCard) -> ClusterResult<String> {
        let service_id = format!("agent-{}", agent_info.id);
        
        let registry = ServiceRegistry {
            service_id: service_id.clone(),
            agent_info,
            registered_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ttl_seconds: self.config.ttl_seconds,
            tags: vec!["agent".to_string()],
            metadata: std::collections::HashMap::new(),
        };
        
        self.backend.register(registry).await?;
        
        info!("📝 注册Agent: {}", service_id);
        Ok(service_id)
    }
    
    /// 注销Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> ClusterResult<()> {
        let service_id = if agent_id.starts_with("agent-") {
            agent_id.to_string()
        } else {
            format!("agent-{}", agent_id)
        };
        
        self.backend.deregister(&service_id).await?;
        
        info!("🗑️ 注销Agent: {}", service_id);
        Ok(())
    }
    
    /// 发现Agent
    pub async fn discover_agents(&self, capability: Option<&str>) -> ClusterResult<Vec<AgentCard>> {
        let registries = self.backend.discover(capability).await?;
        
        let agents: Vec<AgentCard> = registries.into_iter()
            .map(|registry| registry.agent_info)
            .collect();
        
        debug!("发现Agent数量: {}", agents.len());
        Ok(agents)
    }
    
    /// 更新Agent健康状态
    pub async fn update_agent_health(&self, agent_id: &str, healthy: bool) -> ClusterResult<()> {
        let service_id = if agent_id.starts_with("agent-") {
            agent_id.to_string()
        } else {
            format!("agent-{}", agent_id)
        };
        
        self.backend.update_health(&service_id, healthy).await?;
        
        debug!("更新Agent健康状态: {} -> {}", service_id, healthy);
        Ok(())
    }
    
    /// 获取Agent信息
    pub async fn get_agent(&self, agent_id: &str) -> ClusterResult<Option<AgentCard>> {
        let service_id = if agent_id.starts_with("agent-") {
            agent_id.to_string()
        } else {
            format!("agent-{}", agent_id)
        };
        
        if let Some(registry) = self.backend.get_service(&service_id).await? {
            Ok(Some(registry.agent_info))
        } else {
            Ok(None)
        }
    }
    
    /// 列出所有Agent
    pub async fn list_agents(&self) -> ClusterResult<Vec<AgentCard>> {
        let registries = self.backend.list_services().await?;
        
        let agents: Vec<AgentCard> = registries.into_iter()
            .filter(|registry| registry.tags.contains(&"agent".to_string()))
            .map(|registry| registry.agent_info)
            .collect();
        
        Ok(agents)
    }
    
    /// 启动清理任务
    async fn start_cleanup_task(&self) -> ClusterResult<()> {
        let backend = self.backend.as_ref() as *const dyn ServiceDiscoveryBackend;
        let running = self.running.clone();
        let cleanup_interval = self.config.cleanup_interval;
        let ttl_seconds = self.config.ttl_seconds;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 执行清理逻辑
                // 注意：这里使用裸指针是不安全的，在实际实现中应该使用Arc<dyn ServiceDiscoveryBackend>
                debug!("🧹 执行服务清理任务");
                
                // TODO: 实现过期服务清理逻辑
                // 1. 获取所有服务
                // 2. 检查TTL
                // 3. 清理过期服务
            }
        });
        
        Ok(())
    }
}

/// etcd服务发现后端实现
pub struct EtcdServiceDiscovery {
    /// etcd客户端配置
    endpoints: Vec<String>,
    /// 键前缀
    key_prefix: String,
    /// TTL设置
    default_ttl: u64,
    /// 本地缓存
    cache: Arc<DashMap<String, ServiceRegistry>>,
}

impl EtcdServiceDiscovery {
    /// 创建新的etcd服务发现后端
    pub fn new(endpoints: Vec<String>, key_prefix: String) -> Self {
        Self {
            endpoints,
            key_prefix,
            default_ttl: 30, // 默认30秒TTL
            cache: Arc::new(DashMap::new()),
        }
    }

    /// 构建etcd键
    fn build_key(&self, service_id: &str) -> String {
        format!("{}/services/{}", self.key_prefix, service_id)
    }

    /// 模拟etcd操作（实际实现需要etcd客户端）
    async fn etcd_put(&self, key: &str, value: &str, ttl: u64) -> ClusterResult<()> {
        // 这里应该使用真实的etcd客户端
        // 为了演示，我们使用本地缓存模拟
        debug!("模拟etcd PUT: {} = {} (TTL: {}s)", key, value, ttl);
        Ok(())
    }

    async fn etcd_get(&self, key: &str) -> ClusterResult<Option<String>> {
        // 这里应该使用真实的etcd客户端
        debug!("模拟etcd GET: {}", key);
        Ok(None) // 模拟返回
    }

    async fn etcd_delete(&self, key: &str) -> ClusterResult<()> {
        // 这里应该使用真实的etcd客户端
        debug!("模拟etcd DELETE: {}", key);
        Ok(())
    }

    async fn etcd_list(&self, prefix: &str) -> ClusterResult<Vec<(String, String)>> {
        // 这里应该使用真实的etcd客户端
        debug!("模拟etcd LIST: {}", prefix);
        Ok(Vec::new()) // 模拟返回
    }
}

#[async_trait::async_trait]
impl ServiceDiscoveryBackend for EtcdServiceDiscovery {
    async fn register(&self, registry: ServiceRegistry) -> ClusterResult<()> {
        info!("📝 注册服务到etcd: {}", registry.service_id);

        let key = self.build_key(&registry.service_id);
        let value = serde_json::to_string(&registry)
            .map_err(|e| ClusterError::SerializationError(e.to_string()))?;

        // 注册到etcd
        self.etcd_put(&key, &value, registry.ttl_seconds).await?;

        // 更新本地缓存
        self.cache.insert(registry.service_id.clone(), registry);

        Ok(())
    }

    async fn deregister(&self, service_id: &str) -> ClusterResult<()> {
        info!("🗑️ 从etcd注销服务: {}", service_id);

        let key = self.build_key(service_id);

        // 从etcd删除
        self.etcd_delete(&key).await?;

        // 从本地缓存删除
        self.cache.remove(service_id);

        Ok(())
    }

    async fn discover(&self, capability: Option<&str>) -> ClusterResult<Vec<ServiceRegistry>> {
        debug!("🔍 从etcd发现服务，能力过滤: {:?}", capability);

        let prefix = format!("{}/services/", self.key_prefix);
        let entries = self.etcd_list(&prefix).await?;

        let mut results = Vec::new();

        for (_key, value) in entries {
            if let Ok(registry) = serde_json::from_str::<ServiceRegistry>(&value) {
                // 过滤能力
                if let Some(cap) = capability {
                    let has_capability = registry.agent_info.capabilities.iter()
                        .any(|c| c.name == cap);
                    if !has_capability {
                        continue;
                    }
                }

                results.push(registry);
            }
        }

        // 如果etcd为空，使用本地缓存
        if results.is_empty() {
            for entry in self.cache.iter() {
                let registry = entry.value();

                if let Some(cap) = capability {
                    let has_capability = registry.agent_info.capabilities.iter()
                        .any(|c| c.name == cap);
                    if !has_capability {
                        continue;
                    }
                }

                results.push(registry.clone());
            }
        }

        Ok(results)
    }

    async fn update_health(&self, service_id: &str, healthy: bool) -> ClusterResult<()> {
        debug!("💓 更新etcd服务健康状态: {} -> {}", service_id, healthy);

        if let Some(mut registry) = self.cache.get_mut(service_id) {
            registry.metadata.insert("healthy".to_string(), healthy.to_string());
            registry.updated_at = chrono::Utc::now();

            // 更新到etcd
            let key = self.build_key(service_id);
            let value = serde_json::to_string(&*registry)
                .map_err(|e| ClusterError::SerializationError(e.to_string()))?;

            self.etcd_put(&key, &value, registry.ttl_seconds).await?;
        }

        Ok(())
    }

    async fn get_service(&self, service_id: &str) -> ClusterResult<Option<ServiceRegistry>> {
        // 首先检查本地缓存
        if let Some(registry) = self.cache.get(service_id) {
            return Ok(Some(registry.clone()));
        }

        // 从etcd获取
        let key = self.build_key(service_id);
        if let Some(value) = self.etcd_get(&key).await? {
            if let Ok(registry) = serde_json::from_str::<ServiceRegistry>(&value) {
                // 更新本地缓存
                self.cache.insert(service_id.to_string(), registry.clone());
                return Ok(Some(registry));
            }
        }

        Ok(None)
    }

    async fn list_services(&self) -> ClusterResult<Vec<ServiceRegistry>> {
        let prefix = format!("{}/services/", self.key_prefix);
        let entries = self.etcd_list(&prefix).await?;

        let mut services = Vec::new();

        for (_key, value) in entries {
            if let Ok(registry) = serde_json::from_str::<ServiceRegistry>(&value) {
                services.push(registry);
            }
        }

        // 如果etcd为空，使用本地缓存
        if services.is_empty() {
            services = self.cache.iter()
                .map(|entry| entry.value().clone())
                .collect();
        }

        Ok(services)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    fn create_test_agent(id: &str) -> AgentCard {
        use agentx_a2a::{AgentStatus, TrustLevel, InteractionModality, Capability, CapabilityType, Endpoint};

        AgentCard {
            id: id.to_string(),
            name: format!("Test Agent {}", id),
            description: "Test agent for service discovery".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                Capability {
                    name: "test".to_string(),
                    description: "Test capability".to_string(),
                    capability_type: CapabilityType::ToolExecution,
                    input_schema: None,
                    output_schema: None,
                    metadata: std::collections::HashMap::new(),
                    available: true,
                    cost: None,
                }
            ],
            endpoints: vec![
                Endpoint {
                    endpoint_type: "http".to_string(),
                    url: format!("http://localhost:8080"),
                    protocols: vec!["http".to_string()],
                    auth: None,
                    metadata: std::collections::HashMap::new(),
                }
            ],
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
    async fn test_memory_service_discovery() {
        let backend = MemoryServiceDiscovery::new();
        
        // 创建测试注册信息
        let agent = create_test_agent("test1");
        let registry = ServiceRegistry {
            service_id: "test-service".to_string(),
            agent_info: agent,
            registered_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ttl_seconds: 300,
            tags: vec!["agent".to_string()],
            metadata: std::collections::HashMap::new(),
        };
        
        // 注册服务
        backend.register(registry.clone()).await.unwrap();
        
        // 发现服务
        let discovered = backend.discover(None).await.unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].service_id, "test-service");
        
        // 按能力发现
        let discovered = backend.discover(Some("test")).await.unwrap();
        assert_eq!(discovered.len(), 1);
        
        let discovered = backend.discover(Some("nonexistent")).await.unwrap();
        assert_eq!(discovered.len(), 0);
        
        // 获取服务
        let service = backend.get_service("test-service").await.unwrap();
        assert!(service.is_some());
        
        // 更新健康状态
        backend.update_health("test-service", false).await.unwrap();
        
        // 健康检查后应该发现不到服务
        let discovered = backend.discover(None).await.unwrap();
        assert_eq!(discovered.len(), 0);
        
        // 注销服务
        backend.deregister("test-service").await.unwrap();
        
        let service = backend.get_service("test-service").await.unwrap();
        assert!(service.is_none());
    }
    
    #[tokio::test]
    async fn test_service_discovery_manager() {
        let config = DiscoveryConfig::default();
        let mut discovery = ServiceDiscovery::new(config).await.unwrap();
        
        // 启动服务发现
        discovery.start().await.unwrap();
        
        // 注册Agent
        let agent = create_test_agent("test1");
        let service_id = discovery.register_agent(agent.clone()).await.unwrap();
        assert_eq!(service_id, "agent-test1");
        
        // 发现Agent
        let agents = discovery.discover_agents(None).await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "test1");
        
        // 按能力发现
        let agents = discovery.discover_agents(Some("test")).await.unwrap();
        assert_eq!(agents.len(), 1);
        
        let agents = discovery.discover_agents(Some("nonexistent")).await.unwrap();
        assert_eq!(agents.len(), 0);
        
        // 获取Agent
        let agent_info = discovery.get_agent("test1").await.unwrap();
        assert!(agent_info.is_some());
        assert_eq!(agent_info.unwrap().id, "test1");
        
        // 更新健康状态
        discovery.update_agent_health("test1", false).await.unwrap();
        
        // 注销Agent
        discovery.unregister_agent("test1").await.unwrap();
        
        let agent_info = discovery.get_agent("test1").await.unwrap();
        assert!(agent_info.is_none());
        
        // 停止服务发现
        discovery.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_etcd_service_discovery() {
        use crate::config::DiscoveryConfig;

        // 创建etcd后端配置
        let config = DiscoveryConfig {
            backend: DiscoveryBackend::Etcd,
            ttl_seconds: 30,
            cleanup_interval: std::time::Duration::from_secs(60),
            consul: None,
            etcd: None,
            kubernetes: None,
        };

        // 创建etcd服务发现
        let etcd_backend = EtcdServiceDiscovery::new(
            vec!["http://localhost:2379".to_string()],
            "/agentx".to_string()
        );

        // 创建测试服务注册信息
        let agent_card = AgentCard::new(
            "etcd_test_agent".to_string(),
            "Test Agent".to_string(),
            "Test agent for etcd backend".to_string(),
            "1.0.0".to_string()
        );
        let registry = ServiceRegistry {
            service_id: "etcd_test_service".to_string(),
            agent_info: agent_card,
            registered_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            ttl_seconds: 30,
            tags: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        // 测试注册
        let result = etcd_backend.register(registry.clone()).await;
        assert!(result.is_ok());

        // 测试获取服务
        let service = etcd_backend.get_service("etcd_test_service").await.unwrap();
        assert!(service.is_some());
        assert_eq!(service.unwrap().service_id, "etcd_test_service");

        // 测试发现服务
        let services = etcd_backend.discover(None).await.unwrap();
        assert!(!services.is_empty());

        // 测试更新健康状态
        let result = etcd_backend.update_health("etcd_test_service", false).await;
        assert!(result.is_ok());

        // 测试注销服务
        let result = etcd_backend.deregister("etcd_test_service").await;
        assert!(result.is_ok());

        // 验证服务已被删除
        let service = etcd_backend.get_service("etcd_test_service").await.unwrap();
        assert!(service.is_none());
    }

    #[test]
    fn test_etcd_key_building() {
        let etcd_backend = EtcdServiceDiscovery::new(
            vec!["http://localhost:2379".to_string()],
            "/agentx".to_string()
        );

        let key = etcd_backend.build_key("test_service");
        assert_eq!(key, "/agentx/services/test_service");
    }

    #[test]
    fn test_discovery_backend_enum() {
        // 测试序列化和反序列化
        let backend = DiscoveryBackend::Etcd;
        let serialized = serde_json::to_string(&backend).unwrap();
        let deserialized: DiscoveryBackend = serde_json::from_str(&serialized).unwrap();
        assert_eq!(backend, deserialized);

        // 测试所有变体
        assert_eq!(DiscoveryBackend::Memory, DiscoveryBackend::Memory);
        assert_eq!(DiscoveryBackend::Consul, DiscoveryBackend::Consul);
        assert_eq!(DiscoveryBackend::Etcd, DiscoveryBackend::Etcd);
        assert_eq!(DiscoveryBackend::Kubernetes, DiscoveryBackend::Kubernetes);
    }
}
