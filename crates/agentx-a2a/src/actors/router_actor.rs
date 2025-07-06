//! Message Router Actor
//! 
//! This actor handles intelligent message routing with load balancing,
//! failover, and performance optimization using the Actix actor model.

use actix::prelude::*;
use crate::{A2AMessage, A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Message Router Actor
pub struct MessageRouterActor {
    /// Routing statistics
    stats: RouterStats,
    
    /// Route cache
    route_cache: HashMap<String, CachedRoute>,
    
    /// Router configuration
    config: RouterConfig,
}

/// Router statistics
#[derive(Debug, Clone, Default)]
pub struct RouterStats {
    pub total_routes: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Cached route information
#[derive(Debug, Clone)]
pub struct CachedRoute {
    pub target_endpoint: String,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub success_count: u32,
    pub failure_count: u32,
}

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub cache_ttl_seconds: u64,
    pub max_cache_size: usize,
    pub enable_load_balancing: bool,
}

/// Message to route an A2A message
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<RouteResult>")]
pub struct RouteMessage {
    pub message: A2AMessage,
    pub target_endpoints: Vec<String>,
}

/// Route result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub selected_endpoint: String,
    pub route_time_ms: u64,
    pub cache_hit: bool,
}

/// Message to get router statistics
#[derive(Message, Debug)]
#[rtype(result = "A2AResult<RouterStats>")]
pub struct GetRouterStats;

impl Actor for MessageRouterActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Message Router Actor started");
        
        // Start cache cleanup task
        self.start_cache_cleanup(ctx);
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Message Router Actor stopped");
    }
}

impl MessageRouterActor {
    /// Create a new Message Router Actor
    pub fn new(config: RouterConfig) -> Self {
        Self {
            stats: RouterStats::default(),
            route_cache: HashMap::new(),
            config,
        }
    }
    
    /// Start cache cleanup task
    fn start_cache_cleanup(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(
            std::time::Duration::from_secs(60),
            |actor, _ctx| {
                actor.cleanup_cache();
            }
        );
    }
    
    /// Cleanup expired cache entries
    fn cleanup_cache(&mut self) {
        let now = chrono::Utc::now();
        let ttl = chrono::Duration::seconds(self.config.cache_ttl_seconds as i64);
        
        self.route_cache.retain(|_, route| {
            now.signed_duration_since(route.last_used) < ttl
        });
        
        debug!("Cache cleanup completed, {} entries remaining", self.route_cache.len());
    }
    
    /// Select best endpoint for routing
    fn select_endpoint(&self, endpoints: &[String], _message: &A2AMessage) -> String {
        if endpoints.is_empty() {
            return String::new();
        }
        
        // Cache lookup simplified - using message_id as key for now
        // In a real implementation, this would use proper agent addressing
        
        // Simple round-robin selection for now
        let index = (self.stats.total_routes as usize) % endpoints.len();
        endpoints[index].clone()
    }
    
    /// Update route cache
    #[allow(dead_code)]
    fn update_cache(&mut self, agent_id: &str, endpoint: &str, success: bool) {
        let route = self.route_cache.entry(agent_id.to_string()).or_insert_with(|| {
            CachedRoute {
                target_endpoint: endpoint.to_string(),
                last_used: chrono::Utc::now(),
                success_count: 0,
                failure_count: 0,
            }
        });
        
        route.last_used = chrono::Utc::now();
        route.target_endpoint = endpoint.to_string();
        
        if success {
            route.success_count += 1;
        } else {
            route.failure_count += 1;
        }
        
        // Limit cache size
        if self.route_cache.len() > self.config.max_cache_size {
            // Remove oldest entry
            if let Some((oldest_key, _)) = self.route_cache
                .iter()
                .min_by_key(|(_, route)| route.last_used)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                self.route_cache.remove(&oldest_key);
            }
        }
    }
}

/// Handle RouteMessage
impl Handler<RouteMessage> for MessageRouterActor {
    type Result = A2AResult<RouteResult>;
    
    fn handle(&mut self, msg: RouteMessage, _ctx: &mut Self::Context) -> Self::Result {
        let start_time = std::time::Instant::now();
        
        debug!("Routing message {}", msg.message.message_id);
        
        if msg.target_endpoints.is_empty() {
            self.stats.failed_routes += 1;
            return Err(A2AError::agent_not_found("unknown"));
        }
        
        // Cache checking simplified for now
        self.stats.cache_misses += 1;
        
        // Select endpoint
        let selected_endpoint = self.select_endpoint(&msg.target_endpoints, &msg.message);
        
        if selected_endpoint.is_empty() {
            self.stats.failed_routes += 1;
            return Err(A2AError::agent_not_found("unknown"));
        }
        
        // Update statistics
        self.stats.total_routes += 1;
        self.stats.successful_routes += 1;
        
        let route_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(RouteResult {
            selected_endpoint,
            route_time_ms,
            cache_hit: false,
        })
    }
}

/// Handle GetRouterStats
impl Handler<GetRouterStats> for MessageRouterActor {
    type Result = A2AResult<RouterStats>;

    fn handle(&mut self, _msg: GetRouterStats, _ctx: &mut Self::Context) -> Self::Result {
        Ok(self.stats.clone())
    }
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_size: 1000,
            enable_load_balancing: true,
        }
    }
}
