//! 自动扩缩容管理器
//! 
//! 基于负载和性能指标自动调整集群规模

use crate::config::AutoscalerConfig;
use crate::error::{ClusterError, ClusterResult};
use crate::cluster_state::ClusterState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use chrono::{DateTime, Utc, Duration};

/// 扩缩容动作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingAction {
    /// 扩容
    ScaleUp {
        /// 目标实例数
        target_instances: u32,
        /// 扩容原因
        reason: String,
    },
    /// 缩容
    ScaleDown {
        /// 目标实例数
        target_instances: u32,
        /// 缩容原因
        reason: String,
    },
    /// 无需操作
    NoAction,
}

/// 扩缩容决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingDecision {
    /// 决策时间
    pub timestamp: DateTime<Utc>,
    /// 当前实例数
    pub current_instances: u32,
    /// 建议动作
    pub action: ScalingAction,
    /// 决策依据的指标
    pub metrics: HashMap<String, f64>,
    /// 决策置信度 (0.0-1.0)
    pub confidence: f64,
}

/// 扩缩容历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingHistory {
    /// 记录时间
    pub timestamp: DateTime<Utc>,
    /// 执行的动作
    pub action: ScalingAction,
    /// 执行前实例数
    pub before_instances: u32,
    /// 执行后实例数
    pub after_instances: u32,
    /// 执行结果
    pub success: bool,
    /// 错误信息（如果失败）
    pub error_message: Option<String>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU使用率 (0.0-1.0)
    pub cpu_usage: f64,
    /// 内存使用率 (0.0-1.0)
    pub memory_usage: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 消息队列长度
    pub queue_length: u32,
    /// 错误率 (0.0-1.0)
    pub error_rate: f64,
    /// 吞吐量（消息/秒）
    pub throughput: f64,
    /// 自定义指标
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            avg_response_time: 0.0,
            queue_length: 0,
            error_rate: 0.0,
            throughput: 0.0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// 自动扩缩容管理器
pub struct AutoScaler {
    /// 配置
    config: AutoscalerConfig,
    /// 当前性能指标
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    /// 扩缩容历史
    scaling_history: Arc<RwLock<Vec<ScalingHistory>>>,
    /// 最后扩缩容时间
    last_scaling_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl AutoScaler {
    /// 创建新的自动扩缩容管理器
    pub fn new(config: AutoscalerConfig) -> Self {
        info!("🔧 创建自动扩缩容管理器，策略: {:?}", config.strategy);
        
        Self {
            config,
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            scaling_history: Arc::new(RwLock::new(Vec::new())),
            last_scaling_time: Arc::new(RwLock::new(None)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 启动自动扩缩容
    pub async fn start(&self) -> ClusterResult<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(ClusterError::AlreadyRunning("AutoScaler已在运行".to_string()));
            }
            *running = true;
        }

        info!("🚀 启动自动扩缩容管理器");

        // 启动指标收集任务
        self.start_metrics_collection().await?;

        // 启动扩缩容决策任务
        self.start_scaling_decision().await?;

        Ok(())
    }

    /// 停止自动扩缩容
    pub async fn stop(&self) -> ClusterResult<()> {
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        info!("🛑 停止自动扩缩容管理器");
        Ok(())
    }

    /// 更新性能指标
    pub async fn update_metrics(&self, metrics: PerformanceMetrics) -> ClusterResult<()> {
        {
            let mut current_metrics = self.current_metrics.write().await;
            *current_metrics = metrics;
        }

        debug!("📊 更新性能指标");
        Ok(())
    }

    /// 基于集群状态更新指标
    pub async fn update_metrics_from_cluster_state(&self, _cluster_state: &ClusterState) -> ClusterResult<()> {
        // 注意：ClusterState本身不包含详细的agent统计信息
        // 这里我们使用简化的指标计算，实际应该从ClusterStateManager获取详细信息
        let mut metrics = PerformanceMetrics::default();

        // 基于集群基本信息估算指标
        let agent_count = _cluster_state.agent_count as f64;

        // 模拟CPU和内存使用率（实际应该从系统监控获取）
        metrics.cpu_usage = (agent_count / 10.0).min(1.0);
        metrics.memory_usage = (agent_count / 15.0).min(1.0);

        // 模拟响应时间（基于集群负载）
        metrics.avg_response_time = if agent_count > 5.0 { 200.0 } else { 100.0 };

        // 模拟吞吐量
        metrics.throughput = agent_count * 100.0;

        // 模拟错误率
        metrics.error_rate = if agent_count > 8.0 { 0.05 } else { 0.01 };

        self.update_metrics(metrics).await
    }

    /// 做出扩缩容决策
    pub async fn make_scaling_decision(&self, current_instances: u32) -> ClusterResult<ScalingDecision> {
        let metrics = self.current_metrics.read().await.clone();
        let mut decision_metrics = HashMap::new();

        // 收集决策依据的指标
        decision_metrics.insert("cpu_usage".to_string(), metrics.cpu_usage);
        decision_metrics.insert("memory_usage".to_string(), metrics.memory_usage);
        decision_metrics.insert("avg_response_time".to_string(), metrics.avg_response_time);
        decision_metrics.insert("error_rate".to_string(), metrics.error_rate);
        decision_metrics.insert("throughput".to_string(), metrics.throughput);

        let action = match self.config.strategy {
            crate::config::ScalingStrategy::CpuBased => self.decide_cpu_based(&metrics, current_instances).await,
            crate::config::ScalingStrategy::MemoryBased => self.decide_memory_based(&metrics, current_instances).await,
            crate::config::ScalingStrategy::ResponseTimeBased => self.decide_response_time_based(&metrics, current_instances).await,
            crate::config::ScalingStrategy::QueueBased => self.decide_queue_based(&metrics, current_instances).await,
            crate::config::ScalingStrategy::Hybrid => self.decide_hybrid(&metrics, current_instances).await,
            crate::config::ScalingStrategy::CustomMetrics => self.decide_custom_metrics(&metrics, current_instances).await,
        };

        // 计算置信度
        let confidence = self.calculate_confidence(&metrics, &action).await;

        Ok(ScalingDecision {
            timestamp: Utc::now(),
            current_instances,
            action,
            metrics: decision_metrics,
            confidence,
        })
    }

    /// 执行扩缩容动作
    pub async fn execute_scaling_action(&self, decision: &ScalingDecision) -> ClusterResult<bool> {
        // 检查冷却时间
        if !self.check_cooldown().await? {
            debug!("⏰ 扩缩容冷却时间未到，跳过执行");
            return Ok(false);
        }

        // 检查置信度
        if decision.confidence < self.config.min_confidence {
            debug!("🤔 扩缩容决策置信度不足: {:.2}", decision.confidence);
            return Ok(false);
        }

        let before_instances = decision.current_instances;
        let mut success = false;
        let mut error_message = None;

        match &decision.action {
            ScalingAction::ScaleUp { target_instances, reason } => {
                info!("📈 执行扩容: {} -> {}, 原因: {}", before_instances, target_instances, reason);
                match self.scale_up(*target_instances - before_instances).await {
                    Ok(_) => success = true,
                    Err(e) => error_message = Some(e.to_string()),
                }
            }
            ScalingAction::ScaleDown { target_instances, reason } => {
                info!("📉 执行缩容: {} -> {}, 原因: {}", before_instances, target_instances, reason);
                match self.scale_down(before_instances - *target_instances).await {
                    Ok(_) => success = true,
                    Err(e) => error_message = Some(e.to_string()),
                }
            }
            ScalingAction::NoAction => {
                debug!("⚖️ 无需扩缩容操作");
                return Ok(false);
            }
        }

        // 记录扩缩容历史
        let history = ScalingHistory {
            timestamp: Utc::now(),
            action: decision.action.clone(),
            before_instances,
            after_instances: match &decision.action {
                ScalingAction::ScaleUp { target_instances, .. } => *target_instances,
                ScalingAction::ScaleDown { target_instances, .. } => *target_instances,
                ScalingAction::NoAction => before_instances,
            },
            success,
            error_message,
        };

        {
            let mut scaling_history = self.scaling_history.write().await;
            scaling_history.push(history);

            // 限制历史记录数量
            if scaling_history.len() > self.config.max_history_entries {
                scaling_history.remove(0);
            }
        }

        // 更新最后扩缩容时间
        if success {
            let mut last_scaling_time = self.last_scaling_time.write().await;
            *last_scaling_time = Some(Utc::now());
        }

        Ok(success)
    }

    /// 获取扩缩容历史
    pub async fn get_scaling_history(&self) -> Vec<ScalingHistory> {
        self.scaling_history.read().await.clone()
    }

    /// 获取当前性能指标
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.read().await.clone()
    }

    // 私有方法

    async fn start_metrics_collection(&self) -> ClusterResult<()> {
        let _current_metrics = self.current_metrics.clone();
        let running = self.running.clone();
        let collection_interval = self.config.metrics_collection_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);

            loop {
                interval.tick().await;

                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }

                // TODO: 实际的指标收集逻辑
                // 这里应该从系统监控、Prometheus等获取真实指标
                debug!("📊 收集性能指标");
            }
        });

        Ok(())
    }

    async fn start_scaling_decision(&self) -> ClusterResult<()> {
        let running = self.running.clone();
        let decision_interval = self.config.decision_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(decision_interval);

            loop {
                interval.tick().await;

                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }

                // TODO: 实际的扩缩容决策逻辑
                // 这里应该调用make_scaling_decision和execute_scaling_action
                debug!("🤖 执行扩缩容决策");
            }
        });

        Ok(())
    }

    async fn decide_cpu_based(&self, metrics: &PerformanceMetrics, current_instances: u32) -> ScalingAction {
        if metrics.cpu_usage > self.config.scale_up_threshold {
            let target = (current_instances + self.config.scale_up_step).min(self.config.max_instances);
            ScalingAction::ScaleUp {
                target_instances: target,
                reason: format!("CPU使用率过高: {:.1}%", metrics.cpu_usage * 100.0),
            }
        } else if metrics.cpu_usage < self.config.scale_down_threshold && current_instances > self.config.min_instances {
            let target = (current_instances.saturating_sub(self.config.scale_down_step)).max(self.config.min_instances);
            ScalingAction::ScaleDown {
                target_instances: target,
                reason: format!("CPU使用率过低: {:.1}%", metrics.cpu_usage * 100.0),
            }
        } else {
            ScalingAction::NoAction
        }
    }

    async fn decide_memory_based(&self, metrics: &PerformanceMetrics, current_instances: u32) -> ScalingAction {
        if metrics.memory_usage > self.config.scale_up_threshold {
            let target = (current_instances + self.config.scale_up_step).min(self.config.max_instances);
            ScalingAction::ScaleUp {
                target_instances: target,
                reason: format!("内存使用率过高: {:.1}%", metrics.memory_usage * 100.0),
            }
        } else if metrics.memory_usage < self.config.scale_down_threshold && current_instances > self.config.min_instances {
            let target = (current_instances.saturating_sub(self.config.scale_down_step)).max(self.config.min_instances);
            ScalingAction::ScaleDown {
                target_instances: target,
                reason: format!("内存使用率过低: {:.1}%", metrics.memory_usage * 100.0),
            }
        } else {
            ScalingAction::NoAction
        }
    }

    async fn decide_response_time_based(&self, metrics: &PerformanceMetrics, current_instances: u32) -> ScalingAction {
        let threshold_ms = self.config.scale_up_threshold * 1000.0; // 转换为毫秒
        
        if metrics.avg_response_time > threshold_ms {
            let target = (current_instances + self.config.scale_up_step).min(self.config.max_instances);
            ScalingAction::ScaleUp {
                target_instances: target,
                reason: format!("响应时间过长: {:.1}ms", metrics.avg_response_time),
            }
        } else if metrics.avg_response_time < threshold_ms * self.config.scale_down_threshold && current_instances > self.config.min_instances {
            let target = (current_instances.saturating_sub(self.config.scale_down_step)).max(self.config.min_instances);
            ScalingAction::ScaleDown {
                target_instances: target,
                reason: format!("响应时间良好: {:.1}ms", metrics.avg_response_time),
            }
        } else {
            ScalingAction::NoAction
        }
    }

    async fn decide_queue_based(&self, metrics: &PerformanceMetrics, current_instances: u32) -> ScalingAction {
        let threshold = (self.config.scale_up_threshold * 100.0) as u32; // 队列长度阈值
        
        if metrics.queue_length > threshold {
            let target = (current_instances + self.config.scale_up_step).min(self.config.max_instances);
            ScalingAction::ScaleUp {
                target_instances: target,
                reason: format!("消息队列过长: {}", metrics.queue_length),
            }
        } else if metrics.queue_length < (threshold as f64 * self.config.scale_down_threshold) as u32 && current_instances > self.config.min_instances {
            let target = (current_instances.saturating_sub(self.config.scale_down_step)).max(self.config.min_instances);
            ScalingAction::ScaleDown {
                target_instances: target,
                reason: format!("消息队列较短: {}", metrics.queue_length),
            }
        } else {
            ScalingAction::NoAction
        }
    }

    async fn decide_hybrid(&self, metrics: &PerformanceMetrics, current_instances: u32) -> ScalingAction {
        // 混合策略：综合考虑多个指标
        let mut scale_up_score = 0.0;
        let mut scale_down_score = 0.0;

        // CPU指标权重
        if metrics.cpu_usage > self.config.scale_up_threshold {
            scale_up_score += 0.3;
        } else if metrics.cpu_usage < self.config.scale_down_threshold {
            scale_down_score += 0.3;
        }

        // 内存指标权重
        if metrics.memory_usage > self.config.scale_up_threshold {
            scale_up_score += 0.2;
        } else if metrics.memory_usage < self.config.scale_down_threshold {
            scale_down_score += 0.2;
        }

        // 响应时间指标权重
        let response_threshold = self.config.scale_up_threshold * 1000.0;
        if metrics.avg_response_time > response_threshold {
            scale_up_score += 0.3;
        } else if metrics.avg_response_time < response_threshold * self.config.scale_down_threshold {
            scale_down_score += 0.3;
        }

        // 错误率指标权重
        if metrics.error_rate > 0.05 { // 5%错误率阈值
            scale_up_score += 0.2;
        } else if metrics.error_rate < 0.01 { // 1%错误率阈值
            scale_down_score += 0.2;
        }

        if scale_up_score > 0.5 && current_instances < self.config.max_instances {
            let target = (current_instances + self.config.scale_up_step).min(self.config.max_instances);
            ScalingAction::ScaleUp {
                target_instances: target,
                reason: format!("混合指标建议扩容 (评分: {:.2})", scale_up_score),
            }
        } else if scale_down_score > 0.5 && current_instances > self.config.min_instances {
            let target = (current_instances.saturating_sub(self.config.scale_down_step)).max(self.config.min_instances);
            ScalingAction::ScaleDown {
                target_instances: target,
                reason: format!("混合指标建议缩容 (评分: {:.2})", scale_down_score),
            }
        } else {
            ScalingAction::NoAction
        }
    }

    async fn decide_custom_metrics(&self, _metrics: &PerformanceMetrics, _current_instances: u32) -> ScalingAction {
        // 基于自定义指标的决策
        // TODO: 实现自定义指标逻辑
        ScalingAction::NoAction
    }

    async fn calculate_confidence(&self, metrics: &PerformanceMetrics, action: &ScalingAction) -> f64 {
        // 基于指标的稳定性和历史成功率计算置信度
        let mut confidence: f64 = 0.5; // 基础置信度

        // 根据指标的极端程度调整置信度
        match action {
            ScalingAction::ScaleUp { .. } => {
                if metrics.cpu_usage > 0.8 || metrics.memory_usage > 0.8 || metrics.avg_response_time > 2000.0 {
                    confidence += 0.3;
                }
                if metrics.error_rate > 0.1 {
                    confidence += 0.2;
                }
            }
            ScalingAction::ScaleDown { .. } => {
                if metrics.cpu_usage < 0.2 && metrics.memory_usage < 0.2 && metrics.avg_response_time < 100.0 {
                    confidence += 0.3;
                }
                if metrics.error_rate < 0.01 {
                    confidence += 0.2;
                }
            }
            ScalingAction::NoAction => {
                confidence = 1.0; // 不操作总是安全的
            }
        }

        confidence.min(1.0)
    }

    async fn check_cooldown(&self) -> ClusterResult<bool> {
        let last_scaling_time = self.last_scaling_time.read().await;
        
        if let Some(last_time) = *last_scaling_time {
            let elapsed = Utc::now() - last_time;
            let cooldown_duration = Duration::seconds(self.config.cooldown_period.as_secs() as i64);
            Ok(elapsed >= cooldown_duration)
        } else {
            Ok(true) // 首次扩缩容
        }
    }

    async fn scale_up(&self, instances: u32) -> ClusterResult<()> {
        // TODO: 实际的扩容逻辑
        // 这里应该调用容器编排系统（如Kubernetes）或云服务API
        info!("🚀 扩容 {} 个实例", instances);
        Ok(())
    }

    async fn scale_down(&self, instances: u32) -> ClusterResult<()> {
        // TODO: 实际的缩容逻辑
        // 这里应该优雅地停止实例并从负载均衡器中移除
        info!("🔽 缩容 {} 个实例", instances);
        Ok(())
    }
}
