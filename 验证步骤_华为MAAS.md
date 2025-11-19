# 华为 MaaS 功能验证步骤

## ✅ 服务状态

### 后端服务
- **状态**: ✅ 运行中 (PID: 3562)
- **地址**: http://localhost:8080
- **健康检查**: http://localhost:8080/health
- **API 文档**: http://localhost:8080/swagger-ui/
- **日志文件**: `backend-no-auth.log`

### 前端 UI
- **状态**: ✅ 运行中 (PID: 99409)
- **地址**: http://localhost:3001
- **认证**: 已禁用（测试模式）

---

## 🧪 验证步骤

### 步骤 1: 访问 UI
1. 打开浏览器访问: http://localhost:3001
2. 确认页面正常加载

### 步骤 2: 创建华为 MaaS Agent
1. 点击 "Create Agent" 或 "新建 Agent"
2. 填写 Agent 信息：
   - **Name**: `MaaS-Test-Agent`
   - **Description**: `华为 MaaS 测试 Agent`
   - **LLM Provider**: 选择 `maas`
   - **Model**: `deepseek-v3.2-exp`（或其他支持的模型）
   - **API Key**: 输入您的华为 MaaS API Key

3. 点击 "Create" 创建 Agent

### 步骤 3: 测试 Chat 功能
1. 在 Agent 列表中找到刚创建的 Agent
2. 点击进入 Chat 界面
3. 发送测试消息：
   ```
   你好，请介绍一下你自己
   ```
4. 验证回复是否正常

### 步骤 4: 测试多轮对话
1. 继续发送消息：
   ```
   我刚才问了你什么问题？
   ```
2. 验证 Agent 是否记住之前的对话（Memory 功能）

### 步骤 5: 测试流式响应
1. 发送较长的问题：
   ```
   请详细解释一下深度学习的基本原理
   ```
2. 观察回复是否以流式方式逐字显示

---

## 🔍 使用测试脚本验证

如果您想通过 API 直接测试，可以使用自动化测试脚本：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 设置环境变量
export HUAWEI_MAAS_API_KEY="your-api-key-here"
export HUAWEI_MAAS_MODEL="deepseek-v3.2-exp"

# 运行测试脚本
./test_maas_chat.sh
```

---

## 📊 验证检查清单

- [ ] 后端服务正常启动
- [ ] 前端 UI 正常访问
- [ ] 成功创建 MaaS Agent
- [ ] 单轮对话正常工作
- [ ] 多轮对话记忆功能正常
- [ ] 流式响应正常显示
- [ ] API 测试脚本通过

---

## 🔧 故障排查

### 如果 Agent 创建失败
1. 检查 API Key 是否正确
2. 检查模型名称是否支持
3. 查看后端日志：
   ```bash
   tail -f backend-no-auth.log
   ```

### 如果 Chat 无响应
1. 检查网络连接
2. 检查华为 MaaS API 配额
3. 查看浏览器控制台错误
4. 查看后端日志

### 查看服务日志
```bash
# 后端日志
tail -f backend-no-auth.log

# 前端日志
# 在 agentmem-ui 目录下查看 npm 输出
```

---

## 🎉 验证完成

如果所有步骤都通过，说明华为 MaaS 集成成功！

### 支持的功能
✅ 同步对话  
✅ 流式响应  
✅ 多轮对话记忆  
✅ 用户隔离  
✅ Agent 管理  

### 相关文档
- 📖 **使用说明**: `华为MAAS_CHAT功能使用说明.md`
- 🚀 **快速开始**: `华为MAAS_快速开始.md`
- 📊 **集成报告**: `华为MAAS集成验证报告.md`
- 🔧 **编译总结**: `编译成功总结.md`

---

## 🛑 停止服务

### 停止后端
```bash
pkill -f agent-mem-server
```

### 停止前端
```bash
lsof -ti :3001 | xargs kill
```
