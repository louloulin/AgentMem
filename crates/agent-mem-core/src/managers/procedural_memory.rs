//! Procedural Memory Manager
//!
//! 管理程序记忆（Procedural Memory）- 工作流、步骤、方法
//! 参考 MIRIX 的 ProceduralMemoryManager 实现

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

/// 程序记忆项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralMemoryItem {
    /// 记忆 ID
    pub id: String,
    /// 组织 ID
    pub organization_id: String,
    /// 用户 ID
    pub user_id: String,
    /// Agent ID
    pub agent_id: String,
    /// 条目类型（workflow, guide, script）
    pub entry_type: String,
    /// 摘要
    pub summary: String,
    /// 步骤列表
    pub steps: Vec<String>,
    /// 层级路径（如：["workflows", "development", "testing"]）
    pub tree_path: Vec<String>,
    /// 元数据
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 程序记忆查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralQuery {
    /// 条目类型过滤
    pub entry_type: Option<String>,
    /// 摘要搜索
    pub summary_query: Option<String>,
    /// 层级路径过滤
    pub tree_path_prefix: Option<Vec<String>>,
    /// 限制返回数量
    pub limit: Option<i64>,
}

/// 程序记忆管理器
pub struct ProceduralMemoryManager {
    pool: Arc<PgPool>,
}

impl ProceduralMemoryManager {
    /// 创建新的程序记忆管理器
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 创建程序记忆项
    pub async fn create_item(&self, item: ProceduralMemoryItem) -> Result<ProceduralMemoryItem> {
        info!("Creating procedural memory item: {}", item.id);

        let steps_json = serde_json::to_value(&item.steps)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e)))?;

        let tree_path_json = serde_json::to_value(&item.tree_path)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;

        let result = sqlx::query_as!(
            ProceduralMemoryItemRow,
            r#"
            INSERT INTO procedural_memory (
                id, organization_id, user_id, agent_id, entry_type,
                summary, steps, tree_path, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
            item.id,
            item.organization_id,
            item.user_id,
            item.agent_id,
            item.entry_type,
            item.summary,
            steps_json,
            tree_path_json,
            item.metadata,
            item.created_at,
            item.updated_at,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create procedural item: {}", e)))?;

        Ok(result.into())
    }

    /// 获取程序记忆项
    pub async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<ProceduralMemoryItem>> {
        debug!("Getting procedural memory item: {}", item_id);

        let result = sqlx::query_as!(
            ProceduralMemoryItemRow,
            r#"
            SELECT * FROM procedural_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get procedural item: {}", e)))?;

        Ok(result.map(Into::into))
    }

    /// 查询程序记忆项
    pub async fn query_items(&self, user_id: &str, query: ProceduralQuery) -> Result<Vec<ProceduralMemoryItem>> {
        info!("Querying procedural memory items for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM procedural_memory WHERE user_id = $1");
        let mut param_count = 1;

        // 构建动态查询
        if query.entry_type.is_some() {
            param_count += 1;
            sql.push_str(&format!(" AND entry_type = ${}", param_count));
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
        let mut query_builder = sqlx::query_as::<_, ProceduralMemoryItemRow>(&sql);
        query_builder = query_builder.bind(user_id);

        if let Some(entry_type) = query.entry_type {
            query_builder = query_builder.bind(entry_type);
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
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to query procedural items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// 更新程序记忆项
    pub async fn update_item(&self, item: ProceduralMemoryItem) -> Result<bool> {
        info!("Updating procedural memory item: {}", item.id);

        let steps_json = serde_json::to_value(&item.steps)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e)))?;

        let tree_path_json = serde_json::to_value(&item.tree_path)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize tree_path: {}", e)))?;

        let result = sqlx::query!(
            r#"
            UPDATE procedural_memory
            SET entry_type = $1, summary = $2, steps = $3,
                tree_path = $4, metadata = $5, updated_at = $6
            WHERE id = $7 AND user_id = $8
            "#,
            item.entry_type,
            item.summary,
            steps_json,
            tree_path_json,
            item.metadata,
            Utc::now(),
            item.id,
            item.user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to update procedural item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 删除程序记忆项
    pub async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        info!("Deleting procedural memory item: {}", item_id);

        let result = sqlx::query!(
            r#"
            DELETE FROM procedural_memory
            WHERE id = $1 AND user_id = $2
            "#,
            item_id,
            user_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete procedural item: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// 按类型获取程序记忆项
    pub async fn get_items_by_type(&self, user_id: &str, entry_type: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>> {
        debug!("Getting procedural items by type: {}", entry_type);

        let results = sqlx::query_as!(
            ProceduralMemoryItemRow,
            r#"
            SELECT * FROM procedural_memory
            WHERE user_id = $1 AND entry_type = $2
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
            user_id,
            entry_type,
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get items by type: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// 搜索程序记忆项
    pub async fn search_by_summary(&self, user_id: &str, summary_query: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>> {
        debug!("Searching procedural items by summary: {}", summary_query);

        let results = sqlx::query_as!(
            ProceduralMemoryItemRow,
            r#"
            SELECT * FROM procedural_memory
            WHERE user_id = $1 AND summary ILIKE $2
            ORDER BY updated_at DESC
            LIMIT $3
            "#,
            user_id,
            format!("%{}%", summary_query),
            limit,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to search procedural items: {}", e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }
}

/// 数据库行结构
#[derive(Debug, sqlx::FromRow)]
struct ProceduralMemoryItemRow {
    id: String,
    organization_id: String,
    user_id: String,
    agent_id: String,
    entry_type: String,
    summary: String,
    steps: serde_json::Value,
    tree_path: serde_json::Value,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ProceduralMemoryItemRow> for ProceduralMemoryItem {
    fn from(row: ProceduralMemoryItemRow) -> Self {
        let steps: Vec<String> = serde_json::from_value(row.steps).unwrap_or_default();
        let tree_path: Vec<String> = serde_json::from_value(row.tree_path).unwrap_or_default();

        Self {
            id: row.id,
            organization_id: row.organization_id,
            user_id: row.user_id,
            agent_id: row.agent_id,
            entry_type: row.entry_type,
            summary: row.summary,
            steps,
            tree_path,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

