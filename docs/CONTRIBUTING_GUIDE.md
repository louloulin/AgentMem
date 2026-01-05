# Contributing Guide

欢迎贡献 AgentMem 项目！感谢你有兴趣贡献。

## 🚀 快速开始

### 第一次贡献？

1. Fork 项目仓库
2. 创建功能分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'feat: add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 创建 Pull Request

## 📋 贡献类型

我们欢迎以下类型的贡献：

### 代码贡献
- Bug 修复
- 新功能开发
- 性能优化
- 代码重构
- 测试补充

### 文档贡献
- 文档改进
- 示例代码
- 翻译文档
- 错别字修正

### 社区贡献
- 回答 Issue 问题
- 审查 Pull Request
- 分享使用经验
- 撰写博客文章

## 🔧 开发环境设置

### 前置要求

- Rust 1.75+
- Git
- GitHub 账号

### 设置步骤

1. **Fork 并克隆仓库**

```bash
git clone https://github.com/YOUR_USERNAME/agentmem.git
cd agentmem
```

2. **添加上游远程**

```bash
git remote add upstream https://github.com/agentmem/agentmem.git
```

3. **安装开发工具**

```bash
# 安装 Rust 工具链
rustup component add rustfmt clippy

# 安装 pre-commit hooks（可选）
pip install pre-commit
pre-commit install
```

4. **构建项目**

```bash
cargo build --release
```

5. **运行测试**

```bash
cargo test --all-features
```

## 📝 代码规范

### Rust 代码

1. **格式化代码**

```bash
cargo fmt --all
```

2. **Lint 检查**

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

3. **编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example() {
        // 测试代码
    }
}
```

4. **文档注释**

```rust
/// Adds a new memory to the store.
///
/// # Arguments
///
/// * `content` - The memory content to add
///
/// # Returns
///
/// Returns the ID of the created memory.
///
/// # Examples
///
/// ```no_run
/// # use agent_mem::Memory;
/// let memory = Memory::quick();
/// let id = memory.add("Test").await.unwrap();
/// ```
pub async fn add(&self, content: &str) -> Result<String> {
    // 实现
}
```

### Commit 消息规范

遵循 Conventional Commits 规范：

```
type(scope): description

[optional body]

[optional footer]
```

**Type**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档变更
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

**示例**:
```
feat(storage): add PostgreSQL backend support

Implement full PostgreSQL support as an alternative to SQLite.
Includes connection pooling, transaction support, and migration handling.

Closes #123
```

## 🐛 报告 Bug

### 报告 Bug 前

1. 搜索现有 Issues
2. 阅读文档和 FAQ
3. 尝试最新版本

### 报告 Bug 时

使用 [Bug Report 模板](.github/ISSUE_TEMPLATE/bug_report.md)并提供：
- 清晰的描述
- 复现步骤
- 环境信息
- 最小可复现代码
- 错误日志和堆栈跟踪

## 💡 提出新功能

### 功能请求前

1. 阅读 [ROADMAP](docs/plans/) 了解计划
2. 搜索现有的 Feature Requests
3. 考虑与项目目标的契合度

### 功能请求时

使用 [Feature Request 模板](.github/ISSUE_TEMPLATE/feature_request.md)并提供：
- 功能描述
- 解决的问题
- 使用场景
- 实现思路
- 优先级

## 🔄 Pull Request 流程

### 创建 PR 前

1. **更新分支**

```bash
git fetch upstream
git checkout main
git pull upstream main
git checkout feature/amazing-feature
git rebase main
```

2. **解决冲突**

```bash
git rebase main
# 解决冲突
git add .
git rebase --continue
```

3. **运行测试**

```bash
cargo test --all-features
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

### 提交 PR 时

1. 使用 [PR 模板](.github/pull_request_template.md)
2. 填写所有必需部分
3. 关联相关 Issue
4. 添加适当的标签
5. 请求需要的审查者

### PR 审查标准

- ✅ 代码符合风格规范
- ✅ 通过所有测试
- ✅ 添加适当的测试
- ✅ 更新相关文档
- ✅ 没有新的警告
- ✅ 性能无回归

## 🧪 测试指南

### 单元测试

```rust
#[tokio::test]
async fn test_add_memory() {
    let memory = Memory::quick();
    let id = memory.add("Test content").await.unwrap();
    assert!(!id.is_empty());
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_memory_workflow() {
    let memory = Memory::quick();
    
    // Add
    memory.add("User likes coffee").await.unwrap();
    
    // Search
    let results = memory.search("coffee").await.unwrap();
    assert_eq!(results.len(), 1);
    
    // Delete
    memory.delete(&results[0].id).await.unwrap();
}
```

### 基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_add_memory(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let memory = Memory::quick();
    
    c.bench_function("add_memory", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(memory.add("Test content").await)
        })
    });
}

criterion_group!(benches, bench_add_memory);
criterion_main!(benches);
```

## 📚 文档贡献

### 文档类型

1. **API 文档** - 代码注释中的 rustdoc
2. **用户指南** - `docs/guides/`
3. **架构文档** - `docs/architecture/`
4. **示例代码** - `examples/`
5. **教程** - `docs/tutorials/`

### 文档规范

- 使用清晰简洁的语言
- 提供代码示例
- 包含使用场景
- 添加图表和截图（如适用）

## 🎯 贡献者认可

### 贡献者列表

所有贡献者将被添加到 [CONTRIBUTORS.md](CONTRIBUTORS.md) 文件中。

### 发布说明

显著贡献将被提及：
- GitHub Releases 发布说明
- CHANGELOG.md 更新日志
- 项目网站展示

### 认证计划（计划中）

- 贡献者徽章
- 证书计划
- 年度表彰

## 🤝 社区准则

请遵守我们的 [Code of Conduct](../CODE_OF_CONDUCT.md)：
- 尊重不同观点
- 接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

## 📞 获取帮助

### 资源

- 📖 [文档](docs/)
- 💬 [Discussions](https://github.com/agentmem/agentmem/discussions)
- 💬 [Discord](https://discord.gg/agentmem)
- 📧 Email: maintainers@agentmem.dev

### 需要帮助？

- 在 Issue 中使用 `help wanted` 标签
- 在 Discord 中提问
- 联系维护者

## ⭐ 优秀贡献指南

### 什么构成优秀贡献？

1. **明确的目标** - 清晰描述要解决的问题
2. **最小化变更** - 专注解决核心问题
3. **完整测试** - 包含单元测试和集成测试
4. **文档更新** - 更新相关文档
5. **代码质量** - 符合项目规范
6. **持续沟通** - 积极响应审查意见

### 贡献者阶梯

- **新手** - 修复小 bug，改进文档
- **贡献者** - 提交新功能，修复复杂 bug
- **维护者** - 审查 PR，合并 PR，管理版本
- **核心团队** - 架构决策，路线图规划

## 🎉 感谢贡献

感谢你考虑为 AgentMem 做出贡献！每一个贡献都很宝贵，无论是代码、文档、测试还是反馈。

让我们一起构建更好的 AI 记忆系统！🚀

---

*最后更新: 2025-01-05*
