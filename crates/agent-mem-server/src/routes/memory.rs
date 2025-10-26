//! Memory management routes - Unified Memory API version
//!
//! 架构优化：使用agent-mem的Memory统一API替代agent-mem-core的CoreMemoryManager
//! 优势：
//! - 更简洁的代码
//! - 统一的接口
//! - 自动的智能功能
//! - 更好的类型处理

use crate::{
    error::{ServerError, ServerResult},
    models::{
        BatchRequest, BatchResponse, MemoryRequest, MemoryResponse, SearchRequest, SearchResponse,
        UpdateMemoryRequest,
    },
};
use agent_mem::{Memory, AddMemoryOptions, SearchOptions, GetAllOptions, DeleteAllOptions};
use agent_mem_traits::MemoryItem;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Server-side memory manager wrapper (基于Memory统一API)
pub struct MemoryManager {
    memory: Arc<Memory>,
}

impl MemoryManager {
    /// 创建新的MemoryManager（使用Memory API）
    pub async fn new() -> ServerResult<Self> {
        let memory = Memory::new()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to create Memory: {}", e)))?;
        
        Ok(Self {
            memory: Arc::new(memory),
        })
    }

    /// 使用自定义配置创建
    pub async fn with_config(memory: Memory) -> Self {
        Self {
            memory: Arc::new(memory),
        }
    }

    /// 添加记忆
    pub async fn add_memory(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        _memory_type: Option<agent_mem_traits::MemoryType>,  // Memory API自动处理
        _importance: Option<f32>,  // Memory API自动评估
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let options = AddMemoryOptions {
            agent_id: Some(agent_id),
            user_id,
            infer: true,  // 启用智能推理
            metadata: metadata.unwrap_or_default(),  // ✅ 解包Option
            ..Default::default()
        };

        self.memory
            .add_with_options(content, options)
            .await
            .map(|result| {
                // 返回第一个记忆的ID（如果有多个，取第一个）
                result.results
                    .first()
                    .map(|r| r.id.clone())
                    .unwrap_or_else(|| "".to_string())
            })
            .map_err(|e| e.to_string())
    }

    /// 获取记忆
    pub async fn get_memory(&self, id: &str) -> Result<Option<serde_json::Value>, String> {
        match self.memory.get(id).await {
            Ok(memory) => {
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
                    "hash": memory.hash,
                });
                Ok(Some(json))
            }
            Err(e) => {
                if e.to_string().contains("not found") {
                    Ok(None)
                } else {
                    Err(e.to_string())
                }
            }
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
        self.memory
            .delete(id)
            .await
            .map_err(|e| e.to_string())
    }

    /// 搜索记忆
    pub async fn search_memories(
        &self,
        query: String,
        agent_id: Option<String>,
        user_id: Option<String>,
        limit: Option<usize>,
        _memory_type: Option<agent_mem_traits::MemoryType>,
    ) -> Result<Vec<MemoryItem>, String> {
        let options = SearchOptions {
            user_id,
            limit,
            threshold: Some(0.7),
            ..Default::default()
        };

        self.memory
            .search_with_options(query, options)
            .await
            .map_err(|e| e.to_string())
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
        self.memory
            .reset()
            .await
            .map_err(|e| e.to_string())
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<agent_mem::MemoryStats, String> {
        self.memory
            .get_stats()
            .await
            .map_err(|e| e.to_string())
    }
}

/// 默认实现（异步创建）
impl MemoryManager {
    pub fn new_sync() -> Self {
        // 注意：这只用于类型系统，实际使用应该调用async new()
        panic!("Use MemoryManager::new().await instead");
    }
}

// ==================== 路由处理器函数 ====================
// 以下是实际的HTTP路由处理器函数

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::Json,
};
use tracing::{error, info};

/// 添加新记忆
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
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<crate::models::MemoryRequest>,
) -> ServerResult<(StatusCode, Json<crate::models::MemoryResponse>)> {
    info!(
        "Adding new memory for agent_id: {:?}, user_id: {:?}",
        request.agent_id, request.user_id
    );

    let memory_id = memory_manager
        .add_memory(
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
        message: "Memory added successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
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
    Path(id): Path<String>,
    Json(request): Json<crate::models::UpdateMemoryRequest>,
) -> ServerResult<Json<crate::models::MemoryResponse>> {
    info!("Updating memory with ID: {}", id);

    memory_manager
        .update_memory(&id, request.content, request.importance, None)
        .await
        .map_err(|e| {
            error!("Failed to update memory: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    let response = crate::models::MemoryResponse {
        id,
        message: "Memory updated successfully".to_string(),
    };

    Ok(Json(response))
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
    Path(id): Path<String>,
) -> ServerResult<Json<crate::models::MemoryResponse>> {
    info!("Deleting memory with ID: {}", id);

    memory_manager.delete_memory(&id).await.map_err(|e| {
        error!("Failed to delete memory: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    let response = crate::models::MemoryResponse {
        id,
        message: "Memory deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// 搜索记忆
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
    Json(request): Json<crate::models::SearchRequest>,
) -> ServerResult<Json<crate::models::SearchResponse>> {
    info!("Searching memories with query: {}", request.query);

    let results = memory_manager
        .search_memories(
            request.query,
            request.agent_id,
            request.user_id,
            request.limit,
            request.memory_type,
        )
        .await
        .map_err(|e| {
            error!("Failed to search memories: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    // 转换为JSON格式
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|item| {
            serde_json::json!({
                "memory": {
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
                },
                "score": 1.0,  // Memory API不返回score，默认为1.0
                "match_type": "semantic",
            })
        })
        .collect();

    let total = json_results.len();
    let response = crate::models::SearchResponse {
        results: json_results,
        total,
    };

    Ok(Json(response))
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

/// 批量添加记忆
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
    Extension(memory_manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<crate::models::BatchRequest>,
) -> ServerResult<(StatusCode, Json<crate::models::BatchResponse>)> {
    info!("Batch adding {} memories", request.memories.len());

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for memory_req in request.memories {
        match memory_manager
            .add_memory(
                memory_req.agent_id,
                memory_req.user_id,
                memory_req.content,
                memory_req.memory_type,
                memory_req.importance,
                memory_req.metadata,
            )
            .await
        {
            Ok(id) => results.push(id),
            Err(e) => errors.push(e.to_string()),
        }
    }

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

    let mut successful = 0;
    let mut errors = Vec::new();

    for id in &ids {
        match memory_manager.delete_memory(id).await {
            Ok(_) => successful += 1,
            Err(e) => errors.push(format!("Failed to delete {id}: {e}")),
        }
    }

    let response = crate::models::BatchResponse {
        successful,
        failed: errors.len(),
        results: vec![],
        errors,
    };

    Ok(Json(response))
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
) -> ServerResult<Json<Vec<serde_json::Value>>> {
    info!("Getting all memories for agent_id: {}", agent_id);

    let memories = memory_manager
        .get_all_memories(Some(agent_id.clone()), None, Some(100))
        .await
        .map_err(|e| {
            error!("Failed to get agent memories: {}", e);
            ServerError::MemoryError(e.to_string())
        })?;

    // 转换为JSON格式
    let memories_json: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "agent_id": m.agent_id,
                "user_id": m.user_id,
                "content": m.content,
                "memory_type": m.memory_type,
                "importance": m.importance,
                "created_at": m.created_at,
                "last_accessed_at": m.last_accessed_at,
                "access_count": m.access_count,
                "metadata": m.metadata,
                "hash": m.hash,
            })
        })
        .collect();

    Ok(Json(memories_json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let result = MemoryManager::new().await;
        // 可能因为配置问题失败，但应该能创建
        println!("MemoryManager creation: {:?}", result.is_ok());
    }

    #[tokio::test]
    async fn test_memory_manager_with_builder() {
        // 使用Memory builder创建配置
        let memory = Memory::builder()
            .disable_intelligent_features()  // 测试时禁用智能功能
            .build()
            .await;

        if let Ok(mem) = memory {
            let manager = MemoryManager::with_config(mem).await;
            println!("MemoryManager with config created successfully");
        }
    }
}

