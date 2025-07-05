# AgentX Documentation Center

Welcome to the AgentX Documentation Center! Here you'll find comprehensive documentation to help you understand and use the AgentX platform.

[English Documentation](documentation-index.md) | [中文文档](documentation-index-cn.md)

## 📚 Documentation Navigation

### 🚀 Getting Started

- [Project Overview](../README.md) - Basic introduction and features of AgentX
- [Quick Installation](../README.md#quick-start) - 5-minute AgentX experience
- [Basic Usage](../README.md#basic-usage) - Core functionality examples

### 📖 Core Documentation

#### 🏗️ [Architecture Design](architecture/system-architecture.md)
- [Overall Architecture](architecture/system-architecture.md#overall-architecture) - System architecture overview
- [Core Components](architecture/system-architecture.md#core-components) - Detailed component descriptions
- [Data Flow](architecture/system-architecture.md#data-flow) - System data flow design
- [Security Architecture](architecture/system-architecture.md#security-architecture) - Security design and implementation
- [Performance Optimization](architecture/system-architecture.md#performance-optimization) - Performance design principles
- [Scalability](architecture/system-architecture.md#scalability-design) - Scalability design

#### 🔧 [Development Guide](development/development-guide.md)
- [Environment Setup](development/development-guide.md#development-environment-setup) - Development environment configuration
- [Project Structure](development/development-guide.md#project-structure) - Code organization structure
- [Development Workflow](development/development-guide.md#development-workflow) - Standard development process
- [Coding Standards](development/development-guide.md#code-development) - Code quality standards
- [Testing Strategy](development/development-guide.md#testing-strategy) - Testing methods and tools
- [Debugging Tips](development/development-guide.md#debugging-tips) - Debugging and troubleshooting

#### 📡 [API Documentation](api/api-reference.md)
- [Rust API](api/api-reference.md#rust-api) - Native Rust interfaces
- [HTTP REST API](api/api-reference.md#http-rest-api) - RESTful Web API
- [gRPC API](api/api-reference.md#grpc-api) - High-performance RPC interfaces
- [Error Handling](api/api-reference.md#error-handling) - Error codes and handling
- [Authentication](api/api-reference.md#authentication-and-authorization) - Security authentication mechanisms

#### 🔌 [Plugin Development](plugins/plugin-development.md)
- [Plugin Architecture](plugins/plugin-development.md#plugin-architecture) - Plugin system design
- [gRPC Service Definition](plugins/plugin-development.md#grpc-service-definition) - Standard interface specifications
- [Python Plugin Development](plugins/plugin-development.md#python-plugin-development) - Python plugin guide
- [Node.js Plugin Development](plugins/plugin-development.md#nodejs-plugin-development) - Node.js plugin guide
- [Rust Plugin Development](plugins/plugin-development.md#rust-plugin-development) - Rust plugin guide
- [Plugin Testing](plugins/plugin-development.md#plugin-testing) - Testing methods and tools

#### 🚀 [Deployment Guide](deployment/deployment-guide.md)
- [Local Deployment](deployment/deployment-guide.md#local-development-deployment) - Development environment deployment
- [Docker Deployment](deployment/deployment-guide.md#docker-deployment) - Containerized deployment
- [Kubernetes Deployment](deployment/deployment-guide.md#kubernetes-deployment) - Cluster deployment
- [Cloud Platform Deployment](deployment/deployment-guide.md#cloud-platform-deployment) - Cloud service deployment
- [Operations Management](deployment/deployment-guide.md#operations-management) - Monitoring and maintenance

### 🎯 Topic Guides

#### A2A Protocol
- [A2A Protocol Overview](../crates/agentx-a2a/README.md) - Agent-to-Agent communication protocol
- [Message Formats](../crates/agentx-a2a/README.md#core-components) - Standardized message structures
- [Agent Registration](../crates/agentx-a2a/README.md#usage) - Agent registration and discovery
- [Task Management](../crates/agentx-a2a/README.md#core-components) - Task lifecycle management

#### Multi-Framework Support
- [LangChain Integration](plugins/README.md#langchain-plugin-example) - LangChain framework integration
- [AutoGen Integration](plugins/README.md#autogen-plugin-example) - AutoGen framework integration
- [Mastra Integration](plugins/README.md#mastra-plugin-example) - Mastra framework integration
- [Custom Framework](plugins/README.md#rust-plugin-development) - Custom framework integration

#### Enterprise Features
- [Error Recovery](../README.md#enterprise-features) - Fault handling and recovery
- [Security Authentication](../README.md#enterprise-features) - Security and access control
- [Monitoring & Alerting](../README.md#enterprise-features) - System monitoring and alerting
- [Performance Optimization](architecture/README.md#performance-optimization) - Performance tuning guide

### 📋 Examples and Tutorials

#### Basic Examples
- [HTTP Server Example](../examples/http_server_demo.rs) - Start HTTP API service
- [A2A Protocol Example](../examples/a2a_protocol_demo.rs) - A2A protocol usage
- [Plugin SDK Example](../examples/plugin_sdk_demo.rs) - Plugin development example

#### Framework Integration Examples
- [LangChain Plugin Example](../examples/langchain_plugin_demo.rs) - LangChain integration
- [AutoGen Plugin Example](../examples/autogen_plugin_demo.rs) - AutoGen integration
- [Mastra Plugin Example](../examples/mastra_plugin_demo.rs) - Mastra integration

#### Advanced Examples
- [gRPC Plugin Bridge](../examples/grpc_plugin_bridge_demo.rs) - Plugin bridging
- [Ecosystem Demo](../examples/agentx_ecosystem_demo.rs) - Complete ecosystem demo
- [A2A Advanced Features](../examples/a2a_advanced_features_demo.rs) - Advanced functionality

### 🧪 Testing and Quality

#### Testing Documentation
- [Unit Testing](development/README.md#unit-tests) - Unit test writing
- [Integration Testing](development/README.md#integration-tests) - Integration testing methods
- [Performance Testing](../tests/performance_benchmarks.rs) - Performance benchmark testing
- [Error Recovery Testing](../tests/error_recovery_integration_tests.rs) - Error recovery testing

#### Quality Assurance
- [Code Quality](development/README.md#code-quality-checks) - Code quality standards
- [Performance Metrics](../README.md#performance-metrics) - Performance benchmarks and targets
- [Security Audit](architecture/README.md#security-architecture) - Security audit process

### 📊 Project Information

#### Project Status
- [Implementation Summary](../IMPLEMENTATION_SUMMARY.md) - Project implementation status
- [Feature List](../plan3.md) - Detailed feature planning
- [Changelog](../CHANGELOG.md) - Version update records

#### Community and Contribution
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Code of Conduct](../CODE_OF_CONDUCT.md) - Community behavior guidelines
- [Issue Reporting](https://github.com/agentx/agentx/issues) - Bug reports and suggestions

## 🔍 Quick Find

### By Role

#### 🧑‍💻 Developers
- [Development Environment Setup](development/README.md#development-environment-setup)
- [API Reference](api/README.md)
- [Plugin Development Guide](plugins/README.md)
- [Code Examples](../examples/)

#### 🏗️ Architects
- [System Architecture Design](architecture/README.md)
- [Performance and Scalability](architecture/README.md#performance-optimization)
- [Security Architecture](architecture/README.md#security-architecture)
- [Deployment Architecture](deployment/README.md)

#### 🚀 DevOps Engineers
- [Deployment Guide](deployment/README.md)
- [Monitoring and Logging](deployment/README.md#monitoring-and-logging)
- [Backup and Recovery](deployment/README.md#backup-and-recovery)
- [Troubleshooting](development/README.md#debugging-tips)

#### 📊 Product Managers
- [Project Overview](../README.md)
- [Feature Overview](../README.md#key-features)
- [Performance Metrics](../README.md#performance-metrics)
- [Roadmap](../plan3.md)

### By Topic

#### 🔧 Technical Implementation
- [Rust Core Engine](architecture/README.md#core-components)
- [gRPC Plugin System](plugins/README.md#plugin-architecture)
- [A2A Protocol Implementation](../crates/agentx-a2a/README.md)
- [HTTP API Service](../crates/agentx-http/README.md)

#### 🌐 Framework Integration
- [Multi-Framework Support](../README.md#supported-ai-frameworks)
- [LangChain Integration](plugins/README.md#langchain-plugin-example)
- [AutoGen Integration](plugins/README.md#autogen-plugin-example)
- [Mastra Integration](plugins/README.md#mastra-plugin-example)

#### 🛡️ Enterprise Features
- [High Availability](architecture/README.md#scalability-design)
- [Security Mechanisms](architecture/README.md#security-architecture)
- [Monitoring System](deployment/README.md#monitoring-and-logging)
- [Error Recovery](../README.md#enterprise-features)

## 📞 Getting Help

### Documentation Issues
If you find errors in the documentation or need improvements, please:
1. [Submit an Issue](https://github.com/agentx/agentx/issues/new?template=documentation.md)
2. [Submit a Pull Request](https://github.com/agentx/agentx/pulls)
3. [Join the Discussion](https://github.com/agentx/agentx/discussions)

### Technical Support
- **GitHub Issues**: [Bug Reports](https://github.com/agentx/agentx/issues)
- **GitHub Discussions**: [Technical Discussions](https://github.com/agentx/agentx/discussions)
- **Email Support**: agentx-support@example.com

### Community Resources
- **Official Website**: https://agentx.dev
- **Blog**: https://blog.agentx.dev
- **Twitter**: @AgentXFramework
- **Discord**: [AgentX Community](https://discord.gg/agentx)

---

<div align="center">

**📖 Continuously updated, thank you for your attention and support!**

If this project helps you, please give us a ⭐ Star!

</div>
