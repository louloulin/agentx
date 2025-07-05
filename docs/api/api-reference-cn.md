# AgentX API 文档

[English Version](api-reference.md) | [中文版本](api-reference-cn.md)

## 📖 概述

AgentX提供了完整的API接口，包括Rust原生API、HTTP REST API和gRPC API，支持多种编程语言和使用场景。

## 🔧 Rust API

### agentx-core

#### AgentXCore

AgentX的核心管理器，负责系统初始化和组件协调。

```rust
use agentx_core::AgentXCore;

// 创建核心实例
let mut core = AgentXCore::new();

// 初始化系统
core.initialize().await?;

// 获取系统信息
let info = core.get_system_info();
println!("AgentX版本: {}", info.version);
```

#### 错误恢复管理

```rust
use agentx_core::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryStrategy};

// 创建错误恢复管理器
let config = ErrorRecoveryConfig::default();
let manager = ErrorRecoveryManager::new(config);

// 启动管理器
manager.start().await?;

// 注册组件
manager.register_component("my_service", RecoveryStrategy::Retry).await;

// 报告错误
manager.report_error("my_service", ErrorType::Network, "连接失败", 3).await;

// 报告成功
manager.report_success("my_service", 100.0).await;
```

### agentx-a2a

#### A2A协议引擎

```rust
use agentx_a2a::{A2AProtocolEngine, ProtocolEngineConfig};

// 创建协议引擎
let config = ProtocolEngineConfig::default();
let mut engine = A2AProtocolEngine::new(config);

// 注册Agent
let agent_info = AgentInfo {
    id: "my_agent".to_string(),
    name: "我的Agent".to_string(),
    endpoint: "http://localhost:8080".to_string(),
    capabilities: vec!["分析".to_string()],
    status: AgentStatus::Online,
};
engine.register_agent(agent_info);

// 获取Agent列表
let agents = engine.list_agents();
```

#### 消息处理

```rust
use agentx_a2a::{A2AMessage, MessageRole, MessagePart};

// 创建文本消息
let message = A2AMessage::new_text(
    MessageRole::User,
    "请分析这个数据".to_string(),
);

// 创建包含文件的消息
let mut message = A2AMessage::new(MessageRole::User);
message.add_text_part("请分析附件中的数据");
message.add_file_part("data.csv", b"csv,data,here".to_vec(), "text/csv");

// 发送消息
engine.send_message(message).await?;
```

### agentx-grpc

#### 插件管理

```rust
use agentx_grpc::{PluginManager, PluginBridge};

// 创建插件桥接器
let bridge = Arc::new(PluginBridge::new());

// 创建插件管理器
let manager = PluginManager::new(bridge);

// 启动插件
manager.start_plugin("langchain_plugin").await?;

// 停止插件
manager.stop_plugin("langchain_plugin").await?;

// 获取插件状态
let status = manager.get_plugin_status("langchain_plugin").await?;
```

### agentx-cluster

#### 集群管理

```rust
use agentx_cluster::{ClusterManager, ClusterConfig};

// 创建集群管理器
let config = ClusterConfig::default();
let manager = ClusterManager::new(config).await?;

// 启动集群
manager.start().await?;

// 获取集群状态
let state = manager.get_cluster_state().await?;
println!("集群节点数: {}", state.node_count);
```

## 🌐 HTTP REST API

### 基础信息

- **基础URL**: `http://localhost:8080/api/v1`
- **认证**: Bearer Token (可选)
- **内容类型**: `application/json`

### Agent管理

#### 获取Agent列表

```http
GET /api/v1/agents
```

**响应示例**:
```json
{
  "agents": [
    {
      "id": "agent_001",
      "name": "数据分析专家",
      "endpoint": "http://localhost:8081",
      "capabilities": ["数据分析", "可视化"],
      "status": "online",
      "last_seen": "2024-01-05T10:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 10
}
```

#### 注册Agent

```http
POST /api/v1/agents
Content-Type: application/json

{
  "id": "new_agent",
  "name": "新Agent",
  "endpoint": "http://localhost:8082",
  "capabilities": ["翻译", "总结"]
}
```

#### 获取Agent详情

```http
GET /api/v1/agents/{agent_id}
```

### 消息管理

#### 发送消息

```http
POST /api/v1/messages
Content-Type: application/json

{
  "to": "agent_001",
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": "请分析这个数据"
    }
  ]
}
```

#### 获取消息历史

```http
GET /api/v1/messages?agent_id=agent_001&limit=50
```

### 任务管理

#### 创建任务

```http
POST /api/v1/tasks
Content-Type: application/json

{
  "title": "数据分析任务",
  "description": "分析销售数据并生成报告",
  "assigned_to": "agent_001",
  "priority": "high"
}
```

#### 获取任务状态

```http
GET /api/v1/tasks/{task_id}
```

**响应示例**:
```json
{
  "id": "task_123",
  "title": "数据分析任务",
  "status": "running",
  "progress": 65,
  "created_at": "2024-01-05T10:00:00Z",
  "updated_at": "2024-01-05T10:30:00Z"
}
```

### 系统监控

#### 获取系统状态

```http
GET /api/v1/system/status
```

**响应示例**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "agents_count": 5,
  "active_tasks": 12,
  "memory_usage": "128MB",
  "cpu_usage": "15%"
}
```

#### 获取性能指标

```http
GET /api/v1/system/metrics
```

## 🔌 gRPC API

### 服务定义

AgentX使用Protocol Buffers定义gRPC服务接口：

```protobuf
service AgentXPlugin {
  // Agent管理
  rpc RegisterAgent(RegisterAgentRequest) returns (RegisterAgentResponse);
  rpc UnregisterAgent(UnregisterAgentRequest) returns (UnregisterAgentResponse);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  
  // 消息处理
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEvent);
  
  // 任务管理
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
  rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);
}
```

### 客户端示例

#### Python客户端

```python
import grpc
from agentx_pb2 import RegisterAgentRequest, AgentInfo
from agentx_pb2_grpc import AgentXPluginStub

# 连接到AgentX服务
channel = grpc.insecure_channel('localhost:50051')
client = AgentXPluginStub(channel)

# 注册Agent
request = RegisterAgentRequest(
    agent=AgentInfo(
        id="python_agent",
        name="Python分析Agent",
        capabilities=["数据分析", "机器学习"]
    )
)
response = client.RegisterAgent(request)
```

#### Node.js客户端

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// 加载proto文件
const packageDefinition = protoLoader.loadSync('agentx.proto');
const agentx = grpc.loadPackageDefinition(packageDefinition).agentx;

// 创建客户端
const client = new agentx.AgentXPlugin('localhost:50051', 
  grpc.credentials.createInsecure());

// 注册Agent
const request = {
  agent: {
    id: 'nodejs_agent',
    name: 'Node.js Agent',
    capabilities: ['web_scraping', 'api_integration']
  }
};

client.RegisterAgent(request, (error, response) => {
  if (error) {
    console.error('注册失败:', error);
  } else {
    console.log('注册成功:', response);
  }
});
```

## 📊 错误处理

### 错误代码

| 代码 | 名称 | 描述 |
|------|------|------|
| 1000 | `AGENT_NOT_FOUND` | Agent不存在 |
| 1001 | `AGENT_OFFLINE` | Agent离线 |
| 1002 | `INVALID_MESSAGE` | 消息格式无效 |
| 1003 | `TASK_NOT_FOUND` | 任务不存在 |
| 1004 | `PERMISSION_DENIED` | 权限不足 |
| 1005 | `RATE_LIMIT_EXCEEDED` | 请求频率超限 |

### 错误响应格式

```json
{
  "error": {
    "code": 1000,
    "message": "Agent not found",
    "details": "Agent with ID 'unknown_agent' does not exist"
  }
}
```

## 🔐 认证和授权

### API密钥认证

```http
Authorization: Bearer your-api-key-here
```

### JWT认证

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 📈 限流和配额

- **默认限制**: 每分钟1000请求
- **Agent注册**: 每小时10个新Agent
- **消息发送**: 每秒100条消息
- **文件上传**: 单文件最大10MB

## 🔍 调试和测试

### 健康检查

```http
GET /api/v1/health
```

### API测试工具

推荐使用以下工具测试AgentX API：

- **curl**: 命令行HTTP客户端
- **Postman**: 图形化API测试工具
- **grpcurl**: gRPC命令行客户端
- **BloomRPC**: gRPC图形化测试工具

### 示例curl命令

```bash
# 获取系统状态
curl -X GET http://localhost:8080/api/v1/system/status

# 发送消息
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "to": "agent_001",
    "role": "user",
    "content": [{"type": "text", "text": "Hello"}]
  }'
```
