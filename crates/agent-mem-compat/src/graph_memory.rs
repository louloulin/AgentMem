//! # 图记忆管理器
//!
//! 这个模块实现了图记忆功能，将图数据库集成到 Mem0 兼容层中，
//! 支持实体关系存储、图查询和智能记忆融合。

use agent_mem_config::memory::GraphStoreConfig;
use agent_mem_storage::graph::Neo4jStore;
use agent_mem_traits::{AgentMemError, Entity, GraphResult, GraphStore, Relation, Result, Session};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 图记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMemoryConfig {
    /// 图数据库配置
    pub graph_store: GraphStoreConfig,
    /// 是否启用自动实体提取
    pub auto_entity_extraction: bool,
    /// 是否启用关系推理
    pub enable_relation_inference: bool,
    /// 最大图遍历深度
    pub max_traversal_depth: usize,
    /// 实体相似度阈值
    pub entity_similarity_threshold: f32,
    /// 关系置信度阈值
    pub relation_confidence_threshold: f32,
}

impl Default for GraphMemoryConfig {
    fn default() -> Self {
        Self {
            graph_store: GraphStoreConfig {
                provider: "neo4j".to_string(),
                uri: "bolt://localhost:7687".to_string(),
                username: Some("neo4j".to_string()),
                password: Some("password".to_string()),
                database: Some("neo4j".to_string()),
            },
            auto_entity_extraction: true,
            enable_relation_inference: true,
            max_traversal_depth: 3,
            entity_similarity_threshold: 0.8,
            relation_confidence_threshold: 0.7,
        }
    }
}

/// 图记忆管理器
pub struct GraphMemoryManager {
    config: GraphMemoryConfig,
    graph_store: Arc<dyn GraphStore + Send + Sync>,
}

impl GraphMemoryManager {
    /// 创建新的图记忆管理器
    pub async fn new(config: GraphMemoryConfig) -> Result<Self> {
        info!(
            "Initializing GraphMemoryManager with provider: {}",
            config.graph_store.provider
        );

        let graph_store = match config.graph_store.provider.as_str() {
            "neo4j" => {
                let neo4j_store = Neo4jStore::new(config.graph_store.clone()).await?;
                Arc::new(neo4j_store) as Arc<dyn GraphStore + Send + Sync>
            }
            provider => {
                return Err(AgentMemError::config_error(format!(
                    "Unsupported graph store provider: {provider}"
                )));
            }
        };

        Ok(Self {
            config,
            graph_store,
        })
    }

    /// 从文本内容提取实体和关系
    pub async fn extract_entities_and_relations(
        &self,
        content: &str,
        session: &Session,
    ) -> Result<(Vec<Entity>, Vec<Relation>)> {
        debug!(
            "Extracting entities and relations from content: {}",
            content
        );

        if !self.config.auto_entity_extraction {
            return Ok((Vec::new(), Vec::new()));
        }

        // 简化的实体提取实现
        // 在真实实现中，这里会使用 NLP 模型进行实体识别
        let entities = self.extract_entities_simple(content).await?;
        let relations = self.extract_relations_simple(content, &entities).await?;

        info!(
            "Extracted {} entities and {} relations",
            entities.len(),
            relations.len()
        );
        Ok((entities, relations))
    }

    /// 简化的实体提取
    async fn extract_entities_simple(&self, content: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();

        // 简单的关键词匹配实体提取
        let keywords = [
            ("人名", vec!["张三", "李四", "王五", "小明", "小红"]),
            ("地点", vec!["北京", "上海", "广州", "深圳", "杭州"]),
            ("公司", vec!["阿里巴巴", "腾讯", "百度", "字节跳动", "华为"]),
            ("食物", vec!["披萨", "意大利面", "寿司", "火锅", "烧烤"]),
            (
                "技术",
                vec!["人工智能", "机器学习", "深度学习", "区块链", "云计算"],
            ),
        ];

        for (entity_type, words) in keywords.iter() {
            for word in words {
                if content.contains(word) {
                    let entity = Entity {
                        id: format!("{entity_type}_{word}"),
                        name: word.to_string(),
                        entity_type: entity_type.to_string(),
                        attributes: HashMap::from([
                            (
                                "source".to_string(),
                                serde_json::Value::String("text_extraction".to_string()),
                            ),
                            (
                                "confidence".to_string(),
                                serde_json::Value::Number(
                                    serde_json::Number::from_f64(0.8).unwrap(),
                                ),
                            ),
                        ]),
                    };
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    /// 简化的关系提取
    async fn extract_relations_simple(
        &self,
        content: &str,
        entities: &[Entity],
    ) -> Result<Vec<Relation>> {
        let mut relations = Vec::new();

        // 简单的关系模式匹配
        let relation_patterns = [
            ("喜欢", vec!["喜欢", "爱", "偏爱"]),
            ("不喜欢", vec!["不喜欢", "讨厌", "厌恶"]),
            ("工作于", vec!["在", "工作", "就职"]),
            ("居住于", vec!["住在", "居住", "生活在"]),
            ("学习", vec!["学习", "研究", "掌握"]),
        ];

        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let entity1 = &entities[i];
                let entity2 = &entities[j];

                for (relation_type, patterns) in relation_patterns.iter() {
                    for pattern in patterns {
                        if content.contains(pattern)
                            && content.contains(&entity1.name)
                            && content.contains(&entity2.name)
                        {
                            let relation = Relation {
                                id: format!(
                                    "{}_{}_{}_{}",
                                    entity1.id, relation_type, entity2.id, pattern
                                ),
                                source: entity1.id.clone(),
                                target: entity2.id.clone(),
                                relation: relation_type.to_string(),
                                confidence: 0.7,
                            };
                            relations.push(relation);
                        }
                    }
                }
            }
        }

        Ok(relations)
    }

    /// 添加记忆到图数据库
    pub async fn add_memory_to_graph(&self, content: &str, session: &Session) -> Result<()> {
        let (entities, relations) = self
            .extract_entities_and_relations(content, session)
            .await?;

        if !entities.is_empty() {
            self.graph_store.add_entities(&entities, session).await?;
            info!("Added {} entities to graph", entities.len());
        }

        if !relations.is_empty() {
            self.graph_store.add_relations(&relations, session).await?;
            info!("Added {} relations to graph", relations.len());
        }

        Ok(())
    }

    /// 图搜索
    pub async fn search_graph(&self, query: &str, session: &Session) -> Result<Vec<GraphResult>> {
        debug!("Searching graph with query: {}", query);
        let results = self.graph_store.search_graph(query, session).await?;
        info!("Graph search returned {} results", results.len());
        Ok(results)
    }

    /// 获取实体的邻居
    pub async fn get_entity_neighbors(
        &self,
        entity_id: &str,
        depth: Option<usize>,
    ) -> Result<Vec<Entity>> {
        let depth = depth.unwrap_or(self.config.max_traversal_depth);
        debug!(
            "Getting neighbors for entity {} with depth {}",
            entity_id, depth
        );

        let neighbors = self.graph_store.get_neighbors(entity_id, depth).await?;
        info!(
            "Found {} neighbors for entity {}",
            neighbors.len(),
            entity_id
        );
        Ok(neighbors)
    }

    /// 智能记忆融合
    pub async fn fuse_memories(
        &self,
        memories: &[String],
        session: &Session,
    ) -> Result<FusedMemory> {
        info!("Fusing {} memories", memories.len());

        let mut all_entities = Vec::new();
        let mut all_relations = Vec::new();

        // 从所有记忆中提取实体和关系
        for memory in memories {
            let (entities, relations) =
                self.extract_entities_and_relations(memory, session).await?;
            all_entities.extend(entities);
            all_relations.extend(relations);
        }

        // 去重和合并相似实体
        let merged_entities = self.merge_similar_entities(all_entities).await?;
        let merged_relations = self.merge_similar_relations(all_relations).await?;

        // 生成融合后的记忆摘要
        let summary = self
            .generate_memory_summary(&merged_entities, &merged_relations)
            .await?;

        // 计算融合置信度
        let confidence = self.calculate_fusion_confidence(&merged_entities, &merged_relations);

        Ok(FusedMemory {
            summary,
            entities: merged_entities,
            relations: merged_relations,
            confidence,
        })
    }

    /// 合并相似实体
    async fn merge_similar_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>> {
        let mut merged = Vec::new();
        let mut processed = vec![false; entities.len()];

        for i in 0..entities.len() {
            if processed[i] {
                continue;
            }

            let mut base_entity = entities[i].clone();
            processed[i] = true;

            // 查找相似实体
            for j in (i + 1)..entities.len() {
                if processed[j] {
                    continue;
                }

                if self.are_entities_similar(&base_entity, &entities[j]) {
                    // 合并属性
                    for (key, value) in &entities[j].attributes {
                        base_entity.attributes.insert(key.clone(), value.clone());
                    }
                    processed[j] = true;
                }
            }

            merged.push(base_entity);
        }

        Ok(merged)
    }

    /// 判断实体是否相似
    fn are_entities_similar(&self, entity1: &Entity, entity2: &Entity) -> bool {
        // 简单的相似度计算
        entity1.entity_type == entity2.entity_type
            && (entity1.name == entity2.name
                || self.calculate_string_similarity(&entity1.name, &entity2.name)
                    > self.config.entity_similarity_threshold)
    }

    /// 计算字符串相似度
    fn calculate_string_similarity(&self, s1: &str, s2: &str) -> f32 {
        // 简化的 Jaccard 相似度
        let set1: std::collections::HashSet<char> = s1.chars().collect();
        let set2: std::collections::HashSet<char> = s2.chars().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }

    /// 合并相似关系
    async fn merge_similar_relations(&self, relations: Vec<Relation>) -> Result<Vec<Relation>> {
        let mut merged = Vec::new();

        for relation in relations {
            if relation.confidence >= self.config.relation_confidence_threshold {
                merged.push(relation);
            }
        }

        Ok(merged)
    }

    /// 生成记忆摘要
    async fn generate_memory_summary(
        &self,
        entities: &[Entity],
        relations: &[Relation],
    ) -> Result<String> {
        let mut summary = String::new();

        if !entities.is_empty() {
            summary.push_str(&format!("发现 {} 个实体: ", entities.len()));
            let entity_names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
            summary.push_str(&entity_names.join(", "));
            summary.push_str(". ");
        }

        if !relations.is_empty() {
            summary.push_str(&format!("发现 {} 个关系: ", relations.len()));
            let relation_types: Vec<&str> = relations.iter().map(|r| r.relation.as_str()).collect();
            summary.push_str(&relation_types.join(", "));
            summary.push('.');
        }

        Ok(summary)
    }

    /// 计算融合置信度
    fn calculate_fusion_confidence(&self, entities: &[Entity], relations: &[Relation]) -> f32 {
        if entities.is_empty() && relations.is_empty() {
            return 0.0;
        }

        let entity_confidence: f32 = entities
            .iter()
            .filter_map(|e| e.attributes.get("confidence")?.as_f64())
            .map(|c| c as f32)
            .sum::<f32>()
            / entities.len() as f32;

        let relation_confidence: f32 =
            relations.iter().map(|r| r.confidence).sum::<f32>() / relations.len() as f32;

        (entity_confidence + relation_confidence) / 2.0
    }

    /// 重置图数据库
    pub async fn reset(&self) -> Result<()> {
        warn!("Resetting graph database");
        self.graph_store.reset().await
    }
}

/// 融合后的记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedMemory {
    /// 记忆摘要
    pub summary: String,
    /// 提取的实体
    pub entities: Vec<Entity>,
    /// 提取的关系
    pub relations: Vec<Relation>,
    /// 融合置信度
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::Session;
    use async_trait::async_trait;

    #[tokio::test]
    async fn test_graph_memory_manager_creation() {
        let config = GraphMemoryConfig::default();

        // 这个测试在没有真实 Neo4j 实例时会失败，但可以验证配置
        assert_eq!(config.graph_store.provider, "neo4j");
        assert!(config.auto_entity_extraction);
        assert!(config.enable_relation_inference);
    }

    #[test]
    fn test_string_similarity() {
        let config = GraphMemoryConfig::default();
        let manager = GraphMemoryManager {
            config: config.clone(),
            graph_store: Arc::new(MockGraphStore {}),
        };

        let similarity = manager.calculate_string_similarity("hello", "hello");
        assert_eq!(similarity, 1.0);

        let similarity = manager.calculate_string_similarity("hello", "world");
        assert!(similarity < 1.0);
    }

    // Mock implementation for testing
    struct MockGraphStore;

    #[async_trait]
    impl GraphStore for MockGraphStore {
        async fn add_entities(&self, _entities: &[Entity], _session: &Session) -> Result<()> {
            Ok(())
        }

        async fn add_relations(&self, _relations: &[Relation], _session: &Session) -> Result<()> {
            Ok(())
        }

        async fn search_graph(&self, _query: &str, _session: &Session) -> Result<Vec<GraphResult>> {
            Ok(Vec::new())
        }

        async fn get_neighbors(&self, _entity_id: &str, _depth: usize) -> Result<Vec<Entity>> {
            Ok(Vec::new())
        }

        async fn reset(&self) -> Result<()> {
            Ok(())
        }
    }
}
