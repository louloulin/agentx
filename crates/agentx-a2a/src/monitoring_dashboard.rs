//! 性能监控面板
//! 
//! 提供实时性能监控、指标可视化和告警功能

use crate::monitoring::{MonitoringManager, HealthCheck};
use crate::A2AResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use tracing::{debug, info};

/// 监控面板
pub struct MonitoringDashboard {
    /// 监控管理器
    monitoring_manager: MonitoringManager,
    /// 面板配置
    config: DashboardConfig,
    /// 告警规则
    alert_rules: Vec<AlertRule>,
    /// 活跃告警
    active_alerts: Vec<Alert>,
    /// 面板小部件
    widgets: HashMap<String, Widget>,
}

/// 面板配置
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// 刷新间隔（秒）
    pub refresh_interval_seconds: u64,
    /// 数据保留时间（小时）
    pub data_retention_hours: u64,
    /// 是否启用告警
    pub enable_alerts: bool,
    /// 告警检查间隔（秒）
    pub alert_check_interval_seconds: u64,
    /// 最大告警数量
    pub max_alerts: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 30,
            data_retention_hours: 24,
            enable_alerts: true,
            alert_check_interval_seconds: 60,
            max_alerts: 100,
        }
    }
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub rule_id: String,
    /// 规则名称
    pub name: String,
    /// 指标名称
    pub metric_name: String,
    /// 条件
    pub condition: AlertCondition,
    /// 阈值
    pub threshold: f64,
    /// 持续时间（秒）
    pub duration_seconds: u64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
    /// 描述
    pub description: String,
}

/// 告警条件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCondition {
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 大于等于
    GreaterThanOrEqual,
    /// 小于等于
    LessThanOrEqual,
}

/// 告警严重级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// 告警ID
    pub alert_id: String,
    /// 规则ID
    pub rule_id: String,
    /// 告警名称
    pub name: String,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 状态
    pub status: AlertStatus,
    /// 触发时间
    pub triggered_at: DateTime<Utc>,
    /// 解决时间
    pub resolved_at: Option<DateTime<Utc>>,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 描述
    pub description: String,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 告警状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    /// 触发
    Triggered,
    /// 已解决
    Resolved,
    /// 已确认
    Acknowledged,
}

/// 面板小部件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// 小部件ID
    pub widget_id: String,
    /// 小部件类型
    pub widget_type: WidgetType,
    /// 标题
    pub title: String,
    /// 数据源
    pub data_source: DataSource,
    /// 配置
    pub config: WidgetConfig,
    /// 位置
    pub position: WidgetPosition,
}

/// 小部件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    /// 折线图
    LineChart,
    /// 柱状图
    BarChart,
    /// 饼图
    PieChart,
    /// 仪表盘
    Gauge,
    /// 数值显示
    Number,
    /// 表格
    Table,
    /// 状态指示器
    StatusIndicator,
}

/// 数据源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    /// 指标名称
    pub metric_name: String,
    /// 时间范围（分钟）
    pub time_range_minutes: u64,
    /// 聚合方式
    pub aggregation: AggregationType,
    /// 过滤器
    pub filters: HashMap<String, String>,
}

/// 聚合类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    /// 平均值
    Average,
    /// 总和
    Sum,
    /// 最大值
    Max,
    /// 最小值
    Min,
    /// 计数
    Count,
    /// 百分位数
    Percentile(f64),
}

/// 小部件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    /// 颜色
    pub color: Option<String>,
    /// 单位
    pub unit: Option<String>,
    /// 小数位数
    pub decimal_places: Option<u32>,
    /// 最小值
    pub min_value: Option<f64>,
    /// 最大值
    pub max_value: Option<f64>,
    /// 阈值
    pub thresholds: Vec<Threshold>,
}

/// 阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    /// 值
    pub value: f64,
    /// 颜色
    pub color: String,
    /// 标签
    pub label: String,
}

/// 小部件位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    /// X坐标
    pub x: u32,
    /// Y坐标
    pub y: u32,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
}

/// 面板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// 系统概览
    pub system_overview: SystemOverview,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
    /// 健康状态
    pub health_status: HealthCheck,
    /// 活跃告警
    pub active_alerts: Vec<Alert>,
    /// 小部件数据
    pub widget_data: HashMap<String, WidgetData>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 系统概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    /// 注册的Agent数量
    pub registered_agents: u32,
    /// 活跃的Agent数量
    pub active_agents: u32,
    /// 总消息数
    pub total_messages: u64,
    /// 成功消息数
    pub successful_messages: u64,
    /// 失败消息数
    pub failed_messages: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 系统运行时间（秒）
    pub uptime_seconds: u64,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU使用率
    pub cpu_usage: f64,
    /// 内存使用率
    pub memory_usage: f64,
    /// 网络吞吐量（字节/秒）
    pub network_throughput: f64,
    /// 磁盘使用率
    pub disk_usage: f64,
    /// 请求速率（请求/秒）
    pub request_rate: f64,
    /// 错误率
    pub error_rate: f64,
}

/// 小部件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    /// 数据点
    pub data_points: Vec<DataPoint>,
    /// 当前值
    pub current_value: Option<f64>,
    /// 状态
    pub status: Option<String>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 值
    pub value: f64,
    /// 标签
    pub labels: HashMap<String, String>,
}

impl MonitoringDashboard {
    /// 创建新的监控面板
    pub fn new(monitoring_manager: MonitoringManager, config: DashboardConfig) -> Self {
        info!("📊 创建监控面板");
        
        Self {
            monitoring_manager,
            config,
            alert_rules: Vec::new(),
            active_alerts: Vec::new(),
            widgets: HashMap::new(),
        }
    }

    /// 添加告警规则
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        info!("添加告警规则: {}", rule.name);
        self.alert_rules.push(rule);
    }

    /// 添加小部件
    pub fn add_widget(&mut self, widget: Widget) {
        debug!("添加小部件: {}", widget.title);
        self.widgets.insert(widget.widget_id.clone(), widget);
    }

    /// 获取面板数据
    pub fn get_dashboard_data(&self) -> A2AResult<DashboardData> {
        let system_overview = self.collect_system_overview()?;
        let performance_metrics = self.collect_performance_metrics()?;
        let health_status = self.monitoring_manager.get_health_status().clone();
        let widget_data = self.collect_widget_data()?;

        Ok(DashboardData {
            system_overview,
            performance_metrics,
            health_status,
            active_alerts: self.active_alerts.clone(),
            widget_data,
            timestamp: Utc::now(),
        })
    }

    /// 检查告警
    pub fn check_alerts(&mut self) -> A2AResult<Vec<Alert>> {
        if !self.config.enable_alerts {
            return Ok(Vec::new());
        }

        let mut new_alerts = Vec::new();

        for rule in &self.alert_rules {
            if !rule.enabled {
                continue;
            }

            if let Ok(triggered) = self.evaluate_alert_rule(rule) {
                if triggered {
                    let alert = self.create_alert(rule)?;
                    new_alerts.push(alert);
                }
            }
        }

        // 添加新告警
        for alert in &new_alerts {
            self.active_alerts.push(alert.clone());
        }

        // 清理过期告警
        self.cleanup_resolved_alerts();

        Ok(new_alerts)
    }

    /// 解决告警
    pub fn resolve_alert(&mut self, alert_id: &str) -> A2AResult<()> {
        if let Some(alert) = self.active_alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(Utc::now());
            info!("告警已解决: {}", alert_id);
        }
        Ok(())
    }

    /// 确认告警
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> A2AResult<()> {
        if let Some(alert) = self.active_alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.status = AlertStatus::Acknowledged;
            info!("告警已确认: {}", alert_id);
        }
        Ok(())
    }

    /// 获取活跃告警
    pub fn get_active_alerts(&self) -> &[Alert] {
        &self.active_alerts
    }

    /// 获取小部件列表
    pub fn get_widgets(&self) -> &HashMap<String, Widget> {
        &self.widgets
    }

    // 私有方法

    fn collect_system_overview(&self) -> A2AResult<SystemOverview> {
        // 模拟系统概览数据收集
        Ok(SystemOverview {
            registered_agents: 10,
            active_agents: 8,
            total_messages: 15000,
            successful_messages: 14850,
            failed_messages: 150,
            avg_response_time: 125.5,
            uptime_seconds: 86400, // 1天
        })
    }

    fn collect_performance_metrics(&self) -> A2AResult<PerformanceMetrics> {
        // 模拟性能指标收集
        Ok(PerformanceMetrics {
            cpu_usage: 0.65,
            memory_usage: 0.72,
            network_throughput: 1024.0 * 1024.0 * 10.0, // 10 MB/s
            disk_usage: 0.45,
            request_rate: 150.0,
            error_rate: 0.01,
        })
    }

    fn collect_widget_data(&self) -> A2AResult<HashMap<String, WidgetData>> {
        let mut widget_data = HashMap::new();

        for (widget_id, widget) in &self.widgets {
            let data = self.collect_widget_specific_data(widget)?;
            widget_data.insert(widget_id.clone(), data);
        }

        Ok(widget_data)
    }

    fn collect_widget_specific_data(&self, _widget: &Widget) -> A2AResult<WidgetData> {
        // 模拟小部件数据收集
        let now = Utc::now();
        let mut data_points = Vec::new();

        // 生成模拟数据点
        for i in 0..10 {
            let timestamp = now - Duration::minutes(i * 5);
            let value = 50.0 + (i as f64 * 5.0);
            
            data_points.push(DataPoint {
                timestamp,
                value,
                labels: HashMap::new(),
            });
        }

        Ok(WidgetData {
            data_points,
            current_value: Some(75.0),
            status: Some("正常".to_string()),
            updated_at: now,
        })
    }

    fn evaluate_alert_rule(&self, rule: &AlertRule) -> A2AResult<bool> {
        // 简化的告警规则评估
        // 实际应该从监控管理器获取指标数据
        let current_value = 80.0; // 模拟当前值

        let triggered = match rule.condition {
            AlertCondition::GreaterThan => current_value > rule.threshold,
            AlertCondition::LessThan => current_value < rule.threshold,
            AlertCondition::Equal => (current_value - rule.threshold).abs() < f64::EPSILON,
            AlertCondition::NotEqual => (current_value - rule.threshold).abs() > f64::EPSILON,
            AlertCondition::GreaterThanOrEqual => current_value >= rule.threshold,
            AlertCondition::LessThanOrEqual => current_value <= rule.threshold,
        };

        Ok(triggered)
    }

    fn create_alert(&self, rule: &AlertRule) -> A2AResult<Alert> {
        let alert_id = uuid::Uuid::new_v4().to_string();
        
        Ok(Alert {
            alert_id,
            rule_id: rule.rule_id.clone(),
            name: rule.name.clone(),
            severity: rule.severity.clone(),
            status: AlertStatus::Triggered,
            triggered_at: Utc::now(),
            resolved_at: None,
            current_value: 80.0, // 模拟当前值
            threshold: rule.threshold,
            description: rule.description.clone(),
            labels: HashMap::new(),
        })
    }

    fn cleanup_resolved_alerts(&mut self) {
        let cutoff_time = Utc::now() - Duration::hours(24);
        
        self.active_alerts.retain(|alert| {
            if alert.status == AlertStatus::Resolved {
                if let Some(resolved_at) = alert.resolved_at {
                    resolved_at > cutoff_time
                } else {
                    true
                }
            } else {
                true
            }
        });

        // 限制告警数量
        if self.active_alerts.len() > self.config.max_alerts {
            self.active_alerts.sort_by(|a, b| b.triggered_at.cmp(&a.triggered_at));
            self.active_alerts.truncate(self.config.max_alerts);
        }
    }
}
