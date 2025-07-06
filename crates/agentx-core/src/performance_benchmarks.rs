//! 真实性能基准测试模块
//! 
//! 提供真实的网络通信和消息处理性能测试

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use agentx_a2a::{A2AMessage, MessageRole, A2AResult, A2AError};
use agentx_router::MessageRouter;
use tracing::info;

/// 性能基准测试管理器
#[derive(Debug)]
pub struct PerformanceBenchmarkManager {
    /// 基准测试配置
    config: BenchmarkConfig,
    /// 测试结果历史
    results: Arc<RwLock<Vec<BenchmarkResult>>>,
    /// 当前运行的测试
    running_tests: Arc<RwLock<Vec<RunningBenchmark>>>,
}

/// 基准测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// 测试持续时间（秒）
    pub duration_secs: u64,
    /// 并发连接数
    pub concurrent_connections: usize,
    /// 消息大小（字节）
    pub message_size: usize,
    /// 目标QPS
    pub target_qps: u64,
    /// 预热时间（秒）
    pub warmup_secs: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            duration_secs: 60,
            concurrent_connections: 10,
            message_size: 1024,
            target_qps: 1000,
            warmup_secs: 10,
        }
    }
}

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 测试名称
    pub test_name: String,
    /// 测试开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 测试持续时间
    pub duration: Duration,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 50th百分位延迟（毫秒）
    pub p50_latency_ms: f64,
    /// 95th百分位延迟（毫秒）
    pub p95_latency_ms: f64,
    /// 99th百分位延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f64,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f64,
    /// 吞吐量（请求/秒）
    pub throughput_rps: f64,
    /// 错误率
    pub error_rate: f64,
    /// CPU使用率
    pub cpu_usage_percent: f64,
    /// 内存使用量（MB）
    pub memory_usage_mb: f64,
}

/// 运行中的基准测试
#[derive(Debug)]
pub struct RunningBenchmark {
    /// 测试名称
    pub test_name: String,
    /// 开始时间
    pub start_time: Instant,
    /// 配置
    pub config: BenchmarkConfig,
    /// 延迟记录
    pub latencies: Vec<Duration>,
    /// 成功计数
    pub success_count: u64,
    /// 失败计数
    pub failure_count: u64,
}

impl PerformanceBenchmarkManager {
    /// 创建新的性能基准测试管理器
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(Vec::new())),
            running_tests: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 运行消息路由性能测试
    pub async fn run_message_routing_benchmark(&self, router: Arc<MessageRouter>) -> A2AResult<BenchmarkResult> {
        let test_name = "message_routing_benchmark".to_string();
        info!("开始消息路由性能测试: {}", test_name);

        let start_time = Instant::now();
        let mut latencies = Vec::new();
        let mut success_count = 0u64;
        let mut failure_count = 0u64;

        // 预热阶段
        info!("预热阶段开始，持续{}秒", self.config.warmup_secs);
        let warmup_end = start_time + Duration::from_secs(self.config.warmup_secs);
        while Instant::now() < warmup_end {
            let message = self.create_test_message()?;
            let _ = router.route_message(message).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        info!("正式测试开始，持续{}秒", self.config.duration_secs);
        let test_end = Instant::now() + Duration::from_secs(self.config.duration_secs);

        // 并发测试
        let mut handles = Vec::new();
        for _i in 0..self.config.concurrent_connections {
            let router_clone = router.clone();
            let test_end_clone = test_end;
            let message_size = self.config.message_size;
            
            let handle = tokio::spawn(async move {
                let mut local_latencies = Vec::new();
                let mut local_success = 0u64;
                let mut local_failure = 0u64;

                while Instant::now() < test_end_clone {
                    let message = Self::create_test_message_with_size(message_size).unwrap();
                    let request_start = Instant::now();
                    
                    match router_clone.route_message(message).await {
                        Ok(_) => {
                            local_success += 1;
                            local_latencies.push(request_start.elapsed());
                        }
                        Err(_) => {
                            local_failure += 1;
                        }
                    }

                    // 控制QPS
                    tokio::time::sleep(Duration::from_micros(1000)).await;
                }

                (local_latencies, local_success, local_failure)
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let (mut local_latencies, local_success, local_failure) = handle.await
                .map_err(|e| A2AError::internal(format!("任务执行失败: {}", e)))?;
            
            latencies.append(&mut local_latencies);
            success_count += local_success;
            failure_count += local_failure;
        }

        let total_duration = start_time.elapsed();
        let result = self.calculate_benchmark_result(
            test_name,
            total_duration,
            latencies,
            success_count,
            failure_count,
        ).await?;

        // 保存结果
        self.results.write().await.push(result.clone());
        
        info!("消息路由性能测试完成: 吞吐量 {:.2} RPS, 平均延迟 {:.2} ms", 
              result.throughput_rps, result.avg_latency_ms);

        Ok(result)
    }

    /// 创建测试消息
    fn create_test_message(&self) -> A2AResult<A2AMessage> {
        Self::create_test_message_with_size(self.config.message_size)
    }

    /// 创建指定大小的测试消息
    fn create_test_message_with_size(size: usize) -> A2AResult<A2AMessage> {
        let content = "x".repeat(size.saturating_sub(100)); // 预留100字节给其他字段
        Ok(A2AMessage::user_message(content))
    }

    /// 计算基准测试结果
    async fn calculate_benchmark_result(
        &self,
        test_name: String,
        duration: Duration,
        mut latencies: Vec<Duration>,
        success_count: u64,
        failure_count: u64,
    ) -> A2AResult<BenchmarkResult> {
        if latencies.is_empty() {
            return Err(A2AError::internal("没有有效的延迟数据"));
        }

        // 排序延迟数据用于计算百分位数
        latencies.sort();

        let total_requests = success_count + failure_count;
        let avg_latency_ms = latencies.iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .sum::<f64>() / latencies.len() as f64;

        let p50_latency_ms = self.calculate_percentile(&latencies, 50.0);
        let p95_latency_ms = self.calculate_percentile(&latencies, 95.0);
        let p99_latency_ms = self.calculate_percentile(&latencies, 99.0);
        
        let max_latency_ms = latencies.last().unwrap().as_secs_f64() * 1000.0;
        let min_latency_ms = latencies.first().unwrap().as_secs_f64() * 1000.0;
        
        let throughput_rps = success_count as f64 / duration.as_secs_f64();
        let error_rate = failure_count as f64 / total_requests as f64 * 100.0;

        // 获取系统资源使用情况
        let (cpu_usage, memory_usage) = self.get_system_usage().await;

        Ok(BenchmarkResult {
            test_name,
            start_time: chrono::Utc::now(),
            duration,
            total_requests,
            successful_requests: success_count,
            failed_requests: failure_count,
            avg_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            min_latency_ms,
            throughput_rps,
            error_rate,
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
        })
    }

    /// 计算百分位数
    fn calculate_percentile(&self, latencies: &[Duration], percentile: f64) -> f64 {
        let index = (latencies.len() as f64 * percentile / 100.0) as usize;
        let index = index.min(latencies.len() - 1);
        latencies[index].as_secs_f64() * 1000.0
    }

    /// 获取系统资源使用情况
    async fn get_system_usage(&self) -> (f64, f64) {
        // 简化实现，实际应该使用系统API获取真实数据
        (0.0, 0.0)
    }

    /// 获取所有测试结果
    pub async fn get_results(&self) -> Vec<BenchmarkResult> {
        self.results.read().await.clone()
    }

    /// 清除测试结果
    pub async fn clear_results(&self) {
        self.results.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_manager_creation() {
        let config = BenchmarkConfig::default();
        let manager = PerformanceBenchmarkManager::new(config);
        
        let results = manager.get_results().await;
        assert!(results.is_empty());
    }

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.duration_secs, 60);
        assert_eq!(config.concurrent_connections, 10);
        assert_eq!(config.message_size, 1024);
        assert_eq!(config.target_qps, 1000);
        assert_eq!(config.warmup_secs, 10);
    }

    #[test]
    fn test_create_test_message() {
        let message = PerformanceBenchmarkManager::create_test_message_with_size(100).unwrap();
        assert!(matches!(message.role, MessageRole::User));
    }
}
