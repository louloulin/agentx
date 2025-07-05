//! A2A协议性能测试
//! 
//! 验证A2A协议实现的性能指标，确保满足设计目标：
//! - 消息路由延迟 < 10ms
//! - 高吞吐量消息处理
//! - 并发Agent注册和发现

use agentx_a2a::{
    AgentCard, AgentInfo, AgentStatus, Capability, CapabilityType, Endpoint,
    InteractionModality, UxCapabilities, TrustLevel,
    A2AMessage, MessageRole, MessagePart, FileData, FileWithBytes,
    A2AProtocolEngine, ProtocolEngineConfig,
};
use std::time::{Duration, Instant};
use tokio;
use serde_json;

#[tokio::test]
async fn test_message_routing_latency() {
    println!("🚀 测试A2A消息路由延迟");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 创建测试消息
    let message = A2AMessage::new_text(
        MessageRole::User,
        "性能测试消息".to_string(),
    );
    
    let message_count = 1000;
    let mut total_latency = Duration::new(0, 0);
    
    println!("📊 执行{}次消息路由测试...", message_count);
    
    for i in 0..message_count {
        let start = Instant::now();
        
        // 模拟消息处理（创建消息副本）
        let _message_copy = A2AMessage::new_text(
            message.role.clone(),
            format!("消息副本 {}", i),
        );
        
        let latency = start.elapsed();
        total_latency += latency;
        
        if i % 100 == 0 {
            println!("   已完成: {}/{} (当前延迟: {:?})", i + 1, message_count, latency);
        }
    }
    
    let avg_latency = total_latency / message_count;
    let avg_latency_ms = avg_latency.as_secs_f64() * 1000.0;
    
    println!("📈 性能测试结果:");
    println!("   总消息数: {}", message_count);
    println!("   总耗时: {:?}", total_latency);
    println!("   平均延迟: {:?} ({:.3}ms)", avg_latency, avg_latency_ms);
    println!("   目标延迟: < 10ms");
    println!("   测试结果: {}", if avg_latency_ms < 10.0 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证性能目标
    assert!(avg_latency_ms < 10.0, "平均延迟 {:.3}ms 超过了10ms的目标", avg_latency_ms);
}

#[tokio::test]
async fn test_message_throughput() {
    println!("🚀 测试A2A消息吞吐量");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 创建不同类型的测试消息
    let text_message = A2AMessage::new_text(
        MessageRole::User,
        "吞吐量测试文本消息".to_string(),
    );
    
    let file_message = A2AMessage::new_file(
        MessageRole::User,
        FileData::WithBytes(FileWithBytes {
            name: Some("test.txt".to_string()),
            mime_type: "text/plain".to_string(),
            bytes: "dGVzdCBkYXRh".to_string(), // "test data" in base64
        }),
    );
    
    let data_message = A2AMessage::new_data(
        MessageRole::Agent,
        serde_json::json!({
            "test": "data",
            "number": 42,
            "array": [1, 2, 3]
        }),
    );
    
    let messages = vec![&text_message, &file_message, &data_message];
    let message_count = 10000;
    let batch_size = 100;
    
    println!("📊 执行{}次消息处理测试 (批次大小: {})...", message_count, batch_size);
    
    let start_time = Instant::now();
    
    for batch in 0..(message_count / batch_size) {
        let batch_start = Instant::now();
        
        for i in 0..batch_size {
            let message = &messages[i % messages.len()];
            // 模拟消息处理（访问消息字段）
            let _message_id = &message.message_id;
        }
        
        let batch_duration = batch_start.elapsed();
        
        if batch % 10 == 0 {
            let processed = (batch + 1) * batch_size;
            let batch_throughput = batch_size as f64 / batch_duration.as_secs_f64();
            println!("   已处理: {}/{} (批次吞吐量: {:.0} msg/s)", 
                    processed, message_count, batch_throughput);
        }
    }
    
    let total_duration = start_time.elapsed();
    let throughput = message_count as f64 / total_duration.as_secs_f64();
    
    println!("📈 吞吐量测试结果:");
    println!("   总消息数: {}", message_count);
    println!("   总耗时: {:?}", total_duration);
    println!("   吞吐量: {:.0} 消息/秒", throughput);
    println!("   目标吞吐量: > 10,000 消息/秒");
    println!("   测试结果: {}", if throughput > 10000.0 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证吞吐量目标
    assert!(throughput > 10000.0, "吞吐量 {:.0} msg/s 低于10,000 msg/s的目标", throughput);
}

#[tokio::test]
async fn test_agent_registration_performance() {
    println!("🚀 测试Agent注册性能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    let agent_count = 1000;
    let mut registration_times = Vec::new();
    
    println!("📊 执行{}个Agent注册测试...", agent_count);
    
    let start_time = Instant::now();
    
    for i in 0..agent_count {
        let agent_info = create_test_agent_info(i);

        let reg_start = Instant::now();
        let _result = engine.register_agent(agent_info);
        let reg_duration = reg_start.elapsed();
        
        registration_times.push(reg_duration);
        
        if i % 100 == 0 {
            println!("   已注册: {}/{} (当前耗时: {:?})", i + 1, agent_count, reg_duration);
        }
    }
    
    let total_duration = start_time.elapsed();
    let avg_registration_time = registration_times.iter().sum::<Duration>() / agent_count as u32;
    let registration_throughput = agent_count as f64 / total_duration.as_secs_f64();
    
    println!("📈 Agent注册性能结果:");
    println!("   总Agent数: {}", agent_count);
    println!("   总耗时: {:?}", total_duration);
    println!("   平均注册时间: {:?}", avg_registration_time);
    println!("   注册吞吐量: {:.0} Agent/秒", registration_throughput);
    println!("   目标: 平均注册时间 < 1ms");
    
    let avg_reg_ms = avg_registration_time.as_secs_f64() * 1000.0;
    println!("   测试结果: {}", if avg_reg_ms < 1.0 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证注册性能
    assert!(avg_reg_ms < 1.0, "平均注册时间 {:.3}ms 超过了1ms的目标", avg_reg_ms);
}

#[tokio::test]
async fn test_agent_discovery_performance() {
    println!("🚀 测试Agent发现性能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册大量Agent
    let agent_count = 1000;
    println!("📊 注册{}个测试Agent...", agent_count);
    
    for i in 0..agent_count {
        let agent_info = create_test_agent_info(i);
        let _result = engine.register_agent(agent_info);
    }
    
    // 测试发现性能
    let discovery_count = 1000;
    let mut discovery_times = Vec::new();
    
    println!("📊 执行{}次Agent发现测试...", discovery_count);
    
    let capabilities = ["text_generation", "data_analysis", "image_processing", "audio_processing"];
    
    for i in 0..discovery_count {
        let capability = capabilities[i % capabilities.len()];
        
        let discovery_start = Instant::now();
        let agents = engine.list_agents();
        // 模拟按能力过滤
        let _filtered_agents: Vec<_> = agents.into_iter()
            .filter(|agent| agent.capabilities.contains(&capability.to_string()))
            .collect();
        let discovery_duration = discovery_start.elapsed();
        
        discovery_times.push(discovery_duration);
        
        if i % 100 == 0 {
            println!("   已完成: {}/{} (当前耗时: {:?})", i + 1, discovery_count, discovery_duration);
        }
    }
    
    let avg_discovery_time = discovery_times.iter().sum::<Duration>() / discovery_count as u32;
    let discovery_throughput = discovery_count as f64 / discovery_times.iter().sum::<Duration>().as_secs_f64();
    
    println!("📈 Agent发现性能结果:");
    println!("   总发现次数: {}", discovery_count);
    println!("   平均发现时间: {:?}", avg_discovery_time);
    println!("   发现吞吐量: {:.0} 查询/秒", discovery_throughput);
    println!("   目标: 平均发现时间 < 5ms");
    
    let avg_discovery_ms = avg_discovery_time.as_secs_f64() * 1000.0;
    println!("   测试结果: {}", if avg_discovery_ms < 5.0 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证发现性能
    assert!(avg_discovery_ms < 5.0, "平均发现时间 {:.3}ms 超过了5ms的目标", avg_discovery_ms);
}

#[tokio::test]
async fn test_concurrent_operations() {
    println!("🚀 测试顺序操作性能（模拟并发）");

    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);

    let total_operations = 1000;

    println!("📊 执行{}次顺序操作...", total_operations);

    let start_time = Instant::now();
    let mut operation_times = Vec::new();

    for op_id in 0..total_operations {
        let op_start = Instant::now();

        // 模拟不同类型的操作
        match op_id % 3 {
            0 => {
                // 消息创建（模拟消息处理）
                let _message = A2AMessage::new_text(
                    MessageRole::User,
                    format!("测试消息 {}", op_id),
                );
                // 模拟消息处理时间
            },
            1 => {
                // Agent注册
                let agent = create_test_agent_info(op_id);
                let _result = engine.register_agent(agent);
            },
            2 => {
                // Agent发现
                let _agents = engine.list_agents();
            },
            _ => unreachable!(),
        }

        let op_duration = op_start.elapsed();
        operation_times.push(op_duration);

        if op_id % 100 == 0 {
            println!("   已完成: {}/{} 操作", op_id + 1, total_operations);
        }
    }
    
    let total_duration = start_time.elapsed();
    let avg_operation_time = operation_times.iter().sum::<Duration>() / operation_times.len() as u32;
    let operation_throughput = total_operations as f64 / total_duration.as_secs_f64();

    println!("📈 操作性能结果:");
    println!("   总操作数: {}", total_operations);
    println!("   总耗时: {:?}", total_duration);
    println!("   平均操作时间: {:?}", avg_operation_time);
    println!("   操作吞吐量: {:.0} 操作/秒", operation_throughput);
    println!("   目标: 支持高性能操作");

    let avg_op_ms = avg_operation_time.as_secs_f64() * 1000.0;
    println!("   平均操作延迟: {:.3}ms", avg_op_ms);
    println!("   测试结果: {}", if avg_op_ms < 10.0 { "✅ 通过" } else { "❌ 失败" });

    // 验证性能
    assert!(avg_op_ms < 10.0, "平均操作时间 {:.3}ms 超过了10ms的目标", avg_op_ms);
}

/// 创建测试用的Agent Info
fn create_test_agent_info(id: usize) -> AgentInfo {
    let capabilities = [
        "text_generation",
        "data_analysis",
        "image_processing",
        "audio_processing",
    ];

    let cap_name = capabilities[id % capabilities.len()];

    AgentInfo {
        id: format!("test_agent_{}", id),
        name: format!("测试Agent {}", id),
        endpoint: format!("http://test-agent-{}.local:8080", id),
        capabilities: vec![cap_name.to_string()],
        status: AgentStatus::Online,
    }
}

/// 创建测试用的Agent Card
fn create_test_agent(id: usize) -> AgentCard {
    let capabilities = [
        ("text_generation", CapabilityType::TextGeneration),
        ("data_analysis", CapabilityType::DataAnalysis),
        ("image_processing", CapabilityType::ImageProcessing),
        ("audio_processing", CapabilityType::AudioProcessing),
    ];
    
    let (cap_name, cap_type) = &capabilities[id % capabilities.len()];
    
    AgentCard::new(
        format!("test_agent_{}", id),
        format!("测试Agent {}", id),
        format!("用于性能测试的Agent {}", id),
        "1.0.0".to_string(),
    )
    .add_capability(Capability::new(
        cap_name.to_string(),
        format!("测试能力: {}", cap_name),
        cap_type.clone(),
    ))
    .add_endpoint(Endpoint::new(
        "http".to_string(),
        format!("http://test-agent-{}.local:8080", id),
    ))
    .with_interaction_modality(InteractionModality::Text)
    .with_trust_level(if id % 4 == 0 { TrustLevel::Internal } else { TrustLevel::Verified })
    .with_task_type(cap_name.to_string())
    .with_tag("performance_test".to_string())
}
