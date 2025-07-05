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

pub use protocol_actor::*;
pub use registry_actor::*;
pub use router_actor::*;
pub use supervisor_actor::*;
pub use security_actor::*;
pub use metrics_actor::*;

use actix::prelude::*;
use crate::{A2AMessage, AgentCard, A2AError};
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
