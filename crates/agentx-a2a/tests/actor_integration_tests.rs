//! Integration tests for Actix Actor-based A2A protocol implementation

use actix::prelude::*;
use agentx_a2a::*;
use std::time::Duration;

#[actix_rt::test]
async fn test_protocol_actor_message_processing() {
    // Start actor system
    let system = System::new();
    
    system.block_on(async {
        // Create protocol configuration
        let config = ProtocolConfig {
            max_message_size: 1024 * 1024,
            default_timeout: 30,
            max_hops: 10,
            validate_messages: true,
            cache_capabilities: true,
            rate_limit: None,
            handler_pool_size: Some(5),
        };
        
        // Start protocol actor
        let protocol_actor = A2AProtocolActor::new(config).start();
        
        // Create test message
        let message = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Hello from actor test!".to_string(),
        );
        
        let context = ProcessingContext::new()
            .with_priority(8)
            .with_timeout(5000);
        
        // Send message to protocol actor
        let result = protocol_actor
            .send(ProcessA2AMessage { message: message.clone(), context })
            .await;
        
        // Verify response
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_ok());
        
        let response_message = response.unwrap();
        assert!(response_message.is_some());
        
        let response_message = response_message.unwrap();
        assert_eq!(response_message.message_type, MessageType::Response);
        assert_eq!(response_message.from, "agent2");
        assert_eq!(response_message.to, "agent1");
        
        // Get protocol statistics
        let stats_result = protocol_actor.send(GetProtocolStats).await;
        assert!(stats_result.is_ok());
        
        let stats = stats_result.unwrap();
        assert_eq!(stats.messages_processed, 1);
        assert_eq!(stats.messages_failed, 0);
        assert!(stats.average_processing_time_ms > 0.0);
        
        System::current().stop();
    });
}

#[actix_rt::test]
async fn test_registry_actor_agent_management() {
    let system = System::new();
    
    system.block_on(async {
        // Create registry configuration
        let config = RegistryConfig {
            health_check_interval_ms: 1000,
            agent_timeout_ms: 5000,
            max_error_count: 3,
            enable_auto_cleanup: false, // Disable for test
        };
        
        // Start registry actor
        let registry_actor = AgentRegistryActor::new(config).start();
        
        // Create test agent
        let agent_card = AgentCard::new(
            "test-agent-1".to_string(),
            "Test Agent 1".to_string(),
            "A test agent for actor testing".to_string(),
            "1.0.0".to_string(),
        ).add_capability(
            Capability::new(
                "text_generation".to_string(),
                "Generate text content".to_string(),
                CapabilityType::TextGeneration,
            )
        ).add_endpoint(
            Endpoint::new("http".to_string(), "http://localhost:8080".to_string())
        );
        
        // Register agent
        let register_result = registry_actor
            .send(RegisterAgent { agent_card: agent_card.clone() })
            .await;
        
        assert!(register_result.is_ok());
        assert!(register_result.unwrap().is_ok());
        
        // Get agent
        let get_result = registry_actor
            .send(GetAgent { agent_id: "test-agent-1".to_string() })
            .await;
        
        assert!(get_result.is_ok());
        let get_response = get_result.unwrap();
        assert!(get_response.is_ok());
        
        let retrieved_agent = get_response.unwrap();
        assert!(retrieved_agent.is_some());
        assert_eq!(retrieved_agent.unwrap().id, "test-agent-1");
        
        // List agents
        let list_result = registry_actor
            .send(ListAgents { filter: None })
            .await;
        
        assert!(list_result.is_ok());
        let list_response = list_result.unwrap();
        assert!(list_response.is_ok());
        
        let agents = list_response.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "test-agent-1");
        
        // Update agent status
        let update_result = registry_actor
            .send(UpdateAgentStatus {
                agent_id: "test-agent-1".to_string(),
                status: AgentStatus::Busy,
                response_time_ms: Some(150),
            })
            .await;
        
        assert!(update_result.is_ok());
        assert!(update_result.unwrap().is_ok());
        
        // Get registry statistics
        let stats_result = registry_actor.send(GetRegistryStats).await;
        assert!(stats_result.is_ok());
        
        let stats = stats_result.unwrap();
        assert_eq!(stats.total_agents, 1);
        assert_eq!(stats.registration_events, 1);
        
        // Unregister agent
        let unregister_result = registry_actor
            .send(UnregisterAgent { agent_id: "test-agent-1".to_string() })
            .await;
        
        assert!(unregister_result.is_ok());
        assert!(unregister_result.unwrap().is_ok());
        
        // Verify agent is removed
        let final_list_result = registry_actor
            .send(ListAgents { filter: None })
            .await;
        
        assert!(final_list_result.is_ok());
        let final_agents = final_list_result.unwrap().unwrap();
        assert_eq!(final_agents.len(), 0);
        
        System::current().stop();
    });
}

#[actix_rt::test]
async fn test_capability_discovery_with_actors() {
    let system = System::new();
    
    system.block_on(async {
        let config = RegistryConfig::default();
        let registry_actor = AgentRegistryActor::new(config).start();
        
        // Register multiple agents with different capabilities
        let agent1 = AgentCard::new(
            "text-agent".to_string(),
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
            "image-agent".to_string(),
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
        
        // Register both agents
        registry_actor.send(RegisterAgent { agent_card: agent1 }).await.unwrap().unwrap();
        registry_actor.send(RegisterAgent { agent_card: agent2 }).await.unwrap().unwrap();
        
        // Create capability query for text generation
        let query = CapabilityQuery {
            required: vec![
                CapabilityRequirement {
                    name: "text_generation".to_string(),
                    capability_type: Some(CapabilityType::TextGeneration),
                    parameters: std::collections::HashMap::new(),
                    min_version: None,
                }
            ],
            optional: vec![
                CapabilityRequirement {
                    name: "text_analysis".to_string(),
                    capability_type: Some(CapabilityType::TextGeneration),
                    parameters: std::collections::HashMap::new(),
                    min_version: None,
                }
            ],
            filters: QueryFilters::default(),
            max_results: 10,
        };
        
        // Discover agents
        let discover_result = registry_actor
            .send(DiscoverAgents { query })
            .await;
        
        assert!(discover_result.is_ok());
        let discover_response = discover_result.unwrap();
        assert!(discover_response.is_ok());
        
        let matches = discover_response.unwrap();
        
        // Should find text-agent but not image-agent
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].agent_card.id, "text-agent");
        assert!(matches[0].score > 0.0);
        assert!(matches[0].matched_capabilities.contains(&"text_generation".to_string()));
        assert!(matches[0].missing_capabilities.is_empty());
        
        System::current().stop();
    });
}

#[actix_rt::test]
async fn test_actor_system_performance() {
    let system = System::new();
    
    system.block_on(async {
        let config = ProtocolConfig::default();
        let protocol_actor = A2AProtocolActor::new(config).start();
        
        let num_messages = 100;
        let start_time = std::time::Instant::now();
        
        // Send multiple messages concurrently
        let mut futures = Vec::new();
        
        for i in 0..num_messages {
            let message = A2AMessage::text(
                format!("agent{}", i),
                "target-agent".to_string(),
                format!("Performance test message {}", i),
            );
            
            let context = ProcessingContext::new();
            let future = protocol_actor.send(ProcessA2AMessage { message, context });
            futures.push(future);
        }
        
        // Wait for all messages to be processed
        let results = futures::future::join_all(futures).await;
        
        let elapsed = start_time.elapsed();
        
        // Verify all messages were processed successfully
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
        
        // Check performance - should process 100 messages in reasonable time
        let messages_per_second = num_messages as f64 / elapsed.as_secs_f64();
        println!("Processed {} messages in {:?} ({:.2} msg/s)", 
                num_messages, elapsed, messages_per_second);
        
        // Should be able to process at least 50 messages per second
        assert!(messages_per_second > 50.0, 
               "Performance too low: {:.2} msg/s", messages_per_second);
        
        // Get final statistics
        let stats = protocol_actor.send(GetProtocolStats).await.unwrap();
        assert_eq!(stats.messages_processed, num_messages);
        assert_eq!(stats.messages_failed, 0);
        
        // Average processing time should be reasonable (< 20ms per message)
        assert!(stats.average_processing_time_ms < 20.0,
               "Average processing time too high: {:.2}ms", stats.average_processing_time_ms);
        
        System::current().stop();
    });
}

#[actix_rt::test]
async fn test_actor_fault_tolerance() {
    let system = System::new();
    
    system.block_on(async {
        let config = ProtocolConfig {
            validate_messages: true,
            max_message_size: 100, // Very small size to trigger validation errors
            ..Default::default()
        };
        
        let protocol_actor = A2AProtocolActor::new(config).start();
        
        // Send a message that will fail validation (too large)
        let large_message = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "x".repeat(200), // Larger than max_message_size
        );
        
        let context = ProcessingContext::new();
        let result = protocol_actor
            .send(ProcessA2AMessage { message: large_message, context })
            .await;
        
        // Should get validation error
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_err());
        
        // Actor should still be responsive after error
        let valid_message = A2AMessage::text(
            "agent1".to_string(),
            "agent2".to_string(),
            "Valid message".to_string(),
        );
        
        let context = ProcessingContext::new();
        let result = protocol_actor
            .send(ProcessA2AMessage { message: valid_message, context })
            .await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
        
        // Check statistics - should show both successful and failed messages
        let stats = protocol_actor.send(GetProtocolStats).await.unwrap();
        assert_eq!(stats.messages_processed, 1);
        assert_eq!(stats.messages_failed, 1);
        
        System::current().stop();
    });
}
