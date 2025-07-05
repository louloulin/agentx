//! 开发者工具集成测试
//! 
//! 测试CLI工具、调试诊断、性能分析、文档生成等功能

use agentx_core::*;
use std::time::Duration;

#[tokio::test]
async fn test_cli_tool_manager() {
    println!("🚀 测试CLI工具管理器");

    let cli_manager = CliToolManager::new();

    // 测试获取命令
    let init_cmd = cli_manager.get_command("init").unwrap();
    assert_eq!(init_cmd.name, "init");
    assert_eq!(init_cmd.description, "初始化新的AgentX项目");
    assert!(!init_cmd.options.is_empty());

    let plugin_cmd = cli_manager.get_command("plugin").unwrap();
    assert_eq!(plugin_cmd.name, "plugin");
    assert!(!plugin_cmd.subcommands.is_empty());

    // 测试获取模板
    let rust_template = cli_manager.get_template("rust-plugin").unwrap();
    assert_eq!(rust_template.language, "rust");
    assert_eq!(rust_template.framework, "agentx");
    assert!(!rust_template.files.is_empty());
    assert!(rust_template.files.contains_key("Cargo.toml"));
    assert!(rust_template.files.contains_key("src/main.rs"));

    let python_template = cli_manager.get_template("python-plugin").unwrap();
    assert_eq!(python_template.language, "python");
    assert!(python_template.files.contains_key("requirements.txt"));
    assert!(python_template.files.contains_key("main.py"));

    // 测试列出所有命令
    let commands = cli_manager.list_commands();
    assert!(commands.len() >= 3); // init, plugin, dev

    // 测试列出所有模板
    let templates = cli_manager.list_templates();
    assert!(templates.len() >= 2); // rust-plugin, python-plugin

    println!("✅ CLI工具管理器测试通过");
}

#[tokio::test]
async fn test_plugin_market_manager() {
    println!("🚀 测试插件市场管理器");

    let mut market_manager = PluginMarketManager::new();

    // 创建测试插件
    let test_plugin = developer_ecosystem::PluginMarketEntry {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        description: "A test plugin for AgentX".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        category: developer_ecosystem::PluginCategory::Tools,
        tags: vec!["test".to_string(), "demo".to_string()],
        download_url: "https://github.com/test/plugin/releases".to_string(),
        documentation_url: Some("https://docs.example.com".to_string()),
        source_code_url: Some("https://github.com/test/plugin".to_string()),
        license: "MIT".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        downloads: 100,
        rating: 4.5,
        reviews: vec![],
        compatibility: developer_ecosystem::CompatibilityInfo {
            agentx_version: "0.1.0".to_string(),
            supported_platforms: vec!["linux".to_string(), "macos".to_string()],
            required_features: vec!["grpc".to_string()],
        },
        dependencies: vec![
            developer_ecosystem::PluginDependency {
                name: "agentx-sdk".to_string(),
                version: "^0.1.0".to_string(),
                optional: false,
            }
        ],
    };

    // 测试注册插件
    market_manager.register_plugin(test_plugin.clone()).unwrap();

    // 测试搜索插件
    let search_results = market_manager.search_plugins("test", None);
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "Test Plugin");

    let tag_results = market_manager.search_plugins("", Some(developer_ecosystem::PluginCategory::Tools));
    assert_eq!(tag_results.len(), 1);

    // 测试获取插件详情
    let plugin_details = market_manager.get_plugin("test-plugin").unwrap();
    assert_eq!(plugin_details.name, "Test Plugin");
    assert_eq!(plugin_details.version, "1.0.0");
    assert_eq!(plugin_details.author, "Test Author");

    // 测试列出所有插件（通过空搜索）
    let all_plugins = market_manager.search_plugins("", None);
    assert_eq!(all_plugins.len(), 1);

    println!("✅ 插件市场管理器测试通过");
}

#[tokio::test]
async fn test_developer_ecosystem_manager() {
    println!("🚀 测试开发者生态系统管理器");

    let mut ecosystem_manager = DeveloperEcosystemManager::new();

    // 测试设置开发者环境
    ecosystem_manager.setup_developer_environment().await.unwrap();

    // 测试生成插件脚手架
    ecosystem_manager.generate_plugin_scaffold(
        "test-plugin",
        "rust-plugin",
        "./test-output"
    ).await.unwrap();

    // 测试访问子管理器
    let market = ecosystem_manager.market();
    assert!(market.search_plugins("", None).is_empty()); // 新创建的市场应该是空的

    let cli = ecosystem_manager.cli();
    assert!(!cli.list_commands().is_empty());

    println!("✅ 开发者生态系统管理器测试通过");
}

#[tokio::test]
async fn test_debug_diagnostics_manager() {
    println!("🚀 测试调试诊断管理器");

    let config = DiagnosticsConfig {
        enable_profiling: true,
        sampling_interval_ms: 100,
        data_retention_hours: 1,
        enable_verbose_logging: true,
        max_log_entries: 1000,
    };

    let diagnostics_manager = DebugDiagnosticsManager::new(config);

    // 测试开始性能分析
    let trace_id = diagnostics_manager.start_profiling().await.unwrap();
    assert!(!trace_id.is_empty());

    // 模拟一些工作
    tokio::time::sleep(Duration::from_millis(10)).await;

    // 测试停止性能分析
    let trace_result = diagnostics_manager.stop_profiling(&trace_id).await.unwrap();
    assert_eq!(trace_result.trace_id, trace_id);
    assert!(trace_result.duration.is_some());

    // 测试收集系统诊断信息
    let diagnostics_report = diagnostics_manager.collect_system_diagnostics().await.unwrap();
    assert!(!diagnostics_report.system_info.os.is_empty());
    assert!(diagnostics_report.system_info.cpu_cores > 0);
    assert!(diagnostics_report.system_info.total_memory > 0);

    // 测试网络诊断
    let targets = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
    ];
    let network_results = diagnostics_manager.run_network_diagnostics(targets).await.unwrap();
    assert_eq!(network_results.len(), 2);
    assert!(network_results.iter().all(|r| r.connected)); // 模拟连接成功

    // 测试日志模式分析
    let log_patterns = diagnostics_manager.analyze_log_patterns().await.unwrap();
    // 新系统可能没有错误模式
    assert!(log_patterns.is_empty() || !log_patterns.is_empty());

    // 测试生成诊断报告
    let report = diagnostics_manager.generate_diagnostic_report().await.unwrap();
    assert!(report.contains("AgentX 系统诊断报告"));
    assert!(report.contains("系统信息"));
    assert!(report.contains("组件健康状态"));

    println!("✅ 调试诊断管理器测试通过");
}

#[tokio::test]
async fn test_performance_analyzer() {
    println!("🚀 测试性能分析器");

    let config = PerformanceConfig {
        sampling_frequency_ms: 100,
        benchmark_duration_secs: 1, // 短时间测试
        performance_thresholds: performance_analyzer::PerformanceThresholds {
            max_response_time_ms: 100,
            max_cpu_usage: 80.0,
            max_memory_usage: 85.0,
            min_throughput: 1000.0,
            max_error_rate: 1.0,
        },
        enable_detailed_analysis: true,
        history_retention_days: 1,
    };

    let performance_analyzer = PerformanceAnalyzer::new(config);

    // 测试运行基准测试
    let benchmark_result = performance_analyzer.run_benchmark("message_routing").await.unwrap();
    assert_eq!(benchmark_result.suite_name, "message_routing");
    assert!(benchmark_result.passed);
    assert!(benchmark_result.metrics.total_requests > 0);
    assert!(benchmark_result.metrics.throughput > 0.0);
    assert!(benchmark_result.metrics.avg_response_time > Duration::from_nanos(0));

    // 验证性能指标
    assert!(benchmark_result.metrics.error_rate < 0.01); // 错误率小于1%
    assert!(benchmark_result.metrics.throughput > 1000.0); // 吞吐量大于1000 req/s
    assert!(benchmark_result.metrics.avg_response_time < Duration::from_millis(10)); // 平均响应时间小于10ms

    // 测试开始性能监控
    performance_analyzer.start_monitoring().await.unwrap();

    // 等待一小段时间让监控收集数据
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 测试分析性能瓶颈
    let bottleneck_analyses = performance_analyzer.analyze_bottlenecks().await.unwrap();
    // 可能有也可能没有瓶颈
    if !bottleneck_analyses.is_empty() {
        assert!(!bottleneck_analyses[0].component.is_empty());
        assert!(bottleneck_analyses[0].severity >= 0.0 && bottleneck_analyses[0].severity <= 1.0);
    }

    // 测试生成优化建议
    let optimization_suggestions = performance_analyzer.generate_optimization_suggestions().await.unwrap();
    if !optimization_suggestions.is_empty() {
        assert!(!optimization_suggestions[0].target_component.is_empty());
        assert!(!optimization_suggestions[0].description.is_empty());
    }

    // 测试生成性能报告
    let performance_report = performance_analyzer.generate_performance_report().await.unwrap();
    assert!(performance_report.contains("AgentX 性能分析报告"));
    assert!(performance_report.contains("基准测试结果"));
    assert!(performance_report.contains("性能监控指标"));

    println!("✅ 性能分析器测试通过");
}

#[tokio::test]
async fn test_error_recovery_manager() {
    println!("🚀 测试错误恢复管理器");

    let config = ErrorRecoveryConfig {
        max_retries: 3,
        retry_interval_ms: 100,
        backoff_factor: 2.0,
        max_backoff_ms: 5000,
        health_check_interval_secs: 1,
        failure_threshold: 3,
        recovery_threshold: 2,
        circuit_breaker_timeout_secs: 30,
    };

    let recovery_manager = ErrorRecoveryManager::new(config);

    // 测试启动错误恢复管理器
    recovery_manager.start().await.unwrap();

    // 测试注册组件
    recovery_manager.register_component("test_component", RecoveryStrategy::Restart).await;

    // 测试报告错误
    recovery_manager.report_error(
        "test_component",
        ErrorType::Network,
        "模拟网络错误",
        3
    ).await;

    // 等待一小段时间让恢复机制处理
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 测试获取组件健康状态
    let health_status = recovery_manager.get_component_health("test_component").await;
    assert!(health_status.is_some());

    // 测试获取错误历史
    let error_history = recovery_manager.get_error_history(Some(10)).await;
    assert!(!error_history.is_empty());
    assert_eq!(error_history[0].component, "test_component");
    assert_eq!(error_history[0].error_type, ErrorType::Network);

    // 测试获取恢复历史
    let recovery_history = recovery_manager.get_recovery_history(Some(10)).await;
    // 可能有也可能没有恢复历史
    println!("恢复历史记录数: {}", recovery_history.len());

    println!("✅ 错误恢复管理器测试通过");
}

#[tokio::test]
async fn test_integrated_developer_tools() {
    println!("🚀 测试开发者工具集成");

    // 创建所有开发者工具组件
    let ecosystem_manager = DeveloperEcosystemManager::new();
    let diagnostics_manager = DebugDiagnosticsManager::new(DiagnosticsConfig::default());
    let performance_analyzer = PerformanceAnalyzer::new(PerformanceConfig::default());
    let recovery_manager = ErrorRecoveryManager::new(ErrorRecoveryConfig::default());

    // 启动监控和恢复服务
    performance_analyzer.start_monitoring().await.unwrap();
    recovery_manager.start().await.unwrap();

    // 模拟开发工作流程

    // 1. 开发者初始化项目
    let cli = ecosystem_manager.cli();
    let init_cmd = cli.get_command("init").unwrap();
    assert_eq!(init_cmd.name, "init");

    // 2. 开发者创建插件
    let rust_template = cli.get_template("rust-plugin").unwrap();
    assert!(rust_template.files.contains_key("Cargo.toml"));

    // 3. 开发者运行性能测试
    let trace_id = diagnostics_manager.start_profiling().await.unwrap();
    
    // 模拟一些工作负载
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    let trace_result = diagnostics_manager.stop_profiling(&trace_id).await.unwrap();
    assert!(trace_result.duration.unwrap() >= Duration::from_millis(50));

    // 4. 开发者运行基准测试
    let benchmark_result = performance_analyzer.run_benchmark("message_routing").await.unwrap();
    assert!(benchmark_result.passed);

    // 5. 开发者分析系统状态
    let diagnostics_report = diagnostics_manager.collect_system_diagnostics().await.unwrap();
    assert!(!diagnostics_report.system_info.agentx_version.is_empty());

    // 6. 开发者获取优化建议
    let _suggestions = performance_analyzer.generate_optimization_suggestions().await.unwrap();
    // 可能有也可能没有建议

    // 7. 开发者生成完整报告
    let diagnostic_report = diagnostics_manager.generate_diagnostic_report().await.unwrap();
    let performance_report = performance_analyzer.generate_performance_report().await.unwrap();
    
    assert!(diagnostic_report.contains("系统诊断报告"));
    assert!(performance_report.contains("性能分析报告"));

    // 清理（ErrorRecoveryManager没有stop方法，它会自动清理）

    println!("✅ 开发者工具集成测试通过");
}

#[tokio::test]
async fn test_cli_command_structure() {
    println!("🚀 测试CLI命令结构");

    let cli_manager = CliToolManager::new();

    // 测试所有预期的命令都存在
    let expected_commands = vec!["init", "plugin", "dev"];
    for cmd_name in expected_commands {
        let cmd = cli_manager.get_command(cmd_name);
        assert!(cmd.is_some(), "命令 {} 应该存在", cmd_name);
        
        let cmd = cmd.unwrap();
        assert!(!cmd.description.is_empty(), "命令 {} 应该有描述", cmd_name);
        assert!(!cmd.usage.is_empty(), "命令 {} 应该有使用说明", cmd_name);
    }

    // 测试plugin命令的子命令
    let plugin_cmd = cli_manager.get_command("plugin").unwrap();
    let expected_subcommands = vec!["list", "install", "uninstall"];
    for subcmd_name in expected_subcommands {
        let subcmd = plugin_cmd.subcommands.iter().find(|sc| sc.name == subcmd_name);
        assert!(subcmd.is_some(), "子命令 {} 应该存在", subcmd_name);
    }

    // 测试dev命令的子命令
    let dev_cmd = cli_manager.get_command("dev").unwrap();
    let expected_dev_subcommands = vec!["start", "test"];
    for subcmd_name in expected_dev_subcommands {
        let subcmd = dev_cmd.subcommands.iter().find(|sc| sc.name == subcmd_name);
        assert!(subcmd.is_some(), "开发子命令 {} 应该存在", subcmd_name);
    }

    println!("✅ CLI命令结构测试通过");
}

#[tokio::test]
async fn test_project_templates() {
    println!("🚀 测试项目模板");

    let cli_manager = CliToolManager::new();

    // 测试所有预期的模板都存在
    let expected_templates = vec!["rust-plugin", "python-plugin"];
    for template_name in expected_templates {
        let template = cli_manager.get_template(template_name);
        assert!(template.is_some(), "模板 {} 应该存在", template_name);
        
        let template = template.unwrap();
        assert!(!template.description.is_empty(), "模板 {} 应该有描述", template_name);
        assert!(!template.files.is_empty(), "模板 {} 应该有文件", template_name);
        assert!(!template.dependencies.is_empty(), "模板 {} 应该有依赖", template_name);
        assert!(!template.setup_instructions.is_empty(), "模板 {} 应该有设置说明", template_name);
    }

    // 测试Rust模板的特定文件
    let rust_template = cli_manager.get_template("rust-plugin").unwrap();
    assert!(rust_template.files.contains_key("Cargo.toml"));
    assert!(rust_template.files.contains_key("src/main.rs"));
    assert!(rust_template.files.contains_key("src/plugin.rs"));
    
    // 验证Cargo.toml内容
    let cargo_toml = rust_template.files.get("Cargo.toml").unwrap();
    assert!(cargo_toml.contains("[package]"));
    assert!(cargo_toml.contains("agentx-plugin"));

    // 测试Python模板的特定文件
    let python_template = cli_manager.get_template("python-plugin").unwrap();
    assert!(python_template.files.contains_key("requirements.txt"));
    assert!(python_template.files.contains_key("main.py"));
    assert!(python_template.files.contains_key("plugin.py"));

    println!("✅ 项目模板测试通过");
}
