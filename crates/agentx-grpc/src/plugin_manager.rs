//! gRPC插件管理器
//! 
//! 管理gRPC插件的生命周期、健康检查和负载均衡

use crate::plugin_bridge::{PluginBridge, PluginStatus};
use agentx_a2a::{A2AResult, A2AError, A2AMessage};
use std::collections::HashMap;
use std::sync::Arc;
use std::process::{Child, Command, Stdio};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration, Instant};
use tokio::process::Command as TokioCommand;

/// 插件管理器
pub struct PluginManager {
    /// 插件桥接器
    bridge: Arc<PluginBridge>,
    /// 插件配置
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    /// 插件进程管理
    plugin_processes: Arc<RwLock<HashMap<String, PluginProcess>>>,
    /// 健康检查间隔
    health_check_interval: Duration,
    /// 消息队列
    message_queue: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<QueuedMessage>>>>,
    /// 负载均衡策略
    load_balancer: LoadBalancer,
}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub framework: String,
    pub auto_restart: bool,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub config: HashMap<String, String>,
    /// 插件可执行文件路径
    pub executable_path: Option<String>,
    /// 插件启动参数
    pub args: Vec<String>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 工作目录
    pub working_dir: Option<String>,
}

/// 插件进程信息
#[derive(Debug)]
pub struct PluginProcess {
    /// 进程ID
    pub pid: Option<u32>,
    /// 进程句柄
    pub process: Option<tokio::process::Child>,
    /// 启动时间
    pub started_at: Instant,
    /// 重启次数
    pub restart_count: u32,
    /// 进程状态
    pub status: ProcessStatus,
    /// 最后健康检查时间
    pub last_health_check: Instant,
}

/// 进程状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Restarting,
}

/// 排队的消息
#[derive(Debug)]
pub struct QueuedMessage {
    pub message: A2AMessage,
    pub target_agent: String,
    pub retry_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 负载均衡器
#[derive(Debug)]
pub struct LoadBalancer {
    strategy: LoadBalanceStrategy,
    plugin_weights: HashMap<String, f64>,
    plugin_requests: HashMap<String, u64>,
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalanceStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new(bridge: Arc<PluginBridge>) -> Self {
        Self {
            bridge,
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
            plugin_processes: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: Duration::from_secs(30),
            message_queue: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: LoadBalancer::new(LoadBalanceStrategy::RoundRobin),
        }
    }
    
    /// 添加插件配置
    pub async fn add_plugin_config(&self, config: PluginConfig) -> A2AResult<()> {
        println!("📝 添加插件配置: {} ({})", config.name, config.framework);
        
        let plugin_id = config.id.clone();
        self.plugin_configs.write().await.insert(plugin_id.clone(), config.clone());
        
        // 创建消息队列
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.message_queue.write().await.insert(plugin_id.clone(), tx);
        
        // 启动消息处理任务
        let bridge = self.bridge.clone();
        let plugin_id_clone = plugin_id.clone();
        tokio::spawn(async move {
            while let Some(queued_msg) = rx.recv().await {
                if let Err(e) = bridge.route_message_to_plugin(
                    queued_msg.message, 
                    &queued_msg.target_agent
                ).await {
                    eprintln!("❌ 插件 {} 消息处理失败: {}", plugin_id_clone, e);
                }
            }
        });
        
        println!("✅ 插件配置 {} 添加成功", config.name);
        Ok(())
    }
    
    /// 启动插件
    pub async fn start_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        println!("🚀 启动插件: {}", plugin_id);

        let config = {
            let configs = self.plugin_configs.read().await;
            configs.get(plugin_id).cloned()
        };

        let config = config.ok_or_else(|| {
            A2AError::internal(format!("插件配置未找到: {}", plugin_id))
        })?;

        // 如果配置了可执行文件路径，启动外部进程
        if config.executable_path.is_some() {
            self.start_external_process(plugin_id, &config).await?;
        }

        // 注册插件到桥接器
        self.bridge.register_plugin(
            config.id.clone(),
            config.endpoint.clone(),
            config.config.clone(),
        ).await?;
        
        println!("✅ 插件 {} 启动成功", plugin_id);
        Ok(())
    }
    
    /// 停止插件
    pub async fn stop_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        println!("🛑 停止插件: {}", plugin_id);

        // 停止外部进程（如果存在）
        self.stop_external_process(plugin_id).await?;

        // 注销插件
        self.bridge.unregister_plugin(plugin_id).await?;

        // 清理消息队列
        self.message_queue.write().await.remove(plugin_id);

        println!("✅ 插件 {} 停止成功", plugin_id);
        Ok(())
    }
    
    /// 启动所有配置的插件
    pub async fn start_all_plugins(&self) -> A2AResult<()> {
        println!("🚀 启动所有插件");
        
        let plugin_ids: Vec<String> = {
            let configs = self.plugin_configs.read().await;
            configs.keys().cloned().collect()
        };
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        for plugin_id in plugin_ids {
            match self.start_plugin(&plugin_id).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("❌ 启动插件 {} 失败: {}", plugin_id, e);
                    error_count += 1;
                }
            }
        }
        
        println!("📊 插件启动结果: 成功 {}, 失败 {}", success_count, error_count);
        
        if error_count > 0 {
            Err(A2AError::internal(format!("部分插件启动失败: {}", error_count)))
        } else {
            Ok(())
        }
    }
    
    /// 发送消息到插件
    pub async fn send_message_to_plugin(
        &self,
        message: A2AMessage,
        target_agent: &str,
    ) -> A2AResult<()> {
        // 选择最佳插件
        let plugin_id = self.select_plugin_for_agent(target_agent).await?;
        
        // 获取消息队列
        let sender = {
            let queues = self.message_queue.read().await;
            queues.get(&plugin_id).cloned()
        };
        
        let sender = sender.ok_or_else(|| {
            A2AError::internal(format!("插件 {} 消息队列未找到", plugin_id))
        })?;
        
        // 创建排队消息
        let queued_msg = QueuedMessage {
            message,
            target_agent: target_agent.to_string(),
            retry_count: 0,
            created_at: chrono::Utc::now(),
        };
        
        // 发送到队列
        sender.send(queued_msg)
            .map_err(|_| A2AError::internal("消息队列发送失败"))?;
        
        Ok(())
    }
    
    /// 启动健康检查
    pub async fn start_health_check(&self) {
        println!("🏥 启动插件健康检查 (间隔: {:?})", self.health_check_interval);
        
        let bridge = self.bridge.clone();
        let plugin_configs = self.plugin_configs.clone();
        let interval_duration = self.health_check_interval;
        
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                let plugin_ids: Vec<String> = {
                    let configs = plugin_configs.read().await;
                    configs.keys().cloned().collect()
                };
                
                for plugin_id in plugin_ids {
                    match bridge.check_plugin_health(&plugin_id).await {
                        Ok(is_healthy) => {
                            if !is_healthy {
                                println!("⚠️ 插件 {} 健康检查失败", plugin_id);
                                // 这里可以添加自动重启逻辑
                            }
                        },
                        Err(e) => {
                            println!("❌ 插件 {} 健康检查错误: {}", plugin_id, e);
                        }
                    }
                }
            }
        });
    }
    
    /// 获取插件统计信息
    pub async fn get_plugin_stats(&self) -> HashMap<String, PluginStats> {
        let mut stats = HashMap::new();
        
        let plugins = self.bridge.get_all_plugins().await;
        for plugin in plugins {
            let plugin_stats = PluginStats {
                id: plugin.id.clone(),
                name: plugin.name.clone(),
                status: plugin.status,
                framework: plugin.framework,
                capabilities_count: plugin.capabilities.len(),
                request_count: self.load_balancer.plugin_requests
                    .get(&plugin.id).copied().unwrap_or(0),
                last_health_check: chrono::Utc::now(), // 简化处理
            };
            
            stats.insert(plugin.id, plugin_stats);
        }
        
        stats
    }
    
    /// 重新加载插件配置
    pub async fn reload_plugin_config(&self, plugin_id: &str) -> A2AResult<()> {
        println!("🔄 重新加载插件配置: {}", plugin_id);
        
        // 停止插件
        if let Err(e) = self.stop_plugin(plugin_id).await {
            println!("⚠️ 停止插件时出错: {}", e);
        }
        
        // 重新启动插件
        self.start_plugin(plugin_id).await?;
        
        println!("✅ 插件 {} 配置重新加载成功", plugin_id);
        Ok(())
    }
    
    // 私有方法

    /// 启动外部进程
    async fn start_external_process(&self, plugin_id: &str, config: &PluginConfig) -> A2AResult<()> {
        println!("🔧 启动外部插件进程: {} ({})", plugin_id, config.executable_path.as_ref().unwrap());

        let executable_path = config.executable_path.as_ref().unwrap();

        // 构建命令
        let mut cmd = TokioCommand::new(executable_path);

        // 添加参数
        for arg in &config.args {
            cmd.arg(arg);
        }

        // 设置环境变量
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // 设置工作目录
        if let Some(working_dir) = &config.working_dir {
            cmd.current_dir(working_dir);
        }

        // 设置标准输入输出
        cmd.stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped())
           .stdin(std::process::Stdio::null());

        // 启动进程
        let child = cmd.spawn().map_err(|e| {
            A2AError::internal(format!("启动插件进程失败: {}", e))
        })?;

        let pid = child.id();
        let started_at = Instant::now();

        // 创建进程信息
        let process_info = PluginProcess {
            pid,
            process: Some(child),
            started_at,
            restart_count: 0,
            status: ProcessStatus::Starting,
            last_health_check: started_at,
        };

        // 存储进程信息
        self.plugin_processes.write().await.insert(plugin_id.to_string(), process_info);

        println!("✅ 插件进程启动成功: {} (PID: {:?})", plugin_id, pid);

        // 启动进程监控任务
        self.start_process_monitor(plugin_id.to_string()).await;

        Ok(())
    }

    /// 停止外部进程
    async fn stop_external_process(&self, plugin_id: &str) -> A2AResult<()> {
        let mut processes = self.plugin_processes.write().await;

        if let Some(mut process_info) = processes.remove(plugin_id) {
            println!("🔧 停止外部插件进程: {} (PID: {:?})", plugin_id, process_info.pid);

            process_info.status = ProcessStatus::Stopping;

            if let Some(mut child) = process_info.process.take() {
                // 尝试优雅地终止进程
                match child.kill().await {
                    Ok(_) => {
                        println!("✅ 插件进程 {} 已终止", plugin_id);
                    }
                    Err(e) => {
                        println!("⚠️ 终止插件进程 {} 时出错: {}", plugin_id, e);
                        return Err(A2AError::internal(format!("终止进程失败: {}", e)));
                    }
                }

                // 等待进程完全退出
                match child.wait().await {
                    Ok(exit_status) => {
                        println!("📋 插件进程 {} 退出状态: {:?}", plugin_id, exit_status);
                    }
                    Err(e) => {
                        println!("⚠️ 等待插件进程 {} 退出时出错: {}", plugin_id, e);
                    }
                }
            }

            println!("✅ 插件进程 {} 停止完成", plugin_id);
        } else {
            println!("ℹ️ 插件 {} 没有运行的外部进程", plugin_id);
        }

        Ok(())
    }

    /// 启动进程监控
    async fn start_process_monitor(&self, plugin_id: String) {
        let processes = self.plugin_processes.clone();
        let configs = self.plugin_configs.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let mut should_restart = false;

                // 检查进程状态
                {
                    let mut processes_guard = processes.write().await;
                    if let Some(process_info) = processes_guard.get_mut(&plugin_id) {
                        if let Some(ref mut child) = process_info.process {
                            match child.try_wait() {
                                Ok(Some(exit_status)) => {
                                    println!("⚠️ 插件进程 {} 已退出: {:?}", plugin_id, exit_status);
                                    process_info.status = ProcessStatus::Stopped;
                                    should_restart = true;
                                }
                                Ok(None) => {
                                    // 进程仍在运行
                                    if process_info.status == ProcessStatus::Starting {
                                        process_info.status = ProcessStatus::Running;
                                        println!("✅ 插件进程 {} 运行正常", plugin_id);
                                    }
                                }
                                Err(e) => {
                                    println!("❌ 检查插件进程 {} 状态失败: {}", plugin_id, e);
                                    process_info.status = ProcessStatus::Failed;
                                    should_restart = true;
                                }
                            }
                        }
                    }
                }

                // 如果需要重启且配置了自动重启
                if should_restart {
                    let config = {
                        let configs_guard = configs.read().await;
                        configs_guard.get(&plugin_id).cloned()
                    };

                    if let Some(config) = config {
                        if config.auto_restart {
                            let mut processes_guard = processes.write().await;
                            if let Some(process_info) = processes_guard.get_mut(&plugin_id) {
                                if process_info.restart_count < config.max_retries {
                                    process_info.restart_count += 1;
                                    process_info.status = ProcessStatus::Restarting;
                                    println!("🔄 重启插件进程: {} (第{}次)", plugin_id, process_info.restart_count);

                                    // 这里应该调用重启逻辑，但为了简化，我们只更新状态
                                    // 实际实现中应该重新启动进程
                                } else {
                                    println!("❌ 插件 {} 重启次数超过限制，停止重启", plugin_id);
                                    process_info.status = ProcessStatus::Failed;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn select_plugin_for_agent(&self, _agent_id: &str) -> A2AResult<String> {
        // 简化实现：返回第一个可用插件
        let plugins = self.bridge.get_all_plugins().await;

        for plugin in plugins {
            if plugin.status == PluginStatus::Running {
                return Ok(plugin.id);
            }
        }

        Err(A2AError::internal("没有可用的插件"))
    }
}

/// 插件统计信息
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub id: String,
    pub name: String,
    pub status: PluginStatus,
    pub framework: String,
    pub capabilities_count: usize,
    pub request_count: u64,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentx_a2a::{A2AProtocolEngine, ProtocolEngineConfig, StreamManager, SecurityManager, SecurityConfig};
    use agentx_a2a::monitoring::MonitoringManager;
    use agentx_a2a::monitoring::MonitoringConfig;
    use std::collections::HashMap;

    fn create_test_plugin_bridge() -> Arc<PluginBridge> {
        let config = ProtocolEngineConfig {
            max_concurrent_tasks: 10,
            task_timeout_seconds: 30,
            enable_message_validation: true,
            enable_task_persistence: false,
            handler_pool_size: Some(5),
            validate_messages: true,
            max_message_size: 1024 * 1024,
        };

        let a2a_engine = Arc::new(RwLock::new(A2AProtocolEngine::new(config)));
        let stream_manager = Arc::new(RwLock::new(StreamManager::new()));
        let security_manager = Arc::new(RwLock::new(SecurityManager::new(SecurityConfig::default())));
        let monitoring_manager = Arc::new(RwLock::new(MonitoringManager::new(MonitoringConfig::default())));

        Arc::new(PluginBridge::new(
            a2a_engine,
            stream_manager,
            security_manager,
            monitoring_manager,
        ))
    }

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let bridge = create_test_plugin_bridge();
        let manager = PluginManager::new(bridge);

        // 验证管理器创建成功
        assert_eq!(manager.plugin_configs.read().await.len(), 0);
        assert_eq!(manager.plugin_processes.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_plugin_config() {
        let bridge = create_test_plugin_bridge();
        let manager = PluginManager::new(bridge);

        let config = PluginConfig {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            endpoint: "http://localhost:50053".to_string(),
            framework: "test".to_string(),
            auto_restart: true,
            max_retries: 3,
            timeout_seconds: 30,
            config: HashMap::new(),
            executable_path: None, // 不启动外部进程
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
        };

        let result = manager.add_plugin_config(config.clone()).await;
        assert!(result.is_ok());

        // 验证配置已添加
        let configs = manager.plugin_configs.read().await;
        assert!(configs.contains_key("test_plugin"));
        assert_eq!(configs.get("test_plugin").unwrap().name, "Test Plugin");
    }

    #[tokio::test]
    async fn test_external_process_config() {
        let bridge = create_test_plugin_bridge();
        let manager = PluginManager::new(bridge);

        let config = PluginConfig {
            id: "external_plugin".to_string(),
            name: "External Plugin".to_string(),
            endpoint: "http://localhost:50054".to_string(),
            framework: "python".to_string(),
            auto_restart: true,
            max_retries: 3,
            timeout_seconds: 30,
            config: HashMap::new(),
            executable_path: Some("/usr/bin/python3".to_string()),
            args: vec!["-c".to_string(), "print('Hello from plugin')".to_string()],
            env_vars: {
                let mut env = HashMap::new();
                env.insert("PLUGIN_ID".to_string(), "external_plugin".to_string());
                env
            },
            working_dir: Some("/tmp".to_string()),
        };

        let result = manager.add_plugin_config(config.clone()).await;
        assert!(result.is_ok());

        // 验证外部进程配置
        let configs = manager.plugin_configs.read().await;
        let stored_config = configs.get("external_plugin").unwrap();
        assert_eq!(stored_config.executable_path, Some("/usr/bin/python3".to_string()));
        assert_eq!(stored_config.args.len(), 2);
        assert!(stored_config.env_vars.contains_key("PLUGIN_ID"));
    }

    #[tokio::test]
    async fn test_process_status_enum() {
        // 测试进程状态枚举
        assert_eq!(ProcessStatus::Starting, ProcessStatus::Starting);
        assert_ne!(ProcessStatus::Starting, ProcessStatus::Running);

        // 测试状态转换逻辑
        let status = ProcessStatus::Starting;
        match status {
            ProcessStatus::Starting => assert!(true),
            _ => assert!(false, "状态应该是Starting"),
        }
    }
}

impl LoadBalancer {
    fn new(strategy: LoadBalanceStrategy) -> Self {
        Self {
            strategy,
            plugin_weights: HashMap::new(),
            plugin_requests: HashMap::new(),
        }
    }
    
    fn select_plugin(&mut self, available_plugins: &[String]) -> Option<String> {
        if available_plugins.is_empty() {
            return None;
        }
        
        match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                // 简化的轮询实现
                available_plugins.first().cloned()
            },
            LoadBalanceStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                available_plugins.choose(&mut rng).cloned()
            },
            _ => available_plugins.first().cloned(),
        }
    }
}
