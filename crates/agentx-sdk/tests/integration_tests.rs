//! AgentX SDK集成测试
//! 
//! 测试多框架插件的集成功能

use agentx_sdk::{
    init_sdk, PluginBuilder, FrameworkBuilder, ConfigBuilder,
    FrameworkType, PluginCapability, FrameworkUtils, PluginUtils,
    A2AMessage, MessageRole, A2AResult,
};
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_sdk_initialization() -> Result<(), Box<dyn std::error::Error>> {
    // 测试SDK初始化
    let result = init_sdk().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_plugin_creation() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化SDK
    init_sdk().await?;
    
    // 测试创建不同框架的插件
    let frameworks = vec!["langchain", "autogen", "mastra"];
    
    for framework in frameworks {
        let plugin = PluginBuilder::new()
            .framework(framework)
            .capability(PluginCapability::TextProcessing)
            .build()
            .await;
        
        assert!(plugin.is_ok(), "Failed to create {} plugin", framework);
        
        let plugin = plugin.unwrap();
        assert_eq!(plugin.get_info().config.framework, framework);
        assert!(plugin.get_capabilities().contains(&PluginCapability::TextProcessing));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_framework_environment_detection() -> Result<(), Box<dyn std::error::Error>> {
    // 测试环境检测
    let frameworks = vec!["langchain", "autogen", "mastra"];
    
    for framework in frameworks {
        let result = FrameworkUtils::detect_framework_environment(framework).await;
        assert!(result.is_ok(), "Failed to detect {} environment", framework);
        
        let env_info = result.unwrap();
        assert_eq!(env_info.runtime, match framework {
            "langchain" | "autogen" => "python",
            "mastra" => "node",
            _ => "unknown",
        });
    }
    
    Ok(())
}

#[tokio::test]
async fn test_framework_creation() -> Result<(), Box<dyn std::error::Error>> {
    // 测试框架实例创建
    let framework_types = vec![
        FrameworkType::LangChain,
        FrameworkType::AutoGen,
        FrameworkType::Mastra,
    ];
    
    for framework_type in framework_types {
        let config = agentx_sdk::framework::FrameworkConfig {
            framework_type: framework_type.clone(),
            runtime_path: framework_type.default_runtime_path().to_string(),
            working_directory: ".".to_string(),
            environment_variables: HashMap::new(),
            startup_args: Vec::new(),
            dependencies: Vec::new(),
            custom_config: HashMap::new(),
        };
        
        let framework = FrameworkBuilder::new()
            .framework_type(framework_type.clone())
            .config(config)
            .build()
            .await;
        
        assert!(framework.is_ok(), "Failed to create {:?} framework", framework_type);
        
        let framework = framework.unwrap();
        assert_eq!(framework.get_type(), &framework_type);
        assert!(!framework.is_running());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_message_processing() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化SDK
    init_sdk().await?;
    
    // 创建测试插件
    let mut plugin = PluginBuilder::new()
        .framework("custom")
        .capability(PluginCapability::TextProcessing)
        .build()
        .await?;
    
    // 测试消息处理
    let test_message = A2AMessage::agent_message("Test message".to_string());
    let result = plugin.process_message(test_message).await;
    
    assert!(result.is_ok());
    
    if let Ok(Some(response)) = result {
        assert!(!response.message_id.is_empty());
        assert!(!response.parts.is_empty());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化SDK
    init_sdk().await?;
    
    // 创建测试插件
    let mut plugin = PluginBuilder::new()
        .framework("custom")
        .build()
        .await?;
    
    // 测试生命周期方法
    assert!(plugin.initialize().await.is_ok());
    assert!(plugin.start().await.is_ok());
    assert!(plugin.health_check().await.is_ok());
    assert!(plugin.pause().await.is_ok());
    assert!(plugin.resume().await.is_ok());
    assert!(plugin.stop().await.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_configuration_management() -> Result<(), Box<dyn std::error::Error>> {
    // 测试配置构建
    let config = ConfigBuilder::new()
        .framework("test_framework")
        .runtime_path("/usr/bin/python")
        .working_directory("/tmp")
        .env_var("TEST_VAR", "test_value")
        .custom("test_key", serde_json::Value::String("test_value".to_string()))
        .build()?;
    
    assert_eq!(config.framework, "test_framework");
    assert!(config.custom.contains_key("test_key"));
    
    // 测试配置合并
    let base_config = agentx_sdk::PluginConfig::default();
    let override_config = agentx_sdk::PluginConfig {
        framework: "override_framework".to_string(),
        ..Default::default()
    };
    
    let merged = agentx_sdk::ConfigUtils::merge_configs(base_config, override_config);
    assert_eq!(merged.framework, "override_framework");
    
    Ok(())
}

#[tokio::test]
async fn test_message_utilities() -> Result<(), Box<dyn std::error::Error>> {
    // 测试消息工具
    let message = A2AMessage::agent_message("Test content".to_string());
    
    // 测试消息验证
    assert!(agentx_sdk::MessageUtils::validate_message(&message).is_ok());
    
    // 测试文本内容提取
    let content = agentx_sdk::MessageUtils::extract_text_content(&message);
    assert_eq!(content, "Test content");
    
    // 测试消息大小计算
    let size = agentx_sdk::MessageUtils::calculate_message_size(&message);
    assert!(size > 0);
    
    // 测试消息增强
    let mut metadata = HashMap::new();
    metadata.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
    
    let enhanced = agentx_sdk::MessageUtils::enhance_message_metadata(message, metadata);
    assert!(enhanced.metadata.contains_key("test_key"));
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_utilities() -> Result<(), Box<dyn std::error::Error>> {
    // 测试插件工具
    
    // 测试ID生成
    let id1 = PluginUtils::generate_plugin_id("test_framework", "test_plugin");
    let id2 = PluginUtils::generate_plugin_id("test_framework", "test_plugin");
    assert_ne!(id1, id2); // ID应该是唯一的
    assert!(id1.starts_with("test_framework_test_plugin_"));
    
    // 测试版本比较
    use std::cmp::Ordering;
    assert_eq!(PluginUtils::compare_versions("1.0.0", "1.0.1"), Ordering::Less);
    assert_eq!(PluginUtils::compare_versions("2.0.0", "1.9.9"), Ordering::Greater);
    assert_eq!(PluginUtils::compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
    
    Ok(())
}

#[tokio::test]
async fn test_validation_utilities() -> Result<(), Box<dyn std::error::Error>> {
    // 测试验证工具
    
    // 测试URL验证
    assert!(agentx_sdk::ValidationUtils::validate_url("http://localhost:8080"));
    assert!(agentx_sdk::ValidationUtils::validate_url("https://example.com"));
    assert!(!agentx_sdk::ValidationUtils::validate_url("invalid-url"));
    
    // 测试端口验证
    assert!(agentx_sdk::ValidationUtils::validate_port(8080));
    assert!(agentx_sdk::ValidationUtils::validate_port(443));
    assert!(!agentx_sdk::ValidationUtils::validate_port(0));
    
    // 测试框架名称验证
    assert!(agentx_sdk::ValidationUtils::validate_framework_name("langchain"));
    assert!(agentx_sdk::ValidationUtils::validate_framework_name("test_framework"));
    assert!(!agentx_sdk::ValidationUtils::validate_framework_name(""));
    assert!(!agentx_sdk::ValidationUtils::validate_framework_name("invalid framework"));
    
    Ok(())
}

#[tokio::test]
async fn test_performance_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化SDK
    init_sdk().await?;
    
    // 创建测试插件
    let mut plugin = PluginBuilder::new()
        .framework("custom")
        .capability(PluginCapability::TextProcessing)
        .build()
        .await?;
    
    // 性能测试
    let message_count = 100;
    let test_message = A2AMessage::agent_message("Performance test".to_string());
    
    let start_time = std::time::Instant::now();
    let mut success_count = 0;
    
    for _ in 0..message_count {
        if plugin.process_message(test_message.clone()).await.is_ok() {
            success_count += 1;
        }
    }
    
    let duration = start_time.elapsed();
    let throughput = (success_count as f64) / duration.as_secs_f64();
    
    // 验证性能指标
    assert!(success_count > 0, "No messages processed successfully");
    assert!(throughput > 1000.0, "Throughput too low: {} msg/s", throughput);
    
    println!("Performance test results:");
    println!("  Messages processed: {}/{}", success_count, message_count);
    println!("  Throughput: {:.0} msg/s", throughput);
    println!("  Average latency: {:.2}ms", duration.as_millis() as f64 / success_count as f64);
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // 测试错误处理
    
    // 测试无效框架（使用None来测试缺少框架的情况）
    let result = PluginBuilder::new()
        .build()
        .await;
    assert!(result.is_err());
    
    // 测试无效消息
    let empty_message = A2AMessage::agent_message("".to_string());
    let validation_result = agentx_sdk::MessageUtils::validate_message(&empty_message);
    assert!(validation_result.is_ok()); // 空消息现在是允许的
    
    // 测试无效配置
    let config = agentx_sdk::PluginConfig {
        framework: "".to_string(),
        max_connections: 0,
        request_timeout: 0,
        ..Default::default()
    };
    
    let validation_result = PluginUtils::validate_config(&config);
    assert!(validation_result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    // 测试并发操作
    init_sdk().await?;
    
    let mut handles = Vec::new();
    
    // 并发创建多个插件
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let plugin = PluginBuilder::new()
                .framework("custom")
                .capability(PluginCapability::TextProcessing)
                .build()
                .await;
            
            assert!(plugin.is_ok(), "Failed to create plugin {}", i);
            plugin.unwrap()
        });
        
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut plugins = Vec::new();
    for handle in handles {
        let plugin = handle.await?;
        plugins.push(plugin);
    }
    
    assert_eq!(plugins.len(), 10);
    
    // 并发处理消息
    let mut message_handles = Vec::new();
    
    for (i, mut plugin) in plugins.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            let message = A2AMessage::agent_message(format!("Concurrent test {}", i));
            plugin.process_message(message).await
        });
        
        message_handles.push(handle);
    }
    
    // 等待所有消息处理完成
    let mut success_count = 0;
    for handle in message_handles {
        let result = handle.await?;
        if result.is_ok() {
            success_count += 1;
        }
    }
    
    assert!(success_count > 0, "No concurrent messages processed successfully");
    println!("Concurrent test: {}/10 messages processed successfully", success_count);
    
    Ok(())
}
