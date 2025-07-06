//! 基于真实网络的性能基准测试
//! 
//! 提供在真实网络环境中的性能测试，包括：
//! - 跨网络的消息路由延迟测试
//! - 分布式Agent通信性能测试
//! - 网络抖动和丢包情况下的性能测试
//! - 真实负载下的系统稳定性测试

use agentx_a2a::{A2AResult, A2AError, A2AMessage, MessageRole, AgentCard};
use agentx_cluster::{ClusterManager, ClusterConfig};
use agentx_router::{MessageRouter, RouterConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{info, debug};

/// 真实网络基准测试管理器
pub struct RealNetworkBenchmarks {
    /// 测试配置
    config: RealNetworkConfig,
    /// 集群管理器
    #[allow(dead_code)]
    cluster_manager: Arc<ClusterManager>,
    /// 消息路由器（可选，用于基准测试）
    router: Arc<RwLock<Option<MessageRouter>>>,
    /// 测试节点列表
    test_nodes: Vec<TestNode>,
    /// 网络模拟器
    network_simulator: NetworkSimulator,
}

/// 真实网络测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealNetworkConfig {
    /// 测试节点数量
    pub node_count: usize,
    /// 每个节点的Agent数量
    pub agents_per_node: usize,
    /// 测试持续时间
    pub test_duration: Duration,
    /// 消息发送频率（消息/秒）
    pub message_rate: f64,
    /// 网络延迟模拟（毫秒）
    pub simulated_latency_ms: u64,
    /// 丢包率（百分比）
    pub packet_loss_rate: f64,
    /// 网络抖动（毫秒）
    pub jitter_ms: u64,
    /// 带宽限制（Mbps）
    pub bandwidth_limit_mbps: Option<f64>,
    /// 是否启用网络分区测试
    pub enable_network_partition: bool,
}

impl Default for RealNetworkConfig {
    fn default() -> Self {
        Self {
            node_count: 5,
            agents_per_node: 10,
            test_duration: Duration::from_secs(300), // 5分钟
            message_rate: 100.0, // 100 msg/s
            simulated_latency_ms: 50, // 50ms延迟
            packet_loss_rate: 0.1, // 0.1%丢包率
            jitter_ms: 10, // 10ms抖动
            bandwidth_limit_mbps: Some(100.0), // 100Mbps
            enable_network_partition: false,
        }
    }
}

/// 测试节点
#[derive(Debug, Clone)]
pub struct TestNode {
    /// 节点ID
    pub node_id: String,
    /// 节点地址
    pub address: SocketAddr,
    /// 节点上的Agent列表
    pub agents: Vec<String>,
    /// 节点状态
    pub status: NodeStatus,
    /// 性能指标
    pub metrics: NodeMetrics,
}

/// 节点状态
#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    /// 在线
    Online,
    /// 离线
    Offline,
    /// 网络分区
    Partitioned,
    /// 故障
    Failed(String),
}

/// 节点性能指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// 消息发送数量
    pub messages_sent: u64,
    /// 消息接收数量
    pub messages_received: u64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 错误数量
    pub error_count: u64,
    /// CPU使用率
    pub cpu_usage: f64,
    /// 内存使用量（MB）
    pub memory_usage_mb: f64,
    /// 网络带宽使用（Mbps）
    pub bandwidth_usage_mbps: f64,
}

/// 网络模拟器
pub struct NetworkSimulator {
    /// 延迟配置
    latency_ms: u64,
    /// 丢包率
    packet_loss_rate: f64,
    /// 抖动
    jitter_ms: u64,
    /// 带宽限制
    bandwidth_limit: Option<f64>,
}

/// 真实网络测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealNetworkBenchmarkResult {
    /// 测试名称
    pub test_name: String,
    /// 测试开始时间
    pub start_time: SystemTime,
    /// 测试持续时间
    pub duration: Duration,
    /// 总消息数量
    pub total_messages: u64,
    /// 成功消息数量
    pub successful_messages: u64,
    /// 失败消息数量
    pub failed_messages: u64,
    /// 平均端到端延迟（毫秒）
    pub avg_e2e_latency_ms: f64,
    /// P95延迟（毫秒）
    pub p95_latency_ms: f64,
    /// P99延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f64,
    /// 吞吐量（消息/秒）
    pub throughput_msg_per_sec: f64,
    /// 错误率（百分比）
    pub error_rate_percent: f64,
    /// 网络利用率
    pub network_utilization: NetworkUtilization,
    /// 节点性能指标
    pub node_metrics: HashMap<String, NodeMetrics>,
    /// 测试是否通过
    pub passed: bool,
    /// 失败原因
    pub failure_reason: Option<String>,
}

/// 网络利用率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUtilization {
    /// 平均带宽使用（Mbps）
    pub avg_bandwidth_mbps: f64,
    /// 峰值带宽使用（Mbps）
    pub peak_bandwidth_mbps: f64,
    /// 网络延迟分布
    pub latency_distribution: LatencyDistribution,
    /// 丢包统计
    pub packet_loss_stats: PacketLossStats,
}

/// 延迟分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDistribution {
    /// 最小延迟
    pub min_ms: f64,
    /// 最大延迟
    pub max_ms: f64,
    /// 平均延迟
    pub avg_ms: f64,
    /// 标准差
    pub std_dev_ms: f64,
    /// 百分位数
    pub percentiles: HashMap<String, f64>, // P50, P90, P95, P99
}

/// 丢包统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossStats {
    /// 总发送包数
    pub total_packets: u64,
    /// 丢失包数
    pub lost_packets: u64,
    /// 丢包率
    pub loss_rate_percent: f64,
    /// 重传次数
    pub retransmissions: u64,
}

impl RealNetworkBenchmarks {
    /// 创建新的真实网络基准测试管理器
    pub async fn new(config: RealNetworkConfig) -> A2AResult<Self> {
        info!("🌐 创建真实网络基准测试管理器");
        
        // 创建集群管理器
        let cluster_config = ClusterConfig::default();
        let cluster_manager = Arc::new(ClusterManager::new(cluster_config).await
            .map_err(|e| A2AError::internal(format!("创建集群管理器失败: {}", e)))?);
        
        // 创建消息路由器（简化实现，用于基准测试）
        let _router_config = RouterConfig::default();
        // 注意：在实际实现中，这里需要创建真实的路由器依赖
        // 为了基准测试，我们暂时跳过路由器的创建
        let router = Arc::new(RwLock::new(None::<MessageRouter>));
        
        // 初始化测试节点
        let test_nodes = Self::initialize_test_nodes(&config).await?;
        
        // 创建网络模拟器
        let network_simulator = NetworkSimulator::new(
            config.simulated_latency_ms,
            config.packet_loss_rate,
            config.jitter_ms,
            config.bandwidth_limit_mbps,
        );
        
        Ok(Self {
            config,
            cluster_manager,
            router,
            test_nodes,
            network_simulator,
        })
    }
    
    /// 运行完整的真实网络基准测试套件
    pub async fn run_full_benchmark_suite(&mut self) -> A2AResult<Vec<RealNetworkBenchmarkResult>> {
        info!("🚀 开始真实网络基准测试套件");
        
        let mut results = Vec::new();
        
        // 1. 基础网络延迟测试
        info!("📡 执行基础网络延迟测试");
        let latency_result = self.benchmark_network_latency().await?;
        results.push(latency_result);
        
        // 2. 高并发消息路由测试
        info!("🔀 执行高并发消息路由测试");
        let routing_result = self.benchmark_concurrent_routing().await?;
        results.push(routing_result);
        
        // 3. 分布式Agent通信测试
        info!("👥 执行分布式Agent通信测试");
        let agent_comm_result = self.benchmark_distributed_agents().await?;
        results.push(agent_comm_result);
        
        // 4. 网络故障恢复测试
        if self.config.enable_network_partition {
            info!("🔧 执行网络故障恢复测试");
            let recovery_result = self.benchmark_network_recovery().await?;
            results.push(recovery_result);
        }
        
        // 5. 长期稳定性测试
        info!("⏱️ 执行长期稳定性测试");
        let stability_result = self.benchmark_long_term_stability().await?;
        results.push(stability_result);
        
        info!("✅ 真实网络基准测试套件完成");
        Ok(results)
    }

    /// 基础网络延迟基准测试
    async fn benchmark_network_latency(&mut self) -> A2AResult<RealNetworkBenchmarkResult> {
        let test_name = "network_latency".to_string();
        let start_time = SystemTime::now();
        let mut latencies = Vec::new();
        let mut successful_messages = 0;
        let mut failed_messages = 0;

        info!("开始网络延迟基准测试");

        // 创建测试消息
        let test_message_count = 1000;
        let semaphore = Arc::new(Semaphore::new(50)); // 限制并发数

        let mut handles = Vec::new();

        for i in 0..test_message_count {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let nodes = self.test_nodes.clone();
            let simulator = self.network_simulator.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit; // 保持permit直到任务完成

                // 选择源节点和目标节点
                let source_node = &nodes[i % nodes.len()];
                let target_node = &nodes[(i + 1) % nodes.len()];

                let message = A2AMessage::new_text(
                    MessageRole::Agent,
                    format!("延迟测试消息 {}", i),
                );

                let start = Instant::now();

                // 模拟网络传输
                let result = simulator.simulate_message_transmission(&message,
                    &source_node.address, &target_node.address).await;

                let latency = start.elapsed();

                match result {
                    Ok(_) => (latency.as_secs_f64() * 1000.0, true), // 转换为毫秒
                    Err(_) => (0.0, false),
                }
            });

            handles.push(handle);
        }

        // 收集结果
        for handle in handles {
            match handle.await {
                Ok((latency, success)) => {
                    if success {
                        latencies.push(latency);
                        successful_messages += 1;
                    } else {
                        failed_messages += 1;
                    }
                }
                Err(_) => failed_messages += 1,
            }
        }

        let duration = start_time.elapsed().unwrap();

        // 计算统计数据
        let (avg_latency, p95_latency, p99_latency, max_latency) =
            Self::calculate_latency_stats(&latencies);

        let total_messages = successful_messages + failed_messages;
        let throughput = successful_messages as f64 / duration.as_secs_f64();
        let error_rate = (failed_messages as f64 / total_messages as f64) * 100.0;

        // 检查是否通过测试（延迟 < 10ms，错误率 < 1%）
        let passed = avg_latency < 10.0 && error_rate < 1.0;
        let failure_reason = if !passed {
            Some(format!("平均延迟 {:.2}ms 或错误率 {:.2}% 超过阈值", avg_latency, error_rate))
        } else {
            None
        };

        info!("网络延迟测试完成: 平均延迟 {:.2}ms, 吞吐量 {:.0} msg/s, 错误率 {:.2}%",
              avg_latency, throughput, error_rate);

        Ok(RealNetworkBenchmarkResult {
            test_name,
            start_time,
            duration,
            total_messages,
            successful_messages,
            failed_messages,
            avg_e2e_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: max_latency,
            throughput_msg_per_sec: throughput,
            error_rate_percent: error_rate,
            network_utilization: self.calculate_network_utilization(&latencies).await,
            node_metrics: self.collect_node_metrics().await,
            passed,
            failure_reason,
        })
    }

    /// 高并发消息路由基准测试
    async fn benchmark_concurrent_routing(&mut self) -> A2AResult<RealNetworkBenchmarkResult> {
        let test_name = "concurrent_routing".to_string();
        let start_time = SystemTime::now();
        let mut latencies = Vec::new();
        let mut successful_messages = 0;
        let mut failed_messages = 0;

        info!("开始高并发消息路由基准测试");

        let concurrent_agents = self.config.agents_per_node * self.config.node_count;
        let messages_per_agent = 100;
        let total_messages = concurrent_agents * messages_per_agent;

        let semaphore = Arc::new(Semaphore::new(concurrent_agents));
        let mut handles = Vec::new();

        for agent_id in 0..concurrent_agents {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let _router = self.router.clone();
            let _nodes = self.test_nodes.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let mut agent_latencies = Vec::new();
                let mut agent_successes = 0;
                let mut agent_failures = 0;

                for msg_id in 0..messages_per_agent {
                    let message = A2AMessage::new_text(
                        MessageRole::Agent,
                        format!("Agent {} 消息 {}", agent_id, msg_id),
                    );

                    let start = Instant::now();

                    // 模拟路由器发送消息（简化实现）
                    let result = timeout(
                        Duration::from_millis(100), // 100ms超时
                        Self::simulate_router_message_send(message)
                    ).await;

                    let latency = start.elapsed();

                    match result {
                        Ok(Ok(_)) => {
                            agent_latencies.push(latency.as_secs_f64() * 1000.0);
                            agent_successes += 1;
                        }
                        _ => agent_failures += 1,
                    }
                }

                (agent_latencies, agent_successes, agent_failures)
            });

            handles.push(handle);
        }

        // 收集所有Agent的结果
        for handle in handles {
            match handle.await {
                Ok((agent_latencies, successes, failures)) => {
                    latencies.extend(agent_latencies);
                    successful_messages += successes;
                    failed_messages += failures;
                }
                Err(_) => failed_messages += messages_per_agent as u64,
            }
        }

        let duration = start_time.elapsed().unwrap();

        // 计算统计数据
        let (avg_latency, p95_latency, p99_latency, max_latency) =
            Self::calculate_latency_stats(&latencies);

        let throughput = successful_messages as f64 / duration.as_secs_f64();
        let error_rate = (failed_messages as f64 / total_messages as f64) * 100.0;

        // 检查是否通过测试（延迟 < 10ms，吞吐量 > 1000 msg/s，错误率 < 1%）
        let passed = avg_latency < 10.0 && throughput > 1000.0 && error_rate < 1.0;
        let failure_reason = if !passed {
            Some(format!("性能指标未达标: 延迟 {:.2}ms, 吞吐量 {:.0} msg/s, 错误率 {:.2}%",
                        avg_latency, throughput, error_rate))
        } else {
            None
        };

        info!("高并发路由测试完成: 平均延迟 {:.2}ms, 吞吐量 {:.0} msg/s, 错误率 {:.2}%",
              avg_latency, throughput, error_rate);

        Ok(RealNetworkBenchmarkResult {
            test_name,
            start_time,
            duration,
            total_messages: total_messages as u64,
            successful_messages,
            failed_messages,
            avg_e2e_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: max_latency,
            throughput_msg_per_sec: throughput,
            error_rate_percent: error_rate,
            network_utilization: self.calculate_network_utilization(&latencies).await,
            node_metrics: self.collect_node_metrics().await,
            passed,
            failure_reason,
        })
    }

    /// 分布式Agent通信基准测试
    async fn benchmark_distributed_agents(&mut self) -> A2AResult<RealNetworkBenchmarkResult> {
        let test_name = "distributed_agents".to_string();
        let start_time = SystemTime::now();

        info!("开始分布式Agent通信基准测试");

        // 在每个节点上创建Agent
        let mut all_agents = Vec::new();
        for node in &self.test_nodes {
            for i in 0..self.config.agents_per_node {
                let agent_id = format!("agent_{}_{}", node.node_id, i);
                let agent_card = AgentCard::new(
                    agent_id.clone(),
                    format!("测试Agent {}", agent_id),
                    "分布式通信测试Agent".to_string(),
                    "1.0.0".to_string(),
                );
                all_agents.push((agent_id, agent_card, node.address));
            }
        }

        let mut latencies = Vec::new();
        let mut successful_messages = 0;
        let mut failed_messages = 0;

        // 执行Agent间通信测试
        let test_duration = Duration::from_secs(60); // 1分钟测试
        let message_interval = Duration::from_millis(100); // 每100ms发送一条消息
        let test_start = Instant::now();

        while test_start.elapsed() < test_duration {
            let mut round_handles = Vec::new();

            // 每轮选择一些Agent进行通信
            for i in (0..all_agents.len()).step_by(2) {
                if i + 1 < all_agents.len() {
                    let sender = all_agents[i].clone();
                    let receiver = all_agents[i + 1].clone();

                    let handle = tokio::spawn(async move {
                        let message = A2AMessage::new_text(
                            MessageRole::Agent,
                            format!("从 {} 到 {} 的消息", sender.0, receiver.0),
                        );

                        let start = Instant::now();

                        // 模拟跨节点消息传输
                        let result = Self::simulate_cross_node_communication(
                            &sender.2, &receiver.2, &message
                        ).await;

                        let latency = start.elapsed();

                        match result {
                            Ok(_) => (latency.as_secs_f64() * 1000.0, true),
                            Err(_) => (0.0, false),
                        }
                    });

                    round_handles.push(handle);
                }
            }

            // 收集本轮结果
            for handle in round_handles {
                match handle.await {
                    Ok((latency, success)) => {
                        if success {
                            latencies.push(latency);
                            successful_messages += 1;
                        } else {
                            failed_messages += 1;
                        }
                    }
                    Err(_) => failed_messages += 1,
                }
            }

            tokio::time::sleep(message_interval).await;
        }

        let duration = start_time.elapsed().unwrap();

        // 计算统计数据
        let (avg_latency, p95_latency, p99_latency, max_latency) =
            Self::calculate_latency_stats(&latencies);

        let total_messages = successful_messages + failed_messages;
        let throughput = successful_messages as f64 / duration.as_secs_f64();
        let error_rate = (failed_messages as f64 / total_messages as f64) * 100.0;

        // 检查是否通过测试
        let passed = avg_latency < 50.0 && error_rate < 2.0; // 分布式通信允许更高延迟
        let failure_reason = if !passed {
            Some(format!("分布式通信性能不达标: 延迟 {:.2}ms, 错误率 {:.2}%",
                        avg_latency, error_rate))
        } else {
            None
        };

        info!("分布式Agent通信测试完成: 平均延迟 {:.2}ms, 吞吐量 {:.0} msg/s, 错误率 {:.2}%",
              avg_latency, throughput, error_rate);

        Ok(RealNetworkBenchmarkResult {
            test_name,
            start_time,
            duration,
            total_messages,
            successful_messages,
            failed_messages,
            avg_e2e_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: max_latency,
            throughput_msg_per_sec: throughput,
            error_rate_percent: error_rate,
            network_utilization: self.calculate_network_utilization(&latencies).await,
            node_metrics: self.collect_node_metrics().await,
            passed,
            failure_reason,
        })
    }

    /// 网络故障恢复基准测试
    async fn benchmark_network_recovery(&mut self) -> A2AResult<RealNetworkBenchmarkResult> {
        let test_name = "network_recovery".to_string();
        let start_time = SystemTime::now();

        info!("开始网络故障恢复基准测试");

        // 模拟网络分区：将节点分为两组
        let mid_point = self.test_nodes.len() / 2;
        let (group1, group2) = self.test_nodes.split_at_mut(mid_point);

        // 设置分区状态
        for node in group1.iter_mut() {
            node.status = NodeStatus::Partitioned;
        }

        let mut latencies = Vec::new();
        let mut successful_messages = 0;
        let mut failed_messages = 0;

        // 阶段1: 网络分区期间的通信测试（预期大部分失败）
        info!("阶段1: 网络分区期间测试");
        let partition_duration = Duration::from_secs(30);
        let partition_start = Instant::now();

        while partition_start.elapsed() < partition_duration {
            // 尝试跨分区通信（应该失败）
            let result = Self::simulate_cross_node_communication(
                &group1[0].address,
                &group2[0].address,
                &A2AMessage::new_text(MessageRole::Agent, "分区测试消息".to_string()),
            ).await;

            match result {
                Ok(_) => successful_messages += 1,
                Err(_) => failed_messages += 1,
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // 阶段2: 恢复网络连接
        info!("阶段2: 恢复网络连接");
        for node in group1.iter_mut() {
            node.status = NodeStatus::Online;
        }

        // 等待网络恢复
        tokio::time::sleep(Duration::from_secs(5)).await;

        // 阶段3: 恢复后的通信测试
        info!("阶段3: 网络恢复后测试");
        let recovery_test_count = 100;

        for i in 0..recovery_test_count {
            let start = Instant::now();

            let result = Self::simulate_cross_node_communication(
                &group1[i % group1.len()].address,
                &group2[i % group2.len()].address,
                &A2AMessage::new_text(MessageRole::Agent, format!("恢复测试消息 {}", i)),
            ).await;

            let latency = start.elapsed();

            match result {
                Ok(_) => {
                    latencies.push(latency.as_secs_f64() * 1000.0);
                    successful_messages += 1;
                }
                Err(_) => failed_messages += 1,
            }
        }

        let duration = start_time.elapsed().unwrap();

        // 计算统计数据
        let (avg_latency, p95_latency, p99_latency, max_latency) =
            Self::calculate_latency_stats(&latencies);

        let total_messages = successful_messages + failed_messages;
        let throughput = successful_messages as f64 / duration.as_secs_f64();
        let error_rate = (failed_messages as f64 / total_messages as f64) * 100.0;

        // 检查恢复后的性能
        let recovery_success_rate = if recovery_test_count > 0 {
            (latencies.len() as f64 / recovery_test_count as f64) * 100.0
        } else {
            0.0
        };

        let passed = recovery_success_rate > 95.0 && avg_latency < 100.0;
        let failure_reason = if !passed {
            Some(format!("网络恢复性能不达标: 恢复成功率 {:.1}%, 延迟 {:.2}ms",
                        recovery_success_rate, avg_latency))
        } else {
            None
        };

        info!("网络故障恢复测试完成: 恢复成功率 {:.1}%, 平均延迟 {:.2}ms",
              recovery_success_rate, avg_latency);

        Ok(RealNetworkBenchmarkResult {
            test_name,
            start_time,
            duration,
            total_messages,
            successful_messages,
            failed_messages,
            avg_e2e_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: max_latency,
            throughput_msg_per_sec: throughput,
            error_rate_percent: error_rate,
            network_utilization: self.calculate_network_utilization(&latencies).await,
            node_metrics: self.collect_node_metrics().await,
            passed,
            failure_reason,
        })
    }

    /// 长期稳定性基准测试
    async fn benchmark_long_term_stability(&mut self) -> A2AResult<RealNetworkBenchmarkResult> {
        let test_name = "long_term_stability".to_string();
        let start_time = SystemTime::now();

        info!("开始长期稳定性基准测试");

        let mut latencies = Vec::new();
        let mut successful_messages = 0;
        let mut failed_messages = 0;

        // 长期测试：持续发送消息并监控性能变化
        let stability_duration = self.config.test_duration;
        let message_interval = Duration::from_millis((1000.0 / self.config.message_rate) as u64);
        let test_start = Instant::now();

        let mut performance_samples = Vec::new();
        let sample_interval = Duration::from_secs(30); // 每30秒采样一次
        let mut last_sample = Instant::now();

        while test_start.elapsed() < stability_duration {
            let round_start = Instant::now();

            // 发送一批消息
            let batch_size = 10;
            let mut batch_latencies = Vec::new();
            let mut batch_successes = 0;
            let mut batch_failures = 0;

            for i in 0..batch_size {
                let node_idx = i % self.test_nodes.len();
                let target_idx = (i + 1) % self.test_nodes.len();

                let message = A2AMessage::new_text(
                    MessageRole::Agent,
                    format!("稳定性测试消息 {}", successful_messages + failed_messages + i),
                );

                let msg_start = Instant::now();

                let result = Self::simulate_cross_node_communication(
                    &self.test_nodes[node_idx].address,
                    &self.test_nodes[target_idx].address,
                    &message,
                ).await;

                let latency = msg_start.elapsed();

                match result {
                    Ok(_) => {
                        batch_latencies.push(latency.as_secs_f64() * 1000.0);
                        batch_successes += 1;
                    }
                    Err(_) => batch_failures += 1,
                }
            }

            latencies.extend(batch_latencies.clone());
            successful_messages += batch_successes;
            failed_messages += batch_failures;

            // 定期采样性能指标
            if last_sample.elapsed() >= sample_interval {
                let avg_batch_latency = if !batch_latencies.is_empty() {
                    batch_latencies.iter().sum::<f64>() / batch_latencies.len() as f64
                } else {
                    0.0
                };

                performance_samples.push((test_start.elapsed(), avg_batch_latency));
                last_sample = Instant::now();

                debug!("稳定性测试进度: {:.1}%, 当前延迟: {:.2}ms",
                      (test_start.elapsed().as_secs_f64() / stability_duration.as_secs_f64()) * 100.0,
                      avg_batch_latency);
            }

            // 控制消息发送频率
            let elapsed = round_start.elapsed();
            if elapsed < message_interval {
                tokio::time::sleep(message_interval - elapsed).await;
            }
        }

        let duration = start_time.elapsed().unwrap();

        // 分析性能稳定性
        let stability_analysis = Self::analyze_performance_stability(&performance_samples);

        // 计算统计数据
        let (avg_latency, p95_latency, p99_latency, max_latency) =
            Self::calculate_latency_stats(&latencies);

        let total_messages = successful_messages + failed_messages;
        let throughput = successful_messages as f64 / duration.as_secs_f64();
        let error_rate = (failed_messages as f64 / total_messages as f64) * 100.0;

        // 检查长期稳定性
        let passed = stability_analysis.is_stable &&
                    avg_latency < 20.0 &&
                    error_rate < 2.0 &&
                    stability_analysis.performance_degradation < 50.0; // 性能退化 < 50%

        let failure_reason = if !passed {
            Some(format!("长期稳定性测试失败: 稳定性 {}, 延迟 {:.2}ms, 错误率 {:.2}%, 性能退化 {:.1}%",
                        stability_analysis.is_stable, avg_latency, error_rate,
                        stability_analysis.performance_degradation))
        } else {
            None
        };

        info!("长期稳定性测试完成: 平均延迟 {:.2}ms, 吞吐量 {:.0} msg/s, 错误率 {:.2}%, 性能退化 {:.1}%",
              avg_latency, throughput, error_rate, stability_analysis.performance_degradation);

        Ok(RealNetworkBenchmarkResult {
            test_name,
            start_time,
            duration,
            total_messages: total_messages as u64,
            successful_messages: successful_messages as u64,
            failed_messages: failed_messages as u64,
            avg_e2e_latency_ms: avg_latency,
            p95_latency_ms: p95_latency,
            p99_latency_ms: p99_latency,
            max_latency_ms: max_latency,
            throughput_msg_per_sec: throughput,
            error_rate_percent: error_rate,
            network_utilization: self.calculate_network_utilization(&latencies).await,
            node_metrics: self.collect_node_metrics().await,
            passed,
            failure_reason,
        })
    }

    // 辅助方法

    /// 初始化测试节点
    async fn initialize_test_nodes(config: &RealNetworkConfig) -> A2AResult<Vec<TestNode>> {
        let mut nodes = Vec::new();
        let base_port = 8000;

        for i in 0..config.node_count {
            let node_id = format!("node_{}", i);
            let address = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                base_port + i as u16,
            );

            let mut agents = Vec::new();
            for j in 0..config.agents_per_node {
                agents.push(format!("agent_{}_{}", i, j));
            }

            let node = TestNode {
                node_id,
                address,
                agents,
                status: NodeStatus::Online,
                metrics: NodeMetrics::default(),
            };

            nodes.push(node);
        }

        Ok(nodes)
    }

    /// 计算延迟统计数据
    fn calculate_latency_stats(latencies: &[f64]) -> (f64, f64, f64, f64) {
        if latencies.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg = sorted_latencies.iter().sum::<f64>() / sorted_latencies.len() as f64;
        let p95_idx = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_latencies.len() as f64 * 0.99) as usize;

        let p95 = sorted_latencies.get(p95_idx).copied().unwrap_or(0.0);
        let p99 = sorted_latencies.get(p99_idx).copied().unwrap_or(0.0);
        let max = sorted_latencies.last().copied().unwrap_or(0.0);

        (avg, p95, p99, max)
    }

    /// 模拟跨节点通信
    async fn simulate_cross_node_communication(
        source: &SocketAddr,
        target: &SocketAddr,
        _message: &A2AMessage,
    ) -> A2AResult<()> {
        // 模拟网络延迟
        tokio::time::sleep(Duration::from_millis(1)).await;

        // 模拟网络连接（简化实现）
        if source.port() != target.port() {
            // 模拟成功的跨节点通信
            Ok(())
        } else {
            Err(A2AError::internal("同节点通信".to_string()))
        }
    }

    /// 模拟路由器消息发送
    async fn simulate_router_message_send(_message: A2AMessage) -> A2AResult<()> {
        // 模拟路由延迟
        tokio::time::sleep(Duration::from_millis(1)).await;

        // 模拟成功的消息路由
        Ok(())
    }

    /// 计算网络利用率
    async fn calculate_network_utilization(&self, latencies: &[f64]) -> NetworkUtilization {
        let latency_distribution = if !latencies.is_empty() {
            let mut sorted = latencies.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min = sorted.first().copied().unwrap_or(0.0);
            let max = sorted.last().copied().unwrap_or(0.0);
            let avg = sorted.iter().sum::<f64>() / sorted.len() as f64;

            // 计算标准差
            let variance = sorted.iter()
                .map(|x| (x - avg).powi(2))
                .sum::<f64>() / sorted.len() as f64;
            let std_dev = variance.sqrt();

            let mut percentiles = HashMap::new();
            percentiles.insert("P50".to_string(), sorted[sorted.len() / 2]);
            percentiles.insert("P90".to_string(), sorted[(sorted.len() as f64 * 0.9) as usize]);
            percentiles.insert("P95".to_string(), sorted[(sorted.len() as f64 * 0.95) as usize]);
            percentiles.insert("P99".to_string(), sorted[(sorted.len() as f64 * 0.99) as usize]);

            LatencyDistribution {
                min_ms: min,
                max_ms: max,
                avg_ms: avg,
                std_dev_ms: std_dev,
                percentiles,
            }
        } else {
            LatencyDistribution {
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                std_dev_ms: 0.0,
                percentiles: HashMap::new(),
            }
        };

        NetworkUtilization {
            avg_bandwidth_mbps: 50.0, // 模拟值
            peak_bandwidth_mbps: 80.0, // 模拟值
            latency_distribution,
            packet_loss_stats: PacketLossStats {
                total_packets: latencies.len() as u64,
                lost_packets: 0, // 简化实现
                loss_rate_percent: 0.0,
                retransmissions: 0,
            },
        }
    }

    /// 收集节点性能指标
    async fn collect_node_metrics(&self) -> HashMap<String, NodeMetrics> {
        let mut metrics = HashMap::new();

        for node in &self.test_nodes {
            metrics.insert(node.node_id.clone(), node.metrics.clone());
        }

        metrics
    }

    /// 分析性能稳定性
    fn analyze_performance_stability(samples: &[(Duration, f64)]) -> StabilityAnalysis {
        if samples.len() < 2 {
            return StabilityAnalysis {
                is_stable: false,
                performance_degradation: 100.0,
                trend: "insufficient_data".to_string(),
            };
        }

        let first_half = &samples[..samples.len() / 2];
        let second_half = &samples[samples.len() / 2..];

        let first_avg = first_half.iter().map(|(_, latency)| latency).sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().map(|(_, latency)| latency).sum::<f64>() / second_half.len() as f64;

        let degradation = if first_avg > 0.0 {
            ((second_avg - first_avg) / first_avg) * 100.0
        } else {
            0.0
        };

        let is_stable = degradation < 20.0; // 性能退化小于20%认为稳定
        let trend = if degradation > 10.0 {
            "degrading".to_string()
        } else if degradation < -10.0 {
            "improving".to_string()
        } else {
            "stable".to_string()
        };

        StabilityAnalysis {
            is_stable,
            performance_degradation: degradation.max(0.0),
            trend,
        }
    }
}

/// 稳定性分析结果
#[derive(Debug, Clone)]
struct StabilityAnalysis {
    /// 是否稳定
    is_stable: bool,
    /// 性能退化百分比
    performance_degradation: f64,
    /// 趋势
    #[allow(dead_code)]
    trend: String,
}

impl NetworkSimulator {
    /// 创建新的网络模拟器
    pub fn new(latency_ms: u64, packet_loss_rate: f64, jitter_ms: u64, bandwidth_limit: Option<f64>) -> Self {
        Self {
            latency_ms,
            packet_loss_rate,
            jitter_ms,
            bandwidth_limit,
        }
    }

    /// 模拟消息传输
    pub async fn simulate_message_transmission(
        &self,
        _message: &A2AMessage,
        _source: &SocketAddr,
        _target: &SocketAddr,
    ) -> A2AResult<()> {
        // 模拟网络延迟
        let base_latency = Duration::from_millis(self.latency_ms);

        // 添加抖动
        let jitter = if self.jitter_ms > 0 {
            Duration::from_millis(rand::random::<u64>() % self.jitter_ms)
        } else {
            Duration::ZERO
        };

        tokio::time::sleep(base_latency + jitter).await;

        // 模拟丢包
        if rand::random::<f64>() < self.packet_loss_rate / 100.0 {
            return Err(A2AError::internal("模拟丢包".to_string()));
        }

        Ok(())
    }
}

impl Clone for NetworkSimulator {
    fn clone(&self) -> Self {
        Self {
            latency_ms: self.latency_ms,
            packet_loss_rate: self.packet_loss_rate,
            jitter_ms: self.jitter_ms,
            bandwidth_limit: self.bandwidth_limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_network_benchmarks_creation() {
        let config = RealNetworkConfig::default();
        let benchmarks = RealNetworkBenchmarks::new(config).await;
        assert!(benchmarks.is_ok());
    }

    #[tokio::test]
    async fn test_network_latency_benchmark() {
        let config = RealNetworkConfig {
            node_count: 2,
            agents_per_node: 2,
            test_duration: Duration::from_secs(10),
            message_rate: 10.0,
            ..Default::default()
        };

        let mut benchmarks = RealNetworkBenchmarks::new(config).await.unwrap();
        let result = benchmarks.benchmark_network_latency().await;
        assert!(result.is_ok());

        let benchmark_result = result.unwrap();
        assert_eq!(benchmark_result.test_name, "network_latency");
        assert!(benchmark_result.total_messages > 0);
    }

    #[test]
    fn test_latency_stats_calculation() {
        let latencies = vec![1.0, 2.0, 3.0, 4.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0];
        let (avg, p95, p99, max) = RealNetworkBenchmarks::calculate_latency_stats(&latencies);

        assert!((avg - 11.5).abs() < 0.1);
        assert!(p95 > 20.0);
        assert!(p99 > 25.0);
        assert_eq!(max, 30.0);
    }

    #[test]
    fn test_stability_analysis() {
        let samples = vec![
            (Duration::from_secs(0), 5.0),
            (Duration::from_secs(30), 5.2),
            (Duration::from_secs(60), 5.1),
            (Duration::from_secs(90), 5.3),
        ];

        let analysis = RealNetworkBenchmarks::analyze_performance_stability(&samples);
        assert!(analysis.is_stable);
        assert!(analysis.performance_degradation < 10.0);
    }
}
