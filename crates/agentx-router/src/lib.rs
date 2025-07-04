//! AgentX Message Router
//! 
//! This crate implements intelligent message routing for the AgentX platform,
//! providing load balancing, failover, and performance optimization for
//! agent-to-agent communication.

pub mod router;
pub mod strategy;
pub mod metrics;
pub mod cache;
pub mod error;

pub use router::*;
pub use strategy::*;
pub use metrics::*;
pub use cache::*;
pub use error::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
