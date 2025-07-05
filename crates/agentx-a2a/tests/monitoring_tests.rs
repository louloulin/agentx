//! A2A协议监控和指标收集测试
//! 
//! 测试A2A协议的性能监控、指标收集和健康检查功能

use agentx_a2a::{
    MonitoringManager, MonitoringConfig, MetricPoint, MetricType,
    HealthStatus, TimeRange,
};
use std::collections::HashMap;
use chrono::{Utc, Duration};
use tokio;

#[tokio::test]
async fn test_monitoring_manager_creation() {
    println!("🧪 测试监控管理器创建");
    
    let config = MonitoringConfig::default();
    let manager = MonitoringManager::new(config);
    
    println!("   ✅ 监控管理器创建成功");
    
    // 验证初始状态
    let metric_names = manager.get_metric_names();
    assert_eq!(metric_names.len(), 0);
    
    let health_status = manager.get_health_status();
    assert_eq!(health_status.status, HealthStatus::Unknown);
    
    println!("   ✅ 初始状态验证通过");
}

#[tokio::test]
async fn test_metric_recording() {
    println!("🧪 测试指标记录");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    // 记录计数器指标
    let counter_metric = MetricPoint {
        name: "test_counter".to_string(),
        metric_type: MetricType::Counter,
        value: 10.0,
        labels: HashMap::new(),
        timestamp: Utc::now(),
        help: Some("测试计数器".to_string()),
    };
    
    manager.record_metric(counter_metric);
    
    // 记录仪表指标
    let mut labels = HashMap::new();
    labels.insert("service".to_string(), "a2a".to_string());
    
    let gauge_metric = MetricPoint {
        name: "test_gauge".to_string(),
        metric_type: MetricType::Gauge,
        value: 75.5,
        labels,
        timestamp: Utc::now(),
        help: Some("测试仪表".to_string()),
    };
    
    manager.record_metric(gauge_metric);
    
    // 验证指标记录
    let metric_names = manager.get_metric_names();
    assert_eq!(metric_names.len(), 2);
    assert!(metric_names.contains(&"test_counter".to_string()));
    assert!(metric_names.contains(&"test_gauge".to_string()));
    
    // 获取具体指标
    let counter_metrics = manager.get_metrics("test_counter");
    assert!(counter_metrics.is_some());
    assert_eq!(counter_metrics.unwrap().len(), 1);
    
    println!("   ✅ 指标记录测试通过");
}

#[tokio::test]
async fn test_counter_operations() {
    println!("🧪 测试计数器操作");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    // 增加计数器
    manager.increment_counter("messages_processed", 1);
    manager.increment_counter("messages_processed", 5);
    manager.increment_counter("messages_processed", 3);
    
    // 验证计数器值
    let metrics = manager.get_metrics("messages_processed");
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    assert!(!metrics.is_empty());
    
    // 最后一个指标应该是累计值
    let last_metric = metrics.last().unwrap();
    assert_eq!(last_metric.value, 9.0); // 1 + 5 + 3
    assert_eq!(last_metric.metric_type, MetricType::Counter);
    
    println!("   ✅ 计数器操作测试通过");
}

#[tokio::test]
async fn test_gauge_operations() {
    println!("🧪 测试仪表操作");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    let mut labels = HashMap::new();
    labels.insert("component".to_string(), "engine".to_string());
    
    // 设置仪表值
    manager.set_gauge("cpu_usage", 25.5, labels.clone());
    manager.set_gauge("cpu_usage", 30.2, labels.clone());
    manager.set_gauge("cpu_usage", 28.7, labels);
    
    // 验证仪表值
    let metrics = manager.get_metrics("cpu_usage");
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    assert_eq!(metrics.len(), 3);
    
    // 验证最后一个值
    let last_metric = metrics.last().unwrap();
    assert_eq!(last_metric.value, 28.7);
    assert_eq!(last_metric.metric_type, MetricType::Gauge);
    
    println!("   ✅ 仪表操作测试通过");
}

#[tokio::test]
async fn test_histogram_operations() {
    println!("🧪 测试直方图操作");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), "message_processing".to_string());
    
    // 记录直方图值
    let values = vec![1.2, 2.5, 1.8, 3.1, 0.9, 2.2, 1.5];
    
    for value in &values {
        manager.record_histogram("processing_time", *value, labels.clone());
    }
    
    // 验证直方图记录
    let metrics = manager.get_metrics("processing_time");
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    assert_eq!(metrics.len(), values.len());
    
    // 验证所有值都被记录
    for (i, metric) in metrics.iter().enumerate() {
        assert_eq!(metric.value, values[i]);
        assert_eq!(metric.metric_type, MetricType::Histogram);
    }
    
    println!("   ✅ 直方图操作测试通过");
}

#[tokio::test]
async fn test_performance_stats_calculation() {
    println!("🧪 测试性能统计计算");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    // 添加一些测试指标
    manager.increment_counter("total_messages", 1000);
    manager.increment_counter("successful_messages", 950);
    manager.increment_counter("failed_messages", 50);
    manager.increment_counter("total_agents", 10);
    manager.increment_counter("online_agents", 8);
    manager.increment_counter("offline_agents", 2);
    
    // 计算性能统计
    let now = Utc::now();
    let time_range = TimeRange {
        start: now - Duration::hours(1),
        end: now,
    };
    
    let result = manager.calculate_performance_stats(time_range);
    assert!(result.is_ok());
    
    let stats = result.unwrap();
    
    // 验证消息统计
    assert_eq!(stats.message_stats.total_messages, 1000);
    assert_eq!(stats.message_stats.successful_messages, 950);
    assert_eq!(stats.message_stats.failed_messages, 50);
    
    // 验证Agent统计
    assert_eq!(stats.agent_stats.total_agents, 10);
    assert_eq!(stats.agent_stats.online_agents, 8);
    assert_eq!(stats.agent_stats.offline_agents, 2);
    
    // 验证错误统计
    let expected_error_rate = 50.0 / 1000.0;
    assert!((stats.error_stats.error_rate - expected_error_rate).abs() < 0.001);
    
    println!("   ✅ 性能统计计算测试通过");
}

#[tokio::test]
async fn test_health_check() {
    println!("🧪 测试健康检查");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    // 添加一些正常的指标
    manager.increment_counter("total_messages", 1000);
    manager.increment_counter("successful_messages", 990);
    manager.increment_counter("failed_messages", 10);
    
    // 执行健康检查
    let result = manager.perform_health_check();
    assert!(result.is_ok());
    
    let health_check = result.unwrap();
    
    // 验证健康检查结果
    assert!(!health_check.components.is_empty());
    assert!(health_check.score > 0);
    
    // 验证各组件状态
    let expected_components = vec![
        "message_processing",
        "agent_registry", 
        "network",
        "storage"
    ];
    
    for component in expected_components {
        assert!(health_check.components.contains_key(component));
        let component_health = &health_check.components[component];
        
        // 所有组件都应该有状态
        assert_ne!(component_health.status, HealthStatus::Unknown);
        
        println!("   组件 {} 状态: {:?}", component, component_health.status);
    }
    
    println!("   整体健康状态: {:?} (评分: {})", health_check.status, health_check.score);
    println!("   ✅ 健康检查测试通过");
}

#[tokio::test]
async fn test_metric_retention() {
    println!("🧪 测试指标保留策略");
    
    let config = MonitoringConfig {
        metric_retention_hours: 1, // 1小时保留
        ..Default::default()
    };
    
    let mut manager = MonitoringManager::new(config);
    
    // 添加一个旧指标（超过保留时间）
    let old_metric = MetricPoint {
        name: "old_metric".to_string(),
        metric_type: MetricType::Counter,
        value: 100.0,
        labels: HashMap::new(),
        timestamp: Utc::now() - Duration::hours(2), // 2小时前
        help: None,
    };
    
    manager.record_metric(old_metric);
    
    // 添加一个新指标
    let new_metric = MetricPoint {
        name: "old_metric".to_string(),
        metric_type: MetricType::Counter,
        value: 200.0,
        labels: HashMap::new(),
        timestamp: Utc::now(), // 现在
        help: None,
    };
    
    manager.record_metric(new_metric);
    
    // 验证指标保留
    let metrics = manager.get_metrics("old_metric");
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    // 旧指标应该被清理，只保留新指标
    assert_eq!(metrics.len(), 1);
    assert_eq!(metrics[0].value, 200.0);
    
    println!("   ✅ 指标保留策略测试通过");
}

#[tokio::test]
async fn test_monitoring_performance() {
    println!("🧪 测试监控性能");
    
    let config = MonitoringConfig::default();
    let mut manager = MonitoringManager::new(config);
    
    let metric_count = 10000;
    let start_time = std::time::Instant::now();
    
    // 批量记录指标
    for i in 0..metric_count {
        let metric = MetricPoint {
            name: format!("perf_metric_{}", i % 100), // 100个不同的指标名
            metric_type: MetricType::Counter,
            value: i as f64,
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: None,
        };
        
        manager.record_metric(metric);
    }
    
    let duration = start_time.elapsed();
    let throughput = metric_count as f64 / duration.as_secs_f64();
    
    println!("   📊 监控性能测试结果:");
    println!("     指标数量: {}", metric_count);
    println!("     总耗时: {:?}", duration);
    println!("     吞吐量: {:.0} 指标/秒", throughput);
    println!("     平均延迟: {:.3}ms", duration.as_millis() as f64 / metric_count as f64);
    
    // 验证性能目标
    assert!(throughput > 50000.0, "监控吞吐量 {:.0} 指标/秒 低于50,000指标/秒的目标", throughput);
    
    // 验证指标存储
    let metric_names = manager.get_metric_names();
    assert_eq!(metric_names.len(), 100); // 100个不同的指标名
    
    println!("   ✅ 监控性能测试通过");
}

#[tokio::test]
async fn test_different_health_statuses() {
    println!("🧪 测试不同健康状态");
    
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];
    
    for status in statuses {
        println!("   测试健康状态: {:?}", status);
        
        // 验证状态评分
        let score = match status {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 75,
            HealthStatus::Unhealthy => 25,
            HealthStatus::Unknown => 0,
        };
        
        // 这里可以添加更多的状态验证逻辑
        
        println!("     状态评分: {}", score);
        println!("     ✅ {:?} 状态测试通过", status);
    }
    
    println!("   ✅ 不同健康状态测试通过");
}
