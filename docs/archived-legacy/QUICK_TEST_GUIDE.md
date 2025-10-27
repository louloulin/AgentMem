# AgentMem 极简方案 - 快速测试指南

## 问题诊断

如果遇到 "Internal Server Error"，按以下步骤诊断：

### 1. 检查后端服务

```bash
# 健康检查
curl http://localhost:8080/health

# 应该返回: {"status":"healthy",...}
```

### 2. 检查前端服务

```bash
# 访问
curl http://localhost:3001

# 如果看到ENOENT错误，清理缓存
cd agentmem-website
rm -rf .next
npm run dev
```

### 3. 测试新的Memory API

```bash
# 获取agents列表
curl http://localhost:8080/api/v1/agents | python3 -m json.tool

# 使用真实的agent ID测试memories
curl "http://localhost:8080/api/v1/agents/YOUR_AGENT_ID/memories"

# 应该返回空数组[] 或 记忆列表
```

### 4. 添加测试数据

```bash
# 先添加一条记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "YOUR_AGENT_ID",
    "content": "Test memory",
    "importance": 0.8
  }'

# 然后再次获取
curl "http://localhost:8080/api/v1/agents/YOUR_AGENT_ID/memories"
```

## 常见问题

### Q1: "Internal Server Error" 在前端

**原因**: 通常是后端未运行或API端点问题

**解决**:
1. 确保后端运行: `cd agentmen && cargo run`
2. 检查后端日志中的错误信息
3. 测试API endpoint（见上方步骤3）

### Q2: "ENOENT: no such file or directory" 

**原因**: Next.js 缓存问题

**解决**:
```bash
cd agentmem-website
rm -rf .next
npm run dev
```

### Q3: API返回空数组

**原因**: 正常，该agent还没有记忆

**解决**: 添加测试数据（见步骤4）

## 极简测试流程（3分钟）

```bash
# 1. 确保服务运行
curl http://localhost:8080/health
curl http://localhost:3001

# 2. 获取一个agent ID
AGENT_ID=$(curl -s http://localhost:8080/api/v1/agents | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "Using Agent ID: $AGENT_ID"

# 3. 测试Memory API
curl "http://localhost:8080/api/v1/agents/$AGENT_ID/memories"

# 4. 打开浏览器测试
echo "Visit: http://localhost:3001/admin/memories"
```

## 成功标志

✅ 后端健康检查通过  
✅ 前端可以访问（无ENOENT错误）  
✅ Memory API返回200（即使是空数组）  
✅ 浏览器控制台无404错误  

## 如果都不行

最后的办法：

```bash
# 1. 停止所有服务
# 2. 清理所有缓存
cd agentmem-website && rm -rf .next node_modules && npm install

# 3. 重新启动
# Terminal 1:
cd agentmen && cargo run

# Terminal 2:
cd agentmem-website && npm run dev

# 4. 访问
open http://localhost:3001/admin/memories
```

---

**记住**: 极简原则 - 最简单的方法通常最有效。不要过度调试！

