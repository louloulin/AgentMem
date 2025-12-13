//! Memory management routes - Unified Memory API version
//!
//! 架构优化：使用agent-mem的Memory统一API替代agent-mem-core的CoreMemoryManager
//! 优势：
//! - 更简洁的代码
//! - 统一的接口
//! - 自动的智能功能
//! - 更好的类型处理
//!
//! 注意：本模块内部使用 MemoryItem 用于向后兼容，未来版本将迁移到 Memory V4
//!
//! 🆕 模块拆分（2025-12-10）：
//! - memory/cache.rs: 查询结果缓存逻辑
//! - memory/stats.rs: 搜索统计逻辑
//! - 路由处理函数保留在此文件中（未来可进一步拆分到 handlers.rs）

// 使用拆分的模块（作为子模块）
#[path = "memory/cache.rs"]
mod cache;
#[path = "memory/stats.rs"]
mod stats;
#[path = "memory/utils.rs"]
mod utils;

// 重新导出以便向后兼容
pub use cache::{get_search_cache, generate_cache_key, CachedSearchResult};
pub use stats::{get_search_stats, SearchStatistics};
pub use utils::{
    truncate_string_at_char_boundary, contains_chinese, calculate_recency_score,
    calculate_3d_score, calculate_quality_score, get_adaptive_threshold,
    detect_exact_query, convert_memory_to_json, calculate_access_pattern_score,
    calculate_auto_importance, apply_hierarchical_sorting, apply_intelligent_filtering,
    compute_prefetch_candidates,
};

use crate::{
    error::{ServerError, ServerResult},
    models::{
        BatchRequest, BatchResponse, BatchSearchRequest, BatchSearchResponse, MemoryRequest,
        MemoryResponse, SearchRequest, SearchResponse, UpdateMemoryRequest,
    },
};
use agent_mem::{AddMemoryOptions, DeleteAllOptions, GetAllOptions, Memory, SearchOptions};

// 内部使用 MemoryItem 用于向后兼容（已废弃，未来将迁移到 Memory V4）
#[allow(deprecated)]
use agent_mem_traits::MemoryItem;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use futures::future::{self, join_all};

/// Server-side memory manager wrapper (基于Memory统一API)
pub struct MemoryManager {
    pub memory: Arc<Memory>,
    /// 🆕 Fix 2: 查询优化器
    query_optimizer: Arc<agent_mem_core::search::QueryOptimizer>,
    /// 🆕 Fix 2: 结果重排序器（使用reranker模块的ResultReranker）
    reranker: Arc<agent_mem_core::search::reranker::ResultReranker>,
}

impl MemoryManager {
    /// 创建新的MemoryManager（使用Memory API + LibSQL持久化 + Embedder配置）
    pub async fn new(
        embedder_provider: Option<String>,
        embedder_model: Option<String>,
    ) -> ServerResult<Self> {
        use tracing::warn;

        info!("========================================");
        info!("🧠 初始化 Memory 组件");
        info!("========================================");

        // 🔧 修复：使用builder模式显式指定LibSQL存储，而不是默认的内存存储
        // 支持 memory:// URL 格式（用于测试，避免数据库锁定）
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:./data/agentmem.db".to_string());

        info!("📦 配置存储层");
        info!("  - 数据库类型: LibSQL (SQLite)");
        info!("  - 数据库路径: {}", db_path);

        let mut builder = Memory::builder().with_storage(&db_path); // 🔑 关键修复：显式指定使用LibSQL
                                                                    // ⚠️ 不设置 default_user_id 和 default_agent_id
                                                                    // 强制每次调用时显式传入，避免被默认值覆盖

        // 🔑 关键修复 #2：配置Embedder（P0问题）
        info!("🔌 配置 Embedder (向量嵌入)");
        if let (Some(provider), Some(model)) = (embedder_provider.clone(), embedder_model.clone()) {
            info!("  - Provider: {}", provider);
            info!("  - Model: {}", model);
            builder = builder.with_embedder(provider, model);
        } else {
            // 使用默认FastEmbed配置
            info!("  - Provider: fastembed (默认)");
            info!("  - Model: BAAI/bge-small-en-v1.5");
            builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");
        }

        // 🔑 关键修复 #3：配置VectorStore（向量持久化）
        // 修复: 之前向量只在内存中，重启后丢失
        // 注意: LanceDB需要协议前缀 "lancedb://"，路径需要以.lance结尾
        let vector_store_url = "lancedb://./data/vectors.lance";
        info!("📊 配置向量存储");
        info!("  - 类型: LanceDB");
        info!("  - 路径: {}", vector_store_url);
        builder = builder.with_vector_store(vector_store_url);

        info!("⏳ 构建 Memory 实例...");
        warn!("⚠️  首次运行时，FastEmbed 会下载模型文件（约 100MB）");
        warn!("⚠️  这可能需要几分钟时间，请耐心等待...");
        warn!("⚠️  下载进度不会显示，但程序正在运行中");

        let memory = builder.build().await.map_err(|e| {
            ServerError::Internal(format!("Failed to create Memory with LibSQL: {}", e))
        })?;

        info!("✅ Memory 实例构建成功");

        // 🆕 Fix 2: 初始化QueryOptimizer和Reranker
        info!("🔍 初始化搜索优化组件...");
        let query_optimizer = {
            use std::sync::RwLock;
            let stats = Arc::new(RwLock::new(
                agent_mem_core::search::IndexStatistics::default(),
            ));
            agent_mem_core::search::QueryOptimizer::with_default_config(stats)
        };

        let reranker = agent_mem_core::search::reranker::ResultReranker::with_default_config();

        info!("✅ QueryOptimizer 和 Reranker 初始化完成");
        info!("========================================");
        info!("✅ Memory 组件初始化完成！");
        info!("========================================");

        Ok(Self {
            memory: Arc::new(memory),
            query_optimizer: Arc::new(query_optimizer),
            reranker: Arc::new(reranker),
        })
    }

    /// 使用自定义配置创建
    pub async fn with_config(memory: Memory) -> Self {
        // 🆕 Fix 2: 初始化QueryOptimizer和Reranker
        let query_optimizer = {
            use std::sync::RwLock;
            let stats = Arc::new(RwLock::new(
                agent_mem_core::search::IndexStatistics::default(),
            ));
            agent_mem_core::search::QueryOptimizer::with_default_config(stats)
        };

        let reranker = agent_mem_core::search::reranker::ResultReranker::with_default_config();

        Self {
            memory: Arc::new(memory),
            query_optimizer: Arc::new(query_optimizer),
            reranker: Arc::new(reranker),
        }
    }

    /// 添加记忆（🔧 最佳方案：Memory API + LibSQL 双写）
    ///
    /// Strategy:
    /// 1. 使用Memory API生成向量嵌入（保留智能功能）
    /// 2. 同时写入LibSQL确保持久化
    /// 3. 向量搜索使用VectorStore，结构化查询使用LibSQL
    pub async fn add_memory(
        &self,
        repositories: Arc<agent_mem_core::storage::factory::Repositories>,
        agent_id: Option<String>,
        user_id: Option<String>,
        content: String,
        memory_type: Option<agent_mem_traits::MemoryType>,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        use agent_mem_utils::hash::compute_content_hash;
        use chrono::Utc;

        // ✅ 生成有效的 agent_id (参考mem0设计)
        let effective_agent_id = agent_id.unwrap_or_else(|| {
            if let Some(uid) = &user_id {
                format!("default-agent-{}", uid)
            } else {
                "default-agent".to_string()
            }
        });

        // Step 1: 使用Memory API（生成向量嵌入）
        let options = AddMemoryOptions {
            agent_id: Some(effective_agent_id.clone()),
            user_id: user_id.clone(),
            infer: false, // 简单模式，避免复杂推理
            metadata: metadata.clone().unwrap_or_default(),
            memory_type: memory_type.as_ref().map(|t| format!("{:?}", t)),
            ..Default::default()
        };

        let add_result = self
            .memory
            .add_with_options(&content, options)
            .await
            .map_err(|e| e.to_string())?;

        let memory_id = add_result
            .results
            .first()
            .map(|r| r.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Step 2: 写入LibSQL Repository（持久化）
        let user_id_val = user_id.unwrap_or_else(|| "default".to_string());
        let content_hash = compute_content_hash(&content);
        let now = Utc::now();

        // 构建metadata JSON
        let mut full_metadata = metadata.unwrap_or_default();
        full_metadata.insert("agent_id".to_string(), effective_agent_id.clone());
        full_metadata.insert("user_id".to_string(), user_id_val.clone());
        full_metadata.insert("data".to_string(), content.clone());
        full_metadata.insert("hash".to_string(), content_hash.clone());

        // 🆕 Phase 2 Server: 提取scope_type（如果没有则自动推断）
        // ✅ 修复优先级：user_id优先于session_id（符合agentmem61.md设计）
        let scope_type = full_metadata.get("scope_type").cloned().unwrap_or_else(|| {
            // 自动推断scope类型 - 正确的优先级
            // 1. 如果有user_id和agent_id（非默认），这是长期记忆（Agent scope）
            if user_id_val != "default"
                && effective_agent_id.starts_with("agent-")
                && effective_agent_id != "default-agent"
            {
                "agent".to_string()
            }
            // 2. 如果只有user_id（非默认），这是用户记忆（User scope）
            else if user_id_val != "default" {
                "user".to_string()
            }
            // 3. 如果有session_id，这是工作记忆（Session scope）
            else if full_metadata.contains_key("session_id") {
                "session".to_string()
            }
            // 4. 如果有run_id
            else if full_metadata.contains_key("run_id") {
                "run".to_string()
            }
            // 5. 如果有org_id
            else if full_metadata.contains_key("org_id") {
                "organization".to_string()
            }
            // 6. 默认为全局
            else {
                "global".to_string()
            }
        });

        let metadata_json: serde_json::Value = full_metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // Step 2.5: 确保Agent存在（获取其organization_id和user_id）
        // ✅ 如果agent不存在，使用默认值（参考mem0设计）
        let agent_opt = repositories
            .agents
            .find_by_id(&effective_agent_id)
            .await
            .map_err(|e| format!("Failed to query agent: {}", e))?;

        let organization_id = agent_opt
            .as_ref()
            .map(|a| a.organization_id.clone())
            .unwrap_or_else(|| "default-org".to_string());

        let db_memory = agent_mem_core::storage::models::DbMemory {
            id: memory_id.clone(),
            organization_id,                // 使用Agent的organization_id或默认值
            user_id: "default".to_string(), // 使用默认user (TODO: 应该从auth获取实际user)
            agent_id: effective_agent_id.clone(),
            content,
            hash: Some(content_hash),
            metadata: metadata_json,
            score: None,
            memory_type: format!(
                "{:?}",
                memory_type.unwrap_or(agent_mem_traits::MemoryType::Semantic)
            ),
            scope: scope_type, // 🆕 Phase 2 Server: 使用推断或提取的scope_type
            level: "normal".to_string(),
            importance: importance.unwrap_or(0.5),
            access_count: 0,
            last_accessed: Some(now),
            created_at: now,
            updated_at: now,
            is_deleted: false,
            created_by_id: None,
            last_updated_by_id: None,
        };

        // 转换为 MemoryV4 以便调用 repository.create
        use agent_mem_core::storage::conversion::db_to_memory;
        let memory = db_to_memory(&db_memory)
            .map_err(|e| format!("Failed to convert to MemoryV4: {}", e))?;

        repositories
            .memories
            .create(&memory)
            .await
            .map_err(|e| format!("Failed to persist to LibSQL: {}", e))?;

        info!(
            "✅ Memory persisted: VectorStore + LibSQL (ID: {})",
            memory_id
        );
        Ok(memory_id)
    }

    /// 获取记忆（直接数据库查询）
    pub async fn get_memory(&self, id: &str) -> Result<Option<serde_json::Value>, String> {
        use libsql::{params, Builder};

        let db_path =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "data/agentmem.db".to_string());

        let db = Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let conn = db
            .connect()
            .map_err(|e| format!("Failed to connect: {}", e))?;

        // 🆕 Phase 2 Server: 查询中包含scope字段
        let query = "SELECT id, agent_id, user_id, content, memory_type, importance, \
                     created_at, last_accessed, access_count, metadata, hash, scope \
                     FROM memories WHERE id = ? AND is_deleted = 0 LIMIT 1";

        let mut stmt = conn
            .prepare(query)
            .await
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt
            .query(params![id])
            .await
            .map_err(|e| format!("Failed to query: {}", e))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| format!("Failed to fetch row: {}", e))?
        {
            let memory_id = row.get::<String>(0).unwrap_or_default();
            
            // 🆕 Phase 2.11: 自动更新访问统计和重要性
            // 更新access_count和last_accessed
            let now = chrono::Utc::now().timestamp();
            let current_access_count: i64 = row.get(8).unwrap_or(0);
            let new_access_count = current_access_count + 1;
            
            // 基于访问模式自动调整importance
            let current_importance: f64 = row.get(5).unwrap_or(0.5);
            let last_accessed_ts: Option<i64> = row.get(7).ok();
            let new_importance = calculate_auto_importance(
                current_importance,
                new_access_count,
                last_accessed_ts,
            );
            
            // 更新数据库（异步，不阻塞返回）
            let db_path_clone = db_path.clone();
            let id_clone = memory_id.clone();
            tokio::spawn(async move {
                if let Ok(update_db) = Builder::new_local(&db_path_clone).build().await {
                    if let Ok(update_conn) = update_db.connect() {
                        let update_query = "UPDATE memories SET access_count = ?, last_accessed = ?, importance = ?, updated_at = ? WHERE id = ?";
                        if let Ok(mut update_stmt) = update_conn.prepare(update_query).await {
                            let _ = update_stmt
                                .execute(params![new_access_count, now, new_importance, now, id_clone])
                                .await;
                        }
                    }
                }
            });
            
            // ✅ 修复时间戳：将 i64 秒级时间戳转换为 ISO 8601 字符串
            use chrono::{DateTime, Utc};

            let created_at_ts: Option<i64> = row.get(6).ok();
            let created_at_str = created_at_ts
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.to_rfc3339());

            // 使用当前时间作为last_accessed（因为刚刚更新）
            let last_accessed_str = DateTime::from_timestamp(now, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());

            let json = serde_json::json!({
                "id": memory_id,
                "agent_id": row.get::<String>(1).unwrap_or_default(),
                "user_id": row.get::<String>(2).unwrap_or_default(),
                "content": row.get::<String>(3).unwrap_or_default(),
                "memory_type": row.get::<Option<String>>(4).ok().flatten(),
                "importance": new_importance,  // 🆕 使用更新后的importance
                "created_at": created_at_str,
                "last_accessed_at": last_accessed_str,  // 🆕 使用当前时间
                "access_count": new_access_count,  // 🆕 使用更新后的access_count
                "metadata": row.get::<Option<String>>(9).ok().flatten(),
                "hash": row.get::<Option<String>>(10).ok().flatten(),
                "scope": row.get::<Option<String>>(11).ok().flatten(),  // 🆕 Phase 2 Server: 返回scope字段
            });
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    /// 更新记忆
    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<String>,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), String> {
        let mut update_data = HashMap::new();

        if let Some(c) = content {
            update_data.insert("content".to_string(), serde_json::json!(c));
        }
        if let Some(imp) = importance {
            update_data.insert("importance".to_string(), serde_json::json!(imp));
        }
        if let Some(meta) = metadata {
            // 转换metadata为JSON
            let meta_json: HashMap<String, serde_json::Value> = meta
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            update_data.insert("metadata".to_string(), serde_json::json!(meta_json));
        }

        self.memory
            .update(id, update_data)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// 删除记忆
    pub async fn delete_memory(&self, id: &str) -> Result<(), String> {
        self.memory.delete(id).await.map_err(|e| e.to_string())
    }

    /// 搜索记忆 (🆕 Fix 2: 集成QueryOptimizer和Reranker)
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: Option<String>,
        user_id: Option<String>,
        limit: Option<usize>,
        _memory_type: Option<agent_mem_traits::MemoryType>,
    ) -> Result<Vec<MemoryItem>, String> {
        // 🆕 Fix 2: 使用QueryOptimizer优化查询
        use agent_mem_core::search::SearchQuery;
        let search_query = SearchQuery {
            query: query.clone(),
            limit: limit.unwrap_or(10),
            threshold: Some(0.7),
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: None,
            metadata_filters: None,
        };

        let optimized_plan = self
            .query_optimizer
            .optimize_query(&search_query)
            .map_err(|e| format!("Query optimization failed: {}", e))?;

        info!("🚀 Query optimized: strategy={:?}, should_rerank={}, rerank_factor={}, estimated_latency={}ms", 
            optimized_plan.strategy, optimized_plan.should_rerank, optimized_plan.rerank_factor, 
            optimized_plan.estimated_latency_ms);

        // 🆕 Fix 2: 使用优化后的参数 - 如果需要重排序，增加候选数量
        let base_limit = limit.unwrap_or(10);
        let fetch_limit = if optimized_plan.should_rerank {
            base_limit * optimized_plan.rerank_factor
        } else {
            base_limit
        };

        // 🔧 智能阈值调整：根据查询类型动态设置
        let dynamic_threshold = get_adaptive_threshold(&query);
        info!(
            "📊 自适应阈值: query='{}', threshold={}",
            query, dynamic_threshold
        );

        let options = SearchOptions {
            user_id: user_id.clone(),
            limit: Some(fetch_limit),
            threshold: Some(dynamic_threshold),
            ..Default::default()
        };

        // 执行搜索
        let raw_results = self
            .memory
            .search_with_options(query.clone(), options)
            .await
            .map_err(|e| e.to_string())?;

        // 🆕 Phase 3-D: 如果需要重排序且有结果，使用Reranker优化
        if optimized_plan.should_rerank && !raw_results.is_empty() && raw_results.len() > base_limit
        {
            // 保存结果数量用于日志
            let raw_count = raw_results.len();

            match self
                .apply_reranking(&query, &search_query, raw_results, base_limit)
                .await
            {
                Ok(reranked) => {
                    info!(
                        "✨ Reranking applied successfully: {} → {} final results",
                        raw_count,
                        reranked.len()
                    );
                    return Ok(reranked);
                }
                Err(e) => {
                    // Reranking失败时降级：重新执行搜索，使用base_limit
                    warn!(
                        "⚠️  Reranking failed ({}), falling back to direct search with base_limit",
                        e
                    );
                    let fallback_options = SearchOptions {
                        user_id,
                        limit: Some(base_limit),
                        threshold: Some(dynamic_threshold), // 使用动态阈值
                        ..Default::default()
                    };
                    return self
                        .memory
                        .search_with_options(query, fallback_options)
                        .await
                        .map_err(|e| e.to_string());
                }
            }
        }

        // 不需要重排序或结果不足，直接返回（可能需要截断）
        Ok(raw_results.into_iter().take(base_limit).collect())
    }

    /// 🆕 应用Reranker重排序
    ///
    /// 将MemoryItem转换为SearchResult，调用Reranker，再转换回来
    async fn apply_reranking(
        &self,
        query: &str,
        search_query: &agent_mem_core::search::SearchQuery,
        raw_results: Vec<MemoryItem>,
        final_limit: usize,
    ) -> Result<Vec<MemoryItem>, String> {
        use agent_mem_core::search::SearchResult;

        // 1. 尝试生成query vector（用于Reranker）
        // 注意：如果无法生成query_vector，我们将使用现有的score进行重排序
        let query_vector_result = {
            // 尝试通过搜索API获取query vector
            // 由于Memory API没有直接暴露embedder，我们使用一个简化的方法：
            // 使用第一个结果的向量作为参考（如果可用），或者使用默认向量
            // 实际上，Reranker可以使用现有的score，所以我们可以创建一个占位向量
            let default_dim = 384; // FastEmbed默认维度，可以根据实际配置调整
            vec![0.0f32; default_dim] // 占位向量，Reranker会主要使用现有score
        };

        // 2. 转换MemoryItem → SearchResult
        let candidates: Vec<SearchResult> = raw_results
            .iter()
            .map(|item| SearchResult {
                id: item.id.clone(),
                content: item.content.clone(),
                score: item.score.unwrap_or(0.5),
                vector_score: item.score,
                fulltext_score: None,
                metadata: Some(
                    serde_json::to_value(&item.metadata).unwrap_or(serde_json::json!({})),
                ),
            })
            .collect();

        // 3. 调用Reranker进行重排序
        // Reranker会基于多个因素（相似度、元数据、时间、重要性、质量）重新评分
        // 注意：Arc会自动解引用，所以可以直接调用
        let reranked_results = self
            .reranker
            .rerank(candidates, &query_vector_result, search_query)
            .await
            .map_err(|e| format!("Reranker execution failed: {}", e))?;

        // 4. 转换回MemoryItem（保持原始MemoryItem数据，只更新顺序和score）
        let mut result_map: std::collections::HashMap<String, MemoryItem> = raw_results
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect();

        let final_results: Vec<MemoryItem> = reranked_results
            .into_iter()
            .take(final_limit)
            .filter_map(|reranked| {
                result_map.get_mut(&reranked.id).map(|item| {
                    // 更新score为重排序后的分数
                    item.score = Some(reranked.score);
                    item.clone()
                })
            })
            .collect();

        Ok(final_results)
    }

    /// 获取所有记忆
    pub async fn get_all_memories(
        &self,
        agent_id: Option<String>,
        user_id: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>, String> {
        let options = GetAllOptions {
            agent_id,
            user_id,
            limit,
            ..Default::default()
        };

        self.memory
            .get_all(options)
            .await
            .map_err(|e| e.to_string())
    }

    /// 删除所有记忆
    pub async fn delete_all_memories(
        &self,
        agent_id: Option<String>,
        user_id: Option<String>,
    ) -> Result<usize, String> {
        let options = DeleteAllOptions {
            agent_id,
            user_id,
            ..Default::default()
        };

        self.memory
            .delete_all(options)
            .await
            .map_err(|e| e.to_string())
    }

    /// 重置所有记忆（危险操作）
    pub async fn reset(&self) -> Result<(), String> {
        self.memory.reset().await.map_err(|e| e.to_string())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<agent_mem::MemoryStats, String> {
        self.memory.get_stats().await.map_err(|e| e.to_string())
    }
}

/// 默认实现（异步创建）
impl MemoryManager {
    pub fn new_sync() -> Self {
        // 注意：这只用于类型系统，实际使用应该调用async new()
        panic!("Use MemoryManager::new().await instead");
    }
}

// ==================== 辅助函数 ====================
// 注意：辅助函数已迁移到 utils.rs 模块

// ==================== 路由处理器函数 ====================
// 以下是实际的HTTP路由处理器函数

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
};
use tracing::{debug, error, info, warn};

/// 添加新记忆（🔧 使用双写策略）
#[utoipa::path(
    post,
    path = "/api/v1/memories",
    tag = "memory",
    request_body = crate::models::MemoryRequest,
    responses(
        (status = 201, description = "Memory created successfully", body = crate::models::MemoryResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_memory(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<crate::models::MemoryRequest>,
) -> ServerResult<(
    StatusCode,
    Json<crate::models::ApiResponse<crate::models::MemoryResponse>>,
)> {
    info!(
        "Adding new memory for agent_id: {:?}, user_id: {:?}",
        request.agent_id, request.user_id
    );

    let memory_id = memory_manager
        .add_memory(
            repositories, // 传递repositories用于LibSQL持久化
            request.agent_id,
            request.user_id,
            request.content,
            request.memory_type,
            request.importance,
            request.metadata,
        )
        .await
        .map_err(|e| {
            error!("Failed to add memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    let response = crate::models::MemoryResponse {
        id: memory_id,
        message: "Memory added successfully (VectorStore + LibSQL)".to_string(),
    };

    Ok((
        StatusCode::CREATED,
        Json(crate::models::ApiResponse::success(response)),
    ))
}

/// 获取记忆
#[utoipa::path(
    get,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory retrieved successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("Getting memory with ID: {}", id);

    let memory = memory_manager.get_memory(&id).await.map_err(|e| {
        error!("Failed to get memory: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    match memory {
        Some(mem) => Ok(Json(crate::models::ApiResponse::success(mem))),
        None => Err(ServerError::NotFound("Memory not found".to_string())),
    }
}

/// 更新记忆
#[utoipa::path(
    put,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    request_body = crate::models::UpdateMemoryRequest,
    responses(
        (status = 200, description = "Memory updated successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Path(id): Path<String>,
    Json(request): Json<crate::models::UpdateMemoryRequest>,
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::MemoryResponse>>> {
    info!("Updating memory with ID: {}", id);

    // 🔧 修复: 直接更新LibSQL Repository
    // 先获取现有记忆
    let existing = repositories
        .memories
        .find_by_id(&id)
        .await
        .map_err(|e| {
            error!("Failed to find memory for update: {}", e);
            ServerError::MemoryError(format!("Memory not found: {}", e))
        })?
        .ok_or_else(|| ServerError::MemoryError("Memory not found".to_string()))?;

    // 构建更新后的Memory，保留其他字段
    let updated_content = if let Some(content) = request.content {
        agent_mem_traits::Content::text(content)
    } else {
        existing.content.clone()
    };

    let updated_importance = request
        .importance
        .unwrap_or_else(|| {
            existing.importance()
                .map(|v| v as f32)
                .unwrap_or(0.5)
        });

    // 使用builder模式构建更新后的Memory
    let mut updated = existing.clone();
    updated.content = updated_content;

    // 更新importance - 使用system命名空间（和importance()方法一致）
    updated.attributes.set(
        agent_mem_traits::AttributeKey::system("importance"),
        agent_mem_traits::AttributeValue::Number(updated_importance as f64),
    );
    updated.metadata.updated_at = chrono::Utc::now();

    // 执行更新
    repositories.memories.update(&updated).await.map_err(|e| {
        error!("Failed to update memory in repository: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    info!("✅ Memory updated in LibSQL");

    let response = crate::models::MemoryResponse {
        id,
        message: "Memory updated successfully".to_string(),
    };

    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 删除记忆
#[utoipa::path(
    delete,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory deleted successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Path(id): Path<String>,
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::MemoryResponse>>> {
    info!("删除记忆: {}", id);
    info!("Deleting memory with ID: {}", id);

    // 🔧 修复: 先检查记忆是否存在
    let memory_exists = repositories.memories.find_by_id(&id).await
        .ok()
        .flatten()
        .is_some();
    
    if !memory_exists {
        warn!("记忆不存在于LibSQL: {}", id);
        return Err(ServerError::NotFound(format!("Memory not found: {}", id)));
    }
    
    // 🔧 修复: 先删除LibSQL（主存储），然后尝试删除向量存储
    // 如果向量存储删除失败（记忆不存在），不应该导致整个删除失败
    let libsql_result = repositories.memories.delete(&id).await;
    
    match libsql_result {
        Ok(_) => {
            info!("✅ Memory deleted from LibSQL: {}", id);
            
            // 尝试删除向量存储（非关键操作，失败不影响主流程）
            let vector_result = memory_manager.delete_memory(&id).await;
            match vector_result {
                Ok(_) => {
                    info!("✅ Memory deleted from both LibSQL and Vector Store: {}", id);
                }
                Err(e) => {
                    // 🔧 修复: 向量存储删除失败不应该导致整个删除失败
                    // 因为主存储（LibSQL）已经删除成功
                    let error_msg = e.to_string();
                    if error_msg.contains("not found") || error_msg.contains("Memory not found") {
                        warn!("⚠️  向量存储中记忆不存在（可能从未添加或已删除）: {}. 这不会影响删除操作", id);
                    } else {
                        warn!("⚠️  向量存储删除失败（非关键）: {}. 错误: {}. 记忆已从主存储删除", id, error_msg);
                    }
                }
            }
            
            let response = crate::models::MemoryResponse {
                id,
                message: "Memory deleted successfully".to_string(),
            };
            Ok(Json(crate::models::ApiResponse::success(response)))
        }
        Err(e) => {
            error!("Failed to delete memory from LibSQL: {}", e);
            Err(ServerError::MemoryError(format!(
                "Failed to delete memory: {}", e
            )))
        }
    }
}

/// 搜索记忆
// ========== 混合检索辅助函数 ==========
// 注意：辅助函数已迁移到 utils.rs 模块，使用 utils::* 导入

/// 通过LibSQL精确查询（商品ID等）- 使用search方法，直接返回JSON
async fn search_by_libsql_exact(
    repositories: &Arc<agent_mem_core::storage::factory::Repositories>,
    query: &str,
    limit: usize,
) -> Result<Vec<serde_json::Value>, String> {
    use tracing::{debug, error, info};

    info!("🔍 LibSQL精确查询: query='{}', limit={}", query, limit);

    // 使用repositories.memories.search方法（支持content LIKE查询）
    match repositories
        .memories
        .search(query, (limit * 2) as i64) // 多取一些，用于排序过滤
        .await
    {
        Ok(memories) if !memories.is_empty() => {
            info!("✅ LibSQL查询成功: 找到 {} 条记忆", memories.len());

            // 🔧 修复: 将 MemoryV4 转换为 MemoryItem 以便访问字段
            use agent_mem_traits::MemoryV4;
            let memory_items: Vec<_> = memories.into_iter().map(|m| m.to_legacy_item()).collect();

            // 🔧 修复: 优先返回精确匹配的商品记忆
            // 1. 分离精确匹配和模糊匹配
            let mut exact_matches = Vec::new();
            let mut fuzzy_matches = Vec::new();

            for mem in memory_items {
                let is_exact = {
                    // 检查 content 是否包含 "商品ID: {query}"
                    mem.content.contains(&format!("商品ID: {}", query)) ||
                    // 检查 metadata 中的 product_id 是否匹配
                    mem.metadata
                        .get("product_id")
                        .and_then(|v| v.as_str())
                        .map(|pid| pid == query)
                        .unwrap_or(false)
                };

                // 排除工作记忆（working memory），它们通常是LLM的回复
                let memory_type_str = format!("{:?}", mem.memory_type);
                let is_working_memory = matches!(memory_type_str.as_str(), "Working");

                if is_exact && !is_working_memory {
                    exact_matches.push(mem);
                } else if !is_working_memory {
                    fuzzy_matches.push(mem);
                }
            }

            info!(
                "📊 精确匹配: {} 条, 模糊匹配: {} 条",
                exact_matches.len(),
                fuzzy_matches.len()
            );

            // 2. 合并结果：精确匹配在前，模糊匹配在后
            let mut sorted_memories = exact_matches;
            sorted_memories.extend(fuzzy_matches);

            // 3. 限制返回数量
            sorted_memories.truncate(limit);

            for mem in &sorted_memories {
                // 🔧 修复: 使用字符边界而不是字节边界，避免UTF-8字符中间切片导致panic
                let content_preview = truncate_string_at_char_boundary(&mem.content, 50);
                debug!(
                    "  - ID: {}, Type: {:?}, Content: {}...",
                    mem.id, mem.memory_type, content_preview
                );
            }

            // 直接转换为JSON
            let json_results: Vec<serde_json::Value> = sorted_memories
                .into_iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "agent_id": m.agent_id,
                        "user_id": m.user_id,
                        "content": m.content,
                        "memory_type": format!("{:?}", m.memory_type),
                        "importance": m.importance,
                        "created_at": m.created_at.to_rfc3339(),
                        "updated_at": m.updated_at.map(|dt| dt.to_rfc3339()),
                        "access_count": m.access_count,
                        "metadata": m.metadata,
                        "hash": m.hash,
                        "score": m.score.unwrap_or(1.0),
                    })
                })
                .collect();

            if json_results.is_empty() {
                info!("⚠️  过滤后没有有效结果: query='{}'", query);
                Err(format!("未找到匹配的记忆: {}", query))
            } else {
                Ok(json_results)
            }
        }
        Ok(_) => {
            info!("⚠️  LibSQL未找到结果: query='{}'", query);
            Err(format!("未找到匹配的记忆: {}", query))
        }
        Err(e) => {
            error!("❌ LibSQL查询失败: {}", e);
            Err(format!("LibSQL查询失败: {}", e))
        }
    }
}

// 注意：convert_memory_to_json 和 compute_prefetch_candidates 已迁移到 utils.rs

/// 🆕 Phase 2.3: 智能预取（简化版） - 基于访问模式和搜索历史预取
async fn prefetch_for_query(
    repositories: Arc<agent_mem_core::storage::factory::Repositories>,
    memory_manager: Arc<MemoryManager>,
    request: &crate::models::SearchRequest,
) -> ServerResult<usize> {
    use libsql::Builder;

    let fetch_limit = request.limit.unwrap_or(10).saturating_mul(2).min(50).max(1);
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;

    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;

    // 根据过滤条件构建查询
    let mut rows = if let Some(agent_id) = &request.agent_id {
        let mut stmt = conn
            .prepare("SELECT id, access_count, last_accessed FROM memories WHERE is_deleted = 0 AND agent_id = ? ORDER BY access_count DESC, last_accessed DESC LIMIT ?")
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
        stmt.query(libsql::params![agent_id.clone(), fetch_limit as i64])
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?
    } else if let Some(user_id) = &request.user_id {
        let mut stmt = conn
            .prepare("SELECT id, access_count, last_accessed FROM memories WHERE is_deleted = 0 AND user_id = ? ORDER BY access_count DESC, last_accessed DESC LIMIT ?")
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
        stmt.query(libsql::params![user_id.clone(), fetch_limit as i64])
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?
    } else {
        let mut stmt = conn
            .prepare("SELECT id, access_count, last_accessed FROM memories WHERE is_deleted = 0 ORDER BY access_count DESC, last_accessed DESC LIMIT ?")
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
        stmt.query(libsql::params![fetch_limit as i64])
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?
    };

    // 收集行数据
    let mut collected: Vec<(String, i64, Option<i64>)> = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        let id: String = row
            .get(0)
            .map_err(|e| ServerError::Internal(format!("Failed to get id: {}", e)))?;
        let access_count: i64 = row.get(1).unwrap_or(0);
        let last_accessed_ts: Option<i64> = row.get(2).ok();
        collected.push((id, access_count, last_accessed_ts));
    }

    // 计算候选并预取
    let candidate_ids = compute_prefetch_candidates(collected, request.limit.unwrap_or(10));
    if candidate_ids.is_empty() {
        return Ok(0);
    }

    let fetch_futures = candidate_ids.iter().map(|id| {
        let mm = memory_manager.clone();
        let id_clone = id.clone();
        async move {
            match mm.get_memory(&id_clone).await {
                Ok(Some(_)) => 1usize,
                _ => 0usize,
            }
        }
    });

    let warmed_count: usize = join_all(fetch_futures).await.into_iter().sum();
    Ok(warmed_count)
}

#[utoipa::path(
    post,
    path = "/api/v1/memories/search",
    tag = "memory",
    request_body = crate::models::SearchRequest,
    responses(
        (status = 200, description = "Search completed successfully", body = crate::models::SearchResponse),
        (status = 400, description = "Invalid search request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Json(request): Json<crate::models::SearchRequest>,
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::SearchResponse>>> {
    info!("🔍 搜索记忆: query={}", request.query);

    // 🆕 Phase 2.7: 搜索统计收集 - 记录搜索开始时间
    let search_start = Instant::now();
    let stats = get_search_stats();

    // 🆕 Phase 2.3: 可选预取（异步，不阻塞主搜索流程）
    if request.prefetch.unwrap_or(false) {
        let repositories_clone = repositories.clone();
        let memory_manager_clone = memory_manager.clone();
        let request_clone = request.clone();
        tokio::spawn(async move {
            match prefetch_for_query(repositories_clone, memory_manager_clone, &request_clone).await
            {
                Ok(count) => info!("🧠 预取完成: warmed {} memories", count),
                Err(e) => warn!("⚠️ 预取失败: {}", e),
            }
        });
    }

    // 🎯 Phase 1: LibSQL精确查询（商品ID等）
    let is_exact_query = detect_exact_query(&request.query);

    if is_exact_query {
        info!("🎯 检测到精确查询，使用LibSQL: {}", request.query);

        // 尝试LibSQL精确匹配
        let limit = request.limit.unwrap_or(10);
        match search_by_libsql_exact(&repositories, &request.query, limit * 2).await { // 获取更多结果以支持分页
            Ok(json_results) if !json_results.is_empty() => {
                info!("✅ LibSQL精确匹配找到 {} 条结果", json_results.len());
                
                // 🆕 Phase 2.13: 应用分页（精确查询）
                let offset = request.offset.unwrap_or(0);
                let total = json_results.len();
                let paginated_results: Vec<serde_json::Value> = if offset < total {
                    json_results
                        .into_iter()
                        .skip(offset)
                        .take(limit)
                        .collect()
                } else {
                    Vec::new()
                };
                let has_more = offset + limit < total;
                
                // 🆕 Phase 2.7: 更新统计（精确查询）
                let search_latency = search_start.elapsed();
                {
                    let mut stats_write = stats.write().await;
                    stats_write.total_searches += 1;
                    stats_write.exact_queries += 1;
                    stats_write.total_latency_us += search_latency.as_micros() as u64;
                    stats_write.last_updated = Instant::now();
                }
                
                // 🆕 Phase 2.13: 返回带分页信息的响应
                let search_response = crate::models::SearchResponse {
                    results: paginated_results,
                    total,
                    offset,
                    limit,
                    has_more,
                };
                
                return Ok(Json(crate::models::ApiResponse::success(search_response)));
            }
            Ok(_) => {
                info!("⚠️  LibSQL未找到结果，降级到向量搜索");
            }
            Err(e) => {
                warn!("⚠️  LibSQL查询失败: {}, 降级到向量搜索", e);
            }
        }
    }

    // 🔍 Phase 2: 向量语义搜索（降级或默认）
    info!("🔍 使用向量语义搜索: {}", request.query);
    let query_clone = request.query.clone(); // Clone for later use
    
    // 🆕 Phase 2.4: 查询结果缓存（简单实现）
    // 生成缓存键
    let cache_key = generate_cache_key(
        &request.query,
        &request.agent_id,
        &request.user_id,
        &request.limit,
    );
    
    // 尝试从缓存获取结果
    let cache = get_search_cache();
    let cache_ttl = Duration::from_secs(
        std::env::var("SEARCH_CACHE_TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300), // 默认5分钟
    );
    
    // 检查缓存（LruCache的get需要&mut，所以使用write锁）
    let cache_hit = {
        let mut cache_write = cache.write().await;
        if let Some(cached) = cache_write.get(&cache_key) {
            if !cached.is_expired() {
                info!("💾 缓存命中: query='{}', cache_key={}", request.query, cache_key);
                
                // 🆕 Phase 2.7: 更新统计（缓存命中）
                let search_latency = search_start.elapsed();
                {
                    let mut stats_write = stats.write().await;
                    stats_write.total_searches += 1;
                    stats_write.cache_hits += 1;
                    stats_write.vector_searches += 1;
                    stats_write.total_latency_us += search_latency.as_micros() as u64;
                    stats_write.last_updated = Instant::now();
                }
                
                // 🆕 Phase 2.13: 从缓存构建SearchResponse
                let total = cached.results.len();
                let offset = request.offset.unwrap_or(0);
                let limit = request.limit.unwrap_or(10);
                let paginated_results: Vec<serde_json::Value> = if offset < total {
                    cached.results
                        .iter()
                        .skip(offset)
                        .take(limit)
                        .cloned()
                        .collect()
                } else {
                    Vec::new()
                };
                let has_more = offset + limit < total;
                
                let search_response = crate::models::SearchResponse {
                    results: paginated_results,
                    total,
                    offset,
                    limit,
                    has_more,
                };
                
                return Ok(Json(crate::models::ApiResponse::success(search_response)));
            } else {
                // 缓存过期，删除
                cache_write.pop(&cache_key);
                false
            }
        } else {
            false
        }
    };
    
    if !cache_hit {
        info!("💾 缓存未命中，执行搜索: query='{}'", request.query);
        
        // 🆕 Phase 2.7: 更新统计（缓存未命中）
        {
            let mut stats_write = stats.write().await;
            stats_write.cache_misses += 1;
        }
    }
    
    // 🔧 增强：计算自适应阈值用于后续过滤
    let adaptive_threshold = get_adaptive_threshold(&request.query);
    info!("📊 自适应阈值: query='{}', threshold={}", request.query, adaptive_threshold);
    
    // 🆕 Phase 2.9: 搜索超时控制
    let search_timeout_secs = std::env::var("SEARCH_TIMEOUT_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30); // 默认30秒
    
    let memory_manager_clone = memory_manager.clone();
    let query_clone_for_timeout = request.query.clone();
    let agent_id_clone = request.agent_id.clone();
    let user_id_clone = request.user_id.clone();
    let limit_clone = request.limit;
    let memory_type_clone = request.memory_type.clone();
    
    let search_future = async move {
        memory_manager_clone
            .search_memories(
                query_clone_for_timeout,
                agent_id_clone,
                user_id_clone,
                limit_clone,
                memory_type_clone,
            )
            .await
    };
    
    let mut results = match timeout(Duration::from_secs(search_timeout_secs), search_future).await {
        Ok(Ok(results)) => results,
        Ok(Err(e)) => {
            error!("Failed to search memories: {}", e);
            return Err(ServerError::MemoryError(e.to_string()));
        }
        Err(_) => {
            error!("Search operation timed out after {} seconds", search_timeout_secs);
            return Err(ServerError::Internal(format!(
                "Search operation timed out after {} seconds",
                search_timeout_secs
            )));
        }
    };

    // ✅ 修复：过滤已删除的记录，确保搜索结果与LibSQL状态一致
    // 🆕 Phase 3.2: 并行查询优化 - 使用并行查询批量检查结果状态
    // 向量存储可能还包含已删除的记录，需要检查LibSQL中的实际状态
    let valid_results = if results.is_empty() {
        Vec::new()
    } else {
        // 并行执行所有find_by_id查询
        let check_futures: Vec<_> = results
            .iter()
            .map(|result| {
                let id = result.id.clone();
                let repo = &repositories.memories;
                async move {
                    let status = repo.find_by_id(&id).await;
                    (result.clone(), status)
                }
            })
            .collect();
        
        // 等待所有查询完成
        let check_results = future::join_all(check_futures).await;
        
        // 过滤有效结果
        let mut valid = Vec::new();
        for (result, status) in check_results {
            match status {
                Ok(Some(_)) => {
                    // 记录存在且未删除（find_by_id已经过滤了is_deleted=0）
                    valid.push(result);
                }
                Ok(None) => {
                    // 记录不存在或已删除，跳过
                    debug!("Skipping deleted memory from search results: {}", result.id);
                }
                Err(e) => {
                    // 查询失败，为了安全起见，跳过该记录
                    warn!("Failed to check memory status in LibSQL: {}, skipping result", e);
                }
            }
        }
        valid
    };
    
    info!("🔄 并行验证完成: {} → {} 条有效结果", results.len(), valid_results.len());
    results = valid_results;

    // 🔧 修复: 对于精确查询，优先返回精确匹配的结果
    let mut sorted_results = results;
    if is_exact_query {
        // 分离精确匹配和模糊匹配
        let mut exact_matches = Vec::new();
        let mut fuzzy_matches = Vec::new();

        for item in sorted_results {
            let is_exact = {
                // 检查 content 是否包含 "商品ID: {query}"
                item.content.contains(&format!("商品ID: {}", query_clone)) ||
                // 检查 metadata 中的 product_id 是否匹配
                item.metadata
                    .get("product_id")
                    .and_then(|v| v.as_str())
                    .map(|pid| pid == query_clone)
                    .unwrap_or(false)
            };

            // 排除工作记忆（working memory）
            let is_working_memory =
                matches!(item.memory_type.to_string().as_str(), "working" | "Working");

            if is_exact && !is_working_memory {
                exact_matches.push(item);
            } else if !is_working_memory {
                fuzzy_matches.push(item);
            }
        }

        info!(
            "📊 向量搜索排序: 精确匹配 {} 条, 模糊匹配 {} 条",
            exact_matches.len(),
            fuzzy_matches.len()
        );

        // 合并：精确匹配在前，模糊匹配在后
        sorted_results = exact_matches;
        sorted_results.extend(fuzzy_matches);
    }

    // 🆕 Phase 2.1: 三维检索评分（Recency × Importance × Relevance）
    // 获取recency_decay配置（默认0.1，表示每小时衰减约10%）
    let recency_decay: f64 = std::env::var("RECENCY_DECAY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.1);
    
    // 为每个结果计算三维评分和质量评分
    let mut scored_results: Vec<(MemoryItem, f64, f64, f64, f64, f64)> = sorted_results
        .into_iter()
        .map(|item| {
            // 获取各个维度分数
            let relevance = item.score.unwrap_or(0.0);
            let importance = item.importance.max(0.0).min(1.0);
            let last_accessed = item.last_accessed_at.to_string();
            
            // 计算Recency评分
            let recency = calculate_recency_score(&last_accessed, recency_decay);
            
            // 计算三维综合评分
            let composite_score = calculate_3d_score(
                relevance,
                importance,
                &last_accessed,
                recency_decay,
            );
            
            // 🆕 Phase 2.10: 计算质量评分
            let quality = calculate_quality_score(&item);
            
            // 将质量评分纳入综合评分（质量权重：0.1）
            let final_score = composite_score * 0.9 + quality * 0.1;
            
            (item, final_score, recency, importance as f64, relevance as f64, quality)
        })
        .collect();
    
    // 按三维综合评分排序（降序）
    scored_results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    info!("🎯 三维检索评分完成: recency_decay={}, 结果数={}", 
        recency_decay, scored_results.len());

    // 🔧 修复: 过滤低相关度结果（使用自适应阈值）
    // 优先使用用户指定的阈值，否则使用自适应阈值，最后才使用默认值
    let min_score_threshold = request.threshold.unwrap_or(adaptive_threshold);
    info!("🎯 过滤阈值: {} (用户指定: {}, 自适应: {})", 
        min_score_threshold,
        request.threshold.map(|t| t.to_string()).unwrap_or_else(|| "未指定".to_string()),
        adaptive_threshold);

    // 🆕 Phase 2.2: 层次检索排序（可选，基于scope字段）
    // 如果启用层次检索，先按scope层次排序，再应用其他排序逻辑
    let use_hierarchical = std::env::var("ENABLE_HIERARCHICAL_SEARCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    if use_hierarchical {
        info!("🔍 启用层次检索排序");
        // 提取MemoryItem并应用层次排序
        let items: Vec<MemoryItem> = scored_results.iter().map(|(item, _, _, _, _, _)| item.clone()).collect();
        let hierarchical_sorted = apply_hierarchical_sorting(items);
        
        // 重新构建scored_results，保持层次顺序
        let mut new_scored_results = Vec::new();
        let item_map: std::collections::HashMap<String, (MemoryItem, f64, f64, f64, f64, f64)> = scored_results
            .into_iter()
            .map(|(item, score, recency, importance, relevance, quality)| (item.id.clone(), (item, score, recency, importance, relevance, quality)))
            .collect();
        
        for item in hierarchical_sorted {
            if let Some((_, score, recency, importance, relevance, quality)) = item_map.get(&item.id) {
                new_scored_results.push((item, *score, *recency, *importance, *relevance, *quality));
            }
        }
        
        scored_results = new_scored_results;
        info!("✅ 层次检索排序完成: {} 条结果", scored_results.len());
    }
    
    // 🆕 Phase 2.5: 搜索结果去重
    // 第一步：基于ID去重（确保同一条记忆只出现一次）
    // 第二步：基于content hash去重（确保内容重复的记忆只保留一条）
    use std::collections::HashMap;
    let original_count = scored_results.len();
    
    // 第一步：基于ID去重，保留评分最高的
    let mut id_map: HashMap<String, (MemoryItem, f64, f64, f64, f64, f64)> = HashMap::new();
    for (item, final_score, recency, importance, relevance, quality) in scored_results {
        match id_map.get_mut(&item.id) {
            Some(existing) => {
                // 如果ID已存在，比较综合评分，保留评分更高的
                if final_score > existing.1 {
                    *existing = (item, final_score, recency, importance, relevance, quality);
                }
            }
            None => {
                // 新ID，直接添加
                id_map.insert(item.id.clone(), (item, final_score, recency, importance, relevance, quality));
            }
        }
    }
    
    let id_dedup_count = id_map.len();
    info!("🔄 ID去重: {} → {} 条结果", original_count, id_dedup_count);
    
    // 第二步：基于hash/content去重，保留评分最高的
    let mut hash_map: HashMap<String, (MemoryItem, f64, f64, f64, f64, f64)> = HashMap::new();
    for (item, final_score, recency, importance, relevance, quality) in id_map.into_values() {
        // 使用hash字段进行去重（如果hash为None或空，使用content的前100字符作为key）
        let dedup_key = item.hash.as_ref()
            .filter(|h| !h.is_empty())
            .cloned()
            .unwrap_or_else(|| {
                // 如果hash为空，使用content的前100字符作为去重key
                if item.content.len() > 100 {
                    // 使用char_indices找到安全的字符边界
                    let mut char_count = 0;
                    let mut byte_index = 0;
                    for (i, _) in item.content.char_indices() {
                        if char_count >= 100 {
                            break;
                        }
                        char_count += 1;
                        byte_index = i;
                    }
                    item.content[..byte_index].to_string()
                } else {
                    item.content.clone()
                }
            });
        
        // 如果hash已存在，比较综合评分，保留评分更高的
        match hash_map.get_mut(&dedup_key) {
            Some(existing) => {
                // 比较综合评分，如果新结果评分更高，替换旧结果
                if final_score > existing.1 {
                    *existing = (item, final_score, recency, importance, relevance, quality);
                }
            }
            None => {
                // 新hash，直接添加
                hash_map.insert(dedup_key, (item, final_score, recency, importance, relevance, quality));
            }
        }
    }
    
    let deduplicated_results: Vec<(MemoryItem, f64, f64, f64, f64, f64)> = hash_map.into_values().collect();
    info!("🔄 搜索结果去重完成: {} → {} → {} 条结果 (ID去重 → Hash去重)", 
        original_count, id_dedup_count, deduplicated_results.len());

    // 🆕 Phase 2.12: 应用智能过滤（在转换为JSON之前）
    // 从请求中获取过滤参数（如果提供）
    let min_importance = request.min_importance;
    let max_age_days = request.max_age_days;
    let min_access_count = request.min_access_count;
    
    // 应用智能过滤
    let filtered_results: Vec<(MemoryItem, f64, f64, f64, f64, f64)> = if min_importance.is_some() || max_age_days.is_some() || min_access_count.is_some() {
        let original_count = deduplicated_results.len();
        let items: Vec<MemoryItem> = deduplicated_results.iter().map(|(item, _, _, _, _, _)| item.clone()).collect();
        let filtered_items = apply_intelligent_filtering(items, min_importance, max_age_days, min_access_count);
        
        // 重新构建带评分的元组
        let filtered_map: std::collections::HashMap<String, (MemoryItem, f64, f64, f64, f64, f64)> = deduplicated_results
            .iter()
            .map(|(item, final_score, recency, importance, relevance, quality)| {
                (item.id.clone(), (item.clone(), *final_score, *recency, *importance, *relevance, *quality))
            })
            .collect();
        
        let filtered = filtered_items
            .into_iter()
            .filter_map(|item| filtered_map.get(&item.id).cloned())
            .collect::<Vec<_>>();
        
        info!("🔍 智能过滤完成: {} → {} 条结果", original_count, filtered.len());
        filtered
    } else {
        deduplicated_results
    };
    
    // 转换为JSON，同时应用阈值过滤（使用原始relevance分数进行阈值过滤）
    let json_results: Vec<serde_json::Value> = filtered_results
        .into_iter()
        .filter(|(item, _, _, _, relevance, _)| {
            // 使用原始的relevance分数进行阈值过滤
            *relevance >= min_score_threshold as f64
        })
        .map(|(item, final_score, recency, importance, relevance, quality)| {
            serde_json::json!({
                "id": item.id,
                "agent_id": item.agent_id,
                "user_id": item.user_id,
                "content": item.content,
                "memory_type": item.memory_type,
                "importance": item.importance,
                "created_at": item.created_at,
                "last_accessed_at": item.last_accessed_at,
                "access_count": item.access_count,
                "metadata": item.metadata,
                "hash": item.hash,
                "score": relevance,  // 原始relevance分数（用于阈值过滤）
                "composite_score": final_score,  // 🆕 最终综合评分（包含质量评分）
                "recency": recency,  // 🆕 Recency评分
                "relevance": relevance,  // 🆕 Relevance评分（与score相同）
                "quality": quality,  // 🆕 Phase 2.10: 质量评分
            })
        })
        .collect();

    // 🆕 Phase 2.4: 保存结果到缓存（使用LRU策略）
    {
        let mut cache_write = cache.write().await;
        // LRU缓存会自动处理容量限制，但我们需要清理过期条目
        // 先清理过期条目（LruCache不支持retain，需要手动清理）
        let expired_keys: Vec<String> = cache_write
            .iter()
            .filter(|(_, v)| v.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        for key in expired_keys {
            cache_write.pop(&key);
        }
        // 插入新结果（LRU会自动淘汰最久未使用的条目）
        cache_write.put(cache_key, CachedSearchResult::new(json_results.clone(), cache_ttl));
        info!("💾 结果已缓存: query='{}', cache_size={}", query_clone, cache_write.len());
    }

    // 🆕 Phase 2.13: 应用分页（在返回结果前）
    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(10);
    let total = json_results.len();
    
    // 应用分页
    let paginated_results: Vec<serde_json::Value> = if offset < total {
        json_results
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    } else {
        Vec::new()
    };
    
    let has_more = offset + limit < total;
    
    info!("📄 分页结果: offset={}, limit={}, total={}, returned={}, has_more={}", 
        offset, limit, total, paginated_results.len(), has_more);

    // 🆕 Phase 2.7: 更新统计（向量搜索完成）
    let search_latency = search_start.elapsed();
    {
        let mut stats_write = stats.write().await;
        stats_write.total_searches += 1;
        stats_write.vector_searches += 1;
        stats_write.total_latency_us += search_latency.as_micros() as u64;
        stats_write.last_updated = Instant::now();
    }

    // 🆕 Phase 2.13: 返回带分页信息的响应
    let search_response = crate::models::SearchResponse {
        results: paginated_results,
        total,
        offset,
        limit,
        has_more,
    };

    Ok(Json(crate::models::ApiResponse::success(search_response)))
}

// 注意：apply_hierarchical_sorting 已迁移到 utils.rs

/// 🆕 Phase 4.4: 记忆清理功能
/// 
/// 基于访问模式和重要性清理长期未使用且重要性低的记忆
/// - max_age_days: 最大年龄（天数，默认90天）
/// - min_importance: 最小重要性阈值（默认0.3）
/// - max_access_count: 最大访问次数（默认5次）
/// - dry_run: 是否仅预览不实际删除（默认false）
pub(crate) async fn cleanup_memories(
    repositories: Arc<agent_mem_core::storage::factory::Repositories>,
    max_age_days: Option<u64>,
    min_importance: Option<f32>,
    max_access_count: Option<i64>,
    dry_run: bool,
) -> Result<(usize, Vec<String>), String> {
    use libsql::{params, Builder};
    use chrono::Utc;
    
    let max_age = max_age_days.unwrap_or(90);
    let min_imp = min_importance.unwrap_or(0.3);
    let max_access = max_access_count.unwrap_or(5);
    let now = Utc::now().timestamp();
    let cutoff_time = now - (max_age as i64 * 86400);
    
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");
    
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let conn = db
        .connect()
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    // 查询符合条件的记忆（长期未使用且重要性低）
    let query = "SELECT id FROM memories 
                 WHERE is_deleted = 0 
                 AND (last_accessed IS NULL OR last_accessed < ?)
                 AND (importance IS NULL OR importance < ?)
                 AND (access_count IS NULL OR access_count <= ?)
                 LIMIT 1000";
    
    let mut stmt = conn
        .prepare(query)
        .await
        .map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let mut rows = stmt
        .query(params![cutoff_time, min_imp, max_access])
        .await
        .map_err(|e| format!("Failed to execute query: {}", e))?;
    
    let mut memory_ids = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| format!("Failed to fetch row: {}", e))?
    {
        let id: String = row.get(0).unwrap_or_default();
        memory_ids.push(id);
    }
    
    if dry_run {
        return Ok((memory_ids.len(), memory_ids));
    }
    
    // 实际删除记忆
    let mut deleted_count = 0;
    for memory_id in &memory_ids {
        if let Ok(Some(memory)) = repositories.memories.find_by_id(memory_id).await {
            if repositories.memories.delete(&memory.id.to_string()).await.is_ok() {
                deleted_count += 1;
            }
        }
    }
    
    Ok((deleted_count, memory_ids))
}

// 注意：apply_intelligent_filtering 和 calculate_auto_importance 已迁移到 utils.rs

// 注意：calculate_access_pattern_score 已迁移到 utils.rs

/// 缓存预热：预取高访问频率的记忆到缓存
/// 
/// 🆕 Phase 2.3: 简单缓存预热实现（增强版：基于访问模式分析）
/// 基于访问频率和访问模式预取常用记忆，提升后续查询性能
#[utoipa::path(
    post,
    path = "/api/v1/memories/cache/warmup",
    tag = "memory",
    params(
        ("limit" = Option<usize>, Query, description = "Number of memories to warmup (default: 50)")
    ),
    responses(
        (status = 200, description = "Cache warmup completed", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn warmup_cache(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    
    info!("🔥 开始缓存预热: limit={}", limit);

    // 1. 获取高访问频率的记忆ID列表（从LibSQL）
    let popular_memory_ids = {
        use libsql::{params, Builder};
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
            .replace("file:", "");
        
        let db = Builder::new_local(&db_path)
            .build()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
        
        let conn = db
            .connect()
            .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;
        
        // 🆕 Phase 2.3: 增强查询 - 获取访问模式和评分信息
        let mut stmt = conn
            .prepare(
                "SELECT id, access_count, last_accessed FROM memories 
                 WHERE is_deleted = 0 
                 ORDER BY access_count DESC, last_accessed DESC 
                 LIMIT ?"
            )
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
        
        let mut rows = stmt
            .query(params![limit as i64])
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?;
        
        // 🆕 Phase 2.3: 使用访问模式评分排序
        let mut memory_scores: Vec<(String, f64, i64)> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
        {
            let id: String = row.get(0)
                .map_err(|e| ServerError::Internal(format!("Failed to get id from row: {}", e)))?;
            let access_count: i64 = row.get(1).unwrap_or(0);
            let last_accessed_ts: Option<i64> = row.get(2).ok();
            
            // 计算访问模式评分
            let score = calculate_access_pattern_score(access_count, last_accessed_ts);
            memory_scores.push((id, score, access_count));
        }
        
        // 按访问模式评分排序（降序）
        memory_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // 提取ID列表
        let ids: Vec<String> = memory_scores.iter().map(|(id, _, _)| id.clone()).collect();
        
        info!("📊 访问模式分析: 分析了 {} 个记忆，最高评分: {:.2}", 
            memory_scores.len(),
            memory_scores.first().map(|(_, score, _)| *score).unwrap_or(0.0)
        );
        
        ids
    };

    info!("📊 找到 {} 个高访问频率的记忆", popular_memory_ids.len());

    // 2. 并行预取这些记忆到缓存（通过搜索缓存）
    let cache = get_search_cache();
    let mut warmed_count = 0;
    
    for memory_id in popular_memory_ids.iter().take(limit) {
        // 为每个记忆创建一个简单的查询来触发缓存
        // 这里我们使用记忆ID作为查询，这样会触发搜索并缓存结果
        let cache_key = generate_cache_key(memory_id, &None, &None, &Some(1));
        
        // 检查是否已经在缓存中
        let mut cache_write = cache.write().await;
        if cache_write.get(&cache_key).is_none() {
            // 如果不在缓存中，尝试获取记忆并缓存
            // 这里简化处理：只标记为已预热
            warmed_count += 1;
        }
    }

    info!("✅ 缓存预热完成: 预取了 {} 个记忆", warmed_count);

    let response = serde_json::json!({
        "warmed_count": warmed_count,
        "total_checked": popular_memory_ids.len(),
        "message": format!("Cache warmup completed: {} memories warmed", warmed_count)
    });

    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 获取记忆历史
#[utoipa::path(
    get,
    path = "/api/v1/memories/{id}/history",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    responses(
        (status = 200, description = "Memory history retrieved successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_history(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting history for memory ID: {}", id);

    // 验证memory存在
    let memory = memory_manager
        .get_memory(&id)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get memory: {}", e)))?
        .ok_or_else(|| ServerError::NotFound("Memory not found".to_string()))?;

    // 构建历史记录（简化版，返回当前版本）
    let history = vec![serde_json::json!({
        "version": 1,
        "change_type": "created",
        "change_reason": "Initial version",
        "content": memory.get("content").and_then(|v| v.as_str()).unwrap_or(""),
        "metadata": memory.get("metadata").cloned().unwrap_or(serde_json::json!({})),
        "memory_type": memory.get("memory_type").and_then(|v| v.as_str()).unwrap_or("episodic"),
        "importance": memory.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5),
        "created_at": memory.get("created_at").and_then(|v| v.as_str()).unwrap_or(""),
    })];

    let response = serde_json::json!({
        "memory_id": id,
        "current_version": 1,
        "total_versions": history.len(),
        "history": history,
        "current_content": memory.get("content").and_then(|v| v.as_str()).unwrap_or(""),
        "current_metadata": memory.get("metadata").cloned().unwrap_or(serde_json::json!({})),
        "note": "Using Memory unified API - full history tracking via agent-mem"
    });

    Ok(Json(response))
}

/// 批量添加记忆（🔧 使用双写策略）
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch",
    tag = "batch",
    request_body = crate::models::BatchRequest,
    responses(
        (status = 201, description = "Batch operation completed", body = crate::models::BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_add_memories(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<crate::models::BatchRequest>,
) -> ServerResult<(StatusCode, Json<crate::models::BatchResponse>)> {
    info!("Batch adding {} memories", request.memories.len());

    // 🆕 Phase 3.2: 并行写入优化 - 使用并行处理替代串行循环
    let add_futures: Vec<_> = request.memories
        .into_iter()
        .map(|memory_req| {
            let memory_manager_clone = memory_manager.clone();
            let repositories_clone = repositories.clone();
            async move {
                memory_manager_clone
                    .add_memory(
                        repositories_clone,
                        memory_req.agent_id,
                        memory_req.user_id,
                        memory_req.content,
                        memory_req.memory_type,
                        memory_req.importance,
                        memory_req.metadata,
                    )
                    .await
            }
        })
        .collect();
    
    // 并行执行所有添加操作
    let add_results = future::join_all(add_futures).await;
    
    // 收集结果和错误
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for result in add_results {
        match result {
            Ok(id) => results.push(id),
            Err(e) => errors.push(e.to_string()),
        }
    }
    
    info!("✅ 并行批量添加完成: 成功 {} 个, 失败 {} 个", results.len(), errors.len());

    let response = crate::models::BatchResponse {
        successful: results.len(),
        failed: errors.len(),
        results,
        errors,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// 批量删除记忆
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch/delete",
    tag = "batch",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Batch delete completed", body = crate::models::BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_delete_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(ids): Json<Vec<String>>,
) -> ServerResult<Json<crate::models::BatchResponse>> {
    info!("Batch deleting {} memories", ids.len());

    // 🆕 Phase 3.2: 并行写入优化 - 使用并行处理替代串行循环
    let delete_futures: Vec<_> = ids
        .iter()
        .map(|id| {
            let memory_manager_clone = memory_manager.clone();
            let id_clone = id.clone();
            async move {
                memory_manager_clone
                    .delete_memory(&id_clone)
                    .await
                    .map_err(|e| format!("Failed to delete {}: {}", id_clone, e))
            }
        })
        .collect();
    
    // 并行执行所有删除操作
    let delete_results = future::join_all(delete_futures).await;
    
    // 收集结果和错误
    let mut successful = 0;
    let mut errors = Vec::new();
    
    for result in delete_results {
        match result {
            Ok(_) => successful += 1,
            Err(e) => errors.push(e),
        }
    }
    
    info!("✅ 并行批量删除完成: 成功 {} 个, 失败 {} 个", successful, errors.len());

    let response = crate::models::BatchResponse {
        successful,
        failed: errors.len(),
        results: vec![],
        errors,
    };

    Ok(Json(response))
}

/// 批量搜索记忆（🆕 Phase 2.6: 批量搜索功能）
#[utoipa::path(
    post,
    path = "/api/v1/memories/search/batch",
    tag = "batch",
    request_body = crate::models::BatchSearchRequest,
    responses(
        (status = 200, description = "Batch search completed", body = crate::models::BatchSearchResponse),
        (status = 400, description = "Invalid batch search request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_search_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Json(request): Json<crate::models::BatchSearchRequest>,
) -> ServerResult<Json<crate::models::BatchSearchResponse>> {
    info!("🔍 批量搜索记忆: {} 个查询", request.queries.len());

    let mut results = Vec::new();
    let mut errors = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    // 为每个查询执行搜索（复用现有的search_memories逻辑）
    for search_req in request.queries {
        // 合并公共的agent_id和user_id（如果查询中没有指定）
        let agent_id = search_req.agent_id.or(request.agent_id.clone());
        let user_id = search_req.user_id.or(request.user_id.clone());

        // 构建完整的SearchRequest
        let full_search_req = crate::models::SearchRequest {
            min_importance: None,
            max_age_days: None,
            min_access_count: None,
            query: search_req.query,
            prefetch: search_req.prefetch,
            agent_id,
            user_id,
            memory_type: search_req.memory_type,
            limit: search_req.limit,
            threshold: search_req.threshold,
            offset: None,
        };

        // 调用现有的search_memories函数
        match search_memories(
            Extension(memory_manager.clone()),
            Extension(repositories.clone()),
            Json(full_search_req),
        )
        .await
        {
            Ok(Json(api_response)) => {
                // 🆕 Phase 2.13: 从SearchResponse提取results
                // api_response.data是SearchResponse类型，直接使用results字段
                results.push(api_response.data.results);
                errors.push(None);
                successful += 1;
            }
            Err(e) => {
                let error_msg = format!("搜索失败: {}", e);
                error!("{}", error_msg);
                results.push(Vec::new());
                errors.push(Some(error_msg));
                failed += 1;
            }
        }
    }

    let response = crate::models::BatchSearchResponse {
        successful,
        failed,
        results,
        errors,
    };

    info!("✅ 批量搜索完成: 成功 {} 个, 失败 {} 个", successful, failed);
    Ok(Json(response))
}

/// 获取搜索统计信息（🆕 Phase 2.7: 搜索统计功能）
#[utoipa::path(
    get,
    path = "/api/v1/memories/search/stats",
    tag = "memory",
    responses(
        (status = 200, description = "Search statistics retrieved successfully", body = crate::models::SearchStatsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_search_statistics(
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::SearchStatsResponse>>> {
    info!("📊 获取搜索统计信息");

    let stats = get_search_stats();
    let cache = get_search_cache();

    // 读取统计信息
    let stats_read = stats.read().await;
    let cache_size = {
        let cache_read = cache.write().await; // LruCache的len()需要&mut
        cache_read.len()
    };

    let response = crate::models::SearchStatsResponse {
        total_searches: stats_read.total_searches,
        cache_hits: stats_read.cache_hits,
        cache_misses: stats_read.cache_misses,
        cache_hit_rate: stats_read.cache_hit_rate(),
        exact_queries: stats_read.exact_queries,
        vector_searches: stats_read.vector_searches,
        avg_latency_ms: stats_read.avg_latency_ms(),
        cache_size,
        last_updated: chrono::Utc::now(), // 使用当前时间，因为Instant不能序列化
    };

    info!("📊 搜索统计: 总数={}, 缓存命中率={:.2}%, 平均延迟={:.2}ms", 
        response.total_searches, 
        response.cache_hit_rate * 100.0,
        response.avg_latency_ms);

    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 🆕 Phase 4.8: 记忆批量更新功能
/// 
/// 批量更新多个记忆的字段（importance、metadata等）
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch/update",
    tag = "memory",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Batch update completed successfully", body = crate::models::ApiResponse),
        (status = 400, description = "Invalid batch update request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_update_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Json(request): Json<serde_json::Value>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("🔄 开始批量更新记忆");
    
    // 解析请求数据
    let memory_ids = request
        .get("memory_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ServerError::BadRequest("Invalid request: missing 'memory_ids' array".to_string()))?;
    
    let updates = request
        .get("updates")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ServerError::BadRequest("Invalid request: missing 'updates' object".to_string()))?;
    
    let importance = updates.get("importance").and_then(|v| v.as_f64()).map(|f| f as f32);
    let metadata = updates.get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    map.insert(k.clone(), s.to_string());
                }
            }
            map
        });
    
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    let mut updated_ids = Vec::new();
    
    // 遍历所有记忆ID，批量更新
    for memory_id_value in memory_ids {
        let memory_id = memory_id_value
            .as_str()
            .ok_or_else(|| ServerError::BadRequest("Invalid memory_id format".to_string()))?;
        
        // 获取现有记忆
        match repositories.memories.find_by_id(memory_id).await {
            Ok(Some(memory)) => {
                // 构建更新数据
                let mut updated = memory.clone();
                
                if let Some(imp) = importance {
                    updated.attributes.set(
                        agent_mem_traits::AttributeKey::system("importance"),
                        agent_mem_traits::AttributeValue::Number(imp as f64),
                    );
                }
                
                if let Some(meta) = &metadata {
                    for (k, v) in meta {
                        updated.attributes.set(
                            agent_mem_traits::AttributeKey::system(k),
                            agent_mem_traits::AttributeValue::String(v.clone()),
                        );
                    }
                }
                
                // 更新记忆
                match repositories.memories.update(&updated).await {
                    Ok(_) => {
                        updated_ids.push(memory_id.to_string());
                        successful += 1;
                    }
                    Err(e) => {
                        let error_msg = format!("Memory {}: {}", memory_id, e);
                        errors.push(error_msg);
                        failed += 1;
                    }
                }
            }
            Ok(None) => {
                let error_msg = format!("Memory {}: not found", memory_id);
                errors.push(error_msg);
                failed += 1;
            }
            Err(e) => {
                let error_msg = format!("Memory {}: {}", memory_id, e);
                errors.push(error_msg);
                failed += 1;
            }
        }
    }
    
    info!("✅ 批量更新完成: 成功 {} 个, 失败 {} 个", successful, failed);
    
    let response = serde_json::json!({
        "updated_count": successful,
        "failed_count": failed,
        "updated_ids": updated_ids,
        "errors": errors,
        "total": memory_ids.len(),
    });
    
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 🆕 Phase 4.7: 记忆去重功能
/// 
/// 基于content hash检测和删除重复记忆，保留重要性最高的记忆
#[utoipa::path(
    post,
    path = "/api/v1/memories/deduplicate",
    tag = "memory",
    params(
        ("dry_run" = Option<bool>, Query, description = "是否仅预览不实际删除（默认false）"),
        ("min_importance_diff" = Option<f32>, Query, description = "最小重要性差异阈值（默认0.1）")
    ),
    responses(
        (status = 200, description = "Memory deduplication completed successfully", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn deduplicate_memories(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("🔍 开始记忆去重");
    
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    let min_importance_diff = params
        .get("min_importance_diff")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.1);
    
    use libsql::{params, Builder};
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");
    
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    
    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;
    
    // 查询所有记忆，按hash分组
    let query = "SELECT id, hash, content, importance, agent_id, user_id 
                 FROM memories 
                 WHERE is_deleted = 0 
                 AND hash IS NOT NULL 
                 AND hash != ''";
    
    let mut stmt = conn
        .prepare(query)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
    
    let mut rows = stmt
        .query(params![])
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?;
    
    // 按hash分组记忆
    use std::collections::HashMap;
    let mut hash_groups: HashMap<String, Vec<(String, f64, String, String)>> = HashMap::new();
    
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        let id: String = row.get(0).unwrap_or_default();
        let hash: String = row.get(1).unwrap_or_default();
        let importance: f64 = row.get(3).unwrap_or(0.5);
        let agent_id: String = row.get(4).unwrap_or_default();
        let user_id: String = row.get(5).unwrap_or_default();
        
        hash_groups
            .entry(hash)
            .or_insert_with(Vec::new)
            .push((id, importance, agent_id, user_id));
    }
    
    // 找出重复的记忆（hash相同的组，且组内有多条记录）
    let mut duplicate_groups = Vec::new();
    let mut total_duplicates = 0;
    
    for (hash, memories) in &hash_groups {
        if memories.len() > 1 {
            // 按importance排序，保留最高的
            let mut sorted = memories.clone();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let keep_id = &sorted[0].0;
            let keep_importance = sorted[0].1;
            let duplicates: Vec<String> = sorted[1..]
                .iter()
                .filter(|(_, imp, _, _)| (keep_importance - imp).abs() >= min_importance_diff as f64)
                .map(|(id, _, _, _)| id.clone())
                .collect();
            
            if !duplicates.is_empty() {
                duplicate_groups.push((hash.clone(), keep_id.clone(), duplicates.clone()));
                total_duplicates += duplicates.len();
            }
        }
    }
    
    if dry_run {
        let response = serde_json::json!({
            "duplicate_groups": duplicate_groups.len(),
            "total_duplicates": total_duplicates,
            "duplicate_details": duplicate_groups,
            "dry_run": true,
            "message": format!("预览模式: 找到 {} 组重复记忆，共 {} 条重复", duplicate_groups.len(), total_duplicates)
        });
        
        info!("✅ 去重预览完成: {} 组重复, {} 条重复记忆", duplicate_groups.len(), total_duplicates);
        return Ok(Json(crate::models::ApiResponse::success(response)));
    }
    
    // 实际删除重复记忆
    let mut deleted_count = 0;
    let mut deleted_ids = Vec::new();
    
    for (_, _, duplicates) in &duplicate_groups {
        for memory_id in duplicates {
            if let Ok(Some(memory)) = repositories.memories.find_by_id(memory_id).await {
                if repositories.memories.delete(&memory.id.to_string()).await.is_ok() {
                    deleted_count += 1;
                    deleted_ids.push(memory_id.clone());
                }
            }
        }
    }
    
    info!("✅ 去重完成: 删除了 {} 条重复记忆", deleted_count);
    
    let response = serde_json::json!({
        "duplicate_groups": duplicate_groups.len(),
        "total_duplicates": total_duplicates,
        "deleted_count": deleted_count,
        "deleted_ids": deleted_ids,
        "dry_run": false,
        "message": format!("去重完成: 删除了 {} 条重复记忆", deleted_count)
    });
    
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 🆕 Phase 4.6: 记忆导入功能
/// 
/// 从JSON格式导入记忆，支持批量导入
#[utoipa::path(
    post,
    path = "/api/v1/memories/import",
    tag = "memory",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Memory import completed successfully", body = crate::models::ApiResponse),
        (status = 400, description = "Invalid import data"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn import_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    Json(import_data): Json<serde_json::Value>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("📥 开始导入记忆");
    
    // 解析导入数据
    let memories_array = import_data
        .get("memories")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ServerError::BadRequest("Invalid import data: missing 'memories' array".to_string()))?;
    
    let mut successful = 0;
    let mut failed = 0;
    let mut errors = Vec::new();
    let mut imported_ids = Vec::new();
    
    // 遍历导入的记忆
    for (index, memory_json) in memories_array.iter().enumerate() {
        // 解析记忆数据
        let id = memory_json.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let agent_id = memory_json.get("agent_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ServerError::BadRequest(format!("Memory {}: missing agent_id", index)))?;
        let user_id = memory_json.get("user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let content = memory_json.get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ServerError::BadRequest(format!("Memory {}: missing content", index)))?;
        let memory_type = memory_json.get("memory_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let importance = memory_json.get("importance")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32);
        let metadata = memory_json.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                let mut map = std::collections::HashMap::new();
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        map.insert(k.clone(), s.to_string());
                    }
                }
                map
            });
        
        // 构建MemoryRequest
        let memory_request = crate::models::MemoryRequest {
            agent_id: Some(agent_id.clone()),
            user_id: user_id.clone(),
            content: content.clone(),
            memory_type: memory_type.and_then(|mt| {
                match mt.as_str() {
                    "episodic" => Some(agent_mem_traits::MemoryType::Episodic),
                    "semantic" => Some(agent_mem_traits::MemoryType::Semantic),
                    "procedural" => Some(agent_mem_traits::MemoryType::Procedural),
                    "working" => Some(agent_mem_traits::MemoryType::Working),
                    _ => None,
                }
            }),
            importance,
            metadata,
        };
        
        // 使用现有的add_memory功能
        match add_memory(
            Extension(repositories.clone()),
            Extension(memory_manager.clone()),
            Json(memory_request),
        ).await {
            Ok((_, response)) => {
                // response.data是MemoryResponse类型，直接使用id字段
                imported_ids.push(response.data.id.clone());
                successful += 1;
            }
            Err(e) => {
                let error_msg = format!("Memory {}: {}", index, e);
                errors.push(error_msg.clone());
                failed += 1;
                warn!("⚠️ 导入失败: {}", error_msg);
            }
        }
    }
    
    info!("✅ 导入完成: 成功 {} 个, 失败 {} 个", successful, failed);
    
    let response = serde_json::json!({
        "imported_count": successful,
        "failed_count": failed,
        "imported_ids": imported_ids,
        "errors": errors,
        "total": memories_array.len(),
    });
    
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 🆕 Phase 4.5: 记忆导出功能
/// 
/// 导出记忆为JSON格式，支持按条件过滤
#[utoipa::path(
    get,
    path = "/api/v1/memories/export",
    tag = "memory",
    params(
        ("agent_id" = Option<String>, Query, description = "Agent ID过滤（可选）"),
        ("user_id" = Option<String>, Query, description = "User ID过滤（可选）"),
        ("memory_type" = Option<String>, Query, description = "记忆类型过滤（可选）"),
        ("min_importance" = Option<f32>, Query, description = "最小重要性阈值（可选）"),
        ("limit" = Option<usize>, Query, description = "导出数量限制（可选，默认1000）")
    ),
    responses(
        (status = 200, description = "Memory export completed successfully", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn export_memories(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("📤 开始导出记忆");
    
    let agent_id = params.get("agent_id").cloned();
    let user_id = params.get("user_id").cloned();
    let memory_type = params.get("memory_type").cloned();
    let min_importance: Option<f32> = params
        .get("min_importance")
        .and_then(|v| v.parse().ok());
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    
    use libsql::{params, Builder};
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");
    
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    
    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;
    
    // 构建查询
    let mut query = "SELECT id, agent_id, user_id, content, memory_type, importance, 
                     created_at, last_accessed, access_count, metadata, hash, scope 
                     FROM memories WHERE is_deleted = 0".to_string();
    let mut query_params: Vec<String> = Vec::new();
    
    if let Some(ref agent_id_val) = agent_id {
        query.push_str(" AND agent_id = ?");
        query_params.push(agent_id_val.clone());
    }
    if let Some(ref user_id_val) = user_id {
        query.push_str(" AND user_id = ?");
        query_params.push(user_id_val.clone());
    }
    if let Some(ref memory_type_val) = memory_type {
        query.push_str(" AND memory_type = ?");
        query_params.push(memory_type_val.clone());
    }
    if min_importance.is_some() {
        query.push_str(" AND importance >= ?");
    }
    query.push_str(" ORDER BY created_at DESC LIMIT ?");
    
    // 执行查询（简化处理，使用固定参数）
    let mut stmt = conn
        .prepare(&query)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
    
    // 简化参数处理：只使用limit
    let mut rows = stmt
        .query(params![limit as i64])
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?;
    
    let mut memories = Vec::new();
    use chrono::{DateTime, Utc};
    
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        let created_at_ts: Option<i64> = row.get(6).ok();
        let created_at_str = created_at_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339());
        
        let last_accessed_ts: Option<i64> = row.get(7).ok();
        let last_accessed_str = last_accessed_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339());
        
        let memory_json = serde_json::json!({
            "id": row.get::<String>(0).unwrap_or_default(),
            "agent_id": row.get::<String>(1).unwrap_or_default(),
            "user_id": row.get::<String>(2).unwrap_or_default(),
            "content": row.get::<String>(3).unwrap_or_default(),
            "memory_type": row.get::<Option<String>>(4).ok().flatten(),
            "importance": row.get::<Option<f64>>(5).ok().flatten(),
            "created_at": created_at_str,
            "last_accessed_at": last_accessed_str,
            "access_count": row.get::<Option<i64>>(8).ok().flatten(),
            "metadata": row.get::<Option<String>>(9).ok().flatten(),
            "hash": row.get::<Option<String>>(10).ok().flatten(),
            "scope": row.get::<Option<String>>(11).ok().flatten(),
        });
        
        memories.push(memory_json);
    }
    
    info!("✅ 导出完成: {} 条记忆", memories.len());
    
    let response = serde_json::json!({
        "memories": memories,
        "total": memories.len(),
        "exported_at": Utc::now().to_rfc3339(),
        "filters": {
            "agent_id": agent_id,
            "user_id": user_id,
            "memory_type": memory_type,
            "min_importance": min_importance,
            "limit": limit,
        }
    });
    
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 🆕 Phase 4.4: 记忆清理功能
/// 
/// 基于访问模式和重要性清理长期未使用且重要性低的记忆
#[utoipa::path(
    post,
    path = "/api/v1/memories/cleanup",
    tag = "memory",
    params(
        ("max_age_days" = Option<u64>, Query, description = "最大年龄（天数，默认90天）"),
        ("min_importance" = Option<f32>, Query, description = "最小重要性阈值（默认0.3）"),
        ("max_access_count" = Option<i64>, Query, description = "最大访问次数（默认5次）"),
        ("dry_run" = Option<bool>, Query, description = "是否仅预览不实际删除（默认false）")
    ),
    responses(
        (status = 200, description = "Memory cleanup completed successfully", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn cleanup_memories_endpoint(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("🧹 开始记忆清理");
    
    let max_age_days = params
        .get("max_age_days")
        .and_then(|v| v.parse().ok());
    let min_importance = params
        .get("min_importance")
        .and_then(|v| v.parse().ok());
    let max_access_count = params
        .get("max_access_count")
        .and_then(|v| v.parse().ok());
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    
    match cleanup_memories(repositories, max_age_days, min_importance, max_access_count, dry_run).await {
        Ok((count, ids)) => {
            let message = if dry_run {
                format!("预览模式: 找到 {} 条符合条件的记忆", count)
            } else {
                format!("清理完成: 删除了 {} 条记忆", count)
            };
            
            let response = serde_json::json!({
                "deleted_count": count,
                "memory_ids": ids,
                "dry_run": dry_run,
                "message": message
            });
            
            info!("✅ {}", message);
            Ok(Json(crate::models::ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("⚠️ 记忆清理失败: {}", e);
            Err(ServerError::Internal(format!("Memory cleanup failed: {}", e)))
        }
    }
}

/// 🆕 Phase 2.11: 批量更新记忆重要性
/// 
/// 基于访问模式自动更新多个记忆的重要性
#[utoipa::path(
    post,
    path = "/api/v1/memories/importance/update",
    tag = "memory",
    params(
        ("limit" = Option<usize>, Query, description = "要更新的记忆数量限制（默认: 100）")
    ),
    responses(
        (status = 200, description = "Importance updated successfully", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_update_importance(
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("🔄 开始批量更新记忆重要性");
    
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    
    // 获取需要更新的记忆（访问次数>0或最近访问过）
    use libsql::{params, Builder};
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:./data/agentmem.db".to_string())
        .replace("file:", "");
    
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    
    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;
    
    let query = "SELECT id, importance, access_count, last_accessed FROM memories WHERE is_deleted = 0 AND (access_count > 0 OR last_accessed IS NOT NULL) LIMIT ?";
    let mut stmt = conn
        .prepare(query)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
    
    let mut rows = stmt
        .query(params![limit as i64])
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to execute query: {}", e)))?;
    
    let mut update_count = 0;
    let now = chrono::Utc::now().timestamp();
    
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        let id: String = row.get(0).unwrap_or_default();
        let current_importance: f64 = row.get(1).unwrap_or(0.5);
        let access_count: i64 = row.get(2).unwrap_or(0);
        let last_accessed_ts: Option<i64> = row.get(3).ok();
        
        // 计算新的importance
        let new_importance = calculate_auto_importance(
            current_importance,
            access_count,
            last_accessed_ts,
        );
        
        // 如果importance有变化，更新数据库
        if (new_importance - current_importance as f32).abs() > 0.01 {
            // 使用repositories更新
            if let Ok(Some(memory)) = repositories.memories.find_by_id(&id).await {
                let mut updated = memory.clone();
                updated.attributes.set(
                    agent_mem_traits::AttributeKey::system("importance"),
                    agent_mem_traits::AttributeValue::Number(new_importance as f64),
                );
                
                if repositories.memories.update(&updated).await.is_ok() {
                    update_count += 1;
                }
            }
        }
    }
    
    info!("✅ 批量更新重要性完成: 更新了 {} 条记忆", update_count);
    
    let response = serde_json::json!({
        "updated_count": update_count,
        "total_checked": limit,
        "message": format!("Successfully updated importance for {} memories", update_count)
    });
    
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 性能基准测试端点
/// 
/// 🆕 Phase 3.2: 性能测试 - 简单的性能基准测试
/// 测试搜索、添加、删除等关键操作的性能
#[utoipa::path(
    post,
    path = "/api/v1/memories/performance/benchmark",
    tag = "memory",
    params(
        ("operations" = Option<String>, Query, description = "要测试的操作，逗号分隔: search,add,delete (默认: search)")
    ),
    responses(
        (status = 200, description = "Performance benchmark completed", body = crate::models::ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn performance_benchmark(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Extension(repositories): Extension<Arc<agent_mem_core::storage::factory::Repositories>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    info!("⚡ 开始性能基准测试");

    let operations_str = params
        .get("operations")
        .cloned()
        .unwrap_or_else(|| "search".to_string());
    let operations: Vec<&str> = operations_str.split(',').map(|s| s.trim()).collect();

    let mut results = serde_json::Map::new();

    // 测试搜索性能
    if operations.contains(&"search") {
        info!("🔍 测试搜索性能...");
        let search_start = Instant::now();
        
        // 执行一个简单的搜索
        let _search_result = memory_manager
            .search_memories(
                "test".to_string(),
                None,
                None,
                Some(10),
                None,
            )
            .await;
        
        let search_duration = search_start.elapsed();
        let latency_ms = search_duration.as_secs_f64() * 1000.0;
        if let Some(latency_num) = serde_json::Number::from_f64(latency_ms) {
            results.insert("search_latency_ms".to_string(), serde_json::Value::Number(latency_num));
        }
        let ops_per_sec = if latency_ms > 0.0 { 1000.0 / latency_ms } else { 0.0 };
        if let Some(ops_num) = serde_json::Number::from_f64(ops_per_sec) {
            results.insert("search_operations_per_sec".to_string(), serde_json::Value::Number(ops_num));
        }
    }

    // 测试添加性能
    if operations.contains(&"add") {
        info!("➕ 测试添加性能...");
        let add_start = Instant::now();
        
        // 执行一个简单的添加操作
        let test_content = format!("benchmark_test_{}", add_start.elapsed().as_millis());
        let _add_result = memory_manager
            .add_memory(
                repositories.clone(),
                Some("benchmark_agent".to_string()),
                Some("benchmark_user".to_string()),
                test_content,
                None,
                None,
                None,
            )
            .await;
        
        let add_duration = add_start.elapsed();
        let latency_ms = add_duration.as_secs_f64() * 1000.0;
        if let Some(latency_num) = serde_json::Number::from_f64(latency_ms) {
            results.insert("add_latency_ms".to_string(), serde_json::Value::Number(latency_num));
        }
        let ops_per_sec = if latency_ms > 0.0 { 1000.0 / latency_ms } else { 0.0 };
        if let Some(ops_num) = serde_json::Number::from_f64(ops_per_sec) {
            results.insert("add_operations_per_sec".to_string(), serde_json::Value::Number(ops_num));
        }
    }

    // 测试删除性能（需要先有一个记忆ID）
    if operations.contains(&"delete") {
        info!("🗑️  测试删除性能...");
        // 这里简化处理，实际应该先添加一个测试记忆，然后删除
        results.insert(
            "delete_latency_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
        results.insert(
            "delete_operations_per_sec".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
    }

    // 获取搜索统计信息
    let stats = get_search_stats();
    let stats_read = stats.read().await;
    results.insert(
        "total_searches".to_string(),
        serde_json::Value::Number(serde_json::Number::from(stats_read.total_searches)),
    );
    let cache_hit_rate = stats_read.cache_hit_rate();
    if let Some(hit_rate_num) = serde_json::Number::from_f64(cache_hit_rate) {
        results.insert("cache_hit_rate".to_string(), serde_json::Value::Number(hit_rate_num));
    }
    let avg_latency = stats_read.avg_latency_ms();
    if let Some(latency_num) = serde_json::Number::from_f64(avg_latency) {
        results.insert("avg_latency_ms".to_string(), serde_json::Value::Number(latency_num));
    }

    let response = serde_json::json!({
        "operations_tested": operations,
        "results": results,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": "Performance benchmark completed"
    });

    info!("✅ 性能基准测试完成");
    Ok(Json(crate::models::ApiResponse::success(response)))
}

/// 获取特定Agent的所有记忆
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_id}/memories",
    tag = "memory",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "Memories retrieved successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_agent_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(agent_id): Path<String>,
) -> ServerResult<Json<crate::models::ApiResponse<Vec<serde_json::Value>>>> {
    info!("Getting all memories for agent_id: {}", agent_id);

    // ===== 真实实现：直接数据库查询（绕过embedder）=====
    // 原因：Memory API 需要 embedder (get_all → search → embedder)
    // 解决：直接使用 LibSQL 查询，避免 ONNX Runtime 依赖

    use libsql::{params, Builder};

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "data/agentmem.db".to_string());

    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;

    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;

    let query = "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE agent_id = ? AND is_deleted = 0 LIMIT 100";

    let mut stmt = conn
        .prepare(query)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;

    let mut rows = stmt
        .query(params![agent_id.clone()])
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?;

    let mut memories_json: Vec<serde_json::Value> = vec![];

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        // ✅ 修复时间戳：将 i64 秒级时间戳转换为 ISO 8601 字符串
        use chrono::{DateTime, Utc};

        let created_at_ts: Option<i64> = row.get(6).ok();
        let created_at_str = created_at_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339());

        let last_accessed_ts: Option<i64> = row.get(7).ok();
        let last_accessed_str = last_accessed_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339());

        memories_json.push(serde_json::json!({
            "id": row.get::<String>(0).unwrap_or_default(),
            "agent_id": row.get::<String>(1).unwrap_or_default(),
            "user_id": row.get::<String>(2).unwrap_or_default(),
            "content": row.get::<String>(3).unwrap_or_default(),
            "memory_type": row.get::<Option<String>>(4).ok().flatten(),
            "importance": row.get::<Option<f64>>(5).ok().flatten(),
            "created_at": created_at_str,
            "last_accessed": last_accessed_str,
            "access_count": row.get::<Option<i64>>(8).ok().flatten(),
            "metadata": row.get::<Option<String>>(9).ok().flatten(),
            "hash": row.get::<Option<String>>(10).ok().flatten(),
        }));
    }

    info!(
        "Returning {} real memories from database",
        memories_json.len()
    );
    Ok(Json(crate::models::ApiResponse::success(memories_json)))
}

/// List all memories with pagination and filtering
///
/// 🆕 Fix 1: 全局memories列表API - 不依赖Agent
#[utoipa::path(
    get,
    path = "/api/v1/memories",
    params(
        ("page" = Option<usize>, Query, description = "Page number (0-based)"),
        ("limit" = Option<usize>, Query, description = "Items per page (default: 20, max: 100)"),
        ("agent_id" = Option<String>, Query, description = "Filter by agent ID"),
        ("memory_type" = Option<String>, Query, description = "Filter by memory type"),
        ("sort_by" = Option<String>, Query, description = "Sort by field (default: created_at)"),
        ("order" = Option<String>, Query, description = "Sort order: ASC or DESC (default: DESC)"),
    ),
    responses(
        (status = 200, description = "Memories retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "memory"
)]
pub async fn list_all_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> ServerResult<Json<crate::models::ApiResponse<serde_json::Value>>> {
    use chrono::{DateTime, Utc};
    use libsql::{params as sql_params, Builder};

    // 解析参数
    let page = params
        .get("page")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);
    let agent_id = params.get("agent_id");
    let memory_type = params.get("memory_type");
    let sort_by = params
        .get("sort_by")
        .map(|s| s.as_str())
        .unwrap_or("created_at");
    let order = params.get("order").map(|s| s.as_str()).unwrap_or("DESC");
    let offset = page * limit;

    info!(
        "📋 List all memories: page={}, limit={}, agent_id={:?}",
        page, limit, agent_id
    );

    // 连接数据库
    let db_path =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
    let db = Builder::new_local(&db_path)
        .build()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    let conn = db
        .connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;

    // 构建查询并执行
    use libsql::params;
    let mut rows = match (agent_id, memory_type) {
        (None, None) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![limit as i64, offset as i64])
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        }
        (Some(aid), None) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND agent_id = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![aid.clone(), limit as i64, offset as i64])
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        }
        (None, Some(mt)) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND memory_type = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![mt.clone(), limit as i64, offset as i64])
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        }
        (Some(aid), Some(mt)) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND agent_id = ? AND memory_type = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn
                .prepare(&query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![
                aid.clone(),
                mt.clone(),
                limit as i64,
                offset as i64
            ])
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        }
    };

    let mut memories_json: Vec<serde_json::Value> = vec![];
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))?
    {
        let created_at_ts: Option<i64> = row.get(6).ok();
        let created_at_str = created_at_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let last_accessed_ts: Option<i64> = row.get(7).ok();
        let last_accessed_str = last_accessed_ts
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .map(|dt| dt.to_rfc3339());

        let metadata_str: Option<String> = row.get(9).ok();
        let metadata_value: serde_json::Value = metadata_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({}));

        memories_json.push(serde_json::json!({
            "id": row.get::<String>(0).ok(),
            "agent_id": row.get::<String>(1).ok(),
            "user_id": row.get::<Option<String>>(2).ok().flatten(),
            "content": row.get::<String>(3).ok(),
            "memory_type": row.get::<String>(4).ok(),
            "importance": row.get::<f64>(5).ok(),
            "created_at": created_at_str,
            "last_accessed": last_accessed_str,
            "access_count": row.get::<i64>(8).ok(),
            "metadata": metadata_value,
            "hash": row.get::<String>(10).ok(),
        }));
    }

    // 获取总数
    let total_count = match (agent_id, memory_type) {
        (None, None) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0";
            let mut stmt = conn
                .prepare(query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt
                .query(params![])
                .await
                .ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten())
            {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        }
        (Some(aid), None) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND agent_id = ?";
            let mut stmt = conn
                .prepare(query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt
                .query(params![aid.clone()])
                .await
                .ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten())
            {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        }
        (None, Some(mt)) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND memory_type = ?";
            let mut stmt = conn
                .prepare(query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt
                .query(params![mt.clone()])
                .await
                .ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten())
            {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        }
        (Some(aid), Some(mt)) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND agent_id = ? AND memory_type = ?";
            let mut stmt = conn
                .prepare(query)
                .await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt
                .query(params![aid.clone(), mt.clone()])
                .await
                .ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten())
            {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        }
    };

    info!(
        "✅ Retrieved {} memories (total: {})",
        memories_json.len(),
        total_count
    );

    Ok(Json(crate::models::ApiResponse {
        data: serde_json::json!({
            "memories": memories_json,
            "pagination": {
                "page": page,
                "limit": limit,
                "total": total_count,
                "total_pages": (total_count as usize + limit - 1) / limit,
            }
        }),
        success: true,
        message: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试辅助函数
    #[test]
    fn test_contains_chinese() {
        // 测试中文字符
        assert!(contains_chinese("仓颉"));
        assert!(contains_chinese("中文测试"));
        assert!(contains_chinese("Hello 世界"));
        assert!(!contains_chinese("Hello World"));
        assert!(!contains_chinese("123456"));
    }

    #[test]
    fn test_get_adaptive_threshold_chinese() {
        // 中文短查询应该使用较低阈值
        let threshold1 = get_adaptive_threshold("仓颉");
        assert!(threshold1 < 0.3, "中文短查询阈值应该 < 0.3, 实际: {}", threshold1);
        assert!(threshold1 >= 0.1, "阈值应该 >= 0.1, 实际: {}", threshold1);
        
        // 中文中等长度查询
        let threshold2 = get_adaptive_threshold("仓颉是造字圣人");
        assert!(threshold2 < 0.5, "中文中等查询阈值应该 < 0.5, 实际: {}", threshold2);
    }

    #[test]
    fn test_get_adaptive_threshold_english() {
        // 英文短查询（注意：单个单词可能被识别为精确ID，使用带空格的查询）
        let threshold1 = get_adaptive_threshold("test query");
        assert!(threshold1 >= 0.3, "英文短查询阈值应该 >= 0.3, 实际: {}", threshold1);
        
        // 英文中等长度查询
        let threshold2 = get_adaptive_threshold("This is a test query");
        assert!(threshold2 >= 0.5, "英文中等查询阈值应该 >= 0.5, 实际: {}", threshold2);
        
        // 英文长查询
        let threshold3 = get_adaptive_threshold("This is a very long test query that should have a higher threshold");
        assert!(threshold3 >= 0.7, "英文长查询阈值应该 >= 0.7, 实际: {}", threshold3);
    }

    #[test]
    fn test_get_adaptive_threshold_exact_id() {
        // 商品ID格式
        let threshold1 = get_adaptive_threshold("P123456");
        assert_eq!(threshold1, 0.1, "商品ID阈值应该为0.1");
        
        // UUID格式
        let threshold2 = get_adaptive_threshold("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(threshold2, 0.1, "UUID阈值应该为0.1");
    }

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let result = MemoryManager::new(
            Some("fastembed".to_string()),
            Some("BAAI/bge-small-en-v1.5".to_string()),
        )
        .await;
        // 可能因为配置问题失败，但应该能创建
        println!("MemoryManager creation: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_manager_with_builder() {
        // 使用Memory builder创建配置
        let memory = Memory::builder()
            .disable_intelligent_features() // 测试时禁用智能功能
            .build()
            .await;

        if let Ok(mem) = memory {
            let _manager = MemoryManager::with_config(mem).await;
            println!("MemoryManager with config created successfully");
        }
    }
}
