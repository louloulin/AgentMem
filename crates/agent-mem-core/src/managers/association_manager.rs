//! 记忆关联管理器
//!
//! 提供记忆之间的关联管理，包括：
//! - 关联类型定义（因果、时序、相似）
//! - 关联强度计算
//! - 关联存储和查询
//! - 关联可视化数据

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info};

/// 关联类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssociationType {
    /// 因果关联（A 导致 B）
    Causal,
    /// 时序关联（A 在 B 之前）
    Temporal,
    /// 相似关联（A 与 B 相似）
    Similar,
    /// 对比关联（A 与 B 相反）
    Contrast,
    /// 层级关联（A 是 B 的一部分）
    Hierarchical,
    /// 引用关联（A 引用 B）
    Reference,
    /// 自定义关联
    Custom(String),
}

impl AssociationType {
    pub fn as_str(&self) -> &str {
        match self {
            AssociationType::Causal => "causal",
            AssociationType::Temporal => "temporal",
            AssociationType::Similar => "similar",
            AssociationType::Contrast => "contrast",
            AssociationType::Hierarchical => "hierarchical",
            AssociationType::Reference => "reference",
            AssociationType::Custom(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "causal" => Ok(AssociationType::Causal),
            "temporal" => Ok(AssociationType::Temporal),
            "similar" => Ok(AssociationType::Similar),
            "contrast" => Ok(AssociationType::Contrast),
            "hierarchical" => Ok(AssociationType::Hierarchical),
            "reference" => Ok(AssociationType::Reference),
            _ => Ok(AssociationType::Custom(s.to_string())),
        }
    }
}

/// 记忆关联
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 关联统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationStats {
    pub total_associations: i64,
    pub by_type: Vec<TypeCount>,
    pub avg_strength: f32,
    pub strongest_associations: Vec<MemoryAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCount {
    pub association_type: String,
    pub count: i64,
}

/// 关联管理器配置
#[derive(Debug, Clone)]
pub struct AssociationManagerConfig {
    /// 最小关联强度阈值
    pub min_strength_threshold: f32,
    /// 最大关联数量
    pub max_associations_per_memory: usize,
    /// 是否自动创建相似关联
    pub auto_create_similar: bool,
    /// 相似度阈值
    pub similarity_threshold: f32,
}

impl Default for AssociationManagerConfig {
    fn default() -> Self {
        Self {
            min_strength_threshold: 0.3,
            max_associations_per_memory: 50,
            auto_create_similar: true,
            similarity_threshold: 0.7,
        }
    }
}

/// 记忆关联管理器
pub struct AssociationManager {
    pool: Arc<PgPool>,
    config: AssociationManagerConfig,
}

impl AssociationManager {
    /// 创建新的关联管理器
    pub fn new(pool: Arc<PgPool>, config: AssociationManagerConfig) -> Self {
        Self { pool, config }
    }

    /// 使用默认配置创建
    pub fn with_default_config(pool: Arc<PgPool>) -> Self {
        Self::new(pool, AssociationManagerConfig::default())
    }

    /// 创建关联
    pub async fn create_association(
        &self,
        organization_id: &str,
        user_id: &str,
        agent_id: &str,
        from_memory_id: &str,
        to_memory_id: &str,
        association_type: AssociationType,
        strength: f32,
        confidence: f32,
        metadata: serde_json::Value,
    ) -> Result<String> {
        // 验证强度
        if strength < self.config.min_strength_threshold {
            return Err(AgentMemError::validation_error(format!(
                "Association strength {} below threshold {}",
                strength, self.config.min_strength_threshold
            )));
        }

        let association_id = uuid::Uuid::new_v4().to_string();
        let association_type_str = association_type.as_str();

        sqlx::query!(
            r#"
            INSERT INTO memory_associations (
                id, organization_id, user_id, agent_id,
                from_memory_id, to_memory_id, association_type,
                strength, confidence, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            association_id,
            organization_id,
            user_id,
            agent_id,
            from_memory_id,
            to_memory_id,
            association_type_str,
            strength,
            confidence,
            metadata,
            Utc::now(),
            Utc::now(),
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        info!(
            "Created association {} from {} to {} (type: {}, strength: {})",
            association_id, from_memory_id, to_memory_id, association_type_str, strength
        );

        Ok(association_id)
    }

    /// 获取记忆的所有关联
    pub async fn get_associations(
        &self,
        memory_id: &str,
        user_id: &str,
    ) -> Result<Vec<MemoryAssociation>> {
        let results = sqlx::query_as!(
            MemoryAssociation,
            r#"
            SELECT 
                id, organization_id, user_id, agent_id,
                from_memory_id, to_memory_id, association_type,
                strength, confidence, metadata, created_at, updated_at
            FROM memory_associations
            WHERE (from_memory_id = $1 OR to_memory_id = $1)
            AND user_id = $2
            ORDER BY strength DESC
            "#,
            memory_id,
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results)
    }

    /// 按类型获取关联
    pub async fn get_associations_by_type(
        &self,
        memory_id: &str,
        user_id: &str,
        association_type: AssociationType,
    ) -> Result<Vec<MemoryAssociation>> {
        let type_str = association_type.as_str();

        let results = sqlx::query_as!(
            MemoryAssociation,
            r#"
            SELECT 
                id, organization_id, user_id, agent_id,
                from_memory_id, to_memory_id, association_type,
                strength, confidence, metadata, created_at, updated_at
            FROM memory_associations
            WHERE (from_memory_id = $1 OR to_memory_id = $1)
            AND user_id = $2
            AND association_type = $3
            ORDER BY strength DESC
            "#,
            memory_id,
            user_id,
            type_str,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(results)
    }

    /// 更新关联强度
    pub async fn update_strength(
        &self,
        association_id: &str,
        new_strength: f32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE memory_associations
            SET strength = $1, updated_at = $2
            WHERE id = $3
            "#,
            new_strength,
            Utc::now(),
            association_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        debug!("Updated association {} strength to {}", association_id, new_strength);

        Ok(())
    }

    /// 删除关联
    pub async fn delete_association(&self, association_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM memory_associations
            WHERE id = $1
            "#,
            association_id,
        )
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        info!("Deleted association {}", association_id);

        Ok(())
    }

    /// 获取关联统计
    pub async fn get_stats(&self, user_id: &str) -> Result<AssociationStats> {
        // 总数
        let total = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM memory_associations
            WHERE user_id = $1
            "#,
            user_id,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        // 按类型统计
        let by_type = sqlx::query!(
            r#"
            SELECT association_type, COUNT(*) as count
            FROM memory_associations
            WHERE user_id = $1
            GROUP BY association_type
            "#,
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        // 平均强度
        let avg_strength = sqlx::query!(
            r#"
            SELECT AVG(strength) as avg_strength
            FROM memory_associations
            WHERE user_id = $1
            "#,
            user_id,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        // 最强关联
        let strongest = sqlx::query_as!(
            MemoryAssociation,
            r#"
            SELECT 
                id, organization_id, user_id, agent_id,
                from_memory_id, to_memory_id, association_type,
                strength, confidence, metadata, created_at, updated_at
            FROM memory_associations
            WHERE user_id = $1
            ORDER BY strength DESC
            LIMIT 10
            "#,
            user_id,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::from(e))?;

        Ok(AssociationStats {
            total_associations: total.count.unwrap_or(0),
            by_type: by_type
                .into_iter()
                .map(|r| TypeCount {
                    association_type: r.association_type,
                    count: r.count.unwrap_or(0),
                })
                .collect(),
            avg_strength: avg_strength.avg_strength.unwrap_or(0.0) as f32,
            strongest_associations: strongest,
        })
    }
}

