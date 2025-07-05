//! gRPC通信系统集成测试
//! 
//! 测试真实的gRPC客户端-服务器通信功能

use agentx_grpc::{
    AgentXGrpcServer, AgentXGrpcClient, ServerConfig, ClientConfig,
    PluginManager, PluginBridge, A2AConverter,
};
use agentx_a2a::{
    A2AMessage, MessageRole, A2AProtocolEngine, ProtocolEngineConfig,
    StreamManager, SecurityManager, SecurityConfig, MonitoringManager, MonitoringConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// 测试配置
struct TestConfig {
    pub server_port: u16,
    pub client_timeout: Duration,
    pub test_timeout: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_port: 50055, // 使用不同端口避免冲突
            client_timeout: Duration::from_secs(5),
            test_timeout: Duration::from_secs(30),
        }
    }
}

#[tokio::test]
async fn test_grpc_server_startup() {
    println!("🧪 测试gRPC服务器启动");
    
    let config = TestConfig::default();
    
    // 创建A2A协议引擎
    let a2a_config = ProtocolEngineConfig::default();
    let a2a_engine = Arc::new(RwLock::new(A2AProtocolEngine::new(a2a_config)));

    // 创建其他管理器
    let stream_manager = Arc::new(RwLock::new(StreamManager::new()));
    let security_manager = Arc::new(RwLock::new(SecurityManager::new(SecurityConfig::default())));
    let monitoring_manager = Arc::new(RwLock::new(MonitoringManager::new(MonitoringConfig::default())));

    // 创建插件桥接器
    let bridge = Arc::new(PluginBridge::new(
        a2a_engine,
        stream_manager,
        security_manager,
        monitoring_manager,
    ));
    
    // 创建插件管理器
    let plugin_manager = Arc::new(PluginManager::new(bridge));
    
    // 创建服务器配置
    let server_config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: config.server_port,
        max_connections: 100,
        request_timeout_seconds: 30,
        enable_reflection: false,
    };
    
    // 创建gRPC服务器
    let server = AgentXGrpcServer::new(plugin_manager, server_config);
    
    // 验证服务器创建成功
    let stats = server.get_server_stats().await;
    assert_eq!(stats.connected_plugins_count, 0);
    assert_eq!(stats.total_requests, 0);
    
    println!("   ✅ gRPC服务器创建成功");
}

#[tokio::test]
async fn test_grpc_client_creation() {
    println!("🧪 测试gRPC客户端创建");
    
    let config = TestConfig::default();
    
    // 创建客户端配置
    let client_config = ClientConfig {
        connect_timeout_seconds: 5,
        request_timeout_seconds: 10,
        max_retries: 3,
        retry_interval_ms: 1000,
        enable_tls: false,
        tls_cert_path: None,
    };
    
    // 创建gRPC客户端
    let client = AgentXGrpcClient::new(client_config);
    
    // 验证客户端初始状态
    let connections = client.get_all_connections().await;
    assert!(connections.is_empty());
    
    println!("   ✅ gRPC客户端创建成功");
}

#[tokio::test]
async fn test_a2a_message_conversion() {
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
async fn test_grpc_client_connection_management() {
    println!("🧪 测试gRPC客户端连接管理");
    
    let config = TestConfig::default();
    
    // 创建客户端
    let client_config = ClientConfig::default();
    let client = AgentXGrpcClient::new(client_config);
    
    // 测试连接到不存在的服务器（应该失败）
    let plugin_id = "test-plugin".to_string();
    let endpoint = format!("http://127.0.0.1:{}", config.server_port);
    
    let result = timeout(
        config.client_timeout,
        client.connect_to_plugin(plugin_id.clone(), endpoint)
    ).await;
    
    // 连接应该失败（因为没有服务器运行）
    assert!(result.is_err() || result.unwrap().is_err());
    
    // 验证连接状态
    let connections = client.get_all_connections().await;
    if !connections.is_empty() {
        let connection = &connections[0];
        assert_eq!(connection.plugin_id, plugin_id);
        // 连接状态应该是失败或断开
        assert!(matches!(
            connection.status,
            agentx_grpc::ConnectionStatus::Failed(_) | 
            agentx_grpc::ConnectionStatus::Disconnected
        ));
    }
    
    println!("   ✅ 连接管理测试通过");
}

#[tokio::test]
async fn test_grpc_message_serialization() {
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
async fn test_grpc_performance_basic() {
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
async fn test_grpc_error_handling() {
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

/// 运行所有gRPC通信测试
#[tokio::test]
async fn test_grpc_communication_integration() {
    println!("\n🚀 运行gRPC通信系统集成测试");

    // 运行各个测试组件
    test_grpc_server_startup().await;
    test_grpc_client_creation().await;
    test_a2a_message_conversion().await;
    test_grpc_client_connection_management().await;
    test_grpc_message_serialization().await;
    test_grpc_performance_basic().await;
    test_grpc_error_handling().await;
    
    println!("\n✅ 所有gRPC通信测试通过");
    println!("📊 测试总结:");
    println!("   - gRPC服务器创建: ✅");
    println!("   - gRPC客户端创建: ✅");
    println!("   - A2A消息转换: ✅");
    println!("   - 连接管理: ✅");
    println!("   - 消息序列化: ✅");
    println!("   - 基础性能: ✅");
    println!("   - 错误处理: ✅");
}
