//! 插件系统集成测试
//! 
//! 测试插件生命周期管理、安全控制、配置管理等功能

use agentx_sdk::*;
use agentx_sdk::plugin::{PluginStats, PluginEventHandler};
use agentx_a2a::{A2AMessage, A2AResult, TrustLevel};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;

/// 测试插件实现
struct TestPlugin {
    info: PluginInfo,
    status: PluginStatus,
    message_count: Arc<Mutex<u32>>,
}

impl TestPlugin {
    fn new(plugin_id: String, name: String) -> Self {
        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let config = PluginConfig {
            framework: "test".to_string(),
            framework_version: Some("1.0.0".to_string()),
            bind_address: "127.0.0.1:0".to_string(),
            server_address: "127.0.0.1:8080".to_string(),
            max_connections: 10,
            request_timeout: 30,
            enable_tls: false,
            custom: HashMap::new(),
        };

        let stats = PluginStats {
            messages_processed: 0,
            messages_sent: 0,
            messages_received: 0,
            errors: 0,
            started_at: None,
            uptime_seconds: 0,
            avg_response_time_ms: 0.0,
        };

        let info = PluginInfo {
            metadata,
            status: PluginStatus::Registered,
            capabilities: vec![PluginCapability::TextProcessing],
            config,
            stats,
        };

        Self {
            info,
            status: PluginStatus::Registered,
            message_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl PluginLifecycle for TestPlugin {
    async fn initialize(&mut self) -> A2AResult<()> {
        println!("初始化插件: {}", self.info.metadata.id);
        Ok(())
    }

    async fn start(&mut self) -> A2AResult<()> {
        println!("启动插件: {}", self.info.metadata.id);
        self.status = PluginStatus::Running;
        Ok(())
    }

    async fn stop(&mut self) -> A2AResult<()> {
        println!("停止插件: {}", self.info.metadata.id);
        self.status = PluginStatus::Stopped;
        Ok(())
    }

    async fn pause(&mut self) -> A2AResult<()> {
        println!("暂停插件: {}", self.info.metadata.id);
        self.status = PluginStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> A2AResult<()> {
        println!("恢复插件: {}", self.info.metadata.id);
        self.status = PluginStatus::Running;
        Ok(())
    }

    async fn health_check(&self) -> A2AResult<bool> {
        Ok(self.status == PluginStatus::Running)
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn get_info(&self) -> &PluginInfo {
        &self.info
    }

    fn get_status(&self) -> PluginStatus {
        self.status.clone()
    }

    async fn process_message(&mut self, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        let mut count = self.message_count.lock().await;
        *count += 1;
        
        println!("插件 {} 处理消息: {} (总计: {})", 
                 self.info.metadata.id, message.message_id, *count);
        
        // 创建响应消息
        let response = A2AMessage::agent_message(format!(
            "来自插件 {} 的响应: 已处理消息 {}",
            self.info.metadata.id,
            message.message_id
        ));
        
        Ok(Some(response))
    }

    async fn send_message(&mut self, _message: A2AMessage) -> A2AResult<()> {
        Ok(())
    }

    async fn register_event_handler(&mut self, _handler: Box<dyn PluginEventHandler>) -> A2AResult<()> {
        Ok(())
    }

    fn get_capabilities(&self) -> &[PluginCapability] {
        &self.info.capabilities
    }

    async fn update_config(&mut self, config: PluginConfig) -> A2AResult<()> {
        self.info.config = config;
        Ok(())
    }

    fn get_stats(&self) -> &PluginStats {
        &self.info.stats
    }
}

/// 插件事件处理器
struct TestEventHandler;

#[async_trait]
impl PluginEventHandler for TestEventHandler {
    async fn handle_event(&mut self, event: PluginEvent) -> A2AResult<()> {
        println!("处理插件事件: {:?}", event);
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_lifecycle_management() {
    println!("🚀 测试插件生命周期管理");

    let config = LifecycleConfig {
        health_check_interval_secs: 1,
        startup_timeout_secs: 5,
        shutdown_timeout_secs: 5,
        max_restart_attempts: 3,
        restart_delay_secs: 1,
        enable_auto_restart: true,
        enable_health_check: true,
    };

    let lifecycle_manager = PluginLifecycleManager::new(config);

    // 创建测试插件
    let plugin = TestPlugin::new("test_plugin_1".to_string(), "Test Plugin 1".to_string());
    let plugin_id = plugin.get_info().metadata.id.clone();

    // 注册插件
    lifecycle_manager.register_plugin(Box::new(plugin)).await.unwrap();

    // 检查插件状态
    let state = lifecycle_manager.get_plugin_state(&plugin_id).await;
    assert!(state.is_some());
    assert_eq!(state.unwrap().status, PluginStatus::Registered);

    // 启动插件
    lifecycle_manager.start_plugin(&plugin_id).await.unwrap();
    
    // 检查启动后状态
    let state = lifecycle_manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state.status, PluginStatus::Running);
    assert!(state.started_at.is_some());

    // 处理消息
    let test_message = A2AMessage::user_message("测试消息".to_string());
    let response = lifecycle_manager.process_plugin_message(&plugin_id, test_message).await.unwrap();
    assert!(response.is_some());

    // 停止插件
    lifecycle_manager.stop_plugin(&plugin_id).await.unwrap();
    
    // 检查停止后状态
    let state = lifecycle_manager.get_plugin_state(&plugin_id).await.unwrap();
    assert_eq!(state.status, PluginStatus::Stopped);
    assert!(state.stopped_at.is_some());

    println!("✅ 插件生命周期管理测试通过");
}

#[tokio::test]
async fn test_plugin_security_management() {
    println!("🚀 测试插件安全管理");

    let security_config = PluginSecurityConfig {
        enable_permission_check: true,
        enable_resource_limits: true,
        enable_access_control: true,
        enable_security_audit: true,
        max_audit_entries: 1000,
        default_trust_level: TrustLevel::Public,
    };

    let security_manager = PluginSecurityManager::new(security_config);

    // 创建权限策略
    let mut allowed_operations = HashSet::new();
    allowed_operations.insert(Operation::ReadMessage);
    allowed_operations.insert(Operation::SendMessage);

    let permission_policy = PermissionPolicy {
        plugin_id: "test_plugin_security".to_string(),
        allowed_operations,
        denied_operations: HashSet::new(),
        accessible_resources: HashSet::new(),
        trust_level: TrustLevel::Public,
        created_at: std::time::SystemTime::now(),
        expires_at: None,
    };

    // 设置权限策略
    security_manager.set_permission_policy(permission_policy).await.unwrap();

    // 测试权限检查
    let can_read = security_manager.check_permission(
        "test_plugin_security",
        &Operation::ReadMessage,
        None,
    ).await.unwrap();
    assert!(can_read);

    let can_delete = security_manager.check_permission(
        "test_plugin_security",
        &Operation::DeleteAgent,
        None,
    ).await.unwrap();
    assert!(!can_delete); // 应该被拒绝

    // 创建资源限制
    let resource_limits = ResourceLimits {
        plugin_id: "test_plugin_security".to_string(),
        max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
        max_cpu_usage: Some(0.5), // 50%
        max_network_bandwidth: Some(1024 * 1024), // 1MB/s
        max_file_descriptors: Some(100),
        max_concurrent_connections: Some(10),
        rate_limit_per_second: Some(100),
        max_runtime_seconds: Some(3600), // 1小时
    };

    // 设置资源限制
    security_manager.set_resource_limits(resource_limits).await.unwrap();

    // 获取资源限制
    let limits = security_manager.get_resource_limits("test_plugin_security").await;
    assert!(limits.is_some());
    assert_eq!(limits.unwrap().max_memory_bytes, Some(100 * 1024 * 1024));

    // 创建访问控制列表
    let mut allowed_plugins = HashSet::new();
    allowed_plugins.insert("trusted_plugin".to_string());

    let acl = AccessControlList {
        plugin_id: "test_plugin_security".to_string(),
        allowed_plugins,
        denied_plugins: HashSet::new(),
        allowed_agents: HashSet::new(),
        denied_agents: HashSet::new(),
        allowed_ip_ranges: vec!["127.0.0.1".to_string()],
        denied_ip_ranges: vec![],
    };

    // 设置访问控制
    security_manager.set_access_control(acl).await.unwrap();

    // 测试访问控制
    let can_access_trusted = security_manager.check_access_control(
        "test_plugin_security",
        Some("trusted_plugin"),
        None,
        Some("127.0.0.1"),
    ).await.unwrap();
    assert!(can_access_trusted);

    let can_access_untrusted = security_manager.check_access_control(
        "test_plugin_security",
        Some("untrusted_plugin"),
        None,
        Some("127.0.0.1"),
    ).await.unwrap();
    assert!(!can_access_untrusted);

    // 检查审计日志
    let audit_log = security_manager.get_audit_log(Some("test_plugin_security"), Some(10)).await;
    assert!(!audit_log.is_empty());

    println!("✅ 插件安全管理测试通过");
}

#[tokio::test]
async fn test_plugin_config_management() {
    println!("🚀 测试插件配置管理");

    let temp_dir = tempfile::tempdir().unwrap();
    let config_manager_config = ConfigManagerConfig {
        config_directory: temp_dir.path().to_path_buf(),
        enable_validation: true,
        enable_hot_reload: false,
        config_format: ConfigFormat::Json,
        backup_count: 3,
        auto_save_interval_secs: 0, // 禁用自动保存
    };

    let config_manager = PluginConfigManager::new(config_manager_config).await.unwrap();

    // 创建默认配置
    let config_entry = PluginConfigManager::create_default_config(
        "test_plugin_config".to_string(),
        "Test Plugin Config".to_string(),
        "test_framework".to_string(),
    );

    // 保存配置
    config_manager.save_plugin_config("test_plugin_config", config_entry.clone()).await.unwrap();

    // 加载配置
    let loaded_config = config_manager.load_plugin_config("test_plugin_config").await.unwrap();
    assert!(loaded_config.is_some());
    let loaded_config = loaded_config.unwrap();
    assert_eq!(loaded_config.metadata.id, "test_plugin_config");
    assert_eq!(loaded_config.config.framework, "test_framework");

    // 获取配置
    let cached_config = config_manager.get_plugin_config("test_plugin_config").await;
    assert!(cached_config.is_some());

    // 更新配置
    config_manager.update_plugin_config("test_plugin_config", |config| {
        config.config.max_connections = 200;
        config.metadata.description = "Updated description".to_string();
    }).await.unwrap();

    // 验证更新
    let updated_config = config_manager.get_plugin_config("test_plugin_config").await.unwrap();
    assert_eq!(updated_config.config.max_connections, 200);
    assert_eq!(updated_config.metadata.description, "Updated description");
    assert_eq!(updated_config.config_version, 2); // 版本应该增加

    // 禁用插件
    config_manager.disable_plugin("test_plugin_config").await.unwrap();
    let disabled_config = config_manager.get_plugin_config("test_plugin_config").await.unwrap();
    assert!(!disabled_config.enabled);

    // 启用插件
    config_manager.enable_plugin("test_plugin_config").await.unwrap();
    let enabled_config = config_manager.get_plugin_config("test_plugin_config").await.unwrap();
    assert!(enabled_config.enabled);

    // 获取所有配置
    let all_configs = config_manager.get_all_plugin_configs().await;
    assert_eq!(all_configs.len(), 1);
    assert!(all_configs.contains_key("test_plugin_config"));

    // 删除配置
    config_manager.delete_plugin_config("test_plugin_config").await.unwrap();
    let deleted_config = config_manager.get_plugin_config("test_plugin_config").await;
    assert!(deleted_config.is_none());

    println!("✅ 插件配置管理测试通过");
}

#[tokio::test]
async fn test_integrated_plugin_system() {
    println!("🚀 测试集成插件系统");

    // 创建各个管理器
    let lifecycle_manager = PluginLifecycleManager::new(LifecycleConfig::default());
    let security_manager = PluginSecurityManager::new(PluginSecurityConfig::default());
    
    let temp_dir = tempfile::tempdir().unwrap();
    let config_manager_config = ConfigManagerConfig {
        config_directory: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let config_manager = PluginConfigManager::new(config_manager_config).await.unwrap();

    // 创建插件配置
    let config_entry = PluginConfigManager::create_default_config(
        "integrated_test_plugin".to_string(),
        "Integrated Test Plugin".to_string(),
        "test_framework".to_string(),
    );

    // 保存配置
    config_manager.save_plugin_config("integrated_test_plugin", config_entry).await.unwrap();

    // 创建权限策略
    let permission_policy = create_default_permission_policy(
        "integrated_test_plugin".to_string(),
        TrustLevel::Trusted,
    );
    security_manager.set_permission_policy(permission_policy).await.unwrap();

    // 创建并注册插件
    let plugin = TestPlugin::new(
        "integrated_test_plugin".to_string(),
        "Integrated Test Plugin".to_string(),
    );
    lifecycle_manager.register_plugin(Box::new(plugin)).await.unwrap();

    // 启动插件
    lifecycle_manager.start_plugin("integrated_test_plugin").await.unwrap();

    // 检查权限
    let can_create_agent = security_manager.check_permission(
        "integrated_test_plugin",
        &Operation::CreateAgent,
        None,
    ).await.unwrap();
    assert!(can_create_agent); // Trusted级别应该允许

    // 处理消息
    let test_message = A2AMessage::user_message("集成测试消息".to_string());
    let response = lifecycle_manager.process_plugin_message(
        "integrated_test_plugin",
        test_message,
    ).await.unwrap();
    assert!(response.is_some());

    // 获取插件状态
    let state = lifecycle_manager.get_plugin_state("integrated_test_plugin").await.unwrap();
    assert_eq!(state.status, PluginStatus::Running);
    assert_eq!(state.health_status, HealthStatus::Unknown); // 还没有健康检查

    // 停止插件
    lifecycle_manager.stop_plugin("integrated_test_plugin").await.unwrap();

    println!("✅ 集成插件系统测试通过");
}
