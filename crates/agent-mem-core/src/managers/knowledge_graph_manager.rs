//! 知识图谱管理器
//!
//! 提供知识图谱构建和查询功能，包括：
//! - 实体提取
//! - 关系提取
//! - 图谱构建
//! - 图谱查询

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{debug, info};

/// 实体类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Event,
    Concept,
    Object,
    Custom(String),
}

impl EntityType {
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::Person => "person",
            EntityType::Organization => "organization",
            EntityType::Location => "location",
            EntityType::Event => "event",
            EntityType::Concept => "concept",
            EntityType::Object => "object",
            EntityType::Custom(s) => s,
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "person" => Ok(EntityType::Person),
            "organization" => Ok(EntityType::Organization),
            "location" => Ok(EntityType::Location),
            "event" => Ok(EntityType::Event),
            "concept" => Ok(EntityType::Concept),
            "object" => Ok(EntityType::Object),
            _ => Ok(EntityType::Custom(s.to_string())),
        }
    }
}

/// 关系类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    IsA,
    PartOf,
    RelatedTo,
    CausedBy,
    Leads,
    SimilarTo,
    OppositeOf,
    LocatedIn,
    WorksFor,
    Owns,
    Custom(String),
}

impl RelationType {
    pub fn as_str(&self) -> &str {
        match self {
            RelationType::IsA => "is_a",
            RelationType::PartOf => "part_of",
            RelationType::RelatedTo => "related_to",
            RelationType::CausedBy => "caused_by",
            RelationType::Leads => "leads",
            RelationType::SimilarTo => "similar_to",
            RelationType::OppositeOf => "opposite_of",
            RelationType::LocatedIn => "located_in",
            RelationType::WorksFor => "works_for",
            RelationType::Owns => "owns",
            RelationType::Custom(s) => s,
        }
    }
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub properties: serde_json::Value,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub from_entity_id: String,
    pub to_entity_id: String,
    pub relation_type: String,
    pub properties: serde_json::Value,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

/// 图谱查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub paths: Vec<GraphPath>,
}

/// 图谱路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub entities: Vec<String>,
    pub relations: Vec<String>,
    pub length: usize,
}

/// 知识图谱管理器配置
#[derive(Debug, Clone)]
pub struct KnowledgeGraphConfig {
    /// 最小置信度阈值
    pub min_confidence: f32,
    /// 最大路径长度
    pub max_path_length: usize,
    /// 是否启用自动提取
    pub auto_extract: bool,
}

impl Default for KnowledgeGraphConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_path_length: 5,
            auto_extract: true,
        }
    }
}

/// 知识图谱管理器
pub struct KnowledgeGraphManager {
    pool: Arc<PgPool>,
    config: KnowledgeGraphConfig,
}

impl KnowledgeGraphManager {
    /// 创建新的知识图谱管理器
    pub fn new(pool: Arc<PgPool>, config: KnowledgeGraphConfig) -> Self {
        Self { pool, config }
    }

    /// 使用默认配置创建
    pub fn with_default_config(pool: Arc<PgPool>) -> Self {
        Self::new(pool, KnowledgeGraphConfig::default())
    }

    /// 创建实体
    pub async fn create_entity(
        &self,
        organization_id: &str,
        user_id: &str,
        name: &str,
        entity_type: EntityType,
        properties: serde_json::Value,
        confidence: f32,
    ) -> Result<String> {
        let entity_id = uuid::Uuid::new_v4().to_string();
        let entity_type_str = entity_type.as_str();

        sqlx::query(
            r#"
            INSERT INTO knowledge_entities (
                id, organization_id, user_id, name, entity_type,
                properties, confidence, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&entity_id)
        .bind(organization_id)
        .bind(user_id)
        .bind(name)
        .bind(entity_type_str)
        .bind(&properties)
        .bind(confidence)
        .bind(Utc::now())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create entity: {}", e)))?;

        info!("Created entity {} (type: {}, name: {})", entity_id, entity_type_str, name);

        Ok(entity_id)
    }

    /// 创建关系
    pub async fn create_relation(
        &self,
        organization_id: &str,
        user_id: &str,
        from_entity_id: &str,
        to_entity_id: &str,
        relation_type: RelationType,
        properties: serde_json::Value,
        confidence: f32,
    ) -> Result<String> {
        let relation_id = uuid::Uuid::new_v4().to_string();
        let relation_type_str = relation_type.as_str();

        sqlx::query(
            r#"
            INSERT INTO knowledge_relations (
                id, organization_id, user_id, from_entity_id, to_entity_id,
                relation_type, properties, confidence, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&relation_id)
        .bind(organization_id)
        .bind(user_id)
        .bind(from_entity_id)
        .bind(to_entity_id)
        .bind(relation_type_str)
        .bind(&properties)
        .bind(confidence)
        .bind(Utc::now())
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to create relation: {}", e)))?;

        info!(
            "Created relation {} from {} to {} (type: {})",
            relation_id, from_entity_id, to_entity_id, relation_type_str
        );

        Ok(relation_id)
    }

    /// 获取实体
    pub async fn get_entity(&self, entity_id: &str, user_id: &str) -> Result<Option<Entity>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, entity_type, properties, confidence, created_at
            FROM knowledge_entities
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(entity_id)
        .bind(user_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get entity: {}", e)))?;

        Ok(row.map(|r| Entity {
            id: r.get("id"),
            name: r.get("name"),
            entity_type: r.get("entity_type"),
            properties: r.get("properties"),
            confidence: r.get("confidence"),
            created_at: r.get("created_at"),
        }))
    }

    /// 按类型查询实体
    pub async fn get_entities_by_type(
        &self,
        user_id: &str,
        entity_type: EntityType,
    ) -> Result<Vec<Entity>> {
        let type_str = entity_type.as_str();

        let rows = sqlx::query(
            r#"
            SELECT id, name, entity_type, properties, confidence, created_at
            FROM knowledge_entities
            WHERE user_id = $1 AND entity_type = $2
            ORDER BY confidence DESC
            "#,
        )
        .bind(user_id)
        .bind(type_str)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get entities by type: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| Entity {
                id: r.get("id"),
                name: r.get("name"),
                entity_type: r.get("entity_type"),
                properties: r.get("properties"),
                confidence: r.get("confidence"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// 获取实体的所有关系
    pub async fn get_entity_relations(
        &self,
        entity_id: &str,
        user_id: &str,
    ) -> Result<Vec<Relation>> {
        let rows = sqlx::query(
            r#"
            SELECT id, from_entity_id, to_entity_id, relation_type, properties, confidence, created_at
            FROM knowledge_relations
            WHERE (from_entity_id = $1 OR to_entity_id = $1)
            AND user_id = $2
            ORDER BY confidence DESC
            "#,
        )
        .bind(entity_id)
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| AgentMemError::storage_error(&format!("Failed to get entity relations: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| Relation {
                id: r.get("id"),
                from_entity_id: r.get("from_entity_id"),
                to_entity_id: r.get("to_entity_id"),
                relation_type: r.get("relation_type"),
                properties: r.get("properties"),
                confidence: r.get("confidence"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// 简单的实体提取（基于规则）
    pub fn extract_entities_simple(&self, text: &str) -> Vec<(String, EntityType, f32)> {
        let mut entities = Vec::new();

        // 简单的规则：大写开头的词可能是实体
        for word in text.split_whitespace() {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if clean_word.len() > 2 {
                    entities.push((
                        clean_word.to_string(),
                        EntityType::Concept,
                        0.6, // 中等置信度
                    ));
                }
            }
        }

        debug!("Extracted {} entities from text", entities.len());
        entities
    }
}

