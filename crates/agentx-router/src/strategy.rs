//! 路由策略实现
//! 
//! 提供多种路由策略，包括轮询、最少连接、加权轮询等

use crate::{AgentInfo, RouterError};
use agentx_a2a::{A2AMessage, AgentEndpoint};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::debug;

/// 路由策略特征
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// 选择目标Agent
    async fn select_agent(
        &self,
        agents: &[AgentInfo],
        message: &A2AMessage,
    ) -> Result<AgentInfo, RouterError>;

    /// 选择目标端点
    async fn select_endpoint(
        &self,
        endpoints: &[AgentEndpoint],
        message: &A2AMessage,
    ) -> Result<AgentEndpoint, RouterError>;

    /// 策略名称
    fn name(&self) -> &str;
}

/// 轮询策略
pub struct RoundRobinStrategy {
    counter: Arc<AtomicUsize>,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl RoutingStrategy for RoundRobinStrategy {
    async fn select_agent(
        &self,
        agents: &[AgentInfo],
        _message: &A2AMessage,
    ) -> Result<AgentInfo, RouterError> {
        if agents.is_empty() {
            return Err(RouterError::NoAvailableAgents("".to_string()));
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % agents.len();
        debug!("轮询策略选择Agent索引: {}", index);
        
        Ok(agents[index].clone())
    }

    async fn select_endpoint(
        &self,
        endpoints: &[AgentEndpoint],
        _message: &A2AMessage,
    ) -> Result<AgentEndpoint, RouterError> {
        if endpoints.is_empty() {
            return Err(RouterError::NoAvailableEndpoints);
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % endpoints.len();
        debug!("轮询策略选择端点索引: {}", index);
        
        Ok(endpoints[index].clone())
    }

    fn name(&self) -> &str {
        "round_robin"
    }
}

/// 最少连接策略
pub struct LeastConnectionsStrategy;

impl LeastConnectionsStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RoutingStrategy for LeastConnectionsStrategy {
    async fn select_agent(
        &self,
        agents: &[AgentInfo],
        _message: &A2AMessage,
    ) -> Result<AgentInfo, RouterError> {
        if agents.is_empty() {
            return Err(RouterError::NoAvailableAgents("".to_string()));
        }

        // 选择负载最低的Agent
        let selected = agents
            .iter()
            .min_by(|a, b| a.load.partial_cmp(&b.load).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        debug!("最少连接策略选择Agent: {} (负载: {:.2})", selected.card.id, selected.load);
        
        Ok(selected.clone())
    }

    async fn select_endpoint(
        &self,
        endpoints: &[AgentEndpoint],
        _message: &A2AMessage,
    ) -> Result<AgentEndpoint, RouterError> {
        if endpoints.is_empty() {
            return Err(RouterError::NoAvailableEndpoints);
        }

        // 简单选择第一个可用端点
        // 在真实实现中，这里应该考虑端点的连接数
        debug!("最少连接策略选择端点: {}", endpoints[0].url);
        
        Ok(endpoints[0].clone())
    }

    fn name(&self) -> &str {
        "least_connections"
    }
}

/// 加权轮询策略
pub struct WeightedRoundRobinStrategy {
    counter: Arc<AtomicUsize>,
}

impl WeightedRoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl RoutingStrategy for WeightedRoundRobinStrategy {
    async fn select_agent(
        &self,
        agents: &[AgentInfo],
        _message: &A2AMessage,
    ) -> Result<AgentInfo, RouterError> {
        if agents.is_empty() {
            return Err(RouterError::NoAvailableAgents("".to_string()));
        }

        // 基于响应时间计算权重（响应时间越短权重越高）
        let mut weighted_agents = Vec::new();
        for agent in agents {
            let weight = if agent.response_time.average_ms > 0.0 {
                (1000.0 / agent.response_time.average_ms).max(1.0) as usize
            } else {
                10 // 默认权重
            };
            
            for _ in 0..weight {
                weighted_agents.push(agent.clone());
            }
        }

        if weighted_agents.is_empty() {
            return Ok(agents[0].clone());
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % weighted_agents.len();
        debug!("加权轮询策略选择Agent: {}", weighted_agents[index].card.id);
        
        Ok(weighted_agents[index].clone())
    }

    async fn select_endpoint(
        &self,
        endpoints: &[AgentEndpoint],
        _message: &A2AMessage,
    ) -> Result<AgentEndpoint, RouterError> {
        if endpoints.is_empty() {
            return Err(RouterError::NoAvailableEndpoints);
        }

        let index = self.counter.fetch_add(1, Ordering::Relaxed) % endpoints.len();
        debug!("加权轮询策略选择端点: {}", endpoints[index].url);
        
        Ok(endpoints[index].clone())
    }

    fn name(&self) -> &str {
        "weighted_round_robin"
    }
}

/// 响应时间策略
pub struct ResponseTimeStrategy;

impl ResponseTimeStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RoutingStrategy for ResponseTimeStrategy {
    async fn select_agent(
        &self,
        agents: &[AgentInfo],
        _message: &A2AMessage,
    ) -> Result<AgentInfo, RouterError> {
        if agents.is_empty() {
            return Err(RouterError::NoAvailableAgents("".to_string()));
        }

        // 选择平均响应时间最短的Agent
        let selected = agents
            .iter()
            .min_by(|a, b| {
                let a_time = if a.response_time.average_ms > 0.0 { a.response_time.average_ms } else { f64::MAX };
                let b_time = if b.response_time.average_ms > 0.0 { b.response_time.average_ms } else { f64::MAX };
                a_time.partial_cmp(&b_time).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        debug!("响应时间策略选择Agent: {} (平均响应时间: {:.2}ms)", 
               selected.card.id, selected.response_time.average_ms);
        
        Ok(selected.clone())
    }

    async fn select_endpoint(
        &self,
        endpoints: &[AgentEndpoint],
        _message: &A2AMessage,
    ) -> Result<AgentEndpoint, RouterError> {
        if endpoints.is_empty() {
            return Err(RouterError::NoAvailableEndpoints);
        }

        // 选择第一个可用端点
        debug!("响应时间策略选择端点: {}", endpoints[0].url);
        
        Ok(endpoints[0].clone())
    }

    fn name(&self) -> &str {
        "response_time"
    }
}

/// 创建默认策略
pub fn create_default_strategy() -> Box<dyn RoutingStrategy> {
    Box::new(RoundRobinStrategy::new())
}

/// 根据名称创建策略
pub fn create_strategy(name: &str) -> Result<Box<dyn RoutingStrategy>, RouterError> {
    match name {
        "round_robin" => Ok(Box::new(RoundRobinStrategy::new())),
        "least_connections" => Ok(Box::new(LeastConnectionsStrategy::new())),
        "weighted_round_robin" => Ok(Box::new(WeightedRoundRobinStrategy::new())),
        "response_time" => Ok(Box::new(ResponseTimeStrategy::new())),
        _ => Err(RouterError::InvalidStrategy(name.to_string())),
    }
}
