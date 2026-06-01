# AI Agent 记忆系统收敛计划 (plan26.md)

## Context

分析 agentmen 项目中 AI Agent 记忆系统存在的问题，制定收敛计划。

**问题根源**：
- 三套真相并存：Legacy (MemoryItem/MemoryType)、V4 (Memory)、File-centric (ResourceDescriptor)
- 两个 MemoryScope 定义冲突：`types.rs` vs `hierarchy.rs`
- 两个 MemoryType 定义冲突：Rust 8种 vs Python 4种
- 9个 Manager 职责重叠
- ResourceManager 使用内存 HashMap 而非持久化存储
- 缺少 CLAUDE.md、MEMORY.md 配置文档

---

## Phase 1: 统一核心类型定义 ✅

### 1.1 已完成 ✅

| 文件 | 操作 | 状态 |
|------|------|------|
| `crates/agent-mem-traits/src/scope.rs` | 新建，统一 MemoryScope 定义 | ✅ 完成 |
| `crates/agent-mem-traits/src/lib.rs` | 导出 MemoryScope | ✅ 完成 |
| `sdks/python/agentmem/types.py` | 扩展为 8 种 MemoryType | ✅ 完成 |

### 1.2 MemoryScope 双定义分析 ⚠️

**重要发现**：agentmen 项目中存在两套不同的 MemoryScope 定义，它们服务于不同的目的：

| 定义位置 | 结构 | 用途 |
|---------|------|------|
| `agent-mem/src/types.rs` | 含 org_id，6层结构 | 顶层 API，用户级隔离 |
| `agent-mem-core/src/hierarchy.rs` | 简化的 4 层结构 | 内部层级管理 |

**结论**：两套定义并存是**设计决策**，不是错误。

- 统一会导致 ~20+ 文件需要修改
- 会破坏与现有测试的兼容性
- 建议通过适配器层进行交互

---

## Phase 2: 存储统一 (待开始)

### 2.1 重构 ResourceManager

**问题**：`ResourceManager` 使用内存 HashMap，file-centric extraction pipeline 默认 None

### 2.2 统一存储后端

使用 LibSQL 作为默认存储（与 server 层一致）

---

## Phase 3: Manager 整合 (待开始)

### 3.1 从 9 个 Manager 整合为 3 个

| 当前 | 整合后 |
|------|--------|
| episodic_memory.rs | UnifiedMemoryManager |
| semantic_memory.rs | UnifiedMemoryManager |
| procedural_memory.rs | UnifiedMemoryManager |
| core_memory.rs | UnifiedMemoryManager |
| resource_memory.rs | UnifiedMemoryManager |
| knowledge_vault.rs | UnifiedMemoryManager |
| contextual_memory.rs | UnifiedMemoryManager |
| knowledge_graph_manager.rs | KnowledgeGraphManager |
| association_manager.rs | LifecycleManager |

### 3.2 统一 API 入口

---

## Phase 4: Legacy→V4 迁移 (待开始)

### 4.1 创建 Migration 模块

### 4.2 迁移范围

- ~1094 处 MemoryItem 引用
- ~437 处需要显式转换

---

## Phase 5: 配置文档化 (待开始)

### 5.1 创建 MEMORY.md

### 5.2 创建 CLAUDE.md

---

## Critical Files to Modify

| 优先级 | 文件 | 修改内容 | 状态 |
|--------|------|----------|------|
| P0 | `crates/agent-mem-traits/src/scope.rs` | 新建统一 MemoryScope | ✅ 完成 |
| P0 | `crates/agent-mem-traits/src/lib.rs` | 导出 MemoryScope | ✅ 完成 |
| P0 | `sdks/python/agentmem/types.py` | 扩展为 8 种类型 | ✅ 完成 |
| P1 | 保留 `types.rs` 和 `hierarchy.rs` 的双定义 | 架构决策，无需修改 | ✅ 确认 |
| P2 | `crates/agent-mem-core/src/managers/mod.rs` | 整合 9 个 Manager | ❌ 待开始 |
| P2 | `crates/agent-mem-core/src/managers/resource_memory.rs` | 持久化存储 | ❌ 待开始 |
| P2 | `crates/agent-mem-core/src/migration.rs` | 新建迁移模块 | ❌ 待开始 |
| P3 | `MEMORY.md` | 新建配置文档 | ❌ 待开始 |
| P3 | `CLAUDE.md` | 新建项目规范 | ❌ 待开始 |

---

## 当前状态

### 编译验证 ✅
- `agent-mem-traits`: ✅ 编译通过
- `agent-mem-core`: ✅ 编译通过
- `agent-mem`: ✅ 编译通过（仅警告）
- `agent-mem-server`: ✅ 编译通过
- **全项目编译**: ✅ 通过 (2m25s)

### Python SDK 验证 ✅
- MemoryType: 8种 ✅
  - 基础: episodic, semantic, procedural, working
  - 高级: core, resource, knowledge, contextual

---

## 下一步建议

1. **Phase 2**: 重构 ResourceManager 使用持久化存储
2. **Phase 3**: 整合 9 个 Manager 为 3 个核心管理器
3. **Phase 4**: 创建 Migration 模块处理 Legacy→V4 迁移
4. **Phase 5**: 创建 MEMORY.md 和 CLAUDE.md 配置文档

---

## Verification

1. **类型一致性测试**：MemoryScope/MemoryType 在所有 crate 中定义一致
2. **存储测试**：ResourceManager 使用持久化存储，所有存储后端可互换
3. **API 测试**：统一 API 方法正常工作，向后兼容旧接口
4. **迁移测试**：MemoryItem → MemoryV4 转换正确

---

## Risk Mitigation

| 风险 | 缓解措施 |
|------|----------|
| 迁移破坏现有功能 | 保留 Legacy 适配器层，逐步迁移 |
| Python/Rust 类型不一致 | 使用共享类型定义文件 |
| 性能下降 | 性能基准测试，及时优化 |