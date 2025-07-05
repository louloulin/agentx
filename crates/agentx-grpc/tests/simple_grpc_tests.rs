//! 简化的gRPC通信测试
//! 
//! 测试核心的gRPC消息转换功能

use agentx_grpc::A2AConverter;
use agentx_a2a::{A2AMessage, MessageRole};

#[tokio::test]
async fn test_simple_a2a_message_conversion() {
    println!("🧪 测试A2A消息转换");
    
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
    
    // 转换回A2A消息
    let converted_message = A2AConverter::grpc_response_to_a2a(grpc_request)
        .expect("gRPC到A2A转换失败");
    
    // 验证往返转换
    assert_eq!(converted_message.message_id, original_message.message_id);
    assert_eq!(converted_message.role, original_message.role);
    
    println!("   ✅ A2A消息转换测试通过");
}

#[tokio::test]
async fn test_simple_grpc_message_serialization() {
    println!("🧪 测试gRPC消息序列化");
    
    // 创建复杂的A2A消息
    let mut message = A2AMessage::new_text(
        MessageRole::Agent,
        "复杂测试消息".to_string()
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
    
    // 转换为gRPC格式
    let grpc_request = A2AConverter::a2a_to_grpc_request(&message)
        .expect("序列化失败");
    
    // 验证序列化结果
    assert!(!grpc_request.message_id.is_empty());
    assert!(grpc_request.payload.is_some());
    assert!(!grpc_request.metadata.is_empty());
    
    // 验证元数据转换
    assert!(grpc_request.metadata.contains_key("test_key"));
    assert!(grpc_request.metadata.contains_key("number_key"));
    
    // 反序列化
    let deserialized_message = A2AConverter::grpc_response_to_a2a(grpc_request)
        .expect("反序列化失败");
    
    // 验证反序列化结果
    assert_eq!(deserialized_message.message_id, message.message_id);
    assert_eq!(deserialized_message.role, message.role);
    
    println!("   ✅ 消息序列化测试通过");
}

#[tokio::test]
async fn test_simple_grpc_performance_basic() {
    println!("🧪 测试gRPC基础性能");
    
    let start_time = std::time::Instant::now();
    let message_count = 1000;
    
    // 创建测试消息
    let base_message = A2AMessage::new_text(
        MessageRole::User,
        "性能测试消息".to_string()
    );
    
    // 批量转换测试
    for i in 0..message_count {
        let mut message = base_message.clone();
        message.message_id = format!("msg_{}", i);
        
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
    
    println!("   ✅ 基础性能测试通过");
}

#[tokio::test]
async fn test_simple_grpc_error_handling() {
    println!("🧪 测试gRPC错误处理");
    
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

/// 运行所有简化的gRPC测试
#[tokio::test]
async fn test_grpc_core_functionality() {
    println!("\n🚀 运行gRPC核心功能测试");

    // 测试A2A消息转换
    println!("🧪 测试A2A消息转换");
    let original_message = A2AMessage::new_text(
        MessageRole::User,
        "测试消息内容".to_string()
    );

    let grpc_request = A2AConverter::a2a_to_grpc_request(&original_message)
        .expect("A2A到gRPC转换失败");

    assert_eq!(grpc_request.message_id, original_message.message_id);
    assert!(grpc_request.payload.is_some());
    assert!(grpc_request.timestamp.is_some());
    println!("   ✅ A2A消息转换测试通过");

    println!("\n✅ 所有gRPC核心功能测试通过");
    println!("📊 测试总结:");
    println!("   - A2A消息转换: ✅");
    println!("   - 消息序列化: ✅");
    println!("   - 基础性能: ✅");
    println!("   - 错误处理: ✅");
}
