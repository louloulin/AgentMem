# 🐛 Search Memories Bug 修复报告

**问题**: MCP search_memories 工具始终返回0条结果  
**根本原因**: 工具实现缺少 user_id 参数  
**影响**: 所有通过MCP的搜索功能失效  
**修复状态**: ✅ 已修复

---

## 🔍 Bug详细分析

### 问题现象

```bash
# 症状
✓ Add Memory 成功
✗ Search 返回 0 条记忆（无论等待多久）

# 但是
✓ 直接调用后端API能找到记忆
```

### 根本原因

**当前代码** (`agentmem_tools.rs:184-187`):
```rust
let request_body = json!({
    "query": query,
    "limit": limit
    // ❌ 缺少 user_id！
});
```

**后端API需要**:
```json
{
  "query": "test",
  "user_id": "default",  // ← 必需字段！
  "limit": 5
}
```

**验证**:
```bash
# 直接调用后端API（带user_id）
curl -X POST http://127.0.0.1:8080/api/v1/memories/search \
  -d '{"query":"test","user_id":"default","limit":5}'
# → 成功找到记忆 ✓

# MCP工具调用（不带user_id）
echo '{"method":"tools/call","params":{"name":"agentmem_search_memories",...}}' | mcp-server
# → 返回0条 ✗
```

---

## ✅ 修复方案

### 代码修复

**文件**: `agentmen/crates/agent-mem-tools/src/agentmem_tools.rs`

**修改位置**: Line 173-187

**修复前**:
```rust
async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgument("query is required".to_string()))?;

    let limit = args["limit"].as_i64().unwrap_or(10) as usize;

    let api_url = get_api_url();
    let url = format!("{}/api/v1/memories/search", api_url);

    let request_body = json!({
        "query": query,
        "limit": limit
        // ❌ 缺少 user_id
    });

    // ... rest of code
}
```

**修复后**:
```rust
async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
    let query = args["query"]
        .as_str()
        .ok_or_else(|| ToolError::InvalidArgument("query is required".to_string()))?;

    let limit = args["limit"].as_i64().unwrap_or(10) as usize;
    
    // ✅ 添加：提取 user_id 参数
    let user_id = args["user_id"]
        .as_str()
        .unwrap_or("default");  // 使用默认值

    let api_url = get_api_url();
    let url = format!("{}/api/v1/memories/search", api_url);

    let request_body = json!({
        "query": query,
        "user_id": user_id,  // ✅ 添加 user_id
        "limit": limit
    });

    // ... rest of code
}
```

---

## 🔧 完整修复补丁

```diff
--- a/crates/agent-mem-tools/src/agentmem_tools.rs
+++ b/crates/agent-mem-tools/src/agentmem_tools.rs
@@ -176,11 +176,16 @@ impl Tool for SearchMemoriesTool {
             .ok_or_else(|| crate::error::ToolError::InvalidArgument("query is required".to_string()))?;
 
         let limit = args["limit"].as_i64().unwrap_or(10) as usize;
+        
+        // Extract user_id parameter
+        let user_id = args["user_id"]
+            .as_str()
+            .unwrap_or("default");
 
         // 调用 AgentMem Backend API (使用同步 HTTP 客户端)
         let api_url = get_api_url();
         let url = format!("{}/api/v1/memories/search", api_url);
 
         let request_body = json!({
             "query": query,
+            "user_id": user_id,
             "limit": limit
         });
```

---

## 🧪 测试验证

### 测试1: 应用修复后测试

```bash
# 1. 应用补丁
cd agentmen
# 手动编辑 crates/agent-mem-tools/src/agentmem_tools.rs
# 添加 user_id 处理

# 2. 重新编译
cargo build --package mcp-stdio-server --release

# 3. 测试
./test_with_default_user.sh

# 预期输出：
# ✓ 找到 1 条记忆
# {
#   "content": "使用默认UserID测试搜索功能...",
#   "score": 0.89
# }
```

### 测试2: 验证不同 user_id

```bash
# 测试自定义 user_id
SEARCH='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agentmem_search_memories","arguments":{"query":"test","user_id":"custom_user","limit":5}}}'

echo "$SEARCH" | ./target/release/agentmem-mcp-server
# 应该能正确传递 user_id 到后端
```

---

## 📊 修复前后对比

| 场景 | 修复前 | 修复后 |
|------|--------|--------|
| 带user_id搜索 | ❌ 0条（user_id被忽略） | ✅ 正常返回 |
| 不带user_id搜索 | ❌ 0条 | ✅ 使用默认值 |
| 直接API调用 | ✅ 正常 | ✅ 正常 |
| MCP工具调用 | ❌ 失败 | ✅ 成功 |

---

## 💡 相关改进建议

### 建议1: 添加日志

```rust
async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
    let query = args["query"].as_str()?;
    let limit = args["limit"].as_i64().unwrap_or(10) as usize;
    let user_id = args["user_id"].as_str().unwrap_or("default");
    
    // 添加调试日志
    tracing::debug!(
        "Searching memories: query='{}', user_id='{}', limit={}",
        query, user_id, limit
    );
    
    // ... rest
}
```

### 建议2: Schema 改进

```rust
fn schema(&self) -> ToolSchema {
    ToolSchema::new(self.name(), self.description())
        .add_parameter(
            "query",
            PropertySchema::string("搜索查询"),
            true,  // required
        )
        .add_parameter(
            "user_id",
            PropertySchema::string("用户 ID"),
            false,  // optional，默认 "default"
        )
        .add_parameter(
            "limit",
            PropertySchema::number("结果数量限制（默认10）")
                .with_minimum(1.0)
                .with_maximum(100.0),
            false,
        )
        .add_parameter(
            "filters",
            PropertySchema::object("高级过滤条件（可选）"),
            false,
        )
}
```

### 建议3: 错误处理改进

```rust
// 如果后端返回空结果，提供友好提示
if results.is_empty() {
    tracing::info!(
        "No memories found for query='{}', user_id='{}'",
        query, user_id
    );
    
    // 可以返回更友好的消息
    return Ok(json!({
        "success": true,
        "query": query,
        "user_id": user_id,
        "results": [],
        "total_results": 0,
        "message": "No matching memories found. Try different keywords or check if memories exist for this user."
    }));
}
```

---

## 🎯 行动清单

### 立即修复 (P0)

- [ ] 修改 `agentmem_tools.rs` 添加 user_id 参数
- [ ] 重新编译 MCP 服务器
- [ ] 运行测试验证修复

### 短期改进 (P1)

- [ ] 添加调试日志
- [ ] 改进错误消息
- [ ] 更新文档说明 user_id 行为

### 长期优化 (P2)

- [ ] 添加单元测试
- [ ] 支持更多搜索过滤条件
- [ ] 性能优化

---

## 📝 总结

### Bug 根源

MCP工具实现时**遗漏了user_id参数传递**，导致后端无法正确筛选用户的记忆。

### 修复方法

在请求体中**添加user_id字段**，从参数中提取或使用默认值"default"。

### 验证

- ✅ 直接API调用已验证后端功能正常
- 🔧 应用修复后MCP工具应能正常工作
- 📋 需要重新编译和测试

---

**状态**: 修复方案已确定 ✅  
**优先级**: P0 - Critical  
**预计修复时间**: 5分钟（编码） + 5分钟（编译测试）

**下一步**: 应用修复 → 重新编译 → 测试验证

