//! A2A消息转换器测试
//! 
//! 测试A2A协议与gRPC protobuf格式之间的转换功能

use agentx_grpc::A2AConverter;
use agentx_a2a::{A2AMessage, MessageRole};

#[test]
fn test_a2a_to_grpc_conversion() {
    println!("🧪 测试A2A到gRPC转换");
    
    // 创建测试消息
    let original_message = A2AMessage::new_text(
        MessageRole::User,
        "测试消息内容".to_string()
    );
    
    // 转换为gRPC请求
    let grpc_request = A2AConverter::a2a_to_grpc_request(&original_message)
        .expect("A2A到gRPC转换失败");
    
    // 验证转换结果
    assert_eq!(grpc_request.message_id, original_message.message_id);
    assert!(grpc_request.payload.is_some());
    assert!(grpc_request.timestamp.is_some());
    
    println!("   ✅ A2A到gRPC转换测试通过");
}

#[test]
fn test_grpc_to_a2a_conversion() {
    println!("🧪 测试gRPC到A2A转换");
    
    // 创建gRPC请求
    let grpc_request = agentx_grpc::proto::A2aMessageRequest {
        message_id: "test_msg_123".to_string(),
        from_agent: "test_agent".to_string(),
        to_agent: "target_agent".to_string(),
        message_type: 1, // REQUEST
        payload: None,
        metadata: std::collections::HashMap::new(),
        timestamp: Some(prost_types::Timestamp {
            seconds: chrono::Utc::now().timestamp(),
            nanos: 0,
        }),
        ttl_seconds: 300,
    };
    
    // 转换为A2A消息
    let a2a_message = A2AConverter::grpc_response_to_a2a(grpc_request)
        .expect("gRPC到A2A转换失败");
    
    // 验证转换结果
    assert_eq!(a2a_message.message_id, "test_msg_123");
    assert_eq!(a2a_message.role, MessageRole::User);
    
    println!("   ✅ gRPC到A2A转换测试通过");
}

#[test]
fn test_round_trip_conversion() {
    println!("🧪 测试往返转换");
    
    // 创建原始消息
    let original_message = A2AMessage::new_text(
        MessageRole::Agent,
        "往返转换测试消息".to_string()
    );
    
    // 转换为gRPC
    let grpc_request = A2AConverter::a2a_to_grpc_request(&original_message)
        .expect("A2A到gRPC转换失败");
    
    // 转换回A2A
    let converted_message = A2AConverter::grpc_response_to_a2a(grpc_request)
        .expect("gRPC到A2A转换失败");
    
    // 验证往返转换
    assert_eq!(converted_message.message_id, original_message.message_id);
    assert_eq!(converted_message.role, original_message.role);
    
    println!("   ✅ 往返转换测试通过");
}

#[test]
fn test_message_with_metadata() {
    println!("🧪 测试带元数据的消息转换");
    
    // 创建带元数据的消息
    let mut message = A2AMessage::new_text(
        MessageRole::User,
        "带元数据的测试消息".to_string()
    );
    
    // 添加元数据
    message.metadata.insert(
        "test_key".to_string(),
        serde_json::Value::String("test_value".to_string())
    );
    message.metadata.insert(
        "number_key".to_string(),
        serde_json::Value::Number(serde_json::Number::from(42))
    );
    
    // 转换为gRPC
    let grpc_request = A2AConverter::a2a_to_grpc_request(&message)
        .expect("转换失败");
    
    // 验证元数据转换
    assert!(grpc_request.metadata.contains_key("test_key"));
    assert!(grpc_request.metadata.contains_key("number_key"));
    // 注意：JSON值被序列化为字符串，所以会包含引号
    assert_eq!(grpc_request.metadata.get("test_key").unwrap(), "\"test_value\"");
    assert_eq!(grpc_request.metadata.get("number_key").unwrap(), "42");
    
    println!("   ✅ 元数据转换测试通过");
}

#[test]
fn test_performance_conversion() {
    println!("🧪 测试转换性能");
    
    let start_time = std::time::Instant::now();
    let message_count = 1000;
    
    // 创建基础消息
    let base_message = A2AMessage::new_text(
        MessageRole::User,
        "性能测试消息".to_string()
    );
    
    // 批量转换测试
    for i in 0..message_count {
        let mut message = base_message.clone();
        message.message_id = format!("perf_msg_{}", i);
        
        // 转换为gRPC
        let grpc_request = A2AConverter::a2a_to_grpc_request(&message)
            .expect("转换失败");
        
        // 转换回A2A
        let _converted = A2AConverter::grpc_response_to_a2a(grpc_request)
            .expect("反转换失败");
    }
    
    let elapsed = start_time.elapsed();
    let messages_per_second = message_count as f64 / elapsed.as_secs_f64();
    
    println!("   📊 转换性能: {:.0} 消息/秒", messages_per_second);
    println!("   📊 平均延迟: {:.2} ms", elapsed.as_millis() as f64 / message_count as f64);
    
    // 性能要求：至少1000消息/秒
    assert!(messages_per_second > 1000.0, "转换性能不足: {:.0} msg/s", messages_per_second);
    
    println!("   ✅ 性能测试通过");
}

#[test]
fn test_error_handling() {
    println!("🧪 测试错误处理");
    
    // 测试无效消息转换
    let invalid_grpc_request = agentx_grpc::proto::A2aMessageRequest {
        message_id: "".to_string(), // 空ID
        from_agent: "test".to_string(),
        to_agent: "test".to_string(),
        message_type: 999, // 无效类型
        payload: None,
        metadata: std::collections::HashMap::new(),
        timestamp: None,
        ttl_seconds: 0,
    };
    
    // 转换应该成功但使用默认值
    let result = A2AConverter::grpc_response_to_a2a(invalid_grpc_request);
    assert!(result.is_ok());
    
    let converted = result.unwrap();
    assert!(converted.message_id.is_empty());
    assert_eq!(converted.role, MessageRole::Agent); // 默认角色
    
    println!("   ✅ 错误处理测试通过");
}

#[test]
fn test_all_converter_functionality() {
    println!("\n🚀 运行所有转换器功能测试");
    
    test_a2a_to_grpc_conversion();
    test_grpc_to_a2a_conversion();
    test_round_trip_conversion();
    test_message_with_metadata();
    test_performance_conversion();
    test_error_handling();
    
    println!("\n✅ 所有转换器测试通过");
    println!("📊 测试总结:");
    println!("   - A2A到gRPC转换: ✅");
    println!("   - gRPC到A2A转换: ✅");
    println!("   - 往返转换: ✅");
    println!("   - 元数据处理: ✅");
    println!("   - 性能测试: ✅");
    println!("   - 错误处理: ✅");
}
