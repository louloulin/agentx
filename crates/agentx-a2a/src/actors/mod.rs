//! Actix Actor implementations for A2A protocol
//! 
//! This module contains all the Actor implementations that leverage
//! Actix's actor model for concurrent, fault-tolerant A2A protocol processing.

pub mod protocol_actor;
pub mod registry_actor;
pub mod router_actor;
pub mod supervisor_actor;
pub mod security_actor;
pub mod metrics_actor;

// Protocol Actor exports
pub use protocol_actor::{
    A2AProtocolActor, HandleMessage, RegisterHandler, UpdateProtocolConfig,
    GetProtocolStats, ProtocolStats
};

// Registry Actor exports
pub use registry_actor::{
    AgentRegistryActor, RegisterAgent, UnregisterAgent, GetAgent,
    ListAgents, GetRegistryStats, RegistryStats, UpdateAgentStatus,
    DiscoverAgents, AgentFilter,
    PeriodicHealthCheck as RegistryPeriodicHealthCheck
};

// Router Actor exports
pub use router_actor::{
    MessageRouterActor, RouteMessage, GetRouterStats, RouterStats, RouteResult
};

// Supervisor Actor exports
pub use supervisor_actor::{
    PluginSupervisorActor, StartPlugin, StopPlugin, RestartPlugin,
    GetPluginStatus, ListPlugins, GetSupervisorStats, SupervisorStats,
    PluginProcess, PluginStatus, SupervisorConfig,
    PeriodicHealthCheck as SupervisorPeriodicHealthCheck
};

// Security Actor exports
pub use security_actor::{
    SecurityManagerActor, Authenticate, Authorize, InvalidateSession,
    GetAuditEvents, SecurityPolicy, AuthenticationResult, AuthorizationResult,
    Credentials, AuditEvent, AuditFilter
};

// Metrics Actor exports
pub use metrics_actor::{
    MetricsCollectorActor, RecordMetric, GetAllMetrics, GetSystemMetrics,
    GetPerformanceMetrics, GetCustomMetrics, ResetMetrics, CollectMetrics,
    SystemMetrics, PerformanceMetrics as ActorPerformanceMetrics,
    MetricValue, MetricType as ActorMetricType, AllMetrics, MetricsConfig
};

use actix::prelude::*;
use crate::A2AError;
use serde::{Deserialize, Serialize};

/// Common actor messages for the A2A system
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "Result<(), A2AError>")]
pub struct SystemShutdown;

#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "Result<SystemStatus, A2AError>")]
pub struct GetSystemStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_actors: usize,
    pub processed_messages: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
}

// Actor supervision and mailbox configuration would be implemented here
// when needed for specific use cases
