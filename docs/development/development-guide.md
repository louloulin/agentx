# AgentX Development Guide

## 📖 Overview

This guide helps developers quickly get started with AgentX project development, including environment setup, code structure, development workflow, and best practices.

[English Version](development-guide.md) | [中文版本](development-guide-cn.md)

## 🛠️ Development Environment Setup

### System Requirements

- **Operating System**: Linux, macOS, Windows (WSL2 recommended)
- **Rust**: 1.70+ (recommended to install via rustup)
- **Node.js**: 18+ (for Mastra plugin development)
- **Python**: 3.8+ (for LangChain/AutoGen plugin development)
- **Protocol Buffers**: 3.15+
- **Docker**: 20.10+ (optional, for containerized development)

### Install Rust Toolchain

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload environment variables
source ~/.cargo/env

# Install necessary components
rustup component add clippy rustfmt

# Verify installation
rustc --version
cargo --version
```

### Install Protocol Buffers

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
# Using chocolatey
choco install protoc
```

### Clone Project

```bash
git clone https://github.com/agentx/agentx.git
cd agentx

# Build project
cargo build

# Run tests
cargo test
```

## 📁 Project Structure

```
agentx/
├── crates/                     # Rust crates
│   ├── agentx-core/            # Core management module
│   │   ├── src/
│   │   │   ├── lib.rs          # Module entry
│   │   │   ├── protocol_compat.rs  # Protocol compatibility layer
│   │   │   ├── cloud_native.rs     # Cloud native support
│   │   │   ├── developer_ecosystem.rs # Developer ecosystem
│   │   │   └── error_recovery.rs    # Error recovery
│   │   └── Cargo.toml
│   ├── agentx-a2a/             # A2A protocol implementation
│   │   ├── src/
│   │   │   ├── lib.rs          # Module entry
│   │   │   ├── message.rs      # Message formats
│   │   │   ├── agent_card.rs   # Agent descriptions
│   │   │   ├── capability.rs   # Capability matching
│   │   │   ├── protocol_engine.rs # Protocol engine
│   │   │   ├── streaming.rs    # Stream processing
│   │   │   ├── security.rs     # Security management
│   │   │   └── monitoring.rs   # Monitoring system
│   │   └── Cargo.toml
│   ├── agentx-grpc/            # gRPC plugin system
│   ├── agentx-http/            # HTTP API server
│   ├── agentx-cluster/         # Cluster management
│   └── agentx-sdk/             # Developer SDK
├── plugins/                    # Plugin implementations
│   ├── mastra/                 # Mastra plugin
│   ├── langchain/              # LangChain plugin
│   └── autogen/                # AutoGen plugin
├── proto/                      # Protocol Buffers definitions
├── examples/                   # Example code
├── tests/                      # Integration tests
├── docs/                       # Documentation
└── Cargo.toml                  # Workspace configuration
```

## 🔧 Development Workflow

### 1. Create New Feature Branch

```bash
# Create new branch from main
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

### 2. Code Development

#### Coding Standards

- **Naming Convention**: Use snake_case for variables and functions, PascalCase for types
- **Documentation**: Use `///` for public API documentation comments
- **Error Handling**: Use `Result<T, E>` type for error handling, avoid panic
- **Async Programming**: Prefer async/await, avoid blocking operations

#### Example Code Structure

```rust
//! Module documentation comment
//! 
//! Detailed description of module functionality and purpose

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, error};

/// Public struct documentation comment
#[derive(Debug, Clone)]
pub struct ExampleStruct {
    /// Field documentation comment
    pub id: String,
    pub data: HashMap<String, String>,
}

impl ExampleStruct {
    /// Constructor documentation comment
    /// 
    /// # Arguments
    /// 
    /// * `id` - Unique identifier
    /// 
    /// # Returns
    /// 
    /// Returns new ExampleStruct instance
    pub fn new(id: String) -> Self {
        Self {
            id,
            data: HashMap::new(),
        }
    }
    
    /// Async method example
    /// 
    /// # Errors
    /// 
    /// Returns error when operation fails
    pub async fn process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting processing: {}", self.id);
        
        // Actual processing logic
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
}
```

### 3. Testing

#### Unit Tests

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

#### Integration Tests

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

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --package agentx-a2a

# Run integration tests
cargo test --test integration_test

# Run performance tests
cargo test --test performance_benchmarks --release
```

### 4. Code Quality Checks

```bash
# Code formatting
cargo fmt

# Code linting
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Dependency audit
cargo audit
```

### 5. Commit Code

```bash
# Add files
git add .

# Commit with conventional commit message
git commit -m "feat: add new A2A message processing functionality"

# Push branch
git push origin feature/your-feature-name
```

## 🔌 Plugin Development

### gRPC Plugin Development

#### 1. Define Protocol Buffers

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

#### 2. Generate Code

```bash
# Generate Rust code
cargo build

# Generate Python code
python -m grpc_tools.protoc --proto_path=proto --python_out=plugins/langchain --grpc_python_out=plugins/langchain proto/agentx_plugin.proto
```

#### 3. Implement Plugin Server

```rust
// Rust plugin implementation
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
        
        // Handle agent registration logic
        
        let response = RegisterAgentResponse {
            success: true,
            message: "Agent registered successfully".to_string(),
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

#### 4. Python Plugin Example

```python
# plugins/langchain/plugin_server.py
import grpc
from concurrent import futures
import agentx_plugin_pb2_grpc as pb2_grpc
import agentx_plugin_pb2 as pb2

class LangChainPlugin(pb2_grpc.AgentXPluginServicer):
    def RegisterAgent(self, request, context):
        # Handle agent registration
        print(f"Registering agent: {request.agent.name}")
        
        return pb2.RegisterAgentResponse(
            success=True,
            message="LangChain agent registered successfully"
        )
    
    def SendMessage(self, request, context):
        # Handle message sending
        # Call LangChain processing logic
        
        return pb2.SendMessageResponse(
            success=True,
            response="Processing completed"
        )

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    pb2_grpc.add_AgentXPluginServicer_to_server(LangChainPlugin(), server)
    
    listen_addr = '[::]:50052'
    server.add_insecure_port(listen_addr)
    
    print(f"LangChain plugin started on {listen_addr}")
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
```

## 🧪 Testing Strategy

### Test Pyramid

```
┌─────────────────────────────────────┐
│           E2E Tests (Few)           │  ← End-to-end tests
├─────────────────────────────────────┤
│       Integration Tests (Some)      │  ← Integration tests
├─────────────────────────────────────┤
│        Unit Tests (Many)            │  ← Unit tests
└─────────────────────────────────────┘
```

### Test Types

#### 1. Unit Tests
- Test individual functions or methods
- Use mock objects to isolate dependencies
- Fast execution, high coverage

#### 2. Integration Tests
- Test component interactions
- Use real dependency services
- Verify API contracts

#### 3. Performance Tests
- Test system performance metrics
- Load testing and stress testing
- Performance regression detection

#### 4. End-to-End Tests
- Test complete user scenarios
- Use real environments
- Verify overall system functionality

### Testing Tools

```toml
# Cargo.toml [dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
criterion = "0.5"
proptest = "1.0"
```

## 🚀 Deployment and Release

### Local Development Environment

```bash
# Start development server
cargo run --example http_server_demo

# Start plugins
cd plugins/langchain && python plugin_server.py
```

### Docker Development Environment

```bash
# Build Docker image
docker build -t agentx:dev .

# Start development environment
docker-compose -f docker-compose.dev.yml up
```

### Production Deployment

```bash
# Build release version
cargo build --release

# Create Docker image
docker build -t agentx:latest .

# Deploy to Kubernetes
kubectl apply -f k8s/
```

## 📊 Performance Optimization

### Performance Analysis Tools

```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --example http_server_demo

# Memory analysis
cargo install heaptrack
heaptrack target/release/agentx

# Benchmarking
cargo bench
```

### Optimization Tips

1. **Avoid unnecessary cloning**: Use references and borrowing
2. **Use appropriate data structures**: HashMap vs BTreeMap
3. **Async programming**: Avoid blocking operations
4. **Memory pools**: Pre-allocate large objects
5. **Batch processing**: Reduce system call frequency

## 🔍 Debugging Tips

### Logging Configuration

```rust
use tracing::{info, debug, warn, error};
use tracing_subscriber;

// Initialize logging
tracing_subscriber::fmt::init();

// Use logging
info!("System started");
debug!("Debug info: {}", data);
warn!("Warning: unstable connection");
error!("Error: {}", error);
```

### Debugging Tools

```bash
# Debug with rust-gdb
rust-gdb target/debug/agentx

# Debug with lldb
rust-lldb target/debug/agentx

# Environment variable debugging
RUST_LOG=debug cargo run
RUST_BACKTRACE=1 cargo run
```

## 📝 Documentation Writing

### Code Documentation

```rust
/// Calculate the sum of two numbers
/// 
/// # Arguments
/// 
/// * `a` - First number
/// * `b` - Second number
/// 
/// # Returns
/// 
/// Returns the sum of the two numbers
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

### Generate Documentation

```bash
# Generate and open documentation
cargo doc --open

# Test examples in documentation
cargo test --doc
```

## 🤝 Contributing Guidelines

### Commit Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Type descriptions:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation update
- `style`: Code formatting
- `refactor`: Code refactoring
- `test`: Test related
- `chore`: Build process or auxiliary tool changes

### Pull Request Process

1. Fork the project
2. Create feature branch
3. Write code and tests
4. Ensure all tests pass
5. Submit Pull Request
6. Code review
7. Merge to main branch

This development guide provides comprehensive guidance for AgentX project developers, ensuring code quality and development efficiency.
