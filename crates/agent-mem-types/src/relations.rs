//! Relation type definitions
//! 
//! Defines relationships between memories

use serde::{Deserialize, Serialize};

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    /// 引用关系
    References,
    /// 替代关系
    Supersedes,
    /// 部分关系
    PartOf,
    /// 相似关系
    SimilarTo,
    /// 因果关系
    CausedBy,
    /// 自定义关系
    Custom(String),
}

impl RelationType {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            RelationType::References => "references",
            RelationType::Supersedes => "supersedes",
            RelationType::PartOf => "part_of",
            RelationType::SimilarTo => "similar_to",
            RelationType::CausedBy => "caused_by",
            RelationType::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 关系（记忆间的关系）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 目标记忆ID
    pub target_id: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 关系强度（0.0-1.0）
    pub strength: f32,
}

impl Relation {
    /// Create a new relation
    pub fn new(target_id: String, relation_type: RelationType) -> Self {
        Self {
            target_id,
            relation_type,
            strength: 1.0,
        }
    }

    /// Create with custom strength
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

/// 关系图（记忆间的关系网络）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationGraph {
    relations: Vec<Relation>,
}

impl RelationGraph {
    /// Create an empty relation graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a relation
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// Get all relations
    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Find relations by type
    pub fn find_by_type(&self, relation_type: &RelationType) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| std::mem::discriminant(&r.relation_type) == std::mem::discriminant(relation_type))
            .collect()
    }

    /// Find relations targeting a specific memory
    pub fn find_by_target(&self, target_id: &str) -> Vec<&Relation> {
        self.relations
            .iter()
            .filter(|r| r.target_id == target_id)
            .collect()
    }

    /// Get relation count
    pub fn len(&self) -> usize {
        self.relations.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.relations.is_empty()
    }
}
