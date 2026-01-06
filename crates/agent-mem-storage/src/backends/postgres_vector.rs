//! PostgreSQL + pgvector implementation of VectorStore
//!
//! This module provides a VectorStore implementation using PostgreSQL with the pgvector extension.
//! It supports:
//! - Single and multi-vector fields
//! - Multiple distance operators (cosine, L2, inner product)
//! - Efficient vector indexing (IVFFlat, HNSW)
//! - Batch operations

use agent_mem_traits::{
    AgentMemError, HealthStatus, Result, VectorData, VectorSearchResult, VectorStore,
    VectorStoreStats,
};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// pgvector 距离操作符
#[derive(Debug, Clone, Copy)]
pub enum VectorDistanceOperator {
    /// 余弦距离: <=>
    /// 范围: [0, 2]，0 表示完全相同，2 表示完全相反
    Cosine,
    /// L2 距离 (欧几里得距离): <->
    /// 范围: [0, ∞)，0 表示完全相同
    L2,
    /// 内积 (负值): <#>
    /// 范围: (-∞, 0]，值越大表示越相似
    InnerProduct,
}

impl VectorDistanceOperator {
    /// 获取 SQL 操作符
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Cosine => "<=>",
            Self::L2 => "<->",
            Self::InnerProduct => "<#>",
        }
    }

    /// 将距离转换为相似度 (0-1 范围)
    pub fn distance_to_similarity(&self, distance: f32) -> f32 {
        match self {
            Self::Cosine => 1.0 - (distance / 2.0), // 余弦距离 [0,2] -> 相似度 [1,0]
            Self::L2 => 1.0 / (1.0 + distance),     // L2 距离 [0,∞) -> 相似度 [1,0)
            Self::InnerProduct => {
                // 内积是负值，越大越相似
                if distance <= 0.0 {
                    (-distance).min(1.0)
                } else {
                    0.0
                }
            }
        }
    }
}

/// PostgreSQL + pgvector 向量存储配置
#[derive(Debug, Clone)]
pub struct PostgresVectorConfig {
    /// 表名
    pub table_name: String,
    /// 向量列名
    pub vector_column: String,
    /// 向量维度
    pub dimension: usize,
    /// 距离操作符
    pub distance_operator: VectorDistanceOperator,
    /// 是否自动创建表
    pub auto_create_table: bool,
    /// 索引类型 (ivfflat 或 hnsw)
    pub index_type: Option<String>,
}

impl Default for PostgresVectorConfig {
    fn default() -> Self {
        Self {
            table_name: "vectors".to_string(),
            vector_column: "embedding".to_string(),
            dimension: 1536,
            distance_operator: VectorDistanceOperator::Cosine,
            auto_create_table: true,
            index_type: Some("ivfflat".to_string()),
        }
    }
}

/// PostgreSQL + pgvector 向量存储
pub struct PostgresVectorStore {
    pool: Arc<PgPool>,
    config: PostgresVectorConfig,
}

impl PostgresVectorStore {
    /// 创建新的 PostgreSQL 向量存储
    pub fn new(pool: Arc<PgPool>, config: PostgresVectorConfig) -> Self {
        Self { pool, config }
    }

    /// 创建默认配置的向量存储
    pub fn with_defaults(pool: Arc<PgPool>) -> Self {
        Self::new(pool, PostgresVectorConfig::default())
    }

    /// 确保表存在
    async fn ensure_table_exists(&self) -> Result<()> {
        if !self.config.auto_create_table {
            return Ok(());
        }

        let table_name = &self.config.table_name;
        let vector_column = &self.config.vector_column;
        let dimension = self.config.dimension;

        // 启用 pgvector 扩展
        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to create vector extension: {}", e))
            })?;

        // 创建表
        let create_table_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                {} vector({}),
                metadata JSONB DEFAULT '{{}}'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            table_name, vector_column, dimension
        );

        sqlx::query(&create_table_sql)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to create table: {}", e)))?;

        // 创建向量索引
        if let Some(index_type) = &self.config.index_type {
            self.create_vector_index(index_type).await?;
        }

        Ok(())
    }

    /// 创建向量索引
    async fn create_vector_index(&self, index_type: &str) -> Result<()> {
        let table_name = &self.config.table_name;
        let vector_column = &self.config.vector_column;
        let operator = self.config.distance_operator;

        let ops = match operator {
            VectorDistanceOperator::Cosine => "vector_cosine_ops",
            VectorDistanceOperator::L2 => "vector_l2_ops",
            VectorDistanceOperator::InnerProduct => "vector_ip_ops",
        };

        let index_sql = match index_type {
            "ivfflat" => {
                format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_{}_ivfflat ON {} USING ivfflat ({} {}) WITH (lists = 100)",
                    table_name, vector_column, table_name, vector_column, ops
                )
            }
            "hnsw" => {
                format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_{}_hnsw ON {} USING hnsw ({} {}) WITH (m = 16, ef_construction = 64)",
                    table_name, vector_column, table_name, vector_column, ops
                )
            }
            _ => {
                warn!(
                    "Unknown index type: {}, skipping index creation",
                    index_type
                );
                return Ok(());
            }
        };

        sqlx::query(&index_sql)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to create vector index: {}", e))
            })?;

        info!(
            "Created {} index on {}.{}",
            index_type, table_name, vector_column
        );

        Ok(())
    }

    /// 将元数据 HashMap 转换为 JSONB
    fn metadata_to_jsonb(metadata: &HashMap<String, String>) -> JsonValue {
        let map: serde_json::Map<String, JsonValue> = metadata
            .iter()
            .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
            .collect();
        JsonValue::Object(map)
    }

    /// 将 JSONB 转换为元数据 HashMap
    fn jsonb_to_metadata(value: &JsonValue) -> HashMap<String, String> {
        match value {
            JsonValue::Object(map) => map
                .iter()
                .filter_map(|(k, v)| {
                    if let JsonValue::String(s) = v {
                        Some((k.clone(), s.clone()))
                    } else {
                        Some((k.clone(), v.to_string()))
                    }
                })
                .collect(),
            _ => HashMap::new(),
        }
    }
}

#[async_trait]
impl VectorStore for PostgresVectorStore {
    async fn add_vectors(&self, vectors: Vec<VectorData>) -> Result<Vec<String>> {
        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        self.ensure_table_exists().await?;

        let table_name = &self.config.table_name;
        let vector_column = &self.config.vector_column;

        let mut ids = Vec::new();

        for vector_data in vectors {
            let metadata_json = Self::metadata_to_jsonb(&vector_data.metadata);

            // 将 Vec<f32> 转换为 pgvector 格式的字符串
            let vector_str = format!(
                "[{}]",
                vector_data
                    .vector
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let insert_sql = format!(
                r#"
                INSERT INTO {} (id, {}, metadata, created_at, updated_at)
                VALUES ($1, $2::vector, $3, NOW(), NOW())
                ON CONFLICT (id) DO UPDATE
                SET {} = EXCLUDED.{},
                    metadata = EXCLUDED.metadata,
                    updated_at = NOW()
                "#,
                table_name, vector_column, vector_column, vector_column
            );

            sqlx::query(&insert_sql)
                .bind(&vector_data.id)
                .bind(&vector_str)
                .bind(&metadata_json)
                .execute(self.pool.as_ref())
                .await
                .map_err(|e| {
                    AgentMemError::storage_error(&format!("Failed to insert vector: {}", e))
                })?;

            ids.push(vector_data.id);
        }

        info!("Inserted {} vectors", ids.len());
        Ok(ids)
    }

    async fn search_vectors(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        self.search_with_filters(query_vector, limit, &HashMap::new(), threshold)
            .await
    }

    async fn search_with_filters(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filters: &HashMap<String, JsonValue>,
        threshold: Option<f32>,
    ) -> Result<Vec<VectorSearchResult>> {
        let table_name = &self.config.table_name;
        let vector_column = &self.config.vector_column;
        let operator = self.config.distance_operator.to_sql();

        // 将查询向量转换为 pgvector 格式
        let query_vector_str = format!(
            "[{}]",
            query_vector
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        // 构建 SQL 查询
        let mut sql = format!(
            r#"
            SELECT id, {}, metadata, ({} {} $1::vector) as distance
            FROM {}
            "#,
            vector_column, vector_column, operator, table_name
        );

        // 添加元数据过滤条件
        if !filters.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = filters
                .iter()
                .map(|(k, v)| format!("metadata->>'{}' = '{}'", k, v.as_str().unwrap_or("")))
                .collect();
            sql.push_str(&conditions.join(" AND "));
        }

        // 添加阈值过滤
        if let Some(threshold) = threshold {
            let similarity_threshold = match self.config.distance_operator {
                VectorDistanceOperator::Cosine => 2.0 * (1.0 - threshold), // 相似度 -> 距离
                VectorDistanceOperator::L2 => (1.0 / threshold) - 1.0,
                VectorDistanceOperator::InnerProduct => -threshold,
            };

            if filters.is_empty() {
                sql.push_str(&format!(" WHERE distance <= {}", similarity_threshold));
            } else {
                sql.push_str(&format!(" AND distance <= {}", similarity_threshold));
            }
        }

        sql.push_str(&format!(" ORDER BY distance LIMIT {}", limit));

        debug!("Executing vector search SQL: {}", sql);

        let rows = sqlx::query(&sql)
            .bind(&query_vector_str)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to search vectors: {}", e))
            })?;

        let mut results = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to get id: {}", e)))?;

            let vector_str: String = row.try_get(vector_column.as_str()).map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to get vector: {}", e))
            })?;

            let metadata_json: JsonValue = row.try_get("metadata").map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to get metadata: {}", e))
            })?;

            let distance: f32 = row.try_get("distance").map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to get distance: {}", e))
            })?;

            // 解析向量字符串 "[0.1,0.2,...]" -> Vec<f32>
            let vector: Vec<f32> = vector_str
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            let similarity = self
                .config
                .distance_operator
                .distance_to_similarity(distance);

            results.push(VectorSearchResult {
                id,
                vector,
                metadata: Self::jsonb_to_metadata(&metadata_json),
                similarity,
                distance,
            });
        }

        debug!("Found {} similar vectors", results.len());
        Ok(results)
    }

    async fn delete_vectors(&self, ids: Vec<String>) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let table_name = &self.config.table_name;

        // 构建 IN 子句
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
        let delete_sql = format!(
            "DELETE FROM {} WHERE id IN ({})",
            table_name,
            placeholders.join(",")
        );

        let mut query = sqlx::query(&delete_sql);
        for id in &ids {
            query = query.bind(id);
        }

        query.execute(self.pool.as_ref()).await.map_err(|e| {
            AgentMemError::storage_error(&format!("Failed to delete vectors: {}", e))
        })?;

        info!("Deleted {} vectors", ids.len());
        Ok(())
    }

    // 其他方法将在下一部分实现...
    async fn update_vectors(&self, vectors: Vec<VectorData>) -> Result<()> {
        // 使用 add_vectors 的 UPSERT 逻辑
        self.add_vectors(vectors).await?;
        Ok(())
    }

    async fn get_vector(&self, id: &str) -> Result<Option<VectorData>> {
        let table_name = &self.config.table_name;
        let vector_column = &self.config.vector_column;

        let sql = format!(
            "SELECT id, {}, metadata FROM {} WHERE id = $1",
            vector_column, table_name
        );

        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to get vector: {}", e)))?;

        if let Some(row) = row {
            let id: String = row
                .try_get("id")
                .map_err(|e| AgentMemError::storage_error(&format!("Failed to get id: {}", e)))?;

            let vector_str: String = row.try_get(vector_column.as_str()).map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to get vector: {}", e))
            })?;

            let metadata_json: JsonValue = row.try_get("metadata").map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to get metadata: {}", e))
            })?;

            let vector: Vec<f32> = vector_str
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            Ok(Some(VectorData {
                id,
                vector,
                metadata: Self::jsonb_to_metadata(&metadata_json),
            }))
        } else {
            Ok(None)
        }
    }

    async fn count_vectors(&self) -> Result<usize> {
        let table_name = &self.config.table_name;
        let sql = format!("SELECT COUNT(*) as count FROM {}", table_name);

        let row = sqlx::query(&sql)
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to count vectors: {}", e))
            })?;

        let count: i64 = row
            .try_get("count")
            .map_err(|e| AgentMemError::storage_error(&format!("Failed to get count: {}", e)))?;

        Ok(count as usize)
    }

    async fn clear(&self) -> Result<()> {
        let table_name = &self.config.table_name;
        let sql = format!("TRUNCATE TABLE {}", table_name);

        sqlx::query(&sql)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| {
                AgentMemError::storage_error(&format!("Failed to clear vectors: {}", e))
            })?;

        info!("Cleared all vectors from table {}", table_name);
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // 检查数据库连接
        match sqlx::query("SELECT 1").fetch_one(self.pool.as_ref()).await {
            Ok(_) => Ok(HealthStatus {
                status: "healthy".to_string(),
                message: "PostgreSQL vector store is operational".to_string(),
                timestamp: chrono::Utc::now(),
                details: std::collections::HashMap::new(),
            }),
            Err(e) => Ok(HealthStatus {
                status: "unhealthy".to_string(),
                message: format!("Health check failed: {}", e),
                timestamp: chrono::Utc::now(),
                details: std::collections::HashMap::new(),
            }),
        }
    }

    async fn get_stats(&self) -> Result<VectorStoreStats> {
        let count = self.count_vectors().await?;

        Ok(VectorStoreStats {
            total_vectors: count,
            dimension: self.config.dimension,
            index_size: 0, // TODO: 实现索引大小计算
        })
    }

    async fn add_vectors_batch(&self, batches: Vec<Vec<VectorData>>) -> Result<Vec<Vec<String>>> {
        let mut all_ids = Vec::new();
        for batch in batches {
            let ids = self.add_vectors(batch).await?;
            all_ids.push(ids);
        }
        Ok(all_ids)
    }

    async fn delete_vectors_batch(&self, id_batches: Vec<Vec<String>>) -> Result<Vec<bool>> {
        let mut results = Vec::new();
        for batch in id_batches {
            let result = self.delete_vectors(batch).await.is_ok();
            results.push(result);
        }
        Ok(results)
    }
}
