//! Integration tests for A2A protocol implementation

use agentx_a2a::*;
use chrono::Utc;
use std::collections::HashMap;
use tokio_test;

#[tokio::test]
async fn test_a2a_message_creation_and_serialization() {
    // Test creating different types of A2A messages
    let text_message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Hello, world!".to_string(),
    );
    
    assert_eq!(text_message.from, "agent1");
    assert_eq!(text_message.to, "agent2");
    assert_eq!(text_message.message_type, MessageType::Request);
    assert_eq!(text_message.version, A2A_VERSION);
    
    // Test serialization
    let json = serde_json::to_string(&text_message).unwrap();
    let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(text_message, deserialized);
}

#[tokio::test]
async fn test_tool_call_message() {
    let tool_call = A2AMessage::new(
        "agent1".to_string(),
        "agent2".to_string(),
        MessageType::Request,
        MessagePayload::ToolCall(ToolCallPayload {
            name: "search".to_string(),
            call_id: "call123".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("query".to_string(), serde_json::Value::String("test".to_string()));
                params
            },
        }),
    );
    
    // Verify tool call structure
    if let MessagePayload::ToolCall(tool_call_payload) = &tool_call.payload {
        assert_eq!(tool_call_payload.name, "search");
        assert_eq!(tool_call_payload.call_id, "call123");
        assert!(tool_call_payload.parameters.contains_key("query"));
    } else {
        panic!("Expected tool call payload");
    }
    
    // Test serialization
    let json = serde_json::to_string(&tool_call).unwrap();
    let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(tool_call, deserialized);
}

#[tokio::test]
async fn test_agent_card_creation_and_capabilities() {
    let mut agent_card = AgentCard::new(
        "test-agent".to_string(),
        "Test Agent".to_string(),
        "A test agent for integration testing".to_string(),
        "1.0.0".to_string(),
    );
    
    // Add capabilities
    let text_capability = Capability::new(
        "text_generation".to_string(),
        "Generate text content".to_string(),
        CapabilityType::TextGeneration,
    ).with_input_schema(serde_json::json!({
        "type": "object",
        "properties": {
            "prompt": {"type": "string"},
            "max_tokens": {"type": "integer"}
        }
    }));
    
    agent_card = agent_card.add_capability(text_capability);
    
    // Add endpoint
    let endpoint = Endpoint::new(
        "http".to_string(),
        "http://localhost:8080".to_string(),
    ).with_protocol("a2a".to_string());
    
    agent_card = agent_card.add_endpoint(endpoint);
    
    // Verify agent card
    assert_eq!(agent_card.id, "test-agent");
    assert_eq!(agent_card.capabilities.len(), 1);
    assert_eq!(agent_card.endpoints.len(), 1);
    assert!(agent_card.has_capability("text_generation"));
    assert!(!agent_card.has_capability("image_processing"));
    
    // Test serialization
    let json = serde_json::to_string(&agent_card).unwrap();
    let deserialized: AgentCard = serde_json::from_str(&json).unwrap();
    assert_eq!(agent_card, deserialized);
}

#[tokio::test]
async fn test_capability_discovery() {
    let mut discovery = CapabilityDiscovery::new();
    
    // Create test agents with different capabilities
    let agent1 = AgentCard::new(
        "agent1".to_string(),
        "Text Agent".to_string(),
        "Specializes in text processing".to_string(),
        "1.0.0".to_string(),
    ).add_capability(
        Capability::new(
            "text_generation".to_string(),
            "Generate text".to_string(),
            CapabilityType::TextGeneration,
        )
    ).add_capability(
        Capability::new(
            "text_analysis".to_string(),
            "Analyze text".to_string(),
            CapabilityType::TextGeneration,
        )
    );
    
    let agent2 = AgentCard::new(
        "agent2".to_string(),
        "Image Agent".to_string(),
        "Specializes in image processing".to_string(),
        "1.0.0".to_string(),
    ).add_capability(
        Capability::new(
            "image_generation".to_string(),
            "Generate images".to_string(),
            CapabilityType::ImageProcessing,
        )
    );
    
    discovery.register_agent(agent1);
    discovery.register_agent(agent2);
    
    // Test capability query
    let query = CapabilityQuery {
        required: vec![
            CapabilityRequirement {
                name: "text_generation".to_string(),
                capability_type: Some(CapabilityType::TextGeneration),
                parameters: HashMap::new(),
                min_version: None,
            }
        ],
        optional: vec![
            CapabilityRequirement {
                name: "text_analysis".to_string(),
                capability_type: Some(CapabilityType::TextGeneration),
                parameters: HashMap::new(),
                min_version: None,
            }
        ],
        filters: QueryFilters::default(),
        max_results: 10,
    };
    
    let matches = discovery.discover(&query);
    
    // Should find agent1 but not agent2
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].agent_card.id, "agent1");
    assert!(matches[0].score > 0.0);
    assert!(matches[0].matched_capabilities.contains(&"text_generation".to_string()));
    assert!(matches[0].missing_capabilities.is_empty());
}

#[tokio::test]
async fn test_protocol_engine_agent_registration() {
    let config = ProtocolConfig::default();
    let engine = A2AProtocolEngine::new(config);
    
    // Create test agent
    let agent_card = AgentCard::new(
        "test-agent".to_string(),
        "Test Agent".to_string(),
        "A test agent".to_string(),
        "1.0.0".to_string(),
    ).add_endpoint(
        Endpoint::new("http".to_string(), "http://localhost:8080".to_string())
    );
    
    // Register agent
    let result = engine.register_agent(agent_card.clone()).await;
    assert!(result.is_ok());
    
    // Test capability discovery
    let query = CapabilityQuery {
        required: Vec::new(),
        optional: Vec::new(),
        filters: QueryFilters::default(),
        max_results: 10,
    };
    
    let matches = engine.discover_agents(&query).await.unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].agent_card.id, "test-agent");
    
    // Unregister agent
    let result = engine.unregister_agent("test-agent").await;
    assert!(result.is_ok());
    
    // Should not find agent after unregistration
    let matches = engine.discover_agents(&query).await.unwrap();
    assert_eq!(matches.len(), 0);
}

#[tokio::test]
async fn test_message_processing_with_echo_handler() {
    let config = ProtocolConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // Register echo handler
    use agentx_a2a::server::EchoMessageHandler;
    engine.register_handler(Box::new(EchoMessageHandler));
    
    // Create test message
    let message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Hello, echo!".to_string(),
    );
    
    // Process message
    let response = engine.process_message(message.clone()).await.unwrap();
    
    // Should get echo response
    assert!(response.is_some());
    let response = response.unwrap();
    assert_eq!(response.message_type, MessageType::Response);
    assert_eq!(response.from, "agent2");
    assert_eq!(response.to, "agent1");
    
    // Should echo the same payload
    if let (MessagePayload::Text(original), MessagePayload::Text(echoed)) = 
        (&message.payload, &response.payload) {
        assert_eq!(original.content, echoed.content);
    } else {
        panic!("Expected text payloads");
    }
}

#[tokio::test]
async fn test_message_validation() {
    let config = ProtocolConfig::default();
    let engine = A2AProtocolEngine::new(config);
    
    // Test valid message
    let valid_message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Valid message".to_string(),
    );
    
    let result = engine.process_message(valid_message).await;
    // Should not fail validation (though might fail processing due to no handler)
    assert!(result.is_ok() || matches!(result, Err(A2AError::Internal(_))));
    
    // Test invalid message (empty from field)
    let mut invalid_message = A2AMessage::text(
        "".to_string(), // Empty from field
        "agent2".to_string(),
        "Invalid message".to_string(),
    );
    
    let result = engine.process_message(invalid_message).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(A2AError::Validation(_))));
}

#[tokio::test]
async fn test_message_expiration() {
    let config = ProtocolConfig::default();
    let engine = A2AProtocolEngine::new(config);
    
    // Create expired message
    let expired_message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Expired message".to_string(),
    ).with_expiration(Utc::now() - chrono::Duration::hours(1)); // Expired 1 hour ago
    
    let result = engine.process_message(expired_message).await;
    assert!(result.is_err());
    assert!(matches!(result, Err(A2AError::MessageExpired)));
}

#[tokio::test]
async fn test_error_response_creation() {
    let original_message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Test message".to_string(),
    );
    
    let error_response = A2AMessage::error_response(
        &original_message,
        "TEST_ERROR".to_string(),
        "This is a test error".to_string(),
    );
    
    assert_eq!(error_response.message_type, MessageType::Response);
    assert_eq!(error_response.from, "agent2");
    assert_eq!(error_response.to, "agent1");
    
    if let MessagePayload::Error(error_payload) = &error_response.payload {
        assert_eq!(error_payload.code, "TEST_ERROR");
        assert_eq!(error_payload.message, "This is a test error");
    } else {
        panic!("Expected error payload");
    }
}

#[tokio::test]
async fn test_conversation_id_propagation() {
    let conversation_id = "conv-123".to_string();
    
    let original_message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Hello".to_string(),
    ).with_conversation_id(conversation_id.clone());
    
    let response = A2AMessage::response(
        &original_message,
        MessagePayload::Text(TextPayload {
            content: "Hi there!".to_string(),
            format: "plain".to_string(),
            language: None,
        }),
    );
    
    assert_eq!(response.conversation_id, Some(conversation_id));
}

#[tokio::test]
async fn test_message_priority_and_metadata() {
    let message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Priority message".to_string(),
    ).with_priority(9)
     .with_metadata("source".to_string(), serde_json::Value::String("test".to_string()))
     .with_metadata("urgent".to_string(), serde_json::Value::Bool(true));
    
    assert_eq!(message.priority, 9);
    assert_eq!(message.metadata.len(), 2);
    assert_eq!(message.metadata.get("source"), Some(&serde_json::Value::String("test".to_string())));
    assert_eq!(message.metadata.get("urgent"), Some(&serde_json::Value::Bool(true)));
}

#[tokio::test]
async fn test_performance_message_routing_latency() {
    let config = ProtocolConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // Register echo handler
    use agentx_a2a::server::EchoMessageHandler;
    engine.register_handler(Box::new(EchoMessageHandler));
    
    // Measure message processing time
    let start = std::time::Instant::now();
    
    let message = A2AMessage::text(
        "agent1".to_string(),
        "agent2".to_string(),
        "Performance test message".to_string(),
    );
    
    let _response = engine.process_message(message).await.unwrap();
    
    let elapsed = start.elapsed();
    
    // Should process message in less than 10ms (design goal)
    assert!(elapsed.as_millis() < 10, "Message processing took {}ms, expected <10ms", elapsed.as_millis());
}
