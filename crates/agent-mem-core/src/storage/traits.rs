//! Repository trait definitions
//!
//! 定义所有 repository 的 trait 接口，支持多种后端实现（PostgreSQL, LibSQL 等）

use crate::storage::models::*;
use agent_mem_traits::{MemoryV4 as Memory, Result};
use async_trait::async_trait;

/// User repository trait
#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    /// Create a new user
    async fn create(&self, user: &User) -> Result<User>;

    /// Find user by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<User>>;

    /// Find user by email and organization
    async fn find_by_email(&self, email: &str, org_id: &str) -> Result<Option<User>>;

    /// Check if email exists in organization
    async fn email_exists(&self, email: &str, org_id: &str) -> Result<bool>;

    /// Find user by organization ID
    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<User>>;

    /// Update user
    async fn update(&self, user: &User) -> Result<User>;

    /// Update user password
    async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<()>;

    /// Delete user (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// List all users
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<User>>;
}

/// Agent repository trait
#[async_trait]
pub trait AgentRepositoryTrait: Send + Sync {
    /// Create a new agent
    async fn create(&self, agent: &Agent) -> Result<Agent>;

    /// Find agent by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Agent>>;

    /// Find agents by organization ID
    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<Agent>>;

    /// Update agent
    async fn update(&self, agent: &Agent) -> Result<Agent>;

    /// Delete agent (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// List all agents
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Agent>>;
}

/// Message repository trait
#[async_trait]
pub trait MessageRepositoryTrait: Send + Sync {
    /// Create a new message
    async fn create(&self, message: &Message) -> Result<Message>;

    /// Find message by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Message>>;

    /// Find messages by agent ID
    async fn find_by_agent_id(&self, agent_id: &str, limit: i64) -> Result<Vec<Message>>;

    /// Find messages by user ID
    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Message>>;

    /// Update message
    async fn update(&self, message: &Message) -> Result<Message>;

    /// Delete message (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// Delete messages by agent ID
    async fn delete_by_agent_id(&self, agent_id: &str) -> Result<u64>;
}

/// Tool repository trait
#[async_trait]
pub trait ToolRepositoryTrait: Send + Sync {
    /// Create a new tool
    async fn create(&self, tool: &Tool) -> Result<Tool>;

    /// Find tool by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Tool>>;

    /// Find tools by organization ID
    async fn find_by_organization_id(&self, org_id: &str) -> Result<Vec<Tool>>;

    /// Find tools by tags
    async fn find_by_tags(&self, org_id: &str, tags: &[String]) -> Result<Vec<Tool>>;

    /// Update tool
    async fn update(&self, tool: &Tool) -> Result<Tool>;

    /// Delete tool (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// List all tools
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Tool>>;
}

/// Organization repository trait
#[async_trait]
pub trait OrganizationRepositoryTrait: Send + Sync {
    /// Create a new organization
    async fn create(&self, org: &Organization) -> Result<Organization>;

    /// Find organization by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Organization>>;

    /// Find organization by name
    async fn find_by_name(&self, name: &str) -> Result<Option<Organization>>;

    /// Update organization
    async fn update(&self, org: &Organization) -> Result<Organization>;

    /// Delete organization (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// List all organizations
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Organization>>;
}

/// API Key repository trait
#[async_trait]
pub trait ApiKeyRepositoryTrait: Send + Sync {
    /// Create a new API key
    async fn create(&self, api_key: &ApiKey) -> Result<ApiKey>;

    /// Find API key by key string
    async fn find_by_key(&self, key: &str) -> Result<Option<ApiKey>>;

    /// Find API keys by user ID
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<ApiKey>>;

    /// Update API key
    async fn update(&self, api_key: &ApiKey) -> Result<ApiKey>;

    /// Delete API key
    async fn delete(&self, id: &str) -> Result<()>;

    /// Revoke API key (soft delete)
    async fn revoke(&self, id: &str) -> Result<()>;

    /// List all API keys
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ApiKey>>;
}

/// Memory repository trait
#[async_trait]
pub trait MemoryRepositoryTrait: Send + Sync {
    /// Create a new memory
    async fn create(&self, memory: &Memory) -> Result<Memory>;

    /// Find memory by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Memory>>;

    /// 🆕 Phase 1.6: 批量查询优化 - 批量查找多个ID的记忆
    /// 预期效果: 批量查询性能提升10x (N次查询 → 1次查询)
    /// 默认实现使用循环调用find_by_id（fallback），具体实现可以覆盖此方法
    async fn batch_find_by_ids(&self, ids: &[String]) -> Result<Vec<Memory>> {
        // 默认实现：循环调用find_by_id（N+1问题，但作为fallback）
        let mut results = Vec::new();
        for id in ids {
            if let Ok(Some(memory)) = self.find_by_id(id).await {
                results.push(memory);
            }
        }
        Ok(results)
    }

    /// Find memories by agent ID
    async fn find_by_agent_id(&self, agent_id: &str, limit: i64) -> Result<Vec<Memory>>;

    /// Find memories by user ID
    async fn find_by_user_id(&self, user_id: &str, limit: i64) -> Result<Vec<Memory>>;

    /// Search memories by content
    async fn search(&self, query: &str, limit: i64) -> Result<Vec<Memory>>;

    /// Update memory
    async fn update(&self, memory: &Memory) -> Result<Memory>;

    /// Delete memory (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// Delete memories by agent ID
    async fn delete_by_agent_id(&self, agent_id: &str) -> Result<u64>;

    /// List all memories
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Memory>>;
}

/// Block repository trait (Core Memory)
#[async_trait]
pub trait BlockRepositoryTrait: Send + Sync {
    /// Create a new block
    async fn create(&self, block: &Block) -> Result<Block>;

    /// Find block by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<Block>>;

    /// Find blocks by agent ID
    async fn find_by_agent_id(&self, agent_id: &str) -> Result<Vec<Block>>;

    /// Update block
    async fn update(&self, block: &Block) -> Result<Block>;

    /// Delete block (soft delete)
    async fn delete(&self, id: &str) -> Result<()>;

    /// Link block to agent
    async fn link_to_agent(&self, block_id: &str, agent_id: &str) -> Result<()>;

    /// Unlink block from agent
    async fn unlink_from_agent(&self, block_id: &str, agent_id: &str) -> Result<()>;

    /// List all blocks
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Block>>;
}

/// Memory Association model
#[derive(Debug, Clone)]
pub struct MemoryAssociation {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub agent_id: String,
    pub from_memory_id: String,
    pub to_memory_id: String,
    pub association_type: String,
    pub strength: f32,
    pub confidence: f32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Association repository trait
#[async_trait]
pub trait AssociationRepositoryTrait: Send + Sync {
    /// Create a new association
    async fn create(&self, association: &MemoryAssociation) -> Result<MemoryAssociation>;

    /// Find association by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<MemoryAssociation>>;

    /// Get all associations for a memory
    async fn find_by_memory_id(
        &self,
        memory_id: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryAssociation>>;

    /// Get associations by type
    async fn find_by_type(
        &self,
        memory_id: &str,
        user_id: &str,
        association_type: &str,
    ) -> Result<Vec<MemoryAssociation>>;

    /// Update association strength
    async fn update_strength(&self, id: &str, strength: f32) -> Result<()>;

    /// Delete association
    async fn delete(&self, id: &str) -> Result<()>;

    /// Get association count
    async fn count_by_user(&self, user_id: &str) -> Result<i64>;

    /// Get association count by type
    async fn count_by_type(&self, user_id: &str) -> Result<Vec<(String, i64)>>;

    /// Get average strength
    async fn avg_strength(&self, user_id: &str) -> Result<f32>;

    /// Get strongest associations
    async fn find_strongest(&self, user_id: &str, limit: i64) -> Result<Vec<MemoryAssociation>>;
}
