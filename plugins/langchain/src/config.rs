//! LangChain插件配置
//! 
//! 管理LangChain插件的配置参数

use crate::error::{LangChainError, LangChainResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, debug};

/// LangChain插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangChainConfig {
    /// 插件服务器配置
    pub host: String,
    pub port: u16,
    
    /// Python环境配置
    pub python_executable: String,
    pub python_service_host: String,
    pub python_service_port: u16,
    pub python_path: Vec<String>,
    pub environment_variables: HashMap<String, String>,
    
    /// LangChain配置
    pub default_model: String,
    pub api_keys: HashMap<String, String>,
    pub model_configs: HashMap<String, ModelConfig>,
    pub tool_configs: HashMap<String, ToolConfig>,
    
    /// 性能配置
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub memory_limit_mb: usize,
    pub cache_size: usize,
    
    /// 日志配置
    pub log_level: String,
    pub log_file: Option<PathBuf>,
    pub enable_metrics: bool,
    
    /// 安全配置
    pub enable_authentication: bool,
    pub api_key: Option<String>,
    pub allowed_origins: Vec<String>,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub tool_type: String,
    pub enabled: bool,
    pub config: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
}

impl Default for LangChainConfig {
    fn default() -> Self {
        let mut api_keys = HashMap::new();
        let mut environment_variables = HashMap::new();
        let mut model_configs = HashMap::new();
        let mut tool_configs = HashMap::new();
        
        // 默认环境变量
        environment_variables.insert("PYTHONPATH".to_string(), ".".to_string());
        environment_variables.insert("LANGCHAIN_TRACING_V2".to_string(), "false".to_string());
        
        // 默认模型配置
        model_configs.insert("gpt-3.5-turbo".to_string(), ModelConfig {
            provider: "openai".to_string(),
            model_name: "gpt-3.5-turbo".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            parameters: HashMap::new(),
        });
        
        model_configs.insert("gpt-4".to_string(), ModelConfig {
            provider: "openai".to_string(),
            model_name: "gpt-4".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            base_url: None,
            max_tokens: Some(8192),
            temperature: Some(0.7),
            parameters: HashMap::new(),
        });
        
        model_configs.insert("claude-3-sonnet".to_string(), ModelConfig {
            provider: "anthropic".to_string(),
            model_name: "claude-3-sonnet-20240229".to_string(),
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            parameters: HashMap::new(),
        });
        
        // 默认工具配置
        tool_configs.insert("search".to_string(), ToolConfig {
            tool_type: "web_search".to_string(),
            enabled: true,
            config: HashMap::new(),
            dependencies: vec!["requests".to_string(), "beautifulsoup4".to_string()],
        });
        
        tool_configs.insert("calculator".to_string(), ToolConfig {
            tool_type: "math".to_string(),
            enabled: true,
            config: HashMap::new(),
            dependencies: vec!["sympy".to_string()],
        });
        
        Self {
            host: "0.0.0.0".to_string(),
            port: 50052,
            python_executable: "python3".to_string(),
            python_service_host: "127.0.0.1".to_string(),
            python_service_port: 8000,
            python_path: vec![".".to_string()],
            environment_variables,
            default_model: "gpt-3.5-turbo".to_string(),
            api_keys,
            model_configs,
            tool_configs,
            max_concurrent_requests: 100,
            request_timeout_seconds: 300,
            memory_limit_mb: 1024,
            cache_size: 1000,
            log_level: "info".to_string(),
            log_file: None,
            enable_metrics: true,
            enable_authentication: false,
            api_key: None,
            allowed_origins: vec!["*".to_string()],
        }
    }
}

impl LangChainConfig {
    /// 从文件加载配置
    pub async fn load() -> LangChainResult<Self> {
        info!("📋 加载LangChain插件配置...");
        
        // 尝试从多个位置加载配置文件
        let config_paths = vec![
            "langchain_config.toml",
            "config/langchain.toml",
            "/etc/agentx/langchain.toml",
            "~/.agentx/langchain.toml",
        ];
        
        for path in config_paths {
            if let Ok(config) = Self::load_from_file(path).await {
                info!("✅ 从 {} 加载配置成功", path);
                return Ok(config);
            }
        }
        
        // 如果没有找到配置文件，使用默认配置
        info!("📄 使用默认配置");
        let mut config = Self::default();
        
        // 从环境变量覆盖配置
        config.load_from_env();
        
        Ok(config)
    }
    
    /// 从文件加载配置
    async fn load_from_file(path: &str) -> LangChainResult<Self> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| LangChainError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: Self = toml::from_str(&content)
            .map_err(|e| LangChainError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        debug!("从文件加载配置: {}", path);
        Ok(config)
    }
    
    /// 从环境变量加载配置
    fn load_from_env(&mut self) {
        debug!("🌍 从环境变量加载配置...");
        
        // 服务器配置
        if let Ok(host) = std::env::var("LANGCHAIN_HOST") {
            self.host = host;
        }
        if let Ok(port) = std::env::var("LANGCHAIN_PORT") {
            if let Ok(port) = port.parse() {
                self.port = port;
            }
        }
        
        // Python配置
        if let Ok(python) = std::env::var("PYTHON_EXECUTABLE") {
            self.python_executable = python;
        }
        if let Ok(host) = std::env::var("PYTHON_SERVICE_HOST") {
            self.python_service_host = host;
        }
        if let Ok(port) = std::env::var("PYTHON_SERVICE_PORT") {
            if let Ok(port) = port.parse() {
                self.python_service_port = port;
            }
        }
        
        // API密钥
        if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
            self.api_keys.insert("openai".to_string(), openai_key);
        }
        if let Ok(anthropic_key) = std::env::var("ANTHROPIC_API_KEY") {
            self.api_keys.insert("anthropic".to_string(), anthropic_key);
        }
        
        // 默认模型
        if let Ok(model) = std::env::var("DEFAULT_MODEL") {
            self.default_model = model;
        }
        
        // 性能配置
        if let Ok(max_requests) = std::env::var("MAX_CONCURRENT_REQUESTS") {
            if let Ok(max_requests) = max_requests.parse() {
                self.max_concurrent_requests = max_requests;
            }
        }
        
        // 日志配置
        if let Ok(log_level) = std::env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
        
        debug!("✅ 环境变量配置加载完成");
    }
    
    /// 保存配置到文件
    pub async fn save_to_file(&self, path: &str) -> LangChainResult<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| LangChainError::ConfigError(format!("序列化配置失败: {}", e)))?;
        
        tokio::fs::write(path, content).await
            .map_err(|e| LangChainError::ConfigError(format!("写入配置文件失败: {}", e)))?;
        
        info!("💾 配置已保存到: {}", path);
        Ok(())
    }
    
    /// 验证配置
    pub fn validate(&self) -> LangChainResult<()> {
        debug!("🔍 验证配置...");
        
        // 验证端口范围
        if self.port == 0 || self.port > 65535 {
            return Err(LangChainError::ConfigError(
                format!("无效的端口号: {}", self.port)
            ));
        }
        
        if self.python_service_port == 0 || self.python_service_port > 65535 {
            return Err(LangChainError::ConfigError(
                format!("无效的Python服务端口号: {}", self.python_service_port)
            ));
        }
        
        // 验证Python可执行文件
        if self.python_executable.is_empty() {
            return Err(LangChainError::ConfigError(
                "Python可执行文件路径不能为空".to_string()
            ));
        }
        
        // 验证默认模型
        if !self.model_configs.contains_key(&self.default_model) {
            return Err(LangChainError::ConfigError(
                format!("默认模型 '{}' 未在模型配置中定义", self.default_model)
            ));
        }
        
        // 验证性能参数
        if self.max_concurrent_requests == 0 {
            return Err(LangChainError::ConfigError(
                "最大并发请求数必须大于0".to_string()
            ));
        }
        
        if self.request_timeout_seconds == 0 {
            return Err(LangChainError::ConfigError(
                "请求超时时间必须大于0".to_string()
            ));
        }
        
        debug!("✅ 配置验证通过");
        Ok(())
    }
    
    /// 设置参数
    pub fn set_parameter(&mut self, key: &str, value: &str) {
        match key {
            "host" => self.host = value.to_string(),
            "port" => {
                if let Ok(port) = value.parse() {
                    self.port = port;
                }
            }
            "python_executable" => self.python_executable = value.to_string(),
            "default_model" => self.default_model = value.to_string(),
            "log_level" => self.log_level = value.to_string(),
            _ => {
                // 添加到环境变量
                self.environment_variables.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    /// 获取运行时信息
    pub fn get_runtime_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("host".to_string(), self.host.clone());
        info.insert("port".to_string(), self.port.to_string());
        info.insert("python_executable".to_string(), self.python_executable.clone());
        info.insert("python_service_host".to_string(), self.python_service_host.clone());
        info.insert("python_service_port".to_string(), self.python_service_port.to_string());
        info.insert("default_model".to_string(), self.default_model.clone());
        info.insert("max_concurrent_requests".to_string(), self.max_concurrent_requests.to_string());
        info.insert("log_level".to_string(), self.log_level.clone());
        
        info
    }
    
    /// 获取模型配置
    pub fn get_model_config(&self, model_name: &str) -> Option<&ModelConfig> {
        self.model_configs.get(model_name)
    }
    
    /// 获取工具配置
    pub fn get_tool_config(&self, tool_name: &str) -> Option<&ToolConfig> {
        self.tool_configs.get(tool_name)
    }
    
    /// 获取API密钥
    pub fn get_api_key(&self, provider: &str) -> Option<&String> {
        self.api_keys.get(provider)
    }
}
