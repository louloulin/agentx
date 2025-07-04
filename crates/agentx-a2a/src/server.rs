//! A2A Protocol Server
//! 
//! This module provides a server implementation for handling A2A protocol
//! requests and exposing agent capabilities.

use crate::{
    A2AMessage, AgentCard, CapabilityQuery, A2AProtocolEngine, 
    A2AError, A2AResult, MessageHandler, MessageContext
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// A2A Protocol Server
pub struct A2AServer {
    /// Protocol engine
    engine: Arc<A2AProtocolEngine>,
    
    /// Server configuration
    config: ServerConfig,
    
    /// Local agent card
    agent_card: Arc<RwLock<Option<AgentCard>>>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// Maximum request size
    pub max_request_size: usize,
    
    /// Request timeout
    pub request_timeout: std::time::Duration,
    
    /// Enable request logging
    pub enable_logging: bool,
    
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    
    /// Authentication parameters
    pub parameters: std::collections::HashMap<String, String>,
}

/// Authentication types
#[derive(Debug, Clone)]
pub enum AuthType {
    /// No authentication
    None,
    
    /// Bearer token authentication
    Bearer,
    
    /// API key authentication
    ApiKey,
    
    /// OAuth2 authentication
    OAuth2,
}

/// Request/Response types for A2A API
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub message: A2AMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub response: Option<A2AMessage>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityQueryRequest {
    pub query: CapabilityQuery,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityQueryResponse {
    pub matches: Vec<crate::CapabilityMatch>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

/// Default message handler that echoes messages
pub struct EchoMessageHandler;

#[async_trait]
impl MessageHandler for EchoMessageHandler {
    async fn handle(&self, message: &A2AMessage, _context: &MessageContext) -> A2AResult<Option<A2AMessage>> {
        debug!("Echo handler processing message: {}", message.id);
        
        // Create echo response
        let response = A2AMessage::response(
            message,
            message.payload.clone(),
        );
        
        Ok(Some(response))
    }
    
    fn supported_types(&self) -> Vec<crate::MessageType> {
        vec![
            crate::MessageType::Request,
            crate::MessageType::Notification,
        ]
    }
}

impl A2AServer {
    /// Create a new A2A server
    pub fn new(engine: Arc<A2AProtocolEngine>, config: ServerConfig) -> Self {
        Self {
            engine,
            config,
            agent_card: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Set the local agent card
    pub async fn set_agent_card(&self, agent_card: AgentCard) -> A2AResult<()> {
        info!("Setting agent card for server: {}", agent_card.id);
        
        // Register with the protocol engine
        self.engine.register_agent(agent_card.clone()).await?;
        
        // Store locally
        *self.agent_card.write().await = Some(agent_card);
        
        Ok(())
    }
    
    /// Get the local agent card
    pub async fn get_agent_card(&self) -> Option<AgentCard> {
        self.agent_card.read().await.clone()
    }
    
    /// Handle incoming message
    pub async fn handle_message(&self, request: SendMessageRequest) -> SendMessageResponse {
        debug!("Handling incoming message: {}", request.message.id);
        
        match self.engine.process_message(request.message).await {
            Ok(response) => SendMessageResponse {
                response,
                success: true,
                error: None,
            },
            Err(e) => {
                error!("Failed to process message: {}", e);
                SendMessageResponse {
                    response: None,
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    /// Handle capability query
    pub async fn handle_capability_query(&self, request: CapabilityQueryRequest) -> CapabilityQueryResponse {
        debug!("Handling capability query");
        
        match self.engine.discover_agents(&request.query).await {
            Ok(matches) => CapabilityQueryResponse {
                matches,
                success: true,
                error: None,
            },
            Err(e) => {
                error!("Failed to query capabilities: {}", e);
                CapabilityQueryResponse {
                    matches: Vec::new(),
                    success: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }
    
    /// Handle health check
    pub async fn handle_health_check(&self) -> HealthResponse {
        HealthResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            version: crate::A2A_VERSION.to_string(),
        }
    }
    
    /// Validate authentication
    pub fn validate_auth(&self, auth_header: Option<&str>) -> A2AResult<()> {
        if let Some(auth_config) = &self.config.auth {
            match auth_config.auth_type {
                AuthType::None => Ok(()),
                AuthType::Bearer => {
                    if let Some(header) = auth_header {
                        if header.starts_with("Bearer ") {
                            let token = &header[7..];
                            if let Some(expected_token) = auth_config.parameters.get("token") {
                                if token == expected_token {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    Err(A2AError::authentication("Invalid bearer token"))
                }
                AuthType::ApiKey => {
                    if let Some(header) = auth_header {
                        if let Some(expected_key) = auth_config.parameters.get("api_key") {
                            if header == expected_key {
                                return Ok(());
                            }
                        }
                    }
                    Err(A2AError::authentication("Invalid API key"))
                }
                AuthType::OAuth2 => {
                    // OAuth2 validation would be more complex
                    // For now, just check for presence of token
                    if auth_header.is_some() {
                        Ok(())
                    } else {
                        Err(A2AError::authentication("OAuth2 token required"))
                    }
                }
            }
        } else {
            Ok(())
        }
    }
    
    /// Start the server (this would be implemented with a web framework like Axum)
    pub async fn start(&self) -> A2AResult<()> {
        info!("Starting A2A server on {}:{}", self.config.bind_address, self.config.port);
        
        // This is a placeholder - in a real implementation, you would:
        // 1. Set up HTTP server with routes
        // 2. Configure middleware (CORS, logging, auth)
        // 3. Handle graceful shutdown
        // 4. Set up health checks
        
        // Example routes:
        // POST /a2a/messages - handle_message
        // POST /a2a/capabilities/query - handle_capability_query
        // GET /a2a/agent-card - get_agent_card
        // GET /a2a/health - handle_health_check
        
        info!("A2A server started successfully");
        
        // Keep server running
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    
    /// Stop the server
    pub async fn stop(&self) -> A2AResult<()> {
        info!("Stopping A2A server");
        
        // Unregister agent if we have one
        if let Some(agent_card) = self.get_agent_card().await {
            if let Err(e) = self.engine.unregister_agent(&agent_card.id).await {
                warn!("Failed to unregister agent during shutdown: {}", e);
            }
        }
        
        info!("A2A server stopped");
        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            enable_cors: true,
            max_request_size: 1024 * 1024, // 1MB
            request_timeout: std::time::Duration::from_secs(30),
            enable_logging: true,
            auth: None,
        }
    }
}

impl AuthConfig {
    /// Create bearer token authentication
    pub fn bearer_token(token: String) -> Self {
        let mut parameters = std::collections::HashMap::new();
        parameters.insert("token".to_string(), token);
        
        Self {
            auth_type: AuthType::Bearer,
            parameters,
        }
    }
    
    /// Create API key authentication
    pub fn api_key(key: String) -> Self {
        let mut parameters = std::collections::HashMap::new();
        parameters.insert("api_key".to_string(), key);
        
        Self {
            auth_type: AuthType::ApiKey,
            parameters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProtocolConfig, MessageType, MessagePayload, TextPayload};

    #[tokio::test]
    async fn test_server_creation() {
        let engine = Arc::new(A2AProtocolEngine::new(ProtocolConfig::default()));
        let server = A2AServer::new(engine, ServerConfig::default());
        
        assert!(server.get_agent_card().await.is_none());
    }
    
    #[tokio::test]
    async fn test_echo_handler() {
        let handler = EchoMessageHandler;
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
        
        let context = MessageContext::new();
        let response = handler.handle(&message, &context).await.unwrap();
        
        assert!(response.is_some());
        let response = response.unwrap();
        assert_eq!(response.message_type, MessageType::Response);
    }
    
    #[test]
    fn test_auth_config() {
        let auth = AuthConfig::bearer_token("test-token".to_string());
        assert!(matches!(auth.auth_type, AuthType::Bearer));
        assert_eq!(auth.parameters.get("token"), Some(&"test-token".to_string()));
    }
}
