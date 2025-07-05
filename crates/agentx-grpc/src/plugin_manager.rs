//! gRPC插件管理器
//! 
//! 管理gRPC插件的生命周期、健康检查和负载均衡

use crate::plugin_bridge::{PluginBridge, PluginStatus};
use agentx_a2a::{A2AResult, A2AError, A2AMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};

/// 插件管理器
pub struct PluginManager {
    /// 插件桥接器
    bridge: Arc<PluginBridge>,
    /// 插件配置
    plugin_configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
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
