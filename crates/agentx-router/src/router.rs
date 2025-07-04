//! Message Router Implementation
//! 
//! This module implements intelligent message routing with load balancing,
//! failover, and performance optimization.

use agentx_a2a::{A2AMessage, AgentCard, A2AError, A2AResult, A2AClient, AgentEndpoint};
use crate::{RoutingStrategy, RouterMetrics, RouteCache, RouterError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};

/// Message Router for intelligent A2A message routing
pub struct MessageRouter {
    /// Routing strategy
    strategy: Box<dyn RoutingStrategy>,
    
    /// A2A client for communication
    client: Arc<A2AClient>,
    
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
        client: Arc<A2AClient>,
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
        
        debug!("Routing message {} to {}", message.id, message.to);
        
        // Check cache first
        if let Some(cached_route) = self.cache.get_route(&message.to).await {
            if let Ok(result) = self.try_route_to_endpoint(&message, &cached_route).await {
                self.metrics.record_cache_hit().await;
                return Ok(result);
            } else {
                // Cache miss or endpoint failed, invalidate cache
                self.cache.invalidate_agent(&message.to).await;
                self.metrics.record_cache_miss().await;
            }
        }
        
        // Find available agents
        let agents = self.agents.read().await;
        let target_agents: Vec<_> = agents.values()
            .filter(|agent| {
                agent.card.id == message.to && 
                agent.health != HealthStatus::Unhealthy
            })
            .cloned()
            .collect();
        drop(agents);
        
        if target_agents.is_empty() {
            return Err(RouterError::NoAvailableAgents(message.to.clone()));
        }
        
        // Select best endpoint using routing strategy
        let selected_agent = self.strategy.select_agent(&target_agents, &message).await?;
        let selected_endpoint = self.strategy.select_endpoint(&selected_agent.endpoints, &message).await?;
        
        // Attempt routing with retries
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.config.max_attempts {
            attempts += 1;
            
            match self.try_route_to_endpoint(&message, &selected_endpoint).await {
                Ok(mut result) => {
                    result.attempts = attempts;
                    result.routing_time_ms = start_time.elapsed().as_millis() as u64;
                    
                    // Cache successful route
                    self.cache.cache_route(message.to.clone(), selected_endpoint.clone()).await;
                    
                    // Update metrics
                    self.metrics.record_successful_route(
                        &selected_agent.card.id,
                        result.response_time_ms,
                    ).await;
                    
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Routing attempt {} failed: {}", attempts, e);
                    last_error = Some(e);
                    
                    // Update agent health if this was a connectivity issue
                    if e.is_connectivity_error() {
                        self.update_agent_health(&selected_agent.card.id, HealthStatus::Degraded).await;
                    }
                    
                    // Try failover if enabled and we have more attempts
                    if self.config.enable_failover && attempts < self.config.max_attempts {
                        if let Some(failover_endpoint) = self.find_failover_endpoint(&message.to, &selected_endpoint).await {
                            match self.try_route_to_endpoint(&message, &failover_endpoint).await {
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
        self.metrics.record_failed_route(&message.to).await;
        Err(RouterError::RoutingFailed {
            agent_id: message.to,
            attempts,
            last_error: last_error.map(|e| e.to_string()),
        })
    }
    
    /// Try to route message to a specific endpoint
    async fn try_route_to_endpoint(&self, message: &A2AMessage, endpoint: &AgentEndpoint) -> Result<RoutingResult, RouterError> {
        let start_time = std::time::Instant::now();
        
        debug!("Attempting to route to endpoint: {}", endpoint.base_url);
        
        match self.client.send_message(endpoint, message).await {
            Ok(response) => {
                let response_time_ms = start_time.elapsed().as_millis() as u64;
                
                // Update agent response time stats
                self.update_response_time_stats(&endpoint.agent_id, response_time_ms).await;
                
                Ok(RoutingResult {
                    response,
                    selected_agent: endpoint.agent_id.clone(),
                    selected_endpoint: endpoint.clone(),
                    attempts: 1,
                    routing_time_ms: response_time_ms,
                    response_time_ms,
                })
            }
            Err(e) => {
                error!("Failed to send message to {}: {}", endpoint.base_url, e);
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
                .find(|ep| ep.base_url != failed_endpoint.base_url && 
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
            cache_hit_rate: self.cache.get_hit_rate().await,
            total_routes: self.metrics.get_total_routes().await,
            successful_routes: self.metrics.get_successful_routes().await,
            failed_routes: self.metrics.get_failed_routes().await,
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
    use agentx_a2a::{AgentCard, Endpoint, ClientConfig};
    use crate::{RoundRobinStrategy, InMemoryCache, InMemoryMetrics};

    #[tokio::test]
    async fn test_router_creation() {
        let strategy = Box::new(RoundRobinStrategy::new());
        let client = Arc::new(A2AClient::new(ClientConfig::default()));
        let cache = Arc::new(InMemoryCache::new());
        let metrics = Arc::new(InMemoryMetrics::new());
        let config = RouterConfig::default();
        
        let router = MessageRouter::new(strategy, client, cache, metrics, config);
        let stats = router.get_stats().await;
        
        assert_eq!(stats.total_agents, 0);
    }
}
