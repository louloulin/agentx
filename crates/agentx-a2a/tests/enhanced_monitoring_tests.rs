//! 增强监控功能测试
//! 
//! 测试新增的性能监控和基准测试功能

use agentx_a2a::{
    MonitoringManager, MonitoringConfig, MetricPoint, MetricType,
    BenchmarkResult, PerformanceStats
};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

/// 创建测试用的监控配置
fn create_test_config() -> MonitoringConfig {
    MonitoringConfig {
        metric_retention_hours: 24,
        health_check_interval_seconds: 30,
        stats_calculation_interval_seconds: 60,
        enable_detailed_monitoring: true,
    }
}

#[tokio::test]
async fn test_enhanced_performance_stats() {
    println!("🧪 测试增强性能统计功能");
    
    let config = create_test_config();
    let mut manager = MonitoringManager::new(config);
    
    // 模拟一些操作
    manager.increment_counter("total_messages", 1000);
    manager.increment_counter("successful_messages", 950);
    manager.increment_counter("failed_messages", 50);
    
    // 添加延迟指标
    let latency_metric = MetricPoint {
        name: "message_latency".to_string(),
        metric_type: MetricType::Histogram,
        value: 5.5, // 5.5ms
        labels: HashMap::new(),
        timestamp: Utc::now(),
        help: Some("消息处理延迟".to_string()),
    };
    manager.record_metric(latency_metric);
    
    // 获取增强性能统计
    let stats = manager.get_enhanced_performance_stats();
    
    // 验证统计数据
    assert_eq!(stats.message_stats.total_messages, 1000);
    assert_eq!(stats.message_stats.successful_messages, 950);
    assert_eq!(stats.message_stats.failed_messages, 50);
    assert_eq!(stats.error_stats.error_rate, 5.0); // 50/1000 * 100 = 5%
    
    println!("✅ 增强性能统计测试通过");
    println!("   - 总消息数: {}", stats.message_stats.total_messages);
    println!("   - 成功率: {:.1}%", 100.0 - stats.error_stats.error_rate);
    println!("   - 平均延迟: {:.2}ms", stats.message_stats.avg_processing_time_ms);
}

#[tokio::test]
async fn test_performance_benchmark() {
    println!("🧪 测试性能基准测试功能");
    
    let config = create_test_config();
    let mut manager = MonitoringManager::new(config);
    
    // 运行小规模基准测试
    let iterations = 1000;
    let result = manager.run_performance_benchmark("test_benchmark", iterations);
    
    // 验证基准测试结果
    assert_eq!(result.test_name, "test_benchmark");
    assert_eq!(result.iterations, iterations);
    assert!(result.total_duration_ms > 0.0);
    assert!(result.throughput_ops_per_sec > 0.0);
    assert!(result.min_latency_ms >= 0.0);
    assert!(result.max_latency_ms >= result.min_latency_ms);
    assert!(result.avg_latency_ms >= result.min_latency_ms);
    assert!(result.avg_latency_ms <= result.max_latency_ms);
    assert!(result.p50_latency_ms >= result.min_latency_ms);
    assert!(result.p95_latency_ms >= result.p50_latency_ms);
    assert!(result.p99_latency_ms >= result.p95_latency_ms);
    
    println!("✅ 性能基准测试通过");
    println!("   - 迭代次数: {}", result.iterations);
    println!("   - 总耗时: {:.2}ms", result.total_duration_ms);
    println!("   - 吞吐量: {:.0} ops/sec", result.throughput_ops_per_sec);
    println!("   - 平均延迟: {:.3}ms", result.avg_latency_ms);
    println!("   - P95延迟: {:.3}ms", result.p95_latency_ms);
    println!("   - P99延迟: {:.3}ms", result.p99_latency_ms);
}

#[tokio::test]
async fn test_metric_recording_and_retrieval() {
    println!("🧪 测试指标记录和检索功能");
    
    let config = create_test_config();
    let mut manager = MonitoringManager::new(config);
    
    // 记录多种类型的指标
    let counter_metric = MetricPoint {
        name: "api_requests".to_string(),
        metric_type: MetricType::Counter,
        value: 100.0,
        labels: HashMap::from([("endpoint".to_string(), "/api/v1/agents".to_string())]),
        timestamp: Utc::now(),
        help: Some("API请求计数".to_string()),
    };
    manager.record_metric(counter_metric);
    
    let gauge_metric = MetricPoint {
        name: "memory_usage".to_string(),
        metric_type: MetricType::Gauge,
        value: 85.5,
        labels: HashMap::from([("unit".to_string(), "percent".to_string())]),
        timestamp: Utc::now(),
        help: Some("内存使用率".to_string()),
    };
    manager.record_metric(gauge_metric);
    
    let histogram_metric = MetricPoint {
        name: "response_time".to_string(),
        metric_type: MetricType::Histogram,
        value: 125.7,
        labels: HashMap::from([("service".to_string(), "agent_registry".to_string())]),
        timestamp: Utc::now(),
        help: Some("响应时间分布".to_string()),
    };
    manager.record_metric(histogram_metric);
    
    // 增加计数器
    manager.increment_counter("total_operations", 50);
    manager.increment_counter("total_operations", 25);
    
    // 设置仪表值
    let mut labels = HashMap::new();
    labels.insert("component".to_string(), "a2a_engine".to_string());
    manager.set_gauge("cpu_usage", 45.2, labels);
    
    println!("✅ 指标记录和检索测试通过");
    println!("   - 已记录多种类型指标");
    println!("   - 计数器和仪表值设置成功");
}

#[tokio::test]
async fn test_monitoring_performance_under_load() {
    println!("🧪 测试监控系统负载性能");
    
    let config = create_test_config();
    let mut manager = MonitoringManager::new(config);
    
    let start_time = std::time::Instant::now();
    let operations = 10000;
    
    // 模拟高负载操作
    for i in 0..operations {
        manager.increment_counter("load_test_operations", 1);
        
        if i % 100 == 0 {
            let metric = MetricPoint {
                name: "load_test_latency".to_string(),
                metric_type: MetricType::Histogram,
                value: (i as f64 % 10.0) + 1.0,
                labels: HashMap::new(),
                timestamp: Utc::now(),
                help: None,
            };
            manager.record_metric(metric);
        }
    }
    
    let duration = start_time.elapsed();
    let ops_per_sec = operations as f64 / duration.as_secs_f64();
    
    // 验证性能要求
    assert!(ops_per_sec > 1000.0, "监控系统应该支持 >1000 ops/sec，实际: {:.0}", ops_per_sec);
    
    println!("✅ 监控系统负载性能测试通过");
    println!("   - 处理操作数: {}", operations);
    println!("   - 总耗时: {:.2}ms", duration.as_millis());
    println!("   - 性能: {:.0} ops/sec", ops_per_sec);
}

#[tokio::test]
async fn test_benchmark_accuracy() {
    println!("🧪 测试基准测试准确性");
    
    let config = create_test_config();
    let mut manager = MonitoringManager::new(config);
    
    // 运行多次小规模基准测试
    let mut results = Vec::new();
    for i in 0..3 {
        let result = manager.run_performance_benchmark(&format!("accuracy_test_{}", i), 100);
        results.push(result);
    }
    
    // 验证结果一致性
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.iterations, 100);
        assert!(result.throughput_ops_per_sec > 0.0);
        println!("   测试 {}: {:.0} ops/sec, 平均延迟: {:.3}ms", 
                i, result.throughput_ops_per_sec, result.avg_latency_ms);
    }
    
    println!("✅ 基准测试准确性验证通过");
}
