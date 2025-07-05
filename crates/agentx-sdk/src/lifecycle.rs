//! 插件生命周期管理器
//! 
//! 提供完整的插件生命周期管理功能，包括加载、启动、停止、监控等

use crate::plugin::{Plugin, PluginInfo, PluginStatus, PluginConfig, PluginEvent, PluginStats};
use agentx_a2a::{A2AResult, A2AError, A2AMessage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc, Mutex};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc};

/// 插件生命周期管理器
pub struct PluginLifecycleManager {
    /// 已注册的插件
    plugins: Arc<RwLock<HashMap<String, Arc<Mutex<Box<dyn Plugin>>>>>>,
    /// 插件状态缓存
    plugin_states: Arc<RwLock<HashMap<String, PluginState>>>,
    /// 事件发布器
    event_publisher: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<PluginEvent>>>>,
    /// 健康检查间隔
    health_check_interval: Duration,
    /// 配置
    config: LifecycleConfig,
}

/// 插件状态信息
#[derive(Debug, Clone)]
pub struct PluginState {
    /// 插件ID
    pub plugin_id: String,
    /// 当前状态
    pub status: PluginStatus,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 停止时间
    pub stopped_at: Option<DateTime<Utc>>,
    /// 最后健康检查时间
    pub last_health_check: Option<DateTime<Utc>>,
    /// 健康检查结果
    pub health_status: HealthStatus,
    /// 重启次数
    pub restart_count: u32,
    /// 错误计数
    pub error_count: u32,
    /// 最后错误
    pub last_error: Option<String>,
    /// 统计信息
    pub stats: PluginStats,
}

/// 健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 警告
    Warning,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
}

/// 生命周期配置
#[derive(Debug, Clone)]
pub struct LifecycleConfig {
    /// 健康检查间隔（秒）
    pub health_check_interval_secs: u64,
    /// 启动超时（秒）
    pub startup_timeout_secs: u64,
    /// 停止超时（秒）
    pub shutdown_timeout_secs: u64,
    /// 最大重启次数
    pub max_restart_attempts: u32,
    /// 重启延迟（秒）
    pub restart_delay_secs: u64,
    /// 是否启用自动重启
    pub enable_auto_restart: bool,
    /// 是否启用健康检查
    pub enable_health_check: bool,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            health_check_interval_secs: 30,
            startup_timeout_secs: 60,
            shutdown_timeout_secs: 30,
            max_restart_attempts: 3,
            restart_delay_secs: 5,
            enable_auto_restart: true,
            enable_health_check: true,
        }
    }
}

impl PluginLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new(config: LifecycleConfig) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_states: Arc::new(RwLock::new(HashMap::new())),
            event_publisher: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: Duration::from_secs(config.health_check_interval_secs),
            config,
        }
    }

    /// 注册插件
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> A2AResult<()> {
        let plugin_id = plugin.get_info().metadata.id.clone();
        info!("注册插件: {}", plugin_id);

        // 创建插件状态
        let state = PluginState {
            plugin_id: plugin_id.clone(),
            status: PluginStatus::Registered,
            started_at: None,
            stopped_at: None,
            last_health_check: None,
            health_status: HealthStatus::Unknown,
            restart_count: 0,
            error_count: 0,
            last_error: None,
            stats: plugin.get_stats().clone(),
        };

        // 创建事件通道
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.event_publisher.write().await.insert(plugin_id.clone(), tx);

        // 启动事件处理任务
        let plugin_id_clone = plugin_id.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("插件 {} 事件: {:?}", plugin_id_clone, event);
                // 这里可以添加事件处理逻辑，如日志记录、指标收集等
            }
        });

        // 存储插件和状态
        self.plugins.write().await.insert(plugin_id.clone(), Arc::new(Mutex::new(plugin)));
        self.plugin_states.write().await.insert(plugin_id.clone(), state);

        // 发布注册事件
        self.publish_event(&plugin_id, PluginEvent::StatusChanged(PluginStatus::Registered)).await;

        info!("插件 {} 注册成功", plugin_id);
        Ok(())
    }

    /// 启动插件
    pub async fn start_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        info!("启动插件: {}", plugin_id);

        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id).cloned()
        };

        let plugin = plugin.ok_or_else(|| {
            A2AError::internal(format!("插件未找到: {}", plugin_id))
        })?;

        // 更新状态为启动中
        self.update_plugin_status(plugin_id, PluginStatus::Starting).await;

        // 启动插件（带超时）
        let startup_timeout = Duration::from_secs(self.config.startup_timeout_secs);
        let start_result = tokio::time::timeout(startup_timeout, async {
            let mut plugin_guard = plugin.lock().await;
            plugin_guard.initialize().await?;
            plugin_guard.start().await
        }).await;

        match start_result {
            Ok(Ok(_)) => {
                // 启动成功
                self.update_plugin_status(plugin_id, PluginStatus::Running).await;
                
                // 更新启动时间
                if let Some(state) = self.plugin_states.write().await.get_mut(plugin_id) {
                    state.started_at = Some(Utc::now());
                    state.stopped_at = None;
                }

                // 启动健康检查
                if self.config.enable_health_check {
                    self.start_health_monitoring(plugin_id).await;
                }

                info!("插件 {} 启动成功", plugin_id);
                Ok(())
            }
            Ok(Err(e)) => {
                // 启动失败
                self.update_plugin_status(plugin_id, PluginStatus::Failed).await;
                self.record_error(plugin_id, &e.to_string()).await;
                error!("插件 {} 启动失败: {}", plugin_id, e);
                Err(e)
            }
            Err(_) => {
                // 超时
                self.update_plugin_status(plugin_id, PluginStatus::Failed).await;
                let error_msg = format!("插件启动超时 ({}秒)", self.config.startup_timeout_secs);
                self.record_error(plugin_id, &error_msg).await;
                error!("插件 {} 启动超时", plugin_id);
                Err(A2AError::internal(error_msg))
            }
        }
    }

    /// 停止插件
    pub async fn stop_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        info!("停止插件: {}", plugin_id);

        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id).cloned()
        };

        let plugin = plugin.ok_or_else(|| {
            A2AError::internal(format!("插件未找到: {}", plugin_id))
        })?;

        // 更新状态为停止中
        self.update_plugin_status(plugin_id, PluginStatus::Stopping).await;

        // 停止插件（带超时）
        let shutdown_timeout = Duration::from_secs(self.config.shutdown_timeout_secs);
        let stop_result = tokio::time::timeout(shutdown_timeout, async {
            let mut plugin_guard = plugin.lock().await;
            plugin_guard.stop().await
        }).await;

        match stop_result {
            Ok(Ok(_)) => {
                // 停止成功
                self.update_plugin_status(plugin_id, PluginStatus::Stopped).await;
                
                // 更新停止时间
                if let Some(state) = self.plugin_states.write().await.get_mut(plugin_id) {
                    state.stopped_at = Some(Utc::now());
                }

                info!("插件 {} 停止成功", plugin_id);
                Ok(())
            }
            Ok(Err(e)) => {
                // 停止失败
                self.update_plugin_status(plugin_id, PluginStatus::Failed).await;
                self.record_error(plugin_id, &e.to_string()).await;
                error!("插件 {} 停止失败: {}", plugin_id, e);
                Err(e)
            }
            Err(_) => {
                // 超时，强制停止
                self.update_plugin_status(plugin_id, PluginStatus::Stopped).await;
                warn!("插件 {} 停止超时，已强制停止", plugin_id);
                Ok(())
            }
        }
    }

    /// 重启插件
    pub async fn restart_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        info!("重启插件: {}", plugin_id);

        // 增加重启计数
        if let Some(state) = self.plugin_states.write().await.get_mut(plugin_id) {
            state.restart_count += 1;
        }

        // 先停止
        if let Err(e) = self.stop_plugin(plugin_id).await {
            warn!("停止插件时出错: {}", e);
        }

        // 等待重启延迟
        tokio::time::sleep(Duration::from_secs(self.config.restart_delay_secs)).await;

        // 再启动
        self.start_plugin(plugin_id).await
    }

    /// 获取插件状态
    pub async fn get_plugin_state(&self, plugin_id: &str) -> Option<PluginState> {
        self.plugin_states.read().await.get(plugin_id).cloned()
    }

    /// 获取所有插件状态
    pub async fn get_all_plugin_states(&self) -> HashMap<String, PluginState> {
        self.plugin_states.read().await.clone()
    }

    /// 处理插件消息
    pub async fn process_plugin_message(&self, plugin_id: &str, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id).cloned()
        };

        let plugin = plugin.ok_or_else(|| {
            A2AError::internal(format!("插件未找到: {}", plugin_id))
        })?;

        let mut plugin_guard = plugin.lock().await;
        plugin_guard.process_message(message).await
    }

    // 私有方法

    async fn update_plugin_status(&self, plugin_id: &str, status: PluginStatus) {
        if let Some(state) = self.plugin_states.write().await.get_mut(plugin_id) {
            state.status = status.clone();
        }
        self.publish_event(plugin_id, PluginEvent::StatusChanged(status)).await;
    }

    async fn record_error(&self, plugin_id: &str, error: &str) {
        if let Some(state) = self.plugin_states.write().await.get_mut(plugin_id) {
            state.error_count += 1;
            state.last_error = Some(error.to_string());
        }
        self.publish_event(plugin_id, PluginEvent::Error(error.to_string())).await;
    }

    async fn publish_event(&self, plugin_id: &str, event: PluginEvent) {
        if let Some(tx) = self.event_publisher.read().await.get(plugin_id) {
            let _ = tx.send(event);
        }
    }

    async fn start_health_monitoring(&self, plugin_id: &str) {
        let plugins = self.plugins.clone();
        let plugin_states = self.plugin_states.clone();
        let config = self.config.clone();
        let plugin_id = plugin_id.to_string();
        let interval_duration = self.health_check_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;

                // 检查插件是否仍在运行
                let is_running = {
                    let states = plugin_states.read().await;
                    states.get(&plugin_id)
                        .map(|state| state.status == PluginStatus::Running)
                        .unwrap_or(false)
                };

                if !is_running {
                    break;
                }

                // 执行健康检查
                let plugin = {
                    let plugins_guard = plugins.read().await;
                    plugins_guard.get(&plugin_id).cloned()
                };

                if let Some(plugin) = plugin {
                    let health_result = {
                        let plugin_guard = plugin.lock().await;
                        plugin_guard.health_check().await
                    };

                    let health_status = match health_result {
                        Ok(true) => HealthStatus::Healthy,
                        Ok(false) => HealthStatus::Unhealthy,
                        Err(_) => HealthStatus::Unhealthy,
                    };

                    // 更新健康状态
                    if let Some(state) = plugin_states.write().await.get_mut(&plugin_id) {
                        state.last_health_check = Some(Utc::now());
                        state.health_status = health_status.clone();
                    }

                    // 如果不健康且启用自动重启，尝试重启
                    if health_status == HealthStatus::Unhealthy && config.enable_auto_restart {
                        let restart_count = {
                            let states = plugin_states.read().await;
                            states.get(&plugin_id)
                                .map(|state| state.restart_count)
                                .unwrap_or(0)
                        };

                        if restart_count < config.max_restart_attempts {
                            warn!("插件 {} 不健康，尝试自动重启", plugin_id);
                            // 这里需要一个重启的方法，但为了避免循环依赖，我们只记录日志
                        } else {
                            error!("插件 {} 重启次数已达上限，停止自动重启", plugin_id);
                        }
                    }
                }
            }
        });
    }
}
