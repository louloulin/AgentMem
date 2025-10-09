//! Graph API routes
//!
//! This module provides REST API endpoints for knowledge graph visualization and querying.
//!
//! Features:
//! - Entity and relation querying
//! - Graph data export for visualization
//! - Association management
//! - Multi-tenant isolation
//! - OpenAPI documentation

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use crate::models::ApiResponse;
use agent_mem_core::managers::{
    AssociationManager, AssociationManagerConfig, AssociationType, EntityType,
    KnowledgeGraphManager, KnowledgeGraphConfig,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::ToSchema;

/// 图谱查询请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GraphQueryRequest {
    /// 起始实体 ID
    pub start_entity_id: Option<String>,
    /// 最大深度
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    /// 实体类型过滤
    pub entity_types: Option<Vec<String>>,
    /// 关系类型过滤
    pub relation_types: Option<Vec<String>>,
    /// 最小置信度
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
}

fn default_max_depth() -> usize {
    3
}

fn default_min_confidence() -> f32 {
    0.5
}

/// 图谱节点
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub properties: serde_json::Value,
    pub confidence: f32,
}

/// 图谱边
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GraphEdge {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
    pub strength: f32,
}

/// 图谱数据响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GraphDataResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub total_nodes: usize,
    pub total_edges: usize,
}

/// 关联创建请求
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAssociationRequest {
    pub from_memory_id: String,
    pub to_memory_id: String,
    pub association_type: String,
    pub strength: f32,
    pub confidence: f32,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// 关联响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssociationResponse {
    pub id: String,
    pub from_memory_id: String,
    pub to_memory_id: String,
    pub association_type: String,
    pub strength: f32,
    pub confidence: f32,
    pub metadata: serde_json::Value,
}

/// 获取图谱数据
///
/// 导出知识图谱数据用于可视化
#[utoipa::path(
    get,
    path = "/api/v1/graph/data",
    params(
        ("max_depth" = Option<usize>, Query, description = "Maximum depth for graph traversal"),
        ("entity_types" = Option<String>, Query, description = "Comma-separated entity types to filter"),
    ),
    responses(
        (status = 200, description = "Graph data retrieved successfully", body = GraphDataResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "graph"
)]
pub async fn get_graph_data(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<GraphQueryRequest>,
) -> ServerResult<Json<ApiResponse<GraphDataResponse>>> {
    let kg_manager = KnowledgeGraphManager::with_default_config(Arc::new(pool.clone()));
    let assoc_manager = AssociationManager::with_default_config(Arc::new(pool));

    // 获取所有实体
    let mut all_nodes = Vec::new();
    let entity_types = vec![
        EntityType::Person,
        EntityType::Organization,
        EntityType::Location,
        EntityType::Event,
        EntityType::Concept,
        EntityType::Object,
    ];

    for entity_type in entity_types {
        let entities = kg_manager
            .get_entities_by_type(&auth_user.user_id, entity_type)
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        for entity in entities {
            if entity.confidence >= params.min_confidence {
                all_nodes.push(GraphNode {
                    id: entity.id,
                    name: entity.name,
                    node_type: entity.entity_type,
                    properties: entity.properties,
                    confidence: entity.confidence,
                });
            }
        }
    }

    // 获取所有关联作为边
    let mut all_edges = Vec::new();
    for node in &all_nodes {
        let associations = assoc_manager
            .get_associations(&node.id, &auth_user.user_id)
            .await
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        for assoc in associations {
            all_edges.push(GraphEdge {
                id: assoc.id,
                from_id: assoc.from_memory_id,
                to_id: assoc.to_memory_id,
                edge_type: assoc.association_type,
                properties: assoc.metadata,
                strength: assoc.strength,
            });
        }
    }

    let response = GraphDataResponse {
        total_nodes: all_nodes.len(),
        total_edges: all_edges.len(),
        nodes: all_nodes,
        edges: all_edges,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 创建记忆关联
///
/// 在两个记忆之间创建关联
#[utoipa::path(
    post,
    path = "/api/v1/graph/associations",
    request_body = CreateAssociationRequest,
    responses(
        (status = 201, description = "Association created successfully", body = AssociationResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "graph"
)]
pub async fn create_association(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateAssociationRequest>,
) -> ServerResult<(StatusCode, Json<ApiResponse<AssociationResponse>>)> {
    let assoc_manager = AssociationManager::with_default_config(Arc::new(pool));

    let assoc_type = AssociationType::from_str(&req.association_type)
        .map_err(|e| ServerError::BadRequest(e.to_string()))?;

    let association_id = assoc_manager
        .create_association(
            &auth_user.org_id,
            &auth_user.user_id,
            &auth_user.user_id, // agent_id
            &req.from_memory_id,
            &req.to_memory_id,
            assoc_type,
            req.strength,
            req.confidence,
            req.metadata.clone(),
        )
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let response = AssociationResponse {
        id: association_id,
        from_memory_id: req.from_memory_id,
        to_memory_id: req.to_memory_id,
        association_type: req.association_type,
        strength: req.strength,
        confidence: req.confidence,
        metadata: req.metadata,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// 获取记忆的关联
///
/// 获取指定记忆的所有关联
#[utoipa::path(
    get,
    path = "/api/v1/graph/memories/{memory_id}/associations",
    params(
        ("memory_id" = String, Path, description = "Memory ID"),
    ),
    responses(
        (status = 200, description = "Associations retrieved successfully", body = Vec<AssociationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "graph"
)]
pub async fn get_memory_associations(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(memory_id): Path<String>,
) -> ServerResult<Json<ApiResponse<Vec<AssociationResponse>>>> {
    let assoc_manager = AssociationManager::with_default_config(Arc::new(pool));

    let associations = assoc_manager
        .get_associations(&memory_id, &auth_user.user_id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let response: Vec<AssociationResponse> = associations
        .into_iter()
        .map(|assoc| AssociationResponse {
            id: assoc.id,
            from_memory_id: assoc.from_memory_id,
            to_memory_id: assoc.to_memory_id,
            association_type: assoc.association_type,
            strength: assoc.strength,
            confidence: assoc.confidence,
            metadata: assoc.metadata,
        })
        .collect();

    Ok(Json(ApiResponse::success(response)))
}

/// 获取关联统计
///
/// 获取用户的关联统计信息
#[utoipa::path(
    get,
    path = "/api/v1/graph/stats",
    responses(
        (status = 200, description = "Statistics retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = []),
        ("api_key" = [])
    ),
    tag = "graph"
)]
pub async fn get_graph_stats(
    State(pool): State<PgPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<Json<ApiResponse<serde_json::Value>>> {
    let assoc_manager = AssociationManager::with_default_config(Arc::new(pool));

    let stats = assoc_manager
        .get_stats(&auth_user.user_id)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    let response = serde_json::json!({
        "total_associations": stats.total_associations,
        "by_type": stats.by_type,
        "avg_strength": stats.avg_strength,
        "strongest_count": stats.strongest_associations.len(),
    });

    Ok(Json(ApiResponse::success(response)))
}

