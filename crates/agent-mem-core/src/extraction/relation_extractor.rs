//! 关系提取器实现

use super::types::{Entity, EntityType, Relation, RelationType};
use crate::CoreResult;
use async_trait::async_trait;
use tracing::{debug, info};
use uuid::Uuid;

/// 关系提取器 trait
#[async_trait]
pub trait RelationExtractor: Send + Sync {
    /// 从文本和实体中提取关系
    async fn extract_relations(&self, text: &str, entities: &[Entity])
        -> CoreResult<Vec<Relation>>;
}

/// 基于规则的关系提取器
///
/// 使用模式匹配和规则来识别实体之间的关系
pub struct RuleBasedRelationExtractor {
    /// 工作关系关键词
    work_keywords: Vec<&'static str>,
    /// 位置关系关键词
    location_keywords: Vec<&'static str>,
    /// 喜好关系关键词
    like_keywords: Vec<&'static str>,
    /// 厌恶关系关键词
    dislike_keywords: Vec<&'static str>,
    /// 拥有关系关键词
    own_keywords: Vec<&'static str>,
}

impl RuleBasedRelationExtractor {
    /// 创建新的基于规则的关系提取器
    pub fn new() -> Self {
        Self {
            work_keywords: vec![
                "在",
                "工作",
                "就职",
                "任职",
                "供职",
                "works at",
                "employed by",
            ],
            location_keywords: vec!["位于", "在", "坐落于", "located at", "in"],
            like_keywords: vec!["喜欢", "爱", "热爱", "钟爱", "like", "love", "enjoy"],
            dislike_keywords: vec!["不喜欢", "讨厌", "厌恶", "hate", "dislike"],
            own_keywords: vec!["拥有", "有", "持有", "own", "have", "possess"],
        }
    }

    /// 提取工作关系（人物-组织）
    fn extract_work_relations(&self, text: &str, entities: &[Entity]) -> Vec<Relation> {
        let mut relations = Vec::new();

        // 查找人物和组织实体
        let persons: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();

        let orgs: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Organization)
            .collect();

        // 检查工作关系模式
        for person in &persons {
            for org in &orgs {
                // 检查是否包含工作关键词
                let has_work_keyword = self.work_keywords.iter().any(|kw| text.contains(kw));

                if has_work_keyword {
                    // 检查人物和组织在文本中的相对位置
                    if let (Some((p_start, _)), Some((o_start, _))) = (person.span, org.span) {
                        // 如果人物在组织前面，且距离不太远（小于50个字符）
                        if p_start < o_start && (o_start - p_start) < 50 {
                            let relation = Relation::new(
                                format!("rel_{}", Uuid::new_v4()),
                                person.id.clone(),
                                person.name.clone(),
                                "工作于".to_string(),
                                org.id.clone(),
                                org.name.clone(),
                                RelationType::WorksAt,
                                0.75,
                            );
                            relations.push(relation);
                        }
                    }
                }
            }
        }

        relations
    }

    /// 提取位置关系（实体-地点）
    fn extract_location_relations(&self, text: &str, entities: &[Entity]) -> Vec<Relation> {
        let mut relations = Vec::new();

        // 查找地点实体
        let locations: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Location)
            .collect();

        // 查找其他实体
        let other_entities: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type != EntityType::Location)
            .collect();

        // 检查位置关系模式
        for entity in &other_entities {
            for location in &locations {
                let has_location_keyword =
                    self.location_keywords.iter().any(|kw| text.contains(kw));

                if has_location_keyword {
                    if let (Some((e_start, _)), Some((l_start, _))) = (entity.span, location.span) {
                        // 如果实体在地点前面，且距离不太远
                        if e_start < l_start && (l_start - e_start) < 50 {
                            let relation = Relation::new(
                                format!("rel_{}", Uuid::new_v4()),
                                entity.id.clone(),
                                entity.name.clone(),
                                "位于".to_string(),
                                location.id.clone(),
                                location.name.clone(),
                                RelationType::LocatedAt,
                                0.70,
                            );
                            relations.push(relation);
                        }
                    }
                }
            }
        }

        relations
    }

    /// 提取喜好关系（人物-事物）
    fn extract_preference_relations(&self, text: &str, entities: &[Entity]) -> Vec<Relation> {
        let mut relations = Vec::new();

        // 查找人物实体
        let persons: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();

        // 查找其他实体（可能是喜好对象）
        let objects: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type != EntityType::Person)
            .collect();

        // 检查喜好关系
        for person in &persons {
            for object in &objects {
                // 检查喜欢关键词
                let has_like = self.like_keywords.iter().any(|kw| text.contains(kw));
                let has_dislike = self.dislike_keywords.iter().any(|kw| text.contains(kw));

                if has_like || has_dislike {
                    if let (Some((p_start, _)), Some((o_start, _))) = (person.span, object.span) {
                        if p_start < o_start && (o_start - p_start) < 50 {
                            let (predicate, rel_type) = if has_like {
                                ("喜欢".to_string(), RelationType::Likes)
                            } else {
                                ("不喜欢".to_string(), RelationType::Dislikes)
                            };

                            let relation = Relation::new(
                                format!("rel_{}", Uuid::new_v4()),
                                person.id.clone(),
                                person.name.clone(),
                                predicate,
                                object.id.clone(),
                                object.name.clone(),
                                rel_type,
                                0.70,
                            );
                            relations.push(relation);
                        }
                    }
                }
            }
        }

        relations
    }

    /// 提取拥有关系（人物-事物）
    fn extract_ownership_relations(&self, text: &str, entities: &[Entity]) -> Vec<Relation> {
        let mut relations = Vec::new();

        // 查找人物实体
        let persons: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Person)
            .collect();

        // 查找其他实体
        let objects: Vec<&Entity> = entities
            .iter()
            .filter(|e| e.entity_type != EntityType::Person)
            .collect();

        // 检查拥有关系
        for person in &persons {
            for object in &objects {
                let has_own_keyword = self.own_keywords.iter().any(|kw| text.contains(kw));

                if has_own_keyword {
                    if let (Some((p_start, _)), Some((o_start, _))) = (person.span, object.span) {
                        if p_start < o_start && (o_start - p_start) < 50 {
                            let relation = Relation::new(
                                format!("rel_{}", Uuid::new_v4()),
                                person.id.clone(),
                                person.name.clone(),
                                "拥有".to_string(),
                                object.id.clone(),
                                object.name.clone(),
                                RelationType::Owns,
                                0.65,
                            );
                            relations.push(relation);
                        }
                    }
                }
            }
        }

        relations
    }

    /// 去重关系
    fn deduplicate_relations(&self, relations: Vec<Relation>) -> Vec<Relation> {
        let mut unique_relations = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for relation in relations {
            let key = format!(
                "{}:{}:{}",
                relation.subject_id,
                relation.relation_type.as_str(),
                relation.object_id
            );

            if !seen.contains(&key) {
                seen.insert(key);
                unique_relations.push(relation);
            }
        }

        unique_relations
    }
}

#[async_trait]
impl RelationExtractor for RuleBasedRelationExtractor {
    async fn extract_relations(
        &self,
        text: &str,
        entities: &[Entity],
    ) -> CoreResult<Vec<Relation>> {
        info!("开始提取关系，实体数量: {}", entities.len());

        let mut all_relations = Vec::new();

        // 提取各类关系
        all_relations.extend(self.extract_work_relations(text, entities));
        all_relations.extend(self.extract_location_relations(text, entities));
        all_relations.extend(self.extract_preference_relations(text, entities));
        all_relations.extend(self.extract_ownership_relations(text, entities));

        // 去重
        all_relations = self.deduplicate_relations(all_relations);

        debug!("提取到 {} 个关系", all_relations.len());

        Ok(all_relations)
    }
}

impl Default for RuleBasedRelationExtractor {
    fn default() -> Self {
        Self::new()
    }
}
