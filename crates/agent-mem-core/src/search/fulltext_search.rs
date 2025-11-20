//! 全文搜索引擎
//!
//! 提供基于 PostgreSQL 全文搜索的关键词搜索功能

use super::{SearchFilters, SearchQuery, SearchResult};
use agent_mem_traits::{AgentMemError, Result};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::Instant;

/// 全文搜索引擎
pub struct FullTextSearchEngine {
    /// 数据库连接池
    pool: Arc<PgPool>,
}

impl FullTextSearchEngine {
    /// 创建新的全文搜索引擎
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL 连接池
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 执行全文搜索
    ///
    /// # Arguments
    ///
    /// * `query` - 搜索查询参数
    ///
    /// # Returns
    ///
    /// 返回搜索结果列表和搜索时间（毫秒）
    pub async fn search(&self, query: &SearchQuery) -> Result<(Vec<SearchResult>, u64)> {
        let start = Instant::now();

        // 构建 SQL 查询
        let sql = self.build_search_query(query)?;

        // 执行查询
        let rows = sqlx::query(&sql)
            .bind(&query.query)
            .bind(query.limit as i64)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(format!("Full-text search failed: {}", e)))?;

        // 转换为搜索结果
        let results = rows
            .into_iter()
            .map(|row| {
                let id: String = row.try_get("id").unwrap_or_default();
                let content: String = row.try_get("content").unwrap_or_default();
                let rank: f32 = row.try_get("rank").unwrap_or(0.0);

                SearchResult {
                    id,
                    content,
                    score: rank,
                    vector_score: None,
                    fulltext_score: Some(rank),
                    metadata: None,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_millis() as u64;

        Ok((results, elapsed))
    }

    /// 构建搜索 SQL 查询
    fn build_search_query(&self, query: &SearchQuery) -> Result<String> {
        let mut sql = String::from(
            r#"
            SELECT 
                id,
                content,
                ts_rank(search_vector, plainto_tsquery('english', $1)) as rank
            FROM memories
            WHERE search_vector @@ plainto_tsquery('english', $1)
            "#,
        );

        // 添加过滤条件
        if let Some(filters) = &query.filters {
            sql.push_str(&self.build_filter_clause(filters)?);
        }

        // 添加排序和限制
        sql.push_str(" ORDER BY rank DESC LIMIT $2");

        Ok(sql)
    }

    /// 构建过滤条件 SQL
    fn build_filter_clause(&self, filters: &SearchFilters) -> Result<String> {
        let mut clauses = Vec::new();

        if let Some(user_id) = &filters.user_id {
            clauses.push(format!("user_id = '{}'", user_id));
        }

        if let Some(org_id) = &filters.organization_id {
            clauses.push(format!("organization_id = '{}'", org_id));
        }

        if let Some(agent_id) = &filters.agent_id {
            clauses.push(format!("agent_id = '{}'", agent_id));
        }

        if let Some(start_time) = &filters.start_time {
            clauses.push(format!("created_at >= '{}'", start_time.to_rfc3339()));
        }

        if let Some(end_time) = &filters.end_time {
            clauses.push(format!("created_at <= '{}'", end_time.to_rfc3339()));
        }

        if let Some(tags) = &filters.tags {
            if !tags.is_empty() {
                let tags_str = tags
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<_>>()
                    .join(",");
                clauses.push(format!("tags && ARRAY[{}]", tags_str));
            }
        }

        if clauses.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!(" AND {}", clauses.join(" AND ")))
        }
    }

    /// 获取搜索统计信息
    pub async fn get_stats(&self) -> Result<FullTextSearchStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_documents,
                COUNT(CASE WHEN search_vector IS NOT NULL THEN 1 END) as indexed_documents
            FROM memories
            "#,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(format!("Failed to get stats: {}", e)))?;

        let total_documents: i64 = row.try_get("total_documents").unwrap_or(0);
        let indexed_documents: i64 = row.try_get("indexed_documents").unwrap_or(0);

        Ok(FullTextSearchStats {
            total_documents: total_documents as usize,
            indexed_documents: indexed_documents as usize,
        })
    }

    /// 重建全文搜索索引
    pub async fn rebuild_index(&self) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE memories
            SET search_vector = to_tsvector('english', content)
            WHERE search_vector IS NULL
            "#,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(format!("Failed to rebuild index: {}", e)))?;

        Ok(())
    }
}

// ============================================================================
// SearchEngine Trait 实现 (V4)
// ============================================================================

use agent_mem_traits::{Query, QueryIntent, QueryIntentType, SearchEngine};
use async_trait::async_trait;

#[async_trait]
impl SearchEngine for FullTextSearchEngine {
    /// 执行搜索查询（V4 Query 接口）
    async fn search(&self, query: &Query) -> Result<Vec<agent_mem_traits::SearchResultV4>> {
        // 1. 提取查询文本
        let query_text = match &query.intent {
            QueryIntent::NaturalLanguage { text, .. } => text.clone(),
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取自然语言意图
                intents
                    .iter()
                    .find_map(|intent| {
                        if let QueryIntent::NaturalLanguage { text, .. } = intent {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            }
            _ => {
                return Err(AgentMemError::validation_error(
                    format!("Unsupported query intent for FullTextSearchEngine: {:?}. Use NaturalLanguage or Hybrid intent.", query.intent)
                ));
            }
        };

        // 2. 转换 Query V4 到 SearchQuery
        let mut search_query = SearchQuery::from_query_v4(query);
        search_query.query = query_text;

        // 3. 执行全文搜索（使用现有的 search 方法）
        let (results, _elapsed) = self.search(&search_query).await?;

        // 4. 转换 SearchResult 到 SearchResultV4
        let v4_results = results
            .into_iter()
            .map(|r| agent_mem_traits::SearchResultV4 {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: r.vector_score,
                fulltext_score: r.fulltext_score,
                metadata: r.metadata,
            })
            .collect();

        Ok(v4_results)
    }

    /// 获取引擎名称
    fn name(&self) -> &str {
        "FullTextSearchEngine"
    }

    /// 获取支持的查询意图类型
    fn supported_intents(&self) -> Vec<QueryIntentType> {
        vec![
            QueryIntentType::NaturalLanguage, // 主要支持自然语言查询
            QueryIntentType::Hybrid,          // 也支持混合查询（提取文本部分）
        ]
    }
}

/// 全文搜索统计信息
#[derive(Debug, Clone)]
pub struct FullTextSearchStats {
    /// 总文档数
    pub total_documents: usize,
    /// 已索引文档数
    pub indexed_documents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_query() {
        // 注意：这个测试需要实际的数据库连接，这里只测试 SQL 构建逻辑
        let _query = SearchQuery {
            query: "test query".to_string(),
            limit: 10,
            threshold: Some(0.7),
            ..Default::default()
        };

        // 创建一个模拟的引擎（不需要实际连接）
        // 实际测试需要在集成测试中进行
    }

    #[test]
    fn test_build_filter_clause() {
        let _filters = SearchFilters {
            user_id: Some("user-123".to_string()),
            organization_id: Some("org-456".to_string()),
            agent_id: None,
            start_time: None,
            end_time: None,
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        };

        // 测试过滤条件构建
        // 实际测试需要在集成测试中进行
    }
}
