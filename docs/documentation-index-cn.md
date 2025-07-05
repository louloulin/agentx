# AgentX 文档中心

欢迎来到AgentX文档中心！这里包含了AgentX项目的完整文档，帮助您快速了解和使用AgentX平台。

[English Documentation](documentation-index.md) | [中文文档](documentation-index-cn.md)

## 📚 文档导航

### 🚀 快速开始

- [项目概述](../README_CN.md) - AgentX项目的基本介绍和特性
- [快速安装](../README_CN.md#快速开始) - 5分钟快速体验AgentX
- [基本使用](../README_CN.md#基本使用) - 核心功能使用示例

### 📖 核心文档

#### 🏗️ [架构设计](architecture/system-architecture-cn.md)
- [整体架构](architecture/system-architecture-cn.md#整体架构) - 系统架构概览
- [核心组件](architecture/system-architecture-cn.md#核心组件) - 各组件详细说明
- [数据流](architecture/system-architecture-cn.md#数据流) - 系统数据流设计
- [安全架构](architecture/system-architecture-cn.md#安全架构) - 安全设计和实现
- [性能优化](architecture/system-architecture-cn.md#性能优化) - 性能设计原则
- [可扩展性](architecture/system-architecture-cn.md#可扩展性设计) - 扩展性设计

#### 🔧 [开发指南](development/development-guide-cn.md)
- [环境搭建](development/development-guide-cn.md#开发环境搭建) - 开发环境配置
- [项目结构](development/development-guide-cn.md#项目结构) - 代码组织结构
- [开发工作流](development/development-guide-cn.md#开发工作流) - 标准开发流程
- [编码规范](development/development-guide-cn.md#代码开发) - 代码质量标准
- [测试策略](development/development-guide-cn.md#测试策略) - 测试方法和工具
- [调试技巧](development/development-guide-cn.md#调试技巧) - 调试和排错

#### 📡 [API文档](api/api-reference-cn.md)
- [Rust API](api/api-reference-cn.md#rust-api) - 原生Rust接口
- [HTTP REST API](api/api-reference-cn.md#http-rest-api) - RESTful Web API
- [gRPC API](api/api-reference-cn.md#grpc-api) - 高性能RPC接口
- [错误处理](api/api-reference-cn.md#错误处理) - 错误码和处理
- [认证授权](api/api-reference-cn.md#认证和授权) - 安全认证机制

#### 🔌 [插件开发](plugins/plugin-development-cn.md)
- [插件架构](plugins/plugin-development-cn.md#插件架构) - 插件系统设计
- [gRPC服务定义](plugins/plugin-development-cn.md#grpc服务定义) - 标准接口规范
- [Python插件开发](plugins/plugin-development-cn.md#python插件开发) - Python插件指南
- [Node.js插件开发](plugins/plugin-development-cn.md#nodejs插件开发) - Node.js插件指南
- [Rust插件开发](plugins/plugin-development-cn.md#rust插件开发) - Rust插件指南
- [插件测试](plugins/plugin-development-cn.md#插件测试) - 测试方法和工具

#### 🚀 [部署指南](deployment/deployment-guide-cn.md)
- [本地部署](deployment/deployment-guide-cn.md#本地开发部署) - 开发环境部署
- [Docker部署](deployment/deployment-guide-cn.md#docker部署) - 容器化部署
- [Kubernetes部署](deployment/deployment-guide-cn.md#kubernetes部署) - 集群部署
- [云平台部署](deployment/deployment-guide-cn.md#云平台部署) - 云服务部署
- [运维管理](deployment/deployment-guide-cn.md#运维管理) - 监控和维护

### 🎯 专题指南

#### A2A协议
- [A2A协议概述](../crates/agentx-a2a/README.md) - Agent-to-Agent通信协议
- [消息格式](../crates/agentx-a2a/README.md#核心组件) - 标准化消息结构
- [Agent注册](../crates/agentx-a2a/README.md#使用方法) - Agent注册和发现
- [任务管理](../crates/agentx-a2a/README.md#核心组件) - 任务生命周期管理

#### 多框架支持
- [LangChain集成](plugins/plugin-development-cn.md#langchain插件示例) - LangChain框架接入
- [AutoGen集成](plugins/plugin-development-cn.md#autogen插件示例) - AutoGen框架接入
- [Mastra集成](plugins/plugin-development-cn.md#mastra插件示例) - Mastra框架接入
- [自定义框架](plugins/plugin-development-cn.md#rust插件开发) - 自定义框架接入

#### 企业级特性
- [错误恢复](../README_CN.md#企业级特性) - 故障处理和恢复
- [安全认证](../README_CN.md#企业级特性) - 安全和权限控制
- [监控告警](../README_CN.md#企业级特性) - 系统监控和告警
- [性能优化](architecture/system-architecture-cn.md#性能优化) - 性能调优指南

### 📋 示例和教程

#### 基础示例
- [HTTP服务器示例](../examples/http_server_demo.rs) - 启动HTTP API服务
- [A2A协议示例](../examples/a2a_protocol_demo.rs) - A2A协议使用
- [插件SDK示例](../examples/plugin_sdk_demo.rs) - 插件开发示例

#### 框架集成示例
- [LangChain插件示例](../examples/langchain_plugin_demo.rs) - LangChain集成
- [AutoGen插件示例](../examples/autogen_plugin_demo.rs) - AutoGen集成
- [Mastra插件示例](../examples/mastra_plugin_demo.rs) - Mastra集成

#### 高级示例
- [gRPC插件桥接](../examples/grpc_plugin_bridge_demo.rs) - 插件桥接
- [生态系统演示](../examples/agentx_ecosystem_demo.rs) - 完整生态演示
- [A2A高级特性](../examples/a2a_advanced_features_demo.rs) - 高级功能演示

### 🧪 测试和质量

#### 测试文档
- [单元测试](development/development-guide-cn.md#单元测试) - 单元测试编写
- [集成测试](development/development-guide-cn.md#集成测试) - 集成测试方法
- [性能测试](../tests/performance_benchmarks.rs) - 性能基准测试
- [错误恢复测试](../tests/error_recovery_integration_tests.rs) - 错误恢复测试

#### 质量保证
- [代码质量](development/development-guide-cn.md#代码质量检查) - 代码质量标准
- [性能指标](../README_CN.md#性能指标) - 性能基准和目标
- [安全审计](architecture/system-architecture-cn.md#安全架构) - 安全审计流程

### 📊 项目信息

#### 项目状态
- [实施总结](../实施总结.md) - 项目实施情况
- [功能清单](../plan3.md) - 详细功能规划
- [更新日志](../CHANGELOG.md) - 版本更新记录

#### 社区和贡献
- [贡献指南](../CONTRIBUTING_CN.md) - 如何参与贡献
- [行为准则](../CODE_OF_CONDUCT_CN.md) - 社区行为规范
- [问题反馈](https://github.com/agentx/agentx/issues) - 问题报告和建议

## 🔍 快速查找

### 按角色查找

#### 🧑‍💻 开发者
- [开发环境搭建](development/development-guide-cn.md#开发环境搭建)
- [API参考文档](api/api-reference-cn.md)
- [插件开发指南](plugins/plugin-development-cn.md)
- [代码示例](../examples/)

#### 🏗️ 架构师
- [系统架构设计](architecture/system-architecture-cn.md)
- [性能和扩展性](architecture/system-architecture-cn.md#性能优化)
- [安全架构](architecture/system-architecture-cn.md#安全架构)
- [部署架构](deployment/deployment-guide-cn.md)

#### 🚀 运维工程师
- [部署指南](deployment/deployment-guide-cn.md)
- [监控和日志](deployment/deployment-guide-cn.md#监控和日志)
- [备份和恢复](deployment/deployment-guide-cn.md#备份和恢复)
- [故障排除](development/development-guide-cn.md#调试技巧)

#### 📊 产品经理
- [项目概述](../README_CN.md)
- [功能特性](../README_CN.md#核心特性)
- [性能指标](../README_CN.md#性能指标)
- [路线图](../plan3.md)

### 按主题查找

#### 🔧 技术实现
- [Rust核心引擎](architecture/system-architecture-cn.md#核心组件)
- [gRPC插件系统](plugins/plugin-development-cn.md#插件架构)
- [A2A协议实现](../crates/agentx-a2a/README.md)
- [HTTP API服务](../crates/agentx-http/README.md)

#### 🌐 框架集成
- [多框架支持](../README_CN.md#支持的ai框架)
- [LangChain集成](plugins/plugin-development-cn.md#langchain插件示例)
- [AutoGen集成](plugins/plugin-development-cn.md#autogen插件示例)
- [Mastra集成](plugins/plugin-development-cn.md#mastra插件示例)

#### 🛡️ 企业特性
- [高可用性](architecture/system-architecture-cn.md#可扩展性设计)
- [安全机制](architecture/system-architecture-cn.md#安全架构)
- [监控系统](deployment/deployment-guide-cn.md#监控和日志)
- [错误恢复](../README_CN.md#企业级特性)

## 📞 获取帮助

### 文档问题
如果您在文档中发现错误或需要改进，请：
1. [提交Issue](https://github.com/agentx/agentx/issues/new?template=documentation.md)
2. [提交Pull Request](https://github.com/agentx/agentx/pulls)
3. [参与讨论](https://github.com/agentx/agentx/discussions)

### 技术支持
- **GitHub Issues**: [问题反馈](https://github.com/agentx/agentx/issues)
- **GitHub Discussions**: [技术讨论](https://github.com/agentx/agentx/discussions)
- **邮件支持**: agentx-support@example.com

### 社区资源
- **官方网站**: https://agentx.dev
- **博客**: https://blog.agentx.dev
- **Twitter**: @AgentXFramework
- **Discord**: [AgentX社区](https://discord.gg/agentx)

---

<div align="center">

**📖 持续更新中，感谢您的关注和支持！**

如果这个项目对您有帮助，请给我们一个 ⭐ Star！

</div>
