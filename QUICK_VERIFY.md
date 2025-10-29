# 🚀 快速验证指南

## 方式1: 使用快速启动脚本（推荐）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
./quick-start.sh
```

选择选项3，然后按照提示操作。

---

## 方式2: 手动启动 + Open命令

### 步骤1: 启动后端（终端1）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cargo run --bin agent-mem-server
```

等待看到：
```
Server listening on http://0.0.0.0:3001
```

### 步骤2: 启动前端（终端2）

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/agentmem-ui
npm run dev
```

等待看到：
```
- Local: http://localhost:3000
```

### 步骤3: 打开浏览器验证（终端3或当前终端）

```bash
# 方法A: 使用 open 命令打开 Dashboard
open http://localhost:3000/admin

# 方法B: 打开 Chat 页面
open http://localhost:3000/admin/chat

# 方法C: 打开 Agents 页面
open http://localhost:3000/admin/agents

# 方法D: 打开 Memories 页面
open http://localhost:3000/admin/memories

# 方法E: 打开 Demo 页面
open http://localhost:3000/demo

# 方法F: 同时打开所有主要页面
open http://localhost:3000/admin && \
open http://localhost:3000/admin/chat && \
open http://localhost:3000/admin/agents && \
open http://localhost:3000/admin/memories
```

---

## 📝 快速验证清单（5分钟版本）

### ✅ 1. Dashboard验证（1分钟）
- [ ] 打开 `http://localhost:3000/admin`
- [ ] 验证右上角显示绿色 Wifi 图标（WebSocket已连接）
- [ ] 验证显示统计数字（Total Agents, Total Memories等）
- [ ] 验证显示图表（Memory Growth, Agent Activity）

### ✅ 2. Agents验证（1分钟）
- [ ] 打开 `http://localhost:3000/admin/agents`
- [ ] 验证右上角显示绿色 "Live" Badge
- [ ] 点击 "Create Agent"
- [ ] 填写名称："Test Agent"，点击 Create
- [ ] 验证：显示Toast通知 + 新Agent出现在列表中

### ✅ 3. Chat SSE验证（2分钟）
- [ ] 打开 `http://localhost:3000/admin/chat`
- [ ] 选择一个Agent
- [ ] 验证："Stream responses" 已勾选
- [ ] 验证：SSE连接状态显示 "SSE Connected"（绿色）
- [ ] 输入消息："Hello"，点击发送
- [ ] 验证：消息逐字显示（打字效果） + "Live" 徽章

### ✅ 4. Memories验证（1分钟）
- [ ] 打开 `http://localhost:3000/admin/memories`
- [ ] 验证右上角显示绿色 "Live" Badge
- [ ] 选择一个Agent，查看Memory列表
- [ ] 验证：显示该Agent的Memories

### ✅ 5. 实时更新验证（额外2分钟）
- [ ] 保持Dashboard页面打开
- [ ] 在新标签页打开Agents页面，创建一个新Agent
- [ ] 切换回Dashboard标签页
- [ ] 验证：显示Toast通知 "Agent updated" + 统计数字自动刷新

---

## 🔍 详细测试

完整的15个测试场景请参考：
```bash
open /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/START_TESTING.md
```

---

## 🐛 常见问题

### Q1: 后端启动失败
**解决**：
```bash
# 检查端口占用
lsof -i :3001
# 如果有进程占用，结束它
kill -9 <PID>
```

### Q2: 前端启动失败
**解决**：
```bash
cd agentmem-ui
rm -rf .next node_modules
npm install
npm run dev
```

### Q3: WebSocket连接失败（显示红色WifiOff）
**解决**：
- 确认后端服务器正在运行
- 检查控制台错误信息（F12）
- 刷新页面（Cmd+R）

### Q4: SSE连接失败
**解决**：
- 确认后端服务器正在运行
- 检查 `http://localhost:3001/api/v1/sse` 是否可访问
- 查看后端日志

---

## 📊 验证成功标志

全部功能正常时，您应该看到：

**Dashboard页面**:
- ✅ 绿色Wifi图标（WebSocket连接）
- ✅ 实时统计数字
- ✅ 图表显示真实数据
- ✅ Toast通知自动弹出

**Chat页面**:
- ✅ "SSE Connected" 绿色Badge
- ✅ 消息逐字显示（流式模式）
- ✅ "Live" 徽章显示

**Agents/Memories页面**:
- ✅ "Live" 绿色Badge
- ✅ 创建/删除时立即刷新
- ✅ Toast通知

---

## 🎯 下一步

验证通过后：
1. ✅ 在 `START_TESTING.md` 中记录测试结果
2. ✅ 更新 `agentmem39.md` 标记测试完成
3. ✅ 继续 Phase 3 的其他任务

**恭贺完成验证！** 🎉

