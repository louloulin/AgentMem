//! LibSQL implementation of ProceduralMemoryStore

use agent_mem_traits::{
    AgentMemError, ProceduralMemoryItem, ProceduralMemoryStore, ProceduralQuery, Result,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::{params, Connection, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of ProceduralMemoryStore
pub struct LibSqlProceduralStore {
    conn: Arc<Mutex<Connection>>,
}

impl LibSqlProceduralStore {
    /// Create a new LibSQL procedural memory store
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

/// Convert LibSQL row to ProceduralMemoryItem
fn row_to_item(row: &Row) -> Result<ProceduralMemoryItem> {
    let steps_json: String = row.get(6).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get steps: {}", e))
    })?;
    let steps: Vec<String> = serde_json::from_str(&steps_json).unwrap_or_default();

    let metadata_json: String = row.get(9).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get metadata: {}", e))
    })?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let created_at_str: String = row.get(10).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get created_at: {}", e))
    })?;
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse created_at: {}", e)))?
        .with_timezone(&Utc);

    let updated_at_str: String = row.get(11).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get updated_at: {}", e))
    })?;
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to parse updated_at: {}", e)))?
        .with_timezone(&Utc);

    let success_rate_f64: f64 = row.get(7).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get success_rate: {}", e))
    })?;

    let execution_count_i64: i64 = row.get(8).map_err(|e| {
        AgentMemError::storage_error(&format!("Failed to get execution_count: {}", e))
    })?;

    Ok(ProceduralMemoryItem {
        id: row.get(0).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get id: {}", e))
        })?,
        organization_id: row.get(1).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get organization_id: {}", e))
        })?,
        user_id: row.get(2).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get user_id: {}", e))
        })?,
        agent_id: row.get(3).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get agent_id: {}", e))
        })?,
        skill_name: row.get(4).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get skill_name: {}", e))
        })?,
        description: row.get(5).map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to get description: {}", e))
        })?,
        steps,
        success_rate: success_rate_f64 as f32,
        execution_count: execution_count_i64 as i32,
        metadata,
        created_at,
        updated_at,
    })
}

#[async_trait]
impl ProceduralMemoryStore for LibSqlProceduralStore {
    async fn create_item(&self, item: ProceduralMemoryItem) -> Result<ProceduralMemoryItem> {
        let conn = self.conn.lock().await;

        let steps_json = serde_json::to_string(&item.steps)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e)))?;

        let metadata_json = serde_json::to_string(&item.metadata)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize metadata: {}", e)))?;

        conn.execute(
            r#"
            INSERT INTO procedural_memory (
                id, organization_id, user_id, agent_id, skill_name,
                description, steps, success_rate, execution_count, metadata,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                item.id.clone(),
                item.organization_id.clone(),
                item.user_id.clone(),
                item.agent_id.clone(),
                item.skill_name.clone(),
                item.description.clone(),
                steps_json,
                item.success_rate as f64,
                item.execution_count as i64,
                metadata_json,
                item.created_at.to_rfc3339(),
                item.updated_at.to_rfc3339(),
            ],
        )
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create procedural item: {}", e)))?;

        Ok(item)
    }

    async fn get_item(&self, item_id: &str, user_id: &str) -> Result<Option<ProceduralMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT * FROM procedural_memory WHERE id = ? AND user_id = ?")
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(params![item_id, user_id])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to execute query: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to fetch row: {}", e))
        })? {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    async fn query_items(&self, user_id: &str, query: ProceduralQuery) -> Result<Vec<ProceduralMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut sql = String::from("SELECT * FROM procedural_memory WHERE user_id = ?");
        let mut params_vec: Vec<libsql::Value> = vec![libsql::Value::from(user_id.to_string())];

        if let Some(ref skill_name_pattern) = query.skill_name_pattern {
            sql.push_str(" AND skill_name LIKE ?");
            params_vec.push(libsql::Value::from(format!("%{}%", skill_name_pattern)));
        }

        if let Some(min_success_rate) = query.min_success_rate {
            sql.push_str(" AND success_rate >= ?");
            params_vec.push(libsql::Value::from(min_success_rate as f64));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(" LIMIT ?");
            params_vec.push(libsql::Value::from(limit));
        }

        let mut stmt = conn
            .prepare(&sql)
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare statement: {}", e)))?;

        // LibSQL doesn't support &[Value] directly, need to build params dynamically
        let mut rows = match params_vec.len() {
            1 => stmt.query(params![params_vec[0].clone()]).await,
            2 => stmt.query(params![params_vec[0].clone(), params_vec[1].clone()]).await,
            3 => stmt.query(params![params_vec[0].clone(), params_vec[1].clone(), params_vec[2].clone()]).await,
            _ => stmt.query(params![params_vec[0].clone()]).await,
        }
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to fetch row: {}", e))
        })? {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }

    async fn update_item(&self, item: ProceduralMemoryItem) -> Result<bool> {
        let conn = self.conn.lock().await;

        let steps_json = serde_json::to_string(&item.steps)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize steps: {}", e)))?;

        let metadata_json = serde_json::to_string(&item.metadata)
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to serialize metadata: {}", e)))?;

        let result = conn
            .execute(
                r#"
                UPDATE procedural_memory
                SET skill_name = ?, description = ?, steps = ?,
                    success_rate = ?, execution_count = ?, metadata = ?, updated_at = ?
                WHERE id = ? AND user_id = ?
                "#,
                params![
                    item.skill_name,
                    item.description,
                    steps_json,
                    item.success_rate as f64,
                    item.execution_count as i64,
                    metadata_json,
                    item.updated_at.to_rfc3339(),
                    item.id,
                    item.user_id,
                ],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to update procedural item: {}", e)))?;

        Ok(result > 0)
    }

    async fn delete_item(&self, item_id: &str, user_id: &str) -> Result<bool> {
        let conn = self.conn.lock().await;

        let result = conn
            .execute(
                "DELETE FROM procedural_memory WHERE id = ? AND user_id = ?",
                params![item_id, user_id],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to delete procedural item: {}", e)))?;

        Ok(result > 0)
    }

    async fn update_execution_stats(
        &self,
        item_id: &str,
        user_id: &str,
        success: bool,
    ) -> Result<bool> {
        let conn = self.conn.lock().await;

        let success_value = if success { 1.0 } else { 0.0 };

        let result = conn
            .execute(
                r#"
                UPDATE procedural_memory
                SET execution_count = execution_count + 1,
                    success_rate = CASE
                        WHEN ? = 1.0 THEN (success_rate * execution_count + 1.0) / (execution_count + 1)
                        ELSE (success_rate * execution_count) / (execution_count + 1)
                    END,
                    updated_at = datetime('now')
                WHERE id = ? AND user_id = ?
                "#,
                params![success_value, item_id, user_id],
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to update execution stats: {}", e)))?;

        Ok(result > 0)
    }

    async fn get_top_skills(&self, user_id: &str, limit: i64) -> Result<Vec<ProceduralMemoryItem>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT * FROM procedural_memory
                WHERE user_id = ?
                ORDER BY success_rate DESC, execution_count DESC
                LIMIT ?
                "#,
            )
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to prepare statement: {}", e)))?;

        let mut rows = stmt
            .query(params![user_id, limit])
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to execute query: {}", e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to fetch row: {}", e))
        })? {
            results.push(row_to_item(&row)?);
        }

        Ok(results)
    }
}

