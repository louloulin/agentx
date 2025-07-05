//! HTTP API集成测试
//! 
//! 测试HTTP API的各个端点功能

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json;
use tower::ServiceExt;
use agentx_http::{
    config::AppConfig,
    server::HttpServer,
    models::*,
};

/// 创建测试服务器
async fn create_test_server() -> axum::Router {
    let config = AppConfig::default();
    let server = HttpServer::new(config);
    server.create_routes()
}

#[tokio::test]
async fn test_health_check() {
    println!("🧪 测试健康检查端点");
    
    let app = create_test_server().await;
    
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(health_response.status, "healthy");
    assert!(!health_response.version.is_empty());
    
    println!("✅ 健康检查测试通过");
}

#[tokio::test]
async fn test_readiness_check() {
    println!("🧪 测试就绪检查端点");
    
    let app = create_test_server().await;
    
    let request = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(health_response.status, "ready");
    
    println!("✅ 就绪检查测试通过");
}

#[tokio::test]
async fn test_liveness_check() {
    println!("🧪 测试存活检查端点");
    
    let app = create_test_server().await;
    
    let request = Request::builder()
        .uri("/live")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let health_response: HealthResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(health_response.status, "alive");
    
    println!("✅ 存活检查测试通过");
}

#[tokio::test]
async fn test_create_task() {
    println!("🧪 测试创建任务端点");
    
    let app = create_test_server().await;
    
    let create_request = CreateTaskRequest {
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
    
    let request = Request::builder()
        .uri("/api/v1/tasks")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&create_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let task_response: TaskResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(task_response.kind, "test_task");
    assert_eq!(task_response.context_id, Some("test_context".to_string()));
    assert_eq!(task_response.status.state, agentx_a2a::TaskState::Submitted);
    assert_eq!(task_response.history.len(), 1);
    
    println!("✅ 创建任务测试通过");
}

#[tokio::test]
async fn test_register_agent() {
    println!("🧪 测试注册Agent端点");
    
    let app = create_test_server().await;
    
    let register_request = RegisterAgentRequest {
        id: "test_agent".to_string(),
        name: "测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["test_capability".to_string()],
        status: agentx_a2a::AgentStatus::Online,
    };
    
    let request = Request::builder()
        .uri("/api/v1/agents")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&register_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let agent_response: AgentResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(agent_response.id, "test_agent");
    assert_eq!(agent_response.name, "测试Agent");
    assert_eq!(agent_response.capabilities, vec!["test_capability"]);
    
    println!("✅ 注册Agent测试通过");
}

#[tokio::test]
async fn test_send_message() {
    println!("🧪 测试发送消息端点");
    
    let app = create_test_server().await;
    
    let message_request = CreateMessageRequest {
        role: agentx_a2a::MessageRole::User,
        content: MessageContent::Text {
            text: "Hello, Agent!".to_string(),
        },
        task_id: Some("test_task_123".to_string()),
        context_id: Some("test_context".to_string()),
        metadata: std::collections::HashMap::new(),
    };
    
    let request = Request::builder()
        .uri("/api/v1/messages")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&message_request).unwrap()))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let message_response: MessageResponse = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(message_response.role, agentx_a2a::MessageRole::User);
    assert_eq!(message_response.task_id, Some("test_task_123".to_string()));
    assert_eq!(message_response.context_id, Some("test_context".to_string()));
    assert_eq!(message_response.parts.len(), 1);
    
    if let MessagePartResponse::Text { text, .. } = &message_response.parts[0] {
        assert_eq!(text, "Hello, Agent!");
    } else {
        panic!("期望文本消息部分");
    }
    
    println!("✅ 发送消息测试通过");
}

#[tokio::test]
async fn test_get_capabilities() {
    println!("🧪 测试获取能力端点");
    
    let app = create_test_server().await;
    
    let request = Request::builder()
        .uri("/api/v1/agents/capabilities")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let capabilities: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // 验证返回的是有效的JSON
    assert!(capabilities.is_object());
    
    println!("✅ 获取能力测试通过");
}

#[tokio::test]
async fn test_invalid_request() {
    println!("🧪 测试无效请求处理");
    
    let app = create_test_server().await;
    
    // 发送无效的JSON
    let request = Request::builder()
        .uri("/api/v1/tasks")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    // 应该返回400错误
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    println!("✅ 无效请求处理测试通过");
}

#[tokio::test]
async fn test_not_found() {
    println!("🧪 测试404错误处理");
    
    let app = create_test_server().await;
    
    let request = Request::builder()
        .uri("/api/v1/nonexistent")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    println!("✅ 404错误处理测试通过");
}
