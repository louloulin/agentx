# AgentX HTTP API Server

AgentX项目的HTTP/REST API服务器，基于Axum框架构建，提供完整的A2A协议HTTP接口。

## 🚀 特性

- **高性能**: 基于Axum和Tokio的异步架构
- **RESTful设计**: 符合REST原则的API设计
- **OpenAPI 3.0**: 自动生成的API文档和Swagger UI
- **类型安全**: 基于serde的请求/响应验证
- **中间件支持**: CORS、认证、日志、压缩等完整中间件栈
- **配置灵活**: 支持环境变量和配置文件
- **错误处理**: 标准化的错误响应格式

## 📋 API端点

### 任务管理
- `POST /api/v1/tasks` - 创建新任务
- `GET /api/v1/tasks` - 获取任务列表
- `GET /api/v1/tasks/{task_id}` - 获取任务详情
- `POST /api/v1/tasks/{task_id}/cancel` - 取消任务

### 消息管理
- `POST /api/v1/messages` - 发送消息
- `GET /api/v1/messages/{message_id}` - 获取消息详情
- `GET /api/v1/tasks/{task_id}/messages` - 获取任务消息历史

### Agent管理
- `POST /api/v1/agents` - 注册Agent
- `GET /api/v1/agents` - 获取Agent列表
- `GET /api/v1/agents/{agent_id}` - 获取Agent详情
- `DELETE /api/v1/agents/{agent_id}` - 注销Agent
- `GET /api/v1/agents/capabilities` - 获取系统能力

### 健康检查
- `GET /health` - 健康检查
- `GET /ready` - 就绪检查
- `GET /live` - 存活检查

### 文档
- `GET /docs` - Swagger UI文档
- `GET /api-docs/openapi.json` - OpenAPI规范

## 🔧 使用方法

### 启动服务器

```bash
# 使用默认配置启动
cargo run --bin agentx-http

# 使用环境变量配置
AGENTX_HTTP_PORT=8080 AGENTX_HTTP_HOST=0.0.0.0 cargo run --bin agentx-http
```

### 编程方式启动

```rust
use agentx_http::{config::AppConfig, server::start_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::default();
    start_server(config).await
}
```

### 配置选项

```rust
use agentx_http::config::HttpServerConfig;

let config = HttpServerConfig {
    host: "0.0.0.0".to_string(),
    port: 8080,
    enable_cors: true,
    enable_docs: true,
    enable_compression: true,
    request_timeout: 30,
    max_request_size: 10 * 1024 * 1024, // 10MB
    ..Default::default()
};
```

## 📝 API使用示例

### 创建任务

```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "kind": "text_generation",
    "context_id": "my_context",
    "initial_message": {
      "role": "user",
      "content": {
        "type": "text",
        "text": "请生成一首诗"
      }
    }
  }'
```

### 注册Agent

```bash
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my_agent",
    "name": "我的AI Agent",
    "endpoint": "http://localhost:8081",
    "capabilities": ["text_generation", "translation"],
    "status": "online"
  }'
```

### 发送消息

```bash
curl -X POST http://localhost:8080/api/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "role": "user",
    "content": {
      "type": "text",
      "text": "Hello, AI!"
    },
    "task_id": "task_123"
  }'
```

## 🔒 认证

支持API密钥认证：

```bash
# 设置API密钥环境变量
export AGENTX_API_KEY="your-secret-key"

# 在请求中包含认证头
curl -H "Authorization: Bearer your-secret-key" \
  http://localhost:8080/api/v1/tasks
```

## 🌍 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `AGENTX_HTTP_HOST` | 监听地址 | `0.0.0.0` |
| `AGENTX_HTTP_PORT` | 监听端口 | `8080` |
| `AGENTX_HTTP_ENABLE_CORS` | 启用CORS | `true` |
| `AGENTX_HTTP_ENABLE_DOCS` | 启用文档 | `true` |
| `AGENTX_HTTP_REQUEST_TIMEOUT` | 请求超时(秒) | `30` |
| `AGENTX_HTTP_MAX_REQUEST_SIZE` | 最大请求大小(字节) | `10485760` |
| `AGENTX_API_KEY` | API密钥 | 无 |

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行HTTP API测试
cargo test --test simple_api_tests

# 运行演示
cargo run --example http_server_demo
```

## 📊 性能

- **并发处理**: 支持数千个并发连接
- **响应时间**: 平均响应时间 < 10ms
- **吞吐量**: 支持高吞吐量请求处理
- **内存使用**: 优化的内存使用和垃圾回收

## 🔧 开发

### 添加新的API端点

1. 在 `src/handlers/` 中创建处理器
2. 在 `src/models.rs` 中定义请求/响应模型
3. 在 `src/server.rs` 中注册路由
4. 在 `src/docs.rs` 中添加OpenAPI文档

### 添加中间件

```rust
use axum::middleware;

let app = Router::new()
    .route("/api/v1/example", get(example_handler))
    .layer(middleware::from_fn(custom_middleware));
```

## 📄 许可证

本项目采用MIT许可证 - 查看 [LICENSE](../../LICENSE) 文件了解详情。

## 🤝 贡献

欢迎提交Issue和Pull Request来改进这个实现。

## 📞 支持

如有问题或建议，请在GitHub上创建Issue。
