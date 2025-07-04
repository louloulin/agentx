//! A2A Protocol Client
//! 
//! This module provides a client for communicating with other agents
//! using the A2A protocol.

use crate::{A2AMessage, AgentCard, CapabilityQuery, A2AError, A2AResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// A2A Protocol Client
pub struct A2AClient {
    /// HTTP client
    http_client: Client,
    
    /// Client configuration
    config: ClientConfig,
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout
    pub timeout: Duration,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Retry delay
    pub retry_delay: Duration,
    
    /// User agent string
    pub user_agent: String,
    
    /// Default headers
    pub default_headers: std::collections::HashMap<String, String>,
}

/// Agent endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEndpoint {
    /// Agent ID
    pub agent_id: String,
    
    /// Base URL for the agent
    pub base_url: String,
    
    /// Supported protocols
    pub protocols: Vec<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
}

/// A2A Client trait for different transport protocols
#[async_trait]
pub trait A2ATransport: Send + Sync {
    /// Send a message to an agent
    async fn send_message(&self, endpoint: &AgentEndpoint, message: &A2AMessage) -> A2AResult<Option<A2AMessage>>;
    
    /// Query agent capabilities
    async fn query_capabilities(&self, endpoint: &AgentEndpoint, query: &CapabilityQuery) -> A2AResult<Vec<crate::CapabilityMatch>>;
    
    /// Get agent card
    async fn get_agent_card(&self, endpoint: &AgentEndpoint) -> A2AResult<AgentCard>;
    
    /// Health check
    async fn health_check(&self, endpoint: &AgentEndpoint) -> A2AResult<bool>;
}

impl A2AClient {
    /// Create a new A2A client
    pub fn new(config: ClientConfig) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // Add default headers
        for (key, value) in &config.default_headers {
            if let (Ok(name), Ok(value)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(value)
            ) {
                headers.insert(name, value);
            }
        }
        
        let http_client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            http_client,
            config,
        }
    }
    
    /// Send a message to an agent
    pub async fn send_message(&self, endpoint: &AgentEndpoint, message: &A2AMessage) -> A2AResult<Option<A2AMessage>> {
        debug!("Sending message {} to agent {}", message.id, endpoint.agent_id);
        
        let url = format!("{}/a2a/messages", endpoint.base_url);
        let mut retries = 0;
        
        loop {
            let mut request = self.http_client.post(&url).json(message);
            
            // Add authentication if available
            if let Some(token) = &endpoint.auth_token {
                request = request.bearer_auth(token);
            }
            
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if response.status() == reqwest::StatusCode::NO_CONTENT {
                            return Ok(None);
                        }
                        
                        let response_message: A2AMessage = response.json().await?;
                        info!("Received response for message {}", message.id);
                        return Ok(Some(response_message));
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        let error = match status {
                            reqwest::StatusCode::NOT_FOUND => A2AError::agent_not_found(&endpoint.agent_id),
                            reqwest::StatusCode::UNAUTHORIZED => A2AError::authentication("Invalid credentials"),
                            reqwest::StatusCode::FORBIDDEN => A2AError::authorization("Access denied"),
                            reqwest::StatusCode::TOO_MANY_REQUESTS => A2AError::RateLimitExceeded,
                            reqwest::StatusCode::SERVICE_UNAVAILABLE => A2AError::service_unavailable("Agent unavailable"),
                            _ => A2AError::internal(format!("HTTP {}: {}", status, error_text)),
                        };
                        
                        if !error.is_retryable() || retries >= self.config.max_retries {
                            return Err(error);
                        }
                        
                        error!("Request failed (attempt {}): {}", retries + 1, error);
                    }
                }
                Err(e) => {
                    let error = A2AError::Network(e);
                    
                    if !error.is_retryable() || retries >= self.config.max_retries {
                        return Err(error);
                    }
                    
                    error!("Network error (attempt {}): {}", retries + 1, error);
                }
            }
            
            retries += 1;
            tokio::time::sleep(self.config.retry_delay).await;
        }
    }
    
    /// Query agent capabilities
    pub async fn query_capabilities(&self, endpoint: &AgentEndpoint, query: &CapabilityQuery) -> A2AResult<Vec<crate::CapabilityMatch>> {
        debug!("Querying capabilities for agent {}", endpoint.agent_id);
        
        let url = format!("{}/a2a/capabilities/query", endpoint.base_url);
        let mut request = self.http_client.post(&url).json(query);
        
        if let Some(token) = &endpoint.auth_token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            let matches: Vec<crate::CapabilityMatch> = response.json().await?;
            info!("Found {} capability matches for agent {}", matches.len(), endpoint.agent_id);
            Ok(matches)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(A2AError::internal(format!("HTTP {}: {}", status, error_text)))
        }
    }
    
    /// Get agent card
    pub async fn get_agent_card(&self, endpoint: &AgentEndpoint) -> A2AResult<AgentCard> {
        debug!("Getting agent card for {}", endpoint.agent_id);
        
        let url = format!("{}/a2a/agent-card", endpoint.base_url);
        let mut request = self.http_client.get(&url);
        
        if let Some(token) = &endpoint.auth_token {
            request = request.bearer_auth(token);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            let agent_card: AgentCard = response.json().await?;
            info!("Retrieved agent card for {}", endpoint.agent_id);
            Ok(agent_card)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(A2AError::internal(format!("HTTP {}: {}", status, error_text)))
        }
    }
    
    /// Health check
    pub async fn health_check(&self, endpoint: &AgentEndpoint) -> A2AResult<bool> {
        debug!("Health check for agent {}", endpoint.agent_id);
        
        let url = format!("{}/a2a/health", endpoint.base_url);
        let mut request = self.http_client.get(&url);
        
        if let Some(token) = &endpoint.auth_token {
            request = request.bearer_auth(token);
        }
        
        match request.send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                debug!("Agent {} health: {}", endpoint.agent_id, is_healthy);
                Ok(is_healthy)
            }
            Err(_) => {
                debug!("Agent {} is not reachable", endpoint.agent_id);
                Ok(false)
            }
        }
    }
    
    /// Discover agents in a registry
    pub async fn discover_agents(&self, registry_url: &str, query: &CapabilityQuery) -> A2AResult<Vec<crate::CapabilityMatch>> {
        debug!("Discovering agents in registry: {}", registry_url);
        
        let url = format!("{}/a2a/discover", registry_url);
        let response = self.http_client.post(&url).json(query).send().await?;
        
        if response.status().is_success() {
            let matches: Vec<crate::CapabilityMatch> = response.json().await?;
            info!("Discovered {} agents in registry", matches.len());
            Ok(matches)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(A2AError::internal(format!("HTTP {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
impl A2ATransport for A2AClient {
    async fn send_message(&self, endpoint: &AgentEndpoint, message: &A2AMessage) -> A2AResult<Option<A2AMessage>> {
        self.send_message(endpoint, message).await
    }
    
    async fn query_capabilities(&self, endpoint: &AgentEndpoint, query: &CapabilityQuery) -> A2AResult<Vec<crate::CapabilityMatch>> {
        self.query_capabilities(endpoint, query).await
    }
    
    async fn get_agent_card(&self, endpoint: &AgentEndpoint) -> A2AResult<AgentCard> {
        self.get_agent_card(endpoint).await
    }
    
    async fn health_check(&self, endpoint: &AgentEndpoint) -> A2AResult<bool> {
        self.health_check(endpoint).await
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            user_agent: format!("AgentX-A2A-Client/{}", env!("CARGO_PKG_VERSION")),
            default_headers: std::collections::HashMap::new(),
        }
    }
}

impl AgentEndpoint {
    /// Create a new agent endpoint
    pub fn new(agent_id: String, base_url: String) -> Self {
        Self {
            agent_id,
            base_url,
            protocols: vec!["http".to_string()],
            auth_token: None,
        }
    }
    
    /// Set authentication token
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
    
    /// Add supported protocol
    pub fn with_protocol(mut self, protocol: String) -> Self {
        if !self.protocols.contains(&protocol) {
            self.protocols.push(protocol);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.user_agent.contains("AgentX-A2A-Client"));
    }
    
    #[test]
    fn test_agent_endpoint() {
        let endpoint = AgentEndpoint::new(
            "agent1".to_string(),
            "http://localhost:8080".to_string(),
        ).with_auth_token("token123".to_string())
         .with_protocol("grpc".to_string());
        
        assert_eq!(endpoint.agent_id, "agent1");
        assert_eq!(endpoint.auth_token, Some("token123".to_string()));
        assert!(endpoint.protocols.contains(&"grpc".to_string()));
    }
}
