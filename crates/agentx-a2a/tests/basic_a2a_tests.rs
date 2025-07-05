//! 基础A2A协议测试
//! 
//! 这些测试验证A2A协议的基本功能，不依赖复杂的Actor系统

use agentx_a2a::*;
use serde_json;

#[test]
fn test_a2a_message_creation() {
    println!("🧪 测试A2A消息创建");
    
    let user_msg = A2AMessage::user_message("请帮我生成一篇文章".to_string());
    
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.parts.len(), 1);
    assert!(!user_msg.message_id.is_empty());
    
    if let MessagePart::Text(text_part) = &user_msg.parts[0] {
        assert_eq!(text_part.text, "请帮我生成一篇文章");
    } else {
        panic!("期望文本部分");
    }
    
    println!("✅ A2A消息创建测试通过");
}

#[test]
fn test_a2a_message_serialization() {
    println!("🧪 测试A2A消息序列化");
    
    let message = A2AMessage::user_message("测试消息".to_string())
        .with_task_id("task_001".to_string())
        .with_context_id("ctx_001".to_string());
    
    // 序列化
    let json = serde_json::to_string(&message).unwrap();
    println!("序列化JSON: {}", json);
    
    // 反序列化
    let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
    
    // 验证
    assert_eq!(message, deserialized);
    assert_eq!(deserialized.task_id, Some("task_001".to_string()));
    assert_eq!(deserialized.context_id, Some("ctx_001".to_string()));
    
    println!("✅ A2A消息序列化测试通过");
}

#[test]
fn test_a2a_task_creation() {
    println!("🧪 测试A2A任务创建");
    
    let task = A2ATask::new("text_generation".to_string())
        .with_context_id("ctx_002".to_string());
    
    assert_eq!(task.kind, "text_generation");
    assert_eq!(task.context_id, Some("ctx_002".to_string()));
    assert_eq!(task.status.state, TaskState::Submitted);
    assert!(!task.id.is_empty());
    
    println!("✅ A2A任务创建测试通过");
}

#[test]
fn test_json_rpc_request_creation() {
    println!("🧪 测试JSON-RPC请求创建");
    
    let task = A2ATask::new("test_task".to_string());
    let request = JsonRpcRequest::submit_task(
        task.clone(),
        serde_json::Value::String("req_001".to_string())
    );
    
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "submitTask");
    assert!(request.params.is_some());
    assert_eq!(request.id, serde_json::Value::String("req_001".to_string()));
    
    // 测试序列化
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, deserialized);
    
    println!("✅ JSON-RPC请求创建测试通过");
}

#[test]
fn test_json_rpc_response_creation() {
    println!("🧪 测试JSON-RPC响应创建");
    
    // 成功响应
    let success_response = JsonRpcResponse::success(
        serde_json::json!({"status": "ok", "taskId": "task_123"}),
        serde_json::Value::String("req_001".to_string())
    );
    
    assert_eq!(success_response.jsonrpc, "2.0");
    assert!(success_response.result.is_some());
    assert!(success_response.error.is_none());
    
    // 错误响应
    let error_response = JsonRpcResponse::error(
        JsonRpcError::invalid_params(),
        serde_json::Value::String("req_002".to_string())
    );
    
    assert_eq!(error_response.jsonrpc, "2.0");
    assert!(error_response.result.is_none());
    assert!(error_response.error.is_some());
    
    let error = error_response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert_eq!(error.message, "Invalid params");
    
    println!("✅ JSON-RPC响应创建测试通过");
}

#[test]
fn test_file_message_creation() {
    println!("🧪 测试文件消息创建");
    
    let file_data = FileData::WithBytes(FileWithBytes {
        name: Some("test.txt".to_string()),
        mime_type: "text/plain".to_string(),
        bytes: base64::encode("Hello, World!"),
    });
    
    let file_message = A2AMessage::new_file(MessageRole::User, file_data);
    
    assert_eq!(file_message.role, MessageRole::User);
    assert_eq!(file_message.parts.len(), 1);
    
    if let MessagePart::File(file_part) = &file_message.parts[0] {
        if let FileData::WithBytes(file_bytes) = &file_part.file {
            assert_eq!(file_bytes.mime_type, "text/plain");
            assert_eq!(file_bytes.name, Some("test.txt".to_string()));
            
            // 验证base64解码
            let decoded = base64::decode(&file_bytes.bytes).unwrap();
            assert_eq!(String::from_utf8(decoded).unwrap(), "Hello, World!");
        } else {
            panic!("期望FileWithBytes类型");
        }
    } else {
        panic!("期望File部分");
    }
    
    println!("✅ 文件消息创建测试通过");
}

#[test]
fn test_data_message_creation() {
    println!("🧪 测试数据消息创建");
    
    let data = serde_json::json!({
        "type": "analysis_result",
        "confidence": 0.95,
        "categories": ["technology", "ai"],
        "metadata": {
            "model": "gpt-4",
            "timestamp": "2024-01-01T00:00:00Z"
        }
    });
    
    let data_message = A2AMessage::new_data(MessageRole::Agent, data.clone());
    
    assert_eq!(data_message.role, MessageRole::Agent);
    assert_eq!(data_message.parts.len(), 1);
    
    if let MessagePart::Data(data_part) = &data_message.parts[0] {
        assert_eq!(data_part.data, data);
        
        // 验证可以访问嵌套数据
        assert_eq!(data_part.data["type"], "analysis_result");
        assert_eq!(data_part.data["confidence"], 0.95);
        assert_eq!(data_part.data["categories"][0], "technology");
    } else {
        panic!("期望Data部分");
    }
    
    println!("✅ 数据消息创建测试通过");
}

#[test]
fn test_task_lifecycle() {
    println!("🧪 测试任务生命周期");
    
    // 创建任务
    let mut task = A2ATask::new("text_generation".to_string());
    assert_eq!(task.status.state, TaskState::Submitted);
    
    // 添加用户消息
    let user_msg = A2AMessage::user_message("生成一篇关于AI的文章".to_string());
    task = task.add_message(user_msg);
    assert_eq!(task.history.len(), 1);
    
    // 更新为工作状态
    task = task.update_status(TaskState::Working);
    assert_eq!(task.status.state, TaskState::Working);
    
    // 添加Agent响应
    let agent_msg = A2AMessage::agent_message("我正在为您生成文章...".to_string());
    task = task.add_message(agent_msg);
    assert_eq!(task.history.len(), 2);
    
    // 添加工件
    let artifact = Artifact {
        artifact_id: "article_001".to_string(),
        name: Some("AI技术文章".to_string()),
        parts: vec![MessagePart::Text(TextPart {
            text: "人工智能（AI）是计算机科学的一个分支...".to_string(),
            metadata: std::collections::HashMap::new(),
        })],
        metadata: std::collections::HashMap::new(),
    };
    
    task = task.add_artifact(artifact);
    assert_eq!(task.artifacts.len(), 1);
    
    // 完成任务
    task = task.update_status(TaskState::Completed);
    assert_eq!(task.status.state, TaskState::Completed);
    
    // 验证最终状态
    assert_eq!(task.history.len(), 2);
    assert_eq!(task.artifacts.len(), 1);
    assert_eq!(task.artifacts[0].name, Some("AI技术文章".to_string()));
    
    println!("✅ 任务生命周期测试通过");
}

#[test]
fn test_protocol_engine_basic() {
    println!("🧪 测试协议引擎基础功能");
    
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 验证初始状态
    let stats = engine.get_stats();
    assert_eq!(stats.total_tasks, 0);
    assert_eq!(stats.active_tasks, 0);
    
    // 注册Agent
    let agent = AgentInfo {
        id: "test_agent".to_string(),
        name: "测试Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["text_generation".to_string()],
        status: AgentStatus::Online,
    };
    
    engine.register_agent(agent);
    
    // 验证Agent数量
    assert_eq!(engine.get_active_tasks_count(), 0);
    
    println!("✅ 协议引擎基础功能测试通过");
}

#[test]
fn test_message_content_extraction() {
    println!("🧪 测试消息内容提取");
    
    let text_msg = A2AMessage::user_message("这是一条文本消息".to_string());
    assert_eq!(text_msg.get_text_content(), Some("这是一条文本消息".to_string()));
    
    // 测试文件消息（没有文本内容）
    let file_data = FileData::WithBytes(FileWithBytes {
        name: Some("image.png".to_string()),
        mime_type: "image/png".to_string(),
        bytes: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==".to_string(),
    });
    
    let file_msg = A2AMessage::new_file(MessageRole::User, file_data);
    assert_eq!(file_msg.get_text_content(), None);
    
    println!("✅ 消息内容提取测试通过");
}

#[test]
fn test_error_handling() {
    println!("🧪 测试错误处理");
    
    // 测试JSON-RPC错误
    let parse_error = JsonRpcError::parse_error();
    assert_eq!(parse_error.code, -32700);
    assert_eq!(parse_error.message, "Parse error");
    
    let invalid_request = JsonRpcError::invalid_request();
    assert_eq!(invalid_request.code, -32600);
    
    let method_not_found = JsonRpcError::method_not_found();
    assert_eq!(method_not_found.code, -32601);
    
    let invalid_params = JsonRpcError::invalid_params();
    assert_eq!(invalid_params.code, -32602);
    
    let internal_error = JsonRpcError::internal_error();
    assert_eq!(internal_error.code, -32603);
    
    // 测试自定义错误
    let custom_error = JsonRpcError::new(
        -32000,
        "自定义错误".to_string(),
        Some(serde_json::json!({"details": "错误详情"}))
    );
    assert_eq!(custom_error.code, -32000);
    assert_eq!(custom_error.message, "自定义错误");
    assert!(custom_error.data.is_some());
    
    println!("✅ 错误处理测试通过");
}
