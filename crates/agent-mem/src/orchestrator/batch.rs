//! Orchestrator Batch - 批量操作模块
//!
//! 负责所有批量操作，包括批量添加、批量处理等

use std::collections::HashMap;
use tracing::{error, info, warn};

use agent_mem_core::types::MemoryType;
use agent_mem_traits::Result;

use super::core::MemoryOrchestrator;

/// 批量操作模块
///
/// 负责所有批量操作
pub struct BatchModule;

impl BatchModule {
    /// 批量快速添加记忆（无LLM调用，批量嵌入生成，并行写入）
    pub async fn add_memories_batch(
        orchestrator: &MemoryOrchestrator,
        items: Vec<(
            String,
            String,
            Option<String>,
            Option<MemoryType>,
            Option<HashMap<String, serde_json::Value>>,
        )>,
    ) -> Result<Vec<String>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        info!("批量快速添加 {} 个记忆", items.len());

        // Step 1: 批量生成嵌入（关键优化：一次性生成所有嵌入）
        let contents: Vec<String> = items.iter().map(|(c, _, _, _, _)| c.clone()).collect();

        let embeddings = if let Some(embedder) = &orchestrator.embedder {
            embedder.embed_batch(&contents).await?
        } else {
            return Err(agent_mem_traits::AgentMemError::embedding_error(
                "Embedder not initialized",
            ));
        };

        // Step 2: 为每个记忆准备数据并创建并行任务
        let mut memory_ids = Vec::new();
        let mut tasks = Vec::new();

        for (i, (content, agent_id, user_id, memory_type, metadata)) in items.into_iter().enumerate() {
            let memory_id = uuid::Uuid::new_v4().to_string();
            memory_ids.push(memory_id.clone());

            let embedding = embeddings[i].clone();

            // 准备元数据
            let mut full_metadata = HashMap::new();
            full_metadata.insert("agent_id".to_string(), serde_json::json!(agent_id));
            if let Some(uid) = &user_id {
                full_metadata.insert("user_id".to_string(), serde_json::json!(uid));
            }
            if let Some(mt) = &memory_type {
                full_metadata.insert("memory_type".to_string(), serde_json::json!(format!("{:?}", mt)));
            }
            if let Some(meta) = metadata {
                full_metadata.extend(meta);
            }

            // 创建并行任务
            let core_manager = orchestrator.core_manager.clone();
            let vector_store = orchestrator.vector_store.clone();
            let history_manager = orchestrator.history_manager.clone();

            // 为每个async块准备独立的clone
            let content_for_core = content.clone();
            let content_for_history = content.clone();
            let memory_id_for_vector = memory_id.clone();
            let memory_id_for_history = memory_id.clone();
            let embedding_for_vector = embedding.clone();
            let metadata_for_vector = full_metadata.clone();

            let task = async move {
                let (core_result, vector_result, history_result) = tokio::join!(
                    async move {
                        if let Some(manager) = core_manager {
                            manager.create_persona_block(content_for_core, None)
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        } else {
                            Ok::<(), String>(())
                        }
                    },
                    async move {
                        if let Some(store) = vector_store {
                            let string_metadata: HashMap<String, String> = metadata_for_vector
                                .iter()
                                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                .collect();
                            let vector_data = agent_mem_traits::VectorData {
                                id: memory_id_for_vector,
                                vector: embedding_for_vector,
                                metadata: string_metadata,
                            };
                            store.add_vectors(vec![vector_data])
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        } else {
                            Ok::<(), String>(())
                        }
                    },
                    async move {
                        if let Some(history) = history_manager {
                            let entry = crate::history::HistoryEntry {
                                id: uuid::Uuid::new_v4().to_string(),
                                memory_id: memory_id_for_history,
                                old_memory: None,
                                new_memory: Some(content_for_history),
                                event: "ADD".to_string(),
                                created_at: chrono::Utc::now(),
                                updated_at: None,
                                is_deleted: false,
                                actor_id: None,
                                role: Some("user".to_string()),
                            };
                            history.add_history(entry)
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        } else {
                            Ok::<(), String>(())
                        }
                    }
                );

                // 检查结果
                if let Err(e) = core_result {
                    return Err(format!("CoreMemoryManager 失败: {}", e));
                }
                if let Err(e) = vector_result {
                    return Err(format!("VectorStore 失败: {}", e));
                }
                if let Err(e) = history_result {
                    warn!("历史记录失败: {}", e);
                }

                Ok::<(), String>(())
            };

            tasks.push(task);
        }

        // Step 3: 并行执行所有写入任务
        let results = futures::future::join_all(tasks).await;

        // 检查是否有失败
        for (i, result) in results.iter().enumerate() {
            if let Err(e) = result {
                error!("批量添加第 {} 个记忆失败: {}", i, e);
                return Err(agent_mem_traits::AgentMemError::storage_error(&format!(
                    "批量添加失败: {}",
                    e
                )));
            }
        }

        info!("批量快速添加完成: {} 个记忆", memory_ids.len());
        Ok(memory_ids)
    }

    /// 批量优化添加记忆
    pub async fn add_memory_batch_optimized(
        _orchestrator: &MemoryOrchestrator,
        _contents: Vec<String>,
        _agent_id: String,
        _user_id: Option<String>,
        _metadata: HashMap<String, String>,
    ) -> Result<Vec<String>> {
        // TODO: 实现批量优化添加逻辑
        todo!("add_memory_batch_optimized not yet implemented in batch module")
    }
}




