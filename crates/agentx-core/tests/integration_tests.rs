//! AgentX核心功能集成测试
//! 
//! 测试协议兼容、云原生部署和开发者生态系统的集成功能

use agentx_core::*;
use agentx_a2a::*;
use serde_json;
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_protocol_compatibility_integration() {
    let mut core = AgentXCore::new();
    
    // 测试MCP协议兼容
    let mcp_data = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "test-1",
        "method": "tools/call",
        "params": {
            "tool": "calculator",
            "arguments": {"operation": "add", "a": 1, "b": 2}
        }
    });
    
    let a2a_msg = core.protocol_compat()
        .auto_convert_to_a2a(mcp_data)
        .await
        .unwrap();
    
    assert_eq!(a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap(), "mcp");
    assert_eq!(a2a_msg.metadata.get("mcp_method").unwrap().as_str().unwrap(), "tools/call");
    
    // 测试OpenAI协议兼容
    let openai_data = serde_json::json!({
        "role": "user",
        "content": "Hello, how can you help me today?",
        "name": "test_user"
    });
    
    let a2a_msg = core.protocol_compat()
        .auto_convert_to_a2a(openai_data)
        .await
        .unwrap();
    
    assert_eq!(a2a_msg.role, MessageRole::User);
    assert_eq!(a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap(), "openai");
    assert_eq!(a2a_msg.metadata.get("name").unwrap().as_str().unwrap(), "test_user");
    
    println!("✅ 协议兼容性集成测试通过");
}

#[tokio::test]
async fn test_cloud_native_deployment_generation() {
    let mut core = AgentXCore::new();
    
    // 配置Kubernetes部署
    let k8s_config = KubernetesConfig {
        namespace: "agentx-test".to_string(),
        deployment_name: "agentx-core-test".to_string(),
        service_name: "agentx-service-test".to_string(),
        replicas: 2,
        image: "agentx/core".to_string(),
        image_tag: "test".to_string(),
        resources: cloud_native::ResourceRequirements {
            cpu_request: "200m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "1Gi".to_string(),
        },
        env_vars: {
            let mut env = HashMap::new();
            env.insert("RUST_LOG".to_string(), "debug".to_string());
            env.insert("AGENTX_MODE".to_string(), "test".to_string());
            env
        },
        config_maps: vec!["agentx-config".to_string()],
        secrets: vec!["agentx-secrets".to_string()],
        ingress: Some(cloud_native::IngressConfig {
            host: "agentx-test.example.com".to_string(),
            path: "/".to_string(),
            tls_enabled: true,
            cert_manager: true,
        }),
    };
    
    // 配置Docker部署
    let docker_config = DockerConfig {
        image_name: "agentx/core".to_string(),
        tag: "test".to_string(),
        dockerfile_path: "Dockerfile".to_string(),
        build_context: ".".to_string(),
        build_args: HashMap::new(),
        ports: vec![
            cloud_native::PortMapping {
                host_port: 50051,
                container_port: 50051,
                protocol: "tcp".to_string(),
            },
            cloud_native::PortMapping {
                host_port: 8080,
                container_port: 8080,
                protocol: "tcp".to_string(),
            },
        ],
        volumes: vec![
            cloud_native::VolumeMapping {
                host_path: "./data".to_string(),
                container_path: "/app/data".to_string(),
                read_only: false,
            },
        ],
        environment: {
            let mut env = HashMap::new();
            env.insert("RUST_LOG".to_string(), "info".to_string());
            env
        },
    };
    
    *core.cloud_native() = CloudNativeManager::new()
        .with_kubernetes(k8s_config)
        .with_docker(docker_config);
    
    // 生成部署文件
    let deployment_files = core.cloud_native()
        .generate_deployment_files()
        .await
        .unwrap();
    
    // 验证生成的文件
    assert!(deployment_files.contains_key("deployment.yaml"));
    assert!(deployment_files.contains_key("service.yaml"));
    assert!(deployment_files.contains_key("ingress.yaml"));
    assert!(deployment_files.contains_key("Dockerfile"));
    assert!(deployment_files.contains_key("docker-compose.yml"));
    
    // 验证Kubernetes Deployment内容
    let deployment_yaml = deployment_files.get("deployment.yaml").unwrap();
    assert!(deployment_yaml.contains("agentx-core-test"));
    assert!(deployment_yaml.contains("replicas: 2"));
    assert!(deployment_yaml.contains("cpu: 200m"));
    assert!(deployment_yaml.contains("memory: 256Mi"));
    
    // 验证Docker Compose内容
    let compose_yaml = deployment_files.get("docker-compose.yml").unwrap();
    assert!(compose_yaml.contains("agentx/core:test"));
    assert!(compose_yaml.contains("50051:50051"));
    assert!(compose_yaml.contains("8080:8080"));
    
    // 验证配置
    let warnings = core.cloud_native()
        .validate_configuration()
        .await
        .unwrap();
    
    assert!(warnings.is_empty());
    
    println!("✅ 云原生部署生成测试通过");
}

#[tokio::test]
async fn test_developer_ecosystem_integration() {
    let mut core = AgentXCore::new();
    
    // 测试插件市场功能
    let plugin_entry = developer_ecosystem::PluginMarketEntry {
        id: "test-integration-plugin".to_string(),
        name: "Integration Test Plugin".to_string(),
        description: "A plugin for integration testing".to_string(),
        version: "1.0.0".to_string(),
        author: "AgentX Team".to_string(),
        category: developer_ecosystem::PluginCategory::Development,
        tags: vec!["test".to_string(), "integration".to_string()],
        download_url: "https://github.com/agentx/test-plugin/releases/download/v1.0.0/plugin.zip".to_string(),
        documentation_url: Some("https://docs.agentx.dev/plugins/test".to_string()),
        source_code_url: Some("https://github.com/agentx/test-plugin".to_string()),
        license: "MIT".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        downloads: 0,
        rating: 0.0,
        reviews: vec![],
        compatibility: developer_ecosystem::CompatibilityInfo {
            agentx_version: "0.1.0".to_string(),
            supported_platforms: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
            required_features: vec!["grpc".to_string()],
        },
        dependencies: vec![
            developer_ecosystem::PluginDependency {
                name: "agentx-sdk".to_string(),
                version: "0.1.0".to_string(),
                optional: false,
            },
        ],
    };
    
    // 注册插件
    core.developer_ecosystem()
        .market()
        .register_plugin(plugin_entry)
        .unwrap();
    
    // 搜索插件
    let search_results = core.developer_ecosystem()
        .market()
        .search_plugins("integration", Some(developer_ecosystem::PluginCategory::Development));
    
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "Integration Test Plugin");
    
    // 添加评价
    let review = developer_ecosystem::PluginReview {
        reviewer: "test_user".to_string(),
        rating: 5,
        comment: "Excellent plugin for testing!".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    core.developer_ecosystem()
        .market()
        .add_review("test-integration-plugin", review)
        .unwrap();
    
    // 增加下载计数
    core.developer_ecosystem()
        .market()
        .increment_downloads("test-integration-plugin")
        .unwrap();
    
    // 验证更新
    let plugin = core.developer_ecosystem()
        .market()
        .get_plugin("test-integration-plugin")
        .unwrap();
    
    assert_eq!(plugin.downloads, 1);
    assert_eq!(plugin.reviews.len(), 1);
    assert_eq!(plugin.rating, 5.0);
    
    // 测试CLI工具
    let init_command = core.developer_ecosystem()
        .cli()
        .get_command("init")
        .unwrap();
    
    assert_eq!(init_command.name, "init");
    assert!(!init_command.options.is_empty());
    
    let rust_template = core.developer_ecosystem()
        .cli()
        .get_template("rust-plugin")
        .unwrap();
    
    assert_eq!(rust_template.language, "rust");
    assert!(rust_template.files.contains_key("Cargo.toml"));
    assert!(rust_template.files.contains_key("src/main.rs"));
    
    println!("✅ 开发者生态系统集成测试通过");
}

#[tokio::test]
async fn test_full_system_integration() {
    let mut core = AgentXCore::new();
    
    // 初始化系统
    core.initialize().await.unwrap();
    
    // 获取系统信息
    let system_info = core.get_system_info();
    assert_eq!(system_info.version, agentx_core::VERSION);
    assert!(system_info.features.len() >= 5);
    
    // 测试协议转换流程
    let mcp_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "integration-test",
        "method": "completion/complete",
        "params": {
            "prompt": "Write a hello world program",
            "max_tokens": 100
        }
    });
    
    let a2a_message = core.protocol_compat()
        .auto_convert_to_a2a(mcp_message)
        .await
        .unwrap();
    
    // 验证转换结果
    assert_eq!(a2a_message.metadata.get("protocol").unwrap().as_str().unwrap(), "mcp");
    assert_eq!(a2a_message.metadata.get("mcp_method").unwrap().as_str().unwrap(), "completion/complete");
    
    // 测试云原生配置验证
    let warnings = core.cloud_native()
        .validate_configuration()
        .await
        .unwrap();
    
    // 应该有警告，因为没有配置部署方式
    assert!(!warnings.is_empty());
    
    // 测试开发者生态系统
    let commands = core.developer_ecosystem().cli().list_commands();
    assert!(!commands.is_empty());
    
    let templates = core.developer_ecosystem().cli().list_templates();
    assert!(!templates.is_empty());
    
    println!("✅ 完整系统集成测试通过");
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let mut core = AgentXCore::new();
    
    // 协议转换性能测试
    let start = std::time::Instant::now();
    
    for i in 0..1000 {
        let test_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": format!("perf-test-{}", i),
            "method": "tools/call",
            "params": {"tool": "test", "args": {}}
        });
        
        let _a2a_msg = core.protocol_compat()
            .auto_convert_to_a2a(test_message)
            .await
            .unwrap();
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_micros() / 1000;
    
    println!("📊 协议转换性能:");
    println!("   消息数量: 1000");
    println!("   总时间: {:?}", duration);
    println!("   平均时间: {}μs", avg_time);
    println!("   吞吐量: {:.2} 消息/秒", 1000.0 / duration.as_secs_f64());
    
    // 验证性能目标（每次转换应该在1ms以内）
    assert!(avg_time < 1000, "协议转换平均时间超过1ms: {}μs", avg_time);
    
    // 插件搜索性能测试
    let start = std::time::Instant::now();
    
    // 注册大量插件
    for i in 0..100 {
        let plugin = developer_ecosystem::PluginMarketEntry {
            id: format!("perf-plugin-{}", i),
            name: format!("Performance Plugin {}", i),
            description: format!("Plugin {} for performance testing", i),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            category: developer_ecosystem::PluginCategory::Tools,
            tags: vec![format!("perf-{}", i % 10)],
            download_url: "https://example.com/plugin.zip".to_string(),
            documentation_url: None,
            source_code_url: None,
            license: "MIT".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            downloads: i as u64,
            rating: (i % 5 + 1) as f32,
            reviews: vec![],
            compatibility: developer_ecosystem::CompatibilityInfo {
                agentx_version: "0.1.0".to_string(),
                supported_platforms: vec!["linux".to_string()],
                required_features: vec![],
            },
            dependencies: vec![],
        };
        
        core.developer_ecosystem()
            .market()
            .register_plugin(plugin)
            .unwrap();
    }
    
    // 执行搜索
    for i in 0..100 {
        let _results = core.developer_ecosystem()
            .market()
            .search_plugins(&format!("perf-{}", i % 10), None);
    }
    
    let search_duration = start.elapsed();
    let avg_search_time = search_duration.as_micros() / 100;
    
    println!("📊 插件搜索性能:");
    println!("   插件数量: 100");
    println!("   搜索次数: 100");
    println!("   总时间: {:?}", search_duration);
    println!("   平均搜索时间: {}μs", avg_search_time);
    
    // 验证搜索性能（每次搜索应该在10ms以内）
    assert!(avg_search_time < 10000, "插件搜索平均时间超过10ms: {}μs", avg_search_time);
    
    println!("✅ 性能基准测试通过");
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    let mut core = AgentXCore::new();
    
    // 测试无效协议数据处理
    let invalid_data = serde_json::json!({
        "invalid": "data",
        "missing": "required_fields"
    });

    let result = core.protocol_compat()
        .auto_convert_to_a2a(invalid_data)
        .await;

    // 无效数据应该导致错误
    assert!(result.is_err());
    
    // 测试插件市场错误处理
    let invalid_plugin = developer_ecosystem::PluginMarketEntry {
        id: "".to_string(), // 无效的空ID
        name: "".to_string(), // 无效的空名称
        description: "Test".to_string(),
        version: "".to_string(), // 无效的空版本
        author: "Test".to_string(),
        category: developer_ecosystem::PluginCategory::Tools,
        tags: vec![],
        download_url: "invalid-url".to_string(),
        documentation_url: None,
        source_code_url: None,
        license: "MIT".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        downloads: 0,
        rating: 0.0,
        reviews: vec![],
        compatibility: developer_ecosystem::CompatibilityInfo {
            agentx_version: "0.1.0".to_string(),
            supported_platforms: vec![],
            required_features: vec![],
        },
        dependencies: vec![],
    };
    
    let result = core.developer_ecosystem()
        .market()
        .register_plugin(invalid_plugin);
    
    assert!(result.is_err());
    
    // 测试不存在的插件操作
    let result = core.developer_ecosystem()
        .market()
        .get_plugin("nonexistent-plugin");
    
    assert!(result.is_none());
    
    let result = core.developer_ecosystem()
        .market()
        .increment_downloads("nonexistent-plugin");
    
    assert!(result.is_err());
    
    println!("✅ 错误处理和恢复测试通过");
}
