# AgentX 贡献指南 / Contributing Guide

欢迎为 AgentX 项目做出贡献！AgentX 是一个通用的 AI Agent 框架，采用微内核+插件架构，支持多种 AI 框架的互操作性。

## 🌟 项目愿景

AgentX 致力于构建一个统一的 AI Agent 生态系统，让不同框架的 Agent 能够无缝协作，推动 AI Agent 技术的标准化和普及。

## 📋 贡献方式

### 1. 代码贡献

#### 开发环境设置
```bash
# 克隆项目
git clone https://github.com/louloulin/agentx.git
cd agentx

# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# 构建项目
cargo build

# 运行测试
cargo test --workspace
```

#### 代码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 确保所有测试通过
- 新功能必须包含单元测试和集成测试
- 代码覆盖率要求达到 90% 以上

#### 提交规范
```
类型(范围): 简短描述

详细描述（可选）

关联的 Issue: #123
```

类型包括：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建过程或辅助工具的变动

### 2. 文档贡献

#### 文档类型
- **API 文档**: 使用 `cargo doc` 生成
- **用户指南**: 在 `docs/` 目录下
- **开发者文档**: 在 `docs/dev/` 目录下
- **示例代码**: 在 `examples/` 目录下

#### 文档要求
- 中英文双语支持
- 包含完整的代码示例
- 提供清晰的步骤说明
- 定期更新以保持同步

### 3. 插件开发

#### 插件架构
AgentX 采用 gRPC 插件系统，支持多语言插件开发：

```rust
// Rust 插件示例
use agentx_sdk::prelude::*;

#[derive(Debug)]
pub struct MyPlugin {
    config: PluginConfig,
}

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        // 插件逻辑实现
        Ok(PluginResponse::new("success"))
    }
}
```

#### 插件开发指南
1. 使用 `agentx-sdk` 创建插件项目
2. 实现 `Plugin` trait
3. 编写完整的测试用例
4. 提供配置文档和使用示例
5. 遵循插件命名规范

### 4. 问题报告

#### Bug 报告模板
```markdown
## Bug 描述
简要描述遇到的问题

## 复现步骤
1. 执行命令 `...`
2. 观察到 `...`
3. 期望结果是 `...`

## 环境信息
- OS: [e.g. macOS 14.0]
- Rust 版本: [e.g. 1.75.0]
- AgentX 版本: [e.g. 0.1.0]

## 附加信息
- 错误日志
- 配置文件
- 相关截图
```

#### 功能请求模板
```markdown
## 功能描述
详细描述希望添加的功能

## 使用场景
说明该功能的具体应用场景

## 实现建议
如果有实现思路，请详细说明

## 替代方案
是否有其他可行的解决方案
```

## 🏗️ 项目架构

### 核心模块
- **agentx-core**: 核心功能和系统管理
- **agentx-a2a**: A2A 协议实现
- **agentx-router**: 消息路由系统
- **agentx-grpc**: gRPC 插件系统
- **agentx-cluster**: 集群管理
- **agentx-sdk**: 开发者 SDK
- **agentx-http**: HTTP API 服务

### 设计原则
1. **微内核架构**: 核心功能最小化，通过插件扩展
2. **协议无关**: 支持多种通信协议
3. **语言无关**: 支持多语言插件开发
4. **高性能**: 消息路由延迟 < 10ms
5. **高可用**: 支持集群部署和故障恢复

## 🧪 测试策略

### 测试类型
1. **单元测试**: 测试单个函数或方法
2. **集成测试**: 测试模块间的交互
3. **性能测试**: 验证性能指标
4. **端到端测试**: 测试完整的用户场景

### 测试要求
- 所有新功能必须包含测试
- 测试覆盖率不低于 90%
- 性能测试必须满足延迟要求
- 集成测试验证多模块协作

### 运行测试
```bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test -p agentx-core

# 运行性能测试
cargo test --release -- --ignored

# 生成测试覆盖率报告
cargo tarpaulin --out Html
```

## 🚀 发布流程

### 版本管理
- 遵循语义化版本控制 (SemVer)
- 主版本号：不兼容的 API 修改
- 次版本号：向下兼容的功能性新增
- 修订号：向下兼容的问题修正

### 发布检查清单
- [ ] 所有测试通过
- [ ] 文档更新完整
- [ ] 性能基准测试通过
- [ ] 安全审计完成
- [ ] 变更日志更新
- [ ] 版本号更新

## 🤝 社区行为准则

### 我们的承诺
为了营造一个开放和友好的环境，我们承诺：
- 尊重不同的观点和经验
- 优雅地接受建设性批评
- 专注于对社区最有利的事情
- 对其他社区成员表示同理心

### 不可接受的行为
- 使用性别化语言或图像
- 人身攻击或政治攻击
- 公开或私下骚扰
- 未经许可发布他人私人信息

## 📞 联系方式

- **GitHub Issues**: 报告 bug 和功能请求
- **GitHub Discussions**: 技术讨论和问答
- **Email**: agentx-dev@example.com
- **Discord**: [AgentX 开发者社区](https://discord.gg/agentx)

## 📄 许可证

AgentX 项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🙏 致谢

感谢所有为 AgentX 项目做出贡献的开发者！

---

**让我们一起构建更好的 AI Agent 生态系统！** 🚀
