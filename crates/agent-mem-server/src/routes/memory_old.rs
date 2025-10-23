//! Memory management routes

use crate::{
    error::{ServerError, ServerResult},
    models::{
        BatchRequest, BatchResponse, MemoryRequest, MemoryResponse, SearchRequest, SearchResponse,
        UpdateMemoryRequest,
    },
};
use agent_mem_core::{
    manager::MemoryManager as CoreMemoryManager,
    storage::factory::Repositories,
    types::MemoryQuery,
};
use agent_mem_traits::MemoryType;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server-side memory manager wrapper
pub struct MemoryManager {
    core_manager: Arc<RwLock<CoreMemoryManager>>,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            core_manager: Arc::new(RwLock::new(CoreMemoryManager::new())),
        }
    }

    pub async fn add_memory(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String, String> {
        let manager = self.core_manager.read().await;

        // Convert MemoryType from traits to core types
        let core_memory_type = memory_type.map(|mt| match mt {
            MemoryType::Factual => agent_mem_core::types::MemoryType::Semantic, // Map Factual to Semantic
            MemoryType::Episodic => agent_mem_core::types::MemoryType::Episodic,
            MemoryType::Procedural => agent_mem_core::types::MemoryType::Procedural,
            MemoryType::Semantic => agent_mem_core::types::MemoryType::Semantic,
            MemoryType::Working => agent_mem_core::types::MemoryType::Working,
            MemoryType::Core => agent_mem_core::types::MemoryType::Core,
            MemoryType::Resource => agent_mem_core::types::MemoryType::Resource,
            MemoryType::Knowledge => agent_mem_core::types::MemoryType::Knowledge,
            MemoryType::Contextual => agent_mem_core::types::MemoryType::Contextual,
        });

        manager
            .add_memory(
                agent_id,
                user_id,
                content,
                core_memory_type,
                importance,
                metadata,
            )
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<serde_json::Value>, String> {
        let manager = self.core_manager.read().await;
        match manager.get_memory(id).await {
            Ok(Some(memory)) => {
                let json = serde_json::json!({
                    "id": memory.id,
                    "agent_id": memory.agent_id,
                    "user_id": memory.user_id,
                    "content": memory.content,
                    "memory_type": memory.memory_type,
                    "importance": memory.importance,
                    "created_at": memory.created_at,
                    "last_accessed_at": memory.last_accessed_at,
                    "access_count": memory.access_count,
                    "metadata": memory.metadata,
                    "embedding": memory.embedding,
                });
                Ok(Some(json))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<String>,
        importance: Option<f32>,
    ) -> Result<(), String> {
        let manager = self.core_manager.read().await;

        // Update the memory using the correct method signature
        manager
            .update_memory(id, content, importance, None)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_memory(&self, id: &str) -> Result<(), String> {
        let manager = self.core_manager.read().await;
        manager
            .delete_memory(id)
            .await
            .map_err(|e| e.to_string())
            .map(|_| ())
    }

    pub async fn search_memories(
        &self,
        query: &MemoryQuery,
    ) -> Result<Vec<serde_json::Value>, String> {
        let manager = self.core_manager.read().await;
        match manager.search_memories(query.clone()).await {
            Ok(results) => {
                let json_results = results
                    .into_iter()
                    .map(|result| {
                        serde_json::json!({
                            "memory": {
                                "id": result.memory.id,
                                "agent_id": result.memory.agent_id,
                                "user_id": result.memory.user_id,
                                "content": result.memory.content,
                                "memory_type": result.memory.memory_type,
                                "importance": result.memory.importance,
                                "created_at": result.memory.created_at,
                                "last_accessed_at": result.memory.last_accessed_at,
                                "access_count": result.memory.access_count,
                                "metadata": result.memory.metadata,
                                "embedding": result.memory.embedding,
                            },
                            "score": result.score,
                            "match_type": result.match_type,
                        })
                    })
                    .collect();
                Ok(json_results)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn batch_add_memories(
        &self,
        requests: Vec<crate::models::MemoryRequest>,
    ) -> Result<Vec<String>, String> {
        let mut memory_ids = Vec::new();

        for request in requests {
            let memory_id = self
                .add_memory(
                    request.agent_id,
                    request.user_id,
                    request.content,
                    request.memory_type,
                    request.importance,
                    request.metadata,
                )
                .await?;
            memory_ids.push(memory_id);
        }

        Ok(memory_ids)
    }

    pub async fn batch_get_memories(
        &self,
        ids: Vec<String>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut memories = Vec::new();

        for id in ids {
            if let Some(memory) = self.get_memory(&id).await? {
                memories.push(memory);
            }
        }

        Ok(memories)
    }

    pub async fn get_memory_stats(
        &self,
        agent_id: Option<String>,
    ) -> Result<serde_json::Value, String> {
        let manager = self.core_manager.read().await;
        match manager.get_memory_stats(agent_id.as_deref()).await {
            Ok(stats) => {
                let json = serde_json::json!({
                    "total_memories": stats.total_memories,
                    "memory_types": stats.memories_by_type,
                    "memories_by_agent": stats.memories_by_agent,
                    "average_importance": stats.average_importance,
                    "oldest_memory_age_days": stats.oldest_memory_age_days,
                    "most_accessed_memory_id": stats.most_accessed_memory_id,
                    "total_access_count": stats.total_access_count,
                });
                Ok(json)
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
};
use tracing::{error, info};
use utoipa;

/// Add a new memory
#[utoipa::path(
    post,
    path = "/api/v1/memories",
    tag = "memory",
    request_body = MemoryRequest,
    responses(
        (status = 201, description = "Memory created successfully", body = MemoryResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn add_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<MemoryRequest>,
) -> ServerResult<(StatusCode, Json<MemoryResponse>)> {
    info!(
        "Adding new memory for agent_id: {:?}, user_id: {:?}",
        request.agent_id, request.user_id
    );

    let memory_type = request.memory_type.unwrap_or(MemoryType::Episodic);
    let importance = request.importance.unwrap_or(0.5);

    let memory_id = memory_manager
        .add_memory(
            request.agent_id,
            request.user_id,
            request.content,
            Some(memory_type),
            Some(importance),
            request.metadata,
        )
        .await
        .map_err(|e| {
            error!("Failed to add memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    let response = MemoryResponse {
        id: memory_id,
        message: "Memory added successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a memory by ID
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
) -> ServerResult<Json<serde_json::Value>> {
    info!("Getting memory with ID: {}", id);

    let memory = memory_manager.get_memory(&id).await.map_err(|e| {
        error!("Failed to get memory: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    match memory {
        Some(mem) => Ok(Json(mem)),
        None => Err(ServerError::NotFound("Memory not found".to_string())),
    }
}

/// Update a memory
#[utoipa::path(
    put,
    path = "/api/v1/memories/{id}",
    tag = "memory",
    params(
        ("id" = String, Path, description = "Memory ID")
    ),
    request_body = UpdateMemoryRequest,
    responses(
        (status = 200, description = "Memory updated successfully"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_memory(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMemoryRequest>,
) -> ServerResult<Json<MemoryResponse>> {
    info!("Updating memory with ID: {}", id);

    memory_manager
        .update_memory(&id, request.content, request.importance)
        .await
        .map_err(|e| {
            error!("Failed to update memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    let response = MemoryResponse {
        id,
        message: "Memory updated successfully".to_string(),
    };

    Ok(Json(response))
}

/// Delete a memory
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
    Path(id): Path<String>,
) -> ServerResult<Json<MemoryResponse>> {
    info!("Deleting memory with ID: {}", id);

    memory_manager.delete_memory(&id).await.map_err(|e| {
        error!("Failed to delete memory: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    let response = MemoryResponse {
        id,
        message: "Memory deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// Search memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/search",
    tag = "memory",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed successfully", body = SearchResponse),
        (status = 400, description = "Invalid search request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<SearchRequest>,
) -> ServerResult<Json<SearchResponse>> {
    info!("Searching memories with query: {}", request.query);

    // Convert MemoryType from traits to core types
    let core_memory_type = request.memory_type.map(|mt| match mt {
        MemoryType::Factual => agent_mem_core::types::MemoryType::Semantic,
        MemoryType::Episodic => agent_mem_core::types::MemoryType::Episodic,
        MemoryType::Procedural => agent_mem_core::types::MemoryType::Procedural,
        MemoryType::Semantic => agent_mem_core::types::MemoryType::Semantic,
        MemoryType::Working => agent_mem_core::types::MemoryType::Working,
        MemoryType::Core => agent_mem_core::types::MemoryType::Core,
        MemoryType::Resource => agent_mem_core::types::MemoryType::Resource,
        MemoryType::Knowledge => agent_mem_core::types::MemoryType::Knowledge,
        MemoryType::Contextual => agent_mem_core::types::MemoryType::Contextual,
    });

    let query = MemoryQuery {
        agent_id: request.agent_id.unwrap_or_default(),
        user_id: request.user_id,
        memory_type: core_memory_type,
        text_query: Some(request.query),
        vector_query: None,
        min_importance: None,
        max_age_seconds: None,
        limit: request.limit.unwrap_or(10),
    };

    let results = memory_manager.search_memories(&query).await.map_err(|e| {
        error!("Failed to search memories: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    let total = results.len();
    let response = SearchResponse { results, total };

    Ok(Json(response))
}

/// Get memory history
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

    // ✅ 验证 memory 存在
    let memory = memory_manager
        .get_memory(&id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to get memory: {}", e)))?
        .ok_or_else(|| ServerError::not_found("Memory not found"))?;

    // ✅ 构建历史记录
    // 注意：完整的实现需要从 memory_history 表查询
    // 当前返回当前版本作为历史记录（简化版）
    // 数据库迁移已创建，但需要通过 Repository trait 访问
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

    // ✅ 构建响应
    let response = serde_json::json!({
        "memory_id": id,
        "current_version": 1,
        "total_versions": history.len(),
        "history": history,
        "current_content": memory.get("content").and_then(|v| v.as_str()).unwrap_or(""),
        "current_metadata": memory.get("metadata").cloned().unwrap_or(serde_json::json!({})),
        "note": "Memory history table and triggers have been created. Full history tracking will be available once Repository trait is extended with history methods."
    });

    Ok(Json(response))
}

/// Batch add memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch",
    tag = "batch",
    request_body = BatchRequest,
    responses(
        (status = 201, description = "Batch operation completed", body = BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_add_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<BatchRequest>,
) -> ServerResult<(StatusCode, Json<BatchResponse>)> {
    info!("Batch adding {} memories", request.memories.len());

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for memory_req in request.memories {
        let memory_type = memory_req.memory_type.unwrap_or(MemoryType::Episodic);
        let importance = memory_req.importance.unwrap_or(0.5);

        match memory_manager
            .add_memory(
                memory_req.agent_id,
                memory_req.user_id,
                memory_req.content,
                Some(memory_type),
                Some(importance),
                memory_req.metadata,
            )
            .await
        {
            Ok(id) => results.push(id),
            Err(e) => errors.push(e.to_string()),
        }
    }

    let response = BatchResponse {
        successful: results.len(),
        failed: errors.len(),
        results,
        errors,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Batch delete memories
#[utoipa::path(
    post,
    path = "/api/v1/memories/batch/delete",
    tag = "batch",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Batch delete completed", body = BatchResponse),
        (status = 400, description = "Invalid batch request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn batch_delete_memories(
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(ids): Json<Vec<String>>,
) -> ServerResult<Json<BatchResponse>> {
    info!("Batch deleting {} memories", ids.len());

    let mut successful = 0;
    let mut errors = Vec::new();

    for id in &ids {
        match memory_manager.delete_memory(id).await {
            Ok(_) => successful += 1,
            Err(e) => errors.push(format!("Failed to delete {id}: {e}")),
        }
    }

    let response = BatchResponse {
        successful,
        failed: errors.len(),
        results: vec![], // No results for delete operations
        errors,
    };

    Ok(Json(response))
}
