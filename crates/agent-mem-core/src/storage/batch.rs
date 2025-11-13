//! Batch operations for efficient bulk database operations

use sqlx::PgPool;

use super::models::*;
use super::transaction::{retry_operation, RetryConfig};
use crate::{CoreError, CoreResult};

/// Batch operations manager
pub struct BatchOperations {
    pool: PgPool,
    retry_config: RetryConfig,
}

impl BatchOperations {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            retry_config: RetryConfig::default(),
        }
    }

    pub fn with_retry_config(pool: PgPool, retry_config: RetryConfig) -> Self {
        Self { pool, retry_config }
    }

    /// Batch insert agents
    pub async fn batch_insert_agents(&self, agents: &[Agent]) -> CoreResult<u64> {
        if agents.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let agents = agents.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let agents = agents.clone();
            async move {
                let mut inserted = 0u64;

                for agent in agents {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO agents (
                            id, organization_id, agent_type, name, description,
                            system, topic, message_ids, metadata_, llm_config,
                            embedding_config, tool_rules, mcp_tools,
                            created_at, updated_at, is_deleted,
                            created_by_id, last_updated_by_id
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                        ON CONFLICT (id) DO NOTHING
                        "#,
                    )
                    .bind(&agent.id)
                    .bind(&agent.organization_id)
                    .bind(&agent.agent_type)
                    .bind(&agent.name)
                    .bind(&agent.description)
                    .bind(&agent.system)
                    .bind(&agent.topic)
                    .bind(&agent.message_ids)
                    .bind(&agent.metadata_)
                    .bind(&agent.llm_config)
                    .bind(&agent.embedding_config)
                    .bind(&agent.tool_rules)
                    .bind(&agent.mcp_tools)
                    .bind(&agent.created_at)
                    .bind(&agent.updated_at)
                    .bind(&agent.is_deleted)
                    .bind(&agent.created_by_id)
                    .bind(&agent.last_updated_by_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| CoreError::Database(format!("Failed to batch insert agent: {}", e)))?;

                    inserted += result.rows_affected();
                }

                Ok(inserted)
            }
        })
        .await
    }

    /// Batch insert messages
    pub async fn batch_insert_messages(&self, messages: &[Message]) -> CoreResult<u64> {
        if messages.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let messages = messages.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let messages = messages.clone();
            async move {
                let mut inserted = 0u64;

                for message in messages {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO messages (
                            id, organization_id, user_id, agent_id, role,
                            text, content, model, name, tool_calls,
                            tool_call_id, step_id, otid, tool_returns,
                            group_id, sender_id,
                            created_at, updated_at, is_deleted,
                            created_by_id, last_updated_by_id
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                        ON CONFLICT (id) DO NOTHING
                        "#,
                    )
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
                    .bind(&message.last_updated_by_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| CoreError::Database(format!("Failed to batch insert message: {}", e)))?;

                    inserted += result.rows_affected();
                }

                Ok(inserted)
            }
        })
        .await
    }

    /// Batch insert memories
    pub async fn batch_insert_memories(&self, memories: &[DbMemory]) -> CoreResult<u64> {
        if memories.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let memories = memories.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let memories = memories.clone();
            async move {
                let mut inserted = 0u64;

                for memory in memories {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO memories (
                            id, organization_id, user_id, agent_id, content,
                            hash, metadata, score, memory_type, scope, level,
                            importance, access_count, last_accessed,
                            created_at, updated_at, is_deleted,
                            created_by_id, last_updated_by_id
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                        ON CONFLICT (id) DO NOTHING
                        "#,
                    )
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
                    .bind(&memory.last_updated_by_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| CoreError::Database(format!("Failed to batch insert memory: {}", e)))?;

                    inserted += result.rows_affected();
                }

                Ok(inserted)
            }
        })
        .await
    }

    /// Batch insert blocks
    pub async fn batch_insert_blocks(&self, blocks: &[Block]) -> CoreResult<u64> {
        if blocks.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let blocks = blocks.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let blocks = blocks.clone();
            async move {
                let mut inserted = 0u64;

                for block in blocks {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO blocks (
                            id, organization_id, user_id, template_name,
                            description, label, is_template, value, "limit",
                            metadata_, created_at, updated_at, is_deleted,
                            created_by_id, last_updated_by_id
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                        ON CONFLICT (id, label) DO NOTHING
                        "#,
                    )
                    .bind(&block.id)
                    .bind(&block.organization_id)
                    .bind(&block.user_id)
                    .bind(&block.template_name)
                    .bind(&block.description)
                    .bind(&block.label)
                    .bind(&block.is_template)
                    .bind(&block.value)
                    .bind(&block.limit)
                    .bind(&block.metadata_)
                    .bind(&block.created_at)
                    .bind(&block.updated_at)
                    .bind(&block.is_deleted)
                    .bind(&block.created_by_id)
                    .bind(&block.last_updated_by_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| {
                        CoreError::Database(format!("Failed to batch insert block: {}", e))
                    })?;

                    inserted += result.rows_affected();
                }

                Ok(inserted)
            }
        })
        .await
    }

    /// Batch insert tools
    pub async fn batch_insert_tools(&self, tools: &[Tool]) -> CoreResult<u64> {
        if tools.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let tools = tools.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let tools = tools.clone();
            async move {
                let mut inserted = 0u64;

                for tool in tools {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO tools (
                            id, organization_id, name, description,
                            json_schema, source_type, source_code, tags,
                            metadata_, created_at, updated_at, is_deleted,
                            created_by_id, last_updated_by_id
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                        ON CONFLICT (id) DO NOTHING
                        "#,
                    )
                    .bind(&tool.id)
                    .bind(&tool.organization_id)
                    .bind(&tool.name)
                    .bind(&tool.description)
                    .bind(&tool.json_schema)
                    .bind(&tool.source_type)
                    .bind(&tool.source_code)
                    .bind(&tool.tags)
                    .bind(&tool.metadata_)
                    .bind(&tool.created_at)
                    .bind(&tool.updated_at)
                    .bind(&tool.is_deleted)
                    .bind(&tool.created_by_id)
                    .bind(&tool.last_updated_by_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| {
                        CoreError::Database(format!("Failed to batch insert tool: {}", e))
                    })?;

                    inserted += result.rows_affected();
                }

                Ok(inserted)
            }
        })
        .await
    }

    /// Batch delete by IDs (soft delete)
    pub async fn batch_soft_delete(&self, table: &str, ids: &[String]) -> CoreResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let pool = self.pool.clone();
        let table = table.to_string();
        let ids = ids.to_vec();

        retry_operation(self.retry_config.clone(), || {
            let pool = pool.clone();
            let table = table.clone();
            let ids = ids.clone();
            async move {
                let query = format!(
                    "UPDATE {} SET is_deleted = TRUE, updated_at = $1 WHERE id = ANY($2) AND is_deleted = FALSE",
                    table
                );

                let result = sqlx::query(&query)
                    .bind(chrono::Utc::now())
                    .bind(&ids)
                    .execute(&pool)
                    .await
                    .map_err(|e| CoreError::Database(format!("Failed to batch soft delete: {}", e)))?;

                Ok(result.rows_affected())
            }
        })
        .await
    }
}
