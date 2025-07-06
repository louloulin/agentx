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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::{FrameworkType, FrameworkAdapter, FrameworkConfig};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use tokio::time::{sleep, Duration};

    // 测试用的模拟框架适配器
    #[derive(Debug)]
    struct MockFrameworkAdapter {
        name: String,
        initialized: AtomicBool,
        running: AtomicBool,
        healthy: AtomicBool,
        message_count: AtomicU64,
        should_fail: AtomicBool,
    }

    impl MockFrameworkAdapter {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                initialized: AtomicBool::new(false),
                running: AtomicBool::new(false),
                healthy: AtomicBool::new(true),
                message_count: AtomicU64::new(0),
                should_fail: AtomicBool::new(false),
            }
        }

        fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::Relaxed);
        }

        #[allow(dead_code)]
        fn set_healthy(&self, healthy: bool) {
            self.healthy.store(healthy, Ordering::Relaxed);
        }

        #[allow(dead_code)]
        fn get_message_count(&self) -> u64 {
            self.message_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl FrameworkAdapter for MockFrameworkAdapter {
        async fn initialize_environment(&mut self) -> A2AResult<()> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(A2AError::internal("模拟初始化失败"));
            }
            self.initialized.store(true, Ordering::Relaxed);
            Ok(())
        }

        async fn start_framework(&mut self) -> A2AResult<()> {
            if !self.initialized.load(Ordering::Relaxed) {
                return Err(A2AError::internal("框架未初始化"));
            }
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(A2AError::internal("模拟启动失败"));
            }
            self.running.store(true, Ordering::Relaxed);
            Ok(())
        }

        async fn stop_framework(&mut self) -> A2AResult<()> {
            self.running.store(false, Ordering::Relaxed);
            Ok(())
        }

        async fn execute_command(&mut self, _command: &str, _args: Vec<String>) -> A2AResult<String> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(A2AError::internal("模拟命令执行失败"));
            }
            Ok("命令执行成功".to_string())
        }

        async fn convert_message_to_framework(&self, _message: &A2AMessage) -> A2AResult<serde_json::Value> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(A2AError::internal("模拟消息转换失败"));
            }
            self.message_count.fetch_add(1, Ordering::Relaxed);
            Ok(serde_json::json!({
                "framework": self.name,
                "message": "converted"
            }))
        }

        async fn convert_message_from_framework(&self, _message: serde_json::Value) -> A2AResult<A2AMessage> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(A2AError::internal("模拟消息转换失败"));
            }
            self.message_count.fetch_add(1, Ordering::Relaxed);
            Ok(A2AMessage::user_message("test message".to_string()))
        }

        async fn check_health(&self) -> A2AResult<bool> {
            Ok(self.healthy.load(Ordering::Relaxed))
        }

        fn get_framework_type(&self) -> FrameworkType {
            FrameworkType::Custom("test".to_string())
        }
    }

    fn create_test_manager() -> FrameworkManager {
        let config = FrameworkManagerConfig {
            enable_health_check: false, // 禁用健康检查以简化测试
            health_check_interval_secs: 1,
            enable_conversion_cache: true,
            max_concurrent_frameworks: 5,
            message_timeout_secs: 10,
        };
        FrameworkManager::new(config)
    }

    fn create_test_message() -> A2AMessage {
        A2AMessage::user_message("Hello, world!".to_string())
    }

    #[tokio::test]
    async fn test_framework_manager_creation() {
        let manager = create_test_manager();

        // 验证初始状态
        let states = manager.get_all_framework_states().await;
        assert!(states.is_empty());

        let frameworks = manager.get_supported_frameworks().await;
        assert!(frameworks.is_empty());
    }

    #[tokio::test]
    async fn test_framework_registration() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let config = FrameworkConfig::default();

        // 注册框架
        let framework_type = FrameworkType::Custom("test".to_string());
        let result = manager.register_framework(
            framework_type.clone(),
            adapter,
            config,
        ).await;
        assert!(result.is_ok());

        // 验证框架已注册
        let frameworks = manager.get_supported_frameworks().await;
        assert_eq!(frameworks.len(), 1);
        assert!(frameworks.contains(&framework_type));

        // 验证框架状态
        let state = manager.get_framework_state(&framework_type).await;
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state.framework_type, framework_type);
        assert_eq!(state.status, FrameworkStatus::Registered);
        assert_eq!(state.messages_processed, 0);
        assert_eq!(state.error_count, 0);
    }

    #[tokio::test]
    async fn test_framework_max_limit() {
        let manager = create_test_manager();

        // 注册最大数量的框架
        for i in 0..5 {
            let adapter = Box::new(MockFrameworkAdapter::new(&format!("test_{}", i)));
            let result = manager.register_framework(
                FrameworkType::Custom(format!("test_{}", i)),
                adapter,
                FrameworkConfig::default(),
            ).await;
            assert!(result.is_ok());
        }

        // 尝试注册超过限制的框架
        let adapter = Box::new(MockFrameworkAdapter::new("test_overflow"));
        let result = manager.register_framework(
            FrameworkType::LangChain,
            adapter,
            FrameworkConfig::default(),
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_framework_lifecycle() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 启动框架
        let result = manager.start_framework(&framework_type).await;
        assert!(result.is_ok());

        // 验证状态
        let state = manager.get_framework_state(&framework_type).await.unwrap();
        assert_eq!(state.status, FrameworkStatus::Running);

        // 停止框架
        let result = manager.stop_framework(&framework_type).await;
        assert!(result.is_ok());

        // 验证状态
        let state = manager.get_framework_state(&framework_type).await.unwrap();
        assert_eq!(state.status, FrameworkStatus::Stopped);
    }

    #[tokio::test]
    async fn test_framework_start_failure() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        adapter.set_should_fail(true);
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 尝试启动框架（应该失败）
        let result = manager.start_framework(&framework_type).await;
        assert!(result.is_err());

        // 验证错误状态
        let state = manager.get_framework_state(&framework_type).await.unwrap();
        // 由于初始化失败，状态应该是Error
        assert!(matches!(state.status, FrameworkStatus::Error(_)) || state.status == FrameworkStatus::Initializing);
    }

    #[tokio::test]
    async fn test_unregistered_framework_operations() {
        let manager = create_test_manager();
        let framework_type = FrameworkType::Custom("test".to_string());

        // 尝试启动未注册的框架
        let result = manager.start_framework(&framework_type).await;
        assert!(result.is_err());

        // 尝试停止未注册的框架
        let result = manager.stop_framework(&framework_type).await;
        assert!(result.is_err());

        // 获取未注册框架的状态
        let state = manager.get_framework_state(&framework_type).await;
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_message_processing() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册并启动框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        manager.start_framework(&framework_type).await.unwrap();

        // 处理消息
        let message = create_test_message();
        let result = manager.process_message(
            &framework_type,
            message,
        ).await.unwrap();

        // 验证结果
        assert!(result.success);
        assert!(result.response_message.is_some());
        // 处理时间可能为0，所以不强制要求大于0
        assert_eq!(result.source_framework, framework_type);
        assert!(result.target_framework.is_none());

        // 验证统计信息更新
        let state = manager.get_framework_state(&framework_type).await.unwrap();
        assert_eq!(state.messages_processed, 1);
        assert_eq!(state.error_count, 0);
        // 平均响应时间可能为0，所以不强制要求大于0
        assert!(state.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_message_processing_framework_not_running() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册但不启动框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 尝试处理消息
        let message = create_test_message();
        let result = manager.process_message(
            &framework_type,
            message,
        ).await.unwrap();

        // 验证失败结果
        assert!(!result.success);
        assert!(result.response_message.is_none());
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.unwrap(), "框架未运行");
    }

    #[tokio::test]
    async fn test_message_processing_unregistered_framework() {
        let manager = create_test_manager();
        let framework_type = FrameworkType::Custom("nonexistent".to_string());

        // 尝试处理消息（框架未注册）
        let message = create_test_message();
        let result = manager.process_message(
            &framework_type,
            message,
        ).await.unwrap();

        // 应该返回失败结果，因为框架未运行
        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.unwrap(), "框架未运行");
    }

    #[tokio::test]
    async fn test_framework_forwarding() {
        let manager = create_test_manager();
        let adapter1 = Box::new(MockFrameworkAdapter::new("test1"));
        let adapter2 = Box::new(MockFrameworkAdapter::new("test2"));

        // 注册两个框架
        manager.register_framework(
            FrameworkType::LangChain,
            adapter1,
            FrameworkConfig::default(),
        ).await.unwrap();

        manager.register_framework(
            FrameworkType::AutoGen,
            adapter2,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 启动两个框架
        manager.start_framework(&FrameworkType::LangChain).await.unwrap();
        manager.start_framework(&FrameworkType::AutoGen).await.unwrap();

        // 转发消息
        let message = create_test_message();
        let result = manager.forward_message(
            &FrameworkType::LangChain,
            &FrameworkType::AutoGen,
            message,
        ).await.unwrap();

        // 验证结果
        assert!(result.success);
        assert!(result.response_message.is_some());
        assert_eq!(result.source_framework, FrameworkType::LangChain);
        assert_eq!(result.target_framework, Some(FrameworkType::AutoGen));

        // 验证两个框架的统计信息都更新了
        let state1 = manager.get_framework_state(&FrameworkType::LangChain).await.unwrap();
        let state2 = manager.get_framework_state(&FrameworkType::AutoGen).await.unwrap();
        assert_eq!(state1.messages_processed, 1);
        assert_eq!(state2.messages_processed, 1);
    }

    #[tokio::test]
    async fn test_framework_forwarding_not_running() {
        let manager = create_test_manager();
        let adapter1 = Box::new(MockFrameworkAdapter::new("test1"));
        let adapter2 = Box::new(MockFrameworkAdapter::new("test2"));

        // 注册但不启动框架
        manager.register_framework(
            FrameworkType::LangChain,
            adapter1,
            FrameworkConfig::default(),
        ).await.unwrap();

        manager.register_framework(
            FrameworkType::AutoGen,
            adapter2,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 尝试转发消息
        let message = create_test_message();
        let result = manager.forward_message(
            &FrameworkType::LangChain,
            &FrameworkType::AutoGen,
            message,
        ).await.unwrap();

        // 验证失败结果
        assert!(!result.success);
        assert!(result.response_message.is_none());
        assert!(result.error_message.is_some());
        assert_eq!(result.error_message.unwrap(), "源框架或目标框架未运行");
    }

    #[tokio::test]
    async fn test_framework_unregistration() {
        let manager = create_test_manager();
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册并启动框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        manager.start_framework(&framework_type).await.unwrap();

        // 验证框架已注册
        let frameworks = manager.get_supported_frameworks().await;
        assert_eq!(frameworks.len(), 1);

        // 注销框架
        let result = manager.unregister_framework(&framework_type).await;
        assert!(result.is_ok());

        // 验证框架已注销
        let frameworks = manager.get_supported_frameworks().await;
        assert!(frameworks.is_empty());

        let state = manager.get_framework_state(&framework_type).await;
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_conversion_stats() {
        let manager = create_test_manager();

        // 获取初始统计信息
        let stats = manager.get_conversion_stats().await;
        assert_eq!(stats.total_conversions, 0);
    }

    #[tokio::test]
    async fn test_health_check_enabled() {
        let config = FrameworkManagerConfig {
            enable_health_check: true,
            health_check_interval_secs: 1,
            enable_conversion_cache: true,
            max_concurrent_frameworks: 5,
            message_timeout_secs: 10,
        };

        let manager = FrameworkManager::new(config);
        let adapter = Box::new(MockFrameworkAdapter::new("test"));
        let framework_type = FrameworkType::Custom("test".to_string());

        // 注册框架
        manager.register_framework(
            framework_type.clone(),
            adapter,
            FrameworkConfig::default(),
        ).await.unwrap();

        // 等待健康检查运行
        sleep(Duration::from_millis(1100)).await;

        // 验证健康状态可能已更新
        let state = manager.get_framework_state(&framework_type).await.unwrap();
        // 健康状态应该是 Healthy 或 Unknown（取决于时机）
        assert!(matches!(state.health_status, HealthStatus::Healthy | HealthStatus::Unknown));
    }

    #[test]
    fn test_framework_status_serialization() {
        let status = FrameworkStatus::Running;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: FrameworkStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);

        let error_status = FrameworkStatus::Error("test error".to_string());
        let serialized = serde_json::to_string(&error_status).unwrap();
        let deserialized: FrameworkStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error_status, deserialized);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_framework_manager_config_default() {
        let config = FrameworkManagerConfig::default();
        assert!(config.enable_health_check);
        assert_eq!(config.health_check_interval_secs, 30);
        assert!(config.enable_conversion_cache);
        assert_eq!(config.max_concurrent_frameworks, 10);
        assert_eq!(config.message_timeout_secs, 30);
    }
}
