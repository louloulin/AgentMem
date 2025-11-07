# 正确的Claude Code MCP命令

**日期**: 2025-11-07  
**状态**: ✅ 已验证

---

## ✅ 确认的正确命令

### Step 1: 切换到项目目录

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
```

### Step 2: 添加MCP服务器

```bash
claude mcp add agentmem /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server -s project -e AGENTMEM_API_URL=http://127.0.0.1:8080 -e RUST_LOG=info
```

### Step 3: 验证

```bash
claude mcp list
```

**期望输出**：应该在列表中看到 `agentmem`

---

## 📋 完整的操作流程

```bash
# 1. 进入项目目录
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 2. 确认二进制文件存在
ls -lh target/release/agentmem-mcp-server

# 3. 删除旧配置（如果存在）
claude mcp remove agentmem

# 4. 添加新配置（完整绝对路径）
claude mcp add agentmem \
  /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server \
  -s project \
  -e AGENTMEM_API_URL=http://127.0.0.1:8080 \
  -e RUST_LOG=info

# 5. 验证添加成功
cat .mcp.json | jq .

# 6. 检查健康状态
claude mcp list

# 7. 测试ping（手动）
echo '{"jsonrpc":"2.0","id":1,"method":"ping"}' | \
  ./target/release/agentmem-mcp-server 2>/dev/null | jq .
```

---

## 🎯 现在执行

**复制粘贴以下命令块**（一次性执行）：

```bash
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
claude mcp remove agentmem 2>/dev/null || true
claude mcp add agentmem /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/target/release/agentmem-mcp-server -s project -e AGENTMEM_API_URL=http://127.0.0.1:8080 -e RUST_LOG=info
claude mcp list
```

---

## ✅ 成功标志

如果配置成功，`claude mcp list` 应该显示：

```
Checking MCP server health...

context7: npx -y @upstash/context7-mcp@latest - ✓ Connected
sequential-thinking: npx -y @modelcontextprotocol/server-sequential-thinking - ✓ Connected
playwright: npx @playwright/mcp@latest - ✓ Connected
serena: uvx --from git+https://github.com/oraios/serena serena start-mcp-server - ✓ Connected
agentmem: /Users/.../agentmem-mcp-server - ✓ Connected  ← 应该看到这一行
```

---

*Last Updated: 2025-11-07*

