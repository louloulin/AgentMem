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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper function to create test procedural item
    fn create_test_item(id: &str, entry_type: &str, steps: Vec<String>) -> ProceduralMemoryItem {
        let now = Utc::now();
        ProceduralMemoryItem {
            id: id.to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            entry_type: entry_type.to_string(),
            summary: format!("Test {} procedure", entry_type),
            steps,
            tree_path: vec!["workflows".to_string(), entry_type.to_string()],
            metadata: json!({"test": true}),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_procedural_item_creation() {
        let steps = vec![
            "Step 1: Initialize".to_string(),
            "Step 2: Process".to_string(),
            "Step 3: Finalize".to_string(),
        ];
        let item = create_test_item("proc-1", "workflow", steps.clone());

        assert_eq!(item.id, "proc-1");
        assert_eq!(item.entry_type, "workflow");
        assert_eq!(item.steps.len(), 3);
        assert_eq!(item.steps, steps);
    }

    #[test]
    fn test_procedural_item_serialization() {
        let steps = vec!["Step 1".to_string(), "Step 2".to_string()];
        let item = create_test_item("proc-2", "guide", steps);

        // Test serialization
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("proc-2"));
        assert!(json.contains("guide"));

        // Test deserialization
        let deserialized: ProceduralMemoryItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, item.id);
        assert_eq!(deserialized.entry_type, item.entry_type);
        assert_eq!(deserialized.steps, item.steps);
    }

    #[test]
    fn test_procedural_query_default() {
        let query = ProceduralQuery {
            entry_type: None,
            summary_query: None,
            tree_path_prefix: None,
            limit: None,
        };

        assert!(query.entry_type.is_none());
        assert!(query.summary_query.is_none());
        assert!(query.tree_path_prefix.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_procedural_query_with_filters() {
        let query = ProceduralQuery {
            entry_type: Some("workflow".to_string()),
            summary_query: Some("deployment".to_string()),
            tree_path_prefix: Some(vec!["workflows".to_string(), "ci".to_string()]),
            limit: Some(10),
        };

        assert_eq!(query.entry_type.unwrap(), "workflow");
        assert_eq!(query.summary_query.unwrap(), "deployment");
        assert_eq!(query.tree_path_prefix.unwrap().len(), 2);
        assert_eq!(query.limit.unwrap(), 10);
    }

    #[test]
    fn test_empty_steps() {
        let item = create_test_item("proc-empty", "script", vec![]);

        assert_eq!(item.steps.len(), 0);
        assert!(item.steps.is_empty());
    }

    #[test]
    fn test_single_step() {
        let steps = vec!["Single step".to_string()];
        let item = create_test_item("proc-single", "guide", steps.clone());

        assert_eq!(item.steps.len(), 1);
        assert_eq!(item.steps[0], "Single step");
    }

    #[test]
    fn test_many_steps() {
        let steps: Vec<String> = (1..=20).map(|i| format!("Step {}", i)).collect();
        let item = create_test_item("proc-many", "workflow", steps.clone());

        assert_eq!(item.steps.len(), 20);
        assert_eq!(item.steps[0], "Step 1");
        assert_eq!(item.steps[19], "Step 20");
    }

    #[test]
    fn test_entry_types() {
        let workflow = create_test_item("proc-wf", "workflow", vec!["Step 1".to_string()]);
        let guide = create_test_item("proc-guide", "guide", vec!["Step 1".to_string()]);
        let script = create_test_item("proc-script", "script", vec!["Step 1".to_string()]);

        assert_eq!(workflow.entry_type, "workflow");
        assert_eq!(guide.entry_type, "guide");
        assert_eq!(script.entry_type, "script");
    }

    #[test]
    fn test_tree_path() {
        let item = create_test_item("proc-path", "workflow", vec!["Step 1".to_string()]);

        assert_eq!(item.tree_path.len(), 2);
        assert_eq!(item.tree_path[0], "workflows");
        assert_eq!(item.tree_path[1], "workflow");
    }

    #[test]
    fn test_metadata() {
        let item = create_test_item("proc-meta", "workflow", vec!["Step 1".to_string()]);

        assert!(item.metadata.is_object());
        assert_eq!(item.metadata["test"], json!(true));
    }

    #[test]
    fn test_procedural_item_with_empty_strings() {
        let now = Utc::now();
        let item = ProceduralMemoryItem {
            id: "".to_string(),
            organization_id: "".to_string(),
            user_id: "".to_string(),
            agent_id: "".to_string(),
            name: "".to_string(),
            summary: "".to_string(),
            entry_type: "".to_string(),
            steps: vec![],
            tree_path: vec![],
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        // Should handle empty strings without panicking
        assert_eq!(item.name, "");
        assert_eq!(item.steps.len(), 0);
    }

    #[test]
    fn test_steps_with_long_content() {
        let long_step = "x".repeat(10000);
        let steps = vec![long_step.clone(), long_step.clone()];
        let item = create_test_item("proc-long", "process", steps.clone());

        assert_eq!(item.steps.len(), 2);
        assert_eq!(item.steps[0].len(), 10000);
    }

    #[test]
    fn test_query_with_entry_type_filter() {
        let query = ProceduralQuery {
            user_id: Some("user-123".to_string()),
            agent_id: Some("agent-456".to_string()),
            entry_type: Some("workflow".to_string()),
            name_contains: None,
            limit: Some(50),
            offset: Some(0),
        };

        assert_eq!(query.entry_type, Some("workflow".to_string()));
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_query_with_name_contains() {
        let query = ProceduralQuery {
            user_id: Some("user-123".to_string()),
            agent_id: None,
            entry_type: None,
            name_contains: Some("Deploy".to_string()),
            limit: Some(100),
            offset: None,
        };

        assert_eq!(query.name_contains, Some("Deploy".to_string()));
    }

    #[test]
    fn test_procedural_metadata_complex() {
        let complex_metadata = json!({
            "version": "1.0.0",
            "author": "system",
            "tags": ["automation", "deployment", "production"],
            "requirements": {
                "permissions": ["admin", "deploy"],
                "resources": ["server", "database"]
            },
            "execution_stats": {
                "average_duration": 300,
                "success_rate": 0.95,
                "last_run": "2025-10-07T10:00:00Z"
            }
        });

        let now = Utc::now();
        let item = ProceduralMemoryItem {
            id: "proc-complex".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: "Deploy Application".to_string(),
            summary: "Automated deployment process".to_string(),
            entry_type: "workflow".to_string(),
            steps: vec![
                "Build application".to_string(),
                "Run tests".to_string(),
                "Deploy to staging".to_string(),
                "Run smoke tests".to_string(),
                "Deploy to production".to_string(),
            ],
            tree_path: vec!["automation".to_string(), "deployment".to_string()],
            metadata: complex_metadata.clone(),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(item.metadata["version"], "1.0.0");
        assert_eq!(item.metadata["tags"].as_array().unwrap().len(), 3);
        assert_eq!(item.metadata["requirements"]["permissions"][0], "admin");
        assert_eq!(item.metadata["execution_stats"]["success_rate"], 0.95);
    }

    #[test]
    fn test_steps_ordering() {
        let steps = vec![
            "First step".to_string(),
            "Second step".to_string(),
            "Third step".to_string(),
            "Fourth step".to_string(),
            "Fifth step".to_string(),
        ];

        let item = create_test_item("proc-order", "process", steps.clone());

        // Verify order is preserved
        for (i, step) in item.steps.iter().enumerate() {
            assert_eq!(step, &steps[i]);
        }
    }

    #[test]
    fn test_entry_type_variations() {
        let types = vec!["workflow", "procedure", "algorithm", "recipe", "protocol"];

        for (i, entry_type) in types.iter().enumerate() {
            let item = create_test_item(
                &format!("proc-{}", i),
                entry_type,
                vec!["Step 1".to_string()],
            );
            assert_eq!(item.entry_type, *entry_type);
        }
    }

    #[test]
    fn test_tree_path_variations() {
        // Single level
        let item1 = create_test_item(
            "proc-1",
            "process",
            vec!["Step 1".to_string()],
        );
        assert_eq!(item1.tree_path.len(), 2);

        // Deep nesting
        let now = Utc::now();
        let deep_path: Vec<String> = (0..5).map(|i| format!("level-{}", i)).collect();
        let item_deep = ProceduralMemoryItem {
            id: "proc-deep".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: "Deep Process".to_string(),
            summary: "Deeply nested process".to_string(),
            entry_type: "workflow".to_string(),
            steps: vec!["Step 1".to_string()],
            tree_path: deep_path.clone(),
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(item_deep.tree_path.len(), 5);
    }

    #[test]
    fn test_procedural_item_single_vs_multiple_steps() {
        // 单步骤
        let single_step = ProceduralItem {
            id: "proc-1".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: "Simple Task".to_string(),
            entry_type: "task".to_string(),
            tree_path: vec!["tasks".to_string()],
            steps: vec!["Do it".to_string()],
            metadata: json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // 多步骤
        let multi_steps = create_test_item(
            "proc-2",
            "Complex Workflow",
            "workflow",
            vec!["workflows".to_string()],
        );

        assert_eq!(single_step.steps.len(), 1);
        assert!(multi_steps.steps.len() > 1);
    }

    #[test]
    fn test_procedural_item_name_length() {
        // 短名称
        let short_name = create_test_item("proc-1", "Task", "task", vec!["tasks".to_string()]);
        assert!(short_name.name.len() < 10);

        // 长名称
        let long_name = create_test_item(
            "proc-2",
            "Very Long Procedural Item Name That Describes Complex Process",
            "procedure",
            vec!["procedures".to_string()],
        );
        assert!(long_name.name.len() > 30);
    }

    #[test]
    fn test_query_with_all_filters() {
        let query = ProceduralQuery {
            name_query: Some("workflow".to_string()),
            entry_type: Some("workflow".to_string()),
            tree_path_prefix: Some(vec!["automation".to_string(), "workflows".to_string()]),
            limit: Some(50),
        };

        assert!(query.name_query.is_some());
        assert!(query.entry_type.is_some());
        assert!(query.tree_path_prefix.is_some());
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_query_with_no_filters() {
        let query = ProceduralQuery {
            name_query: None,
            entry_type: None,
            tree_path_prefix: None,
            limit: None,
        };

        assert!(query.name_query.is_none());
        assert!(query.entry_type.is_none());
        assert!(query.tree_path_prefix.is_none());
        assert!(query.limit.is_none());
    }
}

