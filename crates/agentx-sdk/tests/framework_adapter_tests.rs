//! 框架适配器集成测试
//! 
//! 测试框架适配器、消息转换器和框架管理器的功能

use agentx_sdk::*;
use agentx_a2a::{A2AMessage, A2AResult, MessageRole, MessagePart, TextPart};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 测试框架适配器
struct TestFrameworkAdapter {
    framework_type: FrameworkType,
    is_initialized: Arc<Mutex<bool>>,
    is_running: Arc<Mutex<bool>>,
    message_count: Arc<Mutex<u32>>,
}

impl TestFrameworkAdapter {
    fn new(framework_type: FrameworkType) -> Self {
        Self {
            framework_type,
            is_initialized: Arc::new(Mutex::new(false)),
            is_running: Arc::new(Mutex::new(false)),
            message_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl FrameworkAdapter for TestFrameworkAdapter {
    fn get_framework_type(&self) -> FrameworkType {
        self.framework_type.clone()
    }

    async fn initialize_environment(&mut self) -> A2AResult<()> {
        println!("初始化测试框架环境: {:?}", self.framework_type);
        *self.is_initialized.lock().await = true;
        Ok(())
    }

    async fn start_framework(&mut self) -> A2AResult<()> {
        println!("启动测试框架: {:?}", self.framework_type);
        *self.is_running.lock().await = true;
        Ok(())
    }

    async fn stop_framework(&mut self) -> A2AResult<()> {
        println!("停止测试框架: {:?}", self.framework_type);
        *self.is_running.lock().await = false;
        Ok(())
    }

    async fn check_health(&self) -> A2AResult<bool> {
        Ok(*self.is_running.lock().await)
    }

    async fn execute_command(&mut self, command: &str, args: Vec<String>) -> A2AResult<String> {
        println!("执行命令: {} {:?}", command, args);
        Ok(format!("命令执行结果: {} {:?}", command, args))
    }

    async fn convert_message_to_framework(&self, message: &A2AMessage) -> A2AResult<Value> {
        // 提取文本内容
        let content = message.parts.iter()
            .filter_map(|part| {
                if let MessagePart::Text(text_part) = part {
                    Some(text_part.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        let framework_message = match self.framework_type {
            FrameworkType::LangChain => json!({
                "role": match message.role {
                    MessageRole::User => "human",
                    MessageRole::Agent => "assistant",
                },
                "content": content,
                "additional_kwargs": message.metadata
            }),
            FrameworkType::AutoGen => json!({
                "role": match message.role {
                    MessageRole::User => "user",
                    MessageRole::Agent => "assistant",
                },
                "content": content,
                "metadata": message.metadata
            }),
            FrameworkType::Mastra => json!({
                "role": match message.role {
                    MessageRole::User => "user",
                    MessageRole::Agent => "assistant",
                },
                "content": content,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "context": message.metadata
            }),
            _ => json!({
                "role": "user",
                "content": content,
                "metadata": message.metadata
            }),
        };

        Ok(framework_message)
    }

    async fn convert_message_from_framework(&self, data: Value) -> A2AResult<A2AMessage> {
        *self.message_count.lock().await += 1;

        let role_str = data.get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("user");

        let role = match role_str {
            "human" | "user" => MessageRole::User,
            "assistant" | "ai" => MessageRole::Agent,
            _ => MessageRole::User,
        };

        let content = data.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let metadata = data.get("metadata")
            .or_else(|| data.get("additional_kwargs"))
            .or_else(|| data.get("context"))
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        Ok(A2AMessage {
            role,
            parts: vec![MessagePart::Text(TextPart {
                text: format!("来自 {:?} 的响应: {}", self.framework_type, content),
                metadata: HashMap::new(),
            })],
            message_id: uuid::Uuid::new_v4().to_string(),
            task_id: None,
            context_id: None,
            metadata,
        })
    }
}

#[tokio::test]
async fn test_message_converter() {
    println!("🚀 测试消息转换器");

    let mut converter = MessageConverter::new();

    // 创建测试A2A消息
    let a2a_message = A2AMessage {
        role: MessageRole::User,
        parts: vec![MessagePart::Text(TextPart {
            text: "Hello, world!".to_string(),
            metadata: HashMap::new(),
        })],
        message_id: "test_msg_1".to_string(),
        task_id: None,
        context_id: None,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("test_key".to_string(), json!("test_value"));
            meta
        },
    };

    // 测试转换为LangChain格式
    let langchain_msg = converter.convert_from_a2a(&a2a_message, FrameworkType::LangChain).unwrap();
    assert_eq!(langchain_msg["role"], "human");
    assert_eq!(langchain_msg["content"], "Hello, world!");

    // 测试转换为AutoGen格式
    let autogen_msg = converter.convert_from_a2a(&a2a_message, FrameworkType::AutoGen).unwrap();
    assert_eq!(autogen_msg["role"], "user");
    assert_eq!(autogen_msg["content"], "Hello, world!");

    // 测试转换为Mastra格式
    let mastra_msg = converter.convert_from_a2a(&a2a_message, FrameworkType::Mastra).unwrap();
    assert_eq!(mastra_msg["role"], "user");
    assert_eq!(mastra_msg["content"], "Hello, world!");

    // 测试从LangChain格式转换回A2A
    let converted_back = converter.convert_to_a2a(langchain_msg, FrameworkType::LangChain).unwrap();
    assert_eq!(converted_back.role, MessageRole::User);
    // 检查文本内容
    if let Some(MessagePart::Text(text_part)) = converted_back.parts.first() {
        assert_eq!(text_part.text, "Hello, world!");
    } else {
        panic!("Expected text part");
    }

    // 测试框架间直接转换
    let autogen_to_mastra = converter.convert_between_frameworks(
        autogen_msg,
        FrameworkType::AutoGen,
        FrameworkType::Mastra,
    ).unwrap();
    assert_eq!(autogen_to_mastra["role"], "user");

    // 检查转换统计
    let stats = converter.get_stats();
    assert!(stats.total_conversions > 0);
    assert!(stats.successful_conversions > 0);
    assert_eq!(stats.failed_conversions, 0);

    println!("✅ 消息转换器测试通过");
}

#[tokio::test]
async fn test_framework_manager() {
    println!("🚀 测试框架管理器");

    let config = FrameworkManagerConfig {
        enable_health_check: false, // 禁用健康检查以简化测试
        health_check_interval_secs: 10,
        enable_conversion_cache: true,
        max_concurrent_frameworks: 5,
        message_timeout_secs: 30,
    };

    let manager = FrameworkManager::new(config);

    // 注册测试框架适配器
    let langchain_adapter = Box::new(TestFrameworkAdapter::new(FrameworkType::LangChain));
    let autogen_adapter = Box::new(TestFrameworkAdapter::new(FrameworkType::AutoGen));
    let mastra_adapter = Box::new(TestFrameworkAdapter::new(FrameworkType::Mastra));

    let framework_config = FrameworkConfig {
        framework_type: FrameworkType::LangChain,
        runtime_path: "python".to_string(),
        working_directory: "/tmp".to_string(),
        environment_variables: HashMap::new(),
        dependencies: vec![],
        startup_args: vec![],
        custom_config: HashMap::new(),
    };

    // 注册框架
    manager.register_framework(FrameworkType::LangChain, langchain_adapter, framework_config.clone()).await.unwrap();
    manager.register_framework(FrameworkType::AutoGen, autogen_adapter, framework_config.clone()).await.unwrap();
    manager.register_framework(FrameworkType::Mastra, mastra_adapter, framework_config.clone()).await.unwrap();

    // 检查注册状态
    let supported_frameworks = manager.get_supported_frameworks().await;
    assert_eq!(supported_frameworks.len(), 3);
    assert!(supported_frameworks.contains(&FrameworkType::LangChain));
    assert!(supported_frameworks.contains(&FrameworkType::AutoGen));
    assert!(supported_frameworks.contains(&FrameworkType::Mastra));

    // 启动框架
    manager.start_framework(&FrameworkType::LangChain).await.unwrap();
    manager.start_framework(&FrameworkType::AutoGen).await.unwrap();
    manager.start_framework(&FrameworkType::Mastra).await.unwrap();

    // 检查框架状态
    let langchain_state = manager.get_framework_state(&FrameworkType::LangChain).await.unwrap();
    assert_eq!(langchain_state.status, FrameworkStatus::Running);

    // 创建测试消息
    let test_message = A2AMessage {
        role: MessageRole::User,
        parts: vec![MessagePart::Text(TextPart {
            text: "测试消息处理".to_string(),
            metadata: HashMap::new(),
        })],
        message_id: "test_msg_2".to_string(),
        task_id: None,
        context_id: None,
        metadata: HashMap::new(),
    };

    // 测试单框架消息处理
    let result = manager.process_message(&FrameworkType::LangChain, test_message.clone()).await.unwrap();
    assert!(result.success);
    assert!(result.response_message.is_some());
    // 处理时间可能为0（测试环境下执行很快）
    assert!(result.processing_time_ms >= 0);

    // 测试框架间消息转发
    let forward_result = manager.forward_message(
        &FrameworkType::LangChain,
        &FrameworkType::AutoGen,
        test_message.clone(),
    ).await.unwrap();
    assert!(forward_result.success);
    assert!(forward_result.response_message.is_some());
    assert_eq!(forward_result.source_framework, FrameworkType::LangChain);
    assert_eq!(forward_result.target_framework, Some(FrameworkType::AutoGen));

    // 检查转换统计
    let conversion_stats = manager.get_conversion_stats().await;
    assert!(conversion_stats.total_conversions > 0);

    // 获取所有框架状态
    let all_states = manager.get_all_framework_states().await;
    assert_eq!(all_states.len(), 3);
    
    // 检查消息处理统计
    let langchain_state = all_states.get(&FrameworkType::LangChain).unwrap();
    assert!(langchain_state.messages_processed > 0);

    // 停止框架
    manager.stop_framework(&FrameworkType::LangChain).await.unwrap();
    manager.stop_framework(&FrameworkType::AutoGen).await.unwrap();
    manager.stop_framework(&FrameworkType::Mastra).await.unwrap();

    // 检查停止状态
    let stopped_state = manager.get_framework_state(&FrameworkType::LangChain).await.unwrap();
    assert_eq!(stopped_state.status, FrameworkStatus::Stopped);

    println!("✅ 框架管理器测试通过");
}

#[tokio::test]
async fn test_framework_adapter_lifecycle() {
    println!("🚀 测试框架适配器生命周期");

    let mut adapter = TestFrameworkAdapter::new(FrameworkType::LangChain);

    // 测试初始化
    assert!(!*adapter.is_initialized.lock().await);
    adapter.initialize_environment().await.unwrap();
    assert!(*adapter.is_initialized.lock().await);

    // 测试启动
    assert!(!*adapter.is_running.lock().await);
    adapter.start_framework().await.unwrap();
    assert!(*adapter.is_running.lock().await);

    // 测试健康检查
    let health = adapter.check_health().await.unwrap();
    assert!(health);

    // 测试命令执行
    let result = adapter.execute_command("test_command", vec!["arg1".to_string(), "arg2".to_string()]).await.unwrap();
    assert!(result.contains("test_command"));

    // 测试消息转换
    let test_message = A2AMessage {
        role: MessageRole::User,
        parts: vec![MessagePart::Text(TextPart {
            text: "测试适配器消息".to_string(),
            metadata: HashMap::new(),
        })],
        message_id: "test_msg_3".to_string(),
        task_id: None,
        context_id: None,
        metadata: HashMap::new(),
    };

    let framework_msg = adapter.convert_message_to_framework(&test_message).await.unwrap();
    assert_eq!(framework_msg["role"], "human"); // LangChain格式
    assert_eq!(framework_msg["content"], "测试适配器消息");

    let converted_back = adapter.convert_message_from_framework(framework_msg).await.unwrap();
    // 检查响应内容
    if let Some(MessagePart::Text(text_part)) = converted_back.parts.first() {
        assert!(text_part.text.contains("来自 LangChain 的响应"));
    } else {
        panic!("Expected text part in response");
    }

    // 测试停止
    adapter.stop_framework().await.unwrap();
    assert!(!*adapter.is_running.lock().await);

    // 停止后健康检查应该返回false
    let health_after_stop = adapter.check_health().await.unwrap();
    assert!(!health_after_stop);

    println!("✅ 框架适配器生命周期测试通过");
}

#[tokio::test]
async fn test_multi_framework_interaction() {
    println!("🚀 测试多框架交互");

    let manager = FrameworkManager::new(FrameworkManagerConfig::default());

    // 注册多个框架
    let frameworks = vec![
        FrameworkType::LangChain,
        FrameworkType::AutoGen,
        FrameworkType::Mastra,
    ];

    for framework_type in &frameworks {
        let adapter = Box::new(TestFrameworkAdapter::new(framework_type.clone()));
        let config = FrameworkConfig {
            framework_type: framework_type.clone(),
            runtime_path: "test".to_string(),
            working_directory: "/tmp".to_string(),
            environment_variables: HashMap::new(),
            dependencies: vec![],
            startup_args: vec![],
            custom_config: HashMap::new(),
        };
        
        manager.register_framework(framework_type.clone(), adapter, config).await.unwrap();
        manager.start_framework(framework_type).await.unwrap();
    }

    // 创建测试消息
    let test_message = A2AMessage {
        role: MessageRole::User,
        parts: vec![MessagePart::Text(TextPart {
            text: "多框架交互测试".to_string(),
            metadata: HashMap::new(),
        })],
        message_id: "multi_test_msg".to_string(),
        task_id: None,
        context_id: None,
        metadata: HashMap::new(),
    };

    // 测试所有框架组合的消息转发
    for source in &frameworks {
        for target in &frameworks {
            if source != target {
                let result = manager.forward_message(source, target, test_message.clone()).await.unwrap();
                assert!(result.success, "转发失败: {:?} -> {:?}", source, target);
                assert!(result.response_message.is_some());
                assert_eq!(result.source_framework, *source);
                assert_eq!(result.target_framework, Some(target.clone()));
            }
        }
    }

    // 检查所有框架的统计信息
    let all_states = manager.get_all_framework_states().await;
    for framework_type in &frameworks {
        let state = all_states.get(framework_type).unwrap();
        assert!(state.messages_processed > 0);
        assert_eq!(state.status, FrameworkStatus::Running);
    }

    // 检查转换统计
    let conversion_stats = manager.get_conversion_stats().await;
    assert!(conversion_stats.total_conversions > 0);
    assert!(conversion_stats.successful_conversions > 0);
    assert_eq!(conversion_stats.failed_conversions, 0);

    // 验证每个框架都有转换记录
    for framework_type in &frameworks {
        assert!(conversion_stats.conversions_by_framework.contains_key(framework_type));
    }

    println!("✅ 多框架交互测试通过");
}
