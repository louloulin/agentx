//! Python桥接器
//! 
//! 提供与Python LangChain环境的通信功能

use crate::config::LangChainConfig;
use crate::error::{LangChainError, LangChainResult};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, error, warn};

/// Python桥接器
pub struct PythonBridge {
    /// HTTP客户端
    client: Client,
    /// Python服务配置
    config: Arc<RwLock<LangChainConfig>>,
    /// Python进程句柄
    python_process: Arc<RwLock<Option<tokio::process::Child>>>,
    /// 服务状态
    service_ready: Arc<RwLock<bool>>,
}

impl PythonBridge {
    /// 创建新的Python桥接器
    pub async fn new(config: &LangChainConfig) -> LangChainResult<Self> {
        info!("🐍 初始化Python桥接器");
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| LangChainError::NetworkError(format!("HTTP客户端创建失败: {}", e)))?;
        
        let bridge = Self {
            client,
            config: Arc::new(RwLock::new(config.clone())),
            python_process: Arc::new(RwLock::new(None)),
            service_ready: Arc::new(RwLock::new(false)),
        };
        
        // 启动Python服务
        bridge.start_python_service().await?;
        
        info!("✅ Python桥接器初始化完成");
        Ok(bridge)
    }
    
    /// 启动Python服务
    async fn start_python_service(&self) -> LangChainResult<()> {
        info!("🚀 启动Python LangChain服务...");
        
        let config = self.config.read().await;
        
        // 检查Python环境
        self.check_python_environment().await?;
        
        // 启动Python服务进程
        let mut cmd = Command::new(&config.python_executable);
        cmd.arg("-m")
            .arg("agentx_langchain_service")
            .arg("--host")
            .arg(&config.python_service_host)
            .arg("--port")
            .arg(&config.python_service_port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // 设置环境变量
        for (key, value) in &config.environment_variables {
            cmd.env(key, value);
        }
        
        let child = tokio::process::Command::from(cmd)
            .spawn()
            .map_err(|e| LangChainError::ProcessError(format!("Python服务启动失败: {}", e)))?;
        
        {
            let mut process = self.python_process.write().await;
            *process = Some(child);
        }
        
        // 等待服务就绪
        self.wait_for_service_ready().await?;
        
        info!("✅ Python LangChain服务启动成功");
        Ok(())
    }
    
    /// 检查Python环境
    async fn check_python_environment(&self) -> LangChainResult<()> {
        info!("🔍 检查Python环境...");
        
        let config = self.config.read().await;
        
        // 检查Python可执行文件
        let output = Command::new(&config.python_executable)
            .arg("--version")
            .output()
            .map_err(|e| LangChainError::EnvironmentError(format!("Python不可用: {}", e)))?;
        
        if !output.status.success() {
            return Err(LangChainError::EnvironmentError(
                "Python版本检查失败".to_string()
            ));
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("   Python版本: {}", version.trim());
        
        // 检查必需的包
        let required_packages = vec![
            "langchain",
            "langchain-core",
            "langchain-community",
            "fastapi",
            "uvicorn",
            "pydantic",
        ];
        
        for package in required_packages {
            let output = Command::new(&config.python_executable)
                .arg("-c")
                .arg(&format!("import {}", package))
                .output()
                .map_err(|e| LangChainError::EnvironmentError(format!("包检查失败: {}", e)))?;
            
            if !output.status.success() {
                return Err(LangChainError::EnvironmentError(
                    format!("必需的Python包 '{}' 未安装", package)
                ));
            }
        }
        
        info!("✅ Python环境检查通过");
        Ok(())
    }
    
    /// 等待服务就绪
    async fn wait_for_service_ready(&self) -> LangChainResult<()> {
        info!("⏳ 等待Python服务就绪...");
        
        let config = self.config.read().await;
        let base_url = format!("http://{}:{}", config.python_service_host, config.python_service_port);
        
        for attempt in 1..=30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            
            match self.client.get(&format!("{}/health", base_url)).send().await {
                Ok(response) if response.status().is_success() => {
                    let mut service_ready = self.service_ready.write().await;
                    *service_ready = true;
                    info!("✅ Python服务就绪 (尝试 {})", attempt);
                    return Ok(());
                }
                Ok(_) => {
                    debug!("Python服务未就绪 (尝试 {})", attempt);
                }
                Err(e) => {
                    debug!("连接Python服务失败 (尝试 {}): {}", attempt, e);
                }
            }
        }
        
        Err(LangChainError::ServiceError("Python服务启动超时".to_string()))
    }
    
    /// 关闭Python服务
    pub async fn shutdown(&self) -> LangChainResult<()> {
        info!("🛑 关闭Python服务...");
        
        {
            let mut service_ready = self.service_ready.write().await;
            *service_ready = false;
        }
        
        let mut process = self.python_process.write().await;
        if let Some(mut child) = process.take() {
            if let Err(e) = child.kill().await {
                warn!("终止Python进程时出现警告: {}", e);
            }
        }
        
        info!("✅ Python服务已关闭");
        Ok(())
    }
    
    /// 健康检查
    pub async fn health_check(&self) -> LangChainResult<()> {
        let service_ready = self.service_ready.read().await;
        if !*service_ready {
            return Err(LangChainError::ServiceError("Python服务未就绪".to_string()));
        }
        
        let config = self.config.read().await;
        let base_url = format!("http://{}:{}", config.python_service_host, config.python_service_port);
        
        let response = self.client
            .get(&format!("{}/health", base_url))
            .send()
            .await
            .map_err(|e| LangChainError::NetworkError(format!("健康检查请求失败: {}", e)))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(LangChainError::ServiceError(
                format!("Python服务健康检查失败: {}", response.status())
            ))
        }
    }
    
    /// 获取Python版本
    pub async fn get_python_version(&self) -> LangChainResult<String> {
        let response = self.call_python_api("/version", json!({})).await?;
        
        response.get("python_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| LangChainError::InvalidResponse("缺少Python版本信息".to_string()))
    }
    
    /// 获取LangChain版本
    pub async fn get_langchain_version(&self) -> LangChainResult<String> {
        let response = self.call_python_api("/version", json!({})).await?;
        
        response.get("langchain_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| LangChainError::InvalidResponse("缺少LangChain版本信息".to_string()))
    }
    
    /// 检查包是否已安装
    pub async fn check_package_installed(&self, package: &str) -> LangChainResult<bool> {
        let request = json!({
            "package": package
        });
        
        let response = self.call_python_api("/check_package", request).await?;
        
        response.get("installed")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| LangChainError::InvalidResponse("缺少包安装状态".to_string()))
    }
    
    /// 初始化LangChain环境
    pub async fn initialize_langchain_environment(&self) -> LangChainResult<()> {
        info!("🔧 初始化LangChain环境...");
        
        let request = json!({
            "action": "initialize"
        });
        
        let _response = self.call_python_api("/initialize", request).await?;
        
        info!("✅ LangChain环境初始化完成");
        Ok(())
    }
    
    /// 调用LangChain聊天
    pub async fn call_langchain_chat(&self, request: Value) -> LangChainResult<Value> {
        debug!("💬 调用LangChain聊天API");
        self.call_python_api("/chat", request).await
    }
    
    /// 调用LangChain工具
    pub async fn call_langchain_tool(&self, request: Value) -> LangChainResult<Value> {
        debug!("🔧 调用LangChain工具API");
        self.call_python_api("/tool", request).await
    }
    
    /// 调用LangChain链
    pub async fn call_langchain_chain(&self, request: Value) -> LangChainResult<Value> {
        debug!("⛓️ 调用LangChain链API");
        self.call_python_api("/chain", request).await
    }
    
    /// 创建LangChain Agent
    pub async fn create_langchain_agent(&self, request: Value) -> LangChainResult<Value> {
        debug!("📝 创建LangChain Agent");
        self.call_python_api("/agent/create", request).await
    }
    
    /// 删除LangChain Agent
    pub async fn delete_langchain_agent(&self, agent_id: &str) -> LangChainResult<Value> {
        debug!("🗑️ 删除LangChain Agent: {}", agent_id);
        
        let request = json!({
            "agent_id": agent_id
        });
        
        self.call_python_api("/agent/delete", request).await
    }
    
    /// 预加载模型
    pub async fn preload_model(&self, model_name: &str) -> LangChainResult<Value> {
        debug!("📦 预加载模型: {}", model_name);
        
        let request = json!({
            "model": model_name
        });
        
        self.call_python_api("/model/preload", request).await
    }
    
    /// 预加载工具
    pub async fn preload_tool(&self, tool_name: &str) -> LangChainResult<Value> {
        debug!("🔧 预加载工具: {}", tool_name);
        
        let request = json!({
            "tool": tool_name
        });
        
        self.call_python_api("/tool/preload", request).await
    }
    
    /// 调用Python API
    async fn call_python_api(&self, endpoint: &str, request: Value) -> LangChainResult<Value> {
        let service_ready = self.service_ready.read().await;
        if !*service_ready {
            return Err(LangChainError::ServiceError("Python服务未就绪".to_string()));
        }
        drop(service_ready);
        
        let config = self.config.read().await;
        let base_url = format!("http://{}:{}", config.python_service_host, config.python_service_port);
        let url = format!("{}{}", base_url, endpoint);
        
        debug!("🌐 调用Python API: {}", url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| LangChainError::NetworkError(format!("API请求失败: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            return Err(LangChainError::ApiError(
                format!("API调用失败 ({}): {}", status, error_text)
            ));
        }
        
        let result: Value = response.json().await
            .map_err(|e| LangChainError::InvalidResponse(format!("响应解析失败: {}", e)))?;
        
        debug!("✅ Python API调用成功");
        Ok(result)
    }
}
