//! Orchestrator Retrieval - 检索模块
//!
//! 负责所有检索相关操作，包括搜索、重排序等

use std::collections::HashMap;
use tracing::{debug, info, warn};

use agent_mem_traits::{MemoryItem, Result};

use super::core::MemoryOrchestrator;
use super::utils::UtilsModule;

/// 检索模块
///
/// 负责所有检索相关操作
pub struct RetrievalModule;

impl RetrievalModule {
    /// 搜索记忆
    pub async fn search_memories(
        orchestrator: &MemoryOrchestrator,
        query: String,
        agent_id: String,
        user_id: Option<String>,
        limit: usize,
        _memory_type: Option<agent_mem_core::types::MemoryType>,
    ) -> Result<Vec<MemoryItem>> {
        // 使用混合搜索作为基础实现
        let user_id_str = user_id.unwrap_or_else(|| "default".to_string());
        Self::search_memories_hybrid(orchestrator, query, user_id_str, limit, None, None).await
    }

    /// 混合搜索记忆
    #[cfg(feature = "postgres")]
    pub async fn search_memories_hybrid(
        orchestrator: &MemoryOrchestrator,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        info!(
            "混合搜索记忆: query={}, user_id={}, limit={}",
            query, user_id, limit
        );

        // Step 1: 查询预处理
        let processed_query = UtilsModule::preprocess_query(&query).await?;
        debug!("查询预处理完成: '{}' -> '{}'", query, processed_query);

        // Step 2: 动态阈值调整
        let dynamic_threshold =
            UtilsModule::calculate_dynamic_threshold(&processed_query, threshold);
        debug!("动态阈值: {:?} -> {}", threshold, dynamic_threshold);

        // Step 3: 生成查询向量
        let query_vector = if let Some(embedder) = &orchestrator.embedder {
            UtilsModule::generate_query_embedding(&processed_query, embedder.as_ref()).await?
        } else {
            return Err(agent_mem_traits::AgentMemError::ConfigError(
                "Embedder not configured. Cannot perform vector search without embedder."
                    .to_string(),
            ));
        };

        // Step 4: 构建SearchQuery
        use agent_mem_core::search::{SearchFilters, SearchQuery};
        let mut search_filters = SearchFilters::default();

        // 添加用户ID过滤
        search_filters.user_id = Some(user_id.clone());

        // 添加自定义过滤条件
        if let Some(filters) = filters {
            for (k, v) in filters {
                search_filters.metadata.insert(k, v);
            }
        }

        let search_query = SearchQuery {
            query: processed_query.clone(),
            limit,
            threshold: Some(dynamic_threshold),
            vector_weight: 0.7,
            fulltext_weight: 0.3,
            filters: Some(search_filters),
            metadata_filters: None,
        };

        // Step 5: 执行搜索
        // 如果有 HybridSearchEngine 组件，使用混合搜索
        #[cfg(feature = "postgres")]
        if let Some(hybrid_search_engine) = &orchestrator.hybrid_search_engine {
            let (search_results, _elapsed) = hybrid_search_engine
                .search(&search_query)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!("Search failed: {}", e))
                })?;

            // Step 6: 转换结果
            let mut memory_items =
                UtilsModule::convert_search_results_to_memory_items(search_results);
            info!("✅ 混合搜索完成: {} 个结果", memory_items.len());

            // Step 7: 应用重排序（如果启用）
            if let Some(reranker) = &orchestrator.reranker {
                info!("应用重排序...");
                let search_results_for_rerank: Vec<agent_mem_core::search::SearchResult> =
                    memory_items
                        .iter()
                        .map(|item| agent_mem_core::search::SearchResult {
                            id: item.id.clone(),
                            content: item.content.clone(),
                            score: item.score.unwrap_or(0.0),
                            vector_score: item.score,
                            fulltext_score: None,
                            metadata: item.metadata.clone().map(|m| serde_json::json!(m)),
                        })
                        .collect();

                match reranker
                    .rerank(
                        &processed_query,
                        Some(&query_vector),
                        search_results_for_rerank,
                        &search_query,
                    )
                    .await
                {
                    Ok(reranked_results) => {
                        memory_items =
                            UtilsModule::convert_search_results_to_memory_items(reranked_results);
                        info!("✅ 重排序完成: {} 个结果", memory_items.len());
                    }
                    Err(e) => {
                        warn!("重排序失败: {}, 使用原始结果", e);
                    }
                }
            }

            Ok(memory_items)
        } else {
            // 降级：仅使用向量搜索
            warn!("Search 组件未初始化，降级到向量搜索");
            if let Some(vector_store) = &orchestrator.vector_store {
                let mut filter_map: HashMap<String, serde_json::Value> = HashMap::new();
                filter_map.insert("user_id".to_string(), serde_json::json!(user_id));

                if let Some(filters) = filters {
                    for (k, v) in filters {
                        filter_map.insert(k, serde_json::json!(v));
                    }
                }

                let search_results = vector_store
                    .search_with_filters(query_vector, limit, &filter_map, Some(dynamic_threshold))
                    .await?;

                let memory_items = UtilsModule::convert_search_results_to_memory_items(
                    search_results
                        .into_iter()
                        .map(|r| agent_mem_core::search::SearchResult {
                            id: r.id,
                            content: r.metadata.get("data").unwrap_or(&String::new()).clone(),
                            score: r.similarity,
                            vector_score: Some(r.similarity),
                            fulltext_score: None,
                            metadata: Some(r.metadata),
                        })
                        .collect(),
                );

                Ok(memory_items)
            } else {
                Err(agent_mem_traits::AgentMemError::ConfigError(
                    "Neither Search engine nor VectorStore is configured".to_string(),
                ))
            }
        }
    }

    /// 混合搜索记忆（非 postgres 版本）
    #[cfg(not(feature = "postgres"))]
    pub async fn search_memories_hybrid(
        orchestrator: &MemoryOrchestrator,
        query: String,
        user_id: String,
        limit: usize,
        threshold: Option<f32>,
        filters: Option<HashMap<String, String>>,
    ) -> Result<Vec<MemoryItem>> {
        use agent_mem_traits::Session;
        use chrono::Utc;

        info!(
            "向量搜索（嵌入式模式）: query={}, user_id={}, limit={}",
            query, user_id, limit
        );

        // 动态阈值调整
        let dynamic_threshold = Some(UtilsModule::calculate_dynamic_threshold(&query, threshold));

        // 1. 生成查询向量
        let query_vector = if let Some(embedder) = &orchestrator.embedder {
            UtilsModule::generate_query_embedding(&query, embedder.as_ref()).await?
        } else {
            return Err(agent_mem_traits::AgentMemError::ConfigError(
                "Embedder not configured".to_string(),
            ));
        };

        // 2. 向量搜索
        if let Some(vector_store) = &orchestrator.vector_store {
            let mut filter_map = HashMap::new();
            filter_map.insert(
                "_query_hint".to_string(),
                serde_json::json!(query.to_lowercase()),
            );

            if let Some(filters) = filters {
                for (k, v) in filters {
                    filter_map.insert(k, serde_json::json!(v));
                }
            }

            let search_results = vector_store
                .search_with_filters(query_vector, limit, &filter_map, dynamic_threshold)
                .await?;

            info!("向量搜索完成: {} 个结果", search_results.len());

            // 3. 转换为 MemoryItem
            let mut memory_items: Vec<MemoryItem> = search_results
                .into_iter()
                .map(|result| {
                    let metadata_json: HashMap<String, serde_json::Value> = result
                        .metadata
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                        .collect();

                    MemoryItem {
                        id: result.id.clone(),
                        content: result
                            .metadata
                            .get("data")
                            .unwrap_or(&String::new())
                            .clone(),
                        hash: result.metadata.get("hash").cloned(),
                        metadata: metadata_json,
                        score: Some(result.similarity),
                        created_at: Utc::now(),
                        updated_at: None,
                        session: Session::new(),
                        memory_type: agent_mem_traits::MemoryType::Semantic,
                        entities: Vec::new(),
                        relations: Vec::new(),
                        agent_id: "default".to_string(),
                        user_id: None,
                        importance: result.similarity,
                        embedding: None,
                        last_accessed_at: Utc::now(),
                        access_count: 0,
                        expires_at: None,
                        version: 1,
                    }
                })
                .collect();

            // 🔧 修复: 验证记忆是否在LibSQL中存在且未删除
            // 过滤掉已删除的记忆（is_deleted=1）
            if let Some(manager) = &orchestrator.memory_manager {
                use futures::future;
                use std::sync::Arc;
                
                // 并行检查每个记忆是否存在
                let check_futures: Vec<_> = memory_items
                    .iter()
                    .map(|item| {
                        let id = item.id.clone();
                        let manager = Arc::clone(manager);
                        async move {
                            // 使用MemoryManager的get_memory方法检查记忆是否存在
                            // get_memory内部会检查is_deleted=0
                            manager.get_memory(&id).await
                                .map(|opt| opt.is_some())
                                .unwrap_or(false)
                        }
                    })
                    .collect();
                
                let check_results = future::join_all(check_futures).await;
                
                // 过滤有效结果（只保留在LibSQL中存在且未删除的记忆）
                memory_items = memory_items
                    .into_iter()
                    .zip(check_results.into_iter())
                    .filter_map(|(item, exists)| {
                        if exists {
                            Some(item)
                        } else {
                            debug!("过滤已删除的记忆: {}", item.id);
                            None
                        }
                    })
                    .collect();
                
                info!("🔄 验证完成: 过滤后剩余 {} 条有效结果", memory_items.len());
            }

            Ok(memory_items)
        } else {
            warn!("VectorStore 未初始化，返回空结果");
            Ok(Vec::new())
        }
    }

    /// 上下文感知重排序
    pub async fn context_aware_rerank(
        orchestrator: &MemoryOrchestrator,
        memories: Vec<MemoryItem>,
        query: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryItem>> {
        // 实现更复杂的上下文感知重排序逻辑
        let mut scored_memories: Vec<(MemoryItem, f32)> = memories
            .into_iter()
            .map(|mem| {
                let mut score = mem.importance;

                // 1. 重要性权重 (40%)
                let importance_score = mem.importance * 0.4;

                // 2. 相关性权重 (30%) - 基于内容匹配
                let relevance_score = if mem.content.to_lowercase().contains(&query.to_lowercase())
                {
                    0.3
                } else {
                    0.1
                };

                // 3. 时间衰减权重 (20%) - 最近访问的记忆得分更高
                let time_score = if let Some(updated_at) = mem.updated_at {
                    let age_days = (chrono::Utc::now() - updated_at).num_days();
                    let decay_factor = (1.0 / (1.0 + age_days as f32 / 30.0)).min(1.0);
                    decay_factor * 0.2
                } else {
                    0.1
                };

                // 4. 访问频率权重 (10%) - 访问次数多的记忆得分更高
                let access_score = {
                    let access_factor = (mem.access_count as f32 / 100.0).min(1.0);
                    access_factor * 0.1
                };

                // 5. 用户相关性权重 - 如果记忆属于当前用户，得分更高
                let user_score = if mem.user_id.as_deref() == Some(user_id) {
                    0.1
                } else {
                    0.0
                };

                score = importance_score + relevance_score + time_score + access_score + user_score;
                (mem, score)
            })
            .collect();

        // 按综合得分排序
        scored_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 返回排序后的记忆（去掉得分）
        Ok(scored_memories.into_iter().map(|(mem, _)| mem).collect())
    }
}
