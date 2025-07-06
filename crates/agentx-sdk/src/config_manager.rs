//! 插件配置管理器
//! 
//! 提供插件配置的加载、验证、热更新和持久化功能

use crate::plugin::{PluginConfig, PluginMetadata};
use crate::security::{PermissionPolicy, ResourceLimits, AccessControlList};
use agentx_a2a::{A2AResult, A2AError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use tracing::{debug, info, warn};
use chrono::{DateTime, Utc};

/// 插件配置管理器
pub struct PluginConfigManager {
    /// 配置存储
    configs: Arc<RwLock<HashMap<String, PluginConfigEntry>>>,
    /// 配置文件路径
    config_dir: PathBuf,
    /// 配置验证器
    validator: ConfigValidator,
    /// 配置
    manager_config: ConfigManagerConfig,
}

/// 插件配置条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigEntry {
    /// 插件元数据
    pub metadata: PluginMetadata,
    /// 插件配置
    pub config: PluginConfig,
    /// 权限策略
    pub permission_policy: Option<PermissionPolicy>,
    /// 资源限制
    pub resource_limits: Option<ResourceLimits>,
    /// 访问控制列表
    pub access_control: Option<AccessControlList>,
    /// 配置版本
    pub config_version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 是否启用
    pub enabled: bool,
    /// 自动启动
    pub auto_start: bool,
}

/// 配置验证器
pub struct ConfigValidator {
    /// 必需字段
    #[allow(dead_code)]
    required_fields: Vec<String>,
    /// 字段验证规则
    validation_rules: HashMap<String, ValidationRule>,
}

/// 验证规则
#[derive(Debug, Clone)]
pub enum ValidationRule {
    /// 字符串长度范围
    StringLength { min: usize, max: usize },
    /// 数值范围
    NumberRange { min: f64, max: f64 },
    /// 正则表达式
    Regex(String),
    /// 枚举值
    Enum(Vec<String>),
    /// 自定义验证函数
    Custom(fn(&serde_json::Value) -> bool),
}

/// 配置管理器配置
#[derive(Debug, Clone)]
pub struct ConfigManagerConfig {
    /// 配置文件目录
    pub config_directory: PathBuf,
    /// 是否启用配置验证
    pub enable_validation: bool,
    /// 是否启用热更新
    pub enable_hot_reload: bool,
    /// 配置文件格式
    pub config_format: ConfigFormat,
    /// 备份配置数量
    pub backup_count: u32,
    /// 自动保存间隔（秒）
    pub auto_save_interval_secs: u64,
}

/// 配置文件格式
#[derive(Debug, Clone)]
pub enum ConfigFormat {
    /// JSON格式
    Json,
    /// YAML格式
    Yaml,
    /// TOML格式
    Toml,
}

impl Default for ConfigManagerConfig {
    fn default() -> Self {
        Self {
            config_directory: PathBuf::from("./configs"),
            enable_validation: true,
            enable_hot_reload: true,
            config_format: ConfigFormat::Json,
            backup_count: 5,
            auto_save_interval_secs: 300, // 5分钟
        }
    }
}

impl PluginConfigManager {
    /// 创建新的配置管理器
    pub async fn new(config: ConfigManagerConfig) -> A2AResult<Self> {
        // 确保配置目录存在
        if !config.config_directory.exists() {
            fs::create_dir_all(&config.config_directory).await
                .map_err(|e| A2AError::internal(format!("创建配置目录失败: {}", e)))?;
        }

        let manager = Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            config_dir: config.config_directory.clone(),
            validator: ConfigValidator::new(),
            manager_config: config,
        };

        // 加载现有配置
        manager.load_all_configs().await?;

        // 启动自动保存任务
        if manager.manager_config.auto_save_interval_secs > 0 {
            manager.start_auto_save_task().await;
        }

        Ok(manager)
    }

    /// 加载插件配置
    pub async fn load_plugin_config(&self, plugin_id: &str) -> A2AResult<Option<PluginConfigEntry>> {
        let config_file = self.get_config_file_path(plugin_id);
        
        if !config_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_file).await
            .map_err(|e| A2AError::internal(format!("读取配置文件失败: {}", e)))?;

        let config_entry: PluginConfigEntry = match self.manager_config.config_format {
            ConfigFormat::Json => serde_json::from_str(&content)
                .map_err(|e| A2AError::internal(format!("解析JSON配置失败: {}", e)))?,
            ConfigFormat::Yaml => serde_yaml::from_str(&content)
                .map_err(|e| A2AError::internal(format!("解析YAML配置失败: {}", e)))?,
            ConfigFormat::Toml => toml::from_str(&content)
                .map_err(|e| A2AError::internal(format!("解析TOML配置失败: {}", e)))?,
        };

        // 验证配置
        if self.manager_config.enable_validation {
            self.validator.validate_config(&config_entry)?;
        }

        // 缓存配置
        self.configs.write().await.insert(plugin_id.to_string(), config_entry.clone());

        info!("加载插件配置: {}", plugin_id);
        Ok(Some(config_entry))
    }

    /// 保存插件配置
    pub async fn save_plugin_config(&self, plugin_id: &str, config_entry: PluginConfigEntry) -> A2AResult<()> {
        // 验证配置
        if self.manager_config.enable_validation {
            self.validator.validate_config(&config_entry)?;
        }

        // 备份现有配置
        self.backup_config(plugin_id).await?;

        // 序列化配置
        let content = match self.manager_config.config_format {
            ConfigFormat::Json => serde_json::to_string_pretty(&config_entry)
                .map_err(|e| A2AError::internal(format!("序列化JSON配置失败: {}", e)))?,
            ConfigFormat::Yaml => serde_yaml::to_string(&config_entry)
                .map_err(|e| A2AError::internal(format!("序列化YAML配置失败: {}", e)))?,
            ConfigFormat::Toml => toml::to_string_pretty(&config_entry)
                .map_err(|e| A2AError::internal(format!("序列化TOML配置失败: {}", e)))?,
        };

        // 写入配置文件
        let config_file = self.get_config_file_path(plugin_id);
        fs::write(&config_file, content).await
            .map_err(|e| A2AError::internal(format!("写入配置文件失败: {}", e)))?;

        // 更新缓存
        self.configs.write().await.insert(plugin_id.to_string(), config_entry);

        info!("保存插件配置: {}", plugin_id);
        Ok(())
    }

    /// 删除插件配置
    pub async fn delete_plugin_config(&self, plugin_id: &str) -> A2AResult<()> {
        let config_file = self.get_config_file_path(plugin_id);
        
        if config_file.exists() {
            // 备份配置
            self.backup_config(plugin_id).await?;
            
            // 删除配置文件
            fs::remove_file(&config_file).await
                .map_err(|e| A2AError::internal(format!("删除配置文件失败: {}", e)))?;
        }

        // 从缓存中移除
        self.configs.write().await.remove(plugin_id);

        info!("删除插件配置: {}", plugin_id);
        Ok(())
    }

    /// 获取插件配置
    pub async fn get_plugin_config(&self, plugin_id: &str) -> Option<PluginConfigEntry> {
        self.configs.read().await.get(plugin_id).cloned()
    }

    /// 获取所有插件配置
    pub async fn get_all_plugin_configs(&self) -> HashMap<String, PluginConfigEntry> {
        self.configs.read().await.clone()
    }

    /// 更新插件配置
    pub async fn update_plugin_config(
        &self,
        plugin_id: &str,
        updater: impl FnOnce(&mut PluginConfigEntry),
    ) -> A2AResult<()> {
        let mut configs = self.configs.write().await;
        
        if let Some(config_entry) = configs.get_mut(plugin_id) {
            updater(config_entry);
            config_entry.config_version += 1;
            config_entry.last_updated = Utc::now();
            
            // 保存到文件
            let config_entry_clone = config_entry.clone();
            drop(configs);
            
            self.save_plugin_config(plugin_id, config_entry_clone).await?;
        } else {
            return Err(A2AError::internal(format!("插件配置未找到: {}", plugin_id)));
        }

        Ok(())
    }

    /// 启用插件
    pub async fn enable_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        self.update_plugin_config(plugin_id, |config| {
            config.enabled = true;
        }).await
    }

    /// 禁用插件
    pub async fn disable_plugin(&self, plugin_id: &str) -> A2AResult<()> {
        self.update_plugin_config(plugin_id, |config| {
            config.enabled = false;
        }).await
    }

    /// 创建默认配置
    pub fn create_default_config(
        plugin_id: String,
        name: String,
        framework: String,
    ) -> PluginConfigEntry {
        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            name: name.clone(),
            version: "1.0.0".to_string(),
            description: format!("Default configuration for {}", name),
            author: "AgentX".to_string(),
            license: "Apache-2.0".to_string(),
            homepage: None,
            repository: None,
            tags: vec![framework.clone()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let config = PluginConfig {
            framework,
            framework_version: None,
            bind_address: "127.0.0.1:0".to_string(),
            server_address: "127.0.0.1:8080".to_string(),
            max_connections: 100,
            request_timeout: 30,
            enable_tls: false,
            custom: HashMap::new(),
        };

        PluginConfigEntry {
            metadata,
            config,
            permission_policy: None,
            resource_limits: None,
            access_control: None,
            config_version: 1,
            last_updated: Utc::now(),
            enabled: true,
            auto_start: false,
        }
    }

    // 私有方法

    async fn load_all_configs(&self) -> A2AResult<()> {
        let mut dir = fs::read_dir(&self.config_dir).await
            .map_err(|e| A2AError::internal(format!("读取配置目录失败: {}", e)))?;

        while let Some(entry) = dir.next_entry().await
            .map_err(|e| A2AError::internal(format!("读取目录条目失败: {}", e)))? {
            
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_stem() {
                    if let Some(plugin_id) = file_name.to_str() {
                        if let Err(e) = self.load_plugin_config(plugin_id).await {
                            warn!("加载插件配置失败 {}: {}", plugin_id, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_config_file_path(&self, plugin_id: &str) -> PathBuf {
        let extension = match self.manager_config.config_format {
            ConfigFormat::Json => "json",
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Toml => "toml",
        };
        
        self.config_dir.join(format!("{}.{}", plugin_id, extension))
    }

    async fn backup_config(&self, plugin_id: &str) -> A2AResult<()> {
        let config_file = self.get_config_file_path(plugin_id);
        
        if !config_file.exists() {
            return Ok(());
        }

        let backup_dir = self.config_dir.join("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).await
                .map_err(|e| A2AError::internal(format!("创建备份目录失败: {}", e)))?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = backup_dir.join(format!("{}_{}.backup", plugin_id, timestamp));

        fs::copy(&config_file, &backup_file).await
            .map_err(|e| A2AError::internal(format!("备份配置文件失败: {}", e)))?;

        // 清理旧备份
        self.cleanup_old_backups(plugin_id).await?;

        Ok(())
    }

    async fn cleanup_old_backups(&self, plugin_id: &str) -> A2AResult<()> {
        let backup_dir = self.config_dir.join("backups");
        
        if !backup_dir.exists() {
            return Ok(());
        }

        let mut backups = Vec::new();
        let mut dir = fs::read_dir(&backup_dir).await
            .map_err(|e| A2AError::internal(format!("读取备份目录失败: {}", e)))?;

        while let Some(entry) = dir.next_entry().await
            .map_err(|e| A2AError::internal(format!("读取备份条目失败: {}", e)))? {
            
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(&format!("{}_", plugin_id)) {
                    if let Ok(metadata) = entry.metadata().await {
                        if let Ok(modified) = metadata.modified() {
                            backups.push((path, modified));
                        }
                    }
                }
            }
        }

        // 按修改时间排序，保留最新的几个
        backups.sort_by_key(|(_, modified)| *modified);
        
        if backups.len() > self.manager_config.backup_count as usize {
            let to_remove = backups.len() - self.manager_config.backup_count as usize;
            for (path, _) in backups.into_iter().take(to_remove) {
                if let Err(e) = fs::remove_file(&path).await {
                    warn!("删除旧备份失败 {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    async fn start_auto_save_task(&self) {
        let _configs = self.configs.clone();
        let _config_dir = self.config_dir.clone();
        let interval_secs = self.manager_config.auto_save_interval_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval_secs)
            );

            loop {
                interval.tick().await;
                
                // 这里可以实现自动保存逻辑
                // 例如检查配置是否有变更，如果有则保存
                debug!("自动保存任务执行");
            }
        });
    }
}

impl ConfigValidator {
    fn new() -> Self {
        let mut validator = Self {
            required_fields: vec![
                "metadata.id".to_string(),
                "metadata.name".to_string(),
                "config.framework".to_string(),
            ],
            validation_rules: HashMap::new(),
        };

        // 添加基本验证规则
        validator.validation_rules.insert(
            "metadata.id".to_string(),
            ValidationRule::StringLength { min: 1, max: 100 },
        );
        validator.validation_rules.insert(
            "metadata.name".to_string(),
            ValidationRule::StringLength { min: 1, max: 200 },
        );

        validator
    }

    fn validate_config(&self, config: &PluginConfigEntry) -> A2AResult<()> {
        // 基本验证
        if config.metadata.id.is_empty() {
            return Err(A2AError::internal("插件ID不能为空"));
        }

        if config.metadata.name.is_empty() {
            return Err(A2AError::internal("插件名称不能为空"));
        }

        if config.config.framework.is_empty() {
            return Err(A2AError::internal("框架类型不能为空"));
        }

        // 更多验证规则可以在这里添加

        Ok(())
    }
}
