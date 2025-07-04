//! Metrics Collector Actor
//! 
//! This actor collects, aggregates, and reports system metrics
//! for monitoring and observability using the Actix actor model.

use actix::prelude::*;
use crate::{A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Metrics Collector Actor
pub struct MetricsCollectorActor {
    /// System metrics
    system_metrics: SystemMetrics,
    
    /// Performance metrics
    performance_metrics: PerformanceMetrics,
    
    /// Custom metrics
    custom_metrics: HashMap<String, MetricValue>,
    
    /// Metrics configuration
    config: MetricsConfig,
}

/// System metrics
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_actors: usize,
    pub total_messages_processed: u64,
    pub messages_per_second: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub throughput_ops_per_second: f64,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(f64),
}

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub collection_interval_ms: u64,
    pub retention_period_hours: u64,
    pub enable_system_metrics: bool,
    pub enable_performance_metrics: bool,
    pub max_custom_metrics: usize,
}

/// Message to record a metric
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct RecordMetric {
    pub name: String,
    pub value: MetricValue,
    pub tags: HashMap<String, String>,
}

/// Message to get system metrics
#[derive(Message, Debug)]
#[rtype(result = "SystemMetrics")]
pub struct GetSystemMetrics;

/// Message to get performance metrics
#[derive(Message, Debug)]
#[rtype(result = "PerformanceMetrics")]
pub struct GetPerformanceMetrics;

/// Message to get custom metrics
#[derive(Message, Debug)]
#[rtype(result = "HashMap<String, MetricValue>")]
pub struct GetCustomMetrics {
    pub filter: Option<String>,
}

/// Message to get all metrics
#[derive(Message, Debug)]
#[rtype(result = "AllMetrics")]
pub struct GetAllMetrics;

/// All metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMetrics {
    pub system: SystemMetrics,
    pub performance: PerformanceMetrics,
    pub custom: HashMap<String, MetricValue>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message to reset metrics
#[derive(Message, Debug)]
#[rtype(result = "A2AResult<()>")]
pub struct ResetMetrics {
    pub metric_type: Option<MetricType>,
}

/// Metric type for reset operation
#[derive(Debug, Clone)]
pub enum MetricType {
    System,
    Performance,
    Custom,
    All,
}

/// Message for periodic metrics collection
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct CollectMetrics;

impl Actor for MetricsCollectorActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Metrics Collector Actor started");
        
        // Start periodic metrics collection
        self.start_metrics_collection(ctx);
        
        // Record start time
        self.system_metrics.uptime_seconds = 0;
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Metrics Collector Actor stopped");
    }
}

impl MetricsCollectorActor {
    /// Create a new Metrics Collector Actor
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            system_metrics: SystemMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            custom_metrics: HashMap::new(),
            config,
        }
    }
    
    /// Start periodic metrics collection
    fn start_metrics_collection(&self, ctx: &mut Context<Self>) {
        let interval = std::time::Duration::from_millis(self.config.collection_interval_ms);
        
        ctx.run_interval(interval, |_actor, ctx| {
            ctx.address().do_send(CollectMetrics);
        });
    }
    
    /// Collect system metrics
    fn collect_system_metrics(&mut self) {
        if !self.config.enable_system_metrics {
            return;
        }
        
        // Update uptime
        self.system_metrics.uptime_seconds += self.config.collection_interval_ms / 1000;
        
        // Collect memory usage (simplified)
        self.system_metrics.memory_usage_mb = self.get_memory_usage();
        
        // Collect CPU usage (simplified)
        self.system_metrics.cpu_usage_percent = self.get_cpu_usage();
        
        // Update active actors count (would be provided by actor system)
        self.system_metrics.active_actors = self.get_active_actors_count();
        
        debug!("System metrics updated: {:?}", self.system_metrics);
    }
    
    /// Collect performance metrics
    fn collect_performance_metrics(&mut self) {
        if !self.config.enable_performance_metrics {
            return;
        }
        
        // These would typically be calculated from collected data points
        // For now, we'll use placeholder values
        
        debug!("Performance metrics updated: {:?}", self.performance_metrics);
    }
    
    /// Get memory usage (placeholder implementation)
    fn get_memory_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        // to get actual memory usage
        100.0 // MB
    }
    
    /// Get CPU usage (placeholder implementation)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        // to get actual CPU usage
        25.0 // Percent
    }
    
    /// Get active actors count (placeholder implementation)
    fn get_active_actors_count(&self) -> usize {
        // In a real implementation, this would query the actor system
        10
    }
    
    /// Record a custom metric
    fn record_custom_metric(&mut self, name: String, value: MetricValue) -> A2AResult<()> {
        if self.custom_metrics.len() >= self.config.max_custom_metrics {
            return Err(A2AError::internal("Maximum custom metrics limit reached"));
        }
        
        self.custom_metrics.insert(name, value);
        Ok(())
    }
    
    /// Update performance metrics with new data point
    fn update_performance_metrics(&mut self, response_time_ms: f64, success: bool) {
        // Update response time metrics (simplified moving average)
        let current_avg = self.performance_metrics.average_response_time_ms;
        let total_messages = self.system_metrics.total_messages_processed;
        
        if total_messages > 0 {
            self.performance_metrics.average_response_time_ms = 
                (current_avg * (total_messages - 1) as f64 + response_time_ms) / total_messages as f64;
        } else {
            self.performance_metrics.average_response_time_ms = response_time_ms;
        }
        
        // Update percentiles (simplified)
        if response_time_ms > self.performance_metrics.p95_response_time_ms {
            self.performance_metrics.p95_response_time_ms = response_time_ms;
        }
        
        if response_time_ms > self.performance_metrics.p99_response_time_ms {
            self.performance_metrics.p99_response_time_ms = response_time_ms;
        }
        
        // Update error rate
        if !success {
            let total_errors = (self.performance_metrics.error_rate_percent / 100.0 * total_messages as f64) + 1.0;
            self.performance_metrics.error_rate_percent = (total_errors / total_messages as f64) * 100.0;
        }
    }
}

/// Handle RecordMetric
impl Handler<RecordMetric> for MetricsCollectorActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: RecordMetric, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Recording metric: {} = {:?}", msg.name, msg.value);
        
        // Handle special metrics that update system/performance metrics
        match msg.name.as_str() {
            "message_processed" => {
                self.system_metrics.total_messages_processed += 1;
                if let MetricValue::Timer(response_time) = msg.value {
                    self.update_performance_metrics(response_time, true);
                }
            }
            "message_failed" => {
                if let MetricValue::Timer(response_time) = msg.value {
                    self.update_performance_metrics(response_time, false);
                }
            }
            _ => {
                // Record as custom metric
                self.record_custom_metric(msg.name, msg.value)?;
            }
        }
        
        Ok(())
    }
}

/// Handle GetSystemMetrics
impl Handler<GetSystemMetrics> for MetricsCollectorActor {
    type Result = SystemMetrics;
    
    fn handle(&mut self, _msg: GetSystemMetrics, _ctx: &mut Self::Context) -> Self::Result {
        self.system_metrics.clone()
    }
}

/// Handle GetPerformanceMetrics
impl Handler<GetPerformanceMetrics> for MetricsCollectorActor {
    type Result = PerformanceMetrics;
    
    fn handle(&mut self, _msg: GetPerformanceMetrics, _ctx: &mut Self::Context) -> Self::Result {
        self.performance_metrics.clone()
    }
}

/// Handle GetCustomMetrics
impl Handler<GetCustomMetrics> for MetricsCollectorActor {
    type Result = HashMap<String, MetricValue>;
    
    fn handle(&mut self, msg: GetCustomMetrics, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(filter) = msg.filter {
            self.custom_metrics
                .iter()
                .filter(|(name, _)| name.contains(&filter))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            self.custom_metrics.clone()
        }
    }
}

/// Handle GetAllMetrics
impl Handler<GetAllMetrics> for MetricsCollectorActor {
    type Result = AllMetrics;
    
    fn handle(&mut self, _msg: GetAllMetrics, _ctx: &mut Self::Context) -> Self::Result {
        AllMetrics {
            system: self.system_metrics.clone(),
            performance: self.performance_metrics.clone(),
            custom: self.custom_metrics.clone(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Handle ResetMetrics
impl Handler<ResetMetrics> for MetricsCollectorActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: ResetMetrics, _ctx: &mut Self::Context) -> Self::Result {
        match msg.metric_type {
            Some(MetricType::System) | None => {
                self.system_metrics = SystemMetrics::default();
            }
            Some(MetricType::Performance) => {
                self.performance_metrics = PerformanceMetrics::default();
            }
            Some(MetricType::Custom) => {
                self.custom_metrics.clear();
            }
            Some(MetricType::All) => {
                self.system_metrics = SystemMetrics::default();
                self.performance_metrics = PerformanceMetrics::default();
                self.custom_metrics.clear();
            }
        }
        
        info!("Metrics reset: {:?}", msg.metric_type);
        Ok(())
    }
}

/// Handle CollectMetrics
impl Handler<CollectMetrics> for MetricsCollectorActor {
    type Result = ();
    
    fn handle(&mut self, _msg: CollectMetrics, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Collecting periodic metrics");
        
        self.collect_system_metrics();
        self.collect_performance_metrics();
        
        // Calculate messages per second
        if self.system_metrics.uptime_seconds > 0 {
            self.system_metrics.messages_per_second = 
                self.system_metrics.total_messages_processed as f64 / self.system_metrics.uptime_seconds as f64;
        }
        
        // Calculate throughput
        self.performance_metrics.throughput_ops_per_second = self.system_metrics.messages_per_second;
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval_ms: 5000, // 5 seconds
            retention_period_hours: 24,   // 24 hours
            enable_system_metrics: true,
            enable_performance_metrics: true,
            max_custom_metrics: 1000,
        }
    }
}
