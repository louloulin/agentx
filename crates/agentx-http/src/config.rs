//! HTTP服务器配置
//! 
//! 定义HTTP服务器的配置选项和环境变量

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// HTTP服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    /// 服务器监听地址
    pub host: String,
    
    /// 服务器监听端口
    pub port: u16,
    
    /// 是否启用CORS
    pub enable_cors: bool,
    
    /// 是否启用请求日志
    pub enable_request_logging: bool,
    
    /// 是否启用压缩
    pub enable_compression: bool,
    
    /// 请求超时时间（秒）
    pub request_timeout: u64,
    
    /// 最大请求体大小（字节）
    pub max_request_size: usize,
    
    /// API密钥（可选）
    pub api_key: Option<String>,
    
    /// JWT密钥（可选）
    pub jwt_secret: Option<String>,
    
    /// 是否启用OpenAPI文档
    pub enable_docs: bool,
    
    /// 文档路径
    pub docs_path: String,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_cors: true,
            enable_request_logging: true,
            enable_compression: true,
            request_timeout: 30,
            max_request_size: 10 * 1024 * 1024, // 10MB
            api_key: None,
            jwt_secret: None,
            enable_docs: true,
            docs_path: "/docs".to_string(),
        }
    }
}

impl HttpServerConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("AGENTX_HTTP"))
            .build()?;
        
        // 设置默认值
        cfg.set_default("host", "0.0.0.0")?;
        cfg.set_default("port", 8080)?;
        cfg.set_default("enable_cors", true)?;
        cfg.set_default("enable_request_logging", true)?;
        cfg.set_default("enable_compression", true)?;
        cfg.set_default("request_timeout", 30)?;
        cfg.set_default("max_request_size", 10 * 1024 * 1024)?;
        cfg.set_default("enable_docs", true)?;
        cfg.set_default("docs_path", "/docs")?;
        
        cfg.try_deserialize()
    }
    
    /// 获取监听地址
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid socket address")
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("端口号不能为0".to_string());
        }
        
        if self.request_timeout == 0 {
            return Err("请求超时时间不能为0".to_string());
        }
        
        if self.max_request_size == 0 {
            return Err("最大请求大小不能为0".to_string());
        }
        
        Ok(())
    }
}

/// A2A协议引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AEngineConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    
    /// 任务超时时间（秒）
    pub task_timeout_seconds: u64,
    
    /// 是否启用消息验证
    pub enable_message_validation: bool,
    
    /// 是否启用任务持久化
    pub enable_task_persistence: bool,
}

impl Default for A2AEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            task_timeout_seconds: 300,
            enable_message_validation: true,
            enable_task_persistence: false,
        }
    }
}

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// HTTP服务器配置
    pub http: HttpServerConfig,
    
    /// A2A协议引擎配置
    pub a2a: A2AEngineConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: HttpServerConfig::default(),
            a2a: A2AEngineConfig::default(),
        }
    }
}

impl AppConfig {
    /// 从环境变量和配置文件加载配置
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();
        
        // 尝试加载配置文件
        if let Ok(config_file) = std::env::var("AGENTX_CONFIG_FILE") {
            cfg = cfg.add_source(config::File::with_name(&config_file));
        }
        
        // 加载环境变量
        cfg = cfg.add_source(config::Environment::with_prefix("AGENTX"));
        
        let config = cfg.build()?;
        config.try_deserialize()
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        self.http.validate()?;
        Ok(())
    }
}
