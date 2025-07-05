//! 工具模块
//! 
//! 提供各种实用工具和辅助函数

use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use crate::plugin::{PluginConfig, PluginInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件工具
pub struct PluginUtils;

impl PluginUtils {
    /// 验证插件配置
    pub fn validate_config(config: &PluginConfig) -> A2AResult<()> {
        if config.framework.is_empty() {
            return Err(A2AError::internal("框架名称不能为空"));
        }
        
        if config.max_connections == 0 {
            return Err(A2AError::internal("最大连接数必须大于0"));
        }
        
        if config.request_timeout == 0 {
            return Err(A2AError::internal("请求超时时间必须大于0"));
        }
        
        Ok(())
    }
    
    /// 生成插件ID
    pub fn generate_plugin_id(framework: &str, name: &str) -> String {
        format!("{}_{}_{}_{}", 
            framework, 
            name, 
            chrono::Utc::now().timestamp(),
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        )
    }
    
    /// 比较插件版本
    pub fn compare_versions(v1: &str, v2: &str) -> std::cmp::Ordering {
        let parse_version = |v: &str| -> Vec<u32> {
            v.split('.')
                .map(|s| s.parse().unwrap_or(0))
                .collect()
        };
        
        let version1 = parse_version(v1);
        let version2 = parse_version(v2);
        
        version1.cmp(&version2)
    }
    
    /// 检查插件兼容性
    pub fn check_compatibility(plugin_info: &PluginInfo, required_version: &str) -> bool {
        Self::compare_versions(&plugin_info.metadata.version, required_version) >= std::cmp::Ordering::Equal
    }
}

/// 框架工具
pub struct FrameworkUtils;

impl FrameworkUtils {
    /// 检测框架环境
    pub async fn detect_framework_environment(framework: &str) -> A2AResult<EnvironmentInfo> {
        match framework {
            "langchain" | "autogen" | "crewai" => {
                Self::detect_python_environment().await
            },
            "mastra" => {
                Self::detect_nodejs_environment().await
            },
            "semantic_kernel" => {
                Self::detect_dotnet_environment().await
            },
            _ => {
                Err(A2AError::internal(format!("不支持的框架: {}", framework)))
            }
        }
    }
    
    async fn detect_python_environment() -> A2AResult<EnvironmentInfo> {
        let output = tokio::process::Command::new("python")
            .args(&["--version"])
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("检测Python环境失败: {}", e)))?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(EnvironmentInfo {
                runtime: "python".to_string(),
                version: version.trim().to_string(),
                available: true,
                path: "python".to_string(),
            })
        } else {
            Ok(EnvironmentInfo {
                runtime: "python".to_string(),
                version: "unknown".to_string(),
                available: false,
                path: "python".to_string(),
            })
        }
    }
    
    async fn detect_nodejs_environment() -> A2AResult<EnvironmentInfo> {
        let output = tokio::process::Command::new("node")
            .args(&["--version"])
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("检测Node.js环境失败: {}", e)))?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(EnvironmentInfo {
                runtime: "node".to_string(),
                version: version.trim().to_string(),
                available: true,
                path: "node".to_string(),
            })
        } else {
            Ok(EnvironmentInfo {
                runtime: "node".to_string(),
                version: "unknown".to_string(),
                available: false,
                path: "node".to_string(),
            })
        }
    }
    
    async fn detect_dotnet_environment() -> A2AResult<EnvironmentInfo> {
        let output = tokio::process::Command::new("dotnet")
            .args(&["--version"])
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("检测.NET环境失败: {}", e)))?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(EnvironmentInfo {
                runtime: "dotnet".to_string(),
                version: version.trim().to_string(),
                available: true,
                path: "dotnet".to_string(),
            })
        } else {
            Ok(EnvironmentInfo {
                runtime: "dotnet".to_string(),
                version: "unknown".to_string(),
                available: false,
                path: "dotnet".to_string(),
            })
        }
    }
    
    /// 安装框架依赖
    pub async fn install_dependencies(framework: &str, dependencies: &[String]) -> A2AResult<()> {
        match framework {
            "langchain" | "autogen" | "crewai" => {
                Self::install_python_dependencies(dependencies).await
            },
            "mastra" => {
                Self::install_nodejs_dependencies(dependencies).await
            },
            "semantic_kernel" => {
                Self::install_dotnet_dependencies(dependencies).await
            },
            _ => {
                Err(A2AError::internal(format!("不支持的框架: {}", framework)))
            }
        }
    }
    
    async fn install_python_dependencies(dependencies: &[String]) -> A2AResult<()> {
        for dep in dependencies {
            tracing::info!("安装Python依赖: {}", dep);
            
            let output = tokio::process::Command::new("pip")
                .args(&["install", dep])
                .output()
                .await
                .map_err(|e| A2AError::internal(format!("安装依赖失败: {}", e)))?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(A2AError::internal(format!("安装{}失败: {}", dep, error)));
            }
        }
        
        Ok(())
    }
    
    async fn install_nodejs_dependencies(dependencies: &[String]) -> A2AResult<()> {
        for dep in dependencies {
            tracing::info!("安装Node.js依赖: {}", dep);
            
            let output = tokio::process::Command::new("npm")
                .args(&["install", dep])
                .output()
                .await
                .map_err(|e| A2AError::internal(format!("安装依赖失败: {}", e)))?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(A2AError::internal(format!("安装{}失败: {}", dep, error)));
            }
        }
        
        Ok(())
    }
    
    async fn install_dotnet_dependencies(dependencies: &[String]) -> A2AResult<()> {
        for dep in dependencies {
            tracing::info!("安装.NET依赖: {}", dep);
            
            let output = tokio::process::Command::new("dotnet")
                .args(&["add", "package", dep])
                .output()
                .await
                .map_err(|e| A2AError::internal(format!("安装依赖失败: {}", e)))?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(A2AError::internal(format!("安装{}失败: {}", dep, error)));
            }
        }
        
        Ok(())
    }
}

/// 消息工具
pub struct MessageUtils;

impl MessageUtils {
    /// 验证消息格式
    pub fn validate_message(message: &A2AMessage) -> A2AResult<()> {
        if message.message_id.is_empty() {
            return Err(A2AError::internal("消息ID不能为空"));
        }
        
        if message.parts.is_empty() {
            return Err(A2AError::internal("消息内容不能为空"));
        }
        
        Ok(())
    }
    
    /// 增强消息元数据
    pub fn enhance_message_metadata(mut message: A2AMessage, metadata: HashMap<String, serde_json::Value>) -> A2AMessage {
        for (key, value) in metadata {
            message.metadata.insert(key, value);
        }
        message
    }
    
    /// 提取消息文本内容
    pub fn extract_text_content(message: &A2AMessage) -> String {
        message.parts
            .iter()
            .filter_map(|part| {
                if let agentx_a2a::MessagePart::Text(text_part) = part {
                    Some(text_part.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// 计算消息大小
    pub fn calculate_message_size(message: &A2AMessage) -> usize {
        serde_json::to_string(message).unwrap_or_default().len()
    }
}

/// 配置工具
pub struct ConfigUtils;

impl ConfigUtils {
    /// 合并配置
    pub fn merge_configs(base: PluginConfig, override_config: PluginConfig) -> PluginConfig {
        PluginConfig {
            framework: if override_config.framework.is_empty() { base.framework } else { override_config.framework },
            framework_version: override_config.framework_version.or(base.framework_version),
            bind_address: if override_config.bind_address == "127.0.0.1:0" { base.bind_address } else { override_config.bind_address },
            server_address: if override_config.server_address == "127.0.0.1:50051" { base.server_address } else { override_config.server_address },
            max_connections: if override_config.max_connections == 100 { base.max_connections } else { override_config.max_connections },
            request_timeout: if override_config.request_timeout == 30 { base.request_timeout } else { override_config.request_timeout },
            enable_tls: override_config.enable_tls || base.enable_tls,
            custom: {
                let mut merged = base.custom;
                merged.extend(override_config.custom);
                merged
            },
        }
    }
    
    /// 从环境变量加载配置
    pub fn load_from_env() -> PluginConfig {
        PluginConfig {
            framework: std::env::var("AGENTX_FRAMEWORK").unwrap_or_else(|_| "custom".to_string()),
            framework_version: std::env::var("AGENTX_FRAMEWORK_VERSION").ok(),
            bind_address: std::env::var("AGENTX_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:0".to_string()),
            server_address: std::env::var("AGENTX_SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:50051".to_string()),
            max_connections: std::env::var("AGENTX_MAX_CONNECTIONS").ok().and_then(|s| s.parse().ok()).unwrap_or(100),
            request_timeout: std::env::var("AGENTX_REQUEST_TIMEOUT").ok().and_then(|s| s.parse().ok()).unwrap_or(30),
            enable_tls: std::env::var("AGENTX_ENABLE_TLS").ok().and_then(|s| s.parse().ok()).unwrap_or(false),
            custom: HashMap::new(),
        }
    }
}

/// 验证工具
pub struct ValidationUtils;

impl ValidationUtils {
    /// 验证URL格式
    pub fn validate_url(url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }
    
    /// 验证端口号
    pub fn validate_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }
    
    /// 验证框架名称
    pub fn validate_framework_name(framework: &str) -> bool {
        !framework.is_empty() && framework.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
}

/// 转换工具
pub struct ConversionUtils;

impl ConversionUtils {
    /// 转换配置为JSON
    pub fn config_to_json(config: &PluginConfig) -> A2AResult<String> {
        serde_json::to_string_pretty(config)
            .map_err(|e| A2AError::internal(format!("配置序列化失败: {}", e)))
    }
    
    /// 从JSON转换配置
    pub fn config_from_json(json: &str) -> A2AResult<PluginConfig> {
        serde_json::from_str(json)
            .map_err(|e| A2AError::internal(format!("配置反序列化失败: {}", e)))
    }
}

/// 测试工具
pub struct TestUtils;

impl TestUtils {
    /// 创建测试消息
    pub fn create_test_message(content: &str) -> A2AMessage {
        A2AMessage::agent_message(content.to_string())
    }
    
    /// 创建测试配置
    pub fn create_test_config(framework: &str) -> PluginConfig {
        PluginConfig::default_for_framework(framework).unwrap_or_default()
    }
    
    /// 验证测试环境
    pub async fn verify_test_environment() -> A2AResult<()> {
        // 检查基本的测试环境要求
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug");
        }
        
        Ok(())
    }
}

/// 环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub runtime: String,
    pub version: String,
    pub available: bool,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_utils() {
        let id = PluginUtils::generate_plugin_id("langchain", "test");
        assert!(id.starts_with("langchain_test_"));
        
        assert_eq!(PluginUtils::compare_versions("1.0.0", "1.0.1"), std::cmp::Ordering::Less);
        assert_eq!(PluginUtils::compare_versions("1.1.0", "1.0.1"), std::cmp::Ordering::Greater);
    }
    
    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::validate_url("http://localhost:8080"));
        assert!(!ValidationUtils::validate_url("invalid-url"));
        
        assert!(ValidationUtils::validate_port(8080));
        assert!(!ValidationUtils::validate_port(0));
        
        assert!(ValidationUtils::validate_framework_name("langchain"));
        assert!(!ValidationUtils::validate_framework_name(""));
    }
    
    #[test]
    fn test_message_utils() {
        let message = TestUtils::create_test_message("Hello");
        assert!(MessageUtils::validate_message(&message).is_ok());
        
        let content = MessageUtils::extract_text_content(&message);
        assert_eq!(content, "Hello");
        
        let size = MessageUtils::calculate_message_size(&message);
        assert!(size > 0);
    }
}
