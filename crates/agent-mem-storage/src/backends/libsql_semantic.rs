//! LibSQL implementation of SemanticMemoryStore

use agent_mem_traits::{
    AgentMemError, Result, SemanticMemoryItem, SemanticMemoryStore, SemanticQuery,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// LibSQL-based semantic memory store
pub struct LibSqlSemanticStore {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlSemanticStore {
    /// Create a new LibSQL semantic store
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SemanticMemoryStore for LibSqlSemanticStore {
    async fn create_item(&self, item: SemanticMemoryItem) -> Result<SemanticMemoryItem> {
        info!("Creating semantic item: {}", item.id);

        let conn = self.conn.lock().await;

        // Convert tree_path to JSON string
        let tree_path_json = serde_json::to_string(&item.tree_path).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to serialize tree_path: {e}"))
        })?;

        conn.execute(
            r#"
            INSERT INTO semantic_memory (
                id, organization_id, user_id, agent_id, name,
                summary, details, source, tree_path,
                metadata, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            libsql::params![
                item.id.clone(),
                item.organization_id.clone(),
                item.user_id.clone(),
                item.agent_id.clone(),
                item.name.clone(),
                item.summary.clone(),
                item.details.clone(),
                item.source.clone(),
                tree_path_json,
                item.metadata.to_string(),
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| {
            AgentMemError::storage_error(format!("Failed to create semantic item: {e}"))
        })?;

        Ok(item)
    }

    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<SemanticMemoryItem>> {
        debug!("Getting semantic item: {}", item_id);

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT * FROM semantic_memory WHERE id = ? AND user_id = ?")
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![item_id, user_id])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to query item: {e}")))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn query_items(
        &self,
        user_id: &str,
        query: SemanticQuery,
    ) -> Result<Vec<SemanticMemoryItem>> {
        info!("Querying semantic items for user: {}", user_id);

        let mut sql = String::from("SELECT * FROM semantic_memory WHERE user_id = ?");
        let mut params = vec![user_id.to_string()];

        if let Some(name_query) = query.name_query {
            sql.push_str(" AND name LIKE ?");
            params.push(format!("%{name_query}%"));
        }

        if let Some(summary_query) = query.summary_query {
            sql.push_str(" AND summary LIKE ?");
            params.push(format!("%{summary_query}%"));
        }

        // Note: LibSQL doesn't have array operators like PostgreSQL's @>
        // We'll need to do tree_path filtering in application code if needed

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params.push(limit.to_string());
        }

        let conn = self.conn.lock().await;
        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare query: {e}")))?;

        // Build params tuple for libsql
        let mut rows = if params.len() == 1 {
            stmt.query(libsql::params![params[0].clone()]).await
        } else if params.len() == 2 {
            stmt.query(libsql::params![params[0].clone(), params[1].clone()])
                .await
        } else if params.len() == 3 {
            stmt.query(libsql::params![
                params[0].clone(),
                params[1].clone(),
                params[2].clone()
            ])
            .await
        } else if params.len() == 4 {
            stmt.query(libsql::params![
                params[0].clone(),
                params[1].clone(),
                params[2].clone(),
                params[3].clone()
            ])
            .await
        } else {
            stmt.query(libsql::params![params[0].clone()]).await
        }
        .map_err(|e| AgentMemError::storage_error(format!("Failed to query items: {e}")))?;

        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            let item = row_to_item(&row)?;

            // Filter by tree_path_prefix if specified
            if let Some(ref prefix) = query.tree_path_prefix {
                if item.tree_path.starts_with(prefix) {
                    items.push(item);
                }
            } else {
                items.push(item);
            }
        }

        Ok(items)
    }

    async fn update_item(&self, item: SemanticMemoryItem) -> Result<bool> {
        debug!("Updating semantic item: {}", item.id);

        let conn = self.conn.lock().await;

        let tree_path_json = serde_json::to_string(&item.tree_path).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to serialize tree_path: {e}"))
        })?;

        let result = conn
            .execute(
                r#"
            UPDATE semantic_memory
            SET name = ?, summary = ?, details = ?, source = ?,
                tree_path = ?, metadata = ?, updated_at = ?
            WHERE id = ? AND user_id = ?
            "#,
                libsql::params![
                    item.name,
                    item.summary,
                    item.details,
                    item.source,
                    tree_path_json,
                    item.metadata.to_string(),
                    Utc::now().to_rfc3339(),
                    item.id,
                    item.user_id,
                ],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to update item: {e}")))?;

        Ok(result > 0)
    }

    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        debug!("Deleting semantic item: {}", item_id);

        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM semantic_memory WHERE id = ? AND user_id = ?",
                libsql::params![item_id, user_id],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to delete item: {e}")))?;

        Ok(result > 0)
    }

    async fn search_by_tree_path(
        &self,
        user_id: &str,
        tree_path: Vec<String>,
    ) -> Result<Vec<SemanticMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT * FROM semantic_memory WHERE user_id = ? ORDER BY created_at DESC")
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![user_id])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to query items: {e}")))?;

        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            let item = row_to_item(&row)?;
            // Filter by tree_path prefix
            if item.tree_path.starts_with(&tree_path) {
                items.push(item);
            }
        }

        Ok(items)
    }

    async fn search_by_name(
        &self,
        user_id: &str,
        name_pattern: &str,
        limit: i64,
    ) -> Result<Vec<SemanticMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT * FROM semantic_memory WHERE user_id = ? AND name LIKE ? ORDER BY created_at DESC LIMIT ?")
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query(libsql::params![
                user_id,
                format!("%{}%", name_pattern),
                limit
            ])
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to query items: {e}")))?;

        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Failed to fetch row: {e}")))?
        {
            items.push(row_to_item(&row)?);
        }

        Ok(items)
    }
}

/// Convert LibSQL row to SemanticMemoryItem
fn row_to_item(row: &libsql::Row) -> Result<SemanticMemoryItem> {
    Ok(SemanticMemoryItem {
        id: row
            .get(0)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get id: {e}")))?,
        organization_id: row.get(1).map_err(|e| {
            AgentMemError::storage_error(format!("Failed to get organization_id: {e}"))
        })?,
        user_id: row
            .get(2)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get user_id: {e}")))?,
        agent_id: row
            .get(3)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get agent_id: {e}")))?,
        name: row
            .get(4)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get name: {e}")))?,
        summary: row
            .get(5)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get summary: {e}")))?,
        details: row
            .get(6)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get details: {e}")))?,
        source: row
            .get(7)
            .map_err(|e| AgentMemError::storage_error(format!("Failed to get source: {e}")))?,
        tree_path: {
            let s: String = row.get(8).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to get tree_path: {e}"))
            })?;
            serde_json::from_str(&s).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to parse tree_path: {e}"))
            })?
        },
        metadata: {
            let s: String = row.get(9).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to get metadata: {e}"))
            })?;
            serde_json::from_str(&s).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to parse metadata: {e}"))
            })?
        },
        created_at: {
            let s: String = row.get(10).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to get created_at: {e}"))
            })?;
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    AgentMemError::storage_error(format!("Failed to parse created_at: {e}"))
                })?
                .with_timezone(&Utc)
        },
        updated_at: {
            let s: String = row.get(11).map_err(|e| {
                AgentMemError::storage_error(format!("Failed to get updated_at: {e}"))
            })?;
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    AgentMemError::storage_error(format!("Failed to parse updated_at: {e}"))
                })?
                .with_timezone(&Utc)
        },
    })
}
