//! 插件核心模块
//! 
//! 定义插件的基础接口、生命周期管理和元数据结构

use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;

/// 插件状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// 已注册
    Registered,
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 失败状态
    Failed,
    /// 错误状态
    Error(String),
}

/// 插件能力枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginCapability {
    /// 文本处理
    TextProcessing,
    /// 图像处理
    ImageProcessing,
    /// 音频处理
    AudioProcessing,
    /// 视频处理
    VideoProcessing,
    /// 工具调用
    ToolCalling,
    /// 工作流执行
    WorkflowExecution,
    /// 多Agent对话
    MultiAgentConversation,
    /// 知识检索
    KnowledgeRetrieval,
    /// 代码生成
    CodeGeneration,
    /// 数据分析
    DataAnalysis,
    /// 自定义能力
    Custom(String),
}

/// 插件事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// 插件启动
    Started,
    /// 插件停止
    Stopped,
    /// 消息接收
    MessageReceived(A2AMessage),
    /// 消息发送
    MessageSent(A2AMessage),
    /// 错误发生
    Error(String),
    /// 状态变更
    StatusChanged(PluginStatus),
    /// 自定义事件
    Custom(String, serde_json::Value),
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件ID
    pub id: String,
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 插件作者
    pub author: String,
    /// 插件许可证
    pub license: String,
    /// 插件主页
    pub homepage: Option<String>,
    /// 插件仓库
    pub repository: Option<String>,
    /// 插件标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 元数据
    pub metadata: PluginMetadata,
    /// 当前状态
    pub status: PluginStatus,
    /// 支持的能力
    pub capabilities: Vec<PluginCapability>,
    /// 配置信息
    pub config: PluginConfig,
    /// 统计信息
    pub stats: PluginStats,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 框架类型
    pub framework: String,
    /// 框架版本
    pub framework_version: Option<String>,
    /// 绑定地址
    pub bind_address: String,
    /// 服务器地址
    pub server_address: String,
    /// 最大连接数
    pub max_connections: u32,
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    /// 是否启用TLS
    pub enable_tls: bool,
    /// 自定义配置
    pub custom: HashMap<String, serde_json::Value>,
}

/// 插件统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    /// 处理的消息数
    pub messages_processed: u64,
    /// 发送的消息数
    pub messages_sent: u64,
    /// 接收的消息数
    pub messages_received: u64,
    /// 错误数
    pub errors: u64,
    /// 启动时间
    pub started_at: Option<DateTime<Utc>>,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
}

/// 插件生命周期管理
#[async_trait]
pub trait PluginLifecycle {
    /// 初始化插件
    async fn initialize(&mut self) -> A2AResult<()>;
    
    /// 启动插件
    async fn start(&mut self) -> A2AResult<()>;
    
    /// 停止插件
    async fn stop(&mut self) -> A2AResult<()>;
    
    /// 暂停插件
    async fn pause(&mut self) -> A2AResult<()>;
    
    /// 恢复插件
    async fn resume(&mut self) -> A2AResult<()>;
    
    /// 重启插件
    async fn restart(&mut self) -> A2AResult<()> {
        self.stop().await?;
        self.start().await
    }
    
    /// 健康检查
    async fn health_check(&self) -> A2AResult<bool>;
}

/// 插件核心接口
#[async_trait]
pub trait Plugin: PluginLifecycle + Send + Sync {
    /// 获取插件信息
    fn get_info(&self) -> &PluginInfo;
    
    /// 获取插件状态
    fn get_status(&self) -> PluginStatus;
    
    /// 处理A2A消息
    async fn process_message(&mut self, message: A2AMessage) -> A2AResult<Option<A2AMessage>>;
    
    /// 发送A2A消息
    async fn send_message(&mut self, message: A2AMessage) -> A2AResult<()>;
    
    /// 注册事件处理器
    async fn register_event_handler(&mut self, handler: Box<dyn PluginEventHandler>) -> A2AResult<()>;
    
    /// 获取插件能力
    fn get_capabilities(&self) -> &[PluginCapability];
    
    /// 更新配置
    async fn update_config(&mut self, config: PluginConfig) -> A2AResult<()>;
    
    /// 获取统计信息
    fn get_stats(&self) -> &PluginStats;
}

/// 插件事件处理器
#[async_trait]
pub trait PluginEventHandler: Send + Sync {
    /// 处理插件事件
    async fn handle_event(&mut self, event: PluginEvent) -> A2AResult<()>;
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            framework: "custom".to_string(),
            framework_version: None,
            bind_address: "127.0.0.1:0".to_string(),
            server_address: "127.0.0.1:50051".to_string(),
            max_connections: 100,
            request_timeout: 30,
            enable_tls: false,
            custom: HashMap::new(),
        }
    }
}

impl PluginConfig {
    /// 从文件加载配置
    pub fn from_file(path: &str) -> A2AResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| A2AError::internal(format!("读取配置文件失败: {}", e)))?;
        
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| A2AError::internal(format!("解析配置文件失败: {}", e)))?;
        
        Ok(config)
    }
    
    /// 为特定框架创建默认配置
    pub fn default_for_framework(framework: &str) -> A2AResult<Self> {
        let mut config = Self::default();
        config.framework = framework.to_string();
        
        // 根据框架设置特定的默认值
        match framework {
            "langchain" => {
                config.framework_version = Some("0.1.0".to_string());
                config.custom.insert("python_path".to_string(), serde_json::Value::String("python".to_string()));
            },
            "autogen" => {
                config.framework_version = Some("0.2.0".to_string());
                config.custom.insert("python_path".to_string(), serde_json::Value::String("python".to_string()));
            },
            "mastra" => {
                config.framework_version = Some("0.1.0".to_string());
                config.custom.insert("node_path".to_string(), serde_json::Value::String("node".to_string()));
            },
            _ => {}
        }
        
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &str) -> A2AResult<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| A2AError::internal(format!("序列化配置失败: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| A2AError::internal(format!("写入配置文件失败: {}", e)))?;
        
        Ok(())
    }
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            messages_processed: 0,
            messages_sent: 0,
            messages_received: 0,
            errors: 0,
            started_at: None,
            uptime_seconds: 0,
            avg_response_time_ms: 0.0,
        }
    }
}

impl PluginMetadata {
    /// 创建新的插件元数据
    pub fn new(name: &str, version: &str, description: &str, author: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            license: "Apache-2.0".to_string(),
            homepage: None,
            repository: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert_eq!(config.framework, "custom");
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_plugin_metadata_new() {
        let metadata = PluginMetadata::new("test", "1.0.0", "Test plugin", "Test Author");
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0.0");
        assert!(!metadata.id.is_empty());
    }

    #[test]
    fn test_plugin_status() {
        let status = PluginStatus::Running;
        assert_eq!(status, PluginStatus::Running);

        let error_status = PluginStatus::Error("Test error".to_string());
        match error_status {
            PluginStatus::Error(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected error status"),
        }
    }
}
