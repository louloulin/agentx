//! A2A协议性能测试
//! 
//! 验证A2A协议实现是否满足性能要求：
//! - 消息处理延迟 < 10ms
//! - 高并发处理能力
//! - 内存使用效率

use agentx_a2a::*;
use std::time::Instant;
use tokio;

#[tokio::test]
async fn test_message_processing_latency() {
    println!("🚀 测试消息处理延迟 (目标: <10ms)");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册测试Agent
    let agent = AgentInfo {
        id: "perf_test_agent".to_string(),
        name: "性能测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["test".to_string()],
        status: AgentStatus::Online,
    };
    engine.register_agent(agent);
    
    // 测试不同类型的请求
    let test_cases = vec![
        ("submitTask", create_submit_task_request()),
        ("sendMessage", create_send_message_request()),
        ("getCapabilities", create_get_capabilities_request()),
    ];
    
    for (test_name, request) in test_cases {
        let start = Instant::now();
        let _response = engine.process_request(request).await;
        let duration = start.elapsed();
        
        let latency_ms = duration.as_millis();
        println!("  {} 延迟: {}ms", test_name, latency_ms);
        
        // 验证延迟 < 10ms
        assert!(latency_ms < 10, 
               "{} 延迟 {}ms 超过10ms目标", test_name, latency_ms);
    }
    
    println!("✅ 消息处理延迟测试通过");
}

#[tokio::test]
async fn test_throughput_performance() {
    println!("🚀 测试吞吐量性能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册Agent
    let agent = AgentInfo {
        id: "throughput_agent".to_string(),
        name: "吞吐量测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["throughput_test".to_string()],
        status: AgentStatus::Online,
    };
    engine.register_agent(agent);
    
    // 测试参数
    let message_count = 1000;
    let start_time = Instant::now();
    
    // 处理大量消息
    for i in 0..message_count {
        let message = A2AMessage::user_message(format!("测试消息 {}", i));
        let request = JsonRpcRequest::send_message(
            message,
            serde_json::Value::Number(serde_json::Number::from(i))
        );
        
        let _response = engine.process_request(request).await;
    }
    
    let total_time = start_time.elapsed();
    let total_ms = total_time.as_millis();
    let avg_latency = total_ms as f64 / message_count as f64;
    let throughput = (message_count as f64) / (total_ms as f64 / 1000.0);
    
    println!("📊 吞吐量测试结果:");
    println!("  消息数量: {}", message_count);
    println!("  总时间: {}ms", total_ms);
    println!("  平均延迟: {:.2}ms", avg_latency);
    println!("  吞吐量: {:.2} 消息/秒", throughput);
    
    // 验证平均延迟 < 5ms
    assert!(avg_latency < 5.0, 
           "平均延迟 {:.2}ms 超过5ms目标", avg_latency);
    
    // 验证吞吐量 > 100 消息/秒
    assert!(throughput > 100.0, 
           "吞吐量 {:.2} 消息/秒 低于100消息/秒目标", throughput);
    
    println!("✅ 吞吐量性能测试通过");
}

#[tokio::test]
async fn test_concurrent_processing() {
    println!("🚀 测试并发处理性能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册Agent
    let agent = AgentInfo {
        id: "concurrent_agent".to_string(),
        name: "并发测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["concurrent_test".to_string()],
        status: AgentStatus::Online,
    };
    engine.register_agent(agent);
    
    // 模拟并发请求（顺序处理以测试单线程性能）
    let concurrent_count = 100;
    let start_time = Instant::now();
    
    for i in 0..concurrent_count {
        let task = A2ATask::new(format!("concurrent_task_{}", i));
        let request = JsonRpcRequest::submit_task(
            task,
            serde_json::Value::Number(serde_json::Number::from(i))
        );
        
        let start = Instant::now();
        let response = engine.process_request(request).await;
        let duration = start.elapsed();
        
        // 验证每个请求都成功
        assert!(response.result.is_some());
        
        // 验证单个请求延迟 < 10ms
        assert!(duration.as_millis() < 10, 
               "请求 {} 延迟 {}ms 超过10ms", i, duration.as_millis());
    }
    
    let total_time = start_time.elapsed();
    let avg_time = total_time.as_millis() / concurrent_count as u128;
    
    println!("📊 并发处理结果:");
    println!("  并发请求数: {}", concurrent_count);
    println!("  总处理时间: {}ms", total_time.as_millis());
    println!("  平均处理时间: {}ms", avg_time);
    
    // 验证平均处理时间 < 5ms
    assert!(avg_time < 5, 
           "平均处理时间 {}ms 超过5ms目标", avg_time);
    
    // 验证引擎状态
    let stats = engine.get_stats();
    assert_eq!(stats.total_tasks, concurrent_count);
    
    println!("✅ 并发处理性能测试通过");
}

#[tokio::test]
async fn test_memory_efficiency() {
    println!("🚀 测试内存使用效率");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册Agent
    let agent = AgentInfo {
        id: "memory_test_agent".to_string(),
        name: "内存测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["memory_test".to_string()],
        status: AgentStatus::Online,
    };
    engine.register_agent(agent);
    
    // 处理大量消息以测试内存使用
    let message_count = 10000;
    let start_time = Instant::now();
    
    for i in 0..message_count {
        // 创建包含大量数据的消息
        let large_data = serde_json::json!({
            "id": i,
            "data": "x".repeat(1000), // 1KB数据
            "metadata": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "source": "memory_test",
                "iteration": i
            }
        });
        
        let message = A2AMessage::new_data(MessageRole::User, large_data);
        let request = JsonRpcRequest::send_message(
            message,
            serde_json::Value::Number(serde_json::Number::from(i))
        );
        
        let _response = engine.process_request(request).await;
        
        // 每1000条消息检查一次性能
        if i % 1000 == 0 && i > 0 {
            let elapsed = start_time.elapsed();
            let avg_time = elapsed.as_millis() / i as u128;
            
            // 确保平均处理时间没有显著增长（内存泄漏指标）
            assert!(avg_time < 10, 
                   "处理 {} 条消息后平均时间 {}ms 超过10ms，可能存在内存问题", 
                   i, avg_time);
        }
    }
    
    let total_time = start_time.elapsed();
    let avg_time = total_time.as_millis() / message_count as u128;
    
    println!("📊 内存效率测试结果:");
    println!("  处理消息数: {}", message_count);
    println!("  总时间: {}ms", total_time.as_millis());
    println!("  平均时间: {}ms", avg_time);
    println!("  数据量: {}MB", (message_count * 1) / 1024); // 约1KB每条消息
    
    // 验证大量数据处理后性能仍然良好
    assert!(avg_time < 5, 
           "处理大量数据后平均时间 {}ms 超过5ms目标", avg_time);
    
    println!("✅ 内存使用效率测试通过");
}

#[tokio::test]
async fn test_error_handling_performance() {
    println!("🚀 测试错误处理性能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 测试各种错误情况的处理时间
    let error_cases = vec![
        ("invalid_method", create_invalid_method_request()),
        ("invalid_params", create_invalid_params_request()),
        ("missing_task", create_missing_task_request()),
    ];
    
    for (error_type, request) in error_cases {
        let start = Instant::now();
        let response = engine.process_request(request).await;
        let duration = start.elapsed();
        
        // 验证返回错误响应
        assert!(response.error.is_some());
        
        // 验证错误处理延迟 < 5ms
        let latency_ms = duration.as_millis();
        assert!(latency_ms < 5, 
               "{} 错误处理延迟 {}ms 超过5ms目标", error_type, latency_ms);
        
        println!("  {} 错误处理延迟: {}ms", error_type, latency_ms);
    }
    
    println!("✅ 错误处理性能测试通过");
}

// 辅助函数
fn create_submit_task_request() -> JsonRpcRequest {
    let task = A2ATask::new("test_task".to_string());
    JsonRpcRequest::submit_task(task, serde_json::Value::String("test_req".to_string()))
}

fn create_send_message_request() -> JsonRpcRequest {
    let message = A2AMessage::user_message("测试消息".to_string());
    JsonRpcRequest::send_message(message, serde_json::Value::String("msg_req".to_string()))
}

fn create_get_capabilities_request() -> JsonRpcRequest {
    JsonRpcRequest::new(
        "getCapabilities".to_string(),
        None,
        serde_json::Value::String("cap_req".to_string())
    )
}

fn create_invalid_method_request() -> JsonRpcRequest {
    JsonRpcRequest::new(
        "invalidMethod".to_string(),
        None,
        serde_json::Value::String("invalid_req".to_string())
    )
}

fn create_invalid_params_request() -> JsonRpcRequest {
    JsonRpcRequest::new(
        "submitTask".to_string(),
        Some(serde_json::json!({"invalid": "params"})),
        serde_json::Value::String("invalid_params_req".to_string())
    )
}

fn create_missing_task_request() -> JsonRpcRequest {
    JsonRpcRequest::get_task(
        "nonexistent_task_id".to_string(),
        serde_json::Value::String("missing_task_req".to_string())
    )
}
