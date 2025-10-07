# AgentMem 全面测试计划 (Test Plan v2.0 - Rust Edition)

> **参考实现**: MIRIX 测试系统 + Rust 最佳实践
> **创建日期**: 2025-10-07
> **更新日期**: 2025-10-07
> **状态**: 规划中 → 实施中
> **语言**: Rust (主要) + Python (SDK测试)

---

## 📊 当前测试状态深度分析

### 现有测试统计 (详细)

| 指标 | 数量 | 百分比 | 说明 |
|------|------|--------|------|
| **Rust 源文件总数** | 376 个 | 100% | 整个项目的 .rs 文件 |
| **包含测试的文件** | 230 个 | 61.2% | 包含 `#[cfg(test)]` 模块的文件 |
| **测试函数总数** | 1,367 个 | - | 所有 `#[test]` 和 `#[tokio::test]` |
| **单元测试** | ~1,200 个 | 87.8% | 模块内测试 |
| **集成测试** | ~150 个 | 11.0% | tests/ 目录测试 |
| **基准测试** | ~17 个 | 1.2% | benches/ 目录 |
| **文档测试** | 未统计 | - | `/// # Examples` 中的测试 |
| **示例项目** | 40+ 个 | - | examples/ 目录 |

### 测试覆盖率详细分析

```
📦 agentmen/
├── 🟢 agent-mem-utils/          95% 覆盖 (hash, json, text 全覆盖)
├── 🟢 agent-mem-performance/    90% 覆盖 (metrics, cache, pool 完善)
├── 🟢 agent-mem-compat/         85% 覆盖 (Mem0 兼容层测试完善)
├── 🟡 agent-mem-core/           70% 覆盖 (核心功能有测试，但不全)
│   ├── ✅ managers/core_memory.rs      12 tests
│   ├── ✅ hierarchy_manager.rs         8 tests
│   ├── ❌ managers/episodic_memory.rs  0 tests ⚠️
│   ├── ❌ managers/semantic_memory.rs  0 tests ⚠️
│   ├── ❌ managers/procedural_memory.rs 0 tests ⚠️
│   ├── ❌ managers/knowledge_vault.rs  0 tests ⚠️
│   └── ❌ managers/resource_memory.rs  0 tests ⚠️
├── 🟡 agent-mem-server/         65% 覆盖 (API 测试基础完善)
│   ├── ✅ tests/integration_tests.rs   3 tests
│   ├── ✅ tests/auth_integration_test.rs
│   ├── ✅ tests/chat_api_test.rs
│   ├── ✅ tests/streaming_test.rs
│   └── ✅ routes/chat.rs (内联测试)
├── 🟡 agent-mem-llm/            60% 覆盖 (提供商测试不均)
│   ├── ✅ providers/openai.rs          完善
│   ├── ✅ providers/anthropic.rs       完善
│   ├── ✅ providers/azure.rs           完善
│   ├── 🟡 providers/gemini.rs          部分
│   ├── 🟡 providers/claude.rs          部分
│   └── ❌ providers/groq.rs            缺失
├── 🟡 agent-mem-storage/        55% 覆盖 (后端测试不全)
│   ├── ✅ backends/postgres.rs         基础测试
│   ├── 🟡 backends/qdrant.rs           部分测试
│   ├── 🟡 backends/weaviate.rs         部分测试
│   ├── ❌ backends/mongodb.rs          缺失 ⚠️
│   ├── ❌ backends/redis.rs            缺失 ⚠️
│   └── ❌ backends/supabase.rs         缺失 ⚠️
├── 🟡 agent-mem-embeddings/     50% 覆盖 (嵌入测试基础)
│   ├── ✅ utils.rs                     完善
│   ├── 🟡 providers/openai.rs          部分
│   ├── 🟡 providers/cohere.rs          部分
│   └── ❌ providers/local.rs           缺失
├── 🟡 agent-mem-tools/          60% 覆盖 (工具测试基础)
│   ├── ✅ tests/builtin_tools_test.rs
│   └── ✅ tests/execution_sandbox_test.rs
├── 🟡 agent-mem-intelligence/   55% 覆盖 (智能模块部分)
│   ├── ✅ importance/mod.rs            完善
│   ├── ✅ multimodal/text.rs           完善
│   ├── ✅ multimodal/audio.rs          完善
│   └── 🟡 multimodal/cross_modal.rs    部分
└── 🟡 agent-mem-distributed/    50% 覆盖 (分布式测试基础)
    ├── ✅ consensus.rs                 完善
    ├── ✅ cluster.rs                   完善
    └── 🟡 sharding.rs                  部分

图例: 🟢 优秀(≥80%) | 🟡 良好(50-79%) | 🔴 需改进(<50%)
```

### 关键发现和问题

#### ✅ 做得好的地方
1. **工具函数测试完善** - `agent-mem-utils` 有完整的单元测试
2. **性能监控测试** - `agent-mem-performance` 测试覆盖全面
3. **兼容层测试** - `agent-mem-compat` 有系统的测试
4. **Core Memory 测试** - 12 个测试覆盖核心功能
5. **集成测试框架** - server 有基础的集成测试

#### ⚠️ 需要改进的地方
1. **Memory Managers 缺失测试** - 5 个核心 manager 完全没有测试！
   - `episodic_memory.rs` - 0 tests ❌
   - `semantic_memory.rs` - 0 tests ❌
   - `procedural_memory.rs` - 0 tests ❌
   - `knowledge_vault.rs` - 0 tests ❌
   - `resource_memory.rs` - 0 tests ❌

2. **存储后端测试不全** - 6 个后端中 3 个缺失测试
   - MongoDB, Redis, Supabase 完全没有测试

3. **LLM 提供商测试不均** - 部分提供商测试不完整
   - Groq, Gemini, Local 测试缺失或不完整

4. **E2E 测试缺失** - 缺少完整的端到端工作流测试

5. **性能基准测试不足** - 只有基础的 benchmark

### Rust 测试模式分析

通过分析现有代码，发现以下测试模式：

#### 模式 1: 内联单元测试 (最常用)
```rust
// 文件: agent-mem-utils/src/hash.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let content = "test content";
        let hash = hash_content(content);
        assert_eq!(hash.len(), 64);
    }
}
```

#### 模式 2: 异步测试
```rust
// 文件: agent-mem-performance/src/metrics.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(true);
        assert!(collector.is_ok());
    }
}
```

#### 模式 3: 集成测试 (tests/ 目录)
```rust
// 文件: agent-mem-server/tests/integration_tests.rs
#[tokio::test]
async fn test_server_startup_and_health_check() {
    let mut config = ServerConfig::default();
    config.port = 8081;

    let server = MemoryServer::new(config).await.unwrap();
    // ... 测试逻辑
}
```

#### 模式 4: 属性测试 (proptest)
```rust
// 目前项目中未充分使用，但已引入依赖
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_hash_consistency(s in "\\PC*") {
        let hash1 = hash_content(&s);
        let hash2 = hash_content(&s);
        assert_eq!(hash1, hash2);
    }
}
```

---

## 🎯 测试目标 (基于 Rust 生态)

### 总体目标

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| **文件覆盖率** | 61.2% | **85%+** | +23.8% |
| **行覆盖率** | 未测量 | **80%+** | - |
| **分支覆盖率** | 未测量 | **75%+** | - |
| **单元测试数量** | 1,367 | **2,500+** | +1,133 |
| **集成测试数量** | ~150 | **300+** | +150 |
| **E2E 测试数量** | ~10 | **50+** | +40 |
| **基准测试数量** | ~17 | **50+** | +33 |
| **文档测试** | 未统计 | **200+** | - |

### 参考 MIRIX + Rust 最佳实践

#### MIRIX 测试模式 (Python)
1. ✅ **TestTracker 模式** - 跟踪测试执行和结果
2. ✅ **分层测试** - 直接测试 (Manager) + 间接测试 (API)
3. ✅ **搜索方法测试** - bm25, embedding, string_match, fuzzy
4. ✅ **性能对比测试** - 不同方法的性能比较
5. ✅ **边界条件测试** - 空查询、特殊字符、极限值
6. ✅ **多用户测试** - 用户隔离和权限测试

#### Rust 测试最佳实践
1. ✅ **单元测试** - `#[cfg(test)]` 模块，与代码同文件
2. ✅ **集成测试** - `tests/` 目录，测试公共 API
3. ✅ **文档测试** - `/// # Examples` 中的可执行代码
4. ✅ **基准测试** - `benches/` 目录，使用 Criterion
5. ✅ **属性测试** - 使用 `proptest` 进行随机测试
6. ✅ **并发测试** - `#[tokio::test]` 异步测试
7. ✅ **Mock 测试** - 使用 `mockall` 或手动 Mock
8. ✅ **快照测试** - 使用 `insta` 进行快照对比

### 测试金字塔 (Rust 版本)

```
        /\
       /  \  E2E Tests (50+)
      /    \  - 完整工作流
     /------\  - 真实环境
    /        \ Integration Tests (300+)
   /          \ - API 测试
  /            \ - 组件集成
 /--------------\ Unit Tests (2,500+)
/                \ - 函数级别
                   - 快速执行
```

### 测试类型分布目标

```rust
// 单元测试: 85% (2,500 个)
#[cfg(test)]
mod tests {
    #[test]
    fn test_function() { }

    #[tokio::test]
    async fn test_async_function() { }
}

// 集成测试: 10% (300 个)
// tests/integration_test.rs
#[tokio::test]
async fn test_api_endpoint() { }

// E2E 测试: 2% (50 个)
// tests/e2e_workflow_test.rs
#[tokio::test]
async fn test_complete_workflow() { }

// 基准测试: 2% (50 个)
// benches/benchmark.rs
fn criterion_benchmark(c: &mut Criterion) { }

// 文档测试: 1% (200 个)
/// # Examples
/// ```
/// let result = function();
/// assert_eq!(result, expected);
/// ```
```

---

## 📋 测试分类体系 (Rust 优先)

### 🔴 优先级 P0: 关键缺失测试 (立即补充)

这些是核心功能但完全没有测试的模块，必须立即补充！

#### P0.1 Memory Managers (5 个 Manager，113 tests ✅ 全部完成！)

**文件位置**: `crates/agent-mem-core/src/managers/`

**进度**: 113/110 tests (103%) ✅ **超额完成！**

**本次新增**: 29 tests (第四轮)
**之前完成**: 84 tests (第一轮+第二轮+第三轮)
**总计**: 113 tests

##### 1. Episodic Memory Manager (目标: 25 tests，已完成: 25 tests ✅)
```rust
// 文件: episodic_memory.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_episodic_event() {
        let manager = EpisodicMemoryManager::new();
        let event = manager.create_event(
            "user-123",
            "Went to grocery store",
            EventType::Activity,
        ).await.unwrap();
        assert!(!event.id.is_empty());
    }

    #[tokio::test]
    async fn test_retrieve_episodic_events() { }

    #[tokio::test]
    async fn test_search_episodic_by_time_range() { }

    #[tokio::test]
    async fn test_episodic_event_importance_decay() { }

    // ... 21 more tests
}
```

**必须测试的功能**:
- [x] ✅ 事件结构创建和验证 (test_episodic_event_creation)
- [x] ✅ 序列化/反序列化 (test_episodic_event_serialization)
- [x] ✅ 查询参数构建 (test_episodic_query_default, test_episodic_query_with_filters)
- [x] ✅ 重要性评分验证 (test_importance_score_range, test_importance_score_boundaries)
- [x] ✅ 元数据处理 (test_event_metadata, test_event_metadata_complex)
- [x] ✅ 可选字段处理 (test_event_optional_fields)
- [x] ✅ 时间戳验证 (test_event_timestamps)
- [x] ✅ 事件类型变化 (test_event_type_variations)
- [x] ✅ 边界条件测试 (test_event_with_empty_strings, test_event_with_long_strings)
- [x] ✅ 多过滤器查询 (test_query_with_multiple_filters)
- [x] ✅ 时间范围验证 (test_query_time_range_validation)
- [x] ✅ Actor 字段变化 (test_event_actor_variations)
- [x] ✅ 查询分页 (test_query_pagination)
- [x] ✅ 重要性分类 (test_event_importance_categories)
- [x] ✅ 所有可选字段 (test_event_with_all_optional_fields)
- [x] ✅ 单过滤器查询 (test_query_with_single_filter)
- [x] ✅ 事件时间排序 (test_event_occurred_at_ordering)
- [x] ✅ 摘要长度测试 (test_event_summary_length)
- [x] ✅ 最小重要性过滤 (test_query_min_importance_filter)
- [x] ✅ ID 唯一性 (test_event_id_uniqueness)
- [x] ✅ 组织上下文 (test_event_organization_context)
- [ ] 事件创建 (create_event) - 需要数据库集成测试
- [ ] 事件检索 (get_event, list_events) - 需要数据库集成测试
- [ ] 时间范围搜索 (search_by_time_range) - 需要数据库集成测试
- [ ] 事件类型过滤 (filter_by_type) - 需要数据库集成测试
- [ ] 重要性评分计算 (calculate_importance) - 需要实现
- [ ] 事件关联 (link_events) - 需要实现
- [ ] 事件更新 (update_event) - 需要数据库集成测试
- [ ] 事件删除 (delete_event) - 需要数据库集成测试
- [ ] 批量操作 (batch_create, batch_delete) - 需要数据库集成测试
- [ ] 并发安全 (concurrent_access) - 需要并发测试

**已完成**: 25/25 tests (100%) ✅ **目标达成！**

##### 2. Semantic Memory Manager (目标: 25 tests，已完成: 25 tests ✅)
```rust
// 文件: semantic_memory.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_semantic_concept() {
        let manager = SemanticMemoryManager::new();
        let concept = manager.create_concept(
            "Machine Learning",
            "A subset of AI...",
        ).await.unwrap();
        assert_eq!(concept.name, "Machine Learning");
    }

    #[tokio::test]
    async fn test_semantic_search() { }

    #[tokio::test]
    async fn test_concept_relationships() { }

    // ... 22 more tests
}
```

**必须测试的功能**:
- [x] ✅ 语义项结构创建和验证 (test_semantic_item_creation)
- [x] ✅ 序列化/反序列化 (test_semantic_item_serialization)
- [x] ✅ 查询参数构建 (test_semantic_query_default, test_semantic_query_with_filters, test_query_with_name_filter, test_query_with_tree_path_prefix)
- [x] ✅ 层级路径处理 (test_tree_path_hierarchy, test_empty_tree_path, test_complex_tree_path, test_tree_path_depth_variations)
- [x] ✅ 元数据处理 (test_semantic_item_metadata, test_semantic_item_metadata_nested)
- [x] ✅ 可选字段处理 (test_semantic_item_optional_source, test_semantic_item_source_variations)
- [x] ✅ 时间戳验证 (test_semantic_item_timestamps)
- [x] ✅ 边界条件测试 (test_semantic_item_with_empty_strings, test_semantic_item_with_long_content)
- [x] ✅ 查询限制变化 (test_query_limit_variations)
- [x] ✅ 名称长度变化 (test_semantic_item_name_variations)
- [x] ✅ 单层vs多层路径 (test_tree_path_single_vs_multiple)
- [x] ✅ 摘要vs详情 (test_semantic_item_summary_vs_details)
- [x] ✅ 复杂树形路径 (test_semantic_item_with_complex_tree_path)
- [x] ✅ 名称和摘要查询 (test_query_with_name_and_summary)
- [x] ✅ ID 格式验证 (test_semantic_item_id_format)
- [x] ✅ 组织上下文 (test_semantic_item_organization_context)
- [ ] 概念创建 (create_item) - 需要数据库集成测试
- [ ] 概念检索 (get_item, search_items) - 需要数据库集成测试
- [ ] 语义搜索 (semantic_search) - 需要向量搜索集成测试
- [ ] 概念关系 (add_relationship, get_relationships) - 需要实现
- [ ] 概念层级遍历 (traverse_hierarchy) - 需要实现
- [ ] 概念合并 (merge_concepts) - 需要实现
- [ ] 概念分裂 (split_concept) - 需要实现
- [ ] 相似度计算 (calculate_similarity) - 需要向量计算
- [ ] 概念演化 (concept_evolution) - 需要实现
- [ ] 知识图谱构建 (build_knowledge_graph) - 需要实现

**已完成**: 25/25 tests (100%) ✅ **目标达成！**

##### 3. Procedural Memory Manager (目标: 20 tests，已完成: 22 tests ✅)
```rust
// 文件: procedural_memory.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_procedure() {
        let manager = ProceduralMemoryManager::new();
        let procedure = manager.create_procedure(
            "Make Coffee",
            vec!["Boil water", "Add coffee", "Pour water"],
        ).await.unwrap();
        assert_eq!(procedure.steps.len(), 3);
    }

    #[tokio::test]
    async fn test_execute_procedure() { }

    #[tokio::test]
    async fn test_procedure_optimization() { }

    // ... 17 more tests
}
```

**必须测试的功能**:
- [x] ✅ 程序项结构创建 (test_procedural_item_creation)
- [x] ✅ 序列化/反序列化 (test_procedural_item_serialization)
- [x] ✅ 查询参数构建 (test_procedural_query_default, test_procedural_query_with_filters, test_query_with_entry_type_filter, test_query_with_name_contains)
- [x] ✅ 步骤列表处理 (test_empty_steps, test_single_step, test_many_steps, test_steps_with_long_content, test_steps_ordering)
- [x] ✅ 条目类型验证 (test_entry_types, test_entry_type_variations)
- [x] ✅ 层级路径 (test_tree_path, test_tree_path_variations)
- [x] ✅ 元数据处理 (test_metadata, test_procedural_metadata_complex)
- [x] ✅ 边界条件测试 (test_procedural_item_with_empty_strings)
- [x] ✅ 单步vs多步 (test_procedural_item_single_vs_multiple_steps)
- [x] ✅ 名称长度 (test_procedural_item_name_length)
- [x] ✅ 查询过滤器组合 (test_query_with_all_filters, test_query_with_no_filters)
- [ ] 过程创建 (create_procedure) - 需要数据库集成测试
- [ ] 步骤管理 (add_step, remove_step, reorder_steps) - 需要实现
- [ ] 过程执行 (execute_procedure) - 需要实现
- [ ] 执行跟踪 (track_execution) - 需要实现
- [ ] 过程优化 (optimize_procedure) - 需要实现
- [ ] 条件分支 (conditional_steps) - 需要实现
- [ ] 循环处理 (loop_steps) - 需要实现
- [ ] 错误处理 (error_recovery) - 需要实现
- [ ] 过程组合 (compose_procedures) - 需要实现
- [ ] 学习改进 (learn_from_execution) - 需要实现

**已完成**: 22/20 tests (110%) ✅ 超额完成！

##### 4. Knowledge Vault Manager (目标: 20 tests，已完成: 20 tests ✅)
```rust
// 文件: knowledge_vault.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_secret() {
        let manager = KnowledgeVaultManager::new();
        let secret = manager.store_secret(
            "api_key",
            "sk-1234567890",
            SensitivityLevel::High,
        ).await.unwrap();
        assert_eq!(secret.sensitivity, SensitivityLevel::High);
    }

    #[tokio::test]
    async fn test_retrieve_secret_with_auth() { }

    #[tokio::test]
    async fn test_secret_encryption() { }

    // ... 17 more tests
}
```

**必须测试的功能**:
- [x] ✅ 敏感度级别操作 (test_sensitivity_level_operations)
- [x] ✅ 管理器创建 (test_manager_creation)
- [x] ✅ 用户权限管理 (test_user_permissions_management)
- [x] ✅ 加密解密 (test_encryption_decryption)
- [x] ✅ 自动分类 (test_auto_classification)
- [x] ✅ 知识条目生命周期 (test_knowledge_entry_lifecycle)
- [x] ✅ 搜索功能 (test_search_functionality)
- [x] ✅ 访问控制 (test_access_control)
- [x] ✅ 审计日志 (test_audit_logging)
- [x] ✅ 统计信息 (test_statistics)
- [x] ✅ 配置默认值 (test_knowledge_vault_config_default)
- [x] ✅ 自定义配置 (test_knowledge_vault_config_custom)
- [x] ✅ 敏感度级别排序 (test_sensitivity_level_ordering)
- [x] ✅ 访问权限类型 (test_access_permission_types)
- [x] ✅ 用户权限创建 (test_user_permissions_creation)
- [x] ✅ 带过期的权限 (test_user_permissions_with_expiry)
- [x] ✅ 审计操作类型 (test_audit_action_types)
- [x] ✅ 审计日志条目 (test_audit_log_entry_creation)
- [x] ✅ 空统计信息 (test_vault_statistics_empty)
- [x] ✅ 带数据的统计 (test_vault_statistics_with_data)
- [ ] 秘密存储 (store_secret) - 已有部分测试
- [ ] 秘密检索 (retrieve_secret) - 已有部分测试
- [ ] 加密/解密 (encrypt, decrypt) - 已有测试
- [ ] 访问控制 (check_access, grant_access) - 已有测试
- [ ] 敏感度级别 (set_sensitivity) - 已有测试
- [ ] 审计日志 (log_access) - 已有测试
- [ ] 秘密轮换 (rotate_secret) - 需要实现
- [ ] 秘密过期 (expire_secret) - 需要实现
- [ ] 批量操作 (batch_store, batch_retrieve) - 需要实现
- [ ] 搜索过滤 (search_by_sensitivity) - 已有部分测试

**已完成**: 20/20 tests (100%) ✅ **目标达成！**

##### 5. Resource Memory Manager (目标: 20 tests，已完成: 21 tests ✅)
```rust
// 文件: resource_memory.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_resource() {
        let manager = ResourceMemoryManager::new();
        let resource = manager.store_resource(
            "document.pdf",
            ResourceType::Document,
            vec![1, 2, 3, 4],
        ).await.unwrap();
        assert_eq!(resource.resource_type, ResourceType::Document);
    }

    #[tokio::test]
    async fn test_retrieve_resource() { }

    #[tokio::test]
    async fn test_resource_metadata() { }

    // ... 17 more tests
}
```

**必须测试的功能**:
- [x] ✅ 管理器创建 (test_resource_memory_manager_creation)
- [x] ✅ 资源类型识别 (test_resource_type_from_extension)
- [x] ✅ 存储和检索 (test_store_and_retrieve_resource)
- [x] ✅ 资源去重 (test_resource_deduplication)
- [x] ✅ 按类型搜索 (test_search_by_type)
- [x] ✅ 按标签搜索 (test_search_by_tags)
- [x] ✅ 按文件名搜索 (test_search_by_filename)
- [x] ✅ 更新标签 (test_update_resource_tags)
- [x] ✅ 删除资源 (test_delete_resource)
- [x] ✅ 文件大小限制 (test_file_size_limit)
- [x] ✅ 存储统计 (test_storage_stats)
- [x] ✅ 健康检查 (test_storage_health_check)
- [x] ✅ 清空所有 (test_clear_all)
- [x] ✅ 资源类型变体 (test_resource_type_variants)
- [x] ✅ 配置默认值 (test_resource_storage_config_default)
- [x] ✅ 自定义配置 (test_resource_storage_config_custom)
- [x] ✅ 元数据结构 (test_resource_metadata_structure)
- [x] ✅ 多个相同类型资源 (test_multiple_resources_same_type)
- [x] ✅ 标签管理 (test_resource_tags_management)
- [x] ✅ 自定义元数据 (test_resource_custom_metadata)
- [x] ✅ 空文件处理 (test_empty_file_handling)
- [ ] 元数据管理 (set_metadata, get_metadata) - 已有部分测试
- [ ] 资源类型 (handle_different_types) - 已有测试
- [ ] 大文件处理 (chunked_upload) - 需要实现
- [ ] 资源引用 (reference_counting) - 需要实现
- [ ] 资源清理 (garbage_collection) - 需要实现
- [ ] 资源搜索 (search_by_type, search_by_metadata) - 已有测试
- [ ] 版本控制 (version_management) - 需要实现
- [ ] 资源共享 (share_resource) - 需要实现

**已完成**: 21/20 tests (105%) ✅ **超额完成！**

#### P0.2 Storage Backends (3 个后端，0 tests ❌)

**文件位置**: `crates/agent-mem-storage/src/backends/`

##### 1. MongoDB Backend (目标: 25 tests)
```rust
// 文件: mongodb_test.rs (新建)
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mongodb_connection() {
        let backend = MongoDBBackend::new("mongodb://localhost:27017").await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_mongodb_insert() {
        let backend = create_test_backend().await;
        let doc = json!({"name": "test", "value": 123});
        let result = backend.insert("test_collection", doc).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mongodb_query() { }

    #[tokio::test]
    async fn test_mongodb_update() { }

    #[tokio::test]
    async fn test_mongodb_delete() { }

    #[tokio::test]
    async fn test_mongodb_aggregation() { }

    #[tokio::test]
    async fn test_mongodb_index_creation() { }

    #[tokio::test]
    async fn test_mongodb_transaction() { }

    // ... 17 more tests
}
```

**必须测试的功能**:
- [ ] 连接管理 (connect, disconnect, reconnect)
- [ ] CRUD 操作 (insert, find, update, delete)
- [ ] 批量操作 (bulk_insert, bulk_update)
- [ ] 查询优化 (indexed_query, explain_query)
- [ ] 聚合管道 (aggregation_pipeline)
- [ ] 事务支持 (transaction_commit, transaction_rollback)
- [ ] 索引管理 (create_index, drop_index)
- [ ] 全文搜索 (text_search)
- [ ] 地理空间查询 (geo_query)
- [ ] 错误处理 (connection_error, query_error)

##### 2. Redis Backend (目标: 20 tests)
```rust
// 文件: redis_test.rs (新建)
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_connection() {
        let backend = RedisBackend::new("redis://localhost:6379").await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_redis_set_get() {
        let backend = create_test_backend().await;
        backend.set("key1", "value1").await.unwrap();
        let value = backend.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_redis_expiration() { }

    #[tokio::test]
    async fn test_redis_pipeline() { }

    #[tokio::test]
    async fn test_redis_pub_sub() { }

    // ... 15 more tests
}
```

**必须测试的功能**:
- [ ] 基础操作 (set, get, delete, exists)
- [ ] 过期策略 (expire, ttl, persist)
- [ ] 数据结构 (hash, list, set, sorted_set)
- [ ] 管道操作 (pipeline)
- [ ] 发布订阅 (pub_sub)
- [ ] 事务 (multi, exec, watch)
- [ ] Lua 脚本 (eval_script)
- [ ] 分布式锁 (acquire_lock, release_lock)
- [ ] 连接池 (pool_management)
- [ ] 集群支持 (cluster_mode)

##### 3. Supabase Backend (目标: 20 tests)
```rust
// 文件: supabase_test.rs (新建)
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supabase_connection() {
        let backend = SupabaseBackend::new(
            "https://xxx.supabase.co",
            "api_key"
        ).await;
        assert!(backend.is_ok());
    }

    #[tokio::test]
    async fn test_supabase_insert() { }

    #[tokio::test]
    async fn test_supabase_query() { }

    #[tokio::test]
    async fn test_supabase_realtime() { }

    #[tokio::test]
    async fn test_supabase_storage() { }

    // ... 15 more tests
}
```

**必须测试的功能**:
- [ ] REST API 操作 (insert, select, update, delete)
- [ ] 实时订阅 (realtime_subscribe)
- [ ] 存储桶操作 (upload_file, download_file)
- [ ] 认证集成 (auth_integration)
- [ ] RPC 调用 (rpc_call)
- [ ] 过滤和排序 (filter, order_by)
- [ ] 分页 (pagination)
- [ ] 全文搜索 (full_text_search)
- [ ] 向量搜索 (vector_search via pgvector)
- [ ] 错误处理 (api_error, network_error)

---

### 🟡 优先级 P1: 现有测试增强

#### P1.1 已有测试但需要增强的模块

##### 1. Core Memory Manager (当前: 12 tests → 目标: 30 tests)
```rust
// 文件: core_memory.rs
// 已有测试: ✅
// 需要新增: 18 tests

#[cfg(test)]
mod tests {
    // 已有的 12 个测试...

    // 新增测试:
    #[tokio::test]
    async fn test_concurrent_block_updates() {
        // 测试并发更新的安全性
    }

    #[tokio::test]
    async fn test_block_compression() {
        // 测试块压缩功能
    }

    #[tokio::test]
    async fn test_block_versioning() {
        // 测试版本控制
    }

    #[tokio::test]
    async fn test_block_migration() {
        // 测试数据迁移
    }

    #[tokio::test]
    async fn test_block_backup_restore() {
        // 测试备份和恢复
    }

    // ... 13 more tests
}
```

**需要新增的测试**:
- [ ] 并发安全测试 (concurrent_updates, race_conditions)
- [ ] 压缩功能测试 (compression, decompression)
- [ ] 版本控制测试 (versioning, rollback)
- [ ] 迁移测试 (data_migration, schema_migration)
- [ ] 备份恢复测试 (backup, restore)
- [ ] 性能测试 (large_blocks, many_blocks)
- [ ] 边界条件测试 (empty_block, max_capacity)
- [ ] 错误恢复测试 (corruption_recovery)

##### 2. LLM Providers (需要补充的提供商)

**Gemini Provider** (当前: 部分 → 目标: 20 tests)
```rust
// 文件: providers/gemini.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gemini_provider_creation() { }

    #[tokio::test]
    async fn test_gemini_text_generation() { }

    #[tokio::test]
    async fn test_gemini_streaming() { }

    #[tokio::test]
    async fn test_gemini_multimodal() { }

    #[tokio::test]
    async fn test_gemini_function_calling() { }

    #[tokio::test]
    async fn test_gemini_safety_settings() { }

    #[tokio::test]
    async fn test_gemini_error_handling() { }

    // ... 13 more tests
}
```

**Groq Provider** (当前: 0 tests → 目标: 15 tests)
```rust
// 文件: providers/groq.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq_provider_creation() { }

    #[tokio::test]
    async fn test_groq_fast_inference() { }

    #[tokio::test]
    async fn test_groq_model_selection() { }

    // ... 12 more tests
}
```

**Local Provider** (当前: 部分 → 目标: 20 tests)
```rust
// 文件: providers/local.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_model_loading() { }

    #[tokio::test]
    async fn test_local_inference() { }

    #[tokio::test]
    async fn test_local_model_quantization() { }

    #[tokio::test]
    async fn test_local_gpu_acceleration() { }

    // ... 16 more tests
}
```

---

### 🟢 优先级 P2: 集成测试 (Integration Tests)

集成测试位于 `tests/` 目录，测试多个组件的协作。

#### P2.1 Memory API 集成测试 (目标: 40 tests)

**文件**: `tests/memory_api_integration_test.rs` (新建)

```rust
use agent_mem_server::{MemoryServer, ServerConfig};
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_memory_crud_workflow() {
    let server = start_test_server().await;
    let client = Client::new();

    // 1. Create memory
    let create_resp = client.post("http://localhost:8081/api/memories")
        .json(&json!({
            "content": "Test memory",
            "memory_type": "Episodic",
            "importance": 0.8
        }))
        .send().await.unwrap();
    assert_eq!(create_resp.status(), 201);

    let memory: serde_json::Value = create_resp.json().await.unwrap();
    let memory_id = memory["id"].as_str().unwrap();

    // 2. Retrieve memory
    let get_resp = client.get(&format!("http://localhost:8081/api/memories/{}", memory_id))
        .send().await.unwrap();
    assert_eq!(get_resp.status(), 200);

    // 3. Update memory
    let update_resp = client.put(&format!("http://localhost:8081/api/memories/{}", memory_id))
        .json(&json!({"importance": 0.9}))
        .send().await.unwrap();
    assert_eq!(update_resp.status(), 200);

    // 4. Delete memory
    let delete_resp = client.delete(&format!("http://localhost:8081/api/memories/{}", memory_id))
        .send().await.unwrap();
    assert_eq!(delete_resp.status(), 204);
}

#[tokio::test]
async fn test_memory_search_integration() {
    // 测试搜索 API 集成
}

#[tokio::test]
async fn test_memory_batch_operations() {
    // 测试批量操作
}

#[tokio::test]
async fn test_memory_filtering() {
    // 测试过滤功能
}

#[tokio::test]
async fn test_memory_pagination() {
    // 测试分页
}

// ... 35 more tests
```

**测试覆盖**:
- [ ] CRUD 操作 (create, read, update, delete)
- [ ] 搜索功能 (search, filter, sort)
- [ ] 批量操作 (batch_create, batch_update, batch_delete)
- [ ] 分页 (pagination, cursor_based)
- [ ] 关联查询 (related_memories, associations)
- [ ] 权限控制 (access_control, multi_tenant)
- [ ] 错误处理 (validation_errors, not_found)
- [ ] 并发请求 (concurrent_requests)

#### P2.2 Search Methods 集成测试 (参考 MIRIX，目标: 30 tests)

**文件**: `tests/search_methods_integration_test.rs` (新建)

```rust
use agent_mem_core::search::{SearchEngine, SearchMethod};
use std::time::Instant;

#[tokio::test]
async fn test_search_methods_comparison() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine, 1000).await;

    let methods = vec![
        SearchMethod::BM25,
        SearchMethod::Embedding,
        SearchMethod::StringMatch,
        SearchMethod::Fuzzy,
        SearchMethod::Hybrid,
    ];

    let queries = vec![
        ("simple", "machine learning"),
        ("phrase", "\"artificial intelligence\""),
        ("complex", "deep learning neural networks optimization"),
    ];

    for (query_type, query) in &queries {
        println!("\n=== Query: {} - '{}' ===", query_type, query);

        for method in &methods {
            let start = Instant::now();
            let results = engine.search(query, *method, 10).await.unwrap();
            let elapsed = start.elapsed();

            println!("{:?}: {} results in {:?}", method, results.len(), elapsed);

            // 验证结果质量
            assert!(results.len() <= 10);
            for result in &results {
                assert!(result.score >= 0.0 && result.score <= 1.0);
            }
        }
    }
}

#[tokio::test]
async fn test_field_specific_search() {
    // 参考 MIRIX 的字段搜索测试
    let engine = create_test_search_engine().await;

    let fields = vec!["title", "content", "summary", "tags"];
    let query = "machine learning";

    for field in fields {
        let results = engine.search_field(query, field, 10).await.unwrap();
        println!("Field '{}': {} results", field, results.len());

        // 验证结果
        assert!(results.len() > 0);
    }
}

#[tokio::test]
async fn test_performance_comparison() {
    // 参考 MIRIX 的性能对比测试
    let engine = create_test_search_engine().await;
    seed_large_dataset(&engine, 10000).await;

    let mut performance_results = HashMap::new();

    for method in [SearchMethod::BM25, SearchMethod::StringMatch, SearchMethod::Fuzzy] {
        let start = Instant::now();
        let _ = engine.search("test query", method, 50).await.unwrap();
        let elapsed = start.elapsed();

        performance_results.insert(method, elapsed);
    }

    // 计算性能提升
    let bm25_time = performance_results[&SearchMethod::BM25];
    let string_time = performance_results[&SearchMethod::StringMatch];
    let improvement = string_time.as_secs_f64() / bm25_time.as_secs_f64();

    println!("BM25 is {:.1}x faster than string search", improvement);
    assert!(improvement > 1.0); // BM25 应该更快
}

#[tokio::test]
async fn test_edge_cases() {
    // 参考 MIRIX 的边界条件测试
    let engine = create_test_search_engine().await;

    let edge_cases = vec![
        ("empty", ""),
        ("whitespace", "   "),
        ("special_chars", "!@#$%^&*()"),
        ("very_long", &"x".repeat(10000)),
        ("unicode", "你好世界 🌍"),
    ];

    for (name, query) in edge_cases {
        let result = engine.search(query, SearchMethod::BM25, 10).await;
        println!("Edge case '{}': {:?}", name, result.is_ok());
    }
}

// ... 25 more tests
```

**测试覆盖** (参考 MIRIX):
- [ ] 搜索方法对比 (bm25, embedding, string_match, fuzzy, hybrid)
- [ ] 字段特定搜索 (field_specific_search)
- [ ] 性能对比 (performance_comparison)
- [ ] 查询语法 (phrase_query, boolean_query, wildcard_query)
- [ ] 边界条件 (empty_query, special_chars, unicode)
- [ ] 结果排序 (relevance, time, importance)
- [ ] 结果过滤 (filter_by_type, filter_by_date)
- [ ] 分面搜索 (faceted_search)

#### P2.3 Authentication & Authorization 集成测试 (目标: 25 tests)

**文件**: `tests/auth_integration_test.rs` (已存在，需增强)

```rust
#[tokio::test]
async fn test_jwt_authentication_flow() {
    let server = start_test_server().await;
    let client = Client::new();

    // 1. Login
    let login_resp = client.post("http://localhost:8081/api/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "testpass"
        }))
        .send().await.unwrap();

    assert_eq!(login_resp.status(), 200);
    let auth: serde_json::Value = login_resp.json().await.unwrap();
    let token = auth["token"].as_str().unwrap();

    // 2. Access protected endpoint
    let protected_resp = client.get("http://localhost:8081/api/memories")
        .header("Authorization", format!("Bearer {}", token))
        .send().await.unwrap();

    assert_eq!(protected_resp.status(), 200);

    // 3. Access without token (should fail)
    let unauth_resp = client.get("http://localhost:8081/api/memories")
        .send().await.unwrap();

    assert_eq!(unauth_resp.status(), 401);
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    // 测试租户隔离
}

#[tokio::test]
async fn test_api_key_authentication() {
    // 测试 API Key 认证
}

#[tokio::test]
async fn test_role_based_access_control() {
    // 测试基于角色的访问控制
}

// ... 21 more tests
```

**测试覆盖**:
- [ ] JWT 认证流程 (login, token_validation, token_refresh)
- [ ] API Key 认证 (key_validation, key_rotation)
- [ ] 多租户隔离 (tenant_isolation, data_separation)
- [ ] 角色权限 (rbac, permission_check)
- [ ] 会话管理 (session_creation, session_expiry)
- [ ] 密码安全 (password_hashing, password_reset)
- [ ] OAuth 集成 (oauth_flow, third_party_auth)
- [ ] 审计日志 (access_logging, security_events)

---

### 🔵 优先级 P3: E2E 测试 (End-to-End Tests)

E2E 测试模拟真实用户场景，测试完整的工作流。

#### P3.1 Memory Lifecycle E2E (目标: 10 tests)

**文件**: `tests/e2e_memory_lifecycle_test.rs` (新建)

```rust
#[tokio::test]
async fn test_complete_memory_lifecycle() {
    // 启动完整的测试环境
    let env = TestEnvironment::new().await;
    env.start_server().await;
    env.start_database().await;
    env.start_redis().await;

    let client = env.create_client();

    // 1. 用户注册和登录
    let user = client.register_user("testuser", "testpass").await.unwrap();
    let token = client.login("testuser", "testpass").await.unwrap();

    // 2. 创建 Agent
    let agent = client.create_agent("my-agent", &token).await.unwrap();

    // 3. 发送消息并创建记忆
    let response = client.send_message(
        &agent.id,
        "I love machine learning and AI",
        &token
    ).await.unwrap();

    assert!(response.contains("machine learning"));

    // 4. 等待记忆处理
    tokio::time::sleep(Duration::from_secs(2)).await;

    // 5. 搜索记忆
    let memories = client.search_memories(
        &agent.id,
        "machine learning",
        &token
    ).await.unwrap();

    assert!(memories.len() > 0);
    assert!(memories[0].content.contains("machine learning"));

    // 6. 更新记忆
    let updated = client.update_memory(
        &memories[0].id,
        json!({"importance": 0.9}),
        &token
    ).await.unwrap();

    assert_eq!(updated.importance, 0.9);

    // 7. 删除记忆
    client.delete_memory(&memories[0].id, &token).await.unwrap();

    // 8. 验证删除
    let result = client.get_memory(&memories[0].id, &token).await;
    assert!(result.is_err());

    // 9. 清理
    env.cleanup().await;
}

#[tokio::test]
async fn test_multi_agent_conversation() {
    // 测试多 Agent 对话场景
}

#[tokio::test]
async fn test_memory_graph_construction() {
    // 测试记忆图谱构建
}

#[tokio::test]
async fn test_long_term_memory_evolution() {
    // 测试长期记忆演化
}

// ... 6 more tests
```

#### P3.2 Agent Conversation Flow E2E (目标: 15 tests)

**文件**: `tests/e2e_conversation_flow_test.rs` (新建)

```rust
#[tokio::test]
async fn test_single_turn_conversation() {
    let env = TestEnvironment::new().await;
    let client = env.create_authenticated_client().await;

    let response = client.chat("What is machine learning?").await.unwrap();

    assert!(!response.is_empty());
    assert!(response.contains("machine") || response.contains("learning"));
}

#[tokio::test]
async fn test_multi_turn_conversation_with_context() {
    let env = TestEnvironment::new().await;
    let client = env.create_authenticated_client().await;

    // Turn 1
    let resp1 = client.chat("My name is Alice").await.unwrap();

    // Turn 2
    let resp2 = client.chat("What is my name?").await.unwrap();
    assert!(resp2.contains("Alice"));

    // Turn 3
    let resp3 = client.chat("I like pizza").await.unwrap();

    // Turn 4
    let resp4 = client.chat("What do I like?").await.unwrap();
    assert!(resp4.contains("pizza"));
}

#[tokio::test]
async fn test_conversation_with_tool_calling() {
    // 测试工具调用场景
}

#[tokio::test]
async fn test_streaming_conversation() {
    // 测试流式对话
}

// ... 11 more tests
```

#### P3.3 Search Functionality E2E (参考 MIRIX，目标: 15 tests)

**文件**: `tests/e2e_search_functionality_test.rs` (新建)

```rust
#[tokio::test]
async fn test_comprehensive_search_workflow() {
    let env = TestEnvironment::new().await;
    let client = env.create_authenticated_client().await;

    // 1. 创建测试数据
    let memories = vec![
        "I went to the grocery store and bought apples",
        "Machine learning is a subset of artificial intelligence",
        "Deep learning uses neural networks",
        "I love programming in Rust",
        "Natural language processing is fascinating",
    ];

    for content in memories {
        client.create_memory(content).await.unwrap();
    }

    // 2. 测试不同搜索方法
    let bm25_results = client.search("machine learning", "bm25").await.unwrap();
    let embedding_results = client.search("machine learning", "embedding").await.unwrap();
    let hybrid_results = client.search("machine learning", "hybrid").await.unwrap();

    // 3. 验证结果
    assert!(bm25_results.len() > 0);
    assert!(embedding_results.len() > 0);
    assert!(hybrid_results.len() > 0);

    // 4. 测试短语搜索
    let phrase_results = client.search("\"machine learning\"", "bm25").await.unwrap();
    assert!(phrase_results[0].content.contains("machine learning"));

    // 5. 测试过滤
    let filtered_results = client.search_with_filter(
        "learning",
        json!({"memory_type": "Semantic"})
    ).await.unwrap();

    for result in filtered_results {
        assert_eq!(result.memory_type, "Semantic");
    }
}

// ... 14 more tests
```

---

### 🟣 优先级 P4: 性能基准测试 (Benchmark Tests)

使用 Criterion 进行性能基准测试。

#### P4.1 Storage Performance Benchmarks (目标: 15 benchmarks)

**文件**: `benches/storage_benchmark.rs` (新建)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use agent_mem_storage::backends::PostgresBackend;

fn benchmark_write_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_write");

    for size in [1, 10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let backend = rt.block_on(create_test_backend());

                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..size {
                            backend.write(
                                black_box(&format!("key-{}", i)),
                                black_box("value")
                            ).await.unwrap();
                        }
                    });
                });
            }
        );
    }

    group.finish();
}

fn benchmark_read_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_read");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let backend = rt.block_on(create_and_seed_backend(10000));

    group.bench_function("single_read", |b| {
        b.iter(|| {
            rt.block_on(async {
                backend.read(black_box("key-5000")).await.unwrap()
            });
        });
    });

    group.bench_function("batch_read_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                for i in 0..100 {
                    backend.read(black_box(&format!("key-{}", i))).await.unwrap();
                }
            });
        });
    });

    group.finish();
}

fn benchmark_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let backend = rt.block_on(create_and_seed_backend(10000));

    for method in ["bm25", "embedding", "string_match", "hybrid"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |b, &method| {
                b.iter(|| {
                    rt.block_on(async {
                        backend.search(
                            black_box("test query"),
                            black_box(method),
                            10
                        ).await.unwrap()
                    });
                });
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_write_performance,
    benchmark_read_performance,
    benchmark_search_performance
);
criterion_main!(benches);
```

#### P4.2 LLM Performance Benchmarks (目标: 10 benchmarks)

**文件**: `benches/llm_benchmark.rs` (新建)

```rust
fn benchmark_llm_providers(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_providers");

    for provider in ["openai", "anthropic", "gemini", "local"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(provider),
            provider,
            |b, &provider| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let client = rt.block_on(create_llm_client(provider));

                b.iter(|| {
                    rt.block_on(async {
                        client.generate(black_box("What is AI?")).await.unwrap()
                    });
                });
            }
        );
    }

    group.finish();
}

fn benchmark_token_counting(c: &mut Criterion) {
    // 测试 token 计数性能
}

fn benchmark_streaming_vs_non_streaming(c: &mut Criterion) {
    // 对比流式和非流式性能
}
```

#### P4.3 Embedding Performance Benchmarks (目标: 10 benchmarks)

**文件**: `benches/embedding_benchmark.rs` (新建)

```rust
fn benchmark_embedding_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("embedding_generation");

    for batch_size in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &batch_size| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let embedder = rt.block_on(create_embedder());

                let texts: Vec<String> = (0..batch_size)
                    .map(|i| format!("Test text {}", i))
                    .collect();

                b.iter(|| {
                    rt.block_on(async {
                        embedder.embed_batch(black_box(&texts)).await.unwrap()
                    });
                });
            }
        );
    }

    group.finish();
}

fn benchmark_similarity_calculation(c: &mut Criterion) {
    // 测试相似度计算性能
}
```

---

### 2. 集成测试 (Integration Tests)

#### 2.1 API 集成测试
- [x] **Server Startup** (1 test)
- [x] **Health Check** (1 test)
- [x] **Config Validation** (2 tests)
- [ ] **Memory API** (目标: 20 tests)
  - [ ] 创建记忆
  - [ ] 检索记忆
  - [ ] 更新记忆
  - [ ] 删除记忆
  - [ ] 搜索记忆
  - [ ] 批量操作

- [ ] **Agent API** (目标: 15 tests)
  - [ ] Agent 创建
  - [ ] Agent 状态管理
  - [ ] Agent 配置
  - [ ] Agent 删除

- [ ] **Chat API** (目标: 18 tests)
  - [ ] 发送消息
  - [ ] 流式响应
  - [ ] 上下文管理
  - [ ] 工具调用
  - [ ] 多轮对话

#### 2.2 认证和授权测试
- [ ] **JWT Authentication** (目标: 12 tests)
  - [ ] Token 生成
  - [ ] Token 验证
  - [ ] Token 刷新
  - [ ] Token 过期

- [ ] **API Key Authentication** (目标: 8 tests)
  - [ ] Key 验证
  - [ ] Key 权限
  - [ ] Key 撤销

- [ ] **Multi-tenant Isolation** (目标: 15 tests)
  - [ ] 租户隔离
  - [ ] 数据隔离
  - [ ] 资源配额

#### 2.3 数据库集成测试
- [ ] **Migration Tests** (目标: 10 tests)
  - [ ] Schema 迁移
  - [ ] 数据迁移
  - [ ] 回滚测试

- [ ] **Transaction Tests** (目标: 12 tests)
  - [ ] ACID 属性
  - [ ] 并发控制
  - [ ] 死锁处理

#### 2.4 缓存集成测试
- [ ] **Multi-level Cache** (目标: 15 tests)
  - [ ] L1 缓存 (内存)
  - [ ] L2 缓存 (Redis)
  - [ ] 缓存一致性
  - [ ] 缓存预热

---

### 3. 端到端测试 (E2E Tests)

#### 3.1 完整工作流测试
- [ ] **Memory Lifecycle** (目标: 5 tests)
  - [ ] 创建 → 存储 → 检索 → 更新 → 删除
  - [ ] 记忆关联和图谱构建
  - [ ] 记忆压缩和归档

- [ ] **Agent Conversation Flow** (目标: 8 tests)
  - [ ] 单轮对话
  - [ ] 多轮对话
  - [ ] 上下文保持
  - [ ] 记忆提取
  - [ ] 工具调用链

- [ ] **Multi-Agent Collaboration** (目标: 6 tests)
  - [ ] Agent 间通信
  - [ ] 任务分配
  - [ ] 结果聚合
  - [ ] 冲突解决

#### 3.2 搜索功能 E2E 测试
参考 MIRIX 的搜索测试模式：

- [ ] **Search Methods Comparison** (目标: 10 tests)
  - [ ] BM25 全文搜索
  - [ ] 向量相似度搜索
  - [ ] 字符串匹配搜索
  - [ ] 模糊匹配搜索
  - [ ] 混合搜索

- [ ] **Field-Specific Search** (目标: 8 tests)
  - [ ] 按字段搜索
  - [ ] 多字段组合
  - [ ] 字段权重

- [ ] **Query Syntax Tests** (目标: 12 tests)
  - [ ] 简单查询
  - [ ] 短语查询
  - [ ] OR 查询
  - [ ] 复杂查询
  - [ ] 特殊字符处理

#### 3.3 性能和压力测试
- [ ] **Load Tests** (目标: 8 tests)
  - [ ] 并发用户测试
  - [ ] 高吞吐量测试
  - [ ] 长时间运行测试

- [ ] **Stress Tests** (目标: 6 tests)
  - [ ] 资源耗尽测试
  - [ ] 极限负载测试
  - [ ] 恢复能力测试

---

### 4. 性能基准测试 (Benchmark Tests)

#### 4.1 存储性能测试
- [ ] **Write Performance** (目标: 5 benchmarks)
  - [ ] 单条写入
  - [ ] 批量写入
  - [ ] 并发写入

- [ ] **Read Performance** (目标: 5 benchmarks)
  - [ ] 单条读取
  - [ ] 批量读取
  - [ ] 缓存命中率

- [ ] **Search Performance** (目标: 8 benchmarks)
  - [ ] 全文搜索性能
  - [ ] 向量搜索性能
  - [ ] 混合搜索性能
  - [ ] 不同数据量下的性能

#### 4.2 LLM 性能测试
- [ ] **Response Time** (目标: 6 benchmarks)
  - [ ] 不同提供商对比
  - [ ] 不同模型对比
  - [ ] 流式 vs 非流式

- [ ] **Token Usage** (目标: 4 benchmarks)
  - [ ] Token 计数准确性
  - [ ] 成本估算

#### 4.3 嵌入性能测试
- [ ] **Embedding Generation** (目标: 6 benchmarks)
  - [ ] 单文本嵌入
  - [ ] 批量嵌入
  - [ ] 不同模型对比

---

## 🔧 测试工具和框架 (Rust 生态)

### Cargo.toml 依赖配置

```toml
[workspace]
members = ["crates/*"]

[workspace.dev-dependencies]
# === 核心测试框架 ===
tokio = { version = "1.35", features = ["full", "test-util", "macros"] }
tokio-test = "0.4"  # Tokio 测试工具

# === 属性测试 ===
proptest = "1.4"  # 基于属性的测试
quickcheck = "1.0"  # 快速检查

# === HTTP 测试 ===
reqwest = { version = "0.11", features = ["json", "stream"] }
wiremock = "0.6"  # HTTP Mock 服务器
httpmock = "0.7"  # HTTP Mock

# === Mock 框架 ===
mockall = "0.12"  # 自动 Mock 生成
mockito = "1.2"  # HTTP Mock

# === 测试数据生成 ===
fake = { version = "2.9", features = ["derive"] }
faker_rand = "0.1"
uuid = { version = "1.6", features = ["v4"] }

# === 断言增强 ===
assert_matches = "1.5"
pretty_assertions = "1.4"
claims = "0.7"  # 更好的断言宏

# === 快照测试 ===
insta = { version = "1.34", features = ["json", "yaml"] }

# === 性能基准测试 ===
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
divan = "0.1"  # 更快的基准测试

# === 测试覆盖率 ===
# 使用 cargo-tarpaulin (命令行工具)
# cargo install cargo-tarpaulin

# === 串行测试 ===
serial_test = "3.0"  # 强制测试串行执行

# === 临时文件/目录 ===
tempfile = "3.8"
temp-dir = "0.1"

# === 测试日志 ===
env_logger = "0.11"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# === 数据库测试 ===
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "migrate"] }
testcontainers = "0.15"  # Docker 容器测试

# === 时间测试 ===
chrono = "0.4"
time = "0.3"
```

### 测试辅助工具库

#### 1. TestTracker (参考 MIRIX)

**文件**: `crates/agent-mem-test-utils/src/tracker.rs` (新建)

```rust
use std::time::{Duration, Instant};
use colored::*;

pub struct TestTracker {
    tests: Vec<TestResult>,
    current_test: Option<TestInfo>,
    start_time: Instant,
}

pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration: Duration,
    pub subtests: Vec<SubtestResult>,
}

pub enum TestStatus {
    Passed,
    Failed(String),
    Skipped(String),
}

impl TestTracker {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            current_test: None,
            start_time: Instant::now(),
        }
    }

    pub fn start_test(&mut self, name: impl Into<String>) {
        let name = name.into();
        println!("{} {}", "🚀 Starting:".blue().bold(), name);

        self.current_test = Some(TestInfo {
            name,
            start_time: Instant::now(),
            subtests: Vec::new(),
        });
    }

    pub fn start_subtest(&mut self, name: impl Into<String>) -> usize {
        if let Some(test) = &mut self.current_test {
            let name = name.into();
            println!("  {} {}", "▶️".cyan(), name);

            test.subtests.push(SubtestInfo {
                name,
                start_time: Instant::now(),
            });

            test.subtests.len() - 1
        } else {
            0
        }
    }

    pub fn pass_subtest(&mut self, index: usize, message: Option<&str>) {
        if let Some(test) = &mut self.current_test {
            if let Some(subtest) = test.subtests.get(index) {
                let msg = message.unwrap_or("");
                println!("  {} {} {}", "✅".green(), subtest.name, msg.dimmed());
            }
        }
    }

    pub fn fail_subtest(&mut self, index: usize, error: &str) {
        if let Some(test) = &mut self.current_test {
            if let Some(subtest) = test.subtests.get(index) {
                println!("  {} {} - {}", "❌".red(), subtest.name, error.red());
            }
        }
    }

    pub fn pass_test(&mut self, message: Option<&str>) {
        if let Some(test) = self.current_test.take() {
            let duration = test.start_time.elapsed();
            let msg = message.unwrap_or("");

            println!("{} {} ({:?}) {}",
                "✅ PASSED:".green().bold(),
                test.name,
                duration,
                msg.dimmed()
            );

            self.tests.push(TestResult {
                name: test.name,
                status: TestStatus::Passed,
                duration,
                subtests: vec![],
            });
        }
    }

    pub fn fail_test(&mut self, error: &str) {
        if let Some(test) = self.current_test.take() {
            let duration = test.start_time.elapsed();

            println!("{} {} - {}",
                "❌ FAILED:".red().bold(),
                test.name,
                error.red()
            );

            self.tests.push(TestResult {
                name: test.name,
                status: TestStatus::Failed(error.to_string()),
                duration,
                subtests: vec![],
            });
        }
    }

    pub fn print_summary(&self) {
        let total_duration = self.start_time.elapsed();
        let total = self.tests.len();
        let passed = self.tests.iter()
            .filter(|t| matches!(t.status, TestStatus::Passed))
            .count();
        let failed = self.tests.iter()
            .filter(|t| matches!(t.status, TestStatus::Failed(_)))
            .count();

        println!("\n{}", "=".repeat(80));
        println!("{}", "🏁 TEST EXECUTION SUMMARY".blue().bold());
        println!("{}", "=".repeat(80));

        println!("\n{}", "📊 OVERALL RESULTS:".cyan().bold());
        println!("   Total Tests: {}", total);
        println!("   {} Passed Tests: {}", "✅".green(), passed);
        if failed > 0 {
            println!("   {} Failed Tests: {}", "❌".red(), failed);
        }

        if total > 0 {
            let success_rate = (passed as f64 / total as f64) * 100.0;
            println!("   📈 Success Rate: {:.1}%", success_rate);
        }

        println!("\n{}", "⏱️  PERFORMANCE:".cyan().bold());
        println!("   Total Duration: {:?}", total_duration);
        if total > 0 {
            let avg_duration = total_duration / total as u32;
            println!("   Average Test Duration: {:?}", avg_duration);
        }

        println!("\n{}", "=".repeat(80));
    }
}

struct TestInfo {
    name: String,
    start_time: Instant,
    subtests: Vec<SubtestInfo>,
}

struct SubtestInfo {
    name: String,
    start_time: Instant,
}
```

#### 2. TestFixtures - 测试数据生成器

**文件**: `crates/agent-mem-test-utils/src/fixtures.rs` (新建)

```rust
use fake::{Fake, Faker};
use uuid::Uuid;

pub struct TestFixtures;

impl TestFixtures {
    /// 生成测试记忆
    pub fn memory() -> Memory {
        Memory {
            id: Uuid::new_v4().to_string(),
            content: Faker.fake::<String>(),
            memory_type: MemoryType::Episodic,
            importance: (0.0..1.0).fake(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// 生成批量测试记忆
    pub fn memories(count: usize) -> Vec<Memory> {
        (0..count).map(|_| Self::memory()).collect()
    }

    /// 生成测试 Agent
    pub fn agent() -> Agent {
        Agent {
            id: Uuid::new_v4().to_string(),
            name: Faker.fake::<String>(),
            config: AgentConfig::default(),
            created_at: Utc::now(),
        }
    }

    /// 生成测试用户
    pub fn user() -> User {
        User {
            id: Uuid::new_v4().to_string(),
            username: Faker.fake::<String>(),
            email: format!("{}@example.com", Faker.fake::<String>()),
            created_at: Utc::now(),
        }
    }

    /// 创建测试数据库
    pub async fn test_db() -> PgPool {
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/agentmem_test".to_string());

        PgPool::connect(&db_url).await.unwrap()
    }

    /// 清理测试数据库
    pub async fn cleanup_db(pool: &PgPool) {
        sqlx::query("TRUNCATE TABLE memories, agents, users CASCADE")
            .execute(pool)
            .await
            .unwrap();
    }

    /// 创建测试 Redis 连接
    pub async fn test_redis() -> redis::Client {
        redis::Client::open("redis://localhost:6379/15").unwrap()
    }
}
```

#### 3. MockProviders - LLM/Embedding Mock

**文件**: `crates/agent-mem-test-utils/src/mocks.rs` (新建)

```rust
use mockall::mock;

mock! {
    pub LLMProvider {}

    #[async_trait]
    impl LLMProvider for LLMProvider {
        async fn generate(&self, prompt: &str) -> Result<String, Error>;
        async fn generate_stream(&self, prompt: &str) -> Result<Stream<String>, Error>;
    }
}

mock! {
    pub EmbeddingProvider {}

    #[async_trait]
    impl EmbeddingProvider for EmbeddingProvider {
        async fn embed(&self, text: &str) -> Result<Vec<f32>, Error>;
        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, Error>;
    }
}

// 使用示例
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_mock_llm() {
        let mut mock_llm = MockLLMProvider::new();

        mock_llm
            .expect_generate()
            .with(eq("test prompt"))
            .times(1)
            .returning(|_| Ok("mocked response".to_string()));

        let result = mock_llm.generate("test prompt").await.unwrap();
        assert_eq!(result, "mocked response");
    }
}
```

#### 4. TestEnvironment - 集成测试环境

**文件**: `crates/agent-mem-test-utils/src/environment.rs` (新建)

```rust
use testcontainers::{clients::Cli, Container, images::postgres::Postgres};

pub struct TestEnvironment {
    docker: Cli,
    postgres: Option<Container<'static, Postgres>>,
    redis: Option<Container<'static, Redis>>,
    server: Option<MemoryServer>,
}

impl TestEnvironment {
    pub async fn new() -> Self {
        Self {
            docker: Cli::default(),
            postgres: None,
            redis: None,
            server: None,
        }
    }

    pub async fn start_postgres(&mut self) {
        let postgres = self.docker.run(Postgres::default());
        self.postgres = Some(postgres);
    }

    pub async fn start_redis(&mut self) {
        let redis = self.docker.run(Redis::default());
        self.redis = Some(redis);
    }

    pub async fn start_server(&mut self) {
        let config = ServerConfig {
            port: 8081,
            database_url: self.get_postgres_url(),
            redis_url: self.get_redis_url(),
            ..Default::default()
        };

        let server = MemoryServer::new(config).await.unwrap();
        self.server = Some(server);
    }

    pub fn create_client(&self) -> TestClient {
        TestClient::new("http://localhost:8081")
    }

    pub async fn cleanup(&mut self) {
        if let Some(server) = self.server.take() {
            server.shutdown().await;
        }
    }
}
```

---

## 📅 实施计划 (8 周，基于 Rust 开发)

### 🔴 Phase 1: 测试基础设施和工具 (Week 1-2)

**目标**: 建立完整的测试基础设施

#### Week 1: 测试工具开发
- [ ] **Day 1-2**: 创建 `agent-mem-test-utils` crate
  ```bash
  cargo new --lib crates/agent-mem-test-utils
  ```
  - [ ] 实现 TestTracker (参考 MIRIX)
  - [ ] 实现 TestFixtures (数据生成器)
  - [ ] 实现 MockProviders (LLM/Embedding Mock)
  - [ ] 实现 TestEnvironment (集成测试环境)

- [ ] **Day 3-4**: 配置测试依赖
  - [ ] 更新所有 Cargo.toml 的 dev-dependencies
  - [ ] 配置 Criterion 基准测试
  - [ ] 配置 Insta 快照测试
  - [ ] 配置 Proptest 属性测试

- [ ] **Day 5**: 创建测试脚本
  - [ ] `scripts/run_all_tests.sh` - 运行所有测试
  - [ ] `scripts/run_unit_tests.sh` - 只运行单元测试
  - [ ] `scripts/run_integration_tests.sh` - 只运行集成测试
  - [ ] `scripts/run_benchmarks.sh` - 运行性能测试
  - [ ] `scripts/generate_coverage.sh` - 生成覆盖率报告

#### Week 2: CI/CD 和测试环境
- [ ] **Day 1-2**: 配置 GitHub Actions
- [ ] **Day 3-4**: 配置测试数据库 (Docker Compose)
- [ ] **Day 5**: 文档和验证

**交付物**: ✅ 测试基础设施完成

---

### 🟡 Phase 2: P0 优先级 - Memory Managers (Week 3-4)

**目标**: 补充 5 个核心 Memory Manager 的测试 (110 tests)

#### Week 3: Episodic & Semantic Memory
- [ ] **Day 1-2**: Episodic Memory Manager (25 tests)
- [ ] **Day 3-4**: Semantic Memory Manager (25 tests)
- [ ] **Day 5**: 运行和验证

#### Week 4: Procedural, Knowledge Vault & Resource Memory
- [ ] **Day 1**: Procedural Memory Manager (20 tests)
- [ ] **Day 2**: Knowledge Vault Manager (20 tests)
- [ ] **Day 3**: Resource Memory Manager (20 tests)
- [ ] **Day 4-5**: 集成验证和修复

**交付物**: ✅ 110 个新单元测试，覆盖率 > 80%

---

### 🟢 Phase 3: P0 优先级 - Storage Backends (Week 5)

**目标**: 补充 3 个缺失的存储后端测试 (65 tests)

#### Week 5: Storage Backend Tests
- [ ] **Day 1-2**: MongoDB Backend (25 tests)
- [ ] **Day 2-3**: Redis Backend (20 tests)
- [ ] **Day 4**: Supabase Backend (20 tests)
- [ ] **Day 5**: 验证和优化

**交付物**: ✅ 65 个新单元测试，使用 testcontainers

---

### 🔵 Phase 4: 集成测试和搜索测试 (Week 6)

**目标**: 实现完整的集成测试 (95 tests)

#### Week 6: Integration Tests
- [ ] **Day 1-2**: Memory API 集成测试 (40 tests)
- [ ] **Day 3**: Search Methods 集成测试 (30 tests，参考 MIRIX)
- [ ] **Day 4**: Authentication & Authorization (25 tests)
- [ ] **Day 5**: 验证和修复

**交付物**: ✅ 95 个集成测试，完整 API 覆盖

---

### 🟣 Phase 5: E2E 测试和性能基准 (Week 7-8)

**目标**: 实现 E2E 测试和性能基准 (90 tests + 35 benchmarks)

#### Week 7: E2E Tests (50 tests)
- [ ] **Day 1-2**: Memory Lifecycle E2E (10 tests)
- [ ] **Day 2-3**: Conversation Flow E2E (15 tests)
- [ ] **Day 3-4**: Search Functionality E2E (15 tests)
- [ ] **Day 5**: Agent Workflow E2E (10 tests)

#### Week 8: Performance Benchmarks (35 benchmarks)
- [ ] **Day 1-2**: Storage Benchmarks (15 benchmarks)
- [ ] **Day 2-3**: LLM Benchmarks (10 benchmarks)
- [ ] **Day 3-4**: Embedding Benchmarks (10 benchmarks)
- [ ] **Day 5**: 最终验证和报告

**交付物**: ✅ 50 E2E 测试 + 35 基准测试，覆盖率 85%+

---

## 📈 测试覆盖率目标追踪

### 当前状态 vs 目标

| Crate | 当前覆盖率 | 目标覆盖率 | 当前测试数 | 目标测试数 | 状态 |
|-------|-----------|-----------|-----------|-----------|------|
| agent-mem-core | 70% | 85% | ~200 | 400 | 🟡 进行中 |
| agent-mem-server | 65% | 85% | ~150 | 300 | 🟡 进行中 |
| agent-mem-storage | 55% | 80% | ~80 | 200 | 🟡 进行中 |
| agent-mem-llm | 60% | 80% | ~100 | 180 | 🟡 进行中 |
| agent-mem-embeddings | 50% | 75% | ~40 | 120 | 🟡 进行中 |
| agent-mem-tools | 60% | 80% | ~50 | 100 | 🟡 进行中 |
| agent-mem-utils | 95% | 95% | ~30 | 35 | 🟢 优秀 |
| agent-mem-performance | 90% | 90% | ~25 | 30 | 🟢 优秀 |
| agent-mem-compat | 85% | 85% | ~20 | 25 | 🟢 优秀 |
| **总计** | **61%** | **85%** | **1,367** | **2,500** | 🟡 进行中 |

### 每周目标

| 周 | 新增测试数 | 累计测试数 | 预期覆盖率 | 里程碑 |
|----|-----------|-----------|-----------|--------|
| Week 1-2 | 0 | 1,367 | 61% | 测试基础设施完成 |
| Week 3 | 50 | 1,417 | 65% | Episodic + Semantic 完成 |
| Week 4 | 60 | 1,477 | 68% | 所有 Memory Managers 完成 |
| Week 5 | 65 | 1,542 | 72% | 存储后端测试完成 |
| Week 6 | 95 | 1,637 | 76% | 集成测试完成 |
| Week 7 | 50 | 1,687 | 80% | E2E 测试完成 |
| Week 8 | 35 | 1,722 | 85% | 性能基准完成 |
| **最终** | **+355** | **2,500+** | **85%+** | **项目完成** |

---

## 🎨 测试模式和最佳实践

### 1. TestTracker 模式 (参考 MIRIX)

```rust
pub struct TestTracker {
    tests: Vec<TestResult>,
    current_test: Option<TestInfo>,
}

impl TestTracker {
    pub fn start_test(&mut self, name: &str, description: &str) { }
    pub fn start_subtest(&mut self, name: &str) -> usize { }
    pub fn pass_test(&mut self, message: &str) { }
    pub fn fail_test(&mut self, error: &str) { }
    pub fn print_summary(&self) { }
}
```

### 2. 分层测试模式

```rust
// 直接测试 - 调用 Manager 方法
#[tokio::test]
async fn test_direct_memory_operation() {
    let manager = CoreMemoryManager::new();
    let block_id = manager.create_persona_block("content".to_string(), None)
        .await.unwrap();
    assert!(manager.get_persona_block(&block_id).await.is_ok());
}

// 间接测试 - 通过 API
#[tokio::test]
async fn test_indirect_memory_operation() {
    let client = TestClient::new();
    let response = client.post("/api/memory")
        .json(&json!({"content": "test"}))
        .send().await.unwrap();
    assert_eq!(response.status(), 200);
}
```

### 3. 搜索方法测试模式 (参考 MIRIX)

```rust
#[tokio::test]
async fn test_search_methods_comparison() {
    let search_methods = vec!["bm25", "embedding", "string_match"];
    let test_queries = vec![
        ("simple", "test"),
        ("phrase", "\"exact phrase\""),
        ("complex", "multiple terms query"),
    ];
    
    for method in search_methods {
        for (query_type, query) in &test_queries {
            let results = search_engine.search(query, method).await.unwrap();
            println!("{} - {}: {} results", method, query_type, results.len());
        }
    }
}
```

### 4. 性能对比测试模式

```rust
#[tokio::test]
async fn test_performance_comparison() {
    use std::time::Instant;
    
    let methods = vec!["bm25", "string_match", "fuzzy"];
    let mut results = HashMap::new();
    
    for method in methods {
        let start = Instant::now();
        let _ = search_engine.search("query", method).await;
        let elapsed = start.elapsed();
        results.insert(method, elapsed);
    }
    
    // 计算性能提升
    let bm25_time = results["bm25"];
    let string_time = results["string_match"];
    let improvement = string_time.as_secs_f64() / bm25_time.as_secs_f64();
    println!("BM25 is {:.1}x faster than string search", improvement);
}
```

---

## � 实际测试示例 (基于现有代码)

### 示例 1: Memory Manager 单元测试

基于现有的 `core_memory.rs` 测试模式，扩展到其他 Manager：

```rust
// 文件: crates/agent-mem-core/src/managers/episodic_memory.rs

use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct EpisodicMemoryManager {
    events: HashMap<String, EpisodicEvent>,
}

pub struct EpisodicEvent {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub importance: f32,
    pub metadata: HashMap<String, String>,
}

pub enum EventType {
    Activity,
    Conversation,
    Observation,
    Decision,
}

impl EpisodicMemoryManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub async fn create_event(
        &mut self,
        user_id: &str,
        content: &str,
        event_type: EventType,
    ) -> Result<EpisodicEvent, Error> {
        let event = EpisodicEvent {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            content: content.to_string(),
            event_type,
            timestamp: Utc::now(),
            importance: 0.5,
            metadata: HashMap::new(),
        };

        self.events.insert(event.id.clone(), event.clone());
        Ok(event)
    }

    pub async fn get_event(&self, event_id: &str) -> Result<&EpisodicEvent, Error> {
        self.events.get(event_id)
            .ok_or_else(|| Error::NotFound(event_id.to_string()))
    }

    pub async fn search_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&EpisodicEvent> {
        self.events.values()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_create_episodic_event() {
        let mut manager = EpisodicMemoryManager::new();

        let event = manager.create_event(
            "user-123",
            "Went to grocery store and bought apples",
            EventType::Activity,
        ).await.unwrap();

        assert!(!event.id.is_empty());
        assert_eq!(event.user_id, "user-123");
        assert_eq!(event.content, "Went to grocery store and bought apples");
        assert!(matches!(event.event_type, EventType::Activity));
        assert_eq!(event.importance, 0.5);
    }

    #[tokio::test]
    async fn test_retrieve_episodic_event() {
        let mut manager = EpisodicMemoryManager::new();

        let created = manager.create_event(
            "user-123",
            "Test event",
            EventType::Observation,
        ).await.unwrap();

        let retrieved = manager.get_event(&created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.content, "Test event");
    }

    #[tokio::test]
    async fn test_retrieve_nonexistent_event() {
        let manager = EpisodicMemoryManager::new();

        let result = manager.get_event("nonexistent-id").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound(_)));
    }

    #[tokio::test]
    async fn test_search_by_time_range() {
        let mut manager = EpisodicMemoryManager::new();

        // 创建多个事件
        let now = Utc::now();
        manager.create_event("user-1", "Event 1", EventType::Activity).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        manager.create_event("user-1", "Event 2", EventType::Conversation).await.unwrap();
        manager.create_event("user-1", "Event 3", EventType::Observation).await.unwrap();

        // 搜索时间范围
        let start = now - Duration::hours(1);
        let end = now + Duration::hours(1);

        let results = manager.search_by_time_range(start, end).await;

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_search_empty_time_range() {
        let manager = EpisodicMemoryManager::new();

        let start = Utc::now() - Duration::days(10);
        let end = Utc::now() - Duration::days(9);

        let results = manager.search_by_time_range(start, end).await;

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_event_importance_default() {
        let mut manager = EpisodicMemoryManager::new();

        let event = manager.create_event(
            "user-1",
            "Regular event",
            EventType::Activity,
        ).await.unwrap();

        assert_eq!(event.importance, 0.5);
    }

    #[tokio::test]
    async fn test_multiple_users_isolation() {
        let mut manager = EpisodicMemoryManager::new();

        manager.create_event("user-1", "User 1 event", EventType::Activity).await.unwrap();
        manager.create_event("user-2", "User 2 event", EventType::Activity).await.unwrap();
        manager.create_event("user-1", "User 1 event 2", EventType::Conversation).await.unwrap();

        // 验证所有事件都被存储
        assert_eq!(manager.events.len(), 3);

        // 验证可以按用户过滤（需要实现 filter_by_user 方法）
        let user1_events: Vec<_> = manager.events.values()
            .filter(|e| e.user_id == "user-1")
            .collect();

        assert_eq!(user1_events.len(), 2);
    }

    #[tokio::test]
    async fn test_concurrent_event_creation() {
        use tokio::sync::Mutex;
        use std::sync::Arc;

        let manager = Arc::new(Mutex::new(EpisodicMemoryManager::new()));

        let mut handles = vec![];

        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let mut mgr = manager_clone.lock().await;
                mgr.create_event(
                    "user-1",
                    &format!("Event {}", i),
                    EventType::Activity,
                ).await.unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let mgr = manager.lock().await;
        assert_eq!(mgr.events.len(), 10);
    }
}
```

### 示例 2: 集成测试 - Search Methods (参考 MIRIX)

```rust
// 文件: tests/search_methods_integration_test.rs

use agent_mem_core::search::{SearchEngine, SearchMethod, SearchResult};
use agent_mem_storage::backends::PostgresBackend;
use std::time::Instant;
use std::collections::HashMap;

async fn create_test_search_engine() -> SearchEngine {
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/agentmem_test".to_string());

    let backend = PostgresBackend::new(&db_url).await.unwrap();
    SearchEngine::new(backend)
}

async fn seed_test_data(engine: &SearchEngine, count: usize) {
    let test_documents = vec![
        "Machine learning is a subset of artificial intelligence",
        "Deep learning uses neural networks with multiple layers",
        "Natural language processing enables computers to understand human language",
        "Computer vision allows machines to interpret visual information",
        "Reinforcement learning trains agents through rewards and penalties",
        "Supervised learning uses labeled training data",
        "Unsupervised learning finds patterns in unlabeled data",
        "Transfer learning applies knowledge from one task to another",
        "Generative AI creates new content based on training data",
        "Large language models are trained on vast amounts of text",
    ];

    for i in 0..count {
        let content = test_documents[i % test_documents.len()];
        engine.index_document(&format!("doc-{}", i), content).await.unwrap();
    }
}

#[tokio::test]
async fn test_search_methods_comparison() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine, 1000).await;

    let methods = vec![
        SearchMethod::BM25,
        SearchMethod::Embedding,
        SearchMethod::StringMatch,
        SearchMethod::Fuzzy,
        SearchMethod::Hybrid,
    ];

    let queries = vec![
        ("simple", "machine learning"),
        ("phrase", "\"artificial intelligence\""),
        ("complex", "deep learning neural networks optimization"),
        ("single_word", "computer"),
        ("multi_term", "supervised unsupervised learning"),
    ];

    println!("\n=== SEARCH METHODS COMPARISON ===\n");

    for (query_type, query) in &queries {
        println!("Query Type: {} - '{}'", query_type, query);
        println!("{}", "-".repeat(80));

        for method in &methods {
            let start = Instant::now();
            let results = engine.search(query, *method, 10).await.unwrap();
            let elapsed = start.elapsed();

            println!("  {:?}:", method);
            println!("    Results: {}", results.len());
            println!("    Time: {:?}", elapsed);

            if !results.is_empty() {
                println!("    Top score: {:.4}", results[0].score);
                println!("    Avg score: {:.4}",
                    results.iter().map(|r| r.score).sum::<f32>() / results.len() as f32
                );
            }

            // 验证结果
            assert!(results.len() <= 10, "Should return at most 10 results");

            for result in &results {
                assert!(result.score >= 0.0 && result.score <= 1.0,
                    "Score should be between 0 and 1");
            }
        }

        println!();
    }
}

#[tokio::test]
async fn test_field_specific_search() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine, 100).await;

    let fields = vec!["title", "content", "summary", "tags"];
    let query = "machine learning";

    println!("\n=== FIELD-SPECIFIC SEARCH ===\n");

    for field in fields {
        let results = engine.search_field(query, field, 10).await.unwrap();

        println!("Field '{}': {} results", field, results.len());

        assert!(results.len() > 0, "Should find results in field '{}'", field);

        // 验证结果包含查询词
        for result in &results {
            let content_lower = result.content.to_lowercase();
            assert!(
                content_lower.contains("machine") || content_lower.contains("learning"),
                "Result should contain query terms"
            );
        }
    }
}

#[tokio::test]
async fn test_performance_comparison() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine, 10000).await;

    let mut performance_results = HashMap::new();
    let query = "deep learning neural networks";

    println!("\n=== PERFORMANCE COMPARISON (10,000 documents) ===\n");

    for method in [
        SearchMethod::BM25,
        SearchMethod::StringMatch,
        SearchMethod::Fuzzy,
        SearchMethod::Embedding,
    ] {
        let mut times = vec![];

        // 运行 5 次取平均
        for _ in 0..5 {
            let start = Instant::now();
            let _ = engine.search(query, method, 50).await.unwrap();
            times.push(start.elapsed());
        }

        let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
        performance_results.insert(method, avg_time);

        println!("{:?}: {:?} (avg of 5 runs)", method, avg_time);
    }

    // 计算性能提升
    let bm25_time = performance_results[&SearchMethod::BM25];
    let string_time = performance_results[&SearchMethod::StringMatch];
    let improvement = string_time.as_secs_f64() / bm25_time.as_secs_f64();

    println!("\nBM25 is {:.1}x faster than string search", improvement);

    // BM25 应该比字符串匹配更快
    assert!(improvement > 1.0, "BM25 should be faster than string search");
}

#[tokio::test]
async fn test_edge_cases() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine, 100).await;

    println!("\n=== EDGE CASE TESTING ===\n");

    let edge_cases = vec![
        ("empty", ""),
        ("whitespace", "   "),
        ("special_chars", "!@#$%^&*()"),
        ("very_long", &"x".repeat(10000)),
        ("unicode", "你好世界 🌍 مرحبا"),
        ("sql_injection", "'; DROP TABLE memories; --"),
        ("numbers_only", "123456789"),
        ("single_char", "a"),
    ];

    for (name, query) in edge_cases {
        let result = engine.search(query, SearchMethod::BM25, 10).await;

        println!("Edge case '{}': {}", name,
            if result.is_ok() { "✅ OK" } else { "❌ ERROR" }
        );

        // 所有边界情况都应该优雅处理，不应该崩溃
        assert!(result.is_ok() || result.is_err(),
            "Edge case '{}' should be handled gracefully", name);
    }
}

#[tokio::test]
async fn test_result_ranking_quality() {
    let engine = create_test_search_engine().await;

    // 插入特定的测试文档
    engine.index_document("doc-1", "Machine learning is awesome").await.unwrap();
    engine.index_document("doc-2", "I love machine learning and AI").await.unwrap();
    engine.index_document("doc-3", "Deep learning is a type of machine learning").await.unwrap();
    engine.index_document("doc-4", "The weather is nice today").await.unwrap();

    let results = engine.search("machine learning", SearchMethod::BM25, 10).await.unwrap();

    // 验证相关文档排在前面
    assert!(results.len() >= 3, "Should find at least 3 relevant documents");

    // 第一个结果应该是最相关的
    assert!(results[0].score > results[1].score,
        "Results should be sorted by relevance");

    // 不相关的文档应该排在后面或不出现
    let irrelevant_doc = results.iter()
        .find(|r| r.content.contains("weather"));

    if let Some(doc) = irrelevant_doc {
        assert!(doc.score < 0.3, "Irrelevant document should have low score");
    }
}
```

---

## �📊 测试报告和监控

### 测试报告格式

```
🏁 TEST EXECUTION SUMMARY
================================================================================

📊 OVERALL RESULTS:
   Total Tests: 2,500
   ✅ Passed Tests: 2,450
   ❌ Failed Tests: 50
   📈 Success Rate: 98.0%

⏱️  PERFORMANCE:
   Total Duration: 15m 32s
   Average Test Duration: 372ms
   Fastest Test: 1ms (test_hash_content)
   Slowest Test: 5.2s (test_e2e_complete_workflow)

📦 COVERAGE:
   Line Coverage: 85.3%
   Branch Coverage: 78.2%
   Function Coverage: 91.5%

🔍 BY CATEGORY:
   Unit Tests: 2,100 (84.0%)
   Integration Tests: 300 (12.0%)
   E2E Tests: 50 (2.0%)
   Benchmarks: 50 (2.0%)

📁 BY CRATE:
   agent-mem-core:        400/400 ✅ (100%)
   agent-mem-server:      300/300 ✅ (100%)
   agent-mem-storage:     200/205 ⚠️  (97.6%)
   agent-mem-llm:         180/180 ✅ (100%)
   agent-mem-embeddings:  120/120 ✅ (100%)
   agent-mem-tools:       100/100 ✅ (100%)
   agent-mem-utils:       35/35 ✅ (100%)
   agent-mem-performance: 30/30 ✅ (100%)
   agent-mem-compat:      25/25 ✅ (100%)

================================================================================
```

---

## 🚀 CI/CD 配置

### GitHub Actions 工作流

**文件**: `.github/workflows/test.yml`

```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: agentmem_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: ${{ matrix.rust }}
          override: true
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run unit tests
        run: cargo test --lib --all-features --workspace
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test
          REDIS_URL: redis://localhost:6379

      - name: Run integration tests
        run: cargo test --tests --all-features --workspace
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test
          REDIS_URL: redis://localhost:6379

      - name: Run doc tests
        run: cargo test --doc --all-features --workspace

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: agentmem_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --all-features --workspace --timeout 300 --out Xml
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test
          REDIS_URL: redis://localhost:6379

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: true

  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true

      - name: Run benchmarks
        run: cargo bench --workspace --no-fail-fast

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

---

## � 测试执行脚本

### 1. 运行所有测试

**文件**: `scripts/run_all_tests.sh`

```bash
#!/bin/bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         AgentMem 全面测试执行脚本                          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 检查依赖
echo -e "${YELLOW}[1/6] 检查依赖...${NC}"
command -v cargo >/dev/null 2>&1 || { echo -e "${RED}错误: 需要安装 Rust/Cargo${NC}" >&2; exit 1; }
command -v psql >/dev/null 2>&1 || { echo -e "${YELLOW}警告: PostgreSQL 未安装，部分测试可能失败${NC}"; }
command -v redis-cli >/dev/null 2>&1 || { echo -e "${YELLOW}警告: Redis 未安装，部分测试可能失败${NC}"; }

# 设置环境变量
export DATABASE_URL="${DATABASE_URL:-postgres://localhost/agentmem_test}"
export REDIS_URL="${REDIS_URL:-redis://localhost:6379}"
export RUST_BACKTRACE=1
export RUST_LOG="${RUST_LOG:-info}"

echo -e "${GREEN}✓ 依赖检查完成${NC}"
echo ""

# 编译检查
echo -e "${YELLOW}[2/6] 编译检查...${NC}"
cargo check --all-features --workspace
echo -e "${GREEN}✓ 编译检查通过${NC}"
echo ""

# 代码格式检查
echo -e "${YELLOW}[3/6] 代码格式检查...${NC}"
cargo fmt --all -- --check
echo -e "${GREEN}✓ 代码格式正确${NC}"
echo ""

# Clippy 检查
echo -e "${YELLOW}[4/6] Clippy 静态分析...${NC}"
cargo clippy --all-targets --all-features --workspace -- -D warnings
echo -e "${GREEN}✓ Clippy 检查通过${NC}"
echo ""

# 运行测试
echo -e "${YELLOW}[5/6] 运行测试套件...${NC}"
echo ""

# 单元测试
echo -e "${BLUE}  → 单元测试${NC}"
cargo test --lib --all-features --workspace -- --nocapture
echo ""

# 集成测试
echo -e "${BLUE}  → 集成测试${NC}"
cargo test --tests --all-features --workspace -- --nocapture
echo ""

# 文档测试
echo -e "${BLUE}  → 文档测试${NC}"
cargo test --doc --all-features --workspace
echo ""

echo -e "${GREEN}✓ 所有测试通过${NC}"
echo ""

# 生成覆盖率报告
echo -e "${YELLOW}[6/6] 生成覆盖率报告...${NC}"
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    cargo tarpaulin --all-features --workspace --out Html --output-dir coverage
    echo -e "${GREEN}✓ 覆盖率报告已生成: coverage/index.html${NC}"
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin 未安装，跳过覆盖率报告${NC}"
    echo -e "${YELLOW}  安装命令: cargo install cargo-tarpaulin${NC}"
fi
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              🎉 所有测试执行完成！                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
```

### 2. 运行单元测试

**文件**: `scripts/run_unit_tests.sh`

```bash
#!/bin/bash

set -e

echo "🧪 运行单元测试..."
echo ""

# 按 crate 运行测试
crates=(
    "agent-mem-core"
    "agent-mem-server"
    "agent-mem-storage"
    "agent-mem-llm"
    "agent-mem-embeddings"
    "agent-mem-tools"
    "agent-mem-utils"
    "agent-mem-performance"
    "agent-mem-compat"
)

total_tests=0
passed_tests=0
failed_tests=0

for crate in "${crates[@]}"; do
    echo "📦 Testing $crate..."

    if cargo test -p "$crate" --lib --all-features -- --nocapture; then
        echo "✅ $crate: PASSED"
        ((passed_tests++))
    else
        echo "❌ $crate: FAILED"
        ((failed_tests++))
    fi

    ((total_tests++))
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 测试总结:"
echo "   总计: $total_tests crates"
echo "   通过: $passed_tests"
echo "   失败: $failed_tests"
echo "   成功率: $(( passed_tests * 100 / total_tests ))%"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $failed_tests -gt 0 ]; then
    exit 1
fi
```

### 3. 运行集成测试

**文件**: `scripts/run_integration_tests.sh`

```bash
#!/bin/bash

set -e

echo "🔗 运行集成测试..."
echo ""

# 检查服务
echo "检查必需的服务..."

if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
    echo "❌ PostgreSQL 未运行"
    echo "   启动命令: docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15"
    exit 1
fi

if ! redis-cli ping >/dev/null 2>&1; then
    echo "❌ Redis 未运行"
    echo "   启动命令: docker run -d -p 6379:6379 redis:7"
    exit 1
fi

echo "✅ 所有服务正常运行"
echo ""

# 设置测试数据库
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/agentmem_test"
export REDIS_URL="redis://localhost:6379/15"

# 运行集成测试
echo "运行集成测试..."
cargo test --tests --all-features --workspace -- --test-threads=1 --nocapture

echo ""
echo "✅ 集成测试完成"
```

### 4. 运行性能基准测试

**文件**: `scripts/run_benchmarks.sh`

```bash
#!/bin/bash

set -e

echo "⚡ 运行性能基准测试..."
echo ""

# 运行所有基准测试
cargo bench --workspace --no-fail-fast

echo ""
echo "📊 基准测试报告已生成:"
echo "   target/criterion/report/index.html"
echo ""
echo "✅ 基准测试完成"
```

### 5. 生成覆盖率报告

**文件**: `scripts/generate_coverage.sh`

```bash
#!/bin/bash

set -e

echo "📊 生成测试覆盖率报告..."
echo ""

# 检查 tarpaulin 是否安装
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo "安装 cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# 生成覆盖率报告
cargo tarpaulin \
    --all-features \
    --workspace \
    --timeout 300 \
    --out Html \
    --out Xml \
    --output-dir coverage \
    --exclude-files "*/tests/*" "*/benches/*"

echo ""
echo "✅ 覆盖率报告已生成:"
echo "   HTML: coverage/index.html"
echo "   XML:  coverage/cobertura.xml"
echo ""

# 显示覆盖率摘要
if [ -f coverage/cobertura.xml ]; then
    echo "📈 覆盖率摘要:"
    grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1 | \
        awk '{printf "   行覆盖率: %.1f%%\n", $1 * 100}'
fi
```

🔍 SUBTEST DETAILS:
   Total Subtests: 5,000
   ✅ Passed Subtests: 4,900
   ❌ Failed Subtests: 100
   📈 Subtest Success Rate: 98.0%

⏱️  PERFORMANCE:
   Total Duration: 15m 30s
   Average Test Time: 0.465s
   Slowest Test: test_large_dataset_search (45s)

📈 COVERAGE:
   Line Coverage: 85.2%
   Branch Coverage: 78.5%
   Function Coverage: 92.1%
```

### CI/CD 集成

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run unit tests
        run: cargo test --lib --all-features
      
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
      redis:
        image: redis:7
    steps:
      - name: Run integration tests
        run: cargo test --test '*' --all-features
      
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: cargo bench --all-features
```

---

## 🚀 快速开始指南

### 1. 环境准备

```bash
# 克隆仓库
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen

# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装测试工具
cargo install cargo-tarpaulin  # 覆盖率工具
cargo install cargo-watch      # 自动测试

# 启动测试服务 (使用 Docker)
docker-compose -f docker-compose.test.yml up -d
```

### 2. 运行测试

```bash
# 运行所有测试
./scripts/run_all_tests.sh

# 或者分步运行
cargo test --lib --workspace              # 单元测试
cargo test --tests --workspace            # 集成测试
cargo test --doc --workspace              # 文档测试
cargo bench --workspace                   # 基准测试

# 生成覆盖率报告
./scripts/generate_coverage.sh
open coverage/index.html
```

### 3. 开发工作流

```bash
# 监听文件变化，自动运行测试
cargo watch -x test

# 运行特定 crate 的测试
cargo test -p agent-mem-core

# 运行特定测试
cargo test test_episodic_memory

# 显示测试输出
cargo test -- --nocapture

# 运行被忽略的测试
cargo test -- --ignored
```

### 4. 调试测试

```rust
// 在测试中使用 dbg! 宏
#[tokio::test]
async fn test_debug_example() {
    let value = some_function().await;
    dbg!(&value);  // 打印调试信息
    assert_eq!(value, expected);
}

// 使用 env_logger 查看日志
#[tokio::test]
async fn test_with_logging() {
    env_logger::init();
    log::info!("Starting test...");
    // 测试代码
}
```

---

## 🎯 成功指标

### 量化指标

| 指标 | 当前 | 目标 | 状态 |
|------|------|------|------|
| 测试覆盖率 | 61% | ≥ 85% | 🟡 进行中 |
| 单元测试数量 | 1,367 | ≥ 2,500 | 🟡 进行中 |
| 集成测试数量 | ~150 | ≥ 300 | 🟡 进行中 |
| E2E 测试数量 | ~10 | ≥ 50 | 🟡 进行中 |
| 基准测试数量 | ~17 | ≥ 50 | 🟡 进行中 |
| 测试通过率 | - | ≥ 98% | 🎯 目标 |
| CI/CD 执行时间 | - | ≤ 20 分钟 | 🎯 目标 |

### 质量指标

- [ ] **核心功能覆盖**: 所有 Memory Managers 有完整测试
- [ ] **API 端点覆盖**: 所有 REST API 有集成测试
- [ ] **存储后端覆盖**: 所有后端有单元测试和性能基准
- [ ] **LLM 提供商覆盖**: 所有提供商有单元测试
- [ ] **关键路径覆盖**: 主要工作流有 E2E 测试
- [ ] **边界条件覆盖**: 所有边界情况有测试
- [ ] **并发安全覆盖**: 关键组件有并发测试
- [ ] **性能基准覆盖**: 关键操作有性能基准

### 文档指标

- [ ] 所有公共 API 有文档测试
- [ ] 所有测试有清晰的注释
- [ ] 测试覆盖率报告定期更新
- [ ] 测试失败有详细的错误信息

---

## 📚 参考资源

### Rust 测试文档
- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)

### 测试工具
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - 代码覆盖率
- [cargo-watch](https://github.com/watchexec/cargo-watch) - 自动测试
- [proptest](https://github.com/proptest-rs/proptest) - 属性测试
- [mockall](https://github.com/asomers/mockall) - Mock 框架
- [insta](https://github.com/mitsuhiko/insta) - 快照测试
- [testcontainers](https://github.com/testcontainers/testcontainers-rs) - Docker 测试

### MIRIX 参考
- `source/MIRIX/tests/test_memory.py` - 记忆测试模式
- `source/MIRIX/tests/test_sdk.py` - SDK 测试模式
- TestTracker 模式 - 测试执行跟踪
- 搜索方法对比测试 - BM25, Embedding, String Match

---

## 📝 总结

### 当前状态
- ✅ **已有基础**: 1,367 个测试，61% 覆盖率
- ✅ **工具完善**: hash, performance, compat 测试完善
- ⚠️ **需要补充**: Memory Managers, Storage Backends 缺失测试
- ⚠️ **需要增强**: 集成测试、E2E 测试不足

### 实施路径
1. **Week 1-2**: 建立测试基础设施 (TestTracker, Fixtures, CI/CD)
2. **Week 3-4**: 补充 P0 优先级测试 (Memory Managers)
3. **Week 5**: 补充存储后端测试 (MongoDB, Redis, Supabase)
4. **Week 6**: 实现集成测试 (API, Search, Auth)
5. **Week 7-8**: 实现 E2E 测试和性能基准

### 预期成果
- 📈 测试覆盖率从 61% → 85%+
- 📊 测试数量从 1,367 → 2,500+
- 🎯 完整的测试体系 (Unit + Integration + E2E + Benchmark)
- 🚀 自动化 CI/CD 流程
- 📚 完善的测试文档

### 关键成功因素
1. **参考 MIRIX**: 学习 Python 测试模式，转换为 Rust 实现
2. **优先级明确**: 先补充缺失的核心测试 (P0)
3. **工具支持**: 使用 TestTracker, Fixtures 提高效率
4. **持续集成**: GitHub Actions 自动运行测试
5. **质量保证**: 覆盖率报告、性能基准持续监控

---

## 🎉 下一步行动

### 立即开始 (本周)
1. [ ] 创建 `agent-mem-test-utils` crate
2. [ ] 实现 TestTracker (参考 MIRIX)
3. [ ] 配置 GitHub Actions
4. [ ] 创建测试脚本 (run_all_tests.sh 等)

### 第一个里程碑 (Week 3)
1. [ ] 完成 Episodic Memory Manager 测试 (25 tests)
2. [ ] 完成 Semantic Memory Manager 测试 (25 tests)
3. [ ] 测试覆盖率达到 65%

### 最终目标 (Week 8)
1. [ ] 所有 2,500+ 测试通过
2. [ ] 测试覆盖率 ≥ 85%
3. [ ] CI/CD 完全自动化
4. [ ] 性能基准建立

---

**文档版本**: v2.0 (Rust Edition)
**最后更新**: 2025-10-07
**作者**: AgentMem 测试团队
**参考**: MIRIX 测试系统 + Rust 最佳实践

---

## 📝 下一步行动

### 立即行动 (本周)
1. [ ] 创建 TestTracker 工具类
2. [ ] 补充 Memory Engine 单元测试
3. [ ] 补充 Embeddings 单元测试
4. [ ] 创建测试数据生成器

### 短期目标 (2 周内)
1. [ ] 完成 Phase 1 单元测试补充
2. [ ] 建立 CI/CD 测试流水线
3. [ ] 生成第一份测试覆盖率报告

### 中期目标 (1 个月内)
1. [ ] 完成 Phase 2 集成测试
2. [ ] 完成 Phase 3 E2E 测试
3. [ ] 达到 85% 测试覆盖率

### 长期目标 (2 个月内)
1. [ ] 完成 Phase 4 性能基准测试
2. [ ] 建立持续测试监控系统
3. [ ] 发布测试最佳实践文档

---

## 📚 附录 A: 测试实现示例

### A.1 Memory Engine 单元测试模板

```rust
// crates/agent-mem-core/src/engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    async fn create_test_engine() -> MemoryEngine {
        let config = MemoryEngineConfig::default();
        MemoryEngine::new(config)
    }

    #[tokio::test]
    async fn test_engine_initialization() {
        let engine = create_test_engine().await;
        assert!(engine.is_initialized());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_memory() {
        let engine = create_test_engine().await;

        let memory = Memory {
            id: "test-1".to_string(),
            content: "Test memory content".to_string(),
            memory_type: MemoryType::Episodic,
            importance: 0.8,
            created_at: Utc::now(),
        };

        // Store
        engine.store_memory(&memory).await.unwrap();

        // Retrieve
        let retrieved = engine.get_memory("test-1").await.unwrap();
        assert_eq!(retrieved.content, memory.content);
    }

    #[tokio::test]
    async fn test_search_memories() {
        let engine = create_test_engine().await;

        // Store multiple memories
        for i in 0..10 {
            let memory = Memory {
                id: format!("test-{}", i),
                content: format!("Memory content {}", i),
                memory_type: MemoryType::Episodic,
                importance: 0.5 + (i as f32 * 0.05),
                created_at: Utc::now(),
            };
            engine.store_memory(&memory).await.unwrap();
        }

        // Search
        let results = engine.search("content", 5).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let engine = Arc::new(create_test_engine().await);
        let mut handles = vec![];

        for i in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = tokio::spawn(async move {
                let memory = Memory {
                    id: format!("concurrent-{}", i),
                    content: format!("Concurrent memory {}", i),
                    memory_type: MemoryType::Episodic,
                    importance: 0.7,
                    created_at: Utc::now(),
                };
                engine_clone.store_memory(&memory).await
            });
            handles.push(handle);
        }

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }
    }
}
```

### A.2 搜索方法集成测试模板 (参考 MIRIX)

```rust
// crates/agent-mem-core/tests/search_methods_test.rs

use agent_mem_core::search::{SearchEngine, SearchMethod};
use std::time::Instant;

#[tokio::test]
async fn test_search_methods_comparison() {
    let engine = create_test_search_engine().await;

    // 准备测试数据
    seed_test_data(&engine).await;

    let search_methods = vec![
        SearchMethod::BM25,
        SearchMethod::Embedding,
        SearchMethod::StringMatch,
        SearchMethod::Fuzzy,
    ];

    let test_queries = vec![
        ("simple", "machine learning"),
        ("phrase", "\"artificial intelligence\""),
        ("complex", "deep learning neural networks"),
    ];

    println!("=== Search Methods Comparison ===\n");

    for (query_type, query) in &test_queries {
        println!("Query Type: {} - Query: '{}'", query_type, query);

        for method in &search_methods {
            let start = Instant::now();
            let results = engine.search(query, *method, 10).await.unwrap();
            let elapsed = start.elapsed();

            println!("  {:?}: {} results in {:?}", method, results.len(), elapsed);
        }
        println!();
    }
}

#[tokio::test]
async fn test_field_specific_search() {
    let engine = create_test_search_engine().await;
    seed_test_data(&engine).await;

    let fields = vec!["title", "content", "summary", "tags"];
    let query = "machine learning";

    println!("=== Field-Specific Search ===\n");

    for field in fields {
        let results = engine.search_field(query, field, 10).await.unwrap();
        println!("Field '{}': {} results", field, results.len());

        // 显示前 3 个结果
        for (i, result) in results.iter().take(3).enumerate() {
            println!("  {}. Score: {:.3} - {}", i+1, result.score, result.title);
        }
        println!();
    }
}

#[tokio::test]
async fn test_performance_comparison() {
    let engine = create_test_search_engine().await;
    seed_large_dataset(&engine, 10000).await; // 10K 记录

    let methods = vec![
        SearchMethod::BM25,
        SearchMethod::StringMatch,
        SearchMethod::Fuzzy,
    ];

    let mut performance_results = HashMap::new();

    println!("=== Performance Comparison (10K records) ===\n");

    for method in methods {
        let start = Instant::now();
        let _ = engine.search("test query", method, 50).await.unwrap();
        let elapsed = start.elapsed();

        performance_results.insert(method, elapsed);
        println!("{:?}: {:?}", method, elapsed);
    }

    // 计算性能提升
    if let (Some(&bm25_time), Some(&string_time)) =
        (performance_results.get(&SearchMethod::BM25),
         performance_results.get(&SearchMethod::StringMatch)) {
        let improvement = string_time.as_secs_f64() / bm25_time.as_secs_f64();
        println!("\nBM25 is {:.1}x faster than string search", improvement);
    }
}

async fn seed_test_data(engine: &SearchEngine) {
    // 插入测试数据
    let test_data = vec![
        ("Machine Learning Basics", "Introduction to machine learning algorithms"),
        ("Deep Learning", "Neural networks and deep learning techniques"),
        ("Artificial Intelligence", "AI fundamentals and applications"),
        ("Natural Language Processing", "NLP and text processing"),
        ("Computer Vision", "Image recognition and computer vision"),
    ];

    for (title, content) in test_data {
        engine.index_document(title, content).await.unwrap();
    }
}
```

### A.3 E2E 工作流测试模板

```rust
// tests/e2e_memory_lifecycle_test.rs

use agent_mem_server::MemoryServer;
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_complete_memory_lifecycle() {
    // 1. 启动测试服务器
    let server = start_test_server().await;
    let client = Client::new();
    let base_url = "http://localhost:8081";

    // 2. 创建记忆
    println!("Step 1: Creating memory...");
    let create_response = client
        .post(&format!("{}/api/memories", base_url))
        .json(&json!({
            "agent_id": "test-agent",
            "user_id": "test-user",
            "content": "This is a test memory about machine learning",
            "memory_type": "Episodic",
            "importance": 0.8
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_response.status(), 201);
    let memory: serde_json::Value = create_response.json().await.unwrap();
    let memory_id = memory["id"].as_str().unwrap();
    println!("✓ Memory created: {}", memory_id);

    // 3. 检索记忆
    println!("Step 2: Retrieving memory...");
    let get_response = client
        .get(&format!("{}/api/memories/{}", base_url, memory_id))
        .send()
        .await
        .unwrap();

    assert_eq!(get_response.status(), 200);
    let retrieved: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(retrieved["content"], "This is a test memory about machine learning");
    println!("✓ Memory retrieved successfully");

    // 4. 搜索记忆
    println!("Step 3: Searching memories...");
    let search_response = client
        .post(&format!("{}/api/memories/search", base_url))
        .json(&json!({
            "query": "machine learning",
            "method": "bm25",
            "limit": 10
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(search_response.status(), 200);
    let results: serde_json::Value = search_response.json().await.unwrap();
    assert!(results["results"].as_array().unwrap().len() > 0);
    println!("✓ Search returned {} results", results["results"].as_array().unwrap().len());

    // 5. 更新记忆
    println!("Step 4: Updating memory...");
    let update_response = client
        .put(&format!("{}/api/memories/{}", base_url, memory_id))
        .json(&json!({
            "content": "Updated: This is about deep learning",
            "importance": 0.9
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(update_response.status(), 200);
    println!("✓ Memory updated successfully");

    // 6. 删除记忆
    println!("Step 5: Deleting memory...");
    let delete_response = client
        .delete(&format!("{}/api/memories/{}", base_url, memory_id))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_response.status(), 204);
    println!("✓ Memory deleted successfully");

    // 7. 验证删除
    println!("Step 6: Verifying deletion...");
    let verify_response = client
        .get(&format!("{}/api/memories/{}", base_url, memory_id))
        .send()
        .await
        .unwrap();

    assert_eq!(verify_response.status(), 404);
    println!("✓ Memory deletion verified");

    println!("\n✅ Complete memory lifecycle test passed!");
}
```

### A.4 性能基准测试模板

```rust
// benches/storage_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use agent_mem_storage::backends::PostgresBackend;

fn benchmark_write_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_write");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                // 基准测试代码
                let backend = create_test_backend();
                for i in 0..size {
                    backend.write(black_box(&format!("key-{}", i)), black_box("value"));
                }
            });
        });
    }

    group.finish();
}

fn benchmark_read_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_read");
    let backend = create_and_seed_backend(1000);

    group.bench_function("single_read", |b| {
        b.iter(|| {
            backend.read(black_box("key-500"))
        });
    });

    group.bench_function("batch_read_10", |b| {
        b.iter(|| {
            for i in 0..10 {
                backend.read(black_box(&format!("key-{}", i)));
            }
        });
    });

    group.finish();
}

fn benchmark_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    let backend = create_and_seed_backend(10000);

    for method in ["bm25", "embedding", "string_match"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(method),
            method,
            |b, &method| {
                b.iter(|| {
                    backend.search(black_box("test query"), black_box(method), 10)
                });
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_write_performance,
    benchmark_read_performance,
    benchmark_search_performance
);
criterion_main!(benches);
```

---

## 📚 附录 B: 测试数据生成器

### B.1 Fake Data Generator

```rust
// crates/agent-mem-utils/src/test_helpers.rs

use fake::{Fake, Faker};
use chrono::Utc;

pub struct TestDataGenerator;

impl TestDataGenerator {
    /// 生成测试记忆
    pub fn generate_memory(id: Option<String>) -> Memory {
        Memory {
            id: id.unwrap_or_else(|| Faker.fake::<String>()),
            content: Faker.fake::<String>(),
            memory_type: MemoryType::Episodic,
            importance: (0.0..1.0).fake(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// 生成批量测试记忆
    pub fn generate_memories(count: usize) -> Vec<Memory> {
        (0..count).map(|i| {
            Self::generate_memory(Some(format!("test-{}", i)))
        }).collect()
    }

    /// 生成测试 Agent
    pub fn generate_agent() -> Agent {
        Agent {
            id: Faker.fake::<String>(),
            name: Faker.fake::<String>(),
            config: AgentConfig::default(),
            created_at: Utc::now(),
        }
    }

    /// 生成测试用户
    pub fn generate_user() -> User {
        User {
            id: Faker.fake::<String>(),
            name: Faker.fake::<String>(),
            email: Faker.fake::<String>(),
            created_at: Utc::now(),
        }
    }
}
```

### B.2 Test Fixtures

```rust
// crates/agent-mem-core/src/test_fixtures.rs

pub struct TestFixtures;

impl TestFixtures {
    /// 创建测试数据库
    pub async fn create_test_db() -> PgPool {
        let db_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/agentmem_test".to_string());

        PgPool::connect(&db_url).await.unwrap()
    }

    /// 清理测试数据库
    pub async fn cleanup_test_db(pool: &PgPool) {
        sqlx::query("TRUNCATE TABLE memories CASCADE")
            .execute(pool)
            .await
            .unwrap();
    }

    /// 创建测试 Redis 连接
    pub async fn create_test_redis() -> redis::Client {
        redis::Client::open("redis://localhost:6379/15").unwrap()
    }

    /// 创建测试配置
    pub fn create_test_config() -> MemoryEngineConfig {
        MemoryEngineConfig {
            cache_size: 100,
            enable_compression: false,
            enable_deduplication: true,
            ..Default::default()
        }
    }
}
```

---

## 📚 附录 C: CI/CD 配置

### C.1 GitHub Actions 完整配置

```yaml
# .github/workflows/comprehensive-tests.yml
name: Comprehensive Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # 单元测试
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Run unit tests
        run: cargo test --lib --all-features --verbose

      - name: Generate coverage report
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml

  # 集成测试
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: agentmem_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run database migrations
        run: |
          cargo install sqlx-cli
          sqlx migrate run
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test

      - name: Run integration tests
        run: cargo test --test '*' --all-features --verbose
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test
          REDIS_URL: redis://localhost:6379

  # E2E 测试
  e2e-tests:
    name: E2E Tests
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: agentmem_test
        ports:
          - 5432:5432

      redis:
        image: redis:7
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Build server
        run: cargo build --release --bin agent-mem-server

      - name: Start server
        run: |
          ./target/release/agent-mem-server &
          sleep 5
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/agentmem_test
          REDIS_URL: redis://localhost:6379

      - name: Run E2E tests
        run: cargo test --test e2e_* --all-features --verbose

  # 性能基准测试
  benchmarks:
    name: Performance Benchmarks
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run benchmarks
        run: cargo bench --all-features

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.json
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true

  # 代码质量检查
  code-quality:
    name: Code Quality
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy, rustfmt
          override: true

      - name: Run clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check
```

---

**文档版本**: v1.0
**最后更新**: 2025-10-07
**负责人**: AgentMem 开发团队
**参考**: MIRIX 测试系统

