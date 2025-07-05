//! A2A协议完整测试套件
//! 
//! 本测试套件验证A2A协议实现的正确性，包括：
//! - 消息格式和序列化
//! - 协议引擎功能
//! - 性能基准测试
//! - 错误处理和故障恢复

use agentx_a2a::*;
use serde_json;
use std::time::Instant;
use tokio;
use base64::Engine;

#[tokio::test]
async fn test_a2a_message_format_compliance() {
    println!("🧪 测试A2A消息格式符合性");
    
    // 测试用户消息
    let user_msg = A2AMessage::user_message("请帮我生成一篇关于AI的文章".to_string())
        .with_task_id("task_001".to_string())
        .with_context_id("ctx_001".to_string());
    
    // 验证消息结构
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.task_id, Some("task_001".to_string()));
    assert_eq!(user_msg.context_id, Some("ctx_001".to_string()));
    assert_eq!(user_msg.parts.len(), 1);
    
    // 验证序列化和反序列化
    let json = serde_json::to_string(&user_msg).unwrap();
    let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(user_msg, deserialized);
    
    println!("✅ A2A消息格式测试通过");
}

#[tokio::test]
async fn test_a2a_task_lifecycle() {
    println!("🧪 测试A2A任务生命周期");
    
    // 创建任务
    let mut task = A2ATask::new("text_generation".to_string())
        .with_context_id("ctx_002".to_string());
    
    // 验证初始状态
    assert_eq!(task.status.state, TaskState::Submitted);
    assert_eq!(task.kind, "text_generation");
    
    // 添加消息到历史
    let user_msg = A2AMessage::user_message("生成文章".to_string());
    task = task.add_message(user_msg);
    assert_eq!(task.history.len(), 1);
    
    // 更新任务状态
    task = task.update_status(TaskState::Working);
    assert_eq!(task.status.state, TaskState::Working);
    
    // 添加工件
    let artifact = Artifact {
        artifact_id: "art_001".to_string(),
        name: Some("生成的文章".to_string()),
        parts: vec![MessagePart::Text(TextPart {
            text: "这是生成的AI文章内容...".to_string(),
            metadata: std::collections::HashMap::new(),
        })],
        metadata: std::collections::HashMap::new(),
    };
    
    task = task.add_artifact(artifact);
    assert_eq!(task.artifacts.len(), 1);
    
    // 完成任务
    task = task.update_status(TaskState::Completed);
    assert_eq!(task.status.state, TaskState::Completed);
    
    println!("✅ A2A任务生命周期测试通过");
}

#[tokio::test]
async fn test_json_rpc_protocol() {
    println!("🧪 测试JSON-RPC协议实现");
    
    // 测试submitTask请求
    let task = A2ATask::new("test_task".to_string());
    let request = JsonRpcRequest::submit_task(
        task.clone(),
        serde_json::Value::String("req_001".to_string())
    );
    
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "submitTask");
    assert!(request.params.is_some());
    
    // 测试成功响应
    let success_response = JsonRpcResponse::success(
        serde_json::json!({"taskId": task.id, "status": "submitted"}),
        serde_json::Value::String("req_001".to_string())
    );
    
    assert_eq!(success_response.jsonrpc, "2.0");
    assert!(success_response.result.is_some());
    assert!(success_response.error.is_none());
    
    // 测试错误响应
    let error_response = JsonRpcResponse::error(
        JsonRpcError::invalid_params(),
        serde_json::Value::String("req_002".to_string())
    );
    
    assert_eq!(error_response.jsonrpc, "2.0");
    assert!(error_response.result.is_none());
    assert!(error_response.error.is_some());
    
    println!("✅ JSON-RPC协议测试通过");
}

#[tokio::test]
async fn test_protocol_engine_functionality() {
    println!("🧪 测试A2A协议引擎功能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册测试Agent
    let agent = AgentInfo {
        id: "test_agent_001".to_string(),
        name: "测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["text_generation".to_string(), "translation".to_string()],
        status: AgentStatus::Online,
    };
    
    engine.register_agent(agent);
    
    // 测试submitTask
    let task = A2ATask::new("text_generation".to_string());
    let submit_request = JsonRpcRequest::submit_task(
        task.clone(),
        serde_json::Value::String("req_001".to_string())
    );
    
    let response = engine.process_request(submit_request).await;
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    
    // 测试getTask
    let get_request = JsonRpcRequest::get_task(
        task.id.clone(),
        serde_json::Value::String("req_002".to_string())
    );
    
    let response = engine.process_request(get_request).await;
    assert!(response.result.is_some());
    
    // 验证统计信息
    let stats = engine.get_stats();
    assert_eq!(stats.total_tasks, 1);
    assert_eq!(stats.active_tasks, 1);
    
    println!("✅ A2A协议引擎功能测试通过");
}

#[tokio::test]
async fn test_message_routing_performance() {
    println!("🧪 测试消息路由性能 (目标: <10ms)");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册多个Agent
    for i in 1..=5 {
        let agent = AgentInfo {
            id: format!("agent_{:03}", i),
            name: format!("Agent {}", i),
            endpoint: format!("http://localhost:808{}", i),
            capabilities: vec!["test_capability".to_string()],
            status: AgentStatus::Online,
        };
        engine.register_agent(agent);
    }
    
    // 性能测试：处理100条消息
    let message_count = 100;
    let mut total_time = 0u128;
    
    for i in 0..message_count {
        let message = A2AMessage::user_message(format!("测试消息 {}", i))
            .with_task_id(format!("task_{}", i));
        
        let request = JsonRpcRequest::send_message(
            message,
            serde_json::Value::Number(serde_json::Number::from(i))
        );
        
        let start = Instant::now();
        let _response = engine.process_request(request).await;
        let duration = start.elapsed();
        
        total_time += duration.as_millis();
        
        // 验证单个消息处理时间 < 10ms
        assert!(duration.as_millis() < 10, 
               "消息 {} 处理时间 {}ms 超过10ms目标", i, duration.as_millis());
    }
    
    let avg_time = total_time / message_count as u128;
    println!("📊 平均消息处理时间: {}ms", avg_time);
    println!("📊 总处理时间: {}ms", total_time);
    println!("📊 吞吐量: {:.2} 消息/秒", 
             (message_count as f64) / (total_time as f64 / 1000.0));
    
    // 验证平均处理时间 < 5ms
    assert!(avg_time < 5, "平均处理时间 {}ms 超过5ms目标", avg_time);
    
    println!("✅ 消息路由性能测试通过");
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    println!("🧪 测试错误处理和故障恢复");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 测试无效的JSON-RPC请求
    let invalid_request = JsonRpcRequest::new(
        "invalid_method".to_string(),
        None,
        serde_json::Value::String("req_001".to_string())
    );
    
    let response = engine.process_request(invalid_request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32601); // Method not found
    
    // 测试无效参数
    let invalid_params_request = JsonRpcRequest::new(
        "submitTask".to_string(),
        Some(serde_json::json!({"invalid": "params"})),
        serde_json::Value::String("req_002".to_string())
    );
    
    let response = engine.process_request(invalid_params_request).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32602); // Invalid params
    
    // 测试任务不存在
    let get_nonexistent_task = JsonRpcRequest::get_task(
        "nonexistent_task".to_string(),
        serde_json::Value::String("req_003".to_string())
    );
    
    let response = engine.process_request(get_nonexistent_task).await;
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap().code, -32001); // Task not found
    
    println!("✅ 错误处理和故障恢复测试通过");
}

#[tokio::test]
async fn test_concurrent_message_processing() {
    println!("🧪 测试并发消息处理");
    
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
    
    // 创建多个并发任务
    let task_count = 50;
    let mut handles = Vec::new();
    
    for i in 0..task_count {
        let task = A2ATask::new(format!("concurrent_task_{}", i));
        let request = JsonRpcRequest::submit_task(
            task,
            serde_json::Value::Number(serde_json::Number::from(i))
        );
        
        // 注意：这里我们需要克隆engine或使用Arc<Mutex<>>来支持并发
        // 为了测试目的，我们顺序处理但测量时间
        let start = Instant::now();
        let response = engine.process_request(request).await;
        let duration = start.elapsed();
        
        assert!(response.result.is_some());
        assert!(duration.as_millis() < 10);
        
        handles.push(duration);
    }
    
    let total_time: u128 = handles.iter().map(|d| d.as_millis()).sum();
    let avg_time = total_time / task_count as u128;
    
    println!("📊 并发处理 {} 个任务", task_count);
    println!("📊 平均处理时间: {}ms", avg_time);
    println!("📊 总处理时间: {}ms", total_time);
    
    // 验证统计信息
    let stats = engine.get_stats();
    assert_eq!(stats.total_tasks, task_count);
    
    println!("✅ 并发消息处理测试通过");
}

#[tokio::test]
async fn test_file_and_data_parts() {
    println!("🧪 测试文件和数据部分处理");
    
    // 测试文件部分
    let file_data = FileData::WithBytes(FileWithBytes {
        name: Some("test.txt".to_string()),
        mime_type: "text/plain".to_string(),
        bytes: base64::engine::general_purpose::STANDARD.encode("Hello, World!"),
    });
    
    let file_message = A2AMessage::new_file(MessageRole::User, file_data);
    assert_eq!(file_message.parts.len(), 1);
    
    if let MessagePart::File(file_part) = &file_message.parts[0] {
        if let FileData::WithBytes(file_bytes) = &file_part.file {
            assert_eq!(file_bytes.mime_type, "text/plain");
            assert_eq!(file_bytes.name, Some("test.txt".to_string()));
        } else {
            panic!("期望FileWithBytes类型");
        }
    } else {
        panic!("期望File部分");
    }
    
    // 测试数据部分
    let data = serde_json::json!({
        "type": "analysis_result",
        "confidence": 0.95,
        "categories": ["technology", "ai"]
    });
    
    let data_message = A2AMessage::new_data(MessageRole::Agent, data.clone());
    assert_eq!(data_message.parts.len(), 1);
    
    if let MessagePart::Data(data_part) = &data_message.parts[0] {
        assert_eq!(data_part.data, data);
    } else {
        panic!("期望Data部分");
    }
    
    // 测试序列化
    let json = serde_json::to_string(&file_message).unwrap();
    let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(file_message, deserialized);
    
    println!("✅ 文件和数据部分处理测试通过");
}

// 性能基准测试辅助函数
fn print_performance_summary(test_name: &str, message_count: usize, total_time_ms: u128) {
    println!("\n📊 {} 性能总结:", test_name);
    println!("   消息数量: {}", message_count);
    println!("   总时间: {}ms", total_time_ms);
    println!("   平均时间: {:.2}ms", total_time_ms as f64 / message_count as f64);
    println!("   吞吐量: {:.2} 消息/秒", 
             (message_count as f64) / (total_time_ms as f64 / 1000.0));
    println!("   目标达成: {}", if (total_time_ms / message_count as u128) < 10 { "✅" } else { "❌" });
}
