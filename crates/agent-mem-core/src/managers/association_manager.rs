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
use std::sync::Arc;
use tracing::{debug, info};

use crate::storage::traits::{
    AssociationRepositoryTrait, MemoryAssociation as RepoMemoryAssociation,
};

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
    repository: Arc<dyn AssociationRepositoryTrait>,
    config: AssociationManagerConfig,
}

impl AssociationManager {
    /// 创建新的关联管理器
    pub fn new(
        repository: Arc<dyn AssociationRepositoryTrait>,
        config: AssociationManagerConfig,
    ) -> Self {
        Self { repository, config }
    }

    /// 使用默认配置创建
    pub fn with_default_config(repository: Arc<dyn AssociationRepositoryTrait>) -> Self {
        Self::new(repository, AssociationManagerConfig::default())
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

        let association_id = format!("assoc-{}", uuid::Uuid::new_v4());
        let association_type_str = association_type.as_str().to_string();
        let now = Utc::now();

        let association = RepoMemoryAssociation {
            id: association_id.clone(),
            organization_id: organization_id.to_string(),
            user_id: user_id.to_string(),
            agent_id: agent_id.to_string(),
            from_memory_id: from_memory_id.to_string(),
            to_memory_id: to_memory_id.to_string(),
            association_type: association_type_str.clone(),
            strength,
            confidence,
            metadata,
            created_at: now,
            updated_at: now,
        };

        self.repository.create(&association).await?;

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
        let repo_associations = self
            .repository
            .find_by_memory_id(memory_id, user_id)
            .await?;

        let results = repo_associations
            .into_iter()
            .map(|a| MemoryAssociation {
                id: a.id,
                organization_id: a.organization_id,
                user_id: a.user_id,
                agent_id: a.agent_id,
                from_memory_id: a.from_memory_id,
                to_memory_id: a.to_memory_id,
                association_type: a.association_type,
                strength: a.strength,
                confidence: a.confidence,
                metadata: a.metadata,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
            .collect();

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
        let repo_associations = self
            .repository
            .find_by_type(memory_id, user_id, type_str)
            .await?;

        let results = repo_associations
            .into_iter()
            .map(|a| MemoryAssociation {
                id: a.id,
                organization_id: a.organization_id,
                user_id: a.user_id,
                agent_id: a.agent_id,
                from_memory_id: a.from_memory_id,
                to_memory_id: a.to_memory_id,
                association_type: a.association_type,
                strength: a.strength,
                confidence: a.confidence,
                metadata: a.metadata,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
            .collect();

        Ok(results)
    }

    /// 更新关联强度
    pub async fn update_strength(&self, association_id: &str, new_strength: f32) -> Result<()> {
        self.repository
            .update_strength(association_id, new_strength)
            .await?;
        debug!(
            "Updated association {} strength to {}",
            association_id, new_strength
        );
        Ok(())
    }

    /// 删除关联
    pub async fn delete_association(&self, association_id: &str) -> Result<()> {
        self.repository.delete(association_id).await?;
        info!("Deleted association {}", association_id);
        Ok(())
    }

    /// 获取关联统计
    pub async fn get_stats(&self, user_id: &str) -> Result<AssociationStats> {
        // 总数
        let total = self.repository.count_by_user(user_id).await?;

        // 按类型统计
        let by_type_data = self.repository.count_by_type(user_id).await?;
        let by_type = by_type_data
            .into_iter()
            .map(|(association_type, count)| TypeCount {
                association_type,
                count,
            })
            .collect();

        // 平均强度
        let avg_strength = self.repository.avg_strength(user_id).await?;

        // 最强关联
        let repo_strongest = self.repository.find_strongest(user_id, 10).await?;
        let strongest_associations = repo_strongest
            .into_iter()
            .map(|a| MemoryAssociation {
                id: a.id,
                organization_id: a.organization_id,
                user_id: a.user_id,
                agent_id: a.agent_id,
                from_memory_id: a.from_memory_id,
                to_memory_id: a.to_memory_id,
                association_type: a.association_type,
                strength: a.strength,
                confidence: a.confidence,
                metadata: a.metadata,
                created_at: a.created_at,
                updated_at: a.updated_at,
            })
            .collect();

        Ok(AssociationStats {
            total_associations: total,
            by_type,
            avg_strength,
            strongest_associations,
        })
    }
}
