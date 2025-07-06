//! 高级监控面板模块
//! 
//! 提供实时系统监控、告警管理和可视化面板功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

/// 高级监控管理器
#[derive(Debug)]
pub struct AdvancedMonitoringManager {
    /// 监控配置
    config: MonitoringConfig,
    /// 实时指标
    metrics: Arc<RwLock<SystemMetrics>>,
    /// 告警规则
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    /// 活跃告警
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    /// 监控面板
    dashboards: Arc<RwLock<HashMap<String, Dashboard>>>,
    /// 指标历史
    metrics_history: Arc<RwLock<Vec<MetricsSnapshot>>>,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 指标收集间隔（秒）
    pub collection_interval_secs: u64,
    /// 历史数据保留时间（小时）
    pub retention_hours: u64,
    /// 告警检查间隔（秒）
    pub alert_check_interval_secs: u64,
    /// 最大告警数量
    pub max_alerts: usize,
    /// 启用详细日志
    pub enable_detailed_logging: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: 30,
            retention_hours: 24,
            alert_check_interval_secs: 60,
            max_alerts: 100,
            enable_detailed_logging: true,
        }
    }
}

/// 系统指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// CPU使用率（百分比）
    pub cpu_usage_percent: f64,
    /// 内存使用量（MB）
    pub memory_usage_mb: f64,
    /// 内存使用率（百分比）
    pub memory_usage_percent: f64,
    /// 网络入流量（MB/s）
    pub network_in_mbps: f64,
    /// 网络出流量（MB/s）
    pub network_out_mbps: f64,
    /// 磁盘使用率（百分比）
    pub disk_usage_percent: f64,
    /// 磁盘IO读取（MB/s）
    pub disk_read_mbps: f64,
    /// 磁盘IO写入（MB/s）
    pub disk_write_mbps: f64,
    /// 活跃连接数
    pub active_connections: u64,
    /// 消息处理速率（msg/s）
    pub message_rate: f64,
    /// 错误率（百分比）
    pub error_rate: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            memory_usage_percent: 0.0,
            network_in_mbps: 0.0,
            network_out_mbps: 0.0,
            disk_usage_percent: 0.0,
            disk_read_mbps: 0.0,
            disk_write_mbps: 0.0,
            active_connections: 0,
            message_rate: 0.0,
            error_rate: 0.0,
            avg_response_time_ms: 0.0,
        }
    }
}

/// 指标快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 系统指标
    pub metrics: SystemMetrics,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 监控指标
    pub metric: String,
    /// 比较操作符
    pub operator: ComparisonOperator,
    /// 阈值
    pub threshold: f64,
    /// 持续时间（秒）
    pub duration_secs: u64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
    /// 描述
    pub description: String,
}

/// 比较操作符
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// 告警严重级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// 告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 告警ID
    pub id: String,
    /// 规则ID
    pub rule_id: String,
    /// 告警名称
    pub name: String,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 触发时间
    pub triggered_at: DateTime<Utc>,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 状态
    pub status: AlertStatus,
    /// 描述
    pub description: String,
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

/// 监控面板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// 面板ID
    pub id: String,
    /// 面板名称
    pub name: String,
    /// 小部件列表
    pub widgets: Vec<Widget>,
    /// 刷新间隔（秒）
    pub refresh_interval_secs: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 小部件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// 小部件ID
    pub id: String,
    /// 小部件类型
    pub widget_type: WidgetType,
    /// 标题
    pub title: String,
    /// 监控指标
    pub metric: String,
    /// 位置和大小
    pub layout: WidgetLayout,
    /// 配置选项
    pub options: WidgetOptions,
}

/// 小部件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    Counter,
    Table,
    Alert,
}

/// 小部件布局
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetLayout {
    /// X坐标
    pub x: u32,
    /// Y坐标
    pub y: u32,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

/// 小部件选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetOptions {
    /// 时间范围（小时）
    pub time_range_hours: u64,
    /// 最小值
    pub min_value: Option<f64>,
    /// 最大值
    pub max_value: Option<f64>,
    /// 单位
    pub unit: Option<String>,
    /// 颜色主题
    pub color_theme: Option<String>,
}

impl AdvancedMonitoringManager {
    /// 创建新的高级监控管理器
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 启动监控服务
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("启动高级监控服务");

        // 启动指标收集任务
        let metrics_clone = self.metrics.clone();
        let history_clone = self.metrics_history.clone();
        let collection_interval = self.config.collection_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(collection_interval));
            loop {
                interval.tick().await;
                
                // 收集系统指标
                let new_metrics = Self::collect_system_metrics().await;
                
                // 更新当前指标
                {
                    let mut metrics = metrics_clone.write().await;
                    *metrics = new_metrics.clone();
                }
                
                // 添加到历史记录
                {
                    let mut history = history_clone.write().await;
                    history.push(MetricsSnapshot {
                        timestamp: Utc::now(),
                        metrics: new_metrics,
                    });
                    
                    // 清理过期数据
                    let cutoff = Utc::now() - chrono::Duration::hours(24);
                    history.retain(|snapshot| snapshot.timestamp > cutoff);
                }
            }
        });

        // 启动告警检查任务
        let alert_rules_clone = self.alert_rules.clone();
        let active_alerts_clone = self.active_alerts.clone();
        let metrics_clone = self.metrics.clone();
        let alert_interval = self.config.alert_check_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(alert_interval));
            loop {
                interval.tick().await;
                
                let current_metrics = metrics_clone.read().await.clone();
                let rules = alert_rules_clone.read().await.clone();
                
                for rule in rules {
                    if !rule.enabled {
                        continue;
                    }
                    
                    if Self::check_alert_condition(&rule, &current_metrics) {
                        Self::trigger_alert(&rule, &current_metrics, &active_alerts_clone).await;
                    }
                }
            }
        });

        Ok(())
    }

    /// 收集系统指标
    async fn collect_system_metrics() -> SystemMetrics {
        // 简化实现，实际应该使用系统API获取真实数据
        SystemMetrics {
            timestamp: Utc::now(),
            cpu_usage_percent: rand::random::<f64>() * 100.0,
            memory_usage_mb: 1024.0 + rand::random::<f64>() * 512.0,
            memory_usage_percent: 50.0 + rand::random::<f64>() * 30.0,
            network_in_mbps: rand::random::<f64>() * 10.0,
            network_out_mbps: rand::random::<f64>() * 10.0,
            disk_usage_percent: 60.0 + rand::random::<f64>() * 20.0,
            disk_read_mbps: rand::random::<f64>() * 5.0,
            disk_write_mbps: rand::random::<f64>() * 5.0,
            active_connections: (rand::random::<u64>() % 100) + 10,
            message_rate: 100.0 + rand::random::<f64>() * 900.0,
            error_rate: rand::random::<f64>() * 5.0,
            avg_response_time_ms: 1.0 + rand::random::<f64>() * 10.0,
        }
    }

    /// 检查告警条件
    fn check_alert_condition(rule: &AlertRule, metrics: &SystemMetrics) -> bool {
        let current_value = match rule.metric.as_str() {
            "cpu_usage_percent" => metrics.cpu_usage_percent,
            "memory_usage_percent" => metrics.memory_usage_percent,
            "error_rate" => metrics.error_rate,
            "avg_response_time_ms" => metrics.avg_response_time_ms,
            _ => return false,
        };

        match rule.operator {
            ComparisonOperator::GreaterThan => current_value > rule.threshold,
            ComparisonOperator::LessThan => current_value < rule.threshold,
            ComparisonOperator::GreaterThanOrEqual => current_value >= rule.threshold,
            ComparisonOperator::LessThanOrEqual => current_value <= rule.threshold,
            ComparisonOperator::Equal => (current_value - rule.threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (current_value - rule.threshold).abs() >= f64::EPSILON,
        }
    }

    /// 触发告警
    async fn trigger_alert(
        rule: &AlertRule,
        metrics: &SystemMetrics,
        active_alerts: &Arc<RwLock<Vec<Alert>>>,
    ) {
        let current_value = match rule.metric.as_str() {
            "cpu_usage_percent" => metrics.cpu_usage_percent,
            "memory_usage_percent" => metrics.memory_usage_percent,
            "error_rate" => metrics.error_rate,
            "avg_response_time_ms" => metrics.avg_response_time_ms,
            _ => return,
        };

        let alert = Alert {
            id: format!("alert_{}", uuid::Uuid::new_v4()),
            rule_id: rule.id.clone(),
            name: rule.name.clone(),
            severity: rule.severity.clone(),
            triggered_at: Utc::now(),
            current_value,
            threshold: rule.threshold,
            status: AlertStatus::Active,
            description: rule.description.clone(),
        };

        let mut alerts = active_alerts.write().await;
        alerts.push(alert);
        
        warn!("触发告警: {} - 当前值: {:.2}, 阈值: {:.2}", 
              rule.name, current_value, rule.threshold);
    }

    /// 获取当前系统指标
    pub async fn get_current_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }

    /// 获取指标历史
    pub async fn get_metrics_history(&self, hours: u64) -> Vec<MetricsSnapshot> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        let history = self.metrics_history.read().await;
        history.iter()
            .filter(|snapshot| snapshot.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// 添加告警规则
    pub async fn add_alert_rule(&self, rule: AlertRule) {
        self.alert_rules.write().await.push(rule);
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().await.clone()
    }

    /// 创建监控面板
    pub async fn create_dashboard(&self, dashboard: Dashboard) {
        self.dashboards.write().await.insert(dashboard.id.clone(), dashboard);
    }

    /// 获取监控面板
    pub async fn get_dashboard(&self, id: &str) -> Option<Dashboard> {
        self.dashboards.read().await.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_manager_creation() {
        let config = MonitoringConfig::default();
        let manager = AdvancedMonitoringManager::new(config);
        
        let metrics = manager.get_current_metrics().await;
        assert_eq!(metrics.cpu_usage_percent, 0.0);
    }

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "CPU高使用率".to_string(),
            metric: "cpu_usage_percent".to_string(),
            operator: ComparisonOperator::GreaterThan,
            threshold: 80.0,
            duration_secs: 300,
            severity: AlertSeverity::Warning,
            enabled: true,
            description: "CPU使用率超过80%".to_string(),
        };
        
        assert_eq!(rule.threshold, 80.0);
        assert_eq!(rule.severity, AlertSeverity::Warning);
    }

    #[test]
    fn test_comparison_operators() {
        assert_eq!(ComparisonOperator::GreaterThan, ComparisonOperator::GreaterThan);
        assert_ne!(ComparisonOperator::GreaterThan, ComparisonOperator::LessThan);
    }
}
