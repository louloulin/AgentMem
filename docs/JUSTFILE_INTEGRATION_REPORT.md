# Justfile 集成报告

## 概述

本报告记录了将项目中的启动脚本集成到 `justfile` 的工作，实现了统一的命令行接口来管理项目的构建、测试、部署等任务。

## 完成时间

2025-11-03

## 主要工作

### 1. 创建 Justfile

创建了 `justfile` 文件，包含 **50+ 个命令**，分为以下类别：

#### 命令分类

1. **构建命令** (7个)
   - `build` - 构建所有项目（debug 模式）
   - `build-release` - 构建所有项目（release 模式）
   - `build-server` - 构建 HTTP API 服务器
   - `build-mcp` - 构建 MCP Stdio 服务器
   - `build-ui` - 构建前端 UI
   - `clean` - 清理构建产物
   - `rebuild` - 清理并重新构建

2. **测试命令** (6个)
   - `test` - 运行所有测试
   - `test-package` - 运行特定包的测试
   - `test-integration` - 运行集成测试
   - `test-working-memory` - 运行 Working Memory 测试
   - `test-mcp` - 运行 MCP 功能测试
   - `bench` - 运行性能基准测试

3. **代码质量** (5个)
   - `clippy` - 运行 clippy 检查
   - `fmt` - 格式化代码
   - `fmt-check` - 检查代码格式
   - `doc` - 生成文档
   - `audit` - 运行安全审计

4. **服务启动** (9个)
   - `start-server` - 启动 HTTP API 服务器（前台）
   - `start-server-no-auth` - 启动服务器（无认证，后台）
   - `start-server-onnx` - 启动服务器（ONNX 修复版，后台）
   - `start-server-bg` - 启动服务器（后台，通用）
   - `start-mcp` - 启动 MCP Stdio 服务器
   - `start-ui` - 启动前端 UI
   - `start-full` - 启动全栈（后端 + 前端）
   - `stop` - 停止所有服务

5. **数据库管理** (4个)
   - `db-init` - 初始化数据库
   - `db-migrate` - 运行数据库迁移
   - `db-backup` - 备份数据库
   - `db-restore` - 恢复数据库

6. **MCP 相关** (3个)
   - `mcp-verify` - 验证 MCP 工具功能
   - `mcp-test-chat` - 测试 MCP Chat 功能
   - `mcp-setup-claude` - 配置 Claude Desktop

7. **开发工具** (6个)
   - `watch` - 监听文件变化并自动重新编译
   - `watch-test` - 监听并运行测试
   - `run-example` - 运行示例程序
   - `health` - 检查项目健康状态
   - `logs` - 查看实时日志

8. **部署相关** (5个)
   - `docker-build` - 构建 Docker 镜像
   - `docker-up` - 启动 Docker Compose
   - `docker-down` - 停止 Docker Compose
   - `build-prod` - 构建生产版本
   - `deploy-prod` - 部署到生产环境

9. **快捷命令** (4个)
   - `quick-start` - 快速启动全栈服务
   - `verify` - 构建 + 测试 + 健康检查
   - `dev` - 启动开发模式
   - `rebuild` - 清理并重新构建

10. **信息查看** (2个)
    - `info` - 显示项目信息
    - `env` - 显示环境变量

### 2. 集成三种启动脚本

将项目中的三个启动脚本集成到 justfile：

#### 2.1 无认证模式启动 (`start-server-no-auth`)

**脚本文件**: `start_server_no_auth.sh`

**功能特点**:
- 禁用认证（`ENABLE_AUTH=false`）
- 配置 ONNX Runtime 1.22.0 库路径
- 配置 Zhipu AI LLM Provider
- 配置 FastEmbed Embedder
- 后台运行，输出到 `backend-no-auth.log`
- 自动健康检查

**使用场景**:
- 开发环境快速测试
- 无需处理认证逻辑的场景
- API 功能验证

**启动命令**:
```bash
just start-server-no-auth
```

**日志查看**:
```bash
tail -f backend-no-auth.log
```

#### 2.2 ONNX Runtime 修复版 (`start-server-onnx`)

**脚本文件**: `start_server_with_correct_onnx.sh`

**功能特点**:
- 显式指定 ONNX Runtime 1.22.0 库路径
- 解决 ONNX Runtime 版本冲突问题
- 配置 Zhipu AI LLM Provider
- 配置 FastEmbed Embedder
- 后台运行，输出到 `backend-onnx-fixed.log`
- 启动日志过滤（显示 Embedder 相关信息）

**使用场景**:
- 遇到 ONNX Runtime 版本冲突时
- 需要确保使用特定版本的 ONNX Runtime
- 调试 Embedder 初始化问题

**启动命令**:
```bash
just start-server-onnx
```

**日志查看**:
```bash
tail -f backend-onnx-fixed.log
```

#### 2.3 全栈启动 (`start-full`)

**脚本文件**: `start_full_stack.sh`

**功能特点**:
- 自动检查并启动后端服务器
- 自动检查并启动前端服务器
- 智能依赖安装（检测 node_modules）
- 服务健康检查
- Dashboard 数据验证
- 显示完整的服务信息

**使用场景**:
- 完整的前后端集成测试
- 演示和展示
- 端到端功能验证

**启动命令**:
```bash
just start-full
```

**服务地址**:
- 前端: http://localhost:3001
- 后端: http://localhost:8080
- API 文档: http://localhost:8080/swagger-ui/

**日志查看**:
```bash
# 后端日志
tail -f backend-test.log

# 前端日志
tail -f frontend.log
```

### 3. 环境变量配置

Justfile 预配置了以下环境变量：

```bash
# Rust 配置
export RUST_BACKTRACE := "1"

# LLM 配置
export LLM_PROVIDER := "zhipu"
export LLM_MODEL := "glm-4-plus"
export ZHIPU_API_KEY := "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"

# Embedder 配置
export EMBEDDER_PROVIDER := "fastembed"
export EMBEDDER_MODEL := "BAAI/bge-small-en-v1.5"

# 库路径
export DYLD_LIBRARY_PATH := "./lib:./target/release"
export ORT_DYLIB_PATH := "./lib/libonnxruntime.1.22.0.dylib"
```

### 4. 创建使用指南

创建了 `JUSTFILE_GUIDE.md` 文档，包含：

- Just 安装说明（macOS、Linux、Cargo）
- 快速开始指南
- 所有命令的详细说明
- 完整工作流示例
- 常见问题解答
- 提示和技巧

## 使用示例

### 示例 1: 开发工作流

```bash
# 1. 清理并构建
just rebuild

# 2. 运行测试
just test

# 3. 启动开发模式（无认证）
just start-server-no-auth

# 4. 在另一个终端查看日志
just logs backend

# 5. 开发完成后停止服务
just stop
```

### 示例 2: MCP 功能验证

```bash
# 1. 构建 MCP 服务器
just build-mcp

# 2. 测试 MCP 基础功能
just test-mcp

# 3. 测试 Chat 功能
just mcp-test-chat

# 4. 验证所有工具
just mcp-verify
```

### 示例 3: 全栈测试

```bash
# 1. 启动全栈服务
just start-full

# 2. 在浏览器中访问
# 前端: http://localhost:3001
# 后端: http://localhost:8080

# 3. 查看健康状态
just health

# 4. 停止所有服务
just stop
```

### 示例 4: 生产部署

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

## 优势

### 1. 统一接口
- 所有项目管理任务通过 `just <command>` 统一调用
- 无需记忆复杂的 shell 脚本路径和参数

### 2. 简化操作
- 将复杂的多步骤操作封装为单个命令
- 例如：`just quick-start` 替代手动启动多个服务

### 3. 自文档化
- 使用 `just --list` 查看所有可用命令
- 每个命令都有清晰的中文注释

### 4. 环境一致性
- 统一的环境变量配置
- 确保所有开发者使用相同的配置

### 5. 错误处理
- 脚本包含健康检查和错误处理
- 自动验证服务启动状态

## 技术细节

### Justfile 语法

```makefile
# 注释
command-name:
    @echo "描述"
    shell command

# 带参数的命令
command-with-args arg1 arg2:
    @echo "参数: {{arg1}} {{arg2}}"
    
# 环境变量
export VAR := "value"
```

### 脚本集成方式

1. **直接调用**: `@bash script.sh`
2. **内联命令**: 直接在 justfile 中编写命令
3. **组合命令**: 使用 `&&` 连接多个命令

## 文件清单

### 新增文件
- `justfile` - Just 命令定义文件
- `JUSTFILE_GUIDE.md` - 使用指南
- `docs/JUSTFILE_INTEGRATION_REPORT.md` - 本报告

### 集成的脚本
- `start_server_no_auth.sh` - 无认证模式启动
- `start_server_with_correct_onnx.sh` - ONNX 修复版启动
- `start_full_stack.sh` - 全栈启动

### 其他相关脚本
- `scripts/build.sh` - 构建脚本
- `scripts/init_db.sh` - 数据库初始化
- `scripts/run_comprehensive_tests.sh` - 综合测试
- `examples/mcp-stdio-server/test_server.sh` - MCP 测试

## 后续改进建议

1. **添加更多快捷命令**
   - `just test-all` - 运行所有类型的测试
   - `just lint` - 运行所有代码质量检查
   - `just ci` - 模拟 CI 流程

2. **增强日志管理**
   - 统一日志目录
   - 日志轮转配置
   - 日志聚合查看

3. **添加配置管理**
   - 支持多环境配置（dev、staging、prod）
   - 配置文件验证
   - 配置模板生成

4. **集成更多工具**
   - 数据库迁移工具
   - 性能分析工具
   - 监控工具

5. **文档增强**
   - 添加视频教程
   - 添加故障排查指南
   - 添加最佳实践文档

## 总结

通过集成 justfile，我们成功地：

1. ✅ 统一了项目管理命令接口
2. ✅ 简化了开发和部署流程
3. ✅ 提高了团队协作效率
4. ✅ 改善了新人上手体验
5. ✅ 增强了项目的可维护性

Justfile 现在是 AgentMem 项目的核心工具，为开发、测试、部署提供了统一、简洁、强大的命令行接口。

## 参考资料

- [Just 官方文档](https://just.systems/)
- [Just GitHub 仓库](https://github.com/casey/just)
- [AgentMem 项目文档](../README.md)
- [Justfile 使用指南](../JUSTFILE_GUIDE.md)

