//! 简化的HTTP API测试
//! 
//! 测试HTTP API的基本功能，避免复杂的依赖问题

use agentx_http::{
    config::AppConfig,
    models::*,
    error::{HttpApiError, ErrorResponse},
};
use serde_json;

#[test]
fn test_app_config_creation() {
    println!("🧪 测试应用配置创建");
    
    let config = AppConfig::default();
    
    assert_eq!(config.http.port, 8080);
    assert_eq!(config.http.host, "0.0.0.0");
    assert!(config.http.enable_cors);
    assert!(config.http.enable_docs);
    assert_eq!(config.a2a.max_concurrent_tasks, 1000);
    
    println!("✅ 应用配置创建测试通过");
}

#[test]
fn test_config_validation() {
    println!("🧪 测试配置验证");
    
    let mut config = AppConfig::default();
    
    // 有效配置应该通过验证
    assert!(config.validate().is_ok());
    
    // 无效端口应该失败
    config.http.port = 0;
    assert!(config.validate().is_err());
    
    println!("✅ 配置验证测试通过");
}

#[test]
fn test_create_task_request_serialization() {
    println!("🧪 测试创建任务请求序列化");
    
    let request = CreateTaskRequest {
        kind: "test_task".to_string(),
        context_id: Some("test_context".to_string()),
        initial_message: Some(CreateMessageRequest {
            role: agentx_a2a::MessageRole::User,
            content: MessageContent::Text {
                text: "测试消息".to_string(),
            },
            task_id: None,
            context_id: None,
            metadata: std::collections::HashMap::new(),
        }),
        metadata: std::collections::HashMap::new(),
    };
    
    // 序列化
    let json = serde_json::to_string(&request).unwrap();
    println!("序列化JSON: {}", json);
    
    // 反序列化
    let deserialized: CreateTaskRequest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(request.kind, deserialized.kind);
    assert_eq!(request.context_id, deserialized.context_id);
    
    println!("✅ 创建任务请求序列化测试通过");
}

#[test]
fn test_register_agent_request_serialization() {
    println!("🧪 测试注册Agent请求序列化");
    
    let request = RegisterAgentRequest {
        id: "test_agent".to_string(),
        name: "测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["test_capability".to_string()],
        status: agentx_a2a::AgentStatus::Online,
    };
    
    // 序列化
    let json = serde_json::to_string(&request).unwrap();
    println!("序列化JSON: {}", json);
    
    // 反序列化
    let deserialized: RegisterAgentRequest = serde_json::from_str(&json).unwrap();
    
    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.name, deserialized.name);
    assert_eq!(request.capabilities, deserialized.capabilities);
    
    println!("✅ 注册Agent请求序列化测试通过");
}

#[test]
fn test_message_content_types() {
    println!("🧪 测试消息内容类型");
    
    // 文本消息
    let text_content = MessageContent::Text {
        text: "Hello, World!".to_string(),
    };
    
    let json = serde_json::to_string(&text_content).unwrap();
    let deserialized: MessageContent = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        MessageContent::Text { text } => assert_eq!(text, "Hello, World!"),
        _ => panic!("期望文本消息类型"),
    }
    
    // 文件消息
    let file_content = MessageContent::File {
        name: Some("test.txt".to_string()),
        mime_type: "text/plain".to_string(),
        data: "SGVsbG8sIFdvcmxkIQ==".to_string(), // "Hello, World!" in base64
    };
    
    let json = serde_json::to_string(&file_content).unwrap();
    let deserialized: MessageContent = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        MessageContent::File { name, mime_type, .. } => {
            assert_eq!(name, Some("test.txt".to_string()));
            assert_eq!(mime_type, "text/plain");
        },
        _ => panic!("期望文件消息类型"),
    }
    
    // 数据消息
    let data_content = MessageContent::Data {
        data: serde_json::json!({
            "type": "analysis",
            "result": "positive"
        }),
    };
    
    let json = serde_json::to_string(&data_content).unwrap();
    let deserialized: MessageContent = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        MessageContent::Data { data } => {
            assert_eq!(data["type"], "analysis");
            assert_eq!(data["result"], "positive");
        },
        _ => panic!("期望数据消息类型"),
    }
    
    println!("✅ 消息内容类型测试通过");
}

#[test]
fn test_pagination_query() {
    println!("🧪 测试分页查询参数");
    
    let pagination = PaginationQuery {
        page: 1,
        page_size: 20,
    };
    
    // 序列化
    let json = serde_json::to_string(&pagination).unwrap();
    let deserialized: PaginationQuery = serde_json::from_str(&json).unwrap();
    
    assert_eq!(pagination.page, deserialized.page);
    assert_eq!(pagination.page_size, deserialized.page_size);
    
    println!("✅ 分页查询参数测试通过");
}

#[test]
fn test_health_response() {
    println!("🧪 测试健康检查响应");
    
    let health = HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now(),
        details: Some(serde_json::json!({
            "uptime": "1h 30m",
            "memory": "256MB"
        })),
    };
    
    // 序列化
    let json = serde_json::to_string(&health).unwrap();
    let deserialized: HealthResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(health.status, deserialized.status);
    assert_eq!(health.version, deserialized.version);
    assert!(deserialized.details.is_some());
    
    println!("✅ 健康检查响应测试通过");
}

#[test]
fn test_error_response() {
    println!("🧪 测试错误响应");
    
    let error = ErrorResponse::new(
        "VALIDATION_ERROR".to_string(),
        "参数验证失败".to_string(),
    ).with_details(serde_json::json!({
        "field": "name",
        "reason": "required"
    }));

    // 序列化
    let json = serde_json::to_string(&error).unwrap();
    let deserialized: ErrorResponse = serde_json::from_str(&json).unwrap();
    
    assert_eq!(error.code, deserialized.code);
    assert_eq!(error.message, deserialized.message);
    assert!(deserialized.details.is_some());
    
    println!("✅ 错误响应测试通过");
}

#[test]
fn test_http_api_error_types() {
    println!("🧪 测试HTTP API错误类型");
    
    let validation_error = HttpApiError::ValidationError("参数无效".to_string());
    assert!(validation_error.to_string().contains("请求验证失败"));

    let auth_error = HttpApiError::AuthenticationError("认证失败".to_string());
    assert!(auth_error.to_string().contains("认证失败"));

    let not_found_error = HttpApiError::NotFound("资源不存在".to_string());
    assert!(not_found_error.to_string().contains("资源未找到"));
    
    println!("✅ HTTP API错误类型测试通过");
}

#[test]
fn test_pagination_info() {
    println!("🧪 测试分页信息");
    
    let pagination_info = PaginationInfo {
        page: 2,
        page_size: 10,
        total: 25,
        total_pages: 3,
        has_next: true,
        has_prev: true,
    };
    
    // 验证分页逻辑
    assert_eq!(pagination_info.total_pages, 3);
    assert!(pagination_info.has_next);
    assert!(pagination_info.has_prev);
    
    // 序列化测试
    let json = serde_json::to_string(&pagination_info).unwrap();
    let deserialized: PaginationInfo = serde_json::from_str(&json).unwrap();
    
    assert_eq!(pagination_info.page, deserialized.page);
    assert_eq!(pagination_info.total, deserialized.total);
    
    println!("✅ 分页信息测试通过");
}
