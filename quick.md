# AgentMem 快速开始指南

**版本**: v1.0  
**更新日期**: 2025-11-05  
**预计阅读时间**: 5分钟

---

## 🚀 快速启动

### 一键启动脚本

#### 启动完整服务（后端+前端）

```bash
# 方式1: 使用 just 命令（推荐）
just start-full-with-plugins

# 方式2: 使用启动脚本
bash start_full_stack.sh
```

#### 仅启动后端

```bash
# 方式1: 使用 just 命令
just start-server-with-plugins

# 方式2: 使用启动脚本（推荐，包含完整配置）
bash start_backend.sh
```

#### 仅启动前端

```bash
cd agentmem-ui && npm run dev
```

### 停止服务

```bash
# 停止所有服务
just stop

# 或手动停止
pkill -f agent-mem-server
pkill -f "next dev"
```

---

## ✅ 启动验证

### 检查服务状态

```bash
# 检查后端健康
curl http://localhost:8080/health | jq

# 检查前端
curl -I http://localhost:3001

# 查看后端日志
tail -f backend-no-auth.log

# 查看前端日志
tail -f frontend.log
```

### 预期结果

如果一切正常，你应该看到：

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Database connection successful"
    },
    "memory_system": {
      "status": "healthy",
      "message": "Memory system operational"
    }
  }
}
```

---

## 🌐 访问服务

### Web 界面

| 服务 | URL | 说明 |
|------|-----|------|
| **主页** | http://localhost:3001 | 前端主页 |
| **记忆管理** | http://localhost:3001/admin/memories | 记忆列表与搜索 |
| **知识图谱** | http://localhost:3001/admin/graph | 可视化记忆关系 |
| **插件管理** | http://localhost:3001/admin/plugins | WASM插件管理 |
| **API文档** | http://localhost:8080/swagger-ui/ | Swagger UI |
| **健康检查** | http://localhost:8080/health | 服务状态 |

### API 端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/v1/memories` | GET | 获取记忆列表 |
| `/api/v1/memories` | POST | 添加新记忆 |
| `/api/v1/memories/search` | POST | 向量搜索 |
| `/api/v1/plugins` | GET | 获取插件列表 |
| `/api/v1/plugins` | POST | 注册插件 |
| `/health` | GET | 健康检查 |
| `/metrics` | GET | 性能指标 |

---

## 📋 核心功能

### 1. 记忆管理

#### 添加记忆

```bash
curl -X POST "http://localhost:8080/api/v1/memories" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "agent_id": "my-agent",
    "content": "这是一条测试记忆",
    "memory_type": "Working",
    "importance": 0.8
  }'
```

#### 搜索记忆

```bash
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "query": "AI产品",
    "limit": 5
  }' | jq
```

#### 记忆类型

- `Working` - 工作记忆
- `Episodic` - 情景记忆
- `Semantic` - 语义记忆
- `Procedural` - 程序记忆
- `Factual` - 事实记忆
- `Core` - 核心记忆
- `Resource` - 资源记忆
- `Knowledge` - 知识记忆
- `Contextual` - 上下文记忆

### 2. 向量搜索

#### 特点
- ✅ **语义理解**: 基于 BAAI/bge-small-en-v1.5 (384维)
- ✅ **高性能**: 平均响应时间 3-6ms
- ✅ **准确性**: 100%测试通过率
- ✅ **同义词**: 自动识别相关概念

#### 使用示例

```bash
# 1. 搜索 AI 产品相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "AI产品开发", "limit": 3}'

# 2. 搜索技术团队相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "工程师团队", "limit": 3}'

# 3. 搜索融资相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "B轮融资", "limit": 3}'
```

#### 在 UI 中使用

1. 访问 http://localhost:3001/admin/memories
2. 在顶部搜索框输入关键词
3. 按 Enter 或点击搜索
4. 查看相关性排序的结果

### 3. WASM 插件系统

#### 查看插件

```bash
curl http://localhost:8080/api/v1/plugins \
  -H "X-User-ID: default" | jq
```

#### 注册插件

```bash
curl -X POST "http://localhost:8080/api/v1/plugins" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -d '{
    "name": "hello-plugin",
    "description": "Hello World Plugin",
    "version": "1.0.0",
    "plugin_type": "memory_processor",
    "wasm_path": "target/wasm32-wasip1/release/hello_plugin.wasm"
  }'
```

#### 编译插件

```bash
# 编译所有示例插件
bash build_plugins.sh

# 编译单个插件
cd examples/plugins/hello_plugin
cargo build --target wasm32-wasip1 --release
```

### 4. 知识图谱

#### 功能特点
- ✅ **力导向布局**: 自动优化节点位置
- ✅ **交互式**: 拖拽、缩放、选择
- ✅ **智能标签**: 四种显示模式
- ✅ **统计分析**: 节点、边、度数、密度

#### 访问
http://localhost:3001/admin/graph

#### 控制
- **鼠标拖拽**: 移动节点
- **滚轮**: 缩放
- **点击**: 选择节点
- **悬停**: 查看详情
- **搜索**: 过滤节点

---

## 🧪 测试数据

系统已预置5条测试记忆：

| 类型 | 内容 | 重要性 |
|------|------|--------|
| Episodic | AI产品研发 | 0.9 |
| Semantic | 技术团队（20名工程师） | 0.85 |
| Episodic | B轮融资（5000万美元） | 0.95 |
| Working | 产品迭代（优化体验） | 0.8 |
| Working | 市场拓展（金融、医疗） | 0.7 |

### 测试搜索

尝试这些查询：

```bash
# 1. AI 相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "AI产品", "limit": 3}' | jq '.data[0].content'

# 2. 技术相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "工程师", "limit": 3}' | jq '.data[0].content'

# 3. 融资相关
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "融资", "limit": 3}' | jq '.data[0].content'
```

---

## 🔧 配置说明

### 环境变量

```bash
# Embedder 配置（向量搜索）
EMBEDDER_PROVIDER="fastembed"
EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# LLM 配置
LLM_PROVIDER="zhipu"
LLM_MODEL="glm-4-plus"
ZHIPU_API_KEY="your-api-key"

# 认证配置
ENABLE_AUTH="false"
SERVER_ENABLE_AUTH="false"
AGENT_MEM_ENABLE_AUTH="false"

# ONNX Runtime（macOS）
DYLD_LIBRARY_PATH="$PWD/lib:$PWD/target/release"
ORT_DYLIB_PATH="$PWD/lib/libonnxruntime.1.22.0.dylib"
```

### 服务配置

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| 后端端口 | 8080 | HTTP API 端口 |
| 前端端口 | 3001 | Next.js 开发服务器 |
| 数据库 | SQLite | 内嵌式数据库 |
| 认证 | Disabled | 开发环境禁用 |

---

## 📊 性能指标

### 响应时间

| 操作 | 平均时间 | 说明 |
|------|----------|------|
| 添加记忆 | ~50ms | 包含embedding生成 |
| 向量搜索 | 3-6ms | 非常快速 |
| 列表查询 | <1ms | 几乎实时 |
| 插件加载 | <10ms | LRU缓存 |

### 资源使用

| 资源 | 占用 |
|------|------|
| 内存 | ~1.2GB |
| CPU | 低 |
| 磁盘 | 根据数据量 |

---

## 🐛 故障排查

### 后端无法启动

```bash
# 1. 检查端口占用
lsof -i :8080

# 2. 检查 ONNX Runtime
ls -la lib/libonnxruntime.1.22.0.dylib

# 3. 查看日志
tail -100 backend-no-auth.log

# 4. 尝试前台运行
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
./target/release/agent-mem-server
```

### 前端无法启动

```bash
# 1. 检查端口占用
lsof -i :3001

# 2. 重新安装依赖
cd agentmem-ui
rm -rf node_modules package-lock.json
npm install

# 3. 尝试重启
npm run dev
```

### 搜索返回空结果

可能原因：
1. 记忆没有 embeddings → 重新添加记忆
2. Embedder 未启动 → 检查配置和日志
3. 查询关键词不匹配 → 尝试不同关键词

### 插件加载失败

```bash
# 1. 检查 WASM 文件是否存在
ls -la target/wasm32-wasip1/release/*.wasm

# 2. 重新编译插件
bash build_plugins.sh

# 3. 检查插件路径
curl http://localhost:8080/api/v1/plugins | jq
```

---

## 📚 详细文档

### 核心文档

| 文档 | 说明 |
|------|------|
| [plugin.md](./plugin.md) | WASM插件系统设计与实现 |
| [VECTOR_SEARCH_TEST_REPORT.md](./VECTOR_SEARCH_TEST_REPORT.md) | 向量搜索测试报告 |
| [QUICK_START_SEARCH.md](./QUICK_START_SEARCH.md) | 搜索功能快速指南 |
| [KNOWLEDGE_GRAPH_OPTIMIZATION.md](./KNOWLEDGE_GRAPH_OPTIMIZATION.md) | 知识图谱优化 |
| [EMBEDDER_FIX_REPORT.md](./EMBEDDER_FIX_REPORT.md) | Embedder修复报告 |

### 功能文档

| 文档 | 说明 |
|------|------|
| [PLUGIN_UI_IMPLEMENTATION.md](./PLUGIN_UI_IMPLEMENTATION.md) | 插件UI实现 |
| [PLUGIN_UI_FEATURES.md](./PLUGIN_UI_FEATURES.md) | 插件UI功能 |
| [FULL_STACK_PLUGIN_VERIFICATION.md](./FULL_STACK_PLUGIN_VERIFICATION.md) | 全栈验证 |
| [E2E_WASM_PLUGIN_VERIFICATION.md](./E2E_WASM_PLUGIN_VERIFICATION.md) | E2E测试 |

---

## 🔄 开发流程

### 添加新功能

```bash
# 1. 创建功能分支
git checkout -b feature/my-feature

# 2. 修改代码
# ...

# 3. 运行测试
cargo test
cd agentmem-ui && npm test

# 4. 启动验证
just start-full-with-plugins

# 5. 提交代码
git add .
git commit -m "Add: my feature"
```

### 添加新插件

```bash
# 1. 创建插件项目
cd examples/plugins
cargo new my_plugin --lib

# 2. 添加依赖
cd my_plugin
cargo add agent-mem-plugin-sdk

# 3. 实现插件
# 编辑 src/lib.rs

# 4. 配置编译目标
# 编辑 Cargo.toml 添加:
# [lib]
# crate-type = ["cdylib"]

# 5. 编译
cargo build --target wasm32-wasip1 --release

# 6. 测试
cargo test
```

### 更新文档

```bash
# 1. 编辑 Markdown 文档
vim quick.md

# 2. 更新版本号
# 修改文档顶部的版本信息

# 3. 提交
git add quick.md
git commit -m "Doc: update quick.md"
```

---

## 💡 最佳实践

### 记忆管理

1. **设置合理的重要性分数**
   - 核心信息: 0.9-1.0
   - 重要信息: 0.7-0.9
   - 一般信息: 0.5-0.7
   - 临时信息: 0.0-0.5

2. **选择正确的记忆类型**
   - 事件、对话 → `Episodic`
   - 知识、概念 → `Semantic`
   - 流程、步骤 → `Procedural`
   - 当前任务 → `Working`

3. **添加有意义的内容**
   - ✅ "团队决定采用微服务架构，预计Q2实施"
   - ❌ "开会讨论了一些事情"

### 向量搜索

1. **使用核心关键词**
   - ✅ "AI产品"
   - ❌ "我想找关于AI产品的相关信息"

2. **利用同义词**
   - "技术团队" = "工程师" = "研发人员"

3. **组合查询**
   - "AI技术" (同时相关两个概念)

### 插件开发

1. **遵循命名规范**
   - 插件名: `my-plugin` (kebab-case)
   - 函数名: `process_memory` (snake_case)

2. **实现错误处理**
   ```rust
   use anyhow::{Result, Context};
   
   fn process(input: &str) -> Result<String> {
       // 处理逻辑
       Ok(result)
   }
   ```

3. **添加测试**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_process() {
           // 测试逻辑
       }
   }
   ```

---

## 🎯 快速命令参考

### 常用命令

```bash
# 启动
just start-full-with-plugins          # 启动全栈（推荐）
bash start_backend.sh                 # 仅启动后端
bash start_full_stack.sh              # 启动全栈（脚本）

# 停止
just stop                             # 停止所有服务

# 构建
just build-release                    # 构建生产版本
bash build_plugins.sh                 # 编译所有插件

# 测试
cargo test                            # 运行 Rust 测试
cd agentmem-ui && npm test            # 运行前端测试

# 日志
tail -f backend-no-auth.log           # 后端日志
tail -f frontend.log                  # 前端日志

# 健康检查
curl http://localhost:8080/health | jq
curl http://localhost:3001

# 数据操作
curl http://localhost:8080/api/v1/memories | jq  # 列表
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{"query": "AI", "limit": 5}' | jq          # 搜索
```

---

## 🌟 功能亮点

### 1. 高性能向量搜索 ⚡
- 3-6ms 响应时间
- 384维语义向量
- 100%测试准确率

### 2. 智能知识图谱 🕸️
- 力导向自动布局
- 交互式探索
- 实时统计分析

### 3. WASM 插件系统 🧩
- 安全沙箱隔离
- 热加载支持
- LRU 智能缓存

### 4. 现代化 UI 💎
- Next.js + React
- shadcn/ui 组件
- 响应式设计

---

## 🎉 开始使用

现在你已经了解了 AgentMem 的核心功能！

### 第一步：启动服务
```bash
just start-full-with-plugins
```

### 第二步：访问 UI
打开浏览器访问：http://localhost:3001

### 第三步：尝试搜索
1. 进入记忆管理页面
2. 搜索 "AI产品"
3. 查看结果

### 第四步：探索更多
- 查看知识图谱
- 管理插件
- 添加记忆

---

## 📞 获取帮助

### 常见问题
参考 [故障排查](#故障排查) 章节

### 详细文档
查看 `docs/` 目录下的详细文档

### 日志分析
```bash
# 后端日志
tail -100 backend-no-auth.log

# 前端日志
tail -100 frontend.log
```

---

**祝使用愉快！** 🚀

