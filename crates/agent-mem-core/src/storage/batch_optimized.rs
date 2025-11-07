//! Optimized batch operations with true bulk INSERT
//!
//! This module provides optimized batch operations that use SQL's
//! multi-row INSERT statements instead of looping single INSERTs.

use super::models::*;
use super::transaction::{retry_operation, RetryConfig};
use crate::{CoreError, CoreResult};
#[cfg(feature = "postgres")]
use sqlx::PgPool;

/// Optimized batch operations manager
#[cfg(feature = "postgres")]
pub struct OptimizedBatchOperations {
    pool: PgPool,
    retry_config: RetryConfig,
}

#[cfg(feature = "postgres")]
impl OptimizedBatchOperations {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(pool: PgPool, retry_config: RetryConfig) -> Self {
        Self { pool, retry_config }
    }

    /// Batch insert memories using multi-row INSERT
    ///
    /// This is significantly faster than single-row inserts as it:
    /// - Uses a single SQL statement
    /// - Reduces network roundtrips
    /// - Allows database-level optimizations
    ///
    /// Performance: ~2-3x faster than looping inserts
    pub async fn batch_insert_memories_optimized(&self, memories: &[Memory]) -> CoreResult<u64> {
        if memories.is_empty() {
            return Ok(0);
        }

        // Split into reasonable chunks (PostgreSQL has a parameter limit)
        const CHUNK_SIZE: usize = 1000;
        let mut total_inserted = 0;

        for chunk in memories.chunks(CHUNK_SIZE) {
            let inserted = self.insert_memory_chunk(chunk).await?;
            total_inserted += inserted;
        }

        Ok(total_inserted)
    }

    /// Insert a chunk of memories using a single multi-row INSERT
    async fn insert_memory_chunk(&self, chunk: &[Memory]) -> CoreResult<u64> {
        let pool = self.pool.clone();
        let chunk = chunk.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let chunk = chunk.clone();
            async move {
                // Build VALUES clause dynamically
                let mut query = String::from(
                    r#"
                    INSERT INTO memories (
                        id, organization_id, user_id, agent_id, content,
                        hash, metadata, score, memory_type, scope, level,
                        importance, access_count, last_accessed,
                        created_at, updated_at, is_deleted,
                        created_by_id, last_updated_by_id
                    ) VALUES
                    "#,
                );

                // Add value placeholders
                let mut values = Vec::new();
                for (i, _) in chunk.iter().enumerate() {
                    let base = i * 19;
                    values.push(format!(
                        "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
                        base + 1, base + 2, base + 3, base + 4, base + 5,
                        base + 6, base + 7, base + 8, base + 9, base + 10,
                        base + 11, base + 12, base + 13, base + 14, base + 15,
                        base + 16, base + 17, base + 18, base + 19
                    ));
                }

                query.push_str(&values.join(", "));
                query.push_str(" ON CONFLICT (id) DO NOTHING");

                // Bind all parameters
                let mut sql_query = sqlx::query(&query);
                for memory in &chunk {
                    sql_query = sql_query
                        .bind(&memory.id)
                        .bind(&memory.organization_id)
                        .bind(&memory.user_id)
                        .bind(&memory.agent_id)
                        .bind(&memory.content)
                        .bind(&memory.hash)
                        .bind(&memory.metadata)
                        .bind(&memory.score)
                        .bind(&memory.memory_type)
                        .bind(&memory.scope)
                        .bind(&memory.level)
                        .bind(&memory.importance)
                        .bind(&memory.access_count)
                        .bind(&memory.last_accessed)
                        .bind(&memory.created_at)
                        .bind(&memory.updated_at)
                        .bind(&memory.is_deleted)
                        .bind(&memory.created_by_id)
                        .bind(&memory.last_updated_by_id);
                }

                let result = sql_query
                    .execute(&pool)
                    .await
                    .map_err(|e| CoreError::Database(format!("Failed to batch insert memories: {}", e)))?;

                Ok(result.rows_affected())
            }
        })
        .await
    }

    /// Batch insert messages using multi-row INSERT
    pub async fn batch_insert_messages_optimized(&self, messages: &[Message]) -> CoreResult<u64> {
        if messages.is_empty() {
            return Ok(0);
        }

        const CHUNK_SIZE: usize = 1000;
        let mut total_inserted = 0;

        for chunk in messages.chunks(CHUNK_SIZE) {
            let inserted = self.insert_message_chunk(chunk).await?;
            total_inserted += inserted;
        }

        Ok(total_inserted)
    }

    async fn insert_message_chunk(&self, chunk: &[Message]) -> CoreResult<u64> {
        let pool = self.pool.clone();
        let chunk = chunk.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let chunk = chunk.clone();
            async move {
                let mut query = String::from(
                    r#"
                    INSERT INTO messages (
                        id, organization_id, user_id, agent_id, role,
                        text, content, model, name, tool_calls,
                        tool_call_id, step_id, otid, tool_returns,
                        group_id, sender_id,
                        created_at, updated_at, is_deleted,
                        created_by_id, last_updated_by_id
                    ) VALUES
                    "#,
                );

                let mut values = Vec::new();
                for (i, _) in chunk.iter().enumerate() {
                    let base = i * 21;
                    let placeholders: Vec<String> =
                        (1..=21).map(|j| format!("${}", base + j)).collect();
                    values.push(format!("({})", placeholders.join(", ")));
                }

                query.push_str(&values.join(", "));
                query.push_str(" ON CONFLICT (id) DO NOTHING");

                let mut sql_query = sqlx::query(&query);
                for message in &chunk {
                    sql_query = sql_query
                        .bind(&message.id)
                        .bind(&message.organization_id)
                        .bind(&message.user_id)
                        .bind(&message.agent_id)
                        .bind(&message.role)
                        .bind(&message.text)
                        .bind(&message.content)
                        .bind(&message.model)
                        .bind(&message.name)
                        .bind(&message.tool_calls)
                        .bind(&message.tool_call_id)
                        .bind(&message.step_id)
                        .bind(&message.otid)
                        .bind(&message.tool_returns)
                        .bind(&message.group_id)
                        .bind(&message.sender_id)
                        .bind(&message.created_at)
                        .bind(&message.updated_at)
                        .bind(&message.is_deleted)
                        .bind(&message.created_by_id)
                        .bind(&message.last_updated_by_id);
                }

                let result = sql_query.execute(&pool).await.map_err(|e| {
                    CoreError::Database(format!("Failed to batch insert messages: {}", e))
                })?;

                Ok(result.rows_affected())
            }
        })
        .await
    }

    /// Generic batch insert helper
    ///
    /// This is a template for implementing optimized batch inserts for other tables
    pub async fn batch_insert_generic<T, F>(
        &self,
        items: &[T],
        table_name: &str,
        columns: &[&str],
        bind_fn: F,
    ) -> CoreResult<u64>
    where
        T: Clone + Send + Sync,
        F: Fn(
                &mut sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
                &T,
            ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>
            + Send
            + Sync,
    {
        if items.is_empty() {
            return Ok(0);
        }

        const CHUNK_SIZE: usize = 1000;
        let mut total_inserted = 0;

        for chunk in items.chunks(CHUNK_SIZE) {
            let inserted = self
                .insert_generic_chunk(chunk, table_name, columns, &bind_fn)
                .await?;
            total_inserted += inserted;
        }

        Ok(total_inserted)
    }

    async fn insert_generic_chunk<T, F>(
        &self,
        chunk: &[T],
        table_name: &str,
        columns: &[&str],
        bind_fn: &F,
    ) -> CoreResult<u64>
    where
        T: Clone,
        F: Fn(
            &mut sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
            &T,
        ) -> sqlx::query::Query<'_, sqlx::Postgres, sqlx::postgres::PgArguments>,
    {
        let column_list = columns.join(", ");
        let num_columns = columns.len();

        let mut query = format!("INSERT INTO {} ({}) VALUES ", table_name, column_list);

        let mut values = Vec::new();
        for (i, _) in chunk.iter().enumerate() {
            let base = i * num_columns;
            let placeholders: Vec<String> = (1..=num_columns)
                .map(|j| format!("${}", base + j))
                .collect();
            values.push(format!("({})", placeholders.join(", ")));
        }

        query.push_str(&values.join(", "));
        query.push_str(" ON CONFLICT DO NOTHING");

        let mut sql_query = sqlx::query(&query);
        for item in chunk {
            sql_query = bind_fn(&mut sql_query, item);
        }

        let result = sql_query
            .execute(&self.pool)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to batch insert: {}", e)))?;

        Ok(result.rows_affected())
    }
}

/// Performance comparison data
pub struct BatchPerformanceMetrics {
    pub operation: String,
    pub items_count: usize,
    pub old_method_ms: u64,
    pub optimized_method_ms: u64,
    pub speedup: f64,
}

impl BatchPerformanceMetrics {
    pub fn new(operation: String, items: usize, old_ms: u64, new_ms: u64) -> Self {
        let speedup = if new_ms > 0 {
            old_ms as f64 / new_ms as f64
        } else {
            0.0
        };

        Self {
            operation,
            items_count: items,
            old_method_ms: old_ms,
            optimized_method_ms: new_ms,
            speedup,
        }
    }

    pub fn format_report(&self) -> String {
        format!(
            "Operation: {}\n\
             Items: {}\n\
             Old method: {}ms\n\
             Optimized: {}ms\n\
             Speedup: {:.2}x\n\
             Improvement: {:.1}%",
            self.operation,
            self.items_count,
            self.old_method_ms,
            self.optimized_method_ms,
            self.speedup,
            (self.speedup - 1.0) * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metrics =
            BatchPerformanceMetrics::new("batch_insert_memories".to_string(), 1000, 5000, 2000);

        assert_eq!(metrics.items_count, 1000);
        assert_eq!(metrics.old_method_ms, 5000);
        assert_eq!(metrics.optimized_method_ms, 2000);
        assert_eq!(metrics.speedup, 2.5);
    }
}
