# AgentMem Justfile 使用指南

本项目使用 [just](https://github.com/casey/just) 作为命令运行器，统一管理构建、测试、部署等任务。

## 安装 just

### macOS
```bash
brew install just
```

### Linux
```bash
# Arch Linux
pacman -S just

# Ubuntu/Debian
wget -qO - 'https://proget.makedeb.org/debian-feeds/prebuilt-mpr.pub' | gpg --dearmor | sudo tee /usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg 1> /dev/null
echo "deb [arch=amd64 signed-by=/usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg] https://proget.makedeb.org prebuilt-mpr $(lsb_release -cs)" | sudo tee /etc/apt/sources.list.d/prebuilt-mpr.list
sudo apt update
sudo apt install just
```

### 使用 Cargo
```bash
cargo install just
```

## 快速开始

### 查看所有可用命令
```bash
just --list
# 或简写
just
```

### 快速启动全栈服务
```bash
# 构建并启动后端 + 前端
just quick-start
```

### 开发模式
```bash
# 启动开发模式（带热重载）
just dev
```

## 常用命令

### 🔨 构建相关

```bash
# 构建所有项目（debug 模式）
just build

# 构建所有项目（release 模式）
just build-release

# 构建 HTTP API 服务器
just build-server

# 构建 MCP Stdio 服务器
just build-mcp

# 构建前端 UI
just build-ui

# 清理构建产物
just clean

# 清理并重新构建
just rebuild
```

### 🧪 测试相关

```bash
# 运行所有测试
just test

# 运行特定包的测试
just test-package agent-mem-core

# 运行集成测试
just test-integration

# 运行 Working Memory 测试
just test-working-memory

# 运行 MCP 功能测试
just test-mcp

# 运行性能基准测试
just bench
```

### 🚀 服务启动

```bash
# 启动 HTTP API 服务器（无认证模式，前台运行）
just start-server

# 启动 HTTP API 服务器（无认证模式，后台运行）
just start-server-no-auth

# 启动 HTTP API 服务器（带 ONNX Runtime 修复，后台运行）
just start-server-onnx

# 启动 HTTP API 服务器（后台运行，通用）
just start-server-bg

# 启动 MCP Stdio 服务器
just start-mcp

# 启动前端 UI
just start-ui

# 启动全栈（后端 + 前端）
just start-full

# 停止所有服务
just stop
```

#### 三种启动脚本说明

**1. `start-server-no-auth` - 无认证模式启动**
- 脚本文件：`start_server_no_auth.sh`
- 特点：禁用认证，适合开发和测试
- 日志文件：`backend-no-auth.log`
- 用途：快速开发测试，无需处理认证逻辑

**2. `start-server-onnx` - ONNX Runtime 修复版**
- 脚本文件：`start_server_with_correct_onnx.sh`
- 特点：显式指定 ONNX Runtime 1.22.0 库路径
- 日志文件：`backend-onnx-fixed.log`
- 用途：解决 ONNX Runtime 版本冲突问题

**3. `start-full` - 全栈启动**
- 脚本文件：`start_full_stack.sh`
- 特点：自动启动后端和前端，带健康检查
- 日志文件：`backend-test.log` + `frontend.log`
- 用途：完整的前后端集成测试

### 🗄️ 数据库管理

```bash
# 初始化数据库
just db-init

# 运行数据库迁移
just db-migrate

# 备份数据库
just db-backup

# 恢复数据库
just db-restore
```

### 💬 MCP 相关

```bash
# 验证 MCP 工具功能
just mcp-verify

# 测试 MCP Chat 功能并验证 Working Memory
just mcp-test-chat

# 配置 Claude Desktop
just mcp-setup-claude
```

### 🔍 代码质量

```bash
# 运行 clippy 检查
just clippy

# 格式化代码
just fmt

# 检查代码格式
just fmt-check

# 生成文档
just doc

# 运行安全审计
just audit
```

### 🛠️ 开发工具

```bash
# 监听文件变化并自动重新编译
just watch

# 监听并运行测试
just watch-test

# 运行示例程序
just run-example chat-demo

# 检查项目健康状态
just health

# 查看实时日志
just logs backend    # 后端日志
just logs frontend   # 前端日志
just logs ui         # UI 日志
```

### 🐳 部署相关

```bash
# 构建 Docker 镜像
just docker-build

# 启动 Docker Compose
just docker-up

# 停止 Docker Compose
just docker-down

# 构建生产版本
just build-prod

# 部署到生产环境
just deploy-prod
```

### 📊 信息查看

```bash
# 显示项目信息
just info

# 显示环境变量
just env
```

## 完整工作流示例

### 1. 新功能开发流程

```bash
# 1. 清理并构建
just rebuild

# 2. 运行测试确保基础功能正常
just test

# 3. 启动开发模式
just dev

# 4. 在另一个终端查看日志
just logs backend

# 5. 开发完成后运行代码检查
just clippy
just fmt

# 6. 运行完整测试
just test

# 7. 停止服务
just stop
```

### 2. MCP 功能验证流程

```bash
# 1. 构建 MCP 服务器
just build-mcp

# 2. 测试 MCP 基础功能
just test-mcp

# 3. 测试 Chat 功能并验证 Working Memory
just mcp-test-chat

# 4. 验证所有 MCP 工具
just mcp-verify

# 5. 配置 Claude Desktop（查看配置信息）
just mcp-setup-claude
```

### 3. 生产部署流程

```bash
# 1. 运行完整测试
just test

# 2. 代码质量检查
just clippy
just fmt-check

# 3. 安全审计
just audit

# 4. 构建生产版本
just build-prod

# 5. 构建 Docker 镜像
just docker-build

# 6. 部署
just deploy-prod
```

### 4. 数据库维护流程

```bash
# 1. 备份当前数据库
just db-backup

# 2. 运行数据库迁移
just db-migrate

# 3. 如果出现问题，恢复数据库
just db-restore
```

## 环境变量配置

justfile 已经预配置了以下环境变量：

```bash
# Rust 配置
RUST_BACKTRACE=1

# LLM 配置
LLM_PROVIDER=zhipu
LLM_MODEL=glm-4.6
ZHIPU_API_KEY=<your-api-key>

# Embedder 配置
EMBEDDER_PROVIDER=fastembed
EMBEDDER_MODEL=BAAI/bge-small-en-v1.5

# 库路径
DYLD_LIBRARY_PATH=./lib:./target/release
ORT_DYLIB_PATH=./lib/libonnxruntime.1.22.0.dylib
```

如需修改，请编辑 `justfile` 文件顶部的配置部分。

## 常见问题

### Q: 如何查看某个命令的详细信息？
A: 使用 `just --show <command>` 查看命令定义，例如：
```bash
just --show build-server
```

### Q: 如何传递参数给命令？
A: 某些命令支持参数，例如：
```bash
just test-package agent-mem-core
just run-example chat-demo
just logs backend
```

### Q: 服务启动后如何查看状态？
A: 使用健康检查命令：
```bash
just health
```

### Q: 如何同时运行多个命令？
A: 可以使用 `&&` 连接：
```bash
just build && just test && just start-server
```

或者使用预定义的组合命令：
```bash
just verify  # 构建 + 测试 + 健康检查
```

## 提示和技巧

1. **Tab 补全**: 如果你的 shell 支持，可以配置 just 的 tab 补全
2. **别名**: 可以在 shell 配置文件中为常用命令创建别名
3. **并行执行**: 某些命令可以并行运行以提高效率
4. **日志查看**: 使用 `just logs` 命令实时查看服务日志

## 更多信息

- [just 官方文档](https://just.systems/)
- [AgentMem 项目文档](./docs/)
- [MCP 集成指南](./examples/mcp-stdio-server/README.md)

