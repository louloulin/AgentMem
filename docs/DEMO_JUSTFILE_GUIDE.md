# 演示运行脚本指南（Justfile）

## 概述

所有演示相关的运行脚本已固化到 `justfile` 中，按照 `x.md` 演示计划组织。

## 快速开始

### 一键完整演示
```bash
just demo-full
```
这会执行完整的演示流程：
1. 构建项目（release 模式）
2. 启动服务（后端+前端）
3. 创建演示数据（30条记忆）
4. 验证数据
5. 验证 UI 功能
6. 打开浏览器

### 快速演示（服务已运行）
```bash
just demo-quick
```
假设服务已经运行，只执行：
1. 创建演示数据
2. 验证数据
3. 打开浏览器

## 命令列表

### 准备阶段

#### `just demo-prepare`
准备演示环境：
- 停止现有服务
- 初始化数据库
- 清理临时文件

**使用场景**: 全新开始演示

#### `just demo-create-data`
创建演示数据（30条记忆）：
- 按照 `x.md` 计划创建
- 包含4个阶段的数据
- 自动创建或使用现有 Agent

**使用场景**: 需要重新创建演示数据

#### `just demo-verify-data`
验证演示数据：
- 检查记忆数量
- 检查 Agent 数量
- 显示统计信息

**使用场景**: 验证数据是否正确创建

### 启动服务

#### `just demo-start`
启动演示服务：
- 启动后端 API（端口 8080）
- 启动前端 UI（端口 3001）
- 检查服务健康状态

**使用场景**: 启动演示环境

### 验证测试

#### `just demo-verify-ui`
验证 UI 功能（完整测试）：
- 运行所有测试用例（7个搜索测试）
- 验证过滤功能
- 验证 UI 页面可访问性
- 生成验证报告

**使用场景**: 全面验证功能

#### `just demo-test-search`
测试搜索功能（7个测试用例）：
1. 基础信息检索 - '王总'
2. 关系网络查询 - '张总'
3. 项目状态查询 - 'AI产品'
4. 历史对话查询 - '融资'
5. 个性化建议 - '会议'
6. 语义搜索 - '技术相关的工作'
7. 团队成员查询 - '陈副总'

**使用场景**: 快速测试搜索功能

### 浏览器操作

#### `just demo-open-browser`
打开浏览器验证页面：
- 记忆管理页面 (`/admin/memories`)
- Agent 管理页面 (`/admin/agents`)
- 关系图谱页面 (`/admin/graph`)

**使用场景**: 在浏览器中手动验证

### 状态检查

#### `just demo-status`
检查演示环境状态：
- 后端服务状态
- 前端服务状态
- 演示数据统计
- Agent 统计

**使用场景**: 检查环境是否就绪

### 数据管理

#### `just demo-reset`
重置演示数据：
- 警告并等待5秒
- 重新创建演示数据

**使用场景**: 需要重新开始演示

### 帮助

#### `just demo-help`
显示演示命令帮助：
- 列出所有演示命令
- 说明使用场景
- 提供文档链接

## 使用场景示例

### 场景1：全新演示环境

```bash
# 1. 准备环境
just demo-prepare

# 2. 构建项目
just build-release

# 3. 启动服务
just demo-start

# 4. 创建数据
just demo-create-data

# 5. 验证
just demo-verify-ui

# 6. 打开浏览器
just demo-open-browser
```

### 场景2：服务已运行，只需数据

```bash
# 快速准备
just demo-quick
```

### 场景3：验证搜索功能

```bash
# 测试搜索
just demo-test-search
```

### 场景4：检查环境状态

```bash
# 检查状态
just demo-status
```

## 环境变量

演示相关的环境变量定义在 `justfile` 中：

```justfile
export DEMO_API_URL := "http://localhost:8080"
export DEMO_UI_URL := "http://localhost:3001"
export DEMO_USER_ID := "default"
export DEMO_ORG_ID := "default-org"
```

可以在运行命令前覆盖这些变量：

```bash
DEMO_API_URL="http://other-host:8080" just demo-status
```

## 演示流程（按照 x.md 计划）

### 第一部分：基础演示（5分钟）

1. **清理和重置**（30秒）
   ```bash
   just demo-prepare
   ```

2. **创建基础记忆**（2分钟）
   ```bash
   just demo-create-data
   ```

3. **检索测试**（2.5分钟）
   ```bash
   just demo-test-search
   # 或在浏览器中手动测试
   just demo-open-browser
   ```

### 第二部分：深度演示（7分钟）

4. **项目记忆创建**（2分钟）
   - 数据已在 `demo-create-data` 中创建
   - 验证：`just demo-verify-data`

5. **个性化服务**（2.5分钟）
   - 在浏览器中测试搜索"会议"
   - 验证偏好记忆应用

6. **跨会话记忆**（2.5分钟）
   - 测试历史对话查询
   - 验证长期记忆能力

### 第三部分：总结和Q&A（3分钟）

7. **价值总结**
   - 展示搜索功能
   - 展示记忆类型过滤
   - 展示分页功能

## 故障排查

### 问题1：服务未启动

**症状**: `demo-status` 显示服务未运行

**解决**:
```bash
just demo-start
```

### 问题2：数据未创建

**症状**: `demo-verify-data` 显示记忆数量为0

**解决**:
```bash
just demo-create-data
just demo-verify-data
```

### 问题3：搜索测试失败

**症状**: `demo-test-search` 返回错误

**检查**:
1. 服务是否运行：`just demo-status`
2. 数据是否存在：`just demo-verify-data`
3. API 是否正常：`curl http://localhost:8080/health`

## 相关文档

- **演示计划**: `x.md`
- **浏览器验证指南**: `docs/BROWSER_VERIFICATION_GUIDE.md`
- **UI验证报告**: `docs/UI_FUNCTIONALITY_VERIFICATION_REPORT.md`
- **问题分析**: `docs/COMPREHENSIVE_ANALYSIS_AND_FIXES.md`

## 最佳实践

1. **演示前**: 运行 `just demo-status` 检查环境
2. **演示中**: 使用 `just demo-open-browser` 打开浏览器
3. **演示后**: 运行 `just demo-verify-ui` 验证功能
4. **重置**: 使用 `just demo-reset` 清理数据

## 命令速查表

| 命令 | 用途 | 时间 |
|------|------|------|
| `just demo-full` | 完整演示流程 | ~5分钟 |
| `just demo-quick` | 快速演示 | ~30秒 |
| `just demo-prepare` | 准备环境 | ~10秒 |
| `just demo-create-data` | 创建数据 | ~30秒 |
| `just demo-verify-data` | 验证数据 | ~5秒 |
| `just demo-verify-ui` | 验证UI | ~1分钟 |
| `just demo-test-search` | 测试搜索 | ~10秒 |
| `just demo-open-browser` | 打开浏览器 | ~3秒 |
| `just demo-status` | 检查状态 | ~5秒 |
| `just demo-reset` | 重置数据 | ~30秒 |

---

**创建时间**: 2025-11-04
**状态**: ✅ 已完成，可以开始使用

