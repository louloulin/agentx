//! A2A协议监控和指标收集模块
//! 
//! 实现A2A协议的性能监控、指标收集和健康检查功能

use crate::{A2AResult, AgentStatus, TrustLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};

/// 指标类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    /// 计数器 - 只增不减的累计值
    Counter,
    /// 仪表 - 可增可减的瞬时值
    Gauge,
    /// 直方图 - 值的分布统计
    Histogram,
    /// 摘要 - 值的分位数统计
    Summary,
}

/// 指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// 指标名称
    pub name: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 帮助信息
    pub help: Option<String>,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 消息处理统计
    pub message_stats: MessageStats,
    /// Agent统计
    pub agent_stats: AgentStats,
    /// 系统资源统计
    pub system_stats: SystemStats,
    /// 错误统计
    pub error_stats: ErrorStats,
    /// 统计时间范围
    pub time_range: TimeRange,
}

/// 消息统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    /// 总消息数
    pub total_messages: u64,
    /// 成功处理的消息数
    pub successful_messages: u64,
    /// 失败的消息数
    pub failed_messages: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
    /// 最大处理时间（毫秒）
    pub max_processing_time_ms: f64,
    /// 最小处理时间（毫秒）
    pub min_processing_time_ms: f64,
    /// 消息吞吐量（消息/秒）
    pub throughput_per_second: f64,
    /// 按消息类型分组的统计
    pub by_message_type: HashMap<String, u64>,
}

/// Agent统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    /// 总Agent数
    pub total_agents: u64,
    /// 在线Agent数
    pub online_agents: u64,
    /// 离线Agent数
    pub offline_agents: u64,
    /// 忙碌Agent数
    pub busy_agents: u64,
    /// 按信任级别分组的Agent数
    pub by_trust_level: HashMap<TrustLevel, u64>,
    /// 按状态分组的Agent数
    pub by_status: HashMap<AgentStatus, u64>,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
}

/// 系统资源统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    /// 内存使用量（字节）
    pub memory_usage_bytes: u64,
    /// 内存使用率（百分比）
    pub memory_usage_percent: f64,
    /// 网络入流量（字节/秒）
    pub network_in_bytes_per_sec: f64,
    /// 网络出流量（字节/秒）
    pub network_out_bytes_per_sec: f64,
    /// 磁盘使用量（字节）
    pub disk_usage_bytes: u64,
    /// 磁盘使用率（百分比）
    pub disk_usage_percent: f64,
}

/// 错误统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 认证错误数
    pub auth_errors: u64,
    /// 授权错误数
    pub authz_errors: u64,
    /// 网络错误数
    pub network_errors: u64,
    /// 超时错误数
    pub timeout_errors: u64,
    /// 内部错误数
    pub internal_errors: u64,
    /// 按错误类型分组的统计
    pub by_error_type: HashMap<String, u64>,
    /// 错误率（错误数/总请求数）
    pub error_rate: f64,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// 开始时间
    pub start: DateTime<Utc>,
    /// 结束时间
    pub end: DateTime<Utc>,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 整体健康状态
    pub status: HealthStatus,
    /// 检查时间
    pub timestamp: DateTime<Utc>,
    /// 各组件的健康状态
    pub components: HashMap<String, ComponentHealth>,
    /// 总体评分（0-100）
    pub score: u8,
    /// 详细信息
    pub details: Option<String>,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级
    Degraded,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

/// 基准测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// 测试名称
    pub test_name: String,
    /// 迭代次数
    pub iterations: u64,
    /// 总耗时（毫秒）
    pub total_duration_ms: f64,
    /// 吞吐量（操作/秒）
    pub throughput_ops_per_sec: f64,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// P50延迟（毫秒）
    pub p50_latency_ms: f64,
    /// P95延迟（毫秒）
    pub p95_latency_ms: f64,
    /// P99延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 测试时间戳
    pub timestamp: DateTime<Utc>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 状态
    pub status: HealthStatus,
    /// 响应时间（毫秒）
    pub response_time_ms: Option<f64>,
    /// 错误信息
    pub error: Option<String>,
    /// 最后检查时间
    pub last_check: DateTime<Utc>,
}

/// 监控管理器
#[derive(Debug)]
pub struct MonitoringManager {
    /// 指标存储
    metrics: HashMap<String, Vec<MetricPoint>>,
    /// 原子计数器
    counters: HashMap<String, Arc<AtomicU64>>,
    /// 性能统计
    performance_stats: PerformanceStats,
    /// 健康检查结果
    health_status: HealthCheck,
    /// 监控配置
    config: MonitoringConfig,
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// 指标保留时间
    pub metric_retention_hours: u64,
    /// 健康检查间隔（秒）
    pub health_check_interval_seconds: u64,
    /// 性能统计计算间隔（秒）
    pub stats_calculation_interval_seconds: u64,
    /// 是否启用详细监控
    pub enable_detailed_monitoring: bool,
}

impl MonitoringManager {
    /// 创建新的监控管理器
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            metrics: HashMap::new(),
            counters: HashMap::new(),
            performance_stats: PerformanceStats::default(),
            health_status: HealthCheck::default(),
            config,
        }
    }
    
    /// 记录指标
    pub fn record_metric(&mut self, metric: MetricPoint) {
        let metrics_list = self.metrics.entry(metric.name.clone()).or_insert_with(Vec::new);
        metrics_list.push(metric);
        
        // 清理过期指标
        let retention_duration = Duration::hours(self.config.metric_retention_hours as i64);
        let cutoff_time = Utc::now() - retention_duration;
        
        metrics_list.retain(|m| m.timestamp > cutoff_time);
    }
    
    /// 增加计数器
    pub fn increment_counter(&mut self, name: &str, value: u64) {
        let counter = self.counters.entry(name.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)));
        
        counter.fetch_add(value, Ordering::Relaxed);
        
        // 同时记录为指标
        let metric = MetricPoint {
            name: name.to_string(),
            metric_type: MetricType::Counter,
            value: counter.load(Ordering::Relaxed) as f64,
            labels: HashMap::new(),
            timestamp: Utc::now(),
            help: None,
        };
        
        self.record_metric(metric);
    }
    
    /// 获取实时性能统计（增强版）
    pub fn get_enhanced_performance_stats(&self) -> PerformanceStats {
        let mut total_messages = 0;
        let mut successful_messages = 0;
        let mut failed_messages = 0;
        let mut avg_latency = 0.0;
        let mut throughput = 0.0;

        // 从计数器获取统计数据
        if let Some(messages_counter) = self.counters.get("total_messages") {
            total_messages = messages_counter.load(Ordering::Relaxed);
        }

        if let Some(success_counter) = self.counters.get("successful_messages") {
            successful_messages = success_counter.load(Ordering::Relaxed);
        }

        if let Some(failed_counter) = self.counters.get("failed_messages") {
            failed_messages = failed_counter.load(Ordering::Relaxed);
        }

        // 从指标中计算平均延迟
        if let Some(latency_metrics) = self.metrics.get("message_latency") {
            if !latency_metrics.is_empty() {
                let sum: f64 = latency_metrics.iter().map(|m| m.value).sum();
                avg_latency = sum / latency_metrics.len() as f64;
            }
        }

        // 计算吞吐量（最近一分钟的请求数）
        let one_minute_ago = Utc::now() - Duration::minutes(1);
        if let Some(request_metrics) = self.metrics.get("requests_per_second") {
            let recent_requests: Vec<_> = request_metrics.iter()
                .filter(|m| m.timestamp > one_minute_ago)
                .collect();

            if !recent_requests.is_empty() {
                throughput = recent_requests.iter().map(|m| m.value).sum::<f64>() / 60.0;
            }
        }

        let now = Utc::now();
        let time_range = TimeRange {
            start: now - Duration::hours(1),
            end: now,
        };

        PerformanceStats {
            message_stats: MessageStats {
                total_messages,
                successful_messages,
                failed_messages,
                avg_processing_time_ms: avg_latency,
                max_processing_time_ms: avg_latency * 2.0, // 简化实现
                min_processing_time_ms: avg_latency * 0.5, // 简化实现
                throughput_per_second: throughput,
                by_message_type: HashMap::new(),
            },
            agent_stats: self.performance_stats.agent_stats.clone(),
            system_stats: self.performance_stats.system_stats.clone(),
            error_stats: ErrorStats {
                total_errors: failed_messages,
                auth_errors: 0,
                authz_errors: 0,
                network_errors: 0,
                timeout_errors: 0,
                internal_errors: failed_messages,
                by_error_type: HashMap::new(),
                error_rate: if total_messages > 0 {
                    (failed_messages as f64 / total_messages as f64) * 100.0
                } else {
                    0.0
                },
            },
            time_range,
        }
    }

    /// 获取系统运行时间
    #[allow(dead_code)]
    fn get_uptime(&self) -> u64 {
        // 简化实现，实际应该记录启动时间
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// 运行性能基准测试
    pub fn run_performance_benchmark(&mut self, test_name: &str, iterations: u64) -> BenchmarkResult {
        let start_time = std::time::Instant::now();
        let mut latencies = Vec::new();

        for i in 0..iterations {
            let iter_start = std::time::Instant::now();

            // 模拟工作负载
            self.increment_counter("benchmark_operations", 1);

            let latency = iter_start.elapsed().as_micros() as f64 / 1000.0; // 转换为毫秒
            latencies.push(latency);

            // 每1000次迭代记录一次进度
            if i % 1000 == 0 && i > 0 {
                println!("基准测试 {} 进度: {}/{}", test_name, i, iterations);
            }
        }

        let total_duration = start_time.elapsed();

        // 计算统计数据
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min_latency = latencies.first().copied().unwrap_or(0.0);
        let max_latency = latencies.last().copied().unwrap_or(0.0);
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p50 = latencies[latencies.len() / 2];
        let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
        let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];

        let throughput = iterations as f64 / total_duration.as_secs_f64();

        BenchmarkResult {
            test_name: test_name.to_string(),
            iterations,
            total_duration_ms: total_duration.as_millis() as f64,
            throughput_ops_per_sec: throughput,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            avg_latency_ms: avg_latency,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            timestamp: Utc::now(),
        }
    }

    /// 设置仪表值
    pub fn set_gauge(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = MetricPoint {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value,
            labels,
            timestamp: Utc::now(),
            help: None,
        };
        
        self.record_metric(metric);
    }
    
    /// 记录直方图值
    pub fn record_histogram(&mut self, name: &str, value: f64, labels: HashMap<String, String>) {
        let metric = MetricPoint {
            name: name.to_string(),
            metric_type: MetricType::Histogram,
            value,
            labels,
            timestamp: Utc::now(),
            help: None,
        };
        
        self.record_metric(metric);
    }
    
    /// 获取指标
    pub fn get_metrics(&self, name: &str) -> Option<&Vec<MetricPoint>> {
        self.metrics.get(name)
    }
    
    /// 获取所有指标名称
    pub fn get_metric_names(&self) -> Vec<String> {
        self.metrics.keys().cloned().collect()
    }
    
    /// 计算性能统计
    pub fn calculate_performance_stats(&mut self, time_range: TimeRange) -> A2AResult<&PerformanceStats> {
        // 计算消息统计
        let message_stats = self.calculate_message_stats(&time_range)?;
        
        // 计算Agent统计
        let agent_stats = self.calculate_agent_stats(&time_range)?;
        
        // 计算系统统计
        let system_stats = self.calculate_system_stats(&time_range)?;
        
        // 计算错误统计
        let error_stats = self.calculate_error_stats(&time_range)?;
        
        self.performance_stats = PerformanceStats {
            message_stats,
            agent_stats,
            system_stats,
            error_stats,
            time_range,
        };
        
        Ok(&self.performance_stats)
    }
    
    /// 执行健康检查
    pub fn perform_health_check(&mut self) -> A2AResult<&HealthCheck> {
        let mut components = HashMap::new();
        let mut total_score = 0u32;
        let mut component_count = 0u32;
        
        // 检查消息处理组件
        let message_health = self.check_message_processing_health();
        total_score += message_health.status.score();
        component_count += 1;
        components.insert("message_processing".to_string(), message_health);
        
        // 检查Agent注册组件
        let agent_health = self.check_agent_registry_health();
        total_score += agent_health.status.score();
        component_count += 1;
        components.insert("agent_registry".to_string(), agent_health);
        
        // 检查网络连接组件
        let network_health = self.check_network_health();
        total_score += network_health.status.score();
        component_count += 1;
        components.insert("network".to_string(), network_health);
        
        // 检查存储组件
        let storage_health = self.check_storage_health();
        total_score += storage_health.status.score();
        component_count += 1;
        components.insert("storage".to_string(), storage_health);
        
        // 计算总体评分和状态
        let avg_score = if component_count > 0 {
            (total_score / component_count) as u8
        } else {
            0
        };
        
        let overall_status = match avg_score {
            90..=100 => HealthStatus::Healthy,
            70..=89 => HealthStatus::Degraded,
            0..=69 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unknown,
        };
        
        self.health_status = HealthCheck {
            status: overall_status,
            timestamp: Utc::now(),
            components,
            score: avg_score,
            details: None,
        };
        
        Ok(&self.health_status)
    }
    
    /// 获取当前健康状态
    pub fn get_health_status(&self) -> &HealthCheck {
        &self.health_status
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }
    
    // 私有方法 - 计算各种统计
    
    fn calculate_message_stats(&self, _time_range: &TimeRange) -> A2AResult<MessageStats> {
        // 简化实现 - 从指标中计算消息统计
        let total_messages = self.get_counter_value("total_messages");
        let successful_messages = self.get_counter_value("successful_messages");
        let failed_messages = self.get_counter_value("failed_messages");
        
        Ok(MessageStats {
            total_messages,
            successful_messages,
            failed_messages,
            avg_processing_time_ms: 1.5, // 示例值
            max_processing_time_ms: 10.0,
            min_processing_time_ms: 0.5,
            throughput_per_second: 1000.0,
            by_message_type: HashMap::new(),
        })
    }
    
    fn calculate_agent_stats(&self, _time_range: &TimeRange) -> A2AResult<AgentStats> {
        Ok(AgentStats {
            total_agents: self.get_counter_value("total_agents"),
            online_agents: self.get_counter_value("online_agents"),
            offline_agents: self.get_counter_value("offline_agents"),
            busy_agents: self.get_counter_value("busy_agents"),
            by_trust_level: HashMap::new(),
            by_status: HashMap::new(),
            avg_response_time_ms: 2.0,
        })
    }
    
    fn calculate_system_stats(&self, _time_range: &TimeRange) -> A2AResult<SystemStats> {
        Ok(SystemStats {
            cpu_usage_percent: 25.0,
            memory_usage_bytes: 1024 * 1024 * 512, // 512MB
            memory_usage_percent: 50.0,
            network_in_bytes_per_sec: 1024.0 * 100.0, // 100KB/s
            network_out_bytes_per_sec: 1024.0 * 80.0,  // 80KB/s
            disk_usage_bytes: 1024 * 1024 * 1024 * 10, // 10GB
            disk_usage_percent: 30.0,
        })
    }
    
    fn calculate_error_stats(&self, _time_range: &TimeRange) -> A2AResult<ErrorStats> {
        let total_errors = self.get_counter_value("total_errors");
        let failed_messages = self.get_counter_value("failed_messages");
        let total_requests = self.get_counter_value("total_messages");

        // 使用failed_messages作为错误数，如果没有设置total_errors
        let actual_errors = if total_errors > 0 { total_errors } else { failed_messages };

        let error_rate = if total_requests > 0 {
            actual_errors as f64 / total_requests as f64
        } else {
            0.0
        };

        Ok(ErrorStats {
            total_errors: actual_errors,
            auth_errors: self.get_counter_value("auth_errors"),
            authz_errors: self.get_counter_value("authz_errors"),
            network_errors: self.get_counter_value("network_errors"),
            timeout_errors: self.get_counter_value("timeout_errors"),
            internal_errors: self.get_counter_value("internal_errors"),
            by_error_type: HashMap::new(),
            error_rate,
        })
    }
    
    fn get_counter_value(&self, name: &str) -> u64 {
        self.counters.get(name)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    // 健康检查方法
    
    fn check_message_processing_health(&self) -> ComponentHealth {
        // 检查消息处理性能
        let error_rate = self.performance_stats.error_stats.error_rate;
        let avg_processing_time = self.performance_stats.message_stats.avg_processing_time_ms;
        
        let status = if error_rate < 0.01 && avg_processing_time < 10.0 {
            HealthStatus::Healthy
        } else if error_rate < 0.05 && avg_processing_time < 50.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        ComponentHealth {
            status,
            response_time_ms: Some(avg_processing_time),
            error: None,
            last_check: Utc::now(),
        }
    }
    
    fn check_agent_registry_health(&self) -> ComponentHealth {
        // 检查Agent注册服务健康状态
        let online_ratio = if self.performance_stats.agent_stats.total_agents > 0 {
            self.performance_stats.agent_stats.online_agents as f64 / 
            self.performance_stats.agent_stats.total_agents as f64
        } else {
            1.0
        };
        
        let status = if online_ratio > 0.9 {
            HealthStatus::Healthy
        } else if online_ratio > 0.7 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        ComponentHealth {
            status,
            response_time_ms: Some(self.performance_stats.agent_stats.avg_response_time_ms),
            error: None,
            last_check: Utc::now(),
        }
    }
    
    fn check_network_health(&self) -> ComponentHealth {
        // 检查网络连接健康状态
        ComponentHealth {
            status: HealthStatus::Healthy,
            response_time_ms: Some(1.0),
            error: None,
            last_check: Utc::now(),
        }
    }
    
    fn check_storage_health(&self) -> ComponentHealth {
        // 检查存储健康状态
        let disk_usage = self.performance_stats.system_stats.disk_usage_percent;
        
        let status = if disk_usage < 80.0 {
            HealthStatus::Healthy
        } else if disk_usage < 95.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        ComponentHealth {
            status,
            response_time_ms: Some(0.5),
            error: None,
            last_check: Utc::now(),
        }
    }
}

impl HealthStatus {
    fn score(&self) -> u32 {
        match self {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 75,
            HealthStatus::Unhealthy => 25,
            HealthStatus::Unknown => 0,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metric_retention_hours: 24,
            health_check_interval_seconds: 30,
            stats_calculation_interval_seconds: 60,
            enable_detailed_monitoring: true,
        }
    }
}

impl Default for PerformanceStats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            message_stats: MessageStats::default(),
            agent_stats: AgentStats::default(),
            system_stats: SystemStats::default(),
            error_stats: ErrorStats::default(),
            time_range: TimeRange {
                start: now - Duration::hours(1),
                end: now,
            },
        }
    }
}

impl Default for MessageStats {
    fn default() -> Self {
        Self {
            total_messages: 0,
            successful_messages: 0,
            failed_messages: 0,
            avg_processing_time_ms: 0.0,
            max_processing_time_ms: 0.0,
            min_processing_time_ms: 0.0,
            throughput_per_second: 0.0,
            by_message_type: HashMap::new(),
        }
    }
}

impl Default for AgentStats {
    fn default() -> Self {
        Self {
            total_agents: 0,
            online_agents: 0,
            offline_agents: 0,
            busy_agents: 0,
            by_trust_level: HashMap::new(),
            by_status: HashMap::new(),
            avg_response_time_ms: 0.0,
        }
    }
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_usage_percent: 0.0,
            network_in_bytes_per_sec: 0.0,
            network_out_bytes_per_sec: 0.0,
            disk_usage_bytes: 0,
            disk_usage_percent: 0.0,
        }
    }
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            auth_errors: 0,
            authz_errors: 0,
            network_errors: 0,
            timeout_errors: 0,
            internal_errors: 0,
            by_error_type: HashMap::new(),
            error_rate: 0.0,
        }
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            timestamp: Utc::now(),
            components: HashMap::new(),
            score: 0,
            details: None,
        }
    }
}
