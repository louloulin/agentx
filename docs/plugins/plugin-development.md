# AgentX Plugin Development Guide

## 📖 Overview

AgentX adopts a gRPC plugin architecture supporting plugin development in multiple programming languages. This guide provides detailed instructions on developing AgentX plugins for different AI frameworks.

[English Version](plugin-development.md) | [中文版本](plugin-development-cn.md)

## 🏗️ Plugin Architecture

### Design Principles

- **Process Isolation**: Each plugin runs in an independent process, ensuring fault isolation
- **Standard Interface**: Unified gRPC service definitions supporting multi-language implementations
- **Hot Plugging**: Runtime dynamic loading and unloading of plugins
- **Framework Agnostic**: Independent of specific AI frameworks, integrated through adapter patterns

### Plugin Communication Flow

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

## 🔌 gRPC Service Definition

### Core Service Interface

```protobuf
// proto/agentx_plugin.proto
syntax = "proto3";

package agentx.plugin;

// Main plugin service
service AgentXPlugin {
  // Agent management
  rpc RegisterAgent(RegisterAgentRequest) returns (RegisterAgentResponse);
  rpc UnregisterAgent(UnregisterAgentRequest) returns (UnregisterAgentResponse);
  rpc GetAgentStatus(GetAgentStatusRequest) returns (GetAgentStatusResponse);
  
  // Message handling
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEvent);
  
  // Task management
  rpc CreateTask(CreateTaskRequest) returns (CreateTaskResponse);
  rpc GetTaskStatus(GetTaskStatusRequest) returns (GetTaskStatusResponse);
  rpc CancelTask(CancelTaskRequest) returns (CancelTaskResponse);
  
  // Capability queries
  rpc GetCapabilities(GetCapabilitiesRequest) returns (GetCapabilitiesResponse);
  rpc QueryAgents(QueryAgentsRequest) returns (QueryAgentsResponse);
}

// Agent information
message AgentInfo {
  string id = 1;
  string name = 2;
  string description = 3;
  repeated string capabilities = 4;
  string endpoint = 5;
  AgentStatus status = 6;
  map<string, string> metadata = 7;
}

// Agent status
enum AgentStatus {
  UNKNOWN = 0;
  ONLINE = 1;
  OFFLINE = 2;
  BUSY = 3;
  ERROR = 4;
}
```

## 🐍 Python Plugin Development

### Environment Setup

```bash
# Install dependencies
pip install grpcio grpcio-tools

# Generate Python code
python -m grpc_tools.protoc \
  --proto_path=proto \
  --python_out=plugins/python \
  --grpc_python_out=plugins/python \
  proto/agentx_plugin.proto
```

### LangChain Plugin Example

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
        """Register LangChain Agent"""
        try:
            agent_info = request.agent
            
            # Create LangChain Agent
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
        """Handle message sending"""
        try:
            message = request.message
            agent_id = message.to
            
            if agent_id not in self.agents:
                return pb2.SendMessageResponse(
                    success=False,
                    message="Agent not found"
                )
            
            # Extract text content
            text_content = ""
            for part in message.content:
                if part.HasField('text'):
                    text_content += part.text + " "
            
            # Execute LangChain Agent
            agent_executor = self.agents[agent_id]
            result = agent_executor.invoke({"input": text_content.strip()})
            
            # Construct response message
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

def serve():
    """Start plugin server"""
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

## 🟨 Node.js Plugin Development

### Environment Setup

```bash
# Initialize Node.js project
npm init -y

# Install dependencies
npm install @grpc/grpc-js @grpc/proto-loader

# Install TypeScript type definitions (optional)
npm install -D @types/node typescript
```

### Mastra Plugin Example

```javascript
// plugins/mastra/mastra_plugin.js
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const { Mastra } = require('@mastra/core');

// Load proto file
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
      
      // Create Mastra Agent
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

      // Extract text content
      let textContent = '';
      for (const part of message.content) {
        if (part.text) {
          textContent += part.text + ' ';
        }
      }

      // Use Mastra to process message
      const agent = this.agents.get(agentId);
      const result = await agent.generate(textContent.trim());

      // Construct response message
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
}

function serve() {
  const server = new grpc.Server();
  const plugin = new MastraPlugin();

  server.addService(agentxProto.AgentXPlugin.service, {
    registerAgent: plugin.registerAgent.bind(plugin),
    sendMessage: plugin.sendMessage.bind(plugin)
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

## 🦀 Rust Plugin Development

### Environment Setup

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

### Custom Framework Plugin Example

```rust
// src/lib.rs
use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

// Generated protobuf code
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
        // Custom processing logic
        info!("Processing message with agent {}: {}", self.name, content);
        
        // Simulate processing
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

        // Extract text content
        let mut text_content = String::new();
        for part in &message.content {
            if let Some(content) = &part.content {
                match content {
                    message_part::Content::Text(text) => {
                        text_content.push_str(text);
                        text_content.push(' ');
                    }
                    _ => {} // Handle other content types
                }
            }
        }

        // Process message
        let result = agent.process_message(&text_content.trim()).await
            .map_err(|e| Status::internal(format!("Processing failed: {}", e)))?;

        // Construct response message
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

## 🧪 Plugin Testing

### Unit Testing Example

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
        """Test agent registration"""
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

if __name__ == '__main__':
    unittest.main()
```

This plugin development guide provides a complete multi-language plugin development workflow, helping developers create high-quality AI framework plugins for the AgentX platform.
