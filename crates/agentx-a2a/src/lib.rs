//! AgentX A2A Protocol Implementation
//!
//! This crate implements the Agent-to-Agent (A2A) protocol for AgentX platform.
//! It provides message formats, serialization, and communication primitives
//! for inter-agent communication following the A2A specification.

pub mod message;
pub mod agent_card;
pub mod capability;
// pub mod protocol;
pub mod protocol_engine;
pub mod error;
pub mod streaming;
pub mod security;
pub mod encryption;
pub mod monitoring;
pub mod monitoring_dashboard;
// pub mod client;
// pub mod server;
pub mod actors;

pub use message::*;
pub use agent_card::*;
pub use capability::*;
// pub use protocol::*;
pub use protocol_engine::*;
pub use error::*;
pub use streaming::*;
// 选择性导出以避免冲突
pub use security::{AuthType, SecurityManager, SecurityContext, SecurityConfig, AuthCredentials, SignatureAlgorithm};
pub use encryption::*;
// 选择性导出监控模块以避免冲突
pub use monitoring::{
    MetricType as MonitoringMetricType, MetricPoint, PerformanceStats,
    MessageStats, AgentStats, SystemStats, MonitoringConfig
};
pub use monitoring_dashboard::{
    MonitoringDashboard, DashboardConfig, AlertRule, AlertSeverity, Alert,
    PerformanceMetrics as DashboardPerformanceMetrics, Widget, WidgetType
};
pub use actors::*;

/// A2A Protocol version
pub const A2A_VERSION: &str = "1.0";

/// A2A Protocol namespace
pub const A2A_NAMESPACE: &str = "https://schemas.google.com/a2a/v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(A2A_VERSION, "1.0");
    }

    #[test]
    fn test_namespace() {
        assert_eq!(A2A_NAMESPACE, "https://schemas.google.com/a2a/v1");
    }
}
