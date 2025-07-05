//! 框架管理器
//! 
//! 统一管理所有AI框架适配器的生命周期和交互

use crate::framework::{FrameworkType, FrameworkAdapter, FrameworkConfig};
use crate::message_converter::{MessageConverter, ConversionStats};
use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use chrono::{DateTime, Utc};

/// 框架管理器
pub struct FrameworkManager {
    /// 已注册的框架适配器
    adapters: Arc<RwLock<HashMap<FrameworkType, Box<dyn FrameworkAdapter>>>>,
    /// 消息转换器
    message_converter: Arc<RwLock<MessageConverter>>,
    /// 框架状态
    framework_states: Arc<RwLock<HashMap<FrameworkType, FrameworkState>>>,
    /// 管理器配置
    config: FrameworkManagerConfig,
}

/// 框架状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkState {
    /// 框架类型
    pub framework_type: FrameworkType,
    /// 当前状态
    pub status: FrameworkStatus,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
    /// 处理的消息数量
    pub messages_processed: u64,
    /// 错误计数
    pub error_count: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 健康状态
    pub health_status: HealthStatus,
    /// 配置信息
    pub config: FrameworkConfig,
}

/// 框架状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkStatus {
    /// 已注册
    Registered,
    /// 初始化中
    Initializing,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// 健康状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// 框架管理器配置
#[derive(Debug, Clone)]
pub struct FrameworkManagerConfig {
    /// 是否启用自动健康检查
    pub enable_health_check: bool,
    /// 健康检查间隔（秒）
    pub health_check_interval_secs: u64,
    /// 是否启用消息转换缓存
    pub enable_conversion_cache: bool,
    /// 最大并发框架数量
    pub max_concurrent_frameworks: usize,
    /// 消息处理超时（秒）
    pub message_timeout_secs: u64,
}

impl Default for FrameworkManagerConfig {
    fn default() -> Self {
        Self {
            enable_health_check: true,
            health_check_interval_secs: 30,
            enable_conversion_cache: true,
            max_concurrent_frameworks: 10,
            message_timeout_secs: 30,
        }
    }
}

/// 框架交互结果
#[derive(Debug, Clone)]
pub struct FrameworkInteractionResult {
    /// 源框架
    pub source_framework: FrameworkType,
    /// 目标框架
    pub target_framework: Option<FrameworkType>,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error_message: Option<String>,
    /// 响应消息
    pub response_message: Option<A2AMessage>,
}

impl FrameworkManager {
    /// 创建新的框架管理器
    pub fn new(config: FrameworkManagerConfig) -> Self {
        let manager = Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            message_converter: Arc::new(RwLock::new(MessageConverter::new())),
            framework_states: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // 启动健康检查任务
        if manager.config.enable_health_check {
            manager.start_health_check_task();
        }

        manager
    }

    /// 注册框架适配器
    pub async fn register_framework(
        &self,
        framework_type: FrameworkType,
        adapter: Box<dyn FrameworkAdapter>,
        config: FrameworkConfig,
    ) -> A2AResult<()> {
        info!("注册框架适配器: {:?}", framework_type);

        // 检查是否已达到最大框架数量
        let current_count = self.adapters.read().await.len();
        if current_count >= self.config.max_concurrent_frameworks {
            return Err(A2AError::internal(format!(
                "已达到最大框架数量限制: {}",
                self.config.max_concurrent_frameworks
            )));
        }

        // 创建框架状态
        let state = FrameworkState {
            framework_type: framework_type.clone(),
            status: FrameworkStatus::Registered,
            registered_at: Utc::now(),
            last_activity: Utc::now(),
            messages_processed: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            health_status: HealthStatus::Unknown,
            config,
        };

        // 存储适配器和状态
        self.adapters.write().await.insert(framework_type.clone(), adapter);
        self.framework_states.write().await.insert(framework_type.clone(), state);

        info!("框架适配器注册成功: {:?}", framework_type);
        Ok(())
    }

    /// 注销框架适配器
    pub async fn unregister_framework(&self, framework_type: &FrameworkType) -> A2AResult<()> {
        info!("注销框架适配器: {:?}", framework_type);

        // 先停止框架
        if let Err(e) = self.stop_framework(framework_type).await {
            warn!("停止框架时出错: {}", e);
        }

        // 移除适配器和状态
        self.adapters.write().await.remove(framework_type);
        self.framework_states.write().await.remove(framework_type);

        info!("框架适配器注销成功: {:?}", framework_type);
        Ok(())
    }

    /// 启动框架
    pub async fn start_framework(&self, framework_type: &FrameworkType) -> A2AResult<()> {
        info!("启动框架: {:?}", framework_type);

        let adapter = {
            let adapters = self.adapters.read().await;
            adapters.get(framework_type).is_some()
        };

        if !adapter {
            return Err(A2AError::internal(format!("框架未注册: {:?}", framework_type)));
        }

        // 更新状态为初始化中
        self.update_framework_status(framework_type, FrameworkStatus::Initializing).await;

        // 初始化和启动框架
        let result = {
            let mut adapters = self.adapters.write().await;
            if let Some(adapter) = adapters.get_mut(framework_type) {
                // 初始化环境
                adapter.initialize_environment().await?;
                // 启动框架
                adapter.start_framework().await
            } else {
                return Err(A2AError::internal(format!("框架适配器未找到: {:?}", framework_type)));
            }
        };

        match result {
            Ok(_) => {
                self.update_framework_status(framework_type, FrameworkStatus::Running).await;
                info!("框架启动成功: {:?}", framework_type);
                Ok(())
            }
            Err(e) => {
                self.update_framework_status(framework_type, FrameworkStatus::Error(e.to_string())).await;
                error!("框架启动失败: {:?} - {}", framework_type, e);
                Err(e)
            }
        }
    }

    /// 停止框架
    pub async fn stop_framework(&self, framework_type: &FrameworkType) -> A2AResult<()> {
        info!("停止框架: {:?}", framework_type);

        // 更新状态为停止中
        self.update_framework_status(framework_type, FrameworkStatus::Stopping).await;

        let result = {
            let mut adapters = self.adapters.write().await;
            if let Some(adapter) = adapters.get_mut(framework_type) {
                adapter.stop_framework().await
            } else {
                return Err(A2AError::internal(format!("框架适配器未找到: {:?}", framework_type)));
            }
        };

        match result {
            Ok(_) => {
                self.update_framework_status(framework_type, FrameworkStatus::Stopped).await;
                info!("框架停止成功: {:?}", framework_type);
                Ok(())
            }
            Err(e) => {
                self.update_framework_status(framework_type, FrameworkStatus::Error(e.to_string())).await;
                error!("框架停止失败: {:?} - {}", framework_type, e);
                Err(e)
            }
        }
    }

    /// 处理消息（单框架）
    pub async fn process_message(
        &self,
        framework_type: &FrameworkType,
        message: A2AMessage,
    ) -> A2AResult<FrameworkInteractionResult> {
        let start_time = std::time::Instant::now();
        debug!("处理消息 - 框架: {:?}, 消息ID: {}", framework_type, message.message_id);

        // 检查框架状态
        let is_running = {
            let states = self.framework_states.read().await;
            states.get(framework_type)
                .map(|state| state.status == FrameworkStatus::Running)
                .unwrap_or(false)
        };

        if !is_running {
            return Ok(FrameworkInteractionResult {
                source_framework: framework_type.clone(),
                target_framework: None,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                success: false,
                error_message: Some("框架未运行".to_string()),
                response_message: None,
            });
        }

        // 转换消息格式
        let framework_message = {
            let mut converter = self.message_converter.write().await;
            converter.convert_from_a2a(&message, framework_type.clone())?
        };

        // 处理消息
        let result = {
            let adapters = self.adapters.read().await;
            if let Some(adapter) = adapters.get(framework_type) {
                adapter.convert_message_from_framework(framework_message).await
            } else {
                return Err(A2AError::internal(format!("框架适配器未找到: {:?}", framework_type)));
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // 更新统计信息
        self.update_framework_stats(framework_type, processing_time, result.is_ok()).await;

        match result {
            Ok(response_message) => {
                Ok(FrameworkInteractionResult {
                    source_framework: framework_type.clone(),
                    target_framework: None,
                    processing_time_ms: processing_time,
                    success: true,
                    error_message: None,
                    response_message: Some(response_message),
                })
            }
            Err(e) => {
                Ok(FrameworkInteractionResult {
                    source_framework: framework_type.clone(),
                    target_framework: None,
                    processing_time_ms: processing_time,
                    success: false,
                    error_message: Some(e.to_string()),
                    response_message: None,
                })
            }
        }
    }

    /// 框架间消息转发
    pub async fn forward_message(
        &self,
        source_framework: &FrameworkType,
        target_framework: &FrameworkType,
        message: A2AMessage,
    ) -> A2AResult<FrameworkInteractionResult> {
        let start_time = std::time::Instant::now();
        debug!("转发消息 - 从 {:?} 到 {:?}", source_framework, target_framework);

        // 检查两个框架都在运行
        let both_running = {
            let states = self.framework_states.read().await;
            let source_running = states.get(source_framework)
                .map(|state| state.status == FrameworkStatus::Running)
                .unwrap_or(false);
            let target_running = states.get(target_framework)
                .map(|state| state.status == FrameworkStatus::Running)
                .unwrap_or(false);
            source_running && target_running
        };

        if !both_running {
            return Ok(FrameworkInteractionResult {
                source_framework: source_framework.clone(),
                target_framework: Some(target_framework.clone()),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                success: false,
                error_message: Some("源框架或目标框架未运行".to_string()),
                response_message: None,
            });
        }

        // 转换消息格式
        let target_message = {
            let mut converter = self.message_converter.write().await;
            let source_format = converter.convert_from_a2a(&message, source_framework.clone())?;
            converter.convert_between_frameworks(source_format, source_framework.clone(), target_framework.clone())?
        };

        // 发送到目标框架
        let result = {
            let adapters = self.adapters.read().await;
            if let Some(adapter) = adapters.get(target_framework) {
                adapter.convert_message_from_framework(target_message).await
            } else {
                return Err(A2AError::internal(format!("目标框架适配器未找到: {:?}", target_framework)));
            }
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        // 更新两个框架的统计信息
        self.update_framework_stats(source_framework, processing_time / 2, result.is_ok()).await;
        self.update_framework_stats(target_framework, processing_time / 2, result.is_ok()).await;

        match result {
            Ok(response_message) => {
                Ok(FrameworkInteractionResult {
                    source_framework: source_framework.clone(),
                    target_framework: Some(target_framework.clone()),
                    processing_time_ms: processing_time,
                    success: true,
                    error_message: None,
                    response_message: Some(response_message),
                })
            }
            Err(e) => {
                Ok(FrameworkInteractionResult {
                    source_framework: source_framework.clone(),
                    target_framework: Some(target_framework.clone()),
                    processing_time_ms: processing_time,
                    success: false,
                    error_message: Some(e.to_string()),
                    response_message: None,
                })
            }
        }
    }

    /// 获取框架状态
    pub async fn get_framework_state(&self, framework_type: &FrameworkType) -> Option<FrameworkState> {
        self.framework_states.read().await.get(framework_type).cloned()
    }

    /// 获取所有框架状态
    pub async fn get_all_framework_states(&self) -> HashMap<FrameworkType, FrameworkState> {
        self.framework_states.read().await.clone()
    }

    /// 获取转换统计信息
    pub async fn get_conversion_stats(&self) -> ConversionStats {
        self.message_converter.read().await.get_stats().clone()
    }

    /// 获取支持的框架列表
    pub async fn get_supported_frameworks(&self) -> Vec<FrameworkType> {
        self.adapters.read().await.keys().cloned().collect()
    }

    // 私有方法

    async fn update_framework_status(&self, framework_type: &FrameworkType, status: FrameworkStatus) {
        if let Some(state) = self.framework_states.write().await.get_mut(framework_type) {
            state.status = status;
            state.last_activity = Utc::now();
        }
    }

    async fn update_framework_stats(&self, framework_type: &FrameworkType, processing_time_ms: u64, success: bool) {
        if let Some(state) = self.framework_states.write().await.get_mut(framework_type) {
            state.messages_processed += 1;
            if !success {
                state.error_count += 1;
            }
            
            // 更新平均响应时间
            let total_time = state.avg_response_time_ms * (state.messages_processed - 1) as f64 + processing_time_ms as f64;
            state.avg_response_time_ms = total_time / state.messages_processed as f64;
            
            state.last_activity = Utc::now();
        }
    }

    fn start_health_check_task(&self) {
        let adapters = self.adapters.clone();
        let framework_states = self.framework_states.clone();
        let interval_secs = self.config.health_check_interval_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval_secs)
            );

            loop {
                interval.tick().await;

                let framework_types: Vec<FrameworkType> = {
                    adapters.read().await.keys().cloned().collect()
                };

                for framework_type in framework_types {
                    let health_result = {
                        let adapters_guard = adapters.read().await;
                        if let Some(adapter) = adapters_guard.get(&framework_type) {
                            adapter.check_health().await
                        } else {
                            continue;
                        }
                    };

                    let health_status = match health_result {
                        Ok(true) => HealthStatus::Healthy,
                        Ok(false) => HealthStatus::Unhealthy,
                        Err(_) => HealthStatus::Unhealthy,
                    };

                    // 更新健康状态
                    if let Some(state) = framework_states.write().await.get_mut(&framework_type) {
                        state.health_status = health_status;
                        state.last_activity = Utc::now();
                    }
                }
            }
        });
    }
}
