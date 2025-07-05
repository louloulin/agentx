//! Agent Card Implementation for A2A Protocol v0.2.5
//!
//! This module implements the Agent Card format for capability discovery
//! in the A2A protocol. Agent Cards allow agents to advertise their
//! capabilities and services to other agents.
//!
//! Based on Google's A2A specification: https://a2aproject.github.io/A2A/
//!
//! Agent Cards are the foundation of A2A's discovery mechanism, enabling
//! agents to find and interact with each other in a standardized way.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent Card - Describes an agent's capabilities and metadata
///
/// Compliant with A2A Protocol v0.2.5 specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct AgentCard {
    /// Agent unique identifier
    pub id: String,
    
    /// Agent display name
    pub name: String,
    
    /// Agent description
    pub description: String,
    
    /// Agent version
    pub version: String,
    
    /// Agent capabilities
    pub capabilities: Vec<Capability>,
    
    /// Agent endpoints for communication
    pub endpoints: Vec<Endpoint>,
    
    /// Agent metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Card creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Card last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Card expiration time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Agent status
    #[serde(default)]
    pub status: AgentStatus,
    
    /// Supported A2A protocol versions
    #[serde(default)]
    pub supported_versions: Vec<String>,
    
    /// Agent tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Supported interaction modalities (A2A specific)
    #[serde(default)]
    pub interaction_modalities: Vec<InteractionModality>,

    /// UX negotiation capabilities (A2A specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ux_capabilities: Option<UxCapabilities>,

    /// Agent trust level for enterprise environments
    #[serde(default)]
    pub trust_level: TrustLevel,

    /// Supported task types
    #[serde(default)]
    pub supported_task_types: Vec<String>,
}

/// Agent capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    /// Capability name/identifier
    pub name: String,
    
    /// Capability description
    pub description: String,
    
    /// Capability type
    #[serde(rename = "type")]
    pub capability_type: CapabilityType,
    
    /// Input schema (JSON Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    
    /// Output schema (JSON Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    
    /// Capability metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Whether this capability is available
    #[serde(default = "default_true")]
    pub available: bool,
    
    /// Cost information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<CostInfo>,
}

/// Types of capabilities an agent can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityType {
    /// Text processing and generation
    TextGeneration,
    
    /// Image processing and generation
    ImageProcessing,
    
    /// Audio processing
    AudioProcessing,
    
    /// Video processing
    VideoProcessing,
    
    /// Data analysis
    DataAnalysis,
    
    /// Tool/function execution
    ToolExecution,
    
    /// Workflow orchestration
    WorkflowOrchestration,
    
    /// Knowledge retrieval
    KnowledgeRetrieval,
    
    /// Code generation and execution
    CodeExecution,
    
    /// Custom capability
    Custom(String),
}

/// Agent communication endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Endpoint {
    /// Endpoint type (http, grpc, websocket, etc.)
    #[serde(rename = "type")]
    pub endpoint_type: String,
    
    /// Endpoint URL
    pub url: String,
    
    /// Supported protocols
    #[serde(default)]
    pub protocols: Vec<String>,
    
    /// Authentication requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthInfo>,
    
    /// Endpoint metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Authentication information for endpoints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthInfo {
    /// Authentication type (bearer, api_key, oauth2, etc.)
    #[serde(rename = "type")]
    pub auth_type: String,
    
    /// Authentication parameters
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Cost information for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostInfo {
    /// Cost model (per_request, per_token, per_minute, etc.)
    pub model: String,
    
    /// Cost amount
    pub amount: f64,
    
    /// Currency code
    pub currency: String,
    
    /// Cost description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Interaction modalities supported by the agent (A2A specific)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum InteractionModality {
    /// Text-based interaction
    Text,
    /// Form-based interaction with structured inputs
    Forms,
    /// Media-based interaction (images, audio, video)
    Media,
    /// File-based interaction
    Files,
    /// Real-time streaming interaction
    Streaming,
    /// Custom modality
    Custom(String),
}

/// UX negotiation capabilities (A2A specific)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UxCapabilities {
    /// Supported UI components
    pub supported_components: Vec<String>,
    /// Dynamic UX adaptation support
    pub dynamic_adaptation: bool,
    /// Multi-modal UX support
    pub multimodal_support: bool,
    /// Custom UX negotiation protocols
    #[serde(default)]
    pub custom_protocols: Vec<String>,
}

/// Trust level for enterprise environments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum TrustLevel {
    /// Public agent - no special trust
    #[default]
    Public,
    /// Verified agent - identity verified
    Verified,
    /// Trusted agent - within organization
    Trusted,
    /// Internal agent - fully trusted internal system
    Internal,
}

impl TrustLevel {
    /// Get numeric trust score
    pub fn trust_score(&self) -> u8 {
        match self {
            TrustLevel::Public => 1,
            TrustLevel::Verified => 3,
            TrustLevel::Trusted => 7,
            TrustLevel::Internal => 10,
        }
    }
}

/// Agent status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum AgentStatus {
    /// Agent is online and available
    #[default]
    Online,
    
    /// Agent is offline
    Offline,
    
    /// Agent is busy
    Busy,
    
    /// Agent is in maintenance mode
    Maintenance,
    
    /// Agent status is unknown
    Unknown,
}

impl AgentCard {
    /// Create a new agent card
    pub fn new(id: String, name: String, description: String, version: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            version,
            capabilities: Vec::new(),
            endpoints: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            status: AgentStatus::Online,
            supported_versions: vec![crate::A2A_VERSION.to_string()],
            tags: Vec::new(),
            interaction_modalities: vec![InteractionModality::Text], // Default to text
            ux_capabilities: None,
            trust_level: TrustLevel::Public,
            supported_task_types: Vec::new(),
        }
    }
    
    /// Add a capability to the agent card
    pub fn add_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self.updated_at = Utc::now();
        self
    }
    
    /// Add an endpoint to the agent card
    pub fn add_endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoints.push(endpoint);
        self.updated_at = Utc::now();
        self
    }
    
    /// Set agent status
    pub fn with_status(mut self, status: AgentStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
        self
    }
    
    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
        self
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self.updated_at = Utc::now();
        self
    }
    
    /// Check if the agent card has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Find capability by name
    pub fn find_capability(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }
    
    /// Check if agent has a specific capability
    pub fn has_capability(&self, name: &str) -> bool {
        self.find_capability(name).is_some()
    }
    
    /// Get available capabilities
    pub fn available_capabilities(&self) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.available).collect()
    }

    /// Add interaction modality
    pub fn with_interaction_modality(mut self, modality: InteractionModality) -> Self {
        if !self.interaction_modalities.contains(&modality) {
            self.interaction_modalities.push(modality);
            self.updated_at = Utc::now();
        }
        self
    }

    /// Set UX capabilities
    pub fn with_ux_capabilities(mut self, ux_capabilities: UxCapabilities) -> Self {
        self.ux_capabilities = Some(ux_capabilities);
        self.updated_at = Utc::now();
        self
    }

    /// Set trust level
    pub fn with_trust_level(mut self, trust_level: TrustLevel) -> Self {
        self.trust_level = trust_level;
        self.updated_at = Utc::now();
        self
    }

    /// Add supported task type
    pub fn with_task_type(mut self, task_type: String) -> Self {
        if !self.supported_task_types.contains(&task_type) {
            self.supported_task_types.push(task_type);
            self.updated_at = Utc::now();
        }
        self
    }

    /// Check if agent supports a specific interaction modality
    pub fn supports_modality(&self, modality: &InteractionModality) -> bool {
        self.interaction_modalities.contains(modality)
    }

    /// Check if agent supports a specific task type
    pub fn supports_task_type(&self, task_type: &str) -> bool {
        self.supported_task_types.contains(&task_type.to_string())
    }

    /// Get agent's trust score (for enterprise environments)
    pub fn trust_score(&self) -> u8 {
        match self.trust_level {
            TrustLevel::Public => 1,
            TrustLevel::Verified => 3,
            TrustLevel::Trusted => 7,
            TrustLevel::Internal => 10,
        }
    }

    /// Check if agent can handle multimodal interactions
    pub fn is_multimodal(&self) -> bool {
        self.interaction_modalities.len() > 1 ||
        self.interaction_modalities.contains(&InteractionModality::Media)
    }
}

impl Capability {
    /// Create a new capability
    pub fn new(
        name: String,
        description: String,
        capability_type: CapabilityType,
    ) -> Self {
        Self {
            name,
            description,
            capability_type,
            input_schema: None,
            output_schema: None,
            metadata: HashMap::new(),
            available: true,
            cost: None,
        }
    }
    
    /// Set input schema
    pub fn with_input_schema(mut self, schema: serde_json::Value) -> Self {
        self.input_schema = Some(schema);
        self
    }
    
    /// Set output schema
    pub fn with_output_schema(mut self, schema: serde_json::Value) -> Self {
        self.output_schema = Some(schema);
        self
    }
    
    /// Set availability
    pub fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }
    
    /// Set cost information
    pub fn with_cost(mut self, cost: CostInfo) -> Self {
        self.cost = Some(cost);
        self
    }
}

impl UxCapabilities {
    /// Create new UX capabilities
    pub fn new() -> Self {
        Self {
            supported_components: Vec::new(),
            dynamic_adaptation: false,
            multimodal_support: false,
            custom_protocols: Vec::new(),
        }
    }

    /// Add supported UI component
    pub fn with_component(mut self, component: String) -> Self {
        if !self.supported_components.contains(&component) {
            self.supported_components.push(component);
        }
        self
    }

    /// Enable dynamic adaptation
    pub fn with_dynamic_adaptation(mut self) -> Self {
        self.dynamic_adaptation = true;
        self
    }

    /// Enable multimodal support
    pub fn with_multimodal_support(mut self) -> Self {
        self.multimodal_support = true;
        self
    }

    /// Add custom protocol
    pub fn with_custom_protocol(mut self, protocol: String) -> Self {
        if !self.custom_protocols.contains(&protocol) {
            self.custom_protocols.push(protocol);
        }
        self
    }
}

impl Endpoint {
    /// Create a new endpoint
    pub fn new(endpoint_type: String, url: String) -> Self {
        Self {
            endpoint_type,
            url,
            protocols: Vec::new(),
            auth: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add supported protocol
    pub fn with_protocol(mut self, protocol: String) -> Self {
        if !self.protocols.contains(&protocol) {
            self.protocols.push(protocol);
        }
        self
    }
    
    /// Set authentication info
    pub fn with_auth(mut self, auth: AuthInfo) -> Self {
        self.auth = Some(auth);
        self
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent_card() {
        let card = AgentCard::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            "1.0.0".to_string(),
        );
        
        assert_eq!(card.id, "agent1");
        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.status, AgentStatus::Online);
        assert!(!card.is_expired());
    }
    
    #[test]
    fn test_add_capability() {
        let capability = Capability::new(
            "text_generation".to_string(),
            "Generate text content".to_string(),
            CapabilityType::TextGeneration,
        );
        
        let card = AgentCard::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            "1.0.0".to_string(),
        ).add_capability(capability);
        
        assert_eq!(card.capabilities.len(), 1);
        assert!(card.has_capability("text_generation"));
        assert!(!card.has_capability("image_processing"));
    }
    
    #[test]
    fn test_agent_card_serialization() {
        let card = AgentCard::new(
            "agent1".to_string(),
            "Test Agent".to_string(),
            "A test agent".to_string(),
            "1.0.0".to_string(),
        );

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: AgentCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card, deserialized);
    }

    #[test]
    fn test_a2a_specific_features() {
        let ux_capabilities = UxCapabilities::new()
            .with_component("button".to_string())
            .with_component("form".to_string())
            .with_dynamic_adaptation()
            .with_multimodal_support();

        let card = AgentCard::new(
            "a2a_agent".to_string(),
            "A2A Test Agent".to_string(),
            "An agent with A2A-specific features".to_string(),
            "2.0.0".to_string(),
        )
        .with_interaction_modality(InteractionModality::Text)
        .with_interaction_modality(InteractionModality::Forms)
        .with_interaction_modality(InteractionModality::Media)
        .with_ux_capabilities(ux_capabilities)
        .with_trust_level(TrustLevel::Trusted)
        .with_task_type("text_generation".to_string())
        .with_task_type("image_analysis".to_string());

        // Test interaction modalities
        assert!(card.supports_modality(&InteractionModality::Text));
        assert!(card.supports_modality(&InteractionModality::Forms));
        assert!(card.supports_modality(&InteractionModality::Media));
        assert!(!card.supports_modality(&InteractionModality::Streaming));

        // Test multimodal support
        assert!(card.is_multimodal());

        // Test task types
        assert!(card.supports_task_type("text_generation"));
        assert!(card.supports_task_type("image_analysis"));
        assert!(!card.supports_task_type("video_processing"));

        // Test trust level
        assert_eq!(card.trust_level, TrustLevel::Trusted);
        assert_eq!(card.trust_score(), 7);

        // Test UX capabilities
        let ux = card.ux_capabilities.as_ref().unwrap();
        assert!(ux.dynamic_adaptation);
        assert!(ux.multimodal_support);
        assert_eq!(ux.supported_components.len(), 2);
        assert!(ux.supported_components.contains(&"button".to_string()));
        assert!(ux.supported_components.contains(&"form".to_string()));
    }

    #[test]
    fn test_interaction_modality_serialization() {
        let modalities = vec![
            InteractionModality::Text,
            InteractionModality::Forms,
            InteractionModality::Media,
            InteractionModality::Files,
            InteractionModality::Streaming,
            InteractionModality::Custom("voice".to_string()),
        ];

        for modality in modalities {
            let json = serde_json::to_string(&modality).unwrap();
            let deserialized: InteractionModality = serde_json::from_str(&json).unwrap();
            assert_eq!(modality, deserialized);
        }
    }

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Internal.trust_score() > TrustLevel::Trusted.trust_score());
        assert!(TrustLevel::Trusted.trust_score() > TrustLevel::Verified.trust_score());
        assert!(TrustLevel::Verified.trust_score() > TrustLevel::Public.trust_score());
    }
}
