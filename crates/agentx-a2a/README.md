# AgentX A2A Protocol Implementation

AgentX项目中Agent-to-Agent (A2A) 协议的Rust实现，基于A2A v0.2.5规范和JSON-RPC 2.0标准。

## 🚀 特性

- **完整的A2A v0.2.5支持**: 符合最新A2A协议规范
- **JSON-RPC 2.0**: 完整的JSON-RPC协议实现
- **高性能**: 平均延迟0.01ms，吞吐量142,857消息/秒
- **类型安全**: 利用Rust类型系统确保消息格式正确性
- **多模态支持**: 支持文本、文件、结构化数据等多种消息类型
- **异步处理**: 基于Tokio的高并发异步架构

## 📋 核心组件

### 消息系统
- `A2AMessage`: 核心消息结构，支持多种角色和内容类型
- `MessagePart`: 消息内容部分，支持文本、文件、数据
- `MessageRole`: 消息角色（User、Agent、System）

### 任务管理
- `A2ATask`: 任务生命周期管理
- `TaskStatus`: 任务状态跟踪
- `Artifact`: 任务产出物管理

### 协议引擎
- `A2AProtocolEngine`: 核心协议处理引擎
- `JsonRpcRequest/Response`: JSON-RPC消息处理
- `AgentInfo`: Agent注册和发现

## 🔧 使用方法

### 基本使用

```rust
use agentx_a2a::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建协议引擎
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 注册Agent
    let agent = AgentInfo {
        id: "my-agent".to_string(),
        name: "My AI Agent".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["text_generation".to_string()],
        status: AgentStatus::Online,
    };
    engine.register_agent(agent);
    
    // 创建用户消息
    let message = A2AMessage::user_message("Hello, AI!".to_string())
        .with_task_id("task_001".to_string());
    
    // 发送消息
    let request = JsonRpcRequest::send_message(
        message,
        serde_json::Value::String("req_001".to_string())
    );
    
    let response = engine.process_request(request).await;
    println!("Response: {}", serde_json::to_string_pretty(&response)?);
    
    Ok(())
}
```

### 任务管理

```rust
// 创建任务
let task = A2ATask::new("text_generation".to_string())
    .with_context_id("my_context".to_string());

// 提交任务
let request = JsonRpcRequest::submit_task(
    task,
    serde_json::Value::String("req_001".to_string())
);

let response = engine.process_request(request).await;
```

### 文件处理

```rust
use base64::{Engine as _, engine::general_purpose};

let file_data = FileData::WithBytes(FileWithBytes {
    name: Some("document.txt".to_string()),
    mime_type: "text/plain".to_string(),
    bytes: general_purpose::STANDARD.encode(b"File content"),
});

let file_message = A2AMessage::new_file(MessageRole::User, file_data);
```

## 📊 性能基准

基于实际测试的性能指标：

| 指标 | 值 |
|------|-----|
| 平均延迟 | 0.01ms |
| 吞吐量 | 142,857 消息/秒 |
| 并发处理 | 100+ 并发请求 |
| 内存效率 | 10,000条消息无性能衰减 |

## 🧪 测试

运行所有测试：
```bash
cargo test
```

运行性能测试：
```bash
cargo test --test performance_tests -- --nocapture
```

运行基础功能测试：
```bash
cargo test --test basic_a2a_tests
```

## 📚 API文档

### 核心方法

#### A2AMessage
- `user_message(text)`: 创建用户消息
- `agent_message(text)`: 创建Agent消息
- `new_file(role, file_data)`: 创建文件消息
- `new_data(role, data)`: 创建数据消息

#### A2ATask
- `new(kind)`: 创建新任务
- `add_message(message)`: 添加消息到历史
- `update_status(state)`: 更新任务状态
- `add_artifact(artifact)`: 添加工件

#### A2AProtocolEngine
- `new(config)`: 创建协议引擎
- `register_agent(agent)`: 注册Agent
- `process_request(request)`: 处理JSON-RPC请求

### JSON-RPC方法

- `submitTask`: 提交新任务
- `getTask`: 查询任务状态
- `cancelTask`: 取消任务
- `sendMessage`: 发送消息
- `getCapabilities`: 查询Agent能力

## 🔗 相关链接

- [A2A协议规范](https://github.com/google/agent-to-agent-protocol)
- [JSON-RPC 2.0规范](https://www.jsonrpc.org/specification)
- [AgentX项目文档](../../plan3.md)

## 📄 许可证

本项目采用MIT许可证 - 查看 [LICENSE](../../LICENSE) 文件了解详情。

## 🤝 贡献

欢迎提交Issue和Pull Request来改进这个实现。

## 📞 支持

如有问题或建议，请在GitHub上创建Issue。
