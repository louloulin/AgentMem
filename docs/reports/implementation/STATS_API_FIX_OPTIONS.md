# Stats API 修复方案

## 问题根源

FastEmbed 需要 `libonnxruntime.dylib` 系统库，但该库缺失导致 Embedder 初始化失败。Stats API 依赖 MemoryManager 的方法，而这些方法内部使用向量搜索，需要 Embedder。

## 解决方案对比

### ✅ 方案A：简化Stats API实现（快速，推荐）

**优点**：
- 立即可用，无需安装系统依赖
- 代码改动小，风险低
- 适合当前演示和验证

**缺点**：
- Memory统计功能暂时返回简化数据（0或模拟数据）
- 无法显示真实的memory数量和类型分布

**实施步骤**：
1. 修改 Stats API，不调用 memory_manager 方法
2. 返回 agents/users/messages 的真实统计
3. Memory相关字段返回简化数据

**预计时间**: 5分钟

---

### 🔧 方案B：安装ONNX Runtime（完整，需要系统配置）

**优点**：
- 完整功能，Memory统计准确
- 解决根本问题

**缺点**：
- 需要下载并安装系统库（~100MB）
- 需要配置环境变量或库路径
- 可能需要解决网络/权限问题

**实施步骤**：
1. 下载 ONNX Runtime dylib (for macOS ARM64)
2. 安装到系统库路径
3. 重启后端服务器

**预计时间**: 15-30分钟（取决于网络）

---

## 📋 当前状态

### 已完成的修改
✅ Stats API 代码已优化，避免直接调用 `get_stats()`
✅ 使用 `get_all_memories()` 方法统计 memories
✅ 编译成功，服务器已启动

### 仍存在的问题
❌ `get_all_memories()` 内部仍触发向量搜索
❌ 向量搜索需要 Embedder
❌ Embedder 初始化失败（缺少 libonnxruntime.dylib）

---

## 🎯 推荐方案

**建议采用方案A（简化实现）+ 未来升级方案B**

理由：
1. 当前任务重点是验证前后端集成和WebSocket/SSE实时通信
2. Dashboard的其他功能（agents/users/messages/WebSocket）都已正常工作
3. Memory统计可以使用简化数据进行演示
4. 后续有时间再安装完整的ONNX Runtime

---

## 🚀 方案A实施代码

修改 `stats.rs`，移除所有 memory_manager 调用：

```rust
// In get_dashboard_stats():
let total_memories = 0i64;  // 临时简化
let memories_by_type: HashMap<String, i64> = HashMap::new();  // 临时简化

// In get_memory_growth():
let total_memories = 0i64;  // 临时简化
let memories_by_type_map: HashMap<String, usize> = HashMap::new();  // 临时简化

// In get_agent_activity_stats():
let total_memories = 0i64;  // 每个agent的memory设为0
```

---

## 📝 下一步行动

**选择 A**：
```bash
# 我可以立即实施方案A，修改代码并重启服务器
# 5分钟内完成
```

**选择 B**：
```bash
# 需要先下载 ONNX Runtime
# 网址：https://github.com/microsoft/onnxruntime/releases
# 下载：onnxruntime-osx-arm64-*.tgz
# 解压并安装 libonnxruntime.dylib
```

**您希望我执行哪个方案？**

