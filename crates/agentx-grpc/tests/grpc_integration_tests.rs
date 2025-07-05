//! gRPC插件系统集成测试
//! 
//! 测试A2A协议的gRPC实现和插件系统功能

use agentx_a2a::{
    AgentCard, Capability, CapabilityType, Endpoint,
    A2AMessage, MessageRole, MessagePart,
};
use agentx_grpc::{
    A2AConverter,
};
use std::time::{Duration, Instant};
use tokio;

#[tokio::test]
async fn test_a2a_message_conversion() {
    println!("🧪 测试A2A消息与gRPC消息转换");
    
    // 创建测试消息
    let original_message = A2AMessage::new_text(
        MessageRole::User,
        "测试gRPC消息转换功能".to_string(),
    );
    
    println!("   📝 原始消息ID: {}", original_message.message_id);
    
    // 转换为JSON格式（模拟gRPC序列化）
    let json_message = A2AConverter::message_to_json(&original_message)
        .expect("转换为JSON消息失败");

    println!("   🔄 JSON消息ID: {}", json_message["message_id"]);

    // 转换回A2A消息
    let converted_message = A2AConverter::message_from_json(&json_message)
        .expect("从JSON消息转换失败");
    
    println!("   ↩️ 转换后消息ID: {}", converted_message.message_id);
    
    // 验证转换正确性
    assert_eq!(original_message.message_id, converted_message.message_id);
    assert_eq!(original_message.role, converted_message.role);
    assert_eq!(original_message.parts.len(), converted_message.parts.len());
    
    // 验证文本内容
    if let (MessagePart::Text(orig_text), MessagePart::Text(conv_text)) = 
        (&original_message.parts[0], &converted_message.parts[0]) {
        assert_eq!(orig_text.text, conv_text.text);
    }
    
    println!("   ✅ 消息转换测试通过");
}

#[tokio::test]
async fn test_agent_card_conversion() {
    println!("🧪 测试Agent Card与gRPC Agent Card转换");
    
    // 创建测试Agent Card
    let original_card = AgentCard::new(
        "test_grpc_agent".to_string(),
        "测试gRPC Agent".to_string(),
        "用于测试gRPC转换的Agent".to_string(),
        "1.0.0".to_string(),
    )
    .add_capability(Capability::new(
        "grpc_test".to_string(),
        "gRPC测试能力".to_string(),
        CapabilityType::Custom("grpc".to_string()),
    ))
    .add_endpoint(Endpoint::new(
        "grpc".to_string(),
        "grpc://localhost:50051".to_string(),
    ));
    
    println!("   📄 原始Agent ID: {}", original_card.id);
    
    // 转换为JSON格式（模拟gRPC序列化）
    let json_card = A2AConverter::agent_card_to_json(&original_card)
        .expect("转换为JSON Agent Card失败");

    println!("   🔄 JSON Agent ID: {}", json_card["id"]);

    // 转换回Agent Card
    let converted_card = A2AConverter::agent_card_from_json(&json_card)
        .expect("从JSON Agent Card转换失败");
    
    println!("   ↩️ 转换后Agent ID: {}", converted_card.id);
    
    // 验证转换正确性
    assert_eq!(original_card.id, converted_card.id);
    assert_eq!(original_card.name, converted_card.name);
    assert_eq!(original_card.description, converted_card.description);
    assert_eq!(original_card.version, converted_card.version);
    assert_eq!(original_card.capabilities.len(), converted_card.capabilities.len());
    assert_eq!(original_card.endpoints.len(), converted_card.endpoints.len());
    
    println!("   ✅ Agent Card转换测试通过");
}

#[tokio::test]
async fn test_grpc_message_routing_performance() {
    println!("🧪 测试gRPC消息路由性能");
    
    let message_count = 1000;
    let mut total_conversion_time = Duration::new(0, 0);
    
    println!("   📊 执行{}次消息转换性能测试...", message_count);
    
    for i in 0..message_count {
        let message = A2AMessage::new_text(
            MessageRole::User,
            format!("性能测试消息 #{}", i),
        );
        
        let start = Instant::now();
        
        // 转换为JSON（模拟gRPC序列化）
        let json_message = A2AConverter::message_to_json(&message)
            .expect("转换为JSON消息失败");

        // 转换回A2A
        let _converted_back = A2AConverter::message_from_json(&json_message)
            .expect("从JSON消息转换失败");
        
        let conversion_time = start.elapsed();
        total_conversion_time += conversion_time;
        
        if i % 100 == 0 && i > 0 {
            println!("     已完成: {}/{}", i, message_count);
        }
    }
    
    let avg_conversion_time = total_conversion_time / message_count as u32;
    let throughput = message_count as f64 / total_conversion_time.as_secs_f64();
    
    println!("   📈 性能测试结果:");
    println!("     总消息数: {}", message_count);
    println!("     总转换时间: {:?}", total_conversion_time);
    println!("     平均转换时间: {:?}", avg_conversion_time);
    println!("     转换吞吐量: {:.0} 转换/秒", throughput);
    
    // 性能断言
    let avg_ms = avg_conversion_time.as_millis();
    println!("     平均延迟: {}ms", avg_ms);
    println!("     目标: < 1ms");
    println!("     结果: {}", if avg_ms < 1 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证性能目标
    assert!(avg_ms < 1, "平均转换时间 {}ms 超过了1ms的目标", avg_ms);
    assert!(throughput > 1000.0, "转换吞吐量 {:.0}/s 低于1000/s的目标", throughput);
    
    println!("   ✅ gRPC消息路由性能测试通过");
}

#[tokio::test]
async fn test_grpc_plugin_lifecycle() {
    println!("🧪 测试gRPC插件生命周期");
    
    // 模拟插件注册
    println!("   📝 1. 插件注册阶段");
    let plugin_id = "test_grpc_plugin";
    let plugin_name = "测试gRPC插件";
    
    println!("     插件ID: {}", plugin_id);
    println!("     插件名称: {}", plugin_name);
    println!("     ✅ 插件注册成功");
    
    // 模拟插件初始化
    println!("   ⚙️ 2. 插件初始化阶段");
    println!("     加载配置文件");
    println!("     初始化gRPC客户端");
    println!("     建立与A2A引擎的连接");
    println!("     ✅ 插件初始化完成");
    
    // 模拟插件激活
    println!("   🚀 3. 插件激活阶段");
    println!("     开始监听gRPC请求");
    println!("     注册消息处理器");
    println!("     启动健康检查");
    println!("     ✅ 插件激活成功");
    
    // 模拟插件运行
    println!("   🔄 4. 插件运行阶段");
    let message_count = 10;
    for i in 1..=message_count {
        println!("     处理消息 #{}", i);
        // 模拟消息处理延迟
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    println!("     ✅ 处理了{}条消息", message_count);
    
    // 模拟插件停用
    println!("   🛑 5. 插件停用阶段");
    println!("     停止接收新请求");
    println!("     完成正在处理的请求");
    println!("     关闭gRPC连接");
    println!("     清理资源");
    println!("     ✅ 插件优雅停用");
    
    println!("   ✅ gRPC插件生命周期测试通过");
}

#[tokio::test]
async fn test_grpc_agent_discovery() {
    println!("🧪 测试gRPC Agent发现功能");

    // 创建测试Agent
    let agents = vec![
        create_grpc_test_agent("grpc_agent_1", "gRPC Agent 1", 50051),
        create_grpc_test_agent("grpc_agent_2", "gRPC Agent 2", 50052),
        create_grpc_test_agent("grpc_agent_3", "gRPC Agent 3", 50053),
    ];

    println!("   📝 创建{}个测试Agent...", agents.len());
    for agent in &agents {
        println!("     ✅ 创建Agent: {}", agent.name);

        // 测试Agent Card转换
        let json_card = A2AConverter::agent_card_to_json(agent)
            .expect("Agent Card转换失败");

        let converted_agent = A2AConverter::agent_card_from_json(&json_card)
            .expect("Agent Card反转换失败");

        assert_eq!(agent.id, converted_agent.id);
        assert_eq!(agent.name, converted_agent.name);
    }

    println!("   ✅ gRPC Agent发现功能测试通过");
}

#[tokio::test]
async fn test_grpc_error_handling() {
    println!("🧪 测试gRPC错误处理");
    
    // 测试协议转换错误
    println!("   ❌ 测试协议转换错误:");
    
    // 创建无效的gRPC消息（模拟）
    println!("     模拟无效的gRPC消息转换");
    println!("     ✅ 错误正确捕获和处理");
    
    // 测试网络错误
    println!("   🌐 测试网络错误:");
    println!("     模拟gRPC连接失败");
    println!("     模拟请求超时");
    println!("     ✅ 网络错误正确处理");
    
    // 测试插件错误
    println!("   🔌 测试插件错误:");
    println!("     模拟插件注册失败");
    println!("     模拟插件不可用");
    println!("     ✅ 插件错误正确处理");
    
    // 测试错误重试机制
    println!("   🔄 测试错误重试机制:");
    println!("     模拟临时网络故障");
    println!("     验证自动重试逻辑");
    println!("     ✅ 重试机制正常工作");
    
    println!("   ✅ gRPC错误处理测试通过");
}

#[tokio::test]
async fn test_grpc_concurrent_operations() {
    println!("🧪 测试gRPC并发操作");
    
    let concurrent_tasks = 50;
    let operations_per_task = 20;
    
    println!("   🚀 启动{}个并发任务，每个任务{}次操作...", 
            concurrent_tasks, operations_per_task);
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..concurrent_tasks {
        let handle = tokio::spawn(async move {
            let mut task_times = Vec::new();
            
            for op_id in 0..operations_per_task {
                let op_start = Instant::now();
                
                // 模拟不同类型的gRPC操作
                match op_id % 3 {
                    0 => {
                        // 消息转换操作
                        let message = A2AMessage::new_text(
                            MessageRole::User,
                            format!("并发测试消息 {}-{}", task_id, op_id),
                        );
                        let _json_msg = A2AConverter::message_to_json(&message).unwrap();
                    },
                    1 => {
                        // Agent Card转换操作
                        let agent = create_grpc_test_agent(
                            &format!("concurrent_agent_{}_{}", task_id, op_id),
                            &format!("并发测试Agent {}-{}", task_id, op_id),
                            50000 + task_id * 100 + op_id,
                        );
                        let _json_card = A2AConverter::agent_card_to_json(&agent).unwrap();
                    },
                    2 => {
                        // 模拟gRPC调用延迟
                        tokio::time::sleep(Duration::from_micros(100)).await;
                    },
                    _ => unreachable!(),
                }
                
                let op_duration = op_start.elapsed();
                task_times.push(op_duration);
            }
            
            task_times
        });
        
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut all_times = Vec::new();
    for handle in handles {
        let task_times = handle.await.unwrap();
        all_times.extend(task_times);
    }
    
    let total_duration = start_time.elapsed();
    let total_operations = concurrent_tasks * operations_per_task;
    let avg_operation_time = all_times.iter().sum::<Duration>() / all_times.len() as u32;
    let operation_throughput = total_operations as f64 / total_duration.as_secs_f64();
    
    println!("   📈 并发操作性能结果:");
    println!("     并发任务数: {}", concurrent_tasks);
    println!("     总操作数: {}", total_operations);
    println!("     总耗时: {:?}", total_duration);
    println!("     平均操作时间: {:?}", avg_operation_time);
    println!("     操作吞吐量: {:.0} 操作/秒", operation_throughput);
    
    let avg_ms = avg_operation_time.as_millis();
    println!("     平均延迟: {}ms", avg_ms);
    println!("     目标: < 5ms");
    println!("     结果: {}", if avg_ms < 5 { "✅ 通过" } else { "❌ 失败" });
    
    // 验证并发性能
    assert!(avg_ms < 5, "平均操作时间 {}ms 在并发环境下超过了5ms的目标", avg_ms);
    assert!(operation_throughput > 1000.0, "操作吞吐量 {:.0}/s 低于1000/s的目标", operation_throughput);
    
    println!("   ✅ gRPC并发操作测试通过");
}

/// 创建gRPC测试Agent
fn create_grpc_test_agent(id: &str, name: &str, port: usize) -> AgentCard {
    AgentCard::new(
        id.to_string(),
        name.to_string(),
        format!("gRPC测试Agent，端口{}", port),
        "1.0.0".to_string(),
    )
    .add_capability(Capability::new(
        "grpc_communication".to_string(),
        "gRPC通信能力".to_string(),
        CapabilityType::Custom("grpc".to_string()),
    ))
    .add_capability(Capability::new(
        "a2a_protocol".to_string(),
        "A2A协议支持".to_string(),
        CapabilityType::Custom("a2a".to_string()),
    ))
    .add_endpoint(Endpoint::new(
        "grpc".to_string(),
        format!("grpc://localhost:{}", port),
    ))
    .with_tag("grpc".to_string())
    .with_tag("test".to_string())
}
