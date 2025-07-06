//! Message Router Implementation
//! 
//! This module implements intelligent message routing with load balancing,
//! failover, and performance optimization.

use agentx_a2a::{A2AMessage, AgentCard, A2AResult};
use crate::{RoutingStrategy, RouterMetrics, RouteCache, RouterError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};

/// Agent端点信息
#[derive(Debug, Clone)]
pub struct AgentEndpoint {
    /// 端点URL
    pub url: String,
    /// 端点类型 (http, grpc, websocket等)
    pub endpoint_type: String,
    /// 端点状态
    pub status: EndpointStatus,
    /// 最后健康检查时间
    pub last_health_check: DateTime<Utc>,
}

/// 端点状态
#[derive(Debug, Clone, PartialEq)]
pub enum EndpointStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

impl AgentEndpoint {
    pub fn new(url: String, endpoint_type: String) -> Self {
        Self {
            url,
            endpoint_type,
            status: EndpointStatus::Unknown,
            last_health_check: Utc::now(),
        }
    }
}

/// A2A客户端接口（简化版本）
#[async_trait]
pub trait A2AClient: Send + Sync {
    /// 发送消息到指定端点
    async fn send_message(&self, endpoint: &AgentEndpoint, message: &A2AMessage) -> A2AResult<A2AMessage>;

    /// 检查端点健康状态
    async fn health_check(&self, endpoint: &AgentEndpoint) -> A2AResult<bool>;
}

/// 默认的A2A客户端实现
#[derive(Debug)]
pub struct DefaultA2AClient;

#[async_trait]
impl A2AClient for DefaultA2AClient {
    async fn send_message(&self, endpoint: &AgentEndpoint, message: &A2AMessage) -> A2AResult<A2AMessage> {
        debug!("发送消息到端点: {} (类型: {})", endpoint.url, endpoint.endpoint_type);

        // 这里应该实现真实的消息发送逻辑
        // 暂时返回一个模拟响应
        Ok(A2AMessage::agent_message(format!(
            "来自 {} 的响应: {}",
            endpoint.url,
            message.message_id
        )))
    }

    async fn health_check(&self, endpoint: &AgentEndpoint) -> A2AResult<bool> {
        debug!("检查端点健康状态: {}", endpoint.url);

        // 这里应该实现真实的健康检查逻辑
        // 暂时返回true
        Ok(true)
    }
}

/// Message Router for intelligent A2A message routing
pub struct MessageRouter {
    /// Routing strategy
    strategy: Box<dyn RoutingStrategy>,
    
    /// A2A client for communication
    client: Arc<dyn A2AClient>,
    
    /// Route cache
    cache: Arc<RouteCache>,
    
    /// Router metrics
    metrics: Arc<RouterMetrics>,
    
    /// Agent registry
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    
    /// Router configuration
    config: RouterConfig,
}

/// Agent information for routing
#[derive(Debug, Clone)]
pub struct AgentInfo {
    /// Agent card
    pub card: AgentCard,
    
    /// Available endpoints
    pub endpoints: Vec<AgentEndpoint>,
    
    /// Current load (0.0 - 1.0)
    pub load: f64,
    
    /// Response time statistics
    pub response_time: ResponseTimeStats,
    
    /// Health status
    pub health: HealthStatus,
    
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Response time statistics
#[derive(Debug, Clone)]
pub struct ResponseTimeStats {
    /// Average response time in milliseconds
    pub average_ms: f64,
    
    /// 95th percentile response time
    pub p95_ms: f64,
    
    /// 99th percentile response time
    pub p99_ms: f64,
    
    /// Sample count
    pub sample_count: u64,
}

/// Health status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Agent is healthy and available
    Healthy,
    
    /// Agent is degraded but still available
    Degraded,
    
    /// Agent is unhealthy and should not receive traffic
    Unhealthy,
    
    /// Health status is unknown
    Unknown,
}

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Maximum routing attempts
    pub max_attempts: u32,
    
    /// Request timeout
    pub timeout_ms: u64,
    
    /// Health check interval
    pub health_check_interval_ms: u64,
    
    /// Cache TTL for routes
    pub cache_ttl_ms: u64,
    
    /// Enable load balancing
    pub enable_load_balancing: bool,
    
    /// Enable failover
    pub enable_failover: bool,
    
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    
    /// Time window for failure counting (ms)
    pub time_window_ms: u64,
    
    /// Recovery timeout (ms)
    pub recovery_timeout_ms: u64,
}

/// Routing result
#[derive(Debug)]
pub struct RoutingResult {
    /// Response message
    pub response: Option<A2AMessage>,
    
    /// Selected agent
    pub selected_agent: String,
    
    /// Selected endpoint
    pub selected_endpoint: AgentEndpoint,
    
    /// Routing attempts made
    pub attempts: u32,
    
    /// Total routing time
    pub routing_time_ms: u64,
    
    /// Response time
    pub response_time_ms: u64,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(
        strategy: Box<dyn RoutingStrategy>,
        client: Arc<dyn A2AClient>,
        cache: Arc<RouteCache>,
        metrics: Arc<RouterMetrics>,
        config: RouterConfig,
    ) -> Self {
        Self {
            strategy,
            client,
            cache,
            metrics,
            agents: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Register an agent
    pub async fn register_agent(&self, agent_card: AgentCard, endpoints: Vec<AgentEndpoint>) -> A2AResult<()> {
        info!("Registering agent: {}", agent_card.id);
        
        let agent_info = AgentInfo {
            card: agent_card.clone(),
            endpoints,
            load: 0.0,
            response_time: ResponseTimeStats {
                average_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                sample_count: 0,
            },
            health: HealthStatus::Unknown,
            last_updated: Utc::now(),
        };
        
        self.agents.write().await.insert(agent_card.id.clone(), agent_info);
        
        // Start health monitoring
        self.start_health_monitoring(agent_card.id).await;
        
        Ok(())
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> A2AResult<()> {
        info!("Unregistering agent: {}", agent_id);
        
        self.agents.write().await.remove(agent_id);
        self.cache.invalidate_agent(agent_id).await;
        
        Ok(())
    }
    
    /// Route a message to its destination
    pub async fn route_message(&self, message: A2AMessage) -> Result<RoutingResult, RouterError> {
        let start_time = std::time::Instant::now();

        // 从元数据中获取目标Agent信息
        let target_agent_id = message.metadata.get("target_agent")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        debug!("Routing message {} to {}", message.message_id, target_agent_id);
        
        // Check cache first
        if let Some(cached_endpoint_url) = self.cache.get_route(&target_agent_id).await {
            // 从缓存的URL创建端点
            let cached_endpoint = AgentEndpoint::new(cached_endpoint_url, "http".to_string());
            if let Ok(result) = self.try_route_to_endpoint(&message, &cached_endpoint, &target_agent_id).await {
                self.metrics.record_cache_hit().await;
                return Ok(result);
            } else {
                // Cache miss or endpoint failed, invalidate cache
                self.cache.invalidate_agent(&target_agent_id).await;
                self.metrics.record_cache_miss().await;
            }
        }

        // Find available agents
        let agents = self.agents.read().await;
        let target_agents: Vec<_> = agents.values()
            .filter(|agent| {
                agent.card.id == target_agent_id &&
                agent.health != HealthStatus::Unhealthy
            })
            .cloned()
            .collect();
        drop(agents);

        if target_agents.is_empty() {
            return Err(RouterError::NoAvailableAgents(target_agent_id.clone()));
        }
        
        // Select best endpoint using routing strategy
        let selected_agent = self.strategy.select_agent(&target_agents, &message).await?;
        let selected_endpoint = self.strategy.select_endpoint(&selected_agent.endpoints, &message).await?;
        
        // Attempt routing with retries
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.config.max_attempts {
            attempts += 1;
            
            match self.try_route_to_endpoint(&message, &selected_endpoint, &target_agent_id).await {
                Ok(mut result) => {
                    result.attempts = attempts;
                    result.routing_time_ms = start_time.elapsed().as_millis() as u64;
                    
                    // Cache successful route
                    self.cache.cache_route(target_agent_id.clone(), selected_endpoint.url.clone()).await;
                    
                    // Update metrics
                    self.metrics.record_successful_route(
                        &selected_agent.card.id,
                        result.response_time_ms,
                    ).await;
                    
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Routing attempt {} failed: {}", attempts, e);

                    // Update agent health if this was a connectivity issue
                    if e.is_connectivity_error() {
                        self.update_agent_health(&selected_agent.card.id, HealthStatus::Degraded).await;
                    }

                    last_error = Some(e);
                    
                    // Try failover if enabled and we have more attempts
                    if self.config.enable_failover && attempts < self.config.max_attempts {
                        if let Some(failover_endpoint) = self.find_failover_endpoint(&target_agent_id, &selected_endpoint).await {
                            match self.try_route_to_endpoint(&message, &failover_endpoint, &target_agent_id).await {
                                Ok(mut result) => {
                                    result.attempts = attempts;
                                    result.routing_time_ms = start_time.elapsed().as_millis() as u64;
                                    return Ok(result);
                                }
                                Err(failover_error) => {
                                    warn!("Failover attempt failed: {}", failover_error);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // All attempts failed
        self.metrics.record_failed_route(&target_agent_id).await;
        Err(RouterError::RoutingFailed {
            agent_id: target_agent_id,
            attempts,
            last_error: last_error.map(|e| e.to_string()),
        })
    }
    
    /// Try to route message to a specific endpoint
    async fn try_route_to_endpoint(&self, message: &A2AMessage, endpoint: &AgentEndpoint, target_agent_id: &str) -> Result<RoutingResult, RouterError> {
        let start_time = std::time::Instant::now();
        
        debug!("Attempting to route to endpoint: {}", endpoint.url);
        
        match self.client.send_message(endpoint, message).await {
            Ok(response) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Update agent response time stats
                self.update_response_time_stats(&target_agent_id, response_time_ms).await;

                Ok(RoutingResult {
                    response: Some(response),
                    selected_agent: target_agent_id.to_string(),
                    selected_endpoint: endpoint.clone(),
                    attempts: 1,
                    routing_time_ms: response_time_ms,
                    response_time_ms,
                })
            }
            Err(e) => {
                error!("Failed to send message to {}: {}", endpoint.url, e);
                Err(RouterError::from(e))
            }
        }
    }
    
    /// Find a failover endpoint for an agent
    async fn find_failover_endpoint(&self, agent_id: &str, failed_endpoint: &AgentEndpoint) -> Option<AgentEndpoint> {
        let agents = self.agents.read().await;
        if let Some(agent_info) = agents.get(agent_id) {
            // Find a different endpoint for the same agent
            agent_info.endpoints.iter()
                .find(|ep| ep.url != failed_endpoint.url &&
                          self.is_endpoint_healthy(ep))
                .cloned()
        } else {
            None
        }
    }
    
    /// Check if an endpoint is healthy
    fn is_endpoint_healthy(&self, _endpoint: &AgentEndpoint) -> bool {
        // This would check circuit breaker state, recent failures, etc.
        // For now, assume all endpoints are healthy
        true
    }
    
    /// Update agent health status
    async fn update_agent_health(&self, agent_id: &str, health: HealthStatus) {
        let mut agents = self.agents.write().await;
        if let Some(agent_info) = agents.get_mut(agent_id) {
            agent_info.health = health;
            agent_info.last_updated = Utc::now();
        }
    }
    
    /// Update response time statistics for an agent
    async fn update_response_time_stats(&self, agent_id: &str, response_time_ms: u64) {
        let mut agents = self.agents.write().await;
        if let Some(agent_info) = agents.get_mut(agent_id) {
            let stats = &mut agent_info.response_time;
            
            // Simple moving average calculation
            let new_count = stats.sample_count + 1;
            stats.average_ms = (stats.average_ms * stats.sample_count as f64 + response_time_ms as f64) / new_count as f64;
            stats.sample_count = new_count;
            
            // Update percentiles (simplified)
            if response_time_ms as f64 > stats.p95_ms {
                stats.p95_ms = response_time_ms as f64;
            }
            if response_time_ms as f64 > stats.p99_ms {
                stats.p99_ms = response_time_ms as f64;
            }
            
            agent_info.last_updated = Utc::now();
        }
    }
    
    /// Start health monitoring for an agent
    async fn start_health_monitoring(&self, agent_id: String) {
        let client = self.client.clone();
        let agents = self.agents.clone();
        let interval = self.config.health_check_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval));
            
            loop {
                interval.tick().await;
                
                // Get agent endpoints
                let endpoints = {
                    let agents_guard = agents.read().await;
                    if let Some(agent_info) = agents_guard.get(&agent_id) {
                        agent_info.endpoints.clone()
                    } else {
                        break; // Agent was removed
                    }
                };
                
                // Check health of all endpoints
                let mut healthy_count = 0;
                for endpoint in &endpoints {
                    if client.health_check(endpoint).await.unwrap_or(false) {
                        healthy_count += 1;
                    }
                }
                
                // Update health status
                let health = if healthy_count == 0 {
                    HealthStatus::Unhealthy
                } else if healthy_count < endpoints.len() {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };
                
                let mut agents_guard = agents.write().await;
                if let Some(agent_info) = agents_guard.get_mut(&agent_id) {
                    agent_info.health = health;
                    agent_info.last_updated = Utc::now();
                }
            }
        });
    }
    
    /// Get agent information
    pub async fn get_agent_info(&self, agent_id: &str) -> Option<AgentInfo> {
        self.agents.read().await.get(agent_id).cloned()
    }

    /// Get router statistics
    pub async fn get_stats(&self) -> RouterStats {
        let agents = self.agents.read().await;
        let total_agents = agents.len();
        let healthy_agents = agents.values().filter(|a| a.health == HealthStatus::Healthy).count();
        let degraded_agents = agents.values().filter(|a| a.health == HealthStatus::Degraded).count();
        let unhealthy_agents = agents.values().filter(|a| a.health == HealthStatus::Unhealthy).count();
        
        RouterStats {
            total_agents,
            healthy_agents,
            degraded_agents,
            unhealthy_agents,
            cache_hit_rate: self.metrics.get_cache_hit_rate().await,
            total_routes: {
                let stats = self.metrics.get_route_stats().await;
                stats.total_requests
            },
            successful_routes: {
                let stats = self.metrics.get_route_stats().await;
                stats.successful_routes
            },
            failed_routes: {
                let stats = self.metrics.get_route_stats().await;
                stats.failed_routes
            },
        }
    }
}

/// Router statistics
#[derive(Debug)]
pub struct RouterStats {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub degraded_agents: usize,
    pub unhealthy_agents: usize,
    pub cache_hit_rate: f64,
    pub total_routes: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            timeout_ms: 30000,
            health_check_interval_ms: 30000,
            cache_ttl_ms: 300000, // 5 minutes
            enable_load_balancing: true,
            enable_failover: true,
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                time_window_ms: 60000,
                recovery_timeout_ms: 30000,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RoundRobinStrategy;
    use agentx_a2a::{AgentStatus, TrustLevel};
    use std::collections::HashMap;

    fn create_test_router() -> MessageRouter {
        let strategy = Box::new(RoundRobinStrategy::new());
        let client = Arc::new(DefaultA2AClient);
        let cache = Arc::new(RouteCache::new(crate::CacheConfig::default()));
        let metrics = Arc::new(RouterMetrics::new());
        let config = RouterConfig::default();

        MessageRouter::new(strategy, client, cache, metrics, config)
    }

    fn create_test_agent_card(id: &str) -> AgentCard {
        AgentCard {
            id: id.to_string(),
            name: format!("Test Agent {}", id),
            description: "Test agent for routing".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            status: AgentStatus::Online,
            supported_versions: vec!["1.0.0".to_string()],
            tags: vec![],
            interaction_modalities: vec![],
            ux_capabilities: None,
            trust_level: TrustLevel::Public,
            supported_task_types: vec![],
        }
    }

    #[tokio::test]
    async fn test_router_creation() {
        let router = create_test_router();
        let stats = router.get_stats().await;

        assert_eq!(stats.total_agents, 0);
        assert_eq!(stats.healthy_agents, 0);
        assert_eq!(stats.degraded_agents, 0);
        assert_eq!(stats.unhealthy_agents, 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let router = create_test_router();
        let agent_card = create_test_agent_card("test_agent_1");
        let endpoints = vec![
            AgentEndpoint::new("http://localhost:8080".to_string(), "http".to_string()),
            AgentEndpoint::new("grpc://localhost:9090".to_string(), "grpc".to_string()),
        ];

        // 注册Agent
        let result = router.register_agent(agent_card, endpoints).await;
        assert!(result.is_ok());

        // 验证Agent已注册
        let stats = router.get_stats().await;
        assert_eq!(stats.total_agents, 1);

        // 验证Agent信息
        let agent_info = router.get_agent_info("test_agent_1").await;
        assert!(agent_info.is_some());
        let info = agent_info.unwrap();
        assert_eq!(info.card.id, "test_agent_1");
        assert_eq!(info.endpoints.len(), 2);
    }

    #[tokio::test]
    async fn test_agent_unregistration() {
        let router = create_test_router();
        let agent_card = create_test_agent_card("test_agent_2");
        let endpoints = vec![
            AgentEndpoint::new("http://localhost:8081".to_string(), "http".to_string()),
        ];

        // 注册Agent
        router.register_agent(agent_card, endpoints).await.unwrap();
        assert_eq!(router.get_stats().await.total_agents, 1);

        // 注销Agent
        let result = router.unregister_agent("test_agent_2").await;
        assert!(result.is_ok());

        // 验证Agent已注销
        let stats = router.get_stats().await;
        assert_eq!(stats.total_agents, 0);

        let agent_info = router.get_agent_info("test_agent_2").await;
        assert!(agent_info.is_none());
    }

    #[tokio::test]
    async fn test_message_routing_no_agents() {
        let router = create_test_router();

        // 创建测试消息
        let mut message = A2AMessage::user_message("Hello, world!".to_string());
        message.metadata.insert("target_agent".to_string(), serde_json::Value::String("nonexistent_agent".to_string()));

        // 尝试路由消息
        let result = router.route_message(message).await;
        assert!(result.is_err());

        if let Err(RouterError::NoAvailableAgents(agent_id)) = result {
            assert_eq!(agent_id, "nonexistent_agent");
        } else {
            panic!("Expected NoAvailableAgents error");
        }
    }

    #[tokio::test]
    async fn test_message_routing_with_agent() {
        let router = create_test_router();
        let agent_card = create_test_agent_card("test_agent_3");
        let endpoints = vec![
            AgentEndpoint::new("http://localhost:8082".to_string(), "http".to_string()),
        ];

        // 注册Agent
        router.register_agent(agent_card, endpoints).await.unwrap();

        // 创建测试消息
        let mut message = A2AMessage::user_message("Hello, agent!".to_string());
        message.metadata.insert("target_agent".to_string(), serde_json::Value::String("test_agent_3".to_string()));

        // 路由消息
        let result = router.route_message(message).await;
        assert!(result.is_ok());

        let routing_result = result.unwrap();
        assert_eq!(routing_result.selected_agent, "test_agent_3");
        assert!(routing_result.response.is_some());
        assert_eq!(routing_result.attempts, 1);
    }

    #[tokio::test]
    async fn test_router_metrics() {
        let router = create_test_router();
        let agent_card = create_test_agent_card("test_agent_4");
        let endpoints = vec![
            AgentEndpoint::new("http://localhost:8083".to_string(), "http".to_string()),
        ];

        // 注册Agent
        router.register_agent(agent_card, endpoints).await.unwrap();

        // 创建并路由多个消息到不同的目标（避免缓存影响）
        for i in 0..5 {
            let mut message = A2AMessage::user_message(format!("Message {}", i));
            // 使用不同的目标Agent ID来避免缓存命中
            message.metadata.insert("target_agent".to_string(), serde_json::Value::String(format!("test_agent_4_{}", i)));

            // 为每个消息注册对应的Agent
            let agent_card = create_test_agent_card(&format!("test_agent_4_{}", i));
            let endpoints = vec![
                AgentEndpoint::new(format!("http://localhost:808{}", 3 + i), "http".to_string()),
            ];
            router.register_agent(agent_card, endpoints).await.unwrap();

            let result = router.route_message(message).await;
            assert!(result.is_ok());
        }

        // 验证指标
        let performance = router.metrics.get_performance_summary().await;
        assert_eq!(performance.total_requests, 5);
        assert_eq!(performance.successful_routes, 5);
        assert_eq!(performance.failed_routes, 0);
        assert_eq!(performance.success_rate, 1.0);
        assert!(performance.avg_routing_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let router = create_test_router();
        let agent_card = create_test_agent_card("test_agent_5");
        let endpoints = vec![
            AgentEndpoint::new("http://localhost:8084".to_string(), "http".to_string()),
        ];

        // 注册Agent
        router.register_agent(agent_card, endpoints).await.unwrap();

        // 第一次路由消息（缓存未命中）
        let mut message1 = A2AMessage::user_message("First message".to_string());
        message1.metadata.insert("target_agent".to_string(), serde_json::Value::String("test_agent_5".to_string()));

        let result1 = router.route_message(message1).await;
        assert!(result1.is_ok());

        // 第二次路由相同目标的消息（应该缓存命中）
        let mut message2 = A2AMessage::user_message("Second message".to_string());
        message2.metadata.insert("target_agent".to_string(), serde_json::Value::String("test_agent_5".to_string()));

        let result2 = router.route_message(message2).await;
        assert!(result2.is_ok());

        // 验证缓存统计
        let cache_hit_rate = router.metrics.get_cache_hit_rate().await;
        assert!(cache_hit_rate > 0.0);
    }
}
