//! 框架适配器模块
//! 
//! 提供对不同AI框架的统一适配接口

use agentx_a2a::{A2AMessage, A2AResult, A2AError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// 支持的框架类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameworkType {
    /// LangChain (Python)
    LangChain,
    /// AutoGen (Python)
    AutoGen,
    /// Mastra (Node.js/TypeScript)
    Mastra,
    /// CrewAI (Python)
    CrewAI,
    /// Semantic Kernel (C#/.NET)
    SemanticKernel,
    /// LangGraph (Python)
    LangGraph,
    /// 自定义框架
    Custom(String),
}

/// 框架配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    /// 框架类型
    pub framework_type: FrameworkType,
    /// 执行环境路径
    pub runtime_path: String,
    /// 工作目录
    pub working_directory: String,
    /// 环境变量
    pub environment_variables: HashMap<String, String>,
    /// 启动参数
    pub startup_args: Vec<String>,
    /// 依赖包列表
    pub dependencies: Vec<String>,
    /// 自定义配置
    pub custom_config: HashMap<String, serde_json::Value>,
}

/// 框架适配器接口
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    /// 获取框架类型
    fn get_framework_type(&self) -> FrameworkType;
    
    /// 初始化框架环境
    async fn initialize_environment(&mut self) -> A2AResult<()>;
    
    /// 启动框架实例
    async fn start_framework(&mut self) -> A2AResult<()>;
    
    /// 停止框架实例
    async fn stop_framework(&mut self) -> A2AResult<()>;
    
    /// 检查框架健康状态
    async fn check_health(&self) -> A2AResult<bool>;
    
    /// 执行框架特定的命令
    async fn execute_command(&mut self, command: &str, args: Vec<String>) -> A2AResult<String>;
    
    /// 转换A2A消息为框架特定格式
    async fn convert_message_to_framework(&self, message: &A2AMessage) -> A2AResult<serde_json::Value>;
    
    /// 转换框架消息为A2A格式
    async fn convert_message_from_framework(&self, data: serde_json::Value) -> A2AResult<A2AMessage>;
}

/// 框架管理器
pub struct Framework {
    /// 框架类型
    framework_type: FrameworkType,
    /// 框架配置
    config: FrameworkConfig,
    /// 适配器实例
    adapter: Box<dyn FrameworkAdapter>,
    /// 运行状态
    is_running: bool,
}

impl Framework {
    /// 创建新的框架实例
    pub fn new(framework_type: FrameworkType, config: FrameworkConfig, adapter: Box<dyn FrameworkAdapter>) -> Self {
        Self {
            framework_type,
            config,
            adapter,
            is_running: false,
        }
    }
    
    /// 获取框架类型
    pub fn get_type(&self) -> &FrameworkType {
        &self.framework_type
    }
    
    /// 获取配置
    pub fn get_config(&self) -> &FrameworkConfig {
        &self.config
    }
    
    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    /// 启动框架
    pub async fn start(&mut self) -> A2AResult<()> {
        if self.is_running {
            return Err(A2AError::internal("框架已经在运行中"));
        }
        
        tracing::info!("启动框架: {:?}", self.framework_type);
        
        self.adapter.initialize_environment().await?;
        self.adapter.start_framework().await?;
        
        self.is_running = true;
        tracing::info!("框架启动成功: {:?}", self.framework_type);
        
        Ok(())
    }
    
    /// 停止框架
    pub async fn stop(&mut self) -> A2AResult<()> {
        if !self.is_running {
            return Ok(());
        }
        
        tracing::info!("停止框架: {:?}", self.framework_type);
        
        self.adapter.stop_framework().await?;
        self.is_running = false;
        
        tracing::info!("框架停止成功: {:?}", self.framework_type);
        Ok(())
    }
    
    /// 处理消息
    pub async fn process_message(&mut self, message: A2AMessage) -> A2AResult<Option<A2AMessage>> {
        if !self.is_running {
            return Err(A2AError::internal("框架未运行"));
        }
        
        // 转换消息格式
        let _framework_message = self.adapter.convert_message_to_framework(&message).await?;
        
        // 这里应该调用框架特定的处理逻辑
        // 暂时返回一个简单的响应
        let response_data = serde_json::json!({
            "type": "response",
            "original_message_id": message.message_id,
            "content": format!("Processed by {:?}", self.framework_type),
            "timestamp": chrono::Utc::now()
        });
        
        // 转换回A2A格式
        let response_message = self.adapter.convert_message_from_framework(response_data).await?;
        
        Ok(Some(response_message))
    }
}

/// LangChain适配器实现
pub struct LangChainAdapter {
    config: FrameworkConfig,
    python_process: Option<tokio::process::Child>,
}

impl LangChainAdapter {
    pub fn new(config: FrameworkConfig) -> Self {
        Self {
            config,
            python_process: None,
        }
    }
}

#[async_trait]
impl FrameworkAdapter for LangChainAdapter {
    fn get_framework_type(&self) -> FrameworkType {
        FrameworkType::LangChain
    }
    
    async fn initialize_environment(&mut self) -> A2AResult<()> {
        tracing::info!("初始化LangChain环境");
        
        // 检查Python环境
        let output = tokio::process::Command::new(&self.config.runtime_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("检查Python版本失败: {}", e)))?;
        
        if !output.status.success() {
            return Err(A2AError::internal("Python环境不可用"));
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Python版本: {}", version.trim());
        
        // 检查LangChain依赖
        let check_deps = tokio::process::Command::new(&self.config.runtime_path)
            .args(&["-c", "import langchain; print(langchain.__version__)"])
            .output()
            .await;
        
        match check_deps {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("LangChain版本: {}", version.trim());
            },
            _ => {
                tracing::warn!("LangChain未安装，尝试安装...");
                // 这里可以添加自动安装逻辑
            }
        }
        
        Ok(())
    }
    
    async fn start_framework(&mut self) -> A2AResult<()> {
        tracing::info!("启动LangChain框架");
        
        // 启动Python进程
        let mut cmd = tokio::process::Command::new(&self.config.runtime_path);
        cmd.args(&self.config.startup_args);
        
        // 设置环境变量
        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }
        
        // 设置工作目录
        cmd.current_dir(&self.config.working_directory);
        
        let child = cmd.spawn()
            .map_err(|e| A2AError::internal(format!("启动Python进程失败: {}", e)))?;
        
        self.python_process = Some(child);
        
        // 等待一段时间确保进程启动
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        Ok(())
    }
    
    async fn stop_framework(&mut self) -> A2AResult<()> {
        tracing::info!("停止LangChain框架");
        
        if let Some(mut process) = self.python_process.take() {
            let _ = process.kill();
            let _ = process.wait().await;
        }
        
        Ok(())
    }
    
    async fn check_health(&self) -> A2AResult<bool> {
        if let Some(_process) = &self.python_process {
            // 简化健康检查，实际实现中可以通过其他方式检查进程状态
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn execute_command(&mut self, command: &str, args: Vec<String>) -> A2AResult<String> {
        let mut cmd_args = vec!["-c".to_string(), command.to_string()];
        cmd_args.extend(args);
        
        let output = tokio::process::Command::new(&self.config.runtime_path)
            .args(&cmd_args)
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("执行命令失败: {}", e)))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(A2AError::internal(format!("命令执行失败: {}", error)))
        }
    }
    
    async fn convert_message_to_framework(&self, message: &A2AMessage) -> A2AResult<serde_json::Value> {
        // 转换A2A消息为LangChain格式
        let framework_message = serde_json::json!({
            "id": message.message_id,
            "role": message.role,
            "content": message.parts,
            "metadata": {
                "timestamp": chrono::Utc::now(),
                "framework": "langchain"
            }
        });

        Ok(framework_message)
    }
    
    async fn convert_message_from_framework(&self, data: serde_json::Value) -> A2AResult<A2AMessage> {
        // 转换LangChain消息为A2A格式
        let content = data.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("Empty response")
            .to_string();
        
        let message = A2AMessage::agent_message(content);
        Ok(message)
    }
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self {
            framework_type: FrameworkType::Custom("unknown".to_string()),
            runtime_path: "python".to_string(),
            working_directory: ".".to_string(),
            environment_variables: HashMap::new(),
            startup_args: Vec::new(),
            dependencies: Vec::new(),
            custom_config: HashMap::new(),
        }
    }
}

/// AutoGen适配器实现
pub struct AutoGenAdapter {
    config: FrameworkConfig,
    python_process: Option<tokio::process::Child>,
}

impl AutoGenAdapter {
    pub fn new(config: FrameworkConfig) -> Self {
        Self {
            config,
            python_process: None,
        }
    }
}

#[async_trait]
impl FrameworkAdapter for AutoGenAdapter {
    fn get_framework_type(&self) -> FrameworkType {
        FrameworkType::AutoGen
    }

    async fn initialize_environment(&mut self) -> A2AResult<()> {
        tracing::info!("初始化AutoGen环境");

        // 检查Python和AutoGen依赖
        let check_deps = tokio::process::Command::new(&self.config.runtime_path)
            .args(&["-c", "import autogen; print(autogen.__version__)"])
            .output()
            .await;

        match check_deps {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                tracing::info!("AutoGen版本: {}", version.trim());
            },
            _ => {
                tracing::warn!("AutoGen未安装");
                return Err(A2AError::internal("AutoGen依赖未安装"));
            }
        }

        Ok(())
    }

    async fn start_framework(&mut self) -> A2AResult<()> {
        tracing::info!("启动AutoGen框架");

        let mut cmd = tokio::process::Command::new(&self.config.runtime_path);
        cmd.args(&self.config.startup_args);

        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }

        cmd.current_dir(&self.config.working_directory);

        let child = cmd.spawn()
            .map_err(|e| A2AError::internal(format!("启动AutoGen进程失败: {}", e)))?;

        self.python_process = Some(child);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    async fn stop_framework(&mut self) -> A2AResult<()> {
        tracing::info!("停止AutoGen框架");

        if let Some(mut process) = self.python_process.take() {
            let _ = process.kill();
            let _ = process.wait().await;
        }

        Ok(())
    }

    async fn check_health(&self) -> A2AResult<bool> {
        if let Some(_process) = &self.python_process {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn execute_command(&mut self, command: &str, args: Vec<String>) -> A2AResult<String> {
        let mut cmd_args = vec!["-c".to_string(), command.to_string()];
        cmd_args.extend(args);

        let output = tokio::process::Command::new(&self.config.runtime_path)
            .args(&cmd_args)
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("执行AutoGen命令失败: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(A2AError::internal(format!("AutoGen命令执行失败: {}", error)))
        }
    }

    async fn convert_message_to_framework(&self, message: &A2AMessage) -> A2AResult<serde_json::Value> {
        let framework_message = serde_json::json!({
            "id": message.message_id,
            "role": message.role,
            "content": message.parts,
            "metadata": {
                "timestamp": chrono::Utc::now(),
                "framework": "autogen"
            }
        });

        Ok(framework_message)
    }

    async fn convert_message_from_framework(&self, data: serde_json::Value) -> A2AResult<A2AMessage> {
        let content = data.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("Empty AutoGen response")
            .to_string();

        let message = A2AMessage::agent_message(content);
        Ok(message)
    }
}

/// Mastra适配器实现
pub struct MastraAdapter {
    config: FrameworkConfig,
    node_process: Option<tokio::process::Child>,
}

impl MastraAdapter {
    pub fn new(config: FrameworkConfig) -> Self {
        Self {
            config,
            node_process: None,
        }
    }
}

#[async_trait]
impl FrameworkAdapter for MastraAdapter {
    fn get_framework_type(&self) -> FrameworkType {
        FrameworkType::Mastra
    }

    async fn initialize_environment(&mut self) -> A2AResult<()> {
        tracing::info!("初始化Mastra环境");

        // 检查Node.js环境
        let output = tokio::process::Command::new(&self.config.runtime_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("检查Node.js版本失败: {}", e)))?;

        if !output.status.success() {
            return Err(A2AError::internal("Node.js环境不可用"));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!("Node.js版本: {}", version.trim());

        Ok(())
    }

    async fn start_framework(&mut self) -> A2AResult<()> {
        tracing::info!("启动Mastra框架");

        let mut cmd = tokio::process::Command::new(&self.config.runtime_path);
        cmd.args(&self.config.startup_args);

        for (key, value) in &self.config.environment_variables {
            cmd.env(key, value);
        }

        cmd.current_dir(&self.config.working_directory);

        let child = cmd.spawn()
            .map_err(|e| A2AError::internal(format!("启动Mastra进程失败: {}", e)))?;

        self.node_process = Some(child);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    async fn stop_framework(&mut self) -> A2AResult<()> {
        tracing::info!("停止Mastra框架");

        if let Some(mut process) = self.node_process.take() {
            let _ = process.kill();
            let _ = process.wait().await;
        }

        Ok(())
    }

    async fn check_health(&self) -> A2AResult<bool> {
        if let Some(_process) = &self.node_process {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn execute_command(&mut self, command: &str, args: Vec<String>) -> A2AResult<String> {
        let mut cmd_args = vec!["-e".to_string(), command.to_string()];
        cmd_args.extend(args);

        let output = tokio::process::Command::new(&self.config.runtime_path)
            .args(&cmd_args)
            .output()
            .await
            .map_err(|e| A2AError::internal(format!("执行Mastra命令失败: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(A2AError::internal(format!("Mastra命令执行失败: {}", error)))
        }
    }

    async fn convert_message_to_framework(&self, message: &A2AMessage) -> A2AResult<serde_json::Value> {
        let framework_message = serde_json::json!({
            "id": message.message_id,
            "role": message.role,
            "content": message.parts,
            "metadata": {
                "timestamp": chrono::Utc::now(),
                "framework": "mastra"
            }
        });

        Ok(framework_message)
    }

    async fn convert_message_from_framework(&self, data: serde_json::Value) -> A2AResult<A2AMessage> {
        let content = data.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("Empty Mastra response")
            .to_string();

        let message = A2AMessage::agent_message(content);
        Ok(message)
    }
}

impl FrameworkType {
    /// 获取框架的默认运行时路径
    pub fn default_runtime_path(&self) -> &'static str {
        match self {
            FrameworkType::LangChain => "python",
            FrameworkType::AutoGen => "python",
            FrameworkType::CrewAI => "python",
            FrameworkType::LangGraph => "python",
            FrameworkType::Mastra => "node",
            FrameworkType::SemanticKernel => "dotnet",
            FrameworkType::Custom(_) => "unknown",
        }
    }

    /// 获取框架的默认依赖列表
    pub fn default_dependencies(&self) -> Vec<&'static str> {
        match self {
            FrameworkType::LangChain => vec!["langchain", "langchain-community"],
            FrameworkType::AutoGen => vec!["pyautogen"],
            FrameworkType::CrewAI => vec!["crewai"],
            FrameworkType::LangGraph => vec!["langgraph"],
            FrameworkType::Mastra => vec!["@mastra/core"],
            FrameworkType::SemanticKernel => vec!["Microsoft.SemanticKernel"],
            FrameworkType::Custom(_) => vec![],
        }
    }
}
