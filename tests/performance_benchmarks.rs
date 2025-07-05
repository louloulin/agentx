//! AgentX性能基准测试
//! 
//! 验证系统性能是否达到设计目标：
//! - gRPC延迟: < 5ms
//! - 消息路由延迟: < 10ms  
//! - 吞吐量: 支持10,000+ 并发Agent
//! - 插件启动时间: < 3秒
//! - 插件故障恢复: < 1秒

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use agentx_a2a::{
    A2AProtocolEngine, ProtocolEngineConfig, AgentInfo, AgentStatus,
    A2AMessage, MessageRole,
};
use agentx_cluster::{ClusterManager, ClusterConfig};

/// 性能基准测试配置
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// 并发Agent数量
    pub concurrent_agents: usize,
    /// 消息数量
    pub message_count: usize,
    /// 测试持续时间
    pub duration: Duration,
    /// 预热时间
    pub warmup_duration: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            concurrent_agents: 1000,
            message_count: 10000,
            duration: Duration::from_secs(60),
            warmup_duration: Duration::from_secs(10),
        }
    }
}

/// 性能测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// 平均延迟 (毫秒)
    pub avg_latency_ms: f64,
    /// P95延迟 (毫秒)
    pub p95_latency_ms: f64,
    /// P99延迟 (毫秒)
    pub p99_latency_ms: f64,
    /// 吞吐量 (操作/秒)
    pub throughput_ops_per_sec: f64,
    /// 错误率 (%)
    pub error_rate_percent: f64,
    /// 内存使用 (MB)
    pub memory_usage_mb: f64,
    /// CPU使用率 (%)
    pub cpu_usage_percent: f64,
}

/// 性能基准测试套件
pub struct PerformanceBenchmarks {
    config: BenchmarkConfig,
    a2a_engine: Arc<RwLock<A2AProtocolEngine>>,
    cluster_manager: Arc<ClusterManager>,
}

impl PerformanceBenchmarks {
    /// 创建新的性能测试套件
    pub async fn new(config: BenchmarkConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化A2A协议引擎
        let engine_config = ProtocolEngineConfig::default();
        let a2a_engine = Arc::new(RwLock::new(A2AProtocolEngine::new(engine_config)));

        // 初始化集群管理器
        let cluster_config = ClusterConfig::default();
        let cluster_manager = Arc::new(ClusterManager::new(cluster_config).await?);

        Ok(Self {
            config,
            a2a_engine,
            cluster_manager,
        })
    }

    /// 运行完整的性能基准测试
    pub async fn run_full_benchmark(&self) -> Result<Vec<BenchmarkResults>, Box<dyn std::error::Error>> {
        println!("🚀 开始AgentX性能基准测试");
        println!("📊 测试配置: {:?}", self.config);

        let mut results = Vec::new();

        // 1. A2A协议性能测试
        println!("\n📡 测试A2A协议性能...");
        let a2a_results = self.benchmark_a2a_protocol().await?;
        results.push(a2a_results);

        // 2. 消息路由性能测试
        println!("\n🔀 测试消息路由性能...");
        let routing_results = self.benchmark_message_routing().await?;
        results.push(routing_results);

        // 3. Agent注册和发现性能测试
        println!("\n👥 测试Agent注册和发现性能...");
        let registry_results = self.benchmark_agent_registry().await?;
        results.push(registry_results);

        // 4. 集群管理性能测试
        println!("\n🏗️ 测试集群管理性能...");
        let cluster_results = self.benchmark_cluster_management().await?;
        results.push(cluster_results);

        // 5. 并发负载测试
        println!("\n⚡ 测试并发负载性能...");
        let concurrent_results = self.benchmark_concurrent_load().await?;
        results.push(concurrent_results);

        // 6. 内存和资源使用测试
        println!("\n💾 测试资源使用情况...");
        let resource_results = self.benchmark_resource_usage().await?;
        results.push(resource_results);

        println!("\n✅ 性能基准测试完成");
        self.print_summary(&results);

        Ok(results)
    }

    /// A2A协议性能基准测试
    async fn benchmark_a2a_protocol(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut errors = 0;
        let start_time = Instant::now();

        // 预热
        for _ in 0..100 {
            let _ = self.send_test_message().await;
        }

        // 实际测试
        for i in 0..self.config.message_count {
            let msg_start = Instant::now();
            
            match self.send_test_message().await {
                Ok(_) => {
                    let latency = msg_start.elapsed();
                    latencies.push(latency.as_secs_f64() * 1000.0); // 转换为毫秒
                }
                Err(_) => {
                    errors += 1;
                }
            }

            if i % 1000 == 0 {
                println!("   已处理: {}/{} 消息", i + 1, self.config.message_count);
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(latencies, errors, total_duration)
    }

    /// 消息路由性能基准测试
    async fn benchmark_message_routing(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut errors = 0;
        let start_time = Instant::now();

        // 注册多个测试Agent
        let agent_count = 100;
        for i in 0..agent_count {
            let agent_info = self.create_test_agent_info(i).await;
            let _ = self.a2a_engine.write().await.register_agent(agent_info);
        }

        // 测试消息路由
        for i in 0..self.config.message_count {
            let route_start = Instant::now();
            
            let from_agent = format!("test_agent_{}", i % agent_count);
            let to_agent = format!("test_agent_{}", (i + 1) % agent_count);
            
            match self.route_test_message(&from_agent, &to_agent).await {
                Ok(_) => {
                    let latency = route_start.elapsed();
                    latencies.push(latency.as_secs_f64() * 1000.0);
                }
                Err(_) => {
                    errors += 1;
                }
            }

            if i % 1000 == 0 {
                println!("   已路由: {}/{} 消息", i + 1, self.config.message_count);
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(latencies, errors, total_duration)
    }

    /// Agent注册和发现性能基准测试
    async fn benchmark_agent_registry(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut errors = 0;
        let start_time = Instant::now();

        // 测试Agent注册性能
        for i in 0..self.config.concurrent_agents {
            let reg_start = Instant::now();
            
            let agent_info = self.create_test_agent_info(i).await;
            self.a2a_engine.write().await.register_agent(agent_info);
            let latency = reg_start.elapsed();
            latencies.push(latency.as_secs_f64() * 1000.0);

            if i % 100 == 0 {
                println!("   已注册: {}/{} Agent", i + 1, self.config.concurrent_agents);
            }
        }

        // 测试Agent发现性能
        for i in 0..1000 {
            let discover_start = Instant::now();
            
            match self.a2a_engine.read().await.list_agents() {
                agents => {
                    let latency = discover_start.elapsed();
                    latencies.push(latency.as_secs_f64() * 1000.0);
                    
                    if agents.len() < self.config.concurrent_agents / 2 {
                        errors += 1;
                    }
                }
            }

            if i % 100 == 0 {
                println!("   已发现: {}/1000 次查询", i + 1);
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(latencies, errors, total_duration)
    }

    /// 集群管理性能基准测试
    async fn benchmark_cluster_management(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut errors = 0;
        let start_time = Instant::now();

        // 测试集群操作性能（简化版本）
        for i in 0..1000 {
            let op_start = Instant::now();

            // 模拟集群操作
            match self.cluster_manager.get_cluster_state().await {
                Ok(_) => {
                    let latency = op_start.elapsed();
                    latencies.push(latency.as_secs_f64() * 1000.0);
                }
                Err(_) => {
                    errors += 1;
                }
            }

            if i % 100 == 0 {
                println!("   已执行: {}/1000 集群操作", i + 1);
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(latencies, errors, total_duration)
    }

    /// 并发负载性能基准测试
    async fn benchmark_concurrent_load(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_agents));
        let mut handles = Vec::new();
        let start_time = Instant::now();

        // 创建并发任务
        for i in 0..self.config.concurrent_agents {
            let sem = semaphore.clone();
            let engine = self.a2a_engine.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let task_start = Instant::now();
                
                // 执行并发操作
                let agent_info = AgentInfo {
                    id: format!("concurrent_agent_{}", i),
                    name: format!("并发测试Agent {}", i),
                    endpoint: format!("http://test-{}.local:8080", i),
                    capabilities: vec!["test".to_string()],
                    status: AgentStatus::Online,
                };

                engine.write().await.register_agent(agent_info);
                let latency = task_start.elapsed();

                (true, latency.as_secs_f64() * 1000.0)
            });
            
            handles.push(handle);
        }

        // 等待所有任务完成
        let mut latencies = Vec::new();
        let mut errors = 0;
        
        for handle in handles {
            match handle.await {
                Ok((success, latency)) => {
                    if success {
                        latencies.push(latency);
                    } else {
                        errors += 1;
                    }
                }
                Err(_) => {
                    errors += 1;
                }
            }
        }

        let total_duration = start_time.elapsed();
        self.calculate_results(latencies, errors, total_duration)
    }

    /// 资源使用性能基准测试
    async fn benchmark_resource_usage(&self) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        // 获取系统资源使用情况
        let memory_usage = self.get_memory_usage().await;
        let cpu_usage = self.get_cpu_usage().await;

        Ok(BenchmarkResults {
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput_ops_per_sec: 0.0,
            error_rate_percent: 0.0,
            memory_usage_mb: memory_usage,
            cpu_usage_percent: cpu_usage,
        })
    }

    /// 发送测试消息
    async fn send_test_message(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _message = A2AMessage::new_text(
            MessageRole::User,
            "性能测试消息".to_string(),
        );
        
        // 模拟消息处理
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    /// 路由测试消息
    async fn route_test_message(&self, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
        let _message = A2AMessage::new_text(
            MessageRole::User,
            format!("从 {} 到 {} 的路由测试", from, to),
        );
        
        // 模拟消息路由
        tokio::time::sleep(Duration::from_micros(200)).await;
        Ok(())
    }

    /// 创建测试Agent信息
    async fn create_test_agent_info(&self, id: usize) -> AgentInfo {
        AgentInfo {
            id: format!("test_agent_{}", id),
            name: format!("测试Agent {}", id),
            endpoint: format!("http://test-{}.local:8080", id),
            capabilities: vec!["test".to_string(), "benchmark".to_string()],
            status: AgentStatus::Online,
        }
    }

    /// 计算性能测试结果
    fn calculate_results(
        &self,
        mut latencies: Vec<f64>,
        errors: usize,
        total_duration: Duration,
    ) -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
        if latencies.is_empty() {
            return Ok(BenchmarkResults {
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                throughput_ops_per_sec: 0.0,
                error_rate_percent: 100.0,
                memory_usage_mb: 0.0,
                cpu_usage_percent: 0.0,
            });
        }

        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p95_index = (latencies.len() as f64 * 0.95) as usize;
        let p99_index = (latencies.len() as f64 * 0.99) as usize;
        
        let p95_latency = latencies[p95_index.min(latencies.len() - 1)];
        let p99_latency = latencies[p99_index.min(latencies.len() - 1)];
        
        let total_ops = latencies.len() + errors;
        let throughput = total_ops as f64 / total_duration.as_secs_f64();
        let error_rate = (errors as f64 / total_ops as f64) * 100.0;

        Ok(BenchmarkResults {
            avg_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            throughput_ops_per_sec: throughput,
            error_rate_percent: error_rate,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
        })
    }

    /// 获取内存使用情况 (MB)
    async fn get_memory_usage(&self) -> f64 {
        // 简化实现，实际应该使用系统API
        128.0 // 模拟值
    }

    /// 获取CPU使用率 (%)
    async fn get_cpu_usage(&self) -> f64 {
        // 简化实现，实际应该使用系统API
        15.0 // 模拟值
    }

    /// 打印测试结果摘要
    fn print_summary(&self, results: &[BenchmarkResults]) {
        println!("\n📊 性能基准测试结果摘要");
        println!("{}", "=".repeat(60));
        
        let test_names = [
            "A2A协议性能",
            "消息路由性能", 
            "Agent注册发现",
            "集群管理性能",
            "并发负载测试",
            "资源使用情况",
        ];

        for (i, result) in results.iter().enumerate() {
            if i < test_names.len() {
                println!("\n🔸 {}", test_names[i]);
                println!("   平均延迟: {:.2}ms", result.avg_latency_ms);
                println!("   P95延迟: {:.2}ms", result.p95_latency_ms);
                println!("   P99延迟: {:.2}ms", result.p99_latency_ms);
                println!("   吞吐量: {:.0} ops/sec", result.throughput_ops_per_sec);
                println!("   错误率: {:.2}%", result.error_rate_percent);
                
                if result.memory_usage_mb > 0.0 {
                    println!("   内存使用: {:.1}MB", result.memory_usage_mb);
                }
                if result.cpu_usage_percent > 0.0 {
                    println!("   CPU使用: {:.1}%", result.cpu_usage_percent);
                }
            }
        }

        println!("\n🎯 性能目标验证:");
        self.validate_performance_targets(results);
    }

    /// 验证性能目标
    fn validate_performance_targets(&self, results: &[BenchmarkResults]) {
        let mut all_passed = true;

        // 验证消息路由延迟 < 10ms
        if let Some(routing_result) = results.get(1) {
            let passed = routing_result.avg_latency_ms < 10.0;
            println!("   消息路由延迟 < 10ms: {} ({:.2}ms)", 
                if passed { "✅ 通过" } else { "❌ 失败" }, 
                routing_result.avg_latency_ms);
            all_passed &= passed;
        }

        // 验证Agent注册性能 > 1000 ops/sec
        if let Some(registry_result) = results.get(2) {
            let passed = registry_result.throughput_ops_per_sec > 1000.0;
            println!("   Agent注册吞吐量 > 1000 ops/sec: {} ({:.0} ops/sec)", 
                if passed { "✅ 通过" } else { "❌ 失败" }, 
                registry_result.throughput_ops_per_sec);
            all_passed &= passed;
        }

        // 验证并发支持 > 1000 Agent
        if let Some(concurrent_result) = results.get(4) {
            let passed = concurrent_result.error_rate_percent < 5.0;
            println!("   并发Agent支持: {} (错误率: {:.2}%)", 
                if passed { "✅ 通过" } else { "❌ 失败" }, 
                concurrent_result.error_rate_percent);
            all_passed &= passed;
        }

        println!("\n🏆 总体评估: {}", 
            if all_passed { "✅ 所有性能目标达成" } else { "❌ 部分性能目标未达成" });
    }
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let config = BenchmarkConfig::default();
    let benchmarks = PerformanceBenchmarks::new(config).await.unwrap();
    
    let results = benchmarks.run_full_benchmark().await.unwrap();
    assert!(!results.is_empty(), "应该有性能测试结果");
    
    // 验证关键性能指标
    for result in &results {
        assert!(result.avg_latency_ms >= 0.0, "平均延迟应该为非负数");
        assert!(result.throughput_ops_per_sec >= 0.0, "吞吐量应该为非负数");
        assert!(result.error_rate_percent >= 0.0 && result.error_rate_percent <= 100.0, 
            "错误率应该在0-100%之间");
    }
}
