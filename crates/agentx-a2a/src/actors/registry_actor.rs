//! Agent Registry Actor
//! 
//! This actor manages agent registration, discovery, and capability matching
//! with high concurrency and consistency using the Actix actor model.

use actix::prelude::*;
use crate::{
    AgentCard, CapabilityQuery, CapabilityMatch, CapabilityDiscovery,
    A2AError, A2AResult, AgentStatus
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

/// Agent Registry Actor
pub struct AgentRegistryActor {
    /// Capability discovery service
    discovery: CapabilityDiscovery,
    
    /// Registry statistics
    stats: RegistryStats,
    
    /// Agent health cache
    health_cache: HashMap<String, AgentHealthInfo>,
    
    /// Registry configuration
    config: RegistryConfig,
}

/// Registry statistics
#[derive(Debug, Clone, Default)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub degraded_agents: usize,
    pub unhealthy_agents: usize,
    pub discovery_queries: u64,
    pub registration_events: u64,
}

/// Agent health information
#[derive(Debug, Clone)]
pub struct AgentHealthInfo {
    pub status: AgentStatus,
    pub last_seen: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_count: u32,
}

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub health_check_interval_ms: u64,
    pub agent_timeout_ms: u64,
    pub max_error_count: u32,
    pub enable_auto_cleanup: bool,
}

/// Message to register an agent
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct RegisterAgent {
    pub agent_card: AgentCard,
}

/// Message to unregister an agent
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct UnregisterAgent {
    pub agent_id: String,
}

/// Message to update agent status
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct UpdateAgentStatus {
    pub agent_id: String,
    pub status: AgentStatus,
    pub response_time_ms: Option<u64>,
}

/// Message to discover agents
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<Vec<CapabilityMatch>>")]
pub struct DiscoverAgents {
    pub query: CapabilityQuery,
}

/// Message to get agent by ID
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<Option<AgentCard>>")]
pub struct GetAgent {
    pub agent_id: String,
}

/// Message to list all agents
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<Vec<AgentCard>>")]
pub struct ListAgents {
    pub filter: Option<AgentFilter>,
}

/// Agent filter for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFilter {
    pub status: Option<AgentStatus>,
    pub capabilities: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Message to get registry statistics
#[derive(Message, Debug)]
#[rtype(result = "RegistryStats")]
pub struct GetRegistryStats;

/// Message for periodic health check
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PeriodicHealthCheck;

impl Actor for AgentRegistryActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Agent Registry Actor started");
        
        // Start periodic health checks
        if self.config.health_check_interval_ms > 0 {
            self.start_health_monitoring(ctx);
        }
        
        // Start periodic cleanup
        if self.config.enable_auto_cleanup {
            self.start_cleanup_task(ctx);
        }
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Agent Registry Actor stopped");
    }
}

impl AgentRegistryActor {
    /// Create a new Agent Registry Actor
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            discovery: CapabilityDiscovery::new(),
            stats: RegistryStats::default(),
            health_cache: HashMap::new(),
            config,
        }
    }
    
    /// Start health monitoring
    fn start_health_monitoring(&self, ctx: &mut Context<Self>) {
        let interval = std::time::Duration::from_millis(self.config.health_check_interval_ms);
        
        ctx.run_interval(interval, |_actor, ctx| {
            ctx.address().do_send(PeriodicHealthCheck);
        });
    }
    
    /// Start cleanup task for expired agents
    fn start_cleanup_task(&self, ctx: &mut Context<Self>) {
        let interval = std::time::Duration::from_secs(300); // 5 minutes
        
        ctx.run_interval(interval, |actor, _ctx| {
            actor.cleanup_expired_agents();
        });
    }
    
    /// Cleanup expired agents
    fn cleanup_expired_agents(&mut self) {
        let now = Utc::now();
        let timeout = chrono::Duration::milliseconds(self.config.agent_timeout_ms as i64);
        
        let expired_agents: Vec<String> = self.health_cache
            .iter()
            .filter(|(_, health)| {
                now.signed_duration_since(health.last_seen) > timeout ||
                health.error_count > self.config.max_error_count
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for agent_id in expired_agents {
            warn!("Removing expired agent: {}", agent_id);
            self.discovery.unregister_agent(&agent_id);
            self.health_cache.remove(&agent_id);
            self.update_stats();
        }
    }
    
    /// Update registry statistics
    fn update_stats(&mut self) {
        let agents = self.discovery.list_agents();
        self.stats.total_agents = agents.len();
        
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        
        for agent in agents {
            match agent.status {
                AgentStatus::Online => healthy += 1,
                AgentStatus::Busy => degraded += 1,
                AgentStatus::Maintenance => degraded += 1,
                AgentStatus::Offline => unhealthy += 1,
                AgentStatus::Unknown => unhealthy += 1,
            }
        }
        
        self.stats.healthy_agents = healthy;
        self.stats.degraded_agents = degraded;
        self.stats.unhealthy_agents = unhealthy;
    }
    
    /// Update agent health information
    fn update_agent_health(&mut self, agent_id: &str, status: AgentStatus, response_time_ms: Option<u64>) {
        let health_info = self.health_cache.entry(agent_id.to_string()).or_insert_with(|| {
            AgentHealthInfo {
                status: AgentStatus::Unknown,
                last_seen: Utc::now(),
                response_time_ms: None,
                error_count: 0,
            }
        });
        
        health_info.status = status;
        health_info.last_seen = Utc::now();
        health_info.response_time_ms = response_time_ms;
        
        // Reset error count on successful health check
        if matches!(status, AgentStatus::Online) {
            health_info.error_count = 0;
        }
    }
}

/// Handle RegisterAgent
impl Handler<RegisterAgent> for AgentRegistryActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: RegisterAgent, _ctx: &mut Self::Context) -> Self::Result {
        info!("Registering agent: {}", msg.agent_card.id);
        
        // Validate agent card
        if msg.agent_card.id.is_empty() {
            return Err(A2AError::validation("Agent ID cannot be empty"));
        }
        
        if msg.agent_card.name.is_empty() {
            return Err(A2AError::validation("Agent name cannot be empty"));
        }
        
        // Register with discovery service
        self.discovery.register_agent(msg.agent_card.clone());
        
        // Initialize health info
        self.update_agent_health(&msg.agent_card.id, msg.agent_card.status.clone(), None);
        
        // Update statistics
        self.stats.registration_events += 1;
        self.update_stats();
        
        debug!("Agent {} registered successfully", msg.agent_card.id);
        Ok(())
    }
}

/// Handle UnregisterAgent
impl Handler<UnregisterAgent> for AgentRegistryActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: UnregisterAgent, _ctx: &mut Self::Context) -> Self::Result {
        info!("Unregistering agent: {}", msg.agent_id);
        
        // Remove from discovery service
        self.discovery.unregister_agent(&msg.agent_id);
        
        // Remove health info
        self.health_cache.remove(&msg.agent_id);
        
        // Update statistics
        self.update_stats();
        
        debug!("Agent {} unregistered successfully", msg.agent_id);
        Ok(())
    }
}

/// Handle UpdateAgentStatus
impl Handler<UpdateAgentStatus> for AgentRegistryActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: UpdateAgentStatus, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Updating agent status: {} -> {:?}", msg.agent_id, msg.status);
        
        // Update health information
        self.update_agent_health(&msg.agent_id, msg.status, msg.response_time_ms);
        
        // Update agent card in discovery service if needed
        if let Some(mut agent_card) = self.discovery.get_agent(&msg.agent_id).cloned() {
            agent_card.status = msg.status;
            agent_card.updated_at = Utc::now();
            self.discovery.update_agent(agent_card);
        }
        
        // Update statistics
        self.update_stats();
        
        Ok(())
    }
}

/// Handle DiscoverAgents
impl Handler<DiscoverAgents> for AgentRegistryActor {
    type Result = A2AResult<Vec<CapabilityMatch>>;
    
    fn handle(&mut self, msg: DiscoverAgents, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Discovering agents with query: {:?}", msg.query);
        
        let matches = self.discovery.discover(&msg.query);
        
        // Update statistics
        self.stats.discovery_queries += 1;
        
        info!("Found {} matching agents", matches.len());
        Ok(matches)
    }
}

/// Handle GetAgent
impl Handler<GetAgent> for AgentRegistryActor {
    type Result = A2AResult<Option<AgentCard>>;
    
    fn handle(&mut self, msg: GetAgent, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Getting agent: {}", msg.agent_id);
        
        let agent = self.discovery.get_agent(&msg.agent_id).cloned();
        Ok(agent)
    }
}

/// Handle ListAgents
impl Handler<ListAgents> for AgentRegistryActor {
    type Result = A2AResult<Vec<AgentCard>>;
    
    fn handle(&mut self, msg: ListAgents, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Listing agents with filter: {:?}", msg.filter);
        
        let mut agents = self.discovery.list_agents().into_iter().cloned().collect::<Vec<_>>();
        
        // Apply filters if provided
        if let Some(filter) = msg.filter {
            if let Some(status) = filter.status {
                agents.retain(|agent| agent.status == status);
            }
            
            if let Some(capabilities) = filter.capabilities {
                agents.retain(|agent| {
                    capabilities.iter().any(|cap| agent.has_capability(cap))
                });
            }
            
            if let Some(tags) = filter.tags {
                agents.retain(|agent| {
                    tags.iter().any(|tag| agent.tags.contains(tag))
                });
            }
        }
        
        debug!("Returning {} agents", agents.len());
        Ok(agents)
    }
}

/// Handle GetRegistryStats
impl Handler<GetRegistryStats> for AgentRegistryActor {
    type Result = RegistryStats;
    
    fn handle(&mut self, _msg: GetRegistryStats, _ctx: &mut Self::Context) -> Self::Result {
        self.stats.clone()
    }
}

/// Handle PeriodicHealthCheck
impl Handler<PeriodicHealthCheck> for AgentRegistryActor {
    type Result = ();
    
    fn handle(&mut self, _msg: PeriodicHealthCheck, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Performing periodic health check");
        
        // In a real implementation, this would:
        // 1. Check health of all registered agents
        // 2. Update their status based on health check results
        // 3. Remove unresponsive agents
        
        // For now, just update statistics
        self.update_stats();
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            health_check_interval_ms: 30000, // 30 seconds
            agent_timeout_ms: 300000,        // 5 minutes
            max_error_count: 5,
            enable_auto_cleanup: true,
        }
    }
}
