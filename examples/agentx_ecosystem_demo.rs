//! AgentX生态系统功能演示
//! 
//! 展示协议兼容、云原生部署和开发者生态系统的完整功能

use agentx_core::*;
use agentx_a2a::*;
use serde_json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🚀 AgentX生态系统功能演示");
    println!("{}", "=".repeat(50));
    
    // 创建AgentX核心实例
    let mut core = AgentXCore::new();
    
    // 初始化系统
    println!("\n📋 初始化AgentX核心系统...");
    core.initialize().await?;
    
    // 显示系统信息
    let system_info = core.get_system_info();
    println!("✅ 系统版本: {}", system_info.version);
    println!("✅ 支持功能: {:?}", system_info.features);
    
    // 演示协议兼容功能
    println!("\n🔄 协议兼容性演示");
    println!("{}", "-".repeat(30));
    
    demo_protocol_compatibility(&mut core).await?;
    
    // 演示云原生部署功能
    println!("\n☁️  云原生部署演示");
    println!("{}", "-".repeat(30));
    
    demo_cloud_native_deployment(&mut core).await?;
    
    // 演示开发者生态系统功能
    println!("\n👥 开发者生态系统演示");
    println!("{}", "-".repeat(30));
    
    demo_developer_ecosystem(&mut core).await?;
    
    // 性能基准测试
    println!("\n📊 性能基准测试");
    println!("{}", "-".repeat(30));
    
    demo_performance_benchmarks(&mut core).await?;
    
    println!("\n🎉 AgentX生态系统演示完成！");
    println!("{}", "=".repeat(50));
    
    Ok(())
}

/// 演示协议兼容功能
async fn demo_protocol_compatibility(core: &mut AgentXCore) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 测试MCP协议兼容...");
    
    // MCP工具调用消息
    let mcp_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "demo-1",
        "method": "tools/call",
        "params": {
            "tool": "calculator",
            "arguments": {
                "operation": "multiply",
                "a": 42,
                "b": 24
            }
        }
    });
    
    let a2a_msg = core.protocol_compat()
        .auto_convert_to_a2a(mcp_message)
        .await?;
    
    println!("  ✅ MCP消息转换成功");
    println!("     协议: {}", a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap());
    println!("     方法: {}", a2a_msg.metadata.get("mcp_method").unwrap().as_str().unwrap());
    
    // OpenAI Assistant API消息
    println!("🔧 测试OpenAI协议兼容...");
    
    let openai_message = serde_json::json!({
        "role": "user",
        "content": "请帮我计算42乘以24的结果",
        "name": "demo_user"
    });
    
    let a2a_msg = core.protocol_compat()
        .auto_convert_to_a2a(openai_message)
        .await?;
    
    println!("  ✅ OpenAI消息转换成功");
    println!("     角色: {:?}", a2a_msg.role);
    println!("     协议: {}", a2a_msg.metadata.get("protocol").unwrap().as_str().unwrap());
    
    // 注册MCP工具
    let mcp_tool = mcp::McpTool {
        name: "calculator".to_string(),
        description: "数学计算工具".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string"},
                "a": {"type": "number"},
                "b": {"type": "number"}
            }
        }),
    };
    
    core.protocol_compat().mcp().register_tool(mcp_tool);
    println!("  ✅ MCP工具注册成功");
    
    // 注册OpenAI Assistant
    let openai_assistant = openai::OpenAIAssistant {
        id: "asst_demo".to_string(),
        name: "Demo Assistant".to_string(),
        description: Some("演示用的AI助手".to_string()),
        model: "gpt-4".to_string(),
        instructions: Some("你是一个有用的AI助手".to_string()),
        tools: vec![],
    };
    
    core.protocol_compat().openai().register_assistant(openai_assistant);
    println!("  ✅ OpenAI Assistant注册成功");
    
    Ok(())
}

/// 演示云原生部署功能
async fn demo_cloud_native_deployment(core: &mut AgentXCore) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐳 配置Docker部署...");
    
    let docker_config = DockerConfig {
        image_name: "agentx/demo".to_string(),
        tag: "latest".to_string(),
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
            env.insert("AGENTX_MODE".to_string(), "production".to_string());
            env
        },
    };
    
    println!("☸️  配置Kubernetes部署...");
    
    let k8s_config = KubernetesConfig {
        namespace: "agentx-demo".to_string(),
        deployment_name: "agentx-demo".to_string(),
        service_name: "agentx-demo-service".to_string(),
        replicas: 3,
        image: "agentx/demo".to_string(),
        image_tag: "latest".to_string(),
        resources: cloud_native::ResourceRequirements {
            cpu_request: "200m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "1Gi".to_string(),
        },
        env_vars: {
            let mut env = HashMap::new();
            env.insert("RUST_LOG".to_string(), "info".to_string());
            env.insert("AGENTX_CLUSTER".to_string(), "demo".to_string());
            env
        },
        config_maps: vec!["agentx-config".to_string()],
        secrets: vec!["agentx-secrets".to_string()],
        ingress: Some(cloud_native::IngressConfig {
            host: "agentx-demo.example.com".to_string(),
            path: "/".to_string(),
            tls_enabled: true,
            cert_manager: true,
        }),
    };
    
    *core.cloud_native() = CloudNativeManager::new()
        .with_docker(docker_config)
        .with_kubernetes(k8s_config);
    
    // 生成部署文件
    let deployment_files = core.cloud_native()
        .generate_deployment_files()
        .await?;
    
    println!("  ✅ 生成的部署文件:");
    for (filename, _content) in &deployment_files {
        println!("     📄 {}", filename);
    }
    
    // 验证配置
    let warnings = core.cloud_native()
        .validate_configuration()
        .await?;
    
    if warnings.is_empty() {
        println!("  ✅ 部署配置验证通过");
    } else {
        println!("  ⚠️  配置警告: {:?}", warnings);
    }
    
    Ok(())
}

/// 演示开发者生态系统功能
async fn demo_developer_ecosystem(core: &mut AgentXCore) -> Result<(), Box<dyn std::error::Error>> {
    println!("🛒 演示插件市场功能...");
    
    // 注册示例插件
    let demo_plugins = vec![
        developer_ecosystem::PluginMarketEntry {
            id: "langchain-adapter".to_string(),
            name: "LangChain Adapter".to_string(),
            description: "LangChain框架适配器插件".to_string(),
            version: "1.2.0".to_string(),
            author: "AgentX Team".to_string(),
            category: developer_ecosystem::PluginCategory::FrameworkAdapter,
            tags: vec!["langchain".to_string(), "python".to_string(), "ai".to_string()],
            download_url: "https://github.com/agentx/langchain-adapter/releases/download/v1.2.0/plugin.zip".to_string(),
            documentation_url: Some("https://docs.agentx.dev/plugins/langchain".to_string()),
            source_code_url: Some("https://github.com/agentx/langchain-adapter".to_string()),
            license: "MIT".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            downloads: 1250,
            rating: 4.8,
            reviews: vec![],
            compatibility: developer_ecosystem::CompatibilityInfo {
                agentx_version: "0.1.0".to_string(),
                supported_platforms: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
                required_features: vec!["grpc".to_string(), "python".to_string()],
            },
            dependencies: vec![
                developer_ecosystem::PluginDependency {
                    name: "agentx-sdk".to_string(),
                    version: "0.1.0".to_string(),
                    optional: false,
                },
            ],
        },
        developer_ecosystem::PluginMarketEntry {
            id: "monitoring-dashboard".to_string(),
            name: "Monitoring Dashboard".to_string(),
            description: "实时监控和可视化仪表板".to_string(),
            version: "2.1.0".to_string(),
            author: "Community".to_string(),
            category: developer_ecosystem::PluginCategory::Monitoring,
            tags: vec!["monitoring".to_string(), "dashboard".to_string(), "metrics".to_string()],
            download_url: "https://github.com/agentx-community/monitoring-dashboard/releases/download/v2.1.0/plugin.zip".to_string(),
            documentation_url: Some("https://docs.agentx.dev/plugins/monitoring".to_string()),
            source_code_url: Some("https://github.com/agentx-community/monitoring-dashboard".to_string()),
            license: "Apache-2.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            downloads: 890,
            rating: 4.5,
            reviews: vec![],
            compatibility: developer_ecosystem::CompatibilityInfo {
                agentx_version: "0.1.0".to_string(),
                supported_platforms: vec!["linux".to_string(), "macos".to_string()],
                required_features: vec!["web-ui".to_string()],
            },
            dependencies: vec![],
        },
    ];
    
    for plugin in demo_plugins {
        core.developer_ecosystem()
            .market()
            .register_plugin(plugin)?;
    }
    
    println!("  ✅ 注册了 {} 个演示插件", 2);
    
    // 搜索插件
    let search_results = core.developer_ecosystem()
        .market()
        .search_plugins("adapter", None);
    
    println!("  🔍 搜索 'adapter' 找到 {} 个插件:", search_results.len());
    for plugin in search_results {
        println!("     📦 {} v{} (⭐ {:.1}, 📥 {})", 
            plugin.name, plugin.version, plugin.rating, plugin.downloads);
    }
    
    // 按分类搜索
    let monitoring_plugins = core.developer_ecosystem()
        .market()
        .get_plugins_by_category(&developer_ecosystem::PluginCategory::Monitoring);
    
    println!("  📊 监控类插件 {} 个:", monitoring_plugins.len());
    for plugin in monitoring_plugins {
        println!("     📈 {} - {}", plugin.name, plugin.description);
    }
    
    println!("🛠️  演示CLI工具功能...");
    
    // 列出可用命令
    let commands = core.developer_ecosystem().cli().list_commands();
    println!("  ✅ 可用CLI命令 {} 个:", commands.len());
    for cmd in commands {
        println!("     🔧 {} - {}", cmd.name, cmd.description);
    }
    
    // 列出项目模板
    let templates = core.developer_ecosystem().cli().list_templates();
    println!("  ✅ 可用项目模板 {} 个:", templates.len());
    for template in templates {
        println!("     📋 {} ({}) - {}", template.name, template.language, template.description);
    }
    
    Ok(())
}

/// 演示性能基准测试
async fn demo_performance_benchmarks(core: &mut AgentXCore) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ 协议转换性能测试...");
    
    let start = std::time::Instant::now();
    let test_count = 1000;
    
    for i in 0..test_count {
        let test_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": format!("perf-{}", i),
            "method": "tools/call",
            "params": {"tool": "test", "args": {}}
        });
        
        let _a2a_msg = core.protocol_compat()
            .auto_convert_to_a2a(test_message)
            .await?;
    }
    
    let duration = start.elapsed();
    let avg_time_us = duration.as_micros() / test_count;
    let throughput = test_count as f64 / duration.as_secs_f64();
    
    println!("  📊 协议转换性能结果:");
    println!("     消息数量: {}", test_count);
    println!("     总时间: {:?}", duration);
    println!("     平均延迟: {}μs", avg_time_us);
    println!("     吞吐量: {:.0} 消息/秒", throughput);
    
    if avg_time_us < 1000 {
        println!("     ✅ 性能目标达成 (< 1ms)");
    } else {
        println!("     ⚠️  性能需要优化 (> 1ms)");
    }
    
    println!("🔍 插件搜索性能测试...");
    
    let start = std::time::Instant::now();
    let search_count = 100;
    
    for i in 0..search_count {
        let _results = core.developer_ecosystem()
            .market()
            .search_plugins(&format!("test-{}", i % 10), None);
    }
    
    let search_duration = start.elapsed();
    let avg_search_time_us = search_duration.as_micros() / search_count;
    let search_throughput = search_count as f64 / search_duration.as_secs_f64();
    
    println!("  📊 插件搜索性能结果:");
    println!("     搜索次数: {}", search_count);
    println!("     总时间: {:?}", search_duration);
    println!("     平均延迟: {}μs", avg_search_time_us);
    println!("     搜索吞吐量: {:.0} 次/秒", search_throughput);
    
    if avg_search_time_us < 10000 {
        println!("     ✅ 搜索性能良好 (< 10ms)");
    } else {
        println!("     ⚠️  搜索性能需要优化 (> 10ms)");
    }
    
    Ok(())
}
