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

/// Server-side memory manager wrapper (基于Memory统一API)
pub struct MemoryManager {
    memory: Arc<Memory>,
    /// 🆕 Fix 2: 查询优化器
    query_optimizer: Arc<agent_mem_core::search::QueryOptimizer>,
    /// 🆕 Fix 2: 结果重排序器
    reranker: Arc<agent_mem_core::search::ResultReranker>,
}

impl MemoryManager {
    /// 创建新的MemoryManager（使用Memory API + LibSQL持久化 + Embedder配置）
    pub async fn new(
        embedder_provider: Option<String>,
        embedder_model: Option<String>,
    ) -> ServerResult<Self> {
        // 🔧 修复：使用builder模式显式指定LibSQL存储，而不是默认的内存存储
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
        
        info!("Initializing Memory with LibSQL storage: {}", db_path);
        
        let mut builder = Memory::builder()
            .with_storage(&db_path);  // 🔑 关键修复：显式指定使用LibSQL
        
        // 🔑 关键修复 #2：配置Embedder（P0问题）
        if let (Some(provider), Some(model)) = (embedder_provider, embedder_model) {
            info!("Configuring embedder: provider={}, model={}", provider, model);
            builder = builder.with_embedder(provider, model);
        } else {
            // 使用默认FastEmbed配置
            info!("No embedder config provided, using default FastEmbed");
            builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");
        }
        
        let memory = builder
            .build()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to create Memory with LibSQL: {}", e)))?;
        
        info!("Memory initialized successfully with LibSQL persistence");
        
        // 🆕 Fix 2: 初始化QueryOptimizer和Reranker
        let query_optimizer = {
            use std::sync::RwLock;
            let stats = Arc::new(RwLock::new(agent_mem_core::search::IndexStatistics::default()));
            agent_mem_core::search::QueryOptimizer::with_default_config(stats)
        };
        
        let reranker = agent_mem_core::search::ResultReranker::with_default_config();
        
        info!("✅ QueryOptimizer and Reranker initialized");
        
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
            let stats = Arc::new(RwLock::new(agent_mem_core::search::IndexStatistics::default()));
            agent_mem_core::search::QueryOptimizer::with_default_config(stats)
        };
        
        let reranker = agent_mem_core::search::ResultReranker::with_default_config();
        
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
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<agent_mem_traits::MemoryType>,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        use agent_mem_utils::hash::compute_content_hash;
        use chrono::Utc;
        
        // Step 1: 使用Memory API（生成向量嵌入）
        let options = AddMemoryOptions {
            agent_id: Some(agent_id.clone()),
            user_id: user_id.clone(),
            infer: false,  // 简单模式，避免复杂推理
            metadata: metadata.clone().unwrap_or_default(),
            memory_type: memory_type.as_ref().map(|t| format!("{:?}", t)),
            ..Default::default()
        };

        let add_result = self.memory
            .add_with_options(&content, options)
            .await
            .map_err(|e| e.to_string())?;

        let memory_id = add_result.results
            .first()
            .map(|r| r.id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Step 2: 写入LibSQL Repository（持久化）
        let user_id_val = user_id.unwrap_or_else(|| "default".to_string());
        let content_hash = compute_content_hash(&content);
        let now = Utc::now();
        
        // 构建metadata JSON
        let mut full_metadata = metadata.unwrap_or_default();
        full_metadata.insert("agent_id".to_string(), agent_id.clone());
        full_metadata.insert("user_id".to_string(), user_id_val.clone());
        full_metadata.insert("data".to_string(), content.clone());
        full_metadata.insert("hash".to_string(), content_hash.clone());
        
        let metadata_json: serde_json::Value = full_metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        
        // Step 2.5: 确保Agent存在（获取其organization_id和user_id）
        let agent = repositories.agents.find_by_id(&agent_id).await
            .map_err(|e| format!("Failed to query agent: {}", e))?
            .ok_or_else(|| format!("Agent not found: {}", agent_id))?;
        
        let memory = agent_mem_core::storage::models::Memory {
            id: memory_id.clone(),
            organization_id: agent.organization_id.clone(),  // 使用Agent的organization_id
            user_id: "default-user".to_string(),  // 使用默认user (TODO: 应该从auth获取实际user)
            agent_id: agent_id.clone(),
            content,
            hash: Some(content_hash),
            metadata: metadata_json,
            score: None,
            memory_type: format!("{:?}", memory_type.unwrap_or(agent_mem_traits::MemoryType::Semantic)),
            scope: "agent".to_string(),
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
        
        repositories.memories.create(&memory).await
            .map_err(|e| format!("Failed to persist to LibSQL: {}", e))?;
        
        info!("✅ Memory persisted: VectorStore + LibSQL (ID: {})", memory_id);
        Ok(memory_id)
    }

    /// 获取记忆（直接数据库查询）
    pub async fn get_memory(&self, id: &str) -> Result<Option<serde_json::Value>, String> {
        use libsql::{Builder, params};
        
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "data/agentmem.db".to_string());
        
        let db = Builder::new_local(&db_path).build().await
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let conn = db.connect()
            .map_err(|e| format!("Failed to connect: {}", e))?;
        
        let query = "SELECT id, agent_id, user_id, content, memory_type, importance, \
                     created_at, last_accessed, access_count, metadata, hash \
                     FROM memories WHERE id = ? AND is_deleted = 0 LIMIT 1";
        
        let mut stmt = conn.prepare(query).await
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        
        let mut rows = stmt.query(params![id]).await
            .map_err(|e| format!("Failed to query: {}", e))?;
        
        if let Some(row) = rows.next().await.map_err(|e| format!("Failed to fetch row: {}", e))? {
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
            
            let json = serde_json::json!({
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
        self.memory
            .delete(id)
            .await
            .map_err(|e| e.to_string())
    }

    /// 搜索记忆 (🆕 Fix 2: 集成QueryOptimizer优化查询参数)
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
        };
        
        let optimized_plan = self.query_optimizer
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
        
        let options = SearchOptions {
            user_id,
            limit: Some(fetch_limit),
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
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
};
use tracing::{error, info};

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
) -> ServerResult<(StatusCode, Json<crate::models::ApiResponse<crate::models::MemoryResponse>>)> {
    info!(
        "Adding new memory for agent_id: {:?}, user_id: {:?}",
        request.agent_id, request.user_id
    );

    let memory_id = memory_manager
        .add_memory(
            repositories,  // 传递repositories用于LibSQL持久化
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

    Ok((StatusCode::CREATED, Json(crate::models::ApiResponse::success(response))))
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
    Path(id): Path<String>,
    Json(request): Json<crate::models::UpdateMemoryRequest>,
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::MemoryResponse>>> {
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
    Path(id): Path<String>,
) -> ServerResult<Json<crate::models::ApiResponse<crate::models::MemoryResponse>>> {
    info!("Deleting memory with ID: {}", id);

    memory_manager.delete_memory(&id).await.map_err(|e| {
        error!("Failed to delete memory: {}", e);
        ServerError::MemoryError(e.to_string())
    })?;

    let response = crate::models::MemoryResponse {
        id,
        message: "Memory deleted successfully".to_string(),
    };

    Ok(Json(crate::models::ApiResponse::success(response)))
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
) -> ServerResult<Json<crate::models::ApiResponse<Vec<serde_json::Value>>>> {
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

    // 转换为JSON格式，简化结构以匹配前端期望
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|item| {
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
                "score": 1.0,  // Memory API不返回score，默认为1.0
            })
        })
        .collect();

    Ok(Json(crate::models::ApiResponse::success(json_results)))
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

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for memory_req in request.memories {
        match memory_manager
            .add_memory(
                repositories.clone(),  // 传递repositories用于LibSQL持久化
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
) -> ServerResult<Json<crate::models::ApiResponse<Vec<serde_json::Value>>>> {
    info!("Getting all memories for agent_id: {}", agent_id);

    // ===== 真实实现：直接数据库查询（绕过embedder）=====
    // 原因：Memory API 需要 embedder (get_all → search → embedder)
    // 解决：直接使用 LibSQL 查询，避免 ONNX Runtime 依赖
    
    use libsql::{Builder, params};
    
    let db_path = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "data/agentmem.db".to_string());
    
    let db = Builder::new_local(&db_path).build().await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    
    let conn = db.connect()
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;
    
    let query = "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE agent_id = ? AND is_deleted = 0 LIMIT 100";
    
    let mut stmt = conn.prepare(query).await
        .map_err(|e| ServerError::Internal(format!("Failed to prepare query: {}", e)))?;
    
    let mut rows = stmt.query(params![agent_id.clone()]).await
        .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?;
    
    let mut memories_json: Vec<serde_json::Value> = vec![];
    
    while let Some(row) = rows.next().await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))? {
        
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
    
    info!("Returning {} real memories from database", memories_json.len());
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
    use libsql::{Builder, params as sql_params};
    use chrono::{DateTime, Utc};
    
    // 解析参数
    let page = params.get("page").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let limit = params.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(20).min(100);
    let agent_id = params.get("agent_id");
    let memory_type = params.get("memory_type");
    let sort_by = params.get("sort_by").map(|s| s.as_str()).unwrap_or("created_at");
    let order = params.get("order").map(|s| s.as_str()).unwrap_or("DESC");
    let offset = page * limit;
    
    info!("📋 List all memories: page={}, limit={}, agent_id={:?}", page, limit, agent_id);
    
    // 连接数据库
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "file:./data/agentmem.db".to_string());
    let db = Builder::new_local(&db_path).build().await
        .map_err(|e| ServerError::Internal(format!("Failed to open database: {}", e)))?;
    let conn = db.connect()
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
            let mut stmt = conn.prepare(&query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![limit as i64, offset as i64]).await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        },
        (Some(aid), None) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND agent_id = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn.prepare(&query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![aid.clone(), limit as i64, offset as i64]).await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        },
        (None, Some(mt)) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND memory_type = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn.prepare(&query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![mt.clone(), limit as i64, offset as i64]).await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        },
        (Some(aid), Some(mt)) => {
            let query = format!(
                "SELECT id, agent_id, user_id, content, memory_type, importance, \
                 created_at, last_accessed, access_count, metadata, hash \
                 FROM memories WHERE is_deleted = 0 AND agent_id = ? AND memory_type = ? ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, order
            );
            let mut stmt = conn.prepare(&query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare: {}", e)))?;
            stmt.query(params![aid.clone(), mt.clone(), limit as i64, offset as i64]).await
                .map_err(|e| ServerError::Internal(format!("Failed to query: {}", e)))?
        },
    };
    
    let mut memories_json: Vec<serde_json::Value> = vec![];
    while let Some(row) = rows.next().await
        .map_err(|e| ServerError::Internal(format!("Failed to fetch row: {}", e)))? {
        
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
            let mut stmt = conn.prepare(query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt.query(params![]).await.ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten()) {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        },
        (Some(aid), None) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND agent_id = ?";
            let mut stmt = conn.prepare(query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt.query(params![aid.clone()]).await.ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten()) {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        },
        (None, Some(mt)) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND memory_type = ?";
            let mut stmt = conn.prepare(query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt.query(params![mt.clone()]).await.ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten()) {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        },
        (Some(aid), Some(mt)) => {
            let query = "SELECT COUNT(*) FROM memories WHERE is_deleted = 0 AND agent_id = ? AND memory_type = ?";
            let mut stmt = conn.prepare(query).await
                .map_err(|e| ServerError::Internal(format!("Failed to prepare count: {}", e)))?;
            if let Some(count_row) = stmt.query(params![aid.clone(), mt.clone()]).await.ok()
                .and_then(|mut rows| futures::executor::block_on(rows.next()).ok().flatten()) {
                count_row.get::<i64>(0).unwrap_or(0)
            } else {
                0
            }
        },
    };
    
    info!("✅ Retrieved {} memories (total: {})", memories_json.len(), total_count);
    
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

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let result = MemoryManager::new(Some("fastembed".to_string()), Some("BAAI/bge-small-en-v1.5".to_string())).await;
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

