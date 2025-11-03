# 运行时验证测试指南

**测试日期**: 2025-10-29  
**测试目的**: 验证 Phase 1 + Phase 2 所有实现的功能  
**测试场景**: 15个  

---

## 🚀 步骤1: 启动后端服务器

打开终端1，执行：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 启动后端服务器
cargo run --bin agent-mem-server
```

**预期输出**:
```
Starting AgentMem Server...
Server listening on http://0.0.0.0:3001
WebSocket endpoint: ws://0.0.0.0:3001/api/v1/ws
SSE endpoint: http://0.0.0.0:3001/api/v1/sse
```

**注意**: 等待后端服务器完全启动（约10-30秒）

---

## 🚀 步骤2: 启动前端服务器

打开终端2，执行：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/agentmem-ui

# 安装依赖（如果尚未安装）
npm install

# 启动开发服务器
npm run dev
```

**预期输出**:
```
▲ Next.js 14.x.x
- Local:        http://localhost:3000
- Ready in Xs
```

**注意**: 等待前端编译完成

---

## 🧪 步骤3: 访问应用

在浏览器中打开：

```
http://localhost:3000
```

**导航到**: `/admin` 页面

---

## ✅ 测试场景执行

### 场景组1: Chat SSE流式响应（3个场景）

#### 场景1.1: 流式模式测试
1. 导航到 `/admin/chat`
2. 选择一个Agent（如果没有，先创建一个）
3. **验证**: "Stream responses" 复选框已勾选
4. **验证**: SSE连接状态显示 "SSE Connected"（绿色Badge）
5. 输入消息："Hello, tell me about yourself"
6. 点击发送
7. **预期结果**:
   - ✅ 消息逐字显示（打字效果）
   - ✅ 显示旋转加载器
   - ✅ 显示 "Live" 徽章（绿色，带闪电图标）
   - ✅ 完成后 "Live" 徽章消失
   - ✅ 自动滚动到最新消息

#### 场景1.2: 标准模式测试
1. 取消勾选 "Stream responses"
2. 输入消息："What can you do?"
3. 点击发送
4. **预期结果**:
   - ✅ 消息一次性完整显示（无打字效果）
   - ✅ 无 "Live" 徽章
   - ✅ 显示"Agent is thinking..."加载状态

#### 场景1.3: 模式切换测试
1. 在流式和标准模式之间切换3次
2. 每次发送一条消息
3. **预期结果**:
   - ✅ 每种模式都正常工作
   - ✅ 无错误或崩溃

**✅ 场景组1 完成**: ⬜ 是 / ⬜ 否

---

### 场景组2: Agents WebSocket更新（4个场景）

#### 场景2.1: 创建Agent实时通知
1. 导航到 `/admin/agents`
2. **验证**: 连接状态显示 "Live"（绿色Badge，带Wifi图标）
3. 点击 "Create Agent"
4. 填写：
   - Name: "Test Agent 1"
   - Description: "Test description"
5. 点击 "Create"
6. **预期结果**:
   - ✅ 显示Toast通知："Agent created"
   - ✅ Agent列表自动刷新，显示新Agent
   - ✅ 无需手动刷新页面

#### 场景2.2: 删除Agent实时通知
1. 在Agents页面
2. 点击某个Agent的删除按钮（垃圾桶图标）
3. 确认删除
4. **预期结果**:
   - ✅ 显示Toast通知："Agent deleted"
   - ✅ Agent从列表中立即消失
   - ✅ 无需手动刷新页面

#### 场景2.3: 连接状态显示
1. 在Agents页面
2. **验证**: 右上角显示 "Live" Badge（绿色，带Wifi图标）
3. **预期结果**:
   - ✅ 连接状态清晰可见
   - ✅ 绿色表示已连接

#### 场景2.4: WebSocket自动重连（稍后测试）

**✅ 场景组2 完成**: ⬜ 是 / ⬜ 否

---

### 场景组3: Memories WebSocket更新（4个场景）

#### 场景3.1: 创建Memory实时通知
1. 导航到 `/admin/memories`
2. **验证**: 连接状态显示 "Live"（绿色Badge）
3. 选择一个Agent
4. 点击 "Add Memory"（如果功能可用）
   - 或通过Chat页面发送消息，触发Memory创建
5. **预期结果**:
   - ✅ 显示Toast通知："Memory created" 或 "Memory updated"
   - ✅ Memory列表自动刷新
   - ✅ 无需手动刷新页面

#### 场景3.2: 删除Memory实时通知
1. 在Memories页面
2. 点击某个Memory的删除按钮
3. 确认删除
4. **预期结果**:
   - ✅ 显示Toast通知："Memory deleted"
   - ✅ Memory从列表中立即消失
   - ✅ 无需手动刷新页面

#### 场景3.3: 智能刷新验证
1. 在Memories页面
2. 选择 "Agent A"
3. 通过另一个浏览器窗口或Chat页面，为 "Agent B" 创建Memory
4. **预期结果**:
   - ✅ "Agent A" 的Memory列表不刷新（智能过滤）
   - ✅ 切换到 "Agent B"，列表显示新Memory

#### 场景3.4: 连接状态显示
1. 在Memories页面
2. **验证**: 右上角显示 "Live" Badge
3. **预期结果**:
   - ✅ 连接状态清晰可见

**✅ 场景组3 完成**: ⬜ 是 / ⬜ 否

---

### 场景组4: Dashboard实时更新（2个场景）

#### 场景4.1: Agent更新触发Dashboard刷新
1. 导航到 `/admin`（Dashboard）
2. **验证**: 连接状态显示绿色Wifi图标
3. 打开新浏览器窗口，导航到 `/admin/agents`
4. 创建一个新Agent
5. 切换回Dashboard窗口
6. **预期结果**:
   - ✅ 显示Toast通知："Agent updated"
   - ✅ Dashboard统计自动刷新
   - ✅ "Total Agents" 数字增加

#### 场景4.2: Memory更新触发Dashboard刷新
1. 在Dashboard页面
2. 通过Chat页面或另一个窗口创建Memory
3. 切换回Dashboard窗口
4. **预期结果**:
   - ✅ 显示Toast通知："Memory updated"
   - ✅ Dashboard统计自动刷新
   - ✅ "Total Memories" 数字增加

**✅ 场景组4 完成**: ⬜ 是 / ⬜ 否

---

### 场景组5: WebSocket自动重连（2个场景）

#### 场景5.1: 断开网络测试
1. 在任意页面（Agents、Memories、Dashboard）
2. 打开浏览器开发者工具（F12）
3. 切换到 Network 标签
4. 选择 "Offline" 模式（模拟断网）
5. **预期结果**:
   - ✅ 连接状态Badge变为 "Disconnected"（灰色，带WifiOff图标）
   - ✅ 控制台显示重连日志

#### 场景5.2: 恢复网络测试
1. 在开发者工具中，关闭 "Offline" 模式
2. 等待5-10秒
3. **预期结果**:
   - ✅ 连接状态Badge自动变回 "Live"（绿色，带Wifi图标）
   - ✅ 控制台显示 "WebSocket connected"
   - ✅ 无需手动刷新页面

**✅ 场景组5 完成**: ⬜ 是 / ⬜ 否

---

## 📊 额外验证项目

### API缓存验证
1. 打开浏览器开发者工具，Network标签
2. 导航到 `/admin`
3. 观察 API 请求
4. 刷新页面（F5）
5. **预期结果**:
   - ✅ 第二次加载时，某些API请求被缓存（减少请求数量）
   - ✅ 在控制台看到 "Cache hit" 日志

### 性能监控
1. 在Dashboard页面，打开 Performance Monitor 组件（如果已实现）
2. **预期结果**:
   - ✅ 显示实时性能指标
   - ✅ FPS、内存、网络等

### Charts实时数据
1. 在Dashboard页面
2. 观察 "Memory Growth" 和 "Agent Activity" 图表
3. **预期结果**:
   - ✅ 图表显示真实数据（非mock）
   - ✅ 点击刷新按钮，数据更新

---

## 🐛 问题记录

如果遇到任何问题，请记录：

| 场景编号 | 问题描述 | 错误信息 | 严重程度 |
|---------|---------|---------|---------|
| 例：1.1 | 流式响应无法显示 | TypeError: ... | 高 |
|  |  |  |  |
|  |  |  |  |

---

## ✅ 测试总结

**完成日期**: _____________  
**测试人员**: _____________  

**总场景数**: 15  
**通过数**: ___  
**失败数**: ___  
**通过率**: ____%  

**总体评价**:
- ⬜ 优秀 (100%)
- ⬜ 良好 (90-99%)
- ⬜ 及格 (80-89%)
- ⬜ 需改进 (<80%)

**主要发现**:
1. _______________________________________________
2. _______________________________________________
3. _______________________________________________

**下一步行动**:
1. _______________________________________________
2. _______________________________________________
3. _______________________________________________

---

**测试完成后，请更新 `agentmem39.md` 标记测试结果！**

