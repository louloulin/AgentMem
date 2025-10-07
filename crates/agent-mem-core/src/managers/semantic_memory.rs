//! Semantic Memory Manager
//!
//! 管理语义记忆（Semantic Memory）- 通用知识、概念、事实
//! 参考 MIRIX 的 SemanticMemoryManager 实现

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// 语义记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryItem {
    /// 记忆 ID
    pub id: String,
    /// 组织 ID
    pub organization_id: String,
    /// 用户 ID
    pub user_id: String,
    /// Agent ID
    pub agent_id: String,
    /// 概念名称
    pub name: String,
    /// 摘要
    pub summary: String,
    /// 详细信息
    pub details: String,
    /// 来源
    pub source: Option<String>,
    /// 层级路径（如：["favorites", "pets", "dog"]）
    pub tree_path: Vec<String>,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 语义记忆查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticQuery {
    /// 名称搜索
    pub name_query: Option<String>,
    /// 摘要搜索
    pub summary_query: Option<String>,
    /// 层级路径过滤
    pub tree_path_prefix: Option<Vec<String>>,
    /// 限制返回数量
    pub limit: Option<i64>,
}

/// 语义记忆管理器
pub struct SemanticMemoryManager {
    pool: Arc<PgPool>,
}

impl SemanticMemoryManager {
    /// 创建新的语义记忆管理器
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建语义记忆项
    pub async fn create_item(&self, item: SemanticMemoryItem) -> Result<SemanticMemoryItem> {
        info!("Creating semantic memory item: {}", item.id);

        let tree_path_json = serde_json::to_value(&item.tree_path)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;

        let result = sqlx::query_as!(
            SemanticMemoryItemRow,
            r#"
            INSERT INTO semantic_memory (
                id, organization_id, user_id, agent_id, name,
                summary, details, source, tree_path, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            item.id,
            item.organization_id,
            item.user_id,
            item.agent_id,
            item.name,
            item.summary,
            item.details,
            item.source,
            tree_path_json,
            item.metadata,
            item.created_at,
            item.updated_at,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create semantic item: {}", e)))?;

        Ok(result.into())
    }

    /// 获取语义记忆项
    pub async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<SemanticMemoryItem>> {
        debug!("Getting semantic memory item: {}", item_id);

        let result = sqlx::query_as!(
            SemanticMemoryItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get semantic item: {}", e)))?;

        Ok(result.map(Into::into))
    }

    /// 查询语义记忆项
    pub async fn query_items(&self, user_id: &str, query: SemanticQuery) -> Result<Vec<SemanticMemoryItem>> {
        info!("Querying semantic memory items for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM semantic_memory WHERE user_id = $1");
        let mut param_count = 1;

        // 构建动态查询
        if query.name_query.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND name ILIKE ${}", param_count));
        }

        if query.summary_query.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND summary ILIKE ${}", param_count));
        }

        if query.tree_path_prefix.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND tree_path @> ${}", param_count));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(limit) = query.limit {
            param_count += 1;
            sql.push_str(&format!(" LIMIT ${}", param_count));
        }

        // 使用 query_as 执行查询
        let mut query_builder = sqlx::query_as::<_, SemanticMemoryItemRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(name_query) = query.name_query {
            query_builder = query_builder.bind(format!("%{}%", name_query));
        }

        if let Some(summary_query) = query.summary_query {
            query_builder = query_builder.bind(format!("%{}%", summary_query));
        }

        if let Some(tree_path_prefix) = query.tree_path_prefix {
            let tree_path_json = serde_json::to_value(&tree_path_prefix)
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;
            query_builder = query_builder.bind(tree_path_json);
        }

        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }

        let results = query_builder
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query semantic items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// 更新语义记忆项
    pub async fn update_item(&self, item: SemanticMemoryItem) -> Result<bool> {
        info!("Updating semantic memory item: {}", item.id);

        let tree_path_json = serde_json::to_value(&item.tree_path)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;

        let result = sqlx::query!(
            r#"
            UPDATE semantic_memory
            SET name = $1, summary = $2, details = $3, source = $4,
                tree_path = $5, metadata = $6, updated_at = $7
            WHERE id = $8 AND user_id = $9
            "#,
            item.name,
            item.summary,
            item.details,
            item.source,
            tree_path_json,
            item.metadata,
            Utc::now(),
            item.id,
            item.user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update semantic item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除语义记忆项
    pub async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        info!("Deleting semantic memory item: {}", item_id);

        let result = sqlx::query!(
            r#"
            DELETE FROM semantic_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete semantic item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 搜索相似的语义记忆项（基于名称）
    pub async fn search_by_name(&self, user_id: &str, name_query: &str, limit: i64) -> Result<Vec<SemanticMemoryItem>> {
        debug!("Searching semantic items by name: {}", name_query);

        let results = sqlx::query_as!(
            SemanticMemoryItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
            user_id,
            format!("%{}%", name_query),
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to search semantic items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// 获取指定层级路径下的所有项
    pub async fn get_items_by_tree_path(&self, user_id: &str, tree_path: &[String]) -> Result<Vec<SemanticMemoryItem>> {
        debug!("Getting semantic items by tree path: {:?}", tree_path);

        let tree_path_json = serde_json::to_value(tree_path)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;

        let results = sqlx::query_as!(
            SemanticMemoryItemRow,
            r#"
            SELECT * FROM semantic_memory
            WHERE user_id = $1 AND tree_path @> $2
            ORDER BY updated_at DESC
            "#,
            user_id,
            tree_path_json,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get items by tree path: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

/// 数据库行结构
#[derive(Debug, sqlx::FromRow)]
struct SemanticMemoryItemRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    name: String,
    summary: String,
    details: String,
    source: Option<String>,
    tree_path: serde_json::Value,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SemanticMemoryItemRow> for SemanticMemoryItem {
    fn from(row: SemanticMemoryItemRow) -> Self {
        let tree_path: Vec<String> = serde_json::from_value(row.tree_path).unwrap_or_default();

        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            name: row.name,
            summary: row.summary,
            details: row.details,
            source: row.source,
            tree_path,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

