# AgentMem 验证快速开始指南

**版本**: v1.0  
**日期**: 2025-10-30  
**预计时间**: 30分钟 (自动化) + 30分钟 (手动验证)

---

## 🚀 一键自动化验证

### 方法1: 使用自动化脚本 (推荐)

```bash
cd agentmen
./verify_agentmem.sh
```

这个脚本会自动完成：
- ✅ 环境检查 (Rust, protoc, Node.js, npm)
- ✅ 编译Backend、Frontend、MCP Server
- ✅ 启动Backend和Frontend服务
- ✅ 运行自动化API测试
- ✅ 验证MCP功能

**预期输出**:
```
==========================================
AgentMem 全面功能验证
版本: v1.0
日期: 2025-10-30
==========================================

==========================================
Phase 1: 环境检查
==========================================
[INFO] 检查Rust环境...
[SUCCESS] rustc 已安装: /usr/local/bin/rustc
rustc 1.70.0 (90c541806 2023-05-31)
...

==========================================
验证完成！
==========================================
[SUCCESS] 所有自动化测试通过
[INFO] 服务正在运行:
  - Backend: http://localhost:8080 (PID: 12345)
  - Frontend: http://localhost:3001 (PID: 12346)
```

### 方法2: 分步手动验证

如果自动化脚本失败，可以按以下步骤手动验证：

#### Step 1: 环境检查 (5分钟)

```bash
# 检查Rust
rustc --version  # 需要 >= 1.70

# 检查protoc
protoc --version  # 需要 >= 3.15

# 检查Node.js
node --version  # 需要 >= 20.0
npm --version   # 需要 >= 10.0

# 检查jq (可选)
jq --version
```

#### Step 2: 编译项目 (10分钟)

```bash
cd agentmen

# 设置protoc路径
export PROTOC=/opt/homebrew/bin/protoc

# 编译Backend
cargo build --release -p agent-mem-server

# 编译MCP Server
cargo build --release -p mcp-stdio-server

# 编译Frontend
cd agentmem-ui
npm install
npm run build
cd ..
```

#### Step 3: 启动服务 (5分钟)

**终端1 - Backend**:
```bash
cd agentmen
./target/release/agent-mem-server \
    --host 0.0.0.0 \
    --port 8080 \
    --log-level info
```

**终端2 - Frontend**:
```bash
cd agentmen/agentmem-ui
npm run dev
```

#### Step 4: 验证Backend (5分钟)

```bash
# Health Check
curl http://localhost:8080/health | jq '.'

# 创建Agent
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "测试Agent",
    "description": "用于测试"
  }' | jq '.'

# 保存返回的agent_id
AGENT_ID="<从上面的响应中复制>"

# 创建Memory
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"content\": \"测试记忆内容\",
    \"memory_type\": \"episodic\",
    \"importance\": 0.8
  }" | jq '.'

# 搜索Memory
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"query\": \"测试\",
    \"limit\": 10
  }" | jq '.'
```

#### Step 5: 验证Frontend (5分钟)

在浏览器中访问以下页面：

1. **主页**: http://localhost:3001
   - [ ] 导航栏显示正确
   - [ ] Hero区域显示
   - [ ] 功能卡片显示

2. **Admin Dashboard**: http://localhost:3001/admin
   - [ ] 统计卡片显示 (Agents, Memories, Users, Messages)
   - [ ] 图表显示 (Memory Growth, Agent Activity)

3. **Agents管理**: http://localhost:3001/admin/agents
   - [ ] 点击"Create Agent"创建新Agent
   - [ ] Agent卡片显示

4. **Memories管理**: http://localhost:3001/admin/memories
   - [ ] 选择Agent
   - [ ] Memories列表显示
   - [ ] 测试搜索和过滤

5. **Chat界面**: http://localhost:3001/admin/chat
   - [ ] 选择Agent
   - [ ] 发送消息
   - [ ] 验证回复

#### Step 6: 验证MCP (5分钟)

```bash
cd agentmen

# 测试Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"clientInfo":{"name":"test","version":"1.0"}}}' | \
  ./target/release/agentmem-mcp-server

# 测试工具列表
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | \
  ./target/release/agentmem-mcp-server

# 测试添加记忆
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"agentmem_add_memory","arguments":{"content":"MCP测试记忆","user_id":"test-user"}}}' | \
  ./target/release/agentmem-mcp-server
```

---

## 📋 验证检查清单

### Backend验证

- [ ] Health Check返回 `{"status": "healthy"}`
- [ ] Swagger UI可访问: http://localhost:8080/swagger-ui
- [ ] 可以创建Agent
- [ ] 可以创建Memory
- [ ] 可以搜索Memory
- [ ] 数据库文件存在: `ls -lh data/agentmem.db`

### Frontend验证

- [ ] 主页可访问
- [ ] Admin Dashboard显示统计数据
- [ ] Agents管理页面可以创建/查看Agent
- [ ] Memories管理页面可以查看/搜索Memory
- [ ] Chat界面可以发送消息
- [ ] 深色模式切换正常

### MCP验证

- [ ] Initialize握手成功
- [ ] 工具列表包含4个工具
- [ ] agentmem_add_memory工具可用
- [ ] agentmem_search_memories工具可用
- [ ] agentmem_chat工具可用
- [ ] agentmem_get_system_prompt工具可用

---

## 🐛 常见问题

### Q1: 编译失败 - protoc not found

**解决**:
```bash
# macOS
brew install protobuf
export PROTOC=/opt/homebrew/bin/protoc

# Linux
sudo apt-get install protobuf-compiler
export PROTOC=/usr/bin/protoc
```

### Q2: 端口被占用

**解决**:
```bash
# 查找占用进程
lsof -i :8080
lsof -i :3001

# 杀死进程
kill -9 <PID>

# 或使用其他端口
./target/release/agent-mem-server --port 8081
```

### Q3: Frontend编译失败

**解决**:
```bash
cd agentmem-ui

# 清理缓存
rm -rf node_modules .next

# 重新安装
npm install

# 重新构建
npm run build
```

### Q4: Backend启动失败 - 数据库错误

**解决**:
```bash
# 检查数据库文件
ls -lh data/agentmem.db

# 如果损坏，删除并重新创建
rm data/agentmem.db
./target/release/agent-mem-server
```

### Q5: UI无法连接Backend

**解决**:
```bash
# 1. 检查Backend是否运行
curl http://localhost:8080/health

# 2. 检查CORS配置
cat config.toml | grep -A 5 "cors"

# 3. 检查Frontend环境变量
cat agentmem-ui/.env.local
# 应该包含:
# NEXT_PUBLIC_API_URL=http://localhost:8080
```

---

## 📊 验证结果记录

### 自动化测试结果

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 环境检查 | ⬜ 通过 / ⬜ 失败 | |
| Backend编译 | ⬜ 通过 / ⬜ 失败 | |
| Frontend编译 | ⬜ 通过 / ⬜ 失败 | |
| MCP编译 | ⬜ 通过 / ⬜ 失败 | |
| Health Check | ⬜ 通过 / ⬜ 失败 | |
| Agent CRUD | ⬜ 通过 / ⬜ 失败 | |
| Memory CRUD | ⬜ 通过 / ⬜ 失败 | |
| Memory Search | ⬜ 通过 / ⬜ 失败 | |
| MCP Initialize | ⬜ 通过 / ⬜ 失败 | |
| MCP Tools | ⬜ 通过 / ⬜ 失败 | |

### 手动测试结果

| 测试项 | 状态 | 备注 |
|--------|------|------|
| 主页访问 | ⬜ 通过 / ⬜ 失败 | |
| Admin Dashboard | ⬜ 通过 / ⬜ 失败 | |
| Agents管理 | ⬜ 通过 / ⬜ 失败 | |
| Memories管理 | ⬜ 通过 / ⬜ 失败 | |
| Chat界面 | ⬜ 通过 / ⬜ 失败 | |
| Users管理 | ⬜ 通过 / ⬜ 失败 | |
| Settings | ⬜ 通过 / ⬜ 失败 | |

---

## 📝 下一步

验证完成后：

1. **记录问题**: 在 `ISSUE_TRACKER.md` 中记录所有发现的问题
2. **更新文档**: 在 `agentmem33.md` 中更新执行记录
3. **修复问题**: 按优先级修复问题
4. **回归测试**: 修复后重新运行验证
5. **发布**: 准备Beta版本发布

---

## 🔗 相关文档

- [完整验证计划](agentmem33.md) - 详细的验证计划和技术分析
- [问题追踪表](ISSUE_TRACKER.md) - 问题记录和修复进度
- [快速启动指南](QUICK_START.md) - 生产环境快速启动
- [测试指南](crates/agent-mem-server/tests/README.md) - 单元测试和集成测试

---

**文档维护**: 请在验证过程中更新测试结果  
**最后更新**: 2025-10-30

