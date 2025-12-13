# 🔧 数据一致性修复实施计划

**日期**: 2025-12-10  
**优先级**: 🔴 P0 - 致命问题  
**预计时间**: 4-6小时

> 🏆 **最终架构决策**: 参见 `FINAL_ARCHITECTURE_DECISION.md` ⭐⭐⭐ - 基于2025最新研究的最终推荐

---

## 📋 问题总结

### 当前状态
- ✅ `add_memory_fast()` 已添加MemoryManager写入（第4个并行任务）
- ✅ MemoryManager使用LibSQL后端（LibSqlMemoryOperations）
- ✅ UnifiedStorageCoordinator已实现
- ❌ **问题**：coordinator.rs中VectorStore失败时只记录警告，没有回滚Repository
- ❌ **问题**：缺少数据一致性检查机制
- ❌ **问题**：缺少数据同步机制

### 代码位置
- **文件**: `crates/agent-mem-core/src/storage/coordinator.rs`
- **问题行**: 171-177（VectorStore失败时只记录警告）

---

## 🎯 修复方案

### 修复1: 实现补偿机制（回滚逻辑）

**文件**: `crates/agent-mem-core/src/storage/coordinator.rs`

**当前代码**（问题）:
```rust
// Line 171-177
if let Err(e) = self.vector_store.add_vectors(vec![vector_data]).await {
    // If vector store fails, we should rollback LibSQL
    // For now, we log the error and continue (LibSQL is primary)
    warn!(
        "Failed to add memory to vector store (non-critical): {}. Memory exists in LibSQL.",
        e
    );
}
```

**修复后**:
```rust
if let Err(e) = self.vector_store.add_vectors(vec![vector_data]).await {
    // VectorStore失败，回滚Repository
    error!("Failed to add memory to vector store: {}. Rolling back Repository.", e);
    
    // 回滚Repository
    if let Err(rollback_err) = self.sql_repository.delete(&memory.id.0).await {
        error!("Failed to rollback Repository: {}", rollback_err);
        return Err(AgentMemError::StorageError(format!(
            "Failed to store to VectorStore and rollback failed: {} (rollback error: {})",
            e, rollback_err
        )));
    }
    
    return Err(AgentMemError::StorageError(format!(
        "Failed to store to VectorStore, Repository rolled back: {}",
        e
    )));
}
```

**影响**:
- ✅ 确保数据一致性（要么都成功，要么都失败）
- ✅ 避免数据丢失
- ⚠️ 增加回滚开销（但这是必要的）

---

### 修复2: 实现数据一致性检查

**文件**: `crates/agent-mem-core/src/storage/coordinator.rs`

**新增方法**:
```rust
/// 数据一致性报告
#[derive(Debug, Clone)]
pub struct ConsistencyReport {
    pub memory_id: String,
    pub repository_exists: bool,
    pub vectorstore_exists: bool,
    pub content_consistent: bool,
    pub consistency_score: f32,
}

impl UnifiedStorageCoordinator {
    /// 验证数据一致性
    pub async fn verify_consistency(&self, memory_id: &str) -> Result<ConsistencyReport> {
        // Step 1: 检查Repository
        let repo_memory = self.sql_repository.find_by_id(memory_id).await?;
        
        // Step 2: 检查VectorStore
        let vector_result = self.vector_store.get(memory_id).await;
        let vector_memory = match vector_result {
            Ok(Some(m)) => Some(m),
            Ok(None) => None,
            Err(e) => {
                warn!("VectorStore查询失败: {}", e);
                None
            }
        };
        
        // Step 3: 比较一致性
        match (repo_memory, vector_memory) {
            (Some(repo), Some(vec)) => {
                // 检查内容是否一致
                let repo_content = match &repo.content {
                    agent_mem_traits::Content::Text(text) => text.clone(),
                    _ => String::new(),
                };
                
                let vec_content = vec.metadata
                    .get("data")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                
                let content_match = repo_content == vec_content;
                let consistency = if content_match { 1.0 } else { 0.0 };
                
                if !content_match {
                    warn!(
                        "数据内容不一致: memory_id={}, Repository内容长度={}, VectorStore内容长度={}",
                        memory_id,
                        repo_content.len(),
                        vec_content.len()
                    );
                }
                
                Ok(ConsistencyReport {
                    memory_id: memory_id.to_string(),
                    repository_exists: true,
                    vectorstore_exists: true,
                    content_consistent: content_match,
                    consistency_score: consistency,
                })
            }
            (Some(_), None) => {
                warn!("数据不一致: Repository有数据，但VectorStore没有: memory_id={}", memory_id);
                Ok(ConsistencyReport {
                    memory_id: memory_id.to_string(),
                    repository_exists: true,
                    vectorstore_exists: false,
                    content_consistent: false,
                    consistency_score: 0.5,
                })
            }
            (None, Some(_)) => {
                warn!("数据不一致: VectorStore有数据，但Repository没有: memory_id={}", memory_id);
                Ok(ConsistencyReport {
                    memory_id: memory_id.to_string(),
                    repository_exists: false,
                    vectorstore_exists: true,
                    content_consistent: false,
                    consistency_score: 0.5,
                })
            }
            (None, None) => {
                Ok(ConsistencyReport {
                    memory_id: memory_id.to_string(),
                    repository_exists: false,
                    vectorstore_exists: false,
                    content_consistent: true,  // 一致（都不存在）
                    consistency_score: 1.0,
                })
            }
        }
    }
    
    /// 批量验证数据一致性
    pub async fn verify_all_consistency(&self) -> Result<Vec<ConsistencyReport>> {
        // 从Repository获取所有memory IDs
        let memories = self.sql_repository.find_all().await?;
        
        let mut reports = Vec::new();
        for memory in memories {
            let report = self.verify_consistency(&memory.id.0).await?;
            reports.push(report);
        }
        
        Ok(reports)
    }
}
```

---

### 修复3: 实现数据同步机制

**文件**: `crates/agent-mem-core/src/storage/coordinator.rs`

**新增方法**:
```rust
/// 数据同步报告
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub total_memories: usize,
    pub synced_count: usize,
    pub error_count: usize,
    pub skipped_count: usize,
}

impl UnifiedStorageCoordinator {
    /// 从Repository同步到VectorStore
    pub async fn sync_vectorstore_from_repository(&self) -> Result<SyncReport> {
        info!("开始同步：从Repository到VectorStore");
        
        // Step 1: 从Repository读取所有记忆
        let memories = self.sql_repository.find_all().await?;
        info!("Repository中有 {} 条记忆", memories.len());
        
        let mut synced_count = 0;
        let mut error_count = 0;
        let mut skipped_count = 0;
        
        for memory in memories {
            // Step 2: 检查VectorStore是否有对应的向量
            let vector_exists = match self.vector_store.get(&memory.id.0).await {
                Ok(Some(_)) => true,
                Ok(None) => false,
                Err(e) => {
                    warn!("检查VectorStore失败: {}", e);
                    false
                }
            };
            
            if vector_exists {
                skipped_count += 1;
                continue;
            }
            
            // Step 3: 生成向量并写入VectorStore
            // 注意：这里需要embedder，但coordinator没有直接访问
            // 需要从memory中提取内容，或者通过其他方式获取embedding
            
            // 方案A: 如果memory中有embedding字段，直接使用
            // 方案B: 需要coordinator持有embedder引用
            // 方案C: 通过外部调用，传入embedding
            
            // 暂时跳过，需要设计接口
            warn!("同步功能需要embedder支持，暂时跳过: memory_id={}", memory.id.0);
            skipped_count += 1;
        }
        
        Ok(SyncReport {
            total_memories: memories.len(),
            synced_count,
            error_count,
            skipped_count,
        })
    }
    
    /// 从VectorStore同步到Repository（反向同步）
    pub async fn sync_repository_from_vectorstore(&self) -> Result<SyncReport> {
        // 这个功能需要VectorStore支持list操作
        // 暂时不实现，因为VectorStore可能不支持
        warn!("反向同步功能需要VectorStore支持list操作，暂时不实现");
        Ok(SyncReport {
            total_memories: 0,
            synced_count: 0,
            error_count: 0,
            skipped_count: 0,
        })
    }
}
```

**注意**: 同步功能需要embedder支持，需要设计接口。

---

### 修复4: 实现混合检索

**文件**: `crates/agent-mem-core/src/storage/coordinator.rs`

**新增方法**:
```rust
impl UnifiedStorageCoordinator {
    /// 混合检索（时间+语义）
    pub async fn hybrid_search(
        &self,
        query: Option<&str>,
        query_embedding: Option<Vec<f32>>,
        agent_id: Option<&str>,
        user_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>> {
        let limit = limit.unwrap_or(10);
        let half_limit = limit / 2;
        
        // Step 1: 并行检索
        let (recent_results, semantic_results) = tokio::join!(
            // 时间优先：最近N条（从Repository）
            async {
                if let Some(agent_id) = agent_id {
                    self.sql_repository
                        .find_by_agent_id(agent_id, user_id, half_limit as i64)
                        .await
                } else {
                    Ok(vec![])
                }
            },
            // 语义优先：最相关M条（从VectorStore）
            async {
                if let (Some(query_emb), Some(agent_id)) = (query_embedding, agent_id) {
                    let mut filters = HashMap::new();
                    filters.insert("agent_id".to_string(), agent_id.to_string());
                    if let Some(user_id) = user_id {
                        filters.insert("user_id".to_string(), user_id.to_string());
                    }
                    
                    match self.vector_store.search(query_emb, filters, half_limit).await {
                        Ok(results) => {
                            // 转换为Memory（需要实现）
                            Ok(self.vector_results_to_memories(results))
                        }
                        Err(e) => {
                            warn!("VectorStore搜索失败: {}", e);
                            Ok(vec![])
                        }
                    }
                } else {
                    Ok(vec![])
                }
            }
        );
        
        // Step 2: 合并去重
        let mut combined = Vec::new();
        let mut seen_ids = HashSet::new();
        
        // 先添加语义结果（相关性高）
        for result in semantic_results? {
            if !seen_ids.contains(&result.id.0) {
                combined.push(result);
                seen_ids.insert(result.id.0.clone());
            }
        }
        
        // 再添加时间结果（保证连贯性）
        for result in recent_results? {
            if !seen_ids.contains(&result.id.0) {
                combined.push(result);
                seen_ids.insert(result.id.0.clone());
            }
        }
        
        // Step 3: 限制总数
        combined.truncate(limit);
        
        Ok(combined)
    }
    
    /// 将VectorStore结果转换为Memory
    fn vector_results_to_memories(&self, results: Vec<agent_mem_traits::VectorData>) -> Vec<Memory> {
        // 需要实现转换逻辑
        // 暂时返回空，需要设计
        vec![]
    }
}
```

---

## 📋 实施清单

### Phase 1: 立即修复（P0 - 今天）

- [ ] **修复1**: 实现补偿机制（回滚逻辑）
  - [ ] 修改coordinator.rs:171-177
  - [ ] VectorStore失败时回滚Repository
  - [ ] 添加错误处理
  - [ ] 添加测试

- [ ] **修复2**: 实现数据一致性检查
  - [ ] 添加verify_consistency方法
  - [ ] 添加verify_all_consistency方法
  - [ ] 添加ConsistencyReport结构
  - [ ] 添加测试

### Phase 2: 功能完善（P1 - 明天）

- [ ] **修复3**: 实现数据同步机制
  - [ ] 添加sync_vectorstore_from_repository方法
  - [ ] 设计embedder接口
  - [ ] 添加SyncReport结构
  - [ ] 添加测试

- [ ] **修复4**: 实现混合检索
  - [ ] 添加hybrid_search方法
  - [ ] 实现vector_results_to_memories转换
  - [ ] 添加测试

### Phase 3: 测试和验证（P1 - 后天）

- [ ] 端到端测试
- [ ] 性能测试
- [ ] 数据一致性测试
- [ ] 文档更新

---

## ✅ 验收标准

- ✅ 存储和检索数据源一致
- ✅ 数据一致性测试通过（100%通过率）
- ✅ 补偿机制工作正常（部分失败时能回滚）
- ✅ 数据同步机制工作正常
- ✅ 混合检索性能提升（延迟 < 100ms P95）

---

**负责人**: AI Assistant  
**审核**: 待用户确认  
**预计完成**: 本周内
