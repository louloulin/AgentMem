# AgentMem 华为 MaaS UI Chat 功能验证指南

**日期**: 2025-11-19  
**目标**: 通过 AgentMem UI 验证华为 MaaS Chat 功能

---

## 🚀 启动服务

### 1. 前端 UI 启动

✅ **已启动成功！**

```bash
cd agentmem-ui
npm run dev
```

**访问地址**: http://localhost:3001

**状态**: 
- ✓ Starting...
- ✓ Ready in 2.3s
- Local: http://localhost:3001
- Network: http://192.168.31.5:3001

### 2. 后端服务启动

**方式 1: 使用启动脚本（推荐）**

```bash
# 如果已编译
./start_server_no_auth.sh --skip-build

# 如果需要重新编译
./start_server_no_auth.sh --build-server
```

**方式 2: 直接编译和启动**

```bash
# 1. 设置环境变量
export MAAS_API_KEY="your_huawei_maas_api_key"

# 2. 编译（带 lumosai 特性）
cargo build --release --bin agent-mem-server --features lumosai

# 3. 启动
ENABLE_AUTH=false \
DYLD_LIBRARY_PATH=./lib:./target/release \
ORT_DYLIB_PATH=./lib/libonnxruntime.1.22.0.dylib \
./target/release/agent-mem-server
```

**服务端口**: http://localhost:8000

---

## 📋 验证步骤

### 步骤 1: 访问 AgentMem UI

1. 打开浏览器访问 http://localhost:3001
2. 你应该看到 AgentMem 的主界面

### 步骤 2: 创建 MaaS Agent

在 UI 中创建一个使用华为 MaaS 的 Agent：

**配置参数**:

```json
{
  "name": "MaaS 测试助手",
  "description": "基于华为 MaaS 的智能对话助手",
  "system": "你是一个由华为 MaaS 平台驱动的 AI 助手，请用中文友好地回答问题。",
  "llm_config": {
    "provider": "maas",
    "model": "deepseek-v3.2-exp",
    "api_key": null
  }
}
```

**或者使用 curl 创建**:

```bash
curl -X POST http://localhost:8000/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MaaS 测试助手",
    "description": "基于华为 MaaS 的智能对话助手",
    "system": "你是一个由华为 MaaS 平台驱动的 AI 助手。",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp",
      "api_key": null
    }
  }'
```

### 步骤 3: 开始对话

在 UI 的 Chat 页面：

1. **选择刚创建的 MaaS Agent**
2. **发送测试消息**:

#### 测试 1: 基本对话

```
你好，请介绍一下你自己
```

**预期响应**: AI 应该介绍自己是基于华为 MaaS 的助手

#### 测试 2: 验证记忆功能（多轮对话）

**第一轮**:
```
我的名字叫小明，我是一名软件工程师，我喜欢用 Rust 编程
```

**第二轮**:
```
我叫什么名字？我的职业是什么？我喜欢用什么语言编程？
```

**预期响应**: AI 应该准确回答：
- 名字：小明
- 职业：软件工程师
- 语言：Rust

#### 测试 3: 任务执行

```
请帮我写一个用 Python 计算斐波那契数列的函数
```

**预期响应**: AI 应该生成正确的 Python 代码

#### 测试 4: 中文能力

```
请用中文解释什么是大语言模型，并举例说明其应用场景
```

**预期响应**: AI 应该用流畅的中文回答

### 步骤 4: 验证 Memory 存储

#### 方式 1: 通过 UI 查看

在 UI 的 "Memories" 或 "历史记录" 页面查看：
- 对话是否被正确存储
- 记忆数量是否增加

#### 方式 2: 通过 API 查看

```bash
# 获取 Agent 的所有记忆
curl -X GET "http://localhost:8000/api/v1/agents/{agent_id}/memories" \
  -H "Content-Type: application/json"

# 搜索特定记忆
curl -X POST "http://localhost:8000/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "{agent_id}",
    "user_id": "default",
    "query": "小明",
    "limit": 10
  }'
```

---

## 🔍 基于 MCP 的验证

### MCP (Model Context Protocol) 概述

MCP 是一个标准化协议，用于连接 AI 系统和外部工具/数据源。AgentMem 通过 MCP 可以：

- ✅ 访问记忆数据
- ✅ 执行工具调用
- ✅ 与其他系统集成

### 验证 MCP 集成

#### 1. 检查 MCP 配置

查看 `.mcp.json` 或配置文件：

```bash
cat .mcp.json
```

**预期内容**:

```json
{
  "mcpServers": {
    "memory": {
      "command": "node",
      "args": ["./mcp-server.js"]
    }
  }
}
```

#### 2. 启动 MCP Server（如果有）

```bash
# 检查是否有 MCP 示例
ls -la target/release/examples/

# 启动 MCP stdio server
./target/release/examples/mcp-stdio-server
```

#### 3. 通过 MCP 测试 Chat 功能

如果 UI 支持 MCP 协议，你可以：

1. **在 UI 设置中配置 MCP**
2. **选择 MCP 作为通信方式**
3. **发送测试消息**

#### 4. 验证 MCP 工具调用

在 Chat 中测试工具调用（如果 Agent 配置了 tools）：

```
请帮我搜索关于"Rust 编程"的记忆
```

**预期**: 
- AI 应该调用 MCP 工具
- 返回相关的记忆结果
- 在 UI 中显示工具调用的过程

---

## 📊 验证检查清单

### 基础功能

- [ ] 前端 UI 正常启动（http://localhost:3001）
- [ ] 后端服务正常启动（http://localhost:8000）
- [ ] 可以访问 UI 主界面
- [ ] 可以创建 MaaS Agent
- [ ] Agent 列表正常显示

### Chat 功能

- [ ] 可以打开 Chat 页面
- [ ] 可以选择 MaaS Agent
- [ ] 可以发送消息
- [ ] AI 正常响应
- [ ] 响应时间合理（<5 秒）

### 华为 MaaS 特性

- [ ] MaaS Provider 正常工作
- [ ] 可以使用 deepseek-v3.2-exp 模型
- [ ] 中文对话流畅
- [ ] 可以切换其他 MaaS 模型

### 记忆功能

- [ ] 对话自动存储
- [ ] 多轮对话记住上下文
- [ ] 可以查看历史记忆
- [ ] 不同用户的记忆隔离
- [ ] 记忆搜索功能正常

### MCP 集成（可选）

- [ ] MCP Server 正常启动
- [ ] UI 可以通过 MCP 通信
- [ ] MCP 工具调用正常
- [ ] 工具调用结果正确返回

---

## 🐛 常见问题和解决方案

### Q1: 后端服务启动失败

**错误**: `No such file or directory`

**解决方案**:

```bash
# 重新编译
cargo build --release --bin agent-mem-server --features lumosai

# 检查是否生成
ls -la target/release/agent-mem-server
```

### Q2: UI 无法连接后端

**错误**: `Failed to fetch` 或 `Network error`

**检查**:

1. 后端是否运行:
   ```bash
   curl http://localhost:8000/api/v1/health
   ```

2. 检查端口:
   ```bash
   lsof -i :8000
   ```

3. 检查 CORS 设置

**解决方案**: 确保后端启动并监听 8000 端口

### Q3: MaaS API 调用失败

**错误**: `API key not configured` 或 `401 Unauthorized`

**检查**:

```bash
# 检查环境变量
echo $MAAS_API_KEY

# 检查 Agent 配置
curl http://localhost:8000/api/v1/agents/{agent_id}
```

**解决方案**:

```bash
# 设置环境变量
export MAAS_API_KEY="your_api_key"

# 重启服务
./start_server_no_auth.sh --skip-build
```

### Q4: 记忆功能不工作

**症状**: 多轮对话不记得之前的内容

**检查**:

1. 查看日志中是否有 Memory 相关错误
2. 检查数据库连接
3. 验证 Memory Backend 是否正确配置

**解决方案**:

```bash
# 查看日志
tail -f backend-no-auth.log | grep -i memory

# 测试 Memory API
curl -X POST http://localhost:8000/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "xxx", "query": "test"}'
```

### Q5: UI 显示异常

**症状**: 页面布局错乱或功能缺失

**解决方案**:

```bash
# 重新安装依赖
cd agentmem-ui
rm -rf node_modules package-lock.json
npm install

# 重启 UI
npm run dev
```

---

## 📸 验证截图建议

在验证过程中，建议截图保存以下内容：

1. **UI 主界面**
   - Agent 列表
   - 创建 Agent 表单

2. **Chat 界面**
   - 对话窗口
   - AI 响应
   - 多轮对话示例

3. **Memory 界面**
   - 记忆列表
   - 记忆详情
   - 搜索结果

4. **开发者工具**
   - Network 请求/响应
   - Console 日志
   - API 调用详情

---

## 📝 验证报告模板

完成验证后，可以使用以下模板记录结果：

```markdown
# AgentMem 华为 MaaS UI 验证报告

## 测试环境
- 日期: YYYY-MM-DD
- 后端版本: x.x.x
- 前端版本: x.x.x
- MaaS 模型: deepseek-v3.2-exp

## 测试结果

### 基础功能
| 功能 | 状态 | 说明 |
|------|------|------|
| 前端启动 | ✅/❌ | |
| 后端启动 | ✅/❌ | |
| Agent 创建 | ✅/❌ | |

### Chat 功能
| 测试项 | 状态 | 响应时间 | 说明 |
|--------|------|----------|------|
| 基本对话 | ✅/❌ | Xms | |
| 多轮对话 | ✅/❌ | Xms | |
| 中文对话 | ✅/❌ | Xms | |

### 记忆功能
| 测试项 | 状态 | 说明 |
|--------|------|------|
| 自动存储 | ✅/❌ | |
| 上下文记忆 | ✅/❌ | |
| 记忆查询 | ✅/❌ | |

### MCP 集成
| 测试项 | 状态 | 说明 |
|--------|------|------|
| MCP Server | ✅/❌ | |
| 工具调用 | ✅/❌ | |

## 遇到的问题
1. ...
2. ...

## 建议
1. ...
2. ...
```

---

## 🎯 完整验证流程脚本

创建一个自动化验证脚本：

```bash
#!/bin/bash
# verify_maas_ui.sh

echo "🚀 AgentMem 华为 MaaS UI 验证脚本"
echo ""

# 1. 检查环境变量
echo "1️⃣ 检查环境变量..."
if [ -z "$MAAS_API_KEY" ]; then
    echo "❌ MAAS_API_KEY 未设置"
    exit 1
fi
echo "✅ MAAS_API_KEY 已设置"

# 2. 启动后端
echo ""
echo "2️⃣ 启动后端服务..."
./start_server_no_auth.sh --skip-build &
BACKEND_PID=$!
sleep 5

# 3. 检查后端健康
echo ""
echo "3️⃣ 检查后端健康..."
if curl -s http://localhost:8000/api/v1/health > /dev/null; then
    echo "✅ 后端服务正常"
else
    echo "❌ 后端服务异常"
    kill $BACKEND_PID
    exit 1
fi

# 4. 启动前端
echo ""
echo "4️⃣ 启动前端 UI..."
cd agentmem-ui
npm run dev &
FRONTEND_PID=$!
sleep 3
cd ..

# 5. 创建测试 Agent
echo ""
echo "5️⃣ 创建测试 Agent..."
AGENT_RESPONSE=$(curl -s -X POST http://localhost:8000/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MaaS 验证助手",
    "system": "你是一个测试助手",
    "llm_config": {
      "provider": "maas",
      "model": "deepseek-v3.2-exp",
      "api_key": null
    }
  }')

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" == "null" ]; then
    echo "❌ Agent 创建失败"
    kill $BACKEND_PID $FRONTEND_PID
    exit 1
fi
echo "✅ Agent 创建成功: $AGENT_ID"

# 6. 测试 Chat
echo ""
echo "6️⃣ 测试 Chat 功能..."
CHAT_RESPONSE=$(curl -s -X POST "http://localhost:8000/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，这是一条测试消息",
    "user_id": "test-user"
  }')

SUCCESS=$(echo $CHAT_RESPONSE | jq -r '.success')
if [ "$SUCCESS" == "true" ]; then
    echo "✅ Chat 功能正常"
    echo "AI 回复: $(echo $CHAT_RESPONSE | jq -r '.data.content' | head -c 100)..."
else
    echo "❌ Chat 功能异常"
    kill $BACKEND_PID $FRONTEND_PID
    exit 1
fi

# 7. 打开浏览器
echo ""
echo "7️⃣ 打开浏览器..."
open http://localhost:3001

echo ""
echo "🎉 验证完成！"
echo ""
echo "服务已启动:"
echo "  - 后端: http://localhost:8000"
echo "  - 前端: http://localhost:3001"
echo "  - Test Agent ID: $AGENT_ID"
echo ""
echo "按 Ctrl+C 停止服务"

# 等待
wait
```

---

## 🎉 总结

### 快速验证步骤

1. **设置环境变量**
   ```bash
   export MAAS_API_KEY="your_key"
   ```

2. **启动服务**
   ```bash
   # 后端
   ./start_server_no_auth.sh --build-server
   
   # 前端
   cd agentmem-ui && npm run dev
   ```

3. **访问 UI**
   - 前端: http://localhost:3001
   - 后端: http://localhost:8000

4. **创建 MaaS Agent 并测试对话**

5. **验证记忆功能**

6. **（可选）测试 MCP 集成**

### 验证成功标志

- ✅ UI 正常显示
- ✅ 可以创建 MaaS Agent
- ✅ Chat 功能正常
- ✅ AI 响应及时准确
- ✅ 多轮对话记住上下文
- ✅ 记忆自动存储和检索

---

**文档版本**: v1.0  
**最后更新**: 2025-11-19  
**作者**: AgentMem Team
