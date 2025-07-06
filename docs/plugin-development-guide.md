# AgentX 插件开发指南

## 🎯 概述

AgentX 采用基于 gRPC 的插件架构，支持多语言插件开发。插件系统允许开发者扩展 AgentX 的功能，集成不同的 AI 框架和服务。

## 🏗️ 插件架构

### 核心概念

1. **插件接口**: 基于 gRPC 的标准化接口
2. **生命周期管理**: 插件的加载、启动、停止和卸载
3. **消息路由**: 插件间的消息传递和路由
4. **配置管理**: 插件的配置和参数管理
5. **错误处理**: 统一的错误处理和恢复机制

### 插件类型

- **框架适配器**: 集成 LangChain、AutoGen、Mastra 等 AI 框架
- **协议转换器**: 支持不同的通信协议
- **数据处理器**: 数据转换和预处理
- **监控插件**: 性能监控和日志收集
- **安全插件**: 认证、授权和加密

## 🚀 快速开始

### 1. 环境准备

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 protobuf 编译器
# macOS
brew install protobuf

# Ubuntu/Debian
sudo apt-get install protobuf-compiler

# 克隆 AgentX 项目
git clone https://github.com/louloulin/agentx.git
cd agentx
```

### 2. 创建插件项目

```bash
# 使用 AgentX CLI 创建插件项目
cargo install agentx-cli
agentx create-plugin my-plugin --lang rust

# 或手动创建
cargo new my-plugin --lib
cd my-plugin
```

### 3. 添加依赖

```toml
# Cargo.toml
[dependencies]
agentx-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
tonic = "0.10"
prost = "0.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
anyhow = "1.0"

[build-dependencies]
tonic-build = "0.10"
```

## 📝 插件开发

### 1. 基础插件结构

```rust
use agentx_sdk::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MyPlugin {
    config: PluginConfig,
    state: PluginState,
}

impl MyPlugin {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            state: PluginState::new(),
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "我的第一个 AgentX 插件".to_string(),
            author: "Your Name".to_string(),
            capabilities: vec![
                "message_processing".to_string(),
                "data_transformation".to_string(),
            ],
            dependencies: HashMap::new(),
        }
    }

    async fn initialize(&mut self) -> PluginResult<()> {
        tracing::info!("初始化插件: {}", self.metadata().name);
        
        // 初始化逻辑
        self.state.set_status(PluginStatus::Ready);
        
        Ok(())
    }

    async fn execute(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        tracing::debug!("处理请求: {:?}", request);
        
        match request.action.as_str() {
            "process_message" => self.process_message(request).await,
            "transform_data" => self.transform_data(request).await,
            _ => Err(PluginError::UnsupportedAction(request.action)),
        }
    }

    async fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("关闭插件: {}", self.metadata().name);
        
        // 清理资源
        self.state.set_status(PluginStatus::Stopped);
        
        Ok(())
    }

    fn health_check(&self) -> PluginHealth {
        PluginHealth {
            status: self.state.status(),
            last_heartbeat: std::time::SystemTime::now(),
            metrics: self.collect_metrics(),
        }
    }
}

impl MyPlugin {
    async fn process_message(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        let message = request.data.get("message")
            .ok_or(PluginError::MissingParameter("message".to_string()))?;
        
        // 处理消息逻辑
        let processed = format!("已处理: {}", message);
        
        let mut response_data = HashMap::new();
        response_data.insert("result".to_string(), processed);
        
        Ok(PluginResponse {
            success: true,
            data: response_data,
            error: None,
        })
    }

    async fn transform_data(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        // 数据转换逻辑
        let input_data = &request.data;
        let transformed_data = self.apply_transformation(input_data)?;
        
        Ok(PluginResponse {
            success: true,
            data: transformed_data,
            error: None,
        })
    }

    fn apply_transformation(&self, data: &HashMap<String, String>) -> PluginResult<HashMap<String, String>> {
        // 实现具体的转换逻辑
        let mut result = HashMap::new();
        
        for (key, value) in data {
            let transformed_key = format!("transformed_{}", key);
            let transformed_value = value.to_uppercase();
            result.insert(transformed_key, transformed_value);
        }
        
        Ok(result)
    }

    fn collect_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("requests_processed".to_string(), 100.0);
        metrics.insert("average_response_time_ms".to_string(), 5.2);
        metrics.insert("error_rate".to_string(), 0.01);
        metrics
    }
}
```

### 2. 插件配置

```rust
// config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyPluginConfig {
    /// 插件启用状态
    pub enabled: bool,
    /// 处理超时时间（秒）
    pub timeout_seconds: u64,
    /// 最大并发请求数
    pub max_concurrent_requests: usize,
    /// 自定义参数
    pub custom_params: std::collections::HashMap<String, String>,
}

impl Default for MyPluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 30,
            max_concurrent_requests: 100,
            custom_params: std::collections::HashMap::new(),
        }
    }
}
```

### 3. 插件注册

```rust
// main.rs
use agentx_sdk::prelude::*;
use my_plugin::MyPlugin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();

    // 加载配置
    let config = PluginConfig::from_file("config.toml")?;
    
    // 创建插件实例
    let mut plugin = MyPlugin::new(config);
    
    // 初始化插件
    plugin.initialize().await?;
    
    // 注册插件到 AgentX
    let plugin_server = PluginServer::new(plugin);
    plugin_server.serve("0.0.0.0:50051").await?;
    
    Ok(())
}
```

## 🧪 测试插件

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use agentx_sdk::testing::*;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let config = PluginConfig::default();
        let mut plugin = MyPlugin::new(config);
        
        let result = plugin.initialize().await;
        assert!(result.is_ok());
        assert_eq!(plugin.state.status(), PluginStatus::Ready);
    }

    #[tokio::test]
    async fn test_message_processing() {
        let config = PluginConfig::default();
        let plugin = MyPlugin::new(config);
        
        let mut request_data = HashMap::new();
        request_data.insert("message".to_string(), "Hello, World!".to_string());
        
        let request = PluginRequest {
            action: "process_message".to_string(),
            data: request_data,
        };
        
        let response = plugin.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.data.contains_key("result"));
    }
}
```

### 2. 集成测试

```rust
// tests/integration_test.rs
use agentx_sdk::testing::*;
use my_plugin::MyPlugin;

#[tokio::test]
async fn test_plugin_integration() {
    let test_env = TestEnvironment::new().await;
    
    // 启动插件服务器
    let plugin = MyPlugin::new(PluginConfig::default());
    let server = test_env.start_plugin_server(plugin).await;
    
    // 创建客户端
    let client = test_env.create_plugin_client().await;
    
    // 测试插件调用
    let response = client.call_plugin("my-plugin", "process_message", 
        hashmap!{"message" => "test"}).await;
    
    assert!(response.is_ok());
}
```

## 📦 插件打包和分发

### 1. 构建插件

```bash
# 构建发布版本
cargo build --release

# 运行测试
cargo test

# 检查代码质量
cargo clippy
cargo fmt --check
```

### 2. 创建插件包

```toml
# plugin.toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "我的第一个 AgentX 插件"
author = "Your Name"
license = "MIT"
repository = "https://github.com/yourusername/my-plugin"

[plugin.capabilities]
message_processing = "1.0"
data_transformation = "1.0"

[plugin.dependencies]
agentx-core = ">=0.1.0"

[plugin.config]
schema = "config-schema.json"
default = "config-default.toml"
```

### 3. 发布插件

```bash
# 打包插件
agentx package

# 发布到插件市场
agentx publish --registry https://plugins.agentx.dev
```

## 🔧 高级功能

### 1. 插件间通信

```rust
impl MyPlugin {
    async fn call_other_plugin(&self, plugin_name: &str, action: &str, data: HashMap<String, String>) -> PluginResult<PluginResponse> {
        let client = self.get_plugin_client(plugin_name)?;
        
        let request = PluginRequest {
            action: action.to_string(),
            data,
        };
        
        client.execute(request).await
    }
}
```

### 2. 事件处理

```rust
#[async_trait]
impl EventHandler for MyPlugin {
    async fn handle_event(&self, event: PluginEvent) -> PluginResult<()> {
        match event.event_type.as_str() {
            "agent_registered" => self.on_agent_registered(event).await,
            "message_received" => self.on_message_received(event).await,
            _ => Ok(()),
        }
    }
}
```

### 3. 状态管理

```rust
impl MyPlugin {
    fn save_state(&self) -> PluginResult<()> {
        let state_data = serde_json::to_string(&self.state)?;
        std::fs::write("plugin-state.json", state_data)?;
        Ok(())
    }
    
    fn load_state(&mut self) -> PluginResult<()> {
        if let Ok(state_data) = std::fs::read_to_string("plugin-state.json") {
            self.state = serde_json::from_str(&state_data)?;
        }
        Ok(())
    }
}
```

## 📚 最佳实践

1. **错误处理**: 使用 `Result` 类型处理所有可能的错误
2. **日志记录**: 使用 `tracing` 记录详细的操作日志
3. **配置管理**: 支持环境变量和配置文件
4. **性能优化**: 使用异步编程和连接池
5. **安全考虑**: 验证输入数据，避免注入攻击
6. **文档完整**: 提供详细的 API 文档和使用示例

## 🔗 相关资源

- [AgentX SDK 文档](https://docs.agentx.dev/sdk)
- [插件 API 参考](https://docs.agentx.dev/api)
- [示例插件](https://github.com/agentx-dev/examples)
- [插件市场](https://plugins.agentx.dev)

---

开始构建您的第一个 AgentX 插件吧！🚀
