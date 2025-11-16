//! Orchestrator Storage - 存储模块
//!
//! 负责所有存储相关操作，包括添加、更新、删除记忆

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use agent_mem_core::types::MemoryType;
use agent_mem_core::storage::conversion::v4_to_legacy;
use agent_mem_traits::{MemoryItem, Result};
use agent_mem_utils::hash::compute_content_hash;

use super::core::MemoryOrchestrator;
use super::utils::UtilsModule;
use crate::types::AddResult;

/// 存储模块
///
/// 负责所有存储相关操作
pub struct StorageModule;

impl StorageModule {
    /// 快速添加记忆（无LLM调用）
    pub async fn add_memory_fast(
        orchestrator: &MemoryOrchestrator,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        info!(
            "添加记忆 (快速模式): content={}, agent_id={}",
            content, agent_id
        );

        let memory_id = uuid::Uuid::new_v4().to_string();

        // Step 1: 生成向量嵌入
        let embedding = if let Some(embedder) = &orchestrator.embedder {
            match embedder.embed(&content).await {
                Ok(emb) => {
                    debug!("✅ 生成嵌入向量，维度: {}", emb.len());
                    emb
                }
                Err(e) => {
                    error!("生成嵌入失败: {}, 中止操作", e);
                    return Err(agent_mem_traits::AgentMemError::EmbeddingError(format!(
                        "Failed to generate embedding: {}",
                        e
                    )));
                }
            }
        } else {
            error!("Embedder 未初始化，中止操作");
            return Err(agent_mem_traits::AgentMemError::embedding_error(
                "Embedder not initialized",
            ));
        };

        // Step 2: 准备 metadata
        let content_hash = compute_content_hash(&content);

        let mut full_metadata: HashMap<String, serde_json::Value> = HashMap::new();
        full_metadata.insert("data".to_string(), serde_json::json!(content.clone()));
        full_metadata.insert("hash".to_string(), serde_json::json!(content_hash));
        full_metadata.insert(
            "created_at".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        let actual_user_id = user_id.unwrap_or_else(|| "default".to_string());
        full_metadata.insert("user_id".to_string(), serde_json::json!(actual_user_id));
        full_metadata.insert("agent_id".to_string(), serde_json::json!(agent_id.clone()));

        let scope_type = UtilsModule::infer_scope_type(&actual_user_id, &agent_id, &metadata);
        full_metadata.insert("scope_type".to_string(), serde_json::json!(scope_type));

        if let Some(custom_meta) = metadata {
            for (k, v) in custom_meta {
                full_metadata.insert(k, v);
            }
        }

        // Step 3: 并行写入 CoreMemoryManager、VectorStore 和 HistoryManager
        let core_manager = orchestrator.core_manager.clone();
        let vector_store = orchestrator.vector_store.clone();
        let history_manager = orchestrator.history_manager.clone();

        // 为每个async块准备独立的clone
        let content_for_core = content.clone();
        let content_for_history = content.clone();
        let memory_id_for_vector = memory_id.clone();
        let memory_id_for_history = memory_id.clone();
        let embedding_for_vector = embedding.clone();
        let full_metadata_for_vector = full_metadata.clone();

        let (core_result, vector_result, history_result) = tokio::join!(
            // 并行任务 1: 存储到 CoreMemoryManager
            async move {
                if let Some(manager) = core_manager {
                    manager.create_persona_block(content_for_core, None).await.map(|_| ()).map_err(|e| e.to_string())
                } else {
                    Ok::<(), String>(())
                }
            },
            // 并行任务 2: 存储到 VectorStore
            async move {
                if let Some(store) = vector_store {
                    let string_metadata: HashMap<String, String> = full_metadata_for_vector
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect();

                    let vector_data = agent_mem_traits::VectorData {
                        id: memory_id_for_vector,
                        vector: embedding_for_vector,
                        metadata: string_metadata,
                    };

                    store.add_vectors(vec![vector_data]).await.map(|_| ()).map_err(|e| e.to_string())
                } else {
                    Ok::<(), String>(())
                }
            },
            // 并行任务 3: 记录历史
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

                    history.add_history(entry).await.map(|_| ()).map_err(|e| e.to_string())
                } else {
                    Ok::<(), String>(())
                }
            }
        );

        // 检查结果
        if let Err(e) = core_result {
            error!("存储到 CoreMemoryManager 失败: {:?}", e);
            return Err(agent_mem_traits::AgentMemError::storage_error(&format!(
                "Failed to store to CoreMemoryManager: {:?}",
                e
            )));
        }

        if let Err(e) = vector_result {
            error!("存储到 VectorStore 失败: {}", e);
            return Err(agent_mem_traits::AgentMemError::storage_error(&format!(
                "Failed to store to VectorStore: {}",
                e
            )));
        }

        if let Err(e) = history_result {
            error!("记录历史失败: {}", e);
            warn!("历史记录失败，但记忆已成功添加: {}", e);
        }

        info!("✅ 记忆添加完成（并行写入）: {}", memory_id);
        Ok(memory_id)
    }

    /// 添加记忆（简单模式，不使用智能推理）
    ///
    /// 直接添加原始内容，不进行事实提取、去重等智能处理
    pub async fn add_memory(
        orchestrator: &MemoryOrchestrator,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        memory_type: Option<MemoryType>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String> {
        // 委托给快速模式
        Self::add_memory_fast(
            orchestrator,
            content,
            agent_id,
            user_id,
            memory_type,
            metadata,
        )
        .await
    }

    /// 更新记忆
    pub async fn update_memory(
        orchestrator: &MemoryOrchestrator,
        memory_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<MemoryItem> {
        info!("更新记忆: {}", memory_id);

        // 从 data 中提取更新字段
        let new_content = data.get("content").and_then(|v| v.as_str().map(|s| s.to_string()));
        let new_importance = data.get("importance").and_then(|v| v.as_f64().map(|f| f as f32));
        let new_metadata: Option<HashMap<String, String>> = data
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            });

        // 1. 使用 MemoryManager 更新记忆
        if let Some(manager) = &orchestrator.memory_manager {
            manager
                .update_memory(memory_id, new_content.clone(), new_importance, new_metadata.clone())
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to update memory in MemoryManager: {}",
                        e
                    ))
                })?;
        }

        // 2. 如果内容更新，需要更新向量存储
        if let Some(new_content) = &new_content {
            if let Some(embedder) = &orchestrator.embedder {
                if let Some(vector_store) = &orchestrator.vector_store {
                    let embedding = embedder.embed(new_content).await.map_err(|e| {
                        agent_mem_traits::AgentMemError::EmbeddingError(format!(
                            "Failed to generate embedding: {}",
                            e
                        ))
                    })?;

                    let mut metadata_map: HashMap<String, String> = HashMap::new();
                    metadata_map.insert("data".to_string(), new_content.clone());
                    if let Some(meta) = &new_metadata {
                        for (k, v) in meta {
                            metadata_map.insert(k.clone(), v.clone());
                        }
                    }

                    let vector_data = agent_mem_traits::VectorData {
                        id: memory_id.to_string(),
                        vector: embedding,
                        metadata: metadata_map,
                    };

                    vector_store
                        .update_vectors(vec![vector_data])
                        .await
                        .map_err(|e| {
                            agent_mem_traits::AgentMemError::storage_error(&format!(
                                "Failed to update vector: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        // 3. 记录历史
        if let Some(history_manager) = &orchestrator.history_manager {
            let entry = crate::history::HistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                memory_id: memory_id.to_string(),
                old_memory: None, // 可以从之前的获取中获取
                new_memory: new_content.clone(),
                event: "UPDATE".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: None,
                is_deleted: false,
                actor_id: None,
                role: Some("system".to_string()),
            };

            let _ = history_manager.add_history(entry).await;
        }

        // 4. 获取更新后的记忆
        Self::get_memory(orchestrator, memory_id).await
    }

    /// 删除记忆
    pub async fn delete_memory(
        orchestrator: &MemoryOrchestrator,
        memory_id: &str,
    ) -> Result<()> {
        info!("删除记忆: {}", memory_id);

        // 1. 先获取记忆内容用于历史记录
        let old_memory = Self::get_memory(orchestrator, memory_id).await.ok();

        // 2. 使用 MemoryManager 删除记忆
        if let Some(manager) = &orchestrator.memory_manager {
            manager
                .delete_memory(memory_id)
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to delete memory from MemoryManager: {}",
                        e
                    ))
                })?;
        }

        // 3. 从向量存储删除
        if let Some(vector_store) = &orchestrator.vector_store {
            vector_store
                .delete_vectors(vec![memory_id.to_string()])
                .await
                .map_err(|e| {
                    agent_mem_traits::AgentMemError::storage_error(&format!(
                        "Failed to delete vector: {}",
                        e
                    ))
                })?;
        }

        // 4. 记录历史
        if let Some(history_manager) = &orchestrator.history_manager {
            let old_content = old_memory
                .as_ref()
                .and_then(|m| m.metadata.get("data"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let entry = crate::history::HistoryEntry {
                id: uuid::Uuid::new_v4().to_string(),
                memory_id: memory_id.to_string(),
                old_memory: old_content,
                new_memory: None,
                event: "DELETE".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: None,
                is_deleted: true,
                actor_id: None,
                role: Some("system".to_string()),
            };

            let _ = history_manager.add_history(entry).await;
        }

        info!("✅ 记忆删除完成: {}", memory_id);
        Ok(())
    }

    /// 获取记忆
    pub async fn get_memory(
        orchestrator: &MemoryOrchestrator,
        memory_id: &str,
    ) -> Result<MemoryItem> {
        debug!("获取记忆: {}", memory_id);

        // 优先从 MemoryManager 获取
        if let Some(manager) = &orchestrator.memory_manager {
            if let Some(memory) = manager.get_memory(memory_id).await.map_err(|e| {
                agent_mem_traits::AgentMemError::storage_error(&format!(
                    "Failed to get memory from MemoryManager: {}",
                    e
                ))
            })? {
                // 转换为 MemoryItem
                return Ok(v4_to_legacy(&memory));
            }
        }

        // 降级：从向量存储获取
        if let Some(vector_store) = &orchestrator.vector_store {
            // 尝试通过 ID 搜索（如果向量存储支持）
            // 这里假设可以通过 metadata 中的 ID 字段来查找
            // 实际实现可能需要根据具体的向量存储 API 调整
            warn!("从向量存储获取记忆的功能需要根据具体实现调整");
        }

        Err(agent_mem_traits::AgentMemError::NotFound(format!(
            "Memory not found: {}",
            memory_id
        )))
    }

    /// 添加记忆 v2（支持 infer 参数）
    pub async fn add_memory_v2(
        orchestrator: &MemoryOrchestrator,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        run_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
        infer: bool,
        memory_type: Option<String>,
        _prompt: Option<String>,
    ) -> Result<AddResult> {
        use crate::types::MemoryEvent;
        use agent_mem_core::types::MemoryType;

        debug!(
            "添加记忆 v2: content={}, agent_id={}, infer={}",
            content, agent_id, infer
        );

        // 根据 infer 参数选择处理模式
        if infer {
            // infer=true: 使用智能推理模式
            info!("使用智能推理模式 (infer=true)");
            Self::add_memory_intelligent(
                orchestrator,
                content,
                agent_id,
                user_id,
                metadata,
            )
            .await
        } else {
            // infer=false: 使用快速模式
            info!("使用快速模式 (infer=false)");

            // 解析 memory_type 字符串
            let mem_type = if let Some(type_str) = memory_type {
                match type_str.as_str() {
                    "core_memory" => Some(MemoryType::Core),
                    "episodic_memory" => Some(MemoryType::Episodic),
                    "semantic_memory" => Some(MemoryType::Semantic),
                    "procedural_memory" => Some(MemoryType::Procedural),
                    _ => None,
                }
            } else {
                None
            };

            // 调用快速添加方法
            let memory_id = Self::add_memory_fast(
                orchestrator,
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                mem_type,
                metadata,
            )
            .await?;

            // 构造返回结果
            let event = MemoryEvent {
                id: memory_id,
                memory: content,
                event: "ADD".to_string(),
                actor_id: user_id.or(Some(agent_id)),
                role: Some("user".to_string()),
            };

            Ok(AddResult {
                results: vec![event],
                relations: None,
            })
        }
    }

    /// 智能添加记忆（使用 Intelligence 组件）
    pub async fn add_memory_intelligent(
        orchestrator: &MemoryOrchestrator,
        content: String,
        agent_id: String,
        user_id: Option<String>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<AddResult> {
        use crate::types::MemoryEvent;
        use super::intelligence::IntelligenceModule;

        info!(
            "智能添加记忆: content={}, agent_id={}, user_id={:?}",
            content, agent_id, user_id
        );

        // 检查 Intelligence 组件是否可用
        if orchestrator.fact_extractor.is_none() {
            warn!("Intelligence 组件未初始化，降级到简单模式");
            let memory_id = Self::add_memory(
                orchestrator,
                content.clone(),
                agent_id.clone(),
                user_id.clone(),
                None,
                metadata.clone(),
            )
            .await?;
            return Ok(AddResult {
                results: vec![MemoryEvent {
                    id: memory_id,
                    memory: content,
                    event: "ADD".to_string(),
                    actor_id: Some(agent_id),
                    role: None,
                }],
                relations: None,
            });
        }

        // 使用 IntelligenceModule 进行智能处理
        IntelligenceModule::add_memory_intelligent(
            orchestrator,
            content,
            agent_id,
            user_id,
            metadata,
        )
        .await
    }
}

