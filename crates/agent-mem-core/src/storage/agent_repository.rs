//! Agent repository implementation

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};

use super::models::{Agent, Block};
use super::repository::Repository;
use crate::{CoreError, CoreResult};

/// Agent repository
pub struct AgentRepository {
    pool: PgPool,
}

impl AgentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List agents by organization
    pub async fn list_by_organization(
        &self,
        organization_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> CoreResult<Vec<Agent>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Agent>(
            r#"
            SELECT * FROM agents
            WHERE organization_id = $1 AND is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(organization_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list agents: {}", e)))?;

        Ok(results)
    }

    /// Get agent with its core memory blocks
    pub async fn get_with_blocks(&self, agent_id: &str) -> CoreResult<Option<(Agent, Vec<Block>)>> {
        // Get agent
        let agent = self.read(agent_id).await?;
        let agent = match agent {
            Some(agent) => agent,
            None => return Ok(None),
        };

        // Get associated blocks
        let blocks = sqlx::query_as::<_, Block>(
            r#"
            SELECT b.* FROM blocks b
            INNER JOIN blocks_agents ba ON b.id = ba.block_id AND b.label = ba.block_label
            WHERE ba.agent_id = $1 AND b.is_deleted = FALSE
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to get agent blocks: {}", e)))?;

        Ok(Some((agent, blocks)))
    }

    /// Add a block to an agent
    pub async fn add_block(
        &self,
        agent_id: &str,
        block_id: &str,
        block_label: &str,
    ) -> CoreResult<()> {
        sqlx::query(
            r#"
            INSERT INTO blocks_agents (block_id, block_label, agent_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(block_id)
        .bind(block_label)
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to add block to agent: {}", e)))?;

        Ok(())
    }

    /// Remove a block from an agent
    pub async fn remove_block(
        &self,
        agent_id: &str,
        block_id: &str,
        block_label: &str,
    ) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM blocks_agents
            WHERE agent_id = $1 AND block_id = $2 AND block_label = $3
            "#,
        )
        .bind(agent_id)
        .bind(block_id)
        .bind(block_label)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to remove block from agent: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}

#[async_trait]
impl Repository<Agent> for AgentRepository {
    async fn create(&self, agent: &Agent) -> CoreResult<Agent> {
        let result = sqlx::query_as::<_, Agent>(
            r#"
            INSERT INTO agents (
                id, organization_id, agent_type, name, description,
                system, topic, message_ids, metadata_, llm_config,
                embedding_config, tool_rules, mcp_tools,
                created_at, updated_at, is_deleted,
                created_by_id, last_updated_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
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
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to create agent: {}", e)))?;

        Ok(result)
    }

    async fn read(&self, id: &str) -> CoreResult<Option<Agent>> {
        let result = sqlx::query_as::<_, Agent>(
            r#"
            SELECT * FROM agents
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to read agent: {}", e)))?;

        Ok(result)
    }

    async fn update(&self, agent: &Agent) -> CoreResult<Agent> {
        let result = sqlx::query_as::<_, Agent>(
            r#"
            UPDATE agents
            SET agent_type = $2, name = $3, description = $4,
                system = $5, topic = $6, message_ids = $7,
                metadata_ = $8, llm_config = $9, embedding_config = $10,
                tool_rules = $11, mcp_tools = $12,
                updated_at = $13, last_updated_by_id = $14
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING *
            "#,
        )
        .bind(&agent.id)
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
        .bind(Utc::now())
        .bind(&agent.last_updated_by_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to update agent: {}", e)))?;

        Ok(result)
    }

    async fn delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            UPDATE agents
            SET is_deleted = TRUE, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to delete agent: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn hard_delete(&self, id: &str) -> CoreResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM agents WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to hard delete agent: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> CoreResult<Vec<Agent>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let results = sqlx::query_as::<_, Agent>(
            r#"
            SELECT * FROM agents
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to list agents: {}", e)))?;

        Ok(results)
    }

    async fn count(&self) -> CoreResult<i64> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count FROM agents
            WHERE is_deleted = FALSE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CoreError::Database(format!("Failed to count agents: {}", e)))?;

        Ok(result.try_get("count").unwrap_or(0))
    }
}
