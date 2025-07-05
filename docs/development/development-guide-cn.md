# AgentX 开发指南

## 📖 概述

本指南将帮助开发者快速上手AgentX项目的开发，包括环境搭建、代码结构、开发流程和最佳实践。

[English Version](development-guide.md) | [中文版本](development-guide-cn.md)

## 🛠️ 开发环境搭建

### 系统要求

- **操作系统**: Linux, macOS, Windows (WSL2推荐)
- **Rust**: 1.70+ (推荐使用rustup安装)
- **Node.js**: 18+ (用于Mastra插件开发)
- **Python**: 3.8+ (用于LangChain/AutoGen插件开发)
- **Protocol Buffers**: 3.15+
- **Docker**: 20.10+ (可选，用于容器化开发)

### 安装Rust工具链

```bash
# 安装rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 安装必要的组件
rustup component add clippy rustfmt

# 验证安装
rustc --version
cargo --version
```

### 安装Protocol Buffers

#### macOS
```bash
brew install protobuf
```

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install protobuf-compiler
```

#### Windows
```bash
# 使用chocolatey
choco install protoc
```

### 克隆项目

```bash
git clone https://github.com/agentx/agentx.git
cd agentx

# 构建项目
cargo build

# 运行测试
cargo test
```

## 📁 项目结构

```
agentx/
├── crates/                     # Rust crates
│   ├── agentx-core/            # 核心管理模块
│   │   ├── src/
│   │   │   ├── lib.rs          # 模块入口
│   │   │   ├── protocol_compat.rs  # 协议兼容层
│   │   │   ├── cloud_native.rs     # 云原生支持
│   │   │   ├── developer_ecosystem.rs # 开发者生态
│   │   │   └── error_recovery.rs    # 错误恢复
│   │   └── Cargo.toml
│   ├── agentx-a2a/             # A2A协议实现
│   │   ├── src/
│   │   │   ├── lib.rs          # 模块入口
│   │   │   ├── message.rs      # 消息格式
│   │   │   ├── agent_card.rs   # Agent描述
│   │   │   ├── capability.rs   # 能力匹配
│   │   │   ├── protocol_engine.rs # 协议引擎
│   │   │   ├── streaming.rs    # 流式处理
│   │   │   ├── security.rs     # 安全管理
│   │   │   └── monitoring.rs   # 监控系统
│   │   └── Cargo.toml
│   ├── agentx-grpc/            # gRPC插件系统
│   ├── agentx-http/            # HTTP API服务器
│   ├── agentx-cluster/         # 集群管理
│   └── agentx-sdk/             # 开发者SDK
├── plugins/                    # 插件实现
│   ├── mastra/                 # Mastra插件
│   ├── langchain/              # LangChain插件
│   └── autogen/                # AutoGen插件
├── proto/                      # Protocol Buffers定义
├── examples/                   # 示例代码
├── tests/                      # 集成测试
├── docs/                       # 文档
└── Cargo.toml                  # 工作空间配置
```

## 🔧 开发工作流

### 1. 创建新功能分支

```bash
# 从main分支创建新分支
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

### 2. 代码开发

#### 编码规范

- **命名规范**: 使用snake_case命名变量和函数，PascalCase命名类型
- **注释规范**: 使用`///`为公共API编写文档注释
- **错误处理**: 使用`Result<T, E>`类型处理错误，避免panic
- **异步编程**: 优先使用async/await，避免阻塞操作

#### 示例代码结构

```rust
//! 模块文档注释
//! 
//! 详细描述模块的功能和用途

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, error};

/// 公共结构体文档注释
#[derive(Debug, Clone)]
pub struct ExampleStruct {
    /// 字段文档注释
    pub id: String,
    pub data: HashMap<String, String>,
}

impl ExampleStruct {
    /// 构造函数文档注释
    /// 
    /// # Arguments
    /// 
    /// * `id` - 唯一标识符
    /// 
    /// # Returns
    /// 
    /// 返回新的ExampleStruct实例
    pub fn new(id: String) -> Self {
        Self {
            id,
            data: HashMap::new(),
        }
    }
    
    /// 异步方法示例
    /// 
    /// # Errors
    /// 
    /// 当操作失败时返回错误
    pub async fn process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("开始处理: {}", self.id);
        
        // 实际处理逻辑
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
}
```

### 3. 测试

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[test]
    fn test_example_struct_creation() {
        let example = ExampleStruct::new("test_id".to_string());
        assert_eq!(example.id, "test_id");
        assert!(example.data.is_empty());
    }
    
    #[tokio::test]
    async fn test_async_process() {
        let mut example = ExampleStruct::new("test_id".to_string());
        let result = example.process().await;
        assert!(result.is_ok());
    }
}
```

#### 集成测试

```rust
// tests/integration_test.rs
use agentx_core::AgentXCore;

#[tokio::test]
async fn test_core_initialization() {
    let mut core = AgentXCore::new();
    let result = core.initialize().await;
    assert!(result.is_ok());
}
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --package agentx-a2a

# 运行集成测试
cargo test --test integration_test

# 运行性能测试
cargo test --test performance_benchmarks --release
```

### 4. 代码质量检查

```bash
# 代码格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 文档生成
cargo doc --open

# 依赖检查
cargo audit
```

### 5. 提交代码

```bash
# 添加文件
git add .

# 提交代码（使用规范的提交信息）
git commit -m "feat: 添加新的A2A消息处理功能"

# 推送分支
git push origin feature/your-feature-name
```

## 🔌 插件开发

### gRPC插件开发

#### 1. 定义Protocol Buffers

```protobuf
// proto/agentx_plugin.proto
syntax = "proto3";

package agentx.plugin;

service AgentXPlugin {
  rpc RegisterAgent(RegisterAgentRequest) returns (RegisterAgentResponse);
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEvent);
}

message RegisterAgentRequest {
  AgentInfo agent = 1;
}

message RegisterAgentResponse {
  bool success = 1;
  string message = 2;
}
```

#### 2. 生成代码

```bash
# 生成Rust代码
cargo build

# 生成Python代码
python -m grpc_tools.protoc --proto_path=proto --python_out=plugins/langchain --grpc_python_out=plugins/langchain proto/agentx_plugin.proto
```

#### 3. 实现插件服务器

```rust
// Rust插件实现
use tonic::{transport::Server, Request, Response, Status};
use agentx_plugin::agent_x_plugin_server::{AgentXPlugin, AgentXPluginServer};

#[derive(Default)]
pub struct MyPlugin;

#[tonic::async_trait]
impl AgentXPlugin for MyPlugin {
    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let req = request.into_inner();
        
        // 处理Agent注册逻辑
        
        let response = RegisterAgentResponse {
            success: true,
            message: "Agent注册成功".to_string(),
        };
        
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let plugin = MyPlugin::default();
    
    Server::builder()
        .add_service(AgentXPluginServer::new(plugin))
        .serve(addr)
        .await?;
    
    Ok(())
}
```

#### 4. Python插件示例

```python
# plugins/langchain/plugin_server.py
import grpc
from concurrent import futures
import agentx_plugin_pb2_grpc as pb2_grpc
import agentx_plugin_pb2 as pb2

class LangChainPlugin(pb2_grpc.AgentXPluginServicer):
    def RegisterAgent(self, request, context):
        # 处理Agent注册
        print(f"注册Agent: {request.agent.name}")
        
        return pb2.RegisterAgentResponse(
            success=True,
            message="LangChain Agent注册成功"
        )
    
    def SendMessage(self, request, context):
        # 处理消息发送
        # 调用LangChain处理逻辑
        
        return pb2.SendMessageResponse(
            success=True,
            response="处理完成"
        )

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    pb2_grpc.add_AgentXPluginServicer_to_server(LangChainPlugin(), server)
    
    listen_addr = '[::]:50052'
    server.add_insecure_port(listen_addr)
    
    print(f"LangChain插件启动在 {listen_addr}")
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
```

## 🧪 测试策略

### 测试金字塔

```
┌─────────────────────────────────────┐
│           E2E Tests (少量)           │  ← 端到端测试
├─────────────────────────────────────┤
│       Integration Tests (中等)       │  ← 集成测试
├─────────────────────────────────────┤
│        Unit Tests (大量)            │  ← 单元测试
└─────────────────────────────────────┘
```

### 测试类型

#### 1. 单元测试
- 测试单个函数或方法
- 使用mock对象隔离依赖
- 快速执行，高覆盖率

#### 2. 集成测试
- 测试组件间的交互
- 使用真实的依赖服务
- 验证API契约

#### 3. 性能测试
- 测试系统性能指标
- 负载测试和压力测试
- 性能回归检测

#### 4. 端到端测试
- 测试完整的用户场景
- 使用真实环境
- 验证系统整体功能

### 测试工具

```toml
# Cargo.toml [dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
criterion = "0.5"
proptest = "1.0"
```

## 🚀 部署和发布

### 本地开发环境

```bash
# 启动开发服务器
cargo run --example http_server_demo

# 启动插件
cd plugins/langchain && python plugin_server.py
```

### Docker开发环境

```bash
# 构建Docker镜像
docker build -t agentx:dev .

# 启动开发环境
docker-compose -f docker-compose.dev.yml up
```

### 生产环境部署

```bash
# 构建发布版本
cargo build --release

# 创建Docker镜像
docker build -t agentx:latest .

# 部署到Kubernetes
kubectl apply -f k8s/
```

## 📊 性能优化

### 性能分析工具

```bash
# CPU性能分析
cargo install flamegraph
cargo flamegraph --example http_server_demo

# 内存分析
cargo install heaptrack
heaptrack target/release/agentx

# 基准测试
cargo bench
```

### 优化建议

1. **避免不必要的克隆**: 使用引用和借用
2. **使用合适的数据结构**: HashMap vs BTreeMap
3. **异步编程**: 避免阻塞操作
4. **内存池**: 预分配大对象
5. **批处理**: 减少系统调用次数

## 🔍 调试技巧

### 日志配置

```rust
use tracing::{info, debug, warn, error};
use tracing_subscriber;

// 初始化日志
tracing_subscriber::fmt::init();

// 使用日志
info!("系统启动");
debug!("调试信息: {}", data);
warn!("警告: 连接不稳定");
error!("错误: {}", error);
```

### 调试工具

```bash
# 使用rust-gdb调试
rust-gdb target/debug/agentx

# 使用lldb调试
rust-lldb target/debug/agentx

# 环境变量调试
RUST_LOG=debug cargo run
RUST_BACKTRACE=1 cargo run
```

## 📝 文档编写

### 代码文档

```rust
/// 计算两个数的和
/// 
/// # Arguments
/// 
/// * `a` - 第一个数
/// * `b` - 第二个数
/// 
/// # Returns
/// 
/// 返回两个数的和
/// 
/// # Examples
/// 
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### 生成文档

```bash
# 生成并打开文档
cargo doc --open

# 测试文档中的示例
cargo test --doc
```

## 🤝 贡献指南

### 提交规范

使用[Conventional Commits](https://www.conventionalcommits.org/)规范：

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

类型说明：
- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

### Pull Request流程

1. Fork项目
2. 创建功能分支
3. 编写代码和测试
4. 确保所有测试通过
5. 提交Pull Request
6. 代码审查
7. 合并到主分支

这个开发指南为AgentX项目的开发者提供了全面的指导，确保代码质量和开发效率。
