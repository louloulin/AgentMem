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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Helper function to create test semantic item
    fn create_test_item(id: &str, name: &str, tree_path: Vec<String>) -> SemanticMemoryItem {
        let now = Utc::now();
        SemanticMemoryItem {
            id: id.to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: name.to_string(),
            summary: format!("Summary of {}", name),
            details: format!("Detailed information about {}", name),
            source: Some("test-source".to_string()),
            tree_path,
            metadata: json!({"category": "test"}),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_semantic_item_creation() {
        let item = create_test_item(
            "item-1",
            "Machine Learning",
            vec!["AI".to_string(), "ML".to_string()],
        );

        assert_eq!(item.id, "item-1");
        assert_eq!(item.name, "Machine Learning");
        assert_eq!(item.tree_path.len(), 2);
        assert_eq!(item.tree_path[0], "AI");
        assert_eq!(item.tree_path[1], "ML");
    }

    #[test]
    fn test_semantic_item_serialization() {
        let item = create_test_item(
            "item-2",
            "Deep Learning",
            vec!["AI".to_string(), "ML".to_string(), "DL".to_string()],
        );

        // Test serialization
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("item-2"));
        assert!(json.contains("Deep Learning"));

        // Test deserialization
        let deserialized: SemanticMemoryItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, item.id);
        assert_eq!(deserialized.name, item.name);
        assert_eq!(deserialized.tree_path, item.tree_path);
    }

    #[test]
    fn test_semantic_query_default() {
        let query = SemanticQuery {
            name_query: None,
            summary_query: None,
            tree_path_prefix: None,
            limit: None,
        };

        assert!(query.name_query.is_none());
        assert!(query.summary_query.is_none());
        assert!(query.tree_path_prefix.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_semantic_query_with_filters() {
        let query = SemanticQuery {
            name_query: Some("machine learning".to_string()),
            summary_query: Some("AI concepts".to_string()),
            tree_path_prefix: Some(vec!["AI".to_string(), "ML".to_string()]),
            limit: Some(20),
        };

        assert_eq!(query.name_query.unwrap(), "machine learning");
        assert_eq!(query.summary_query.unwrap(), "AI concepts");
        assert_eq!(query.tree_path_prefix.unwrap().len(), 2);
        assert_eq!(query.limit.unwrap(), 20);
    }

    #[test]
    fn test_tree_path_hierarchy() {
        let item1 = create_test_item("item-1", "Concept", vec!["root".to_string()]);
        let item2 = create_test_item(
            "item-2",
            "Subconcept",
            vec!["root".to_string(), "child".to_string()],
        );
        let item3 = create_test_item(
            "item-3",
            "Deep Concept",
            vec![
                "root".to_string(),
                "child".to_string(),
                "grandchild".to_string(),
            ],
        );

        assert_eq!(item1.tree_path.len(), 1);
        assert_eq!(item2.tree_path.len(), 2);
        assert_eq!(item3.tree_path.len(), 3);

        // Verify hierarchy
        assert_eq!(item1.tree_path[0], "root");
        assert_eq!(item2.tree_path[0], "root");
        assert_eq!(item3.tree_path[0], "root");
    }

    #[test]
    fn test_semantic_item_metadata() {
        let item = create_test_item("item-meta", "Test", vec!["test".to_string()]);

        assert!(item.metadata.is_object());
        assert_eq!(item.metadata["category"], json!("test"));
    }

    #[test]
    fn test_semantic_item_optional_source() {
        let mut item = create_test_item("item-source", "Test", vec!["test".to_string()]);

        // Test with source
        assert!(item.source.is_some());
        assert_eq!(item.source.unwrap(), "test-source");

        // Test without source
        item.source = None;
        assert!(item.source.is_none());
    }

    #[test]
    fn test_semantic_item_timestamps() {
        let item = create_test_item("item-time", "Test", vec!["test".to_string()]);

        // Timestamps should be set
        assert!(item.created_at <= Utc::now());
        assert!(item.updated_at <= Utc::now());

        // Created and updated should be close
        let diff = item.updated_at - item.created_at;
        assert!(diff.num_seconds().abs() < 1);
    }

    #[test]
    fn test_empty_tree_path() {
        let item = create_test_item("item-empty", "Root Concept", vec![]);

        assert_eq!(item.tree_path.len(), 0);
        assert!(item.tree_path.is_empty());
    }

    #[test]
    fn test_complex_tree_path() {
        let complex_path = vec![
            "favorites".to_string(),
            "pets".to_string(),
            "dogs".to_string(),
            "breeds".to_string(),
            "golden_retriever".to_string(),
        ];

        let item = create_test_item("item-complex", "Golden Retriever", complex_path.clone());

        assert_eq!(item.tree_path.len(), 5);
        assert_eq!(item.tree_path, complex_path);
    }

    #[test]
    fn test_semantic_item_with_empty_strings() {
        let now = Utc::now();
        let item = SemanticMemoryItem {
            id: "".to_string(),
            organization_id: "".to_string(),
            user_id: "".to_string(),
            agent_id: "".to_string(),
            name: "".to_string(),
            summary: "".to_string(),
            details: "".to_string(),
            source: Some("".to_string()),
            tree_path: vec![],
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };

        // Should handle empty strings without panicking
        assert_eq!(item.name, "");
        assert_eq!(item.summary, "");
        assert_eq!(item.tree_path.len(), 0);
    }

    #[test]
    fn test_semantic_item_with_long_content() {
        let long_string = "x".repeat(50000);
        let item = create_test_item("item-long", &long_string, vec!["test".to_string()]);

        assert_eq!(item.name.len(), 50000);
    }

    #[test]
    fn test_query_with_name_filter() {
        let query = SemanticQuery {
            user_id: Some("user-123".to_string()),
            agent_id: Some("agent-456".to_string()),
            name_contains: Some("Machine Learning".to_string()),
            tree_path_prefix: None,
            limit: Some(20),
            offset: Some(0),
        };

        assert_eq!(query.name_contains, Some("Machine Learning".to_string()));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_query_with_tree_path_prefix() {
        let query = SemanticQuery {
            user_id: Some("user-123".to_string()),
            agent_id: None,
            name_contains: None,
            tree_path_prefix: Some(vec!["technology".to_string(), "ai".to_string()]),
            limit: Some(100),
            offset: None,
        };

        assert!(query.tree_path_prefix.is_some());
        assert_eq!(query.tree_path_prefix.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_semantic_item_metadata_nested() {
        let nested_metadata = json!({
            "category": "technology",
            "subcategories": ["ai", "ml", "nlp"],
            "properties": {
                "difficulty": "advanced",
                "prerequisites": ["math", "programming"],
                "estimated_time": 120
            },
            "related_concepts": [
                {"id": "concept-1", "name": "Neural Networks"},
                {"id": "concept-2", "name": "Deep Learning"}
            ]
        });

        let now = Utc::now();
        let item = SemanticMemoryItem {
            id: "item-nested".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: "Machine Learning".to_string(),
            summary: "Advanced ML concepts".to_string(),
            details: "Comprehensive guide".to_string(),
            source: Some("textbook".to_string()),
            tree_path: vec!["technology".to_string(), "ai".to_string()],
            metadata: nested_metadata.clone(),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(item.metadata["category"], "technology");
        assert_eq!(item.metadata["subcategories"].as_array().unwrap().len(), 3);
        assert_eq!(item.metadata["properties"]["difficulty"], "advanced");
        assert_eq!(item.metadata["related_concepts"][0]["name"], "Neural Networks");
    }

    #[test]
    fn test_tree_path_depth_variations() {
        // Depth 1
        let item1 = create_test_item("item-1", "Root", vec!["root".to_string()]);
        assert_eq!(item1.tree_path.len(), 1);

        // Depth 3
        let item3 = create_test_item(
            "item-3",
            "Leaf",
            vec!["root".to_string(), "branch".to_string(), "leaf".to_string()],
        );
        assert_eq!(item3.tree_path.len(), 3);

        // Depth 10
        let deep_path: Vec<String> = (0..10).map(|i| format!("level-{}", i)).collect();
        let item10 = create_test_item("item-10", "Deep", deep_path.clone());
        assert_eq!(item10.tree_path.len(), 10);
    }

    #[test]
    fn test_semantic_item_source_variations() {
        // With source
        let item_with_source = create_test_item("item-1", "Concept", vec!["test".to_string()]);
        assert!(item_with_source.source.is_some());

        // Without source
        let now = Utc::now();
        let item_without_source = SemanticMemoryItem {
            id: "item-2".to_string(),
            organization_id: "test-org".to_string(),
            user_id: "test-user".to_string(),
            agent_id: "test-agent".to_string(),
            name: "Concept".to_string(),
            summary: "Summary".to_string(),
            details: "Details".to_string(),
            source: None,
            tree_path: vec!["test".to_string()],
            metadata: json!({}),
            created_at: now,
            updated_at: now,
        };
        assert!(item_without_source.source.is_none());
    }

    #[test]
    fn test_query_limit_variations() {
        // 小限制
        let query_small = SemanticQuery {
            name_query: None,
            summary_query: None,
            tree_path_prefix: None,
            limit: Some(5),
        };

        // 大限制
        let query_large = SemanticQuery {
            name_query: None,
            summary_query: None,
            tree_path_prefix: None,
            limit: Some(100),
        };

        // 无限制
        let query_unlimited = SemanticQuery {
            name_query: None,
            summary_query: None,
            tree_path_prefix: None,
            limit: None,
        };

        assert_eq!(query_small.limit, Some(5));
        assert_eq!(query_large.limit, Some(100));
        assert!(query_unlimited.limit.is_none());
    }

    #[test]
    fn test_semantic_item_name_variations() {
        // 短名称
        let short_name = create_test_item("item-1", "AI", vec!["tech".to_string()]);
        assert_eq!(short_name.name.len(), 2);

        // 长名称
        let long_name = create_test_item(
            "item-2",
            "Artificial Intelligence and Machine Learning Systems",
            vec!["tech".to_string()],
        );
        assert!(long_name.name.len() > 20);

        // 特殊字符
        let special_chars = create_test_item(
            "item-3",
            "C++ Programming",
            vec!["programming".to_string()],
        );
        assert!(special_chars.name.contains('+'));
    }

    #[test]
    fn test_tree_path_single_vs_multiple() {
        // 单层路径
        let single_level = create_test_item(
            "item-1",
            "Root Concept",
            vec!["root".to_string()],
        );
        assert_eq!(single_level.tree_path.len(), 1);

        // 多层路径
        let multi_level = create_test_item(
            "item-2",
            "Leaf Concept",
            vec![
                "root".to_string(),
                "branch1".to_string(),
                "branch2".to_string(),
                "leaf".to_string(),
            ],
        );
        assert_eq!(multi_level.tree_path.len(), 4);
    }

    #[test]
    fn test_semantic_item_summary_vs_details() {
        let item = create_test_item("item-1", "Machine Learning", vec!["AI".to_string()]);

        // Summary 应该简短
        assert!(!item.summary.is_empty());
        assert!(item.summary.starts_with("Summary of"));

        // Details 应该更详细
        assert!(!item.details.is_empty());
        assert!(item.details.starts_with("Detailed information"));
    }

    #[test]
    fn test_semantic_item_with_complex_tree_path() {
        let complex_path = vec![
            "knowledge".to_string(),
            "technology".to_string(),
            "artificial_intelligence".to_string(),
            "machine_learning".to_string(),
            "deep_learning".to_string(),
            "neural_networks".to_string(),
        ];

        let item = create_test_item("item-1", "CNN Architecture", complex_path.clone());

        assert_eq!(item.tree_path.len(), 6);
        assert_eq!(item.tree_path[0], "knowledge");
        assert_eq!(item.tree_path[5], "neural_networks");
    }

    #[test]
    fn test_query_with_name_and_summary() {
        let query = SemanticQuery {
            name_query: Some("machine learning".to_string()),
            summary_query: Some("artificial intelligence".to_string()),
            tree_path_prefix: None,
            limit: Some(10),
        };

        assert!(query.name_query.is_some());
        assert!(query.summary_query.is_some());
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_semantic_item_id_format() {
        let item1 = create_test_item("item-1", "Concept A", vec!["root".to_string()]);
        let item2 = create_test_item("item-2", "Concept B", vec!["root".to_string()]);

        assert!(!item1.id.is_empty());
        assert!(!item2.id.is_empty());
        assert_ne!(item1.id, item2.id);
    }

    #[test]
    fn test_semantic_item_organization_context() {
        let now = Utc::now();

        let item = SemanticItem {
            id: "item-1".to_string(),
            organization_id: "org-research-lab".to_string(),
            user_id: "researcher-123".to_string(),
            agent_id: "agent-456".to_string(),
            name: "Research Topic".to_string(),
            summary: "Summary of research".to_string(),
            details: "Detailed research information".to_string(),
            tree_path: vec!["research".to_string(), "topics".to_string()],
            source: Some("Academic Paper".to_string()),
            metadata: json!({
                "field": "Computer Science",
                "year": 2025
            }),
            created_at: now,
            updated_at: now,
        };

        assert_eq!(item.organization_id, "org-research-lab");
        assert!(item.metadata["field"].is_string());
    }
}

