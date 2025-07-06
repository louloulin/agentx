//! 分布式链路追踪系统
//! 
//! 提供跨服务的请求追踪、性能分析和故障诊断功能，
//! 支持OpenTelemetry标准和自定义追踪格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, debug, warn};
use uuid::Uuid;
use agentx_a2a::A2AResult;

/// 分布式追踪管理器
pub struct DistributedTracingManager {
    /// 追踪配置
    config: TracingConfig,
    /// 活跃的追踪会话
    active_traces: Arc<RwLock<HashMap<String, TraceSession>>>,
    /// 追踪数据存储
    trace_storage: Arc<RwLock<TraceStorage>>,
    /// 采样器
    sampler: Arc<RwLock<TracingSampler>>,
    /// 导出器
    exporters: Vec<Box<dyn TraceExporter>>,
}

/// 追踪配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// 是否启用追踪
    pub enabled: bool,
    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
    /// 最大追踪深度
    pub max_trace_depth: u32,
    /// 追踪数据保留时间（小时）
    pub retention_hours: u64,
    /// 批量导出大小
    pub batch_export_size: usize,
    /// 导出超时时间（秒）
    pub export_timeout_seconds: u64,
    /// 是否启用性能分析
    pub enable_performance_analysis: bool,
    /// 是否启用错误追踪
    pub enable_error_tracking: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.1, // 10%采样率
            max_trace_depth: 50,
            retention_hours: 24,
            batch_export_size: 100,
            export_timeout_seconds: 30,
            enable_performance_analysis: true,
            enable_error_tracking: true,
        }
    }
}

/// 追踪会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSession {
    /// 追踪ID
    pub trace_id: String,
    /// 根Span ID
    pub root_span_id: String,
    /// 服务名称
    pub service_name: String,
    /// 操作名称
    pub operation_name: String,
    /// 开始时间
    pub start_time: SystemTime,
    /// 结束时间
    pub end_time: Option<SystemTime>,
    /// 状态
    pub status: TraceStatus,
    /// 标签
    pub tags: HashMap<String, String>,
    /// 子Span列表
    pub spans: Vec<Span>,
    /// 错误信息
    pub errors: Vec<TraceError>,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
}

/// Span（追踪片段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span ID
    pub span_id: String,
    /// 父Span ID
    pub parent_span_id: Option<String>,
    /// 追踪ID
    pub trace_id: String,
    /// 服务名称
    pub service_name: String,
    /// 操作名称
    pub operation_name: String,
    /// 开始时间
    pub start_time: SystemTime,
    /// 结束时间
    pub end_time: Option<SystemTime>,
    /// 持续时间
    pub duration: Option<Duration>,
    /// 状态
    pub status: SpanStatus,
    /// 标签
    pub tags: HashMap<String, String>,
    /// 日志事件
    pub logs: Vec<LogEvent>,
    /// 子Span数量
    pub child_count: u32,
}

/// 追踪状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceStatus {
    /// 进行中
    InProgress,
    /// 成功完成
    Completed,
    /// 出现错误
    Error,
    /// 超时
    Timeout,
    /// 已取消
    Cancelled,
}

/// Span状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 进行中
    InProgress,
    /// 成功
    Ok,
    /// 错误
    Error,
    /// 取消
    Cancelled,
}

/// 日志事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// 时间戳
    pub timestamp: SystemTime,
    /// 日志级别
    pub level: LogLevel,
    /// 消息
    pub message: String,
    /// 字段
    pub fields: HashMap<String, String>,
}

/// 日志级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// 追踪错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceError {
    /// 错误ID
    pub error_id: String,
    /// Span ID
    pub span_id: String,
    /// 错误类型
    pub error_type: String,
    /// 错误消息
    pub message: String,
    /// 错误堆栈
    pub stack_trace: Option<String>,
    /// 发生时间
    pub timestamp: SystemTime,
    /// 错误标签
    pub tags: HashMap<String, String>,
}

/// 性能指标
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 总持续时间
    pub total_duration_ms: f64,
    /// 网络延迟
    pub network_latency_ms: f64,
    /// 数据库查询时间
    pub database_time_ms: f64,
    /// 外部服务调用时间
    pub external_service_time_ms: f64,
    /// CPU时间
    pub cpu_time_ms: f64,
    /// 内存使用量
    pub memory_usage_bytes: u64,
    /// 网络IO字节数
    pub network_io_bytes: u64,
    /// 磁盘IO字节数
    pub disk_io_bytes: u64,
}

/// 追踪数据存储
pub struct TraceStorage {
    /// 已完成的追踪
    completed_traces: HashMap<String, TraceSession>,
    /// 追踪索引（按服务名）
    service_index: HashMap<String, Vec<String>>,
    /// 追踪索引（按操作名）
    operation_index: HashMap<String, Vec<String>>,
    /// 错误索引
    error_index: HashMap<String, Vec<String>>,
}

/// 追踪采样器
pub struct TracingSampler {
    /// 采样率
    sampling_rate: f64,
    /// 采样计数器
    sample_counter: u64,
}

/// 追踪导出器trait
pub trait TraceExporter: Send + Sync {
    /// 导出追踪数据
    fn export_traces(&self, traces: &[TraceSession]) -> A2AResult<()>;
    
    /// 导出器名称
    fn name(&self) -> &str;
}

/// 控制台导出器
pub struct ConsoleExporter;

/// Jaeger导出器
pub struct JaegerExporter {
    /// Jaeger端点
    endpoint: String,
    /// 服务名称
    #[allow(dead_code)]
    service_name: String,
}

/// OpenTelemetry导出器
pub struct OpenTelemetryExporter {
    /// OTLP端点
    endpoint: String,
    /// 认证头
    #[allow(dead_code)]
    headers: HashMap<String, String>,
}

impl DistributedTracingManager {
    /// 创建新的分布式追踪管理器
    pub fn new(config: TracingConfig) -> Self {
        info!("🔍 创建分布式追踪管理器");
        
        let sampler = Arc::new(RwLock::new(TracingSampler::new(config.sampling_rate)));
        let storage = TraceStorage::new();

        Self {
            config,
            active_traces: Arc::new(RwLock::new(HashMap::new())),
            trace_storage: Arc::new(RwLock::new(storage)),
            sampler,
            exporters: Vec::new(),
        }
    }
    
    /// 添加导出器
    pub fn add_exporter(&mut self, exporter: Box<dyn TraceExporter>) {
        info!("添加追踪导出器: {}", exporter.name());
        self.exporters.push(exporter);
    }
    
    /// 开始新的追踪
    pub async fn start_trace(
        &self,
        service_name: String,
        operation_name: String,
        tags: HashMap<String, String>,
    ) -> A2AResult<String> {
        if !self.config.enabled {
            return Ok(String::new());
        }
        
        // 检查是否应该采样
        {
            let mut sampler = self.sampler.write().await;
            if !sampler.should_sample() {
                return Ok(String::new());
            }
        }
        
        let trace_id = Uuid::new_v4().to_string();
        let root_span_id = Uuid::new_v4().to_string();
        
        let trace_session = TraceSession {
            trace_id: trace_id.clone(),
            root_span_id: root_span_id.clone(),
            service_name: service_name.clone(),
            operation_name: operation_name.clone(),
            start_time: SystemTime::now(),
            end_time: None,
            status: TraceStatus::InProgress,
            tags,
            spans: Vec::new(),
            errors: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        };
        
        let mut active_traces = self.active_traces.write().await;
        active_traces.insert(trace_id.clone(), trace_session);
        
        debug!("开始新追踪: {} (服务: {}, 操作: {})", 
               trace_id, service_name, operation_name);
        
        Ok(trace_id)
    }
    
    /// 创建新的Span
    pub async fn start_span(
        &self,
        trace_id: &str,
        parent_span_id: Option<String>,
        service_name: String,
        operation_name: String,
        tags: HashMap<String, String>,
    ) -> A2AResult<String> {
        if trace_id.is_empty() {
            return Ok(String::new());
        }
        
        let span_id = Uuid::new_v4().to_string();
        
        let span = Span {
            span_id: span_id.clone(),
            parent_span_id,
            trace_id: trace_id.to_string(),
            service_name,
            operation_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::InProgress,
            tags,
            logs: Vec::new(),
            child_count: 0,
        };
        
        let mut active_traces = self.active_traces.write().await;
        if let Some(trace_session) = active_traces.get_mut(trace_id) {
            let parent_span_id = span.parent_span_id.clone();
            trace_session.spans.push(span);

            // 更新父Span的子Span计数
            if let Some(parent_id) = parent_span_id {
                for existing_span in &mut trace_session.spans {
                    if existing_span.span_id == parent_id {
                        existing_span.child_count += 1;
                        break;
                    }
                }
            }
        }
        
        debug!("创建新Span: {} (追踪: {})", span_id, trace_id);

        Ok(span_id)
    }

    /// 完成Span
    pub async fn finish_span(
        &self,
        trace_id: &str,
        span_id: &str,
        status: SpanStatus,
        tags: Option<HashMap<String, String>>,
    ) -> A2AResult<()> {
        if trace_id.is_empty() || span_id.is_empty() {
            return Ok(());
        }

        let mut active_traces = self.active_traces.write().await;
        if let Some(trace_session) = active_traces.get_mut(trace_id) {
            for span in &mut trace_session.spans {
                if span.span_id == span_id {
                    span.end_time = Some(SystemTime::now());
                    span.status = status;

                    // 计算持续时间
                    if let Ok(duration) = span.end_time.unwrap().duration_since(span.start_time) {
                        span.duration = Some(duration);
                    }

                    // 添加额外标签
                    if let Some(additional_tags) = tags {
                        span.tags.extend(additional_tags);
                    }

                    debug!("完成Span: {} (状态: {:?}, 持续时间: {:?})",
                           span_id, span.status, span.duration);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 添加Span日志
    pub async fn add_span_log(
        &self,
        trace_id: &str,
        span_id: &str,
        level: LogLevel,
        message: String,
        fields: HashMap<String, String>,
    ) -> A2AResult<()> {
        if trace_id.is_empty() || span_id.is_empty() {
            return Ok(());
        }

        let log_event = LogEvent {
            timestamp: SystemTime::now(),
            level,
            message,
            fields,
        };

        let mut active_traces = self.active_traces.write().await;
        if let Some(trace_session) = active_traces.get_mut(trace_id) {
            for span in &mut trace_session.spans {
                if span.span_id == span_id {
                    span.logs.push(log_event);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 记录错误
    pub async fn record_error(
        &self,
        trace_id: &str,
        span_id: &str,
        error_type: String,
        message: String,
        stack_trace: Option<String>,
        tags: HashMap<String, String>,
    ) -> A2AResult<()> {
        if !self.config.enable_error_tracking || trace_id.is_empty() {
            return Ok(());
        }

        let error = TraceError {
            error_id: Uuid::new_v4().to_string(),
            span_id: span_id.to_string(),
            error_type,
            message,
            stack_trace,
            timestamp: SystemTime::now(),
            tags,
        };

        let mut active_traces = self.active_traces.write().await;
        if let Some(trace_session) = active_traces.get_mut(trace_id) {
            trace_session.errors.push(error);
            trace_session.status = TraceStatus::Error;
        }

        Ok(())
    }

    /// 完成追踪
    pub async fn finish_trace(
        &self,
        trace_id: &str,
        status: TraceStatus,
        performance_metrics: Option<PerformanceMetrics>,
    ) -> A2AResult<()> {
        if trace_id.is_empty() {
            return Ok(());
        }

        let mut active_traces = self.active_traces.write().await;
        if let Some(mut trace_session) = active_traces.remove(trace_id) {
            trace_session.end_time = Some(SystemTime::now());
            trace_session.status = status;

            if let Some(metrics) = performance_metrics {
                trace_session.performance_metrics = metrics;
            }

            // 计算总持续时间
            if let Ok(duration) = trace_session.end_time.unwrap().duration_since(trace_session.start_time) {
                trace_session.performance_metrics.total_duration_ms = duration.as_secs_f64() * 1000.0;
            }

            debug!("完成追踪: {} (状态: {:?}, 持续时间: {:.2}ms)",
                   trace_id, trace_session.status, trace_session.performance_metrics.total_duration_ms);

            // 存储到持久化存储
            self.store_trace(trace_session).await?;
        }

        Ok(())
    }

    /// 存储追踪数据
    async fn store_trace(&self, trace_session: TraceSession) -> A2AResult<()> {
        let mut storage = self.trace_storage.write().await;

        // 添加到主存储
        let trace_id = trace_session.trace_id.clone();
        storage.completed_traces.insert(trace_id.clone(), trace_session.clone());

        // 更新索引
        storage.service_index
            .entry(trace_session.service_name.clone())
            .or_insert_with(Vec::new)
            .push(trace_id.clone());

        storage.operation_index
            .entry(trace_session.operation_name.clone())
            .or_insert_with(Vec::new)
            .push(trace_id.clone());

        // 如果有错误，添加到错误索引
        if !trace_session.errors.is_empty() {
            storage.error_index
                .entry("has_errors".to_string())
                .or_insert_with(Vec::new)
                .push(trace_id.clone());
        }

        // 导出追踪数据
        self.export_trace(&trace_session).await?;

        Ok(())
    }

    /// 导出追踪数据
    async fn export_trace(&self, trace_session: &TraceSession) -> A2AResult<()> {
        for exporter in &self.exporters {
            if let Err(e) = exporter.export_traces(&[trace_session.clone()]) {
                warn!("导出追踪数据失败 (导出器: {}): {}", exporter.name(), e);
            }
        }
        Ok(())
    }

    /// 查询追踪数据
    pub async fn query_traces(
        &self,
        service_name: Option<&str>,
        operation_name: Option<&str>,
        status: Option<TraceStatus>,
        time_range: Option<(SystemTime, SystemTime)>,
        limit: Option<usize>,
    ) -> A2AResult<Vec<TraceSession>> {
        let storage = self.trace_storage.read().await;
        let mut results = Vec::new();

        // 根据索引筛选
        let candidate_trace_ids = if let Some(service) = service_name {
            storage.service_index.get(service).cloned().unwrap_or_default()
        } else if let Some(operation) = operation_name {
            storage.operation_index.get(operation).cloned().unwrap_or_default()
        } else {
            storage.completed_traces.keys().cloned().collect()
        };

        for trace_id in candidate_trace_ids {
            if let Some(trace) = storage.completed_traces.get(&trace_id) {
                // 应用过滤条件
                if let Some(required_status) = &status {
                    if trace.status != *required_status {
                        continue;
                    }
                }

                if let Some((start, end)) = time_range {
                    if trace.start_time < start || trace.start_time > end {
                        continue;
                    }
                }

                results.push(trace.clone());

                // 应用限制
                if let Some(max_results) = limit {
                    if results.len() >= max_results {
                        break;
                    }
                }
            }
        }

        // 按开始时间排序（最新的在前）
        results.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        Ok(results)
    }

    /// 获取追踪统计信息
    pub async fn get_trace_statistics(&self) -> A2AResult<TraceStatistics> {
        let storage = self.trace_storage.read().await;
        let active_traces = self.active_traces.read().await;

        let total_traces = storage.completed_traces.len() + active_traces.len();
        let completed_traces = storage.completed_traces.len();
        let active_traces_count = active_traces.len();

        let mut error_traces = 0;
        let mut total_duration_ms = 0.0;
        let mut service_counts = HashMap::new();

        for trace in storage.completed_traces.values() {
            if trace.status == TraceStatus::Error {
                error_traces += 1;
            }

            total_duration_ms += trace.performance_metrics.total_duration_ms;

            *service_counts.entry(trace.service_name.clone()).or_insert(0) += 1;
        }

        let avg_duration_ms = if completed_traces > 0 {
            total_duration_ms / completed_traces as f64
        } else {
            0.0
        };

        let error_rate = if completed_traces > 0 {
            error_traces as f64 / completed_traces as f64
        } else {
            0.0
        };

        Ok(TraceStatistics {
            total_traces,
            completed_traces,
            active_traces: active_traces_count,
            error_traces,
            error_rate,
            avg_duration_ms,
            service_counts,
        })
    }

    /// 清理过期的追踪数据
    pub async fn cleanup_expired_traces(&self) -> A2AResult<usize> {
        let retention_duration = Duration::from_secs(self.config.retention_hours * 3600);
        let cutoff_time = SystemTime::now() - retention_duration;

        let mut storage = self.trace_storage.write().await;
        let mut removed_count = 0;

        // 收集需要删除的追踪ID
        let mut to_remove = Vec::new();
        for (trace_id, trace) in &storage.completed_traces {
            if trace.start_time < cutoff_time {
                to_remove.push(trace_id.clone());
            }
        }

        // 删除过期追踪
        for trace_id in &to_remove {
            if let Some(trace) = storage.completed_traces.remove(trace_id) {
                removed_count += 1;

                // 从索引中删除
                if let Some(service_traces) = storage.service_index.get_mut(&trace.service_name) {
                    service_traces.retain(|id| id != trace_id);
                }

                if let Some(operation_traces) = storage.operation_index.get_mut(&trace.operation_name) {
                    operation_traces.retain(|id| id != trace_id);
                }

                if !trace.errors.is_empty() {
                    if let Some(error_traces) = storage.error_index.get_mut("has_errors") {
                        error_traces.retain(|id| id != trace_id);
                    }
                }
            }
        }

        info!("清理了 {} 个过期追踪", removed_count);
        Ok(removed_count)
    }
}

/// 追踪统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStatistics {
    /// 总追踪数
    pub total_traces: usize,
    /// 已完成追踪数
    pub completed_traces: usize,
    /// 活跃追踪数
    pub active_traces: usize,
    /// 错误追踪数
    pub error_traces: usize,
    /// 错误率
    pub error_rate: f64,
    /// 平均持续时间（毫秒）
    pub avg_duration_ms: f64,
    /// 各服务的追踪数量
    pub service_counts: HashMap<String, usize>,
}

impl TraceStorage {
    fn new() -> Self {
        Self {
            completed_traces: HashMap::new(),
            service_index: HashMap::new(),
            operation_index: HashMap::new(),
            error_index: HashMap::new(),
        }
    }
}

impl TracingSampler {
    fn new(sampling_rate: f64) -> Self {
        Self {
            sampling_rate: sampling_rate.clamp(0.0, 1.0),
            sample_counter: 0,
        }
    }

    fn should_sample(&mut self) -> bool {
        self.sample_counter += 1;

        if self.sampling_rate >= 1.0 {
            return true;
        }

        if self.sampling_rate <= 0.0 {
            return false;
        }

        // 简单的确定性采样
        (self.sample_counter as f64 * self.sampling_rate) % 1.0 < self.sampling_rate
    }
}

impl TraceExporter for ConsoleExporter {
    fn export_traces(&self, traces: &[TraceSession]) -> A2AResult<()> {
        for trace in traces {
            println!("🔍 追踪导出: {} (服务: {}, 操作: {}, 状态: {:?}, 持续时间: {:.2}ms)",
                     trace.trace_id,
                     trace.service_name,
                     trace.operation_name,
                     trace.status,
                     trace.performance_metrics.total_duration_ms);

            for span in &trace.spans {
                println!("  📊 Span: {} (操作: {}, 状态: {:?}, 持续时间: {:?})",
                         span.span_id,
                         span.operation_name,
                         span.status,
                         span.duration);
            }

            if !trace.errors.is_empty() {
                println!("  ❌ 错误数量: {}", trace.errors.len());
                for error in &trace.errors {
                    println!("    - {}: {}", error.error_type, error.message);
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

impl JaegerExporter {
    pub fn new(endpoint: String, service_name: String) -> Self {
        Self {
            endpoint,
            service_name,
        }
    }
}

impl TraceExporter for JaegerExporter {
    fn export_traces(&self, traces: &[TraceSession]) -> A2AResult<()> {
        // 在实际实现中，这里会将追踪数据转换为Jaeger格式并发送到Jaeger收集器
        debug!("导出 {} 个追踪到Jaeger (端点: {})", traces.len(), self.endpoint);

        for trace in traces {
            debug!("导出追踪到Jaeger: {} (服务: {})", trace.trace_id, trace.service_name);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "jaeger"
    }
}

impl OpenTelemetryExporter {
    pub fn new(endpoint: String, headers: HashMap<String, String>) -> Self {
        Self {
            endpoint,
            headers,
        }
    }
}

impl TraceExporter for OpenTelemetryExporter {
    fn export_traces(&self, traces: &[TraceSession]) -> A2AResult<()> {
        // 在实际实现中，这里会将追踪数据转换为OTLP格式并发送到OpenTelemetry收集器
        debug!("导出 {} 个追踪到OpenTelemetry (端点: {})", traces.len(), self.endpoint);

        for trace in traces {
            debug!("导出追踪到OpenTelemetry: {} (服务: {})", trace.trace_id, trace.service_name);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "opentelemetry"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::time::Duration;

    fn create_test_tracing_manager() -> DistributedTracingManager {
        let config = TracingConfig {
            enabled: true,
            sampling_rate: 1.0, // 100%采样用于测试
            max_trace_depth: 10,
            retention_hours: 1,
            batch_export_size: 10,
            export_timeout_seconds: 5,
            enable_performance_analysis: true,
            enable_error_tracking: true,
        };

        let mut manager = DistributedTracingManager::new(config);
        manager.add_exporter(Box::new(ConsoleExporter));
        manager
    }

    #[tokio::test]
    async fn test_tracing_manager_creation() {
        let manager = create_test_tracing_manager();
        assert!(manager.config.enabled);
        assert_eq!(manager.config.sampling_rate, 1.0);
        assert_eq!(manager.exporters.len(), 1);
    }

    #[tokio::test]
    async fn test_trace_lifecycle() {
        let manager = create_test_tracing_manager();

        // 开始追踪
        let mut tags = HashMap::new();
        tags.insert("version".to_string(), "1.0.0".to_string());

        let trace_id = manager.start_trace(
            "test_service".to_string(),
            "test_operation".to_string(),
            tags,
        ).await.unwrap();

        assert!(!trace_id.is_empty());

        // 创建Span
        let mut span_tags = HashMap::new();
        span_tags.insert("component".to_string(), "database".to_string());

        let span_id = manager.start_span(
            &trace_id,
            None,
            "test_service".to_string(),
            "db_query".to_string(),
            span_tags,
        ).await.unwrap();

        assert!(!span_id.is_empty());

        // 添加日志
        let mut log_fields = HashMap::new();
        log_fields.insert("query".to_string(), "SELECT * FROM users".to_string());

        manager.add_span_log(
            &trace_id,
            &span_id,
            LogLevel::Info,
            "执行数据库查询".to_string(),
            log_fields,
        ).await.unwrap();

        // 完成Span
        manager.finish_span(
            &trace_id,
            &span_id,
            SpanStatus::Ok,
            None,
        ).await.unwrap();

        // 完成追踪
        let performance_metrics = PerformanceMetrics {
            total_duration_ms: 150.0,
            database_time_ms: 100.0,
            network_latency_ms: 25.0,
            ..Default::default()
        };

        manager.finish_trace(
            &trace_id,
            TraceStatus::Completed,
            Some(performance_metrics),
        ).await.unwrap();

        // 验证追踪已存储
        let traces = manager.query_traces(
            Some("test_service"),
            None,
            None,
            None,
            None,
        ).await.unwrap();

        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].trace_id, trace_id);
        assert_eq!(traces[0].status, TraceStatus::Completed);
        assert_eq!(traces[0].spans.len(), 1);
        assert_eq!(traces[0].spans[0].logs.len(), 1);
    }

    #[tokio::test]
    async fn test_error_tracking() {
        let manager = create_test_tracing_manager();

        let trace_id = manager.start_trace(
            "error_service".to_string(),
            "error_operation".to_string(),
            HashMap::new(),
        ).await.unwrap();

        let span_id = manager.start_span(
            &trace_id,
            None,
            "error_service".to_string(),
            "failing_operation".to_string(),
            HashMap::new(),
        ).await.unwrap();

        // 记录错误
        let mut error_tags = HashMap::new();
        error_tags.insert("error_code".to_string(), "500".to_string());

        manager.record_error(
            &trace_id,
            &span_id,
            "DatabaseError".to_string(),
            "连接超时".to_string(),
            Some("stack trace here".to_string()),
            error_tags,
        ).await.unwrap();

        manager.finish_span(&trace_id, &span_id, SpanStatus::Error, None).await.unwrap();
        manager.finish_trace(&trace_id, TraceStatus::Error, None).await.unwrap();

        // 验证错误追踪
        let traces = manager.query_traces(
            None,
            None,
            Some(TraceStatus::Error),
            None,
            None,
        ).await.unwrap();

        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].status, TraceStatus::Error);
        assert_eq!(traces[0].errors.len(), 1);
        assert_eq!(traces[0].errors[0].error_type, "DatabaseError");
    }

    #[tokio::test]
    async fn test_trace_statistics() {
        let manager = create_test_tracing_manager();

        // 创建多个追踪
        for i in 0..5 {
            let trace_id = manager.start_trace(
                format!("service_{}", i % 2),
                "test_operation".to_string(),
                HashMap::new(),
            ).await.unwrap();

            let status = if i == 4 { TraceStatus::Error } else { TraceStatus::Completed };
            manager.finish_trace(&trace_id, status, None).await.unwrap();
        }

        let stats = manager.get_trace_statistics().await.unwrap();

        assert_eq!(stats.total_traces, 5);
        assert_eq!(stats.completed_traces, 5);
        assert_eq!(stats.active_traces, 0);
        assert_eq!(stats.error_traces, 1);
        assert_eq!(stats.error_rate, 0.2); // 1/5 = 0.2
        assert_eq!(stats.service_counts.len(), 2);
    }

    #[test]
    fn test_sampling() {
        let mut sampler = TracingSampler::new(0.5); // 50%采样率

        let mut sampled_count = 0;
        let total_samples = 1000;

        for _ in 0..total_samples {
            if sampler.should_sample() {
                sampled_count += 1;
            }
        }

        // 允许一定的误差范围
        let expected = total_samples as f64 * 0.5;
        let tolerance = total_samples as f64 * 0.1; // 10%容差

        assert!((sampled_count as f64 - expected).abs() < tolerance);
    }

    #[test]
    fn test_trace_exporters() {
        let console_exporter = ConsoleExporter;
        assert_eq!(console_exporter.name(), "console");

        let jaeger_exporter = JaegerExporter::new(
            "http://localhost:14268".to_string(),
            "test_service".to_string(),
        );
        assert_eq!(jaeger_exporter.name(), "jaeger");

        let otlp_exporter = OpenTelemetryExporter::new(
            "http://localhost:4317".to_string(),
            HashMap::new(),
        );
        assert_eq!(otlp_exporter.name(), "opentelemetry");
    }
}
