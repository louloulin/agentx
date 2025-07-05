//! AgentX错误恢复集成测试
//! 
//! 验证错误处理和恢复机制的完整功能，包括：
//! - 错误检测和报告
//! - 自动恢复策略
//! - 断路器模式
//! - 组件健康监控

use std::time::Duration;
use std::sync::Arc;
use tokio::time::sleep;
use agentx_core::{
    ErrorRecoveryManager, ErrorRecoveryConfig, ComponentStatus, 
    ErrorType, RecoveryStrategy, AgentXCore
};

/// 错误恢复集成测试套件
struct ErrorRecoveryIntegrationTests {
    manager: ErrorRecoveryManager,
    config: ErrorRecoveryConfig,
}

impl ErrorRecoveryIntegrationTests {
    /// 创建新的测试套件
    fn new() -> Self {
        let config = ErrorRecoveryConfig {
            max_retries: 3,
            retry_interval_ms: 100,
            backoff_factor: 1.5,
            max_backoff_ms: 1000,
            health_check_interval_secs: 1,
            failure_threshold: 2,
            recovery_threshold: 2,
            circuit_breaker_timeout_secs: 5,
        };
        
        let manager = ErrorRecoveryManager::new(config.clone());
        
        Self { manager, config }
    }

    /// 运行完整的错误恢复集成测试
    async fn run_full_test_suite(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 开始错误恢复集成测试");

        // 启动错误恢复管理器
        self.manager.start().await?;

        // 1. 测试组件注册和健康监控
        println!("\n📋 测试组件注册和健康监控...");
        self.test_component_registration().await?;

        // 2. 测试错误检测和报告
        println!("\n🚨 测试错误检测和报告...");
        self.test_error_detection().await?;

        // 3. 测试自动恢复策略
        println!("\n🔄 测试自动恢复策略...");
        self.test_recovery_strategies().await?;

        // 4. 测试断路器模式
        println!("\n⚡ 测试断路器模式...");
        self.test_circuit_breaker().await?;

        // 5. 测试故障转移
        println!("\n🔀 测试故障转移...");
        self.test_failover_mechanism().await?;

        // 6. 测试性能影响
        println!("\n⚡ 测试性能影响...");
        self.test_performance_impact().await?;

        println!("\n✅ 错误恢复集成测试完成");
        Ok(())
    }

    /// 测试组件注册和健康监控
    async fn test_component_registration(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 注册测试组件
        self.manager.register_component("test_service_1", RecoveryStrategy::Retry).await;
        self.manager.register_component("test_service_2", RecoveryStrategy::Restart).await;
        self.manager.register_component("test_service_3", RecoveryStrategy::Failover).await;

        // 验证组件已注册
        let health = self.manager.get_component_health("test_service_1").await;
        assert!(health.is_some(), "组件应该已注册");
        assert_eq!(health.unwrap().status, ComponentStatus::Healthy);

        // 报告成功事件
        self.manager.report_success("test_service_1", 50.0).await;
        self.manager.report_success("test_service_2", 75.0).await;

        // 验证健康状态更新
        let health = self.manager.get_component_health("test_service_1").await.unwrap();
        assert_eq!(health.consecutive_successes, 1);
        assert_eq!(health.total_successes, 1);
        assert!(health.avg_response_time_ms > 0.0);

        println!("   ✅ 组件注册和健康监控测试通过");
        Ok(())
    }

    /// 测试错误检测和报告
    async fn test_error_detection(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 报告网络错误
        self.manager.report_error(
            "test_service_1", 
            ErrorType::Network, 
            "连接超时", 
            3
        ).await;

        // 验证错误被记录
        let error_history = self.manager.get_error_history(Some(10)).await;
        assert!(!error_history.is_empty(), "应该有错误记录");
        
        let latest_error = &error_history[0];
        assert_eq!(latest_error.component, "test_service_1");
        assert_eq!(latest_error.error_type, ErrorType::Network);

        // 验证组件状态更新
        let health = self.manager.get_component_health("test_service_1").await.unwrap();
        assert_eq!(health.consecutive_failures, 1);
        assert_eq!(health.total_failures, 1);

        // 报告更多错误以触发状态变化
        self.manager.report_error(
            "test_service_1", 
            ErrorType::Timeout, 
            "请求超时", 
            4
        ).await;

        let health = self.manager.get_component_health("test_service_1").await.unwrap();
        assert_eq!(health.status, ComponentStatus::Failed);

        println!("   ✅ 错误检测和报告测试通过");
        Ok(())
    }

    /// 测试自动恢复策略
    async fn test_recovery_strategies(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 测试重试策略
        self.manager.report_error(
            "test_service_2", 
            ErrorType::ResourceExhausted, 
            "资源不足", 
            5
        ).await;

        // 等待恢复动作执行
        sleep(Duration::from_millis(200)).await;

        // 验证恢复动作被记录
        let recovery_history = self.manager.get_recovery_history(Some(10)).await;
        let retry_actions: Vec<_> = recovery_history.iter()
            .filter(|a| a.component == "test_service_2" && a.strategy == RecoveryStrategy::Restart)
            .collect();
        
        assert!(!retry_actions.is_empty(), "应该有恢复动作记录");

        // 模拟恢复成功
        self.manager.report_success("test_service_2", 100.0).await;
        self.manager.report_success("test_service_2", 95.0).await;

        let health = self.manager.get_component_health("test_service_2").await.unwrap();
        assert_eq!(health.status, ComponentStatus::Healthy);

        println!("   ✅ 自动恢复策略测试通过");
        Ok(())
    }

    /// 测试断路器模式
    async fn test_circuit_breaker(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 注册新的测试组件
        self.manager.register_component("circuit_test", RecoveryStrategy::Retry).await;

        // 连续报告多个错误以触发断路器
        for i in 1..=5 {
            self.manager.report_error(
                "circuit_test", 
                ErrorType::Internal, 
                &format!("内部错误 {}", i), 
                4
            ).await;
            sleep(Duration::from_millis(50)).await;
        }

        // 验证组件状态为失败
        let health = self.manager.get_component_health("circuit_test").await.unwrap();
        assert_eq!(health.status, ComponentStatus::Failed);
        assert!(health.consecutive_failures >= self.config.failure_threshold);

        // 等待一段时间后尝试恢复
        sleep(Duration::from_millis(200)).await;

        // 报告成功事件以测试恢复
        for _ in 0..3 {
            self.manager.report_success("circuit_test", 80.0).await;
            sleep(Duration::from_millis(50)).await;
        }

        let health = self.manager.get_component_health("circuit_test").await.unwrap();
        assert_eq!(health.status, ComponentStatus::Healthy);

        println!("   ✅ 断路器模式测试通过");
        Ok(())
    }

    /// 测试故障转移机制
    async fn test_failover_mechanism(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 注册主服务和备用服务
        self.manager.register_component("primary_service", RecoveryStrategy::Failover).await;
        self.manager.register_component("backup_service", RecoveryStrategy::Retry).await;

        // 模拟主服务故障
        self.manager.report_error(
            "primary_service", 
            ErrorType::DependencyFailure, 
            "依赖服务不可用", 
            5
        ).await;

        // 等待故障转移执行
        sleep(Duration::from_millis(100)).await;

        // 验证故障转移动作
        let recovery_history = self.manager.get_recovery_history(Some(10)).await;
        let failover_actions: Vec<_> = recovery_history.iter()
            .filter(|a| a.component == "primary_service" && a.strategy == RecoveryStrategy::Failover)
            .collect();
        
        assert!(!failover_actions.is_empty(), "应该有故障转移动作");

        // 验证备用服务正常
        self.manager.report_success("backup_service", 120.0).await;
        let backup_health = self.manager.get_component_health("backup_service").await.unwrap();
        assert_eq!(backup_health.status, ComponentStatus::Healthy);

        println!("   ✅ 故障转移机制测试通过");
        Ok(())
    }

    /// 测试性能影响
    async fn test_performance_impact(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::time::Instant;

        // 注册性能测试组件
        self.manager.register_component("perf_test", RecoveryStrategy::Retry).await;

        // 测试大量成功事件的性能
        let start = Instant::now();
        for i in 0..1000 {
            self.manager.report_success("perf_test", 50.0 + (i % 100) as f64).await;
        }
        let success_duration = start.elapsed();

        // 测试错误事件的性能
        let start = Instant::now();
        for i in 0..100 {
            self.manager.report_error(
                "perf_test", 
                ErrorType::Network, 
                &format!("测试错误 {}", i), 
                2
            ).await;
        }
        let error_duration = start.elapsed();

        // 验证性能指标
        println!("   📊 性能指标:");
        println!("      成功事件处理: {:.2}ms (1000次)", success_duration.as_millis());
        println!("      错误事件处理: {:.2}ms (100次)", error_duration.as_millis());
        
        // 验证平均处理时间在合理范围内
        let avg_success_time = success_duration.as_micros() / 1000;
        let avg_error_time = error_duration.as_micros() / 100;
        
        assert!(avg_success_time < 1000, "成功事件平均处理时间应小于1ms");
        assert!(avg_error_time < 5000, "错误事件平均处理时间应小于5ms");

        // 验证最终状态
        let health = self.manager.get_component_health("perf_test").await.unwrap();
        assert_eq!(health.total_successes, 1000);
        assert_eq!(health.total_failures, 100);

        println!("   ✅ 性能影响测试通过");
        Ok(())
    }
}

/// 测试AgentX核心集成错误恢复
async fn test_agentx_core_error_recovery() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 测试AgentX核心错误恢复集成");

    let mut core = AgentXCore::new();
    core.initialize().await?;

    // 获取错误恢复管理器
    let error_recovery = core.error_recovery();

    // 验证核心组件已注册
    let components = error_recovery.get_all_component_health().await;
    assert!(components.contains_key("protocol_compat"), "协议兼容组件应该已注册");
    assert!(components.contains_key("cloud_native"), "云原生组件应该已注册");
    assert!(components.contains_key("developer_ecosystem"), "开发者生态组件应该已注册");

    // 模拟组件错误
    error_recovery.report_error(
        "protocol_compat", 
        ErrorType::Configuration, 
        "配置错误", 
        3
    ).await;

    // 验证错误被正确处理
    let health = error_recovery.get_component_health("protocol_compat").await.unwrap();
    assert_eq!(health.consecutive_failures, 1);

    // 模拟恢复
    error_recovery.report_success("protocol_compat", 75.0).await;
    error_recovery.report_success("protocol_compat", 80.0).await;

    let health = error_recovery.get_component_health("protocol_compat").await.unwrap();
    assert_eq!(health.status, ComponentStatus::Healthy);

    println!("   ✅ AgentX核心错误恢复集成测试通过");
    Ok(())
}

#[tokio::test]
async fn test_error_recovery_integration() {
    let test_suite = ErrorRecoveryIntegrationTests::new();
    
    let result = test_suite.run_full_test_suite().await;
    assert!(result.is_ok(), "错误恢复集成测试应该成功: {:?}", result);
}

#[tokio::test]
async fn test_agentx_core_integration() {
    let result = test_agentx_core_error_recovery().await;
    assert!(result.is_ok(), "AgentX核心集成测试应该成功: {:?}", result);
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    let manager = ErrorRecoveryManager::new(ErrorRecoveryConfig::default());
    manager.start().await.unwrap();

    // 注册多个组件
    for i in 0..10 {
        manager.register_component(&format!("service_{}", i), RecoveryStrategy::Retry).await;
    }

    // 并发报告错误和成功事件
    let mut handles = Vec::new();
    let manager = Arc::new(manager);

    for i in 0..10 {
        let manager_clone = manager.clone();
        let service_name = format!("service_{}", i);

        let handle = tokio::spawn(async move {
            // 每个服务报告一些错误和成功事件
            for j in 0..5 {
                if j % 2 == 0 {
                    manager_clone.report_success(&service_name, 100.0).await;
                } else {
                    manager_clone.report_error(
                        &service_name,
                        ErrorType::Network,
                        "网络错误",
                        2
                    ).await;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    // 验证所有组件状态
    let all_health = manager.get_all_component_health().await;
    assert_eq!(all_health.len(), 10, "应该有10个组件");

    for (service_name, health) in all_health {
        assert!(health.total_successes > 0, "服务 {} 应该有成功记录", service_name);
        assert!(health.total_failures > 0, "服务 {} 应该有失败记录", service_name);
    }

    println!("✅ 并发错误处理测试通过");
}
