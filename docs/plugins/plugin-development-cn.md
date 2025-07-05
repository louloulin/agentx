# AgentX 插件开发指南

## 📖 概述

AgentX采用gRPC插件架构，支持多种编程语言开发插件。本指南详细介绍如何为不同AI框架开发AgentX插件。

[English Version](plugin-development.md) | [中文版本](plugin-development-cn.md)

## 🏗️ 插件架构

### 设计原则

- **进程隔离**: 每个插件运行在独立进程中，确保故障隔离
- **标准接口**: 统一的gRPC服务定义，支持多语言实现
- **热插拔**: 支持运行时动态加载和卸载插件
- **框架无关**: 不依赖特定AI框架，通过适配器模式集成

### 插件通信流程

```
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Communication                     │
│                                                             │
│  AgentX Core     gRPC Bridge      Plugin Process           │
│      │               │                   │                 │
│      │ 1. Request    │                   │                 │
│      │──────────────►│                   │                 │
│      │               │ 2. gRPC Call      │                 │
│      │               │──────────────────►│                 │
│      │               │                   │ 3. Process      │
│      │               │                   │                 │
│      │               │ 4. Response       │                 │
│      │               │◄──────────────────│                 │
│      │ 5. Result     │                   │                 │
│      │◄──────────────│                   │                 │
└─────────────────────────────────────────────────────────────┘
```

## 🔌 gRPC服务定义

### 核心服务接口

```protobuf
// proto/agentx_plugin.proto
syntax = "proto3";

package agentx.plugin;

// 主要插件服务
service AgentXPlugin {
  // Agent管理
  rpc RegisterAgent(RegisterAgentRequest) returns (RegisterAgentResponse);
  rpc UnregisterAgent(UnregisterAgentRequest) returns (UnregisterAgentResponse);
  rpc GetAgentStatus(GetAgentStatusRequest) returns (GetAgentStatusResponse);

  // 消息处理
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEvent);

  // 任务管理
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
  rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);

  // 能力查询
  rpc GetCapabilities(GetCapabilitiesRequest) returns (GetCapabilitiesResponse);
  rpc QueryAgents(QueryAgentsRequest) returns (QueryAgentsResponse);
}

// Agent信息
message AgentInfo {
  string id = 1;
  string name = 2;
  string description = 3;
  repeated string capabilities = 4;
  string endpoint = 5;
  AgentStatus status = 6;
  map<string, string> metadata = 7;
}

// Agent状态
enum AgentStatus {
  UNKNOWN = 0;
  ONLINE = 1;
  OFFLINE = 2;
  BUSY = 3;
  ERROR = 4;
}

// 消息定义
message A2AMessage {
  string message_id = 1;
  string from = 2;
  string to = 3;
  MessageRole role = 4;
  repeated MessagePart content = 5;
  map<string, string> metadata = 6;
  int64 timestamp = 7;
}

enum MessageRole {
  USER = 0;
  AGENT = 1;
  SYSTEM = 2;
}

message MessagePart {
  oneof content {
    string text = 1;
    FileData file = 2;
    bytes data = 3;
  }
  string mime_type = 4;
  map<string, string> metadata = 5;
}

message FileData {
  string filename = 1;
  bytes content = 2;
  string mime_type = 3;
  int64 size = 4;
}
```

## 🐍 Python插件开发

### 环境准备

```bash
# 安装依赖
pip install grpcio grpcio-tools

# 生成Python代码
python -m grpc_tools.protoc \
  --proto_path=proto \
  --python_out=plugins/python \
  --grpc_python_out=plugins/python \
  proto/agentx_plugin.proto
```

### LangChain插件示例

```python
# plugins/langchain/langchain_plugin.py
import grpc
from concurrent import futures
import logging
from typing import Dict, List, Optional

import agentx_plugin_pb2 as pb2
import agentx_plugin_pb2_grpc as pb2_grpc

from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain.tools import Tool
from langchain_openai import ChatOpenAI
from langchain.prompts import ChatPromptTemplate

class LangChainPlugin(pb2_grpc.AgentXPluginServicer):
    def __init__(self):
        self.agents: Dict[str, AgentExecutor] = {}
        self.llm = ChatOpenAI(temperature=0)

    def RegisterAgent(self, request, context):
        """注册LangChain Agent"""
        try:
            agent_info = request.agent

            # 创建LangChain Agent
            tools = self._create_tools(agent_info.capabilities)
            prompt = ChatPromptTemplate.from_messages([
                ("system", f"You are {agent_info.name}. {agent_info.description}"),
                ("user", "{input}"),
                ("assistant", "{agent_scratchpad}"),
            ])

            agent = create_openai_functions_agent(self.llm, tools, prompt)
            agent_executor = AgentExecutor(agent=agent, tools=tools)

            self.agents[agent_info.id] = agent_executor

            logging.info(f"Registered LangChain agent: {agent_info.name}")

            return pb2.RegisterAgentResponse(
                success=True,
                message=f"Agent {agent_info.name} registered successfully"
            )

        except Exception as e:
            logging.error(f"Failed to register agent: {e}")
            return pb2.RegisterAgentResponse(
                success=False,
                message=f"Registration failed: {str(e)}"
            )

    def SendMessage(self, request, context):
        """处理消息发送"""
        try:
            message = request.message
            agent_id = message.to

            if agent_id not in self.agents:
                return pb2.SendMessageResponse(
                    success=False,
                    message="Agent not found"
                )

            # 提取文本内容
            text_content = ""
            for part in message.content:
                if part.HasField('text'):
                    text_content += part.text + " "

            # 执行LangChain Agent
            agent_executor = self.agents[agent_id]
            result = agent_executor.invoke({"input": text_content.strip()})

            # 构造响应消息
            response_message = pb2.A2AMessage(
                message_id=f"resp_{message.message_id}",
                from_=agent_id,
                to=message.from_,
                role=pb2.MessageRole.AGENT,
                content=[
                    pb2.MessagePart(text=result["output"])
                ]
            )

            return pb2.SendMessageResponse(
                success=True,
                message="Message processed successfully",
                response=response_message
            )

        except Exception as e:
            logging.error(f"Failed to process message: {e}")
            return pb2.SendMessageResponse(
                success=False,
                message=f"Processing failed: {str(e)}"
            )

    def GetCapabilities(self, request, context):
        """获取插件能力"""
        capabilities = [
            "text_processing",
            "question_answering",
            "tool_usage",
            "chain_of_thought",
            "function_calling"
        ]

        return pb2.GetCapabilitiesResponse(
            capabilities=capabilities,
            framework="langchain",
            version="0.1.0"
        )

    def _create_tools(self, capabilities: List[str]) -> List[Tool]:
        """根据能力创建工具"""
        tools = []

        if "web_search" in capabilities:
            tools.append(Tool(
                name="web_search",
                description="Search the web for information",
                func=self._web_search
            ))

        if "calculator" in capabilities:
            tools.append(Tool(
                name="calculator",
                description="Perform mathematical calculations",
                func=self._calculate
            ))

        return tools

    def _web_search(self, query: str) -> str:
        """Web搜索工具"""
        # 实现web搜索逻辑
        return f"Search results for: {query}"

    def _calculate(self, expression: str) -> str:
        """计算器工具"""
        try:
            result = eval(expression)
            return str(result)
        except:
            return "Invalid expression"

def serve():
    """启动插件服务器"""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    pb2_grpc.add_AgentXPluginServicer_to_server(LangChainPlugin(), server)

    listen_addr = '[::]:50052'
    server.add_insecure_port(listen_addr)

    logging.info(f"LangChain plugin server started on {listen_addr}")
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    serve()
```

### AutoGen插件示例

```python
# plugins/autogen/autogen_plugin.py
import grpc
from concurrent import futures
import logging
from typing import Dict, List

import agentx_plugin_pb2 as pb2
import agentx_plugin_pb2_grpc as pb2_grpc

import autogen
from autogen import AssistantAgent, UserProxyAgent, GroupChat, GroupChatManager

class AutoGenPlugin(pb2_grpc.AgentXPluginServicer):
    def __init__(self):
        self.agents: Dict[str, autogen.Agent] = {}
        self.group_chats: Dict[str, GroupChat] = {}

        # AutoGen配置
        self.config_list = [
            {
                "model": "gpt-4",
                "api_key": "your-openai-api-key"
            }
        ]

    def RegisterAgent(self, request, context):
        """注册AutoGen Agent"""
        try:
            agent_info = request.agent

            # 根据能力创建不同类型的Agent
            if "assistant" in agent_info.capabilities:
                agent = AssistantAgent(
                    name=agent_info.name,
                    system_message=agent_info.description,
                    llm_config={"config_list": self.config_list}
                )
            elif "user_proxy" in agent_info.capabilities:
                agent = UserProxyAgent(
                    name=agent_info.name,
                    system_message=agent_info.description,
                    code_execution_config={"work_dir": "workspace"}
                )
            else:
                # 默认创建AssistantAgent
                agent = AssistantAgent(
                    name=agent_info.name,
                    system_message=agent_info.description,
                    llm_config={"config_list": self.config_list}
                )

            self.agents[agent_info.id] = agent

            logging.info(f"Registered AutoGen agent: {agent_info.name}")

            return pb2.RegisterAgentResponse(
                success=True,
                message=f"Agent {agent_info.name} registered successfully"
            )

        except Exception as e:
            logging.error(f"Failed to register agent: {e}")
            return pb2.RegisterAgentResponse(
                success=False,
                message=f"Registration failed: {str(e)}"
            )

    def SendMessage(self, request, context):
        """处理消息发送"""
        try:
            message = request.message
            agent_id = message.to

            if agent_id not in self.agents:
                return pb2.SendMessageResponse(
                    success=False,
                    message="Agent not found"
                )

            # 提取文本内容
            text_content = ""
            for part in message.content:
                if part.HasField('text'):
                    text_content += part.text + " "

            # 使用AutoGen处理消息
            agent = self.agents[agent_id]

            # 创建临时对话
            user_proxy = UserProxyAgent(
                name="temp_user",
                human_input_mode="NEVER",
                code_execution_config=False
            )

            # 发起对话
            user_proxy.initiate_chat(agent, message=text_content.strip())

            # 获取最后的回复
            last_message = agent.last_message()
            response_text = last_message.get("content", "No response")

            # 构造响应消息
            response_message = pb2.A2AMessage(
                message_id=f"resp_{message.message_id}",
                from_=agent_id,
                to=message.from_,
                role=pb2.MessageRole.AGENT,
                content=[
                    pb2.MessagePart(text=response_text)
                ]
            )

            return pb2.SendMessageResponse(
                success=True,
                message="Message processed successfully",
                response=response_message
            )

        except Exception as e:
            logging.error(f"Failed to process message: {e}")
            return pb2.SendMessageResponse(
                success=False,
                message=f"Processing failed: {str(e)}"
            )

    def CreateTask(self, request, context):
        """创建多Agent任务"""
        try:
            task = request.task

            # 创建群聊
            agents_list = []
            for agent_id in task.assigned_agents:
                if agent_id in self.agents:
                    agents_list.append(self.agents[agent_id])

            if len(agents_list) < 2:
                return pb2.CreateTaskResponse(
                    success=False,
                    message="Need at least 2 agents for group chat"
                )

            # 创建群聊管理器
            group_chat = GroupChat(
                agents=agents_list,
                messages=[],
                max_round=10
            )

            manager = GroupChatManager(
                groupchat=group_chat,
                llm_config={"config_list": self.config_list}
            )

            self.group_chats[task.id] = group_chat

            # 启动群聊
            agents_list[0].initiate_chat(manager, message=task.description)

            return pb2.CreateTaskResponse(
                success=True,
                task_id=task.id,
                message="Group chat task created successfully"
            )

        except Exception as e:
            logging.error(f"Failed to create task: {e}")
            return pb2.CreateTaskResponse(
                success=False,
                message=f"Task creation failed: {str(e)}"
            )

def serve():
    """启动插件服务器"""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    pb2_grpc.add_AgentXPluginServicer_to_server(AutoGenPlugin(), server)

    listen_addr = '[::]:50053'
    server.add_insecure_port(listen_addr)

    logging.info(f"AutoGen plugin server started on {listen_addr}")
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    logging.basicConfig(level=logging.INFO)
    serve()
```

## 🟨 Node.js插件开发

### 环境准备

```bash
# 初始化Node.js项目
npm init -y

# 安装依赖
npm install @grpc/grpc-js @grpc/proto-loader

# 生成TypeScript类型定义（可选）
npm install -D @types/node typescript
```

### Mastra插件示例

```javascript
// plugins/mastra/mastra_plugin.js
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const { Mastra } = require('@mastra/core');

// 加载proto文件
const packageDefinition = protoLoader.loadSync('../../proto/agentx_plugin.proto', {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true
});

const agentxProto = grpc.loadPackageDefinition(packageDefinition).agentx.plugin;

class MastraPlugin {
  constructor() {
    this.agents = new Map();
    this.mastra = new Mastra({
      name: 'agentx-mastra-plugin',
      version: '1.0.0'
    });
  }

  async registerAgent(call, callback) {
    try {
      const { agent } = call.request;

      // 创建Mastra Agent
      const mastraAgent = this.mastra.agent({
        name: agent.name,
        instructions: agent.description,
        model: {
          provider: 'OPEN_AI',
          name: 'gpt-4'
        },
        tools: this.createTools(agent.capabilities)
      });

      this.agents.set(agent.id, mastraAgent);

      console.log(`Registered Mastra agent: ${agent.name}`);

      callback(null, {
        success: true,
        message: `Agent ${agent.name} registered successfully`
      });

    } catch (error) {
      console.error('Failed to register agent:', error);
      callback(null, {
        success: false,
        message: `Registration failed: ${error.message}`
      });
    }
  }

  async sendMessage(call, callback) {
    try {
      const { message } = call.request;
      const agentId = message.to;

      if (!this.agents.has(agentId)) {
        callback(null, {
          success: false,
          message: 'Agent not found'
        });
        return;
      }

      // 提取文本内容
      let textContent = '';
      for (const part of message.content) {
        if (part.text) {
          textContent += part.text + ' ';
        }
      }

      // 使用Mastra处理消息
      const agent = this.agents.get(agentId);
      const result = await agent.generate(textContent.trim());

      // 构造响应消息
      const responseMessage = {
        messageId: `resp_${message.messageId}`,
        from: agentId,
        to: message.from,
        role: 'AGENT',
        content: [{
          text: result.text
        }],
        timestamp: Date.now()
      };

      callback(null, {
        success: true,
        message: 'Message processed successfully',
        response: responseMessage
      });

    } catch (error) {
      console.error('Failed to process message:', error);
      callback(null, {
        success: false,
        message: `Processing failed: ${error.message}`
      });
    }
  }

  async getCapabilities(call, callback) {
    const capabilities = [
      'text_generation',
      'conversation',
      'tool_usage',
      'workflow_execution',
      'memory_management'
    ];

    callback(null, {
      capabilities,
      framework: 'mastra',
      version: '1.0.0'
    });
  }

  createTools(capabilities) {
    const tools = [];

    if (capabilities.includes('web_search')) {
      tools.push({
        name: 'web_search',
        description: 'Search the web for information',
        parameters: {
          type: 'object',
          properties: {
            query: {
              type: 'string',
              description: 'Search query'
            }
          },
          required: ['query']
        },
        execute: async ({ query }) => {
          // 实现web搜索逻辑
          return `Search results for: ${query}`;
        }
      });
    }

    if (capabilities.includes('file_operations')) {
      tools.push({
        name: 'read_file',
        description: 'Read file contents',
        parameters: {
          type: 'object',
          properties: {
            filename: {
              type: 'string',
              description: 'File path to read'
            }
          },
          required: ['filename']
        },
        execute: async ({ filename }) => {
          const fs = require('fs').promises;
          try {
            const content = await fs.readFile(filename, 'utf8');
            return content;
          } catch (error) {
            return `Error reading file: ${error.message}`;
          }
        }
      });
    }

    return tools;
  }
}

function serve() {
  const server = new grpc.Server();
  const plugin = new MastraPlugin();

  server.addService(agentxProto.AgentXPlugin.service, {
    registerAgent: plugin.registerAgent.bind(plugin),
    sendMessage: plugin.sendMessage.bind(plugin),
    getCapabilities: plugin.getCapabilities.bind(plugin)
  });

  const port = '0.0.0.0:50054';
  server.bindAsync(port, grpc.ServerCredentials.createInsecure(), (err, port) => {
    if (err) {
      console.error('Failed to start server:', err);
      return;
    }
    console.log(`Mastra plugin server started on ${port}`);
    server.start();
  });
}

if (require.main === module) {
  serve();
}

module.exports = { MastraPlugin };
```

## 🦀 Rust插件开发

### 环境准备

```toml
# Cargo.toml
[package]
name = "agentx-rust-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
tonic = "0.10"
prost = "0.12"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
tonic-build = "0.10"
```

### 自定义框架插件示例

```rust
// src/lib.rs
use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

// 生成的protobuf代码
pub mod agentx_plugin {
    tonic::include_proto!("agentx.plugin");
}

use agentx_plugin::{
    agent_x_plugin_server::{AgentXPlugin, AgentXPluginServer},
    *,
};

#[derive(Debug, Clone)]
pub struct CustomAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

impl CustomAgent {
    pub async fn process_message(&self, content: &str) -> Result<String, String> {
        // 自定义处理逻辑
        info!("Processing message with agent {}: {}", self.name, content);

        // 模拟处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(format!("Processed by {}: {}", self.name, content))
    }
}

#[derive(Default)]
pub struct CustomPlugin {
    agents: Arc<RwLock<HashMap<String, CustomAgent>>>,
}

#[tonic::async_trait]
impl AgentXPlugin for CustomPlugin {
    async fn register_agent(
        &self,
        request: Request<RegisterAgentRequest>,
    ) -> Result<Response<RegisterAgentResponse>, Status> {
        let req = request.into_inner();
        let agent_info = req.agent.ok_or_else(|| {
            Status::invalid_argument("Agent info is required")
        })?;

        let agent = CustomAgent {
            id: agent_info.id.clone(),
            name: agent_info.name.clone(),
            description: agent_info.description.clone(),
            capabilities: agent_info.capabilities.clone(),
        };

        let mut agents = self.agents.write().await;
        agents.insert(agent_info.id.clone(), agent);

        info!("Registered custom agent: {}", agent_info.name);

        let response = RegisterAgentResponse {
            success: true,
            message: format!("Agent {} registered successfully", agent_info.name),
        };

        Ok(Response::new(response))
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();
        let message = req.message.ok_or_else(|| {
            Status::invalid_argument("Message is required")
        })?;

        let agents = self.agents.read().await;
        let agent = agents.get(&message.to).ok_or_else(|| {
            Status::not_found("Agent not found")
        })?;

        // 提取文本内容
        let mut text_content = String::new();
        for part in &message.content {
            if let Some(content) = &part.content {
                match content {
                    message_part::Content::Text(text) => {
                        text_content.push_str(text);
                        text_content.push(' ');
                    }
                    _ => {} // 处理其他类型的内容
                }
            }
        }

        // 处理消息
        let result = agent.process_message(&text_content.trim()).await
            .map_err(|e| Status::internal(format!("Processing failed: {}", e)))?;

        // 构造响应消息
        let response_message = A2aMessage {
            message_id: format!("resp_{}", message.message_id),
            from: message.to.clone(),
            to: message.from.clone(),
            role: MessageRole::Agent as i32,
            content: vec![MessagePart {
                content: Some(message_part::Content::Text(result)),
                mime_type: "text/plain".to_string(),
                metadata: HashMap::new(),
            }],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let response = SendMessageResponse {
            success: true,
            message: "Message processed successfully".to_string(),
            response: Some(response_message),
        };

        Ok(Response::new(response))
    }

    async fn get_capabilities(
        &self,
        _request: Request<GetCapabilitiesRequest>,
    ) -> Result<Response<GetCapabilitiesResponse>, Status> {
        let capabilities = vec![
            "custom_processing".to_string(),
            "text_analysis".to_string(),
            "data_transformation".to_string(),
        ];

        let response = GetCapabilitiesResponse {
            capabilities,
            framework: "custom_rust".to_string(),
            version: "0.1.0".to_string(),
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:50055".parse()?;
    let plugin = CustomPlugin::default();

    info!("Starting custom Rust plugin server on {}", addr);

    Server::builder()
        .add_service(AgentXPluginServer::new(plugin))
        .serve(addr)
        .await?;

    Ok(())
}
```

## 🔧 插件配置和部署

### 插件配置文件

```yaml
# plugins/config/plugin_config.yaml
plugins:
  langchain:
    name: "LangChain Plugin"
    language: "python"
    executable: "python plugins/langchain/langchain_plugin.py"
    port: 50052
    capabilities:
      - text_processing
      - question_answering
      - tool_usage
    environment:
      OPENAI_API_KEY: "${OPENAI_API_KEY}"
      PYTHONPATH: "plugins/langchain"

  autogen:
    name: "AutoGen Plugin"
    language: "python"
    executable: "python plugins/autogen/autogen_plugin.py"
    port: 50053
    capabilities:
      - multi_agent_chat
      - code_execution
      - group_collaboration
    environment:
      OPENAI_API_KEY: "${OPENAI_API_KEY}"
      PYTHONPATH: "plugins/autogen"

  mastra:
    name: "Mastra Plugin"
    language: "nodejs"
    executable: "node plugins/mastra/mastra_plugin.js"
    port: 50054
    capabilities:
      - workflow_execution
      - memory_management
      - tool_integration
    environment:
      NODE_PATH: "plugins/mastra/node_modules"

  custom_rust:
    name: "Custom Rust Plugin"
    language: "rust"
    executable: "./target/release/agentx-rust-plugin"
    port: 50055
    capabilities:
      - custom_processing
      - high_performance
      - system_integration
```

### Docker部署

```dockerfile
# plugins/langchain/Dockerfile
FROM python:3.9-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .

EXPOSE 50052

CMD ["python", "langchain_plugin.py"]
```

```yaml
# docker-compose.plugins.yml
version: '3.8'

services:
  langchain-plugin:
    build: ./plugins/langchain
    ports:
      - "50052:50052"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    networks:
      - agentx-network

  autogen-plugin:
    build: ./plugins/autogen
    ports:
      - "50053:50053"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    networks:
      - agentx-network

  mastra-plugin:
    build: ./plugins/mastra
    ports:
      - "50054:50054"
    networks:
      - agentx-network

networks:
  agentx-network:
    external: true
```

## 🧪 插件测试

### 单元测试示例

```python
# plugins/langchain/test_langchain_plugin.py
import unittest
from unittest.mock import Mock, patch
import grpc_testing
import agentx_plugin_pb2 as pb2
import agentx_plugin_pb2_grpc as pb2_grpc
from langchain_plugin import LangChainPlugin

class TestLangChainPlugin(unittest.TestCase):
    def setUp(self):
        self.plugin = LangChainPlugin()
        self.test_server = grpc_testing.server_from_dictionary(
            {pb2_grpc.DESCRIPTOR.services_by_name['AgentXPlugin']: self.plugin},
            pb2_grpc.DESCRIPTOR
        )

    def test_register_agent(self):
        """测试Agent注册"""
        request = pb2.RegisterAgentRequest(
            agent=pb2.AgentInfo(
                id="test_agent",
                name="Test Agent",
                description="A test agent",
                capabilities=["text_processing"]
            )
        )

        method = self.test_server.invoke_unary_unary(
            pb2_grpc.DESCRIPTOR.services_by_name['AgentXPlugin'].methods_by_name['RegisterAgent'],
            {},
            request,
            None
        )

        response, metadata, code, details = method.termination()

        self.assertEqual(code, grpc.StatusCode.OK)
        self.assertTrue(response.success)
        self.assertIn("test_agent", self.plugin.agents)

    def test_send_message(self):
        """测试消息发送"""
        # 先注册Agent
        self.plugin.agents["test_agent"] = Mock()
        self.plugin.agents["test_agent"].invoke.return_value = {"output": "Test response"}

        request = pb2.SendMessageRequest(
            message=pb2.A2AMessage(
                message_id="test_msg",
                from_="user",
                to="test_agent",
                role=pb2.MessageRole.USER,
                content=[pb2.MessagePart(text="Hello")]
            )
        )

        method = self.test_server.invoke_unary_unary(
            pb2_grpc.DESCRIPTOR.services_by_name['AgentXPlugin'].methods_by_name['SendMessage'],
            {},
            request,
            None
        )

        response, metadata, code, details = method.termination()

        self.assertEqual(code, grpc.StatusCode.OK)
        self.assertTrue(response.success)
        self.assertIsNotNone(response.response)

if __name__ == '__main__':
    unittest.main()
```

### 集成测试

```bash
#!/bin/bash
# scripts/test_plugins.sh

echo "Starting plugin integration tests..."

# 启动AgentX核心服务
cargo run --example http_server_demo &
CORE_PID=$!

# 等待核心服务启动
sleep 5

# 启动插件
python plugins/langchain/langchain_plugin.py &
LANGCHAIN_PID=$!

python plugins/autogen/autogen_plugin.py &
AUTOGEN_PID=$!

node plugins/mastra/mastra_plugin.js &
MASTRA_PID=$!

# 等待插件启动
sleep 10

# 运行集成测试
python tests/integration/test_plugin_integration.py

# 清理进程
kill $CORE_PID $LANGCHAIN_PID $AUTOGEN_PID $MASTRA_PID

echo "Plugin integration tests completed."
```

## 📊 性能优化

### 插件性能最佳实践

1. **连接池管理**: 复用gRPC连接，避免频繁建立连接
2. **异步处理**: 使用异步编程模型处理并发请求
3. **内存管理**: 及时释放不需要的资源，避免内存泄漏
4. **缓存策略**: 缓存常用数据和计算结果
5. **批处理**: 批量处理多个请求，减少网络开销

### 性能监控

```python
# plugins/common/performance_monitor.py
import time
import psutil
from functools import wraps

def monitor_performance(func):
    """性能监控装饰器"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time.time()
        start_memory = psutil.Process().memory_info().rss

        try:
            result = func(*args, **kwargs)
            success = True
        except Exception as e:
            result = e
            success = False

        end_time = time.time()
        end_memory = psutil.Process().memory_info().rss

        # 记录性能指标
        duration = end_time - start_time
        memory_delta = end_memory - start_memory

        print(f"Function {func.__name__}:")
        print(f"  Duration: {duration:.3f}s")
        print(f"  Memory delta: {memory_delta / 1024 / 1024:.2f}MB")
        print(f"  Success: {success}")

        if not success:
            raise result

        return result

    return wrapper
```

这个插件开发指南提供了完整的多语言插件开发流程，帮助开发者为AgentX平台创建高质量的AI框架插件。
```