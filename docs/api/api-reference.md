# AgentX API Documentation

[English Version](api-reference.md) | [中文版本](api-reference-cn.md)

## 📖 Overview

AgentX provides comprehensive API interfaces including native Rust API, HTTP REST API, and gRPC API, supporting multiple programming languages and use cases.

## 🔧 Rust API

### agentx-core

#### AgentXCore

The core manager of AgentX, responsible for system initialization and component coordination.

```rust
use agentx_core::AgentXCore;

// Create core instance
let mut core = AgentXCore::new();

// Initialize system
core.initialize().await?;

// Get system information
let info = core.get_system_info();
println!("AgentX version: {}", info.version);
```

#### Error Recovery Management

```rust
use agentx_core::{ErrorRecoveryManager, ErrorRecoveryConfig, RecoveryStrategy};

// Create error recovery manager
let config = ErrorRecoveryConfig::default();
let manager = ErrorRecoveryManager::new(config);

// Start manager
manager.start().await?;

// Register component
manager.register_component("my_service", RecoveryStrategy::Retry).await;

// Report error
manager.report_error("my_service", ErrorType::Network, "Connection failed", 3).await;

// Report success
manager.report_success("my_service", 100.0).await;
```

### agentx-a2a

#### A2A Protocol Engine

```rust
use agentx_a2a::{A2AProtocolEngine, ProtocolEngineConfig};

// Create protocol engine
let config = ProtocolEngineConfig::default();
let mut engine = A2AProtocolEngine::new(config);

// Register agent
let agent_info = AgentInfo {
    id: "my_agent".to_string(),
    name: "My Agent".to_string(),
    endpoint: "http://localhost:8080".to_string(),
    capabilities: vec!["analysis".to_string()],
    status: AgentStatus::Online,
};
engine.register_agent(agent_info);

// List agents
let agents = engine.list_agents();
```

#### Message Handling

```rust
use agentx_a2a::{A2AMessage, MessageRole, MessagePart};

// Create text message
let message = A2AMessage::new_text(
    MessageRole::User,
    "Please analyze this data".to_string(),
);

// Create message with file
let mut message = A2AMessage::new(MessageRole::User);
message.add_text_part("Please analyze the attached data");
message.add_file_part("data.csv", b"csv,data,here".to_vec(), "text/csv");

// Send message
engine.send_message(message).await?;
```

### agentx-grpc

#### Plugin Management

```rust
use agentx_grpc::{PluginManager, PluginBridge};

// Create plugin bridge
let bridge = Arc::new(PluginBridge::new());

// Create plugin manager
let manager = PluginManager::new(bridge);

// Start plugin
manager.start_plugin("langchain_plugin").await?;

// Stop plugin
manager.stop_plugin("langchain_plugin").await?;

// Get plugin status
let status = manager.get_plugin_status("langchain_plugin").await?;
```

### agentx-cluster

#### Cluster Management

```rust
use agentx_cluster::{ClusterManager, ClusterConfig};

// Create cluster manager
let config = ClusterConfig::default();
let manager = ClusterManager::new(config).await?;

// Start cluster
manager.start().await?;

// Get cluster state
let state = manager.get_cluster_state().await?;
println!("Cluster nodes: {}", state.node_count);
```

## 🌐 HTTP REST API

### Basic Information

- **Base URL**: `http://localhost:8080/api/v1`
- **Authentication**: Bearer Token (optional)
- **Content Type**: `application/json`

### Agent Management

#### List Agents

```http
GET /api/v1/agents
```

**Response Example**:
```json
{
  "agents": [
    {
      "id": "agent_001",
      "name": "Data Analysis Expert",
      "endpoint": "http://localhost:8081",
      "capabilities": ["data_analysis", "visualization"],
      "status": "online",
      "last_seen": "2024-01-05T10:30:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 10
}
```

#### Register Agent

```http
POST /api/v1/agents
Content-Type: application/json

{
  "id": "new_agent",
  "name": "New Agent",
  "endpoint": "http://localhost:8082",
  "capabilities": ["translation", "summarization"]
}
```

#### Get Agent Details

```http
GET /api/v1/agents/{agent_id}
```

### Message Management

#### Send Message

```http
POST /api/v1/messages
Content-Type: application/json

{
  "to": "agent_001",
  "role": "user",
  "content": [
    {
      "type": "text",
      "text": "Please analyze this data"
    }
  ]
}
```

#### Get Message History

```http
GET /api/v1/messages?agent_id=agent_001&limit=50
```

### Task Management

#### Create Task

```http
POST /api/v1/tasks
Content-Type: application/json

{
  "title": "Data Analysis Task",
  "description": "Analyze sales data and generate report",
  "assigned_to": "agent_001",
  "priority": "high"
}
```

#### Get Task Status

```http
GET /api/v1/tasks/{task_id}
```

**Response Example**:
```json
{
  "id": "task_123",
  "title": "Data Analysis Task",
  "status": "running",
  "progress": 65,
  "created_at": "2024-01-05T10:00:00Z",
  "updated_at": "2024-01-05T10:30:00Z"
}
```

### System Monitoring

#### Get System Status

```http
GET /api/v1/system/status
```

**Response Example**:
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

#### Get Performance Metrics

```http
GET /api/v1/system/metrics
```

## 🔌 gRPC API

### Service Definition

AgentX uses Protocol Buffers to define gRPC service interfaces:

```protobuf
service AgentXPlugin {
  // Agent management
  rpc RegisterAgent(RegisterAgentRequest) returns (RegisterAgentResponse);
  rpc UnregisterAgent(UnregisterAgentRequest) returns (UnregisterAgentResponse);
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse);
  
  // Message handling
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEvent);
  
  // Task management
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
  rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);
}
```

### Client Examples

#### Python Client

```python
import grpc
from agentx_pb2 import RegisterAgentRequest, AgentInfo
from agentx_pb2_grpc import AgentXPluginStub

# Connect to AgentX service
channel = grpc.insecure_channel('localhost:50051')
client = AgentXPluginStub(channel)

# Register agent
request = RegisterAgentRequest(
    agent=AgentInfo(
        id="python_agent",
        name="Python Analysis Agent",
        capabilities=["data_analysis", "machine_learning"]
    )
)
response = client.RegisterAgent(request)
```

#### Node.js Client

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// Load proto file
const packageDefinition = protoLoader.loadSync('agentx.proto');
const agentx = grpc.loadPackageDefinition(packageDefinition).agentx;

// Create client
const client = new agentx.AgentXPlugin('localhost:50051', 
  grpc.credentials.createInsecure());

// Register agent
const request = {
  agent: {
    id: 'nodejs_agent',
    name: 'Node.js Agent',
    capabilities: ['web_scraping', 'api_integration']
  }
};

client.RegisterAgent(request, (error, response) => {
  if (error) {
    console.error('Registration failed:', error);
  } else {
    console.log('Registration successful:', response);
  }
});
```

## 📊 Error Handling

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1000 | `AGENT_NOT_FOUND` | Agent does not exist |
| 1001 | `AGENT_OFFLINE` | Agent is offline |
| 1002 | `INVALID_MESSAGE` | Invalid message format |
| 1003 | `TASK_NOT_FOUND` | Task does not exist |
| 1004 | `PERMISSION_DENIED` | Insufficient permissions |
| 1005 | `RATE_LIMIT_EXCEEDED` | Request rate limit exceeded |

### Error Response Format

```json
{
  "error": {
    "code": 1000,
    "message": "Agent not found",
    "details": "Agent with ID 'unknown_agent' does not exist"
  }
}
```

## 🔐 Authentication and Authorization

### API Key Authentication

```http
Authorization: Bearer your-api-key-here
```

### JWT Authentication

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 📈 Rate Limiting and Quotas

- **Default Limit**: 1000 requests per minute
- **Agent Registration**: 10 new agents per hour
- **Message Sending**: 100 messages per second
- **File Upload**: Maximum 10MB per file

## 🔍 Debugging and Testing

### Health Check

```http
GET /api/v1/health
```

### API Testing Tools

Recommended tools for testing AgentX API:

- **curl**: Command-line HTTP client
- **Postman**: Graphical API testing tool
- **grpcurl**: gRPC command-line client
- **BloomRPC**: Graphical gRPC testing tool

### Example curl Commands

```bash
# Get system status
curl -X GET http://localhost:8080/api/v1/system/status

# Send message
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "to": "agent_001",
    "role": "user",
    "content": [{"type": "text", "text": "Hello"}]
  }'
```
