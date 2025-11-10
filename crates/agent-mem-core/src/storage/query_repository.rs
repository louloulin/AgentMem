//! Query-based Repository - 基于Query抽象的存储层
//!
//! 直接使用V4.0 Query抽象替代传统的Scope查询

use async_trait::async_trait;
use sqlx::PgPool;

use super::models::DbMemory;
use crate::query::{Query, QueryIntent, Constraint, Preference, AttributeKey};
use crate::{CoreError, CoreResult};

/// 基于Query的存储库
pub struct QueryBasedRepository {
    pool: PgPool,
}

impl QueryBasedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 使用Query抽象进行检索（取代Scope）
    pub async fn retrieve(&self, query: &Query) -> CoreResult<Vec<DbMemory>> {
        // 根据Query构建SQL
        let mut sql = String::from("SELECT * FROM memories WHERE is_deleted = FALSE");
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();
        
        // 1. 处理约束（Constraints）
        for constraint in &query.constraints {
            match constraint {
                Constraint::AttributeMatch { key, value } => {
                    // AttributeMatch替代了原来的Scope！
                    // user::id=xxx 或 agent::id=xxx
                    if key.namespace == "user" && key.name == "id" {
                        conditions.push(format!("user_id = ${}", params.len() + 1));
                        params.push(value.to_string());
                    } else if key.namespace == "agent" && key.name == "id" {
                        conditions.push(format!("agent_id = ${}", params.len() + 1));
                        params.push(value.to_string());
                    } else if key.namespace == "org" && key.name == "id" {
                        conditions.push(format!("organization_id = ${}", params.len() + 1));
                        params.push(value.to_string());
                    }
                    // 其他属性可以从JSON attributes_json字段查询
                    else {
                        let json_path = format!("$.{}.{}", key.namespace, key.name);
                        conditions.push(format!(
                            "JSON_EXTRACT(attributes_json, '{}') = ${}",
                            json_path,
                            params.len() + 1
                        ));
                        params.push(value.to_string());
                    }
                }
                Constraint::TimeRange { start, end } => {
                    if let Some(start_time) = start {
                        conditions.push(format!("created_at >= ${}", params.len() + 1));
                        params.push(start_time.to_rfc3339());
                    }
                    if let Some(end_time) = end {
                        conditions.push(format!("created_at <= ${}", params.len() + 1));
                        params.push(end_time.to_rfc3339());
                    }
                }
                Constraint::Limit(limit) => {
                    sql.push_str(&format!(" LIMIT {}", limit));
                }
                Constraint::MinScore(_score) => {
                    // 向量检索时使用
                }
                _ => {}
            }
        }

        // 添加条件到SQL
        if !conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&conditions.join(" AND "));
        }

        // 2. 处理偏好（Preferences） - 影响排序
        let mut order_clauses = Vec::new();
        for pref in &query.preferences {
            match pref {
                Preference::PreferRecent => {
                    order_clauses.push("created_at DESC");
                }
                Preference::PreferImportant => {
                    order_clauses.push("importance DESC");
                }
                Preference::PreferType { memory_type: _ } => {
                    // 可以加权处理
                }
            }
        }

        // 默认排序
        if order_clauses.is_empty() {
            order_clauses.push("importance DESC");
            order_clauses.push("created_at DESC");
        }

        sql.push_str(" ORDER BY ");
        sql.push_str(&order_clauses.join(", "));

        // 执行查询
        let mut query_builder = sqlx::query_as::<_, Memory>(&sql);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let results = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Query retrieval failed: {}", e)))?;

        Ok(results)
    }

    /// 便捷方法：通过属性查询（替代list_by_agent/list_by_user）
    pub async fn find_by_attribute(
        &self,
        namespace: &str,
        name: &str,
        value: &str,
        limit: Option<i64>,
    ) -> CoreResult<Vec<Memory>> {
        let mut query = Query::new();
        
        // 添加属性约束
        query.add_constraint(Constraint::AttributeMatch {
            key: AttributeKey {
                namespace: namespace.to_string(),
                name: name.to_string(),
            },
            value: crate::types::AttributeValue::String(value.to_string()),
        });

        // 添加limit
        if let Some(lim) = limit {
            query.add_constraint(Constraint::Limit(lim as usize));
        }

        // 偏好最新和重要的
        query.add_preference(Preference::PreferRecent);
        query.add_preference(Preference::PreferImportant);

        self.retrieve(&query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryBuilder;

    #[test]
    fn test_query_replaces_scope() {
        // 原来: list_by_agent("agent-123")
        // 现在: Query with AttributeMatch(agent::id=agent-123)
        let query = QueryBuilder::new()
            .with_attribute("agent", "id", "agent-123")
            .prefer_recent()
            .limit(10)
            .build();

        assert_eq!(query.constraints.len(), 2); // AttributeMatch + Limit
        assert_eq!(query.preferences.len(), 1); // PreferRecent
    }
}

