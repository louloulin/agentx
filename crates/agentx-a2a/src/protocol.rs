//! A2A Protocol Engine
//! 
//! This module implements the core A2A protocol engine for message
//! processing, routing, and agent communication.

use crate::{
    A2AMessage, AgentCard, CapabilityDiscovery, CapabilityQuery, 
    A2AError, A2AResult, MessageType, MessagePayload, ErrorPayload
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// A2A Protocol Engine
pub struct A2AProtocolEngine {
    /// Agent registry for capability discovery
    discovery: Arc<RwLock<CapabilityDiscovery>>,
    
    /// Message handlers by message type
    handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
    
    /// Protocol configuration
    config: ProtocolConfig,
    
    /// Message interceptors
    interceptors: Vec<Box<dyn MessageInterceptor>>,
}

/// Protocol configuration
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Default message timeout in seconds
    pub default_timeout: u64,

    /// Maximum number of hops for message routing
    pub max_hops: u8,

    /// Enable message validation
    pub validate_messages: bool,

    /// Enable capability caching
    pub cache_capabilities: bool,

    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,

    /// Handler pool size for actor-based processing
    pub handler_pool_size: Option<usize>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub max_requests_per_second: u32,
    
    /// Burst capacity
    pub burst_capacity: u32,
}

/// Message handler trait
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a message
    async fn handle(&self, message: &A2AMessage, context: &MessageContext) -> A2AResult<Option<A2AMessage>>;
    
    /// Get supported message types
    fn supported_types(&self) -> Vec<MessageType>;
}

/// Message interceptor trait for middleware functionality
#[async_trait]
pub trait MessageInterceptor: Send + Sync {
    /// Intercept incoming message (before processing)
    async fn intercept_incoming(&self, message: &mut A2AMessage, context: &MessageContext) -> A2AResult<()>;
    
    /// Intercept outgoing message (after processing)
    async fn intercept_outgoing(&self, message: &mut A2AMessage, context: &MessageContext) -> A2AResult<()>;
}

/// Message processing context
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Source agent information
    pub source_agent: Option<AgentCard>,
    
    /// Target agent information
    pub target_agent: Option<AgentCard>,
    
    /// Processing metadata
    pub metadata: HashMap<String, String>,
    
    /// Hop count for routing
    pub hop_count: u8,
    
    /// Processing start time
    pub start_time: chrono::DateTime<Utc>,
}

impl A2AProtocolEngine {
    /// Create a new protocol engine
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            discovery: Arc::new(RwLock::new(CapabilityDiscovery::new())),
            handlers: HashMap::new(),
            config,
            interceptors: Vec::new(),
        }
    }
    
    /// Register a message handler
    pub fn register_handler(&mut self, handler: Box<dyn MessageHandler>) {
        for message_type in handler.supported_types() {
            self.handlers.insert(message_type, handler.clone());
        }
    }
    
    /// Add a message interceptor
    pub fn add_interceptor(&mut self, interceptor: Box<dyn MessageInterceptor>) {
        self.interceptors.push(interceptor);
    }
    
    /// Register an agent
    pub async fn register_agent(&self, agent_card: AgentCard) -> A2AResult<()> {
        info!("Registering agent: {}", agent_card.id);
        
        // Validate agent card
        self.validate_agent_card(&agent_card)?;
        
        // Register with discovery service
        let mut discovery = self.discovery.write().await;
        discovery.register_agent(agent_card);
        
        Ok(())
    }
    
    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> A2AResult<()> {
        info!("Unregistering agent: {}", agent_id);
        
        let mut discovery = self.discovery.write().await;
        discovery.unregister_agent(agent_id);
        
        Ok(())
    }
    
    /// Discover agents by capability
    pub async fn discover_agents(&self, query: &CapabilityQuery) -> A2AResult<Vec<crate::CapabilityMatch>> {
        debug!("Discovering agents with query: {:?}", query);
        
        let discovery = self.discovery.read().await;
        let matches = discovery.discover(query);
        
        info!("Found {} matching agents", matches.len());
        Ok(matches)
    }
    
    /// Process an incoming message
    pub async fn process_message(&self, mut message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let start_time = Utc::now();
        
        debug!("Processing message: {} from {} to {}", 
               message.id, message.from, message.to);
        
        // Validate message
        if self.config.validate_messages {
            self.validate_message(&message)?;
        }
        
        // Check if message has expired
        if message.is_expired() {
            warn!("Message {} has expired", message.id);
            return Err(A2AError::MessageExpired);
        }
        
        // Create processing context
        let mut context = MessageContext {
            source_agent: None,
            target_agent: None,
            metadata: HashMap::new(),
            hop_count: 0,
            start_time,
        };
        
        // Load agent information
        let discovery = self.discovery.read().await;
        context.source_agent = discovery.get_agent(&message.from).cloned();
        context.target_agent = discovery.get_agent(&message.to).cloned();
        drop(discovery);
        
        // Run incoming interceptors
        for interceptor in &self.interceptors {
            interceptor.intercept_incoming(&mut message, &context).await?;
        }
        
        // Find and execute handler
        let response = if let Some(handler) = self.handlers.get(&message.message_type) {
            handler.handle(&message, &context).await?
        } else {
            warn!("No handler found for message type: {:?}", message.message_type);
            Some(A2AMessage::error_response(
                &message,
                "NO_HANDLER".to_string(),
                format!("No handler for message type: {:?}", message.message_type),
            ))
        };
        
        // Run outgoing interceptors
        if let Some(mut response) = response {
            for interceptor in &self.interceptors {
                interceptor.intercept_outgoing(&mut response, &context).await?;
            }
            
            debug!("Message processing completed: {}", message.id);
            Ok(Some(response))
        } else {
            debug!("Message processing completed with no response: {}", message.id);
            Ok(None)
        }
    }
    
    /// Route a message to its destination
    pub async fn route_message(&self, message: A2AMessage) -> A2AResult<()> {
        debug!("Routing message: {} to {}", message.id, message.to);
        
        // Find target agent
        let discovery = self.discovery.read().await;
        let target_agent = discovery.get_agent(&message.to)
            .ok_or_else(|| A2AError::agent_not_found(&message.to))?;
        
        // Check if agent is available
        if target_agent.status != crate::AgentStatus::Online {
            return Err(A2AError::service_unavailable(
                format!("Agent {} is not online", message.to)
            ));
        }
        
        // Route to agent endpoint
        // This would typically involve sending the message via HTTP, gRPC, etc.
        // For now, we'll just log the routing
        info!("Message {} routed to agent {}", message.id, message.to);
        
        Ok(())
    }
    
    /// Validate a message
    fn validate_message(&self, message: &A2AMessage) -> A2AResult<()> {
        // Check message size
        let message_size = serde_json::to_string(message)?.len();
        if message_size > self.config.max_message_size {
            return Err(A2AError::validation(
                format!("Message size {} exceeds maximum {}", 
                       message_size, self.config.max_message_size)
            ));
        }
        
        // Check required fields
        if message.id.is_empty() {
            return Err(A2AError::validation("Message ID is required"));
        }
        
        if message.from.is_empty() {
            return Err(A2AError::validation("Source agent ID is required"));
        }
        
        if message.to.is_empty() {
            return Err(A2AError::validation("Target agent ID is required"));
        }
        
        // Check version compatibility
        if message.version != crate::A2A_VERSION {
            return Err(A2AError::version_mismatch(crate::A2A_VERSION, &message.version));
        }
        
        Ok(())
    }
    
    /// Validate an agent card
    fn validate_agent_card(&self, agent_card: &AgentCard) -> A2AResult<()> {
        if agent_card.id.is_empty() {
            return Err(A2AError::validation("Agent ID is required"));
        }
        
        if agent_card.name.is_empty() {
            return Err(A2AError::validation("Agent name is required"));
        }
        
        if agent_card.endpoints.is_empty() {
            return Err(A2AError::validation("At least one endpoint is required"));
        }
        
        // Validate endpoints
        for endpoint in &agent_card.endpoints {
            if endpoint.url.is_empty() {
                return Err(A2AError::validation("Endpoint URL is required"));
            }
            
            // Basic URL validation
            if !endpoint.url.starts_with("http://") && !endpoint.url.starts_with("https://") {
                return Err(A2AError::validation(
                    format!("Invalid endpoint URL: {}", endpoint.url)
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB
            default_timeout: 30,           // 30 seconds
            max_hops: 10,
            validate_messages: true,
            cache_capabilities: true,
            rate_limit: None,
            handler_pool_size: Some(10),   // Default pool size
        }
    }
}

impl MessageContext {
    /// Create a new message context
    pub fn new() -> Self {
        Self {
            source_agent: None,
            target_agent: None,
            metadata: HashMap::new(),
            hop_count: 0,
            start_time: Utc::now(),
        }
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Increment hop count
    pub fn increment_hops(&mut self) {
        self.hop_count += 1;
    }
}

impl Default for MessageContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentCard, Endpoint, MessagePayload, TextPayload};

    #[tokio::test]
    async fn test_register_agent() {
        let engine = A2AProtocolEngine::new(ProtocolConfig::default());
        
        let agent_card = AgentCard::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            "1.0.0".to_string(),
        ).add_endpoint(
            Endpoint::new("http".to_string(), "http://localhost:8080".to_string())
        );
        
        let result = engine.register_agent(agent_card).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_message_validation() {
        let engine = A2AProtocolEngine::new(ProtocolConfig::default());
        
        let message = A2AMessage::new(
            "agent1".to_string(),
            "agent2".to_string(),
            MessageType::Request,
            MessagePayload::Text(TextPayload {
                content: "Hello".to_string(),
                format: "plain".to_string(),
                language: None,
            }),
        );
        
        let result = engine.validate_message(&message);
        assert!(result.is_ok());
    }
}
