//! Orchestrator Batch - 批量操作模块
//!
//! 负责所有批量操作，包括批量添加、批量处理等

use std::collections::HashMap;
use tracing::{debug, error, info, warn};

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
            info!("开始批量生成嵌入: {} 个文本", contents.len());
            let result = embedder.embed_batch(&contents).await?;
            info!("批量嵌入生成成功: {} 个向量", result.len());
            result
        } else {
            warn!("Embedder 未初始化，无法批量生成嵌入");
            warn!("Orchestrator embedder 状态: None");
            return Err(agent_mem_traits::AgentMemError::embedding_error(
                "Embedder not initialized",
            ));
        };

        // Step 2: 准备批量数据
        let mut memory_ids = Vec::new();
        let mut vector_data_batch = Vec::new();
        let mut memory_manager_batch = Vec::new();
        let mut history_entries = Vec::new();

        for (i, (content, agent_id, user_id, memory_type, metadata)) in
            items.into_iter().enumerate()
        {
            let memory_id = uuid::Uuid::new_v4().to_string();
            memory_ids.push(memory_id.clone());

            let embedding = embeddings[i].clone();

            // 准备元数据
            let mut full_metadata = HashMap::new();
            full_metadata.insert("agent_id".to_string(), serde_json::json!(agent_id.clone()));
            if let Some(uid) = &user_id {
                full_metadata.insert("user_id".to_string(), serde_json::json!(uid));
            }
            if let Some(mt) = &memory_type {
                full_metadata.insert(
                    "memory_type".to_string(),
                    serde_json::json!(format!("{:?}", mt)),
                );
            }
            if let Some(meta) = metadata {
                full_metadata.extend(meta);
            }

            // 准备向量数据（批量写入VectorStore）
            let string_metadata: HashMap<String, String> = full_metadata
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect();
            vector_data_batch.push(agent_mem_traits::VectorData {
                id: memory_id.clone(),
                vector: embedding,
                metadata: string_metadata.clone(),
            });

            // 准备MemoryManager批量数据
            let mut metadata_for_manager: std::collections::HashMap<String, String> = string_metadata;
            metadata_for_manager.insert("_memory_id".to_string(), memory_id.clone());
            memory_manager_batch.push((
                memory_id.clone(),
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                memory_type,
                metadata_for_manager,
            ));

            // 准备历史记录
            history_entries.push(crate::history::HistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                memory_id: memory_id.clone(),
                old_memory: None,
                new_memory: Some(content),
                event: "ADD".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: None,
                is_deleted: false,
                actor_id: None,
                role: Some("user".to_string()),
            });
        }

        // Step 3: 批量写入（Phase 1.1 优化：使用批量操作）
        let core_manager = orchestrator.core_manager.clone();
        let vector_store = orchestrator.vector_store.clone();
        let history_manager = orchestrator.history_manager.clone();
        let memory_manager = orchestrator.memory_manager.clone();

        // 并行执行批量写入
        // 准备数据副本用于并行任务
        let memory_manager_batch_clone = memory_manager_batch.clone();
        let (core_result, vector_result, history_result, db_result) = tokio::join!(
            // CoreMemoryManager（可选，非关键）
            async move {
                if let Some(manager) = core_manager {
                    // 批量写入CoreMemoryManager（如果支持）
                    for (_, content, _, _, _, _) in &memory_manager_batch_clone {
                        if let Err(e) = manager.create_persona_block(content.clone(), None).await {
                            warn!("CoreMemoryManager批量写入失败: {}", e);
                        }
                    }
                }
                Ok::<(), String>(())
            },
            // VectorStore批量写入
            async move {
                if let Some(store) = vector_store {
                    store
                        .add_vectors(vector_data_batch)
                        .await
                        .map(|_| ())
                        .map_err(|e| format!("VectorStore批量写入失败: {e}"))
                } else {
                    Ok::<(), String>(())
                }
            },
            // HistoryManager批量写入
            async move {
                if let Some(history) = history_manager {
                    for entry in history_entries {
                        if let Err(e) = history.add_history(entry).await {
                            warn!("历史记录批量写入失败: {}", e);
                        }
                    }
                }
                Ok::<(), String>(())
            },
            // MemoryManager批量写入（关键：主存储）
            async move {
                if let Some(manager) = memory_manager {
                    use agent_mem_core::types::MemoryType;
                    for (memory_id, content, agent_id, user_id, memory_type, metadata) in memory_manager_batch {
                        match manager
                            .add_memory(
                                agent_id.clone(),
                                user_id.clone(),
                                content,
                                Some(memory_type.unwrap_or(MemoryType::Episodic)),
                                Some(1.0), // importance
                                Some(metadata),
                            )
                            .await
                        {
                            Ok(_) => {
                                debug!("MemoryManager批量写入成功: {}", memory_id);
                            }
                            Err(e) => {
                                error!("MemoryManager批量写入失败: {} - {}", memory_id, e);
                                return Err(format!("MemoryManager批量写入失败: {e}"));
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err("MemoryManager未初始化 - 致命错误!".to_string())
                }
            }
        );

        // Step 4: 检查结果并处理错误
        if let Err(e) = core_result {
            warn!("CoreMemoryManager批量写入警告: {}", e);
        }

        if let Err(e) = vector_result {
            error!("VectorStore批量写入失败: {}", e);
            // 如果VectorStore失败，需要回滚MemoryManager（数据一致性保证）
            if let Some(manager) = &orchestrator.memory_manager {
                warn!("开始回滚MemoryManager以确保数据一致性...");
                for memory_id in &memory_ids {
                    if let Err(rollback_err) = manager.delete_memory(memory_id).await {
                        error!("回滚MemoryManager失败: {} - {}", memory_id, rollback_err);
                    }
                }
            }
            return Err(agent_mem_traits::AgentMemError::storage_error(format!(
                "VectorStore批量写入失败，已回滚MemoryManager: {e}"
            )));
        }

        if let Err(e) = history_result {
            warn!("历史记录批量写入警告: {}", e);
        }

        if let Err(e) = db_result {
            error!("MemoryManager批量写入失败: {}", e);
            return Err(agent_mem_traits::AgentMemError::storage_error(format!(
                "MemoryManager批量写入失败: {e}"
            )));
        }

        info!("✅ 批量快速添加完成: {} 个记忆（批量嵌入+批量写入）", memory_ids.len());
        Ok(memory_ids)
    }

    /// 批量优化添加记忆
    pub async fn add_memory_batch_optimized(
        orchestrator: &MemoryOrchestrator,
        contents: Vec<String>,
        agent_id: String,
        user_id: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<Vec<String>> {
        if contents.is_empty() {
            return Ok(Vec::new());
        }

        info!("批量优化添加 {} 个记忆", contents.len());
        
        // 检查 embedder 是否初始化（添加详细日志）
        if orchestrator.embedder.is_none() {
            warn!("Embedder 未初始化，无法进行批量添加");
            warn!("Orchestrator embedder 状态: None");
            return Err(agent_mem_traits::AgentMemError::embedding_error(
                "Embedder not initialized",
            ));
        } else {
            info!("✅ Embedder 已初始化，可以进行批量添加");
        }

        // 转换 metadata 类型: HashMap<String, String> -> HashMap<String, serde_json::Value>
        let metadata_json: HashMap<String, serde_json::Value> = metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        // 构建批量添加项
        let items: Vec<(
            String,
            String,
            Option<String>,
            Option<MemoryType>,
            Option<HashMap<String, serde_json::Value>>,
        )> = contents
            .into_iter()
            .map(|content| {
                (
                    content,
                    agent_id.clone(),
                    user_id.clone(),
                    None,
                    Some(metadata_json.clone()),
                )
            })
            .collect();

        // 使用批量添加方法
        Self::add_memories_batch(orchestrator, items).await
    }
}
