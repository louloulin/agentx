# AgentX .gitignore 配置说明

本文档说明了 AgentX 项目中 `.gitignore` 文件的配置和用途。

## 🎯 目标

`.gitignore` 文件确保以下类型的文件不会被 Git 跟踪：

1. **编译产物和构建文件**
2. **IDE 和编辑器临时文件**
3. **日志和调试文件**
4. **敏感配置信息**
5. **测试和开发临时文件**

## 📁 被忽略的文件类型

### Rust 相关
- `target/` - Cargo 编译输出目录
- `Cargo.lock` - 依赖锁定文件（库项目通常不提交）
- `*.rlib`, `*.rmeta` - Rust 编译产物
- `.rustc_info.json` - Rust 编译器信息缓存

### 开发工具
- `.vscode/`, `.idea/` - IDE 配置文件
- `*.swp`, `*.swo` - Vim 临时文件
- `.DS_Store`, `Thumbs.db` - 操作系统生成的文件

### 日志和临时文件
- `*.log` - 日志文件
- `*.tmp`, `*.temp` - 临时文件
- `.cache/`, `.temp/` - 缓存和临时目录

### 配置和敏感信息
- `.env*` - 环境变量文件
- `*.key`, `*.pem` - 密钥和证书文件
- `secrets.toml` - 敏感配置文件

### AgentX 项目特定
- `test_messages/`, `test_agents/` - 测试数据目录
- `debug_logs/` - 调试日志目录
- `protocol_cache/` - 协议缓存文件
- `performance_results/` - 性能测试结果

## 🔧 使用方法

### 检查文件是否被忽略
```bash
git check-ignore <文件路径>
```

### 强制添加被忽略的文件
```bash
git add -f <文件路径>
```

### 查看所有被忽略的文件
```bash
git status --ignored
```

## 📝 维护建议

1. **定期检查**: 确保新的临时文件类型被正确忽略
2. **团队同步**: 团队成员应该使用相同的 `.gitignore` 配置
3. **敏感信息**: 永远不要提交包含密码、密钥或其他敏感信息的文件
4. **本地配置**: 使用 `local.toml` 或 `development.toml` 进行本地配置覆盖

## ⚠️ 注意事项

- 如果文件已经被 Git 跟踪，添加到 `.gitignore` 不会自动移除它们
- 使用 `git rm --cached <文件>` 来停止跟踪已提交的文件
- 避免忽略重要的配置文件，除非它们包含敏感信息

## 🔍 验证配置

项目包含以下验证机制：

1. **自动测试**: CI/CD 流程会检查是否有不应该提交的文件
2. **预提交钩子**: 可以配置 Git 钩子来防止意外提交
3. **定期审查**: 定期检查仓库中是否有不应该存在的文件

## 📚 相关资源

- [Git .gitignore 文档](https://git-scm.com/docs/gitignore)
- [GitHub .gitignore 模板](https://github.com/github/gitignore)
- [Rust .gitignore 最佳实践](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
