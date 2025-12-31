//! Schema Evolution System
//!
//! Phase 5.3: Schema演化系统
//! - Schema更新机制
//! - Schema演化算法
//! - Schema创建支持
//!
//! 参考认知发展理论，实现Schema的自动演化和更新

use agent_mem_traits::{Result, AgentMemError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Schema演化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEvolutionConfig {
    /// 启用Schema演化
    pub enable_evolution: bool,
    /// 自动演化阈值（相似记忆数量）
    pub auto_evolution_threshold: usize,
    /// Schema合并阈值（相似度）
    pub merge_threshold: f64,
    /// Schema分裂阈值（差异度）
    pub split_threshold: f64,
    /// 最小Schema大小
    pub min_schema_size: usize,
    /// 最大Schema数量
    pub max_schema_count: usize,
}

impl Default for SchemaEvolutionConfig {
    fn default() -> Self {
        Self {
            enable_evolution: true,
            auto_evolution_threshold: 10,
            merge_threshold: 0.8,
            split_threshold: 0.3,
            min_schema_size: 3,
            max_schema_count: 100,
        }
    }
}

/// Schema定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema ID
    pub id: String,
    /// Schema名称
    pub name: String,
    /// Schema描述
    pub description: String,
    /// Schema模式（抽象表示）
    pub pattern: SchemaPattern,
    /// 关联的记忆ID列表
    pub memory_ids: Vec<String>,
    /// Schema版本
    pub version: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 使用次数
    pub usage_count: u64,
    /// 置信度
    pub confidence: f64,
}

/// Schema模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPattern {
    /// 核心概念
    pub core_concept: String,
    /// 关键属性
    pub key_attributes: Vec<String>,
    /// 关系模式
    pub relation_patterns: Vec<RelationPattern>,
    /// 典型示例
    pub typical_examples: Vec<String>,
    /// 语义向量（可选）
    pub semantic_vector: Option<Vec<f32>>,
}

/// 关系模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationPattern {
    /// 关系类型
    pub relation_type: String,
    /// 目标概念
    pub target_concept: String,
    /// 频率
    pub frequency: f64,
}

/// Schema演化操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaEvolutionOperation {
    /// 创建新Schema
    Create {
        schema: Schema,
    },
    /// 更新Schema
    Update {
        schema_id: String,
        changes: SchemaChanges,
    },
    /// 合并Schema
    Merge {
        source_schemas: Vec<String>,
        target_schema_id: String,
    },
    /// 分裂Schema
    Split {
        source_schema_id: String,
        new_schemas: Vec<Schema>,
    },
    /// 删除Schema
    Delete {
        schema_id: String,
    },
}

/// Schema变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaChanges {
    /// 新增的记忆ID
    pub added_memories: Vec<String>,
    /// 移除的记忆ID
    pub removed_memories: Vec<String>,
    /// 更新的属性
    pub updated_attributes: HashMap<String, String>,
    /// 更新的模式
    pub updated_pattern: Option<SchemaPattern>,
}

/// Schema演化历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEvolutionHistory {
    /// 操作类型
    pub operation: SchemaEvolutionOperation,
    /// 操作时间
    pub timestamp: DateTime<Utc>,
    /// 操作原因
    pub reason: String,
    /// 操作结果
    pub result: EvolutionResult,
}

/// 演化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionResult {
    /// 成功
    Success {
        message: String,
    },
    /// 失败
    Failure {
        error: String,
    },
}

/// Schema演化引擎
pub struct SchemaEvolutionEngine {
    config: SchemaEvolutionConfig,
    /// Schema集合
    schemas: Arc<RwLock<HashMap<String, Schema>>>,
    /// 演化历史
    evolution_history: Arc<RwLock<Vec<SchemaEvolutionHistory>>>,
    /// 记忆到Schema的映射
    memory_to_schema: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SchemaEvolutionEngine {
    /// 创建新的Schema演化引擎
    pub fn new(config: SchemaEvolutionConfig) -> Self {
        Self {
            config,
            schemas: Arc::new(RwLock::new(HashMap::new())),
            evolution_history: Arc::new(RwLock::new(Vec::new())),
            memory_to_schema: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(SchemaEvolutionConfig::default())
    }

    /// Schema更新机制
    ///
    /// 根据新记忆自动更新Schema
    pub async fn update_schema_from_memories(
        &self,
        schema_id: &str,
        new_memory_ids: Vec<String>,
    ) -> Result<()> {
        if !self.config.enable_evolution {
            return Ok(());
        }

        info!("更新Schema: {} (新增 {} 条记忆)", schema_id, new_memory_ids.len());

        let mut schemas = self.schemas.write().await;
        let schema = schemas
            .get_mut(schema_id)
            .ok_or_else(|| AgentMemError::not_found(format!("Schema不存在: {schema_id}")))?;

        // 添加新记忆
        for memory_id in &new_memory_ids {
            if !schema.memory_ids.contains(memory_id) {
                schema.memory_ids.push(memory_id.clone());
            }
        }

        // 更新Schema模式（基于所有记忆）
        schema.pattern = self.extract_pattern_from_memories(&schema.memory_ids).await?;
        schema.version += 1;
        schema.updated_at = Utc::now();
        schema.usage_count += new_memory_ids.len() as u64;

        // 更新记忆到Schema的映射
        {
            let mut memory_map = self.memory_to_schema.write().await;
            for memory_id in &new_memory_ids {
                memory_map
                    .entry(memory_id.clone())
                    .or_insert_with(Vec::new)
                    .push(schema_id.to_string());
            }
        }

        // 记录演化历史
        {
            let mut history = self.evolution_history.write().await;
            history.push(SchemaEvolutionHistory {
                operation: SchemaEvolutionOperation::Update {
                    schema_id: schema_id.to_string(),
                    changes: SchemaChanges {
                        added_memories: new_memory_ids,
                        removed_memories: Vec::new(),
                        updated_attributes: HashMap::new(),
                        updated_pattern: Some(schema.pattern.clone()),
                    },
                },
                timestamp: Utc::now(),
                reason: "自动更新：新增记忆".to_string(),
                result: EvolutionResult::Success {
                    message: "Schema更新成功".to_string(),
                },
            });
        }

        Ok(())
    }

    /// Schema演化算法
    ///
    /// 自动演化Schema（合并、分裂、创建）
    pub async fn evolve_schemas(&self) -> Result<Vec<SchemaEvolutionOperation>> {
        if !self.config.enable_evolution {
            return Ok(Vec::new());
        }

        info!("执行Schema演化算法");

        let mut operations = Vec::new();

        // 1. 检查是否需要合并Schema
        let merge_operations = self.check_and_merge_schemas().await?;
        operations.extend(merge_operations);

        // 2. 检查是否需要分裂Schema
        let split_operations = self.check_and_split_schemas().await?;
        operations.extend(split_operations);

        // 3. 检查是否需要创建新Schema
        let create_operations = self.check_and_create_schemas().await?;
        operations.extend(create_operations);

        info!("Schema演化完成，执行了 {} 个操作", operations.len());
        Ok(operations)
    }

    /// 检查并合并Schema
    async fn check_and_merge_schemas(&self) -> Result<Vec<SchemaEvolutionOperation>> {
        let schemas = self.schemas.read().await;
        let schema_list: Vec<_> = schemas.values().collect();
        let mut operations = Vec::new();

        // 检查所有Schema对
        for i in 0..schema_list.len() {
            for j in (i + 1)..schema_list.len() {
                let schema1 = schema_list[i];
                let schema2 = schema_list[j];

                // 计算相似度
                let similarity = self.calculate_schema_similarity(schema1, schema2).await?;

                if similarity >= self.config.merge_threshold {
                    // 合并Schema
                    let merged_schema = self.merge_schemas(schema1, schema2).await?;
                    operations.push(SchemaEvolutionOperation::Merge {
                        source_schemas: vec![schema1.id.clone(), schema2.id.clone()],
                        target_schema_id: merged_schema.id.clone(),
                    });

                    // 执行合并
                    {
                        let mut schemas = self.schemas.write().await;
                        schemas.remove(&schema1.id);
                        schemas.remove(&schema2.id);
                        schemas.insert(merged_schema.id.clone(), merged_schema);
                    }
                }
            }
        }

        Ok(operations)
    }

    /// 检查并分裂Schema
    async fn check_and_split_schemas(&self) -> Result<Vec<SchemaEvolutionOperation>> {
        let schemas = self.schemas.read().await;
        let mut operations = Vec::new();

        for schema in schemas.values() {
            // 如果Schema太大或内部差异太大，考虑分裂
            if schema.memory_ids.len() > self.config.min_schema_size * 3 {
                let internal_diversity = self.calculate_internal_diversity(schema).await?;

                if internal_diversity < self.config.split_threshold {
                    // 分裂Schema
                    let new_schemas = self.split_schema(schema).await?;
                    operations.push(SchemaEvolutionOperation::Split {
                        source_schema_id: schema.id.clone(),
                        new_schemas: new_schemas.clone(),
                    });

                    // 执行分裂
                    {
                        let mut schemas = self.schemas.write().await;
                        schemas.remove(&schema.id);
                        for new_schema in new_schemas {
                            schemas.insert(new_schema.id.clone(), new_schema);
                        }
                    }
                }
            }
        }

        Ok(operations)
    }

    /// 检查并创建新Schema
    async fn check_and_create_schemas(&self) -> Result<Vec<SchemaEvolutionOperation>> {
        // 简化实现：检查未关联的记忆，如果数量足够，创建新Schema
        let operations = Vec::new();

        // 这里应该检查未关联的记忆
        // 简化实现：返回空操作列表

        Ok(operations)
    }

    /// Schema创建支持
    ///
    /// 手动创建Schema
    pub async fn create_schema(&self, schema: Schema) -> Result<()> {
        let schema_id = schema.id.clone();

        // 检查Schema数量限制
        {
            let schemas = self.schemas.read().await;
            if schemas.len() >= self.config.max_schema_count {
                return Err(AgentMemError::validation_error(format!(
                    "Schema数量已达上限: {}",
                    self.config.max_schema_count
                )));
            }
        }

        // 添加Schema
        {
            let mut schemas = self.schemas.write().await;
            schemas.insert(schema_id.clone(), schema.clone());
        }

        // 更新记忆到Schema的映射
        {
            let mut memory_map = self.memory_to_schema.write().await;
            for memory_id in &schema.memory_ids {
                memory_map
                    .entry(memory_id.clone())
                    .or_insert_with(Vec::new)
                    .push(schema_id.clone());
            }
        }

        // 记录演化历史
        {
            let mut history = self.evolution_history.write().await;
            history.push(SchemaEvolutionHistory {
                operation: SchemaEvolutionOperation::Create {
                    schema: schema.clone(),
                },
                timestamp: Utc::now(),
                reason: "手动创建Schema".to_string(),
                result: EvolutionResult::Success {
                    message: "Schema创建成功".to_string(),
                },
            });
        }

        info!("创建Schema: {} ({})", schema.name, schema_id);
        Ok(())
    }

    /// 计算Schema相似度
    async fn calculate_schema_similarity(&self, schema1: &Schema, schema2: &Schema) -> Result<f64> {
        // 基于核心概念和关键属性的相似度
        let concept_similarity = if schema1.pattern.core_concept == schema2.pattern.core_concept {
            1.0
        } else {
            0.5
        };

        // 基于关键属性的相似度
        let attr_overlap = self.calculate_attribute_overlap(
            &schema1.pattern.key_attributes,
            &schema2.pattern.key_attributes,
        );
        let attr_similarity = attr_overlap as f64
            / schema1.pattern.key_attributes.len().max(schema2.pattern.key_attributes.len()) as f64;

        // 综合相似度
        Ok((concept_similarity * 0.6 + attr_similarity * 0.4).min(1.0))
    }

    /// 计算属性重叠
    fn calculate_attribute_overlap(&self, attrs1: &[String], attrs2: &[String]) -> usize {
        let set1: HashSet<_> = attrs1.iter().collect();
        let set2: HashSet<_> = attrs2.iter().collect();
        set1.intersection(&set2).count()
    }

    /// 计算内部多样性
    async fn calculate_internal_diversity(&self, schema: &Schema) -> Result<f64> {
        // 简化实现：基于记忆数量的多样性
        // 实际应该基于记忆内容的多样性
        if schema.memory_ids.len() <= 1 {
            return Ok(1.0);
        }

        // 简化：返回固定值
        Ok(0.5)
    }

    /// 合并Schema
    async fn merge_schemas(&self, schema1: &Schema, schema2: &Schema) -> Result<Schema> {
        let merged_memory_ids: Vec<String> = schema1
            .memory_ids
            .iter()
            .chain(schema2.memory_ids.iter())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let merged_pattern = SchemaPattern {
            core_concept: format!("{}+{}", schema1.pattern.core_concept, schema2.pattern.core_concept),
            key_attributes: {
                let mut attrs = schema1.pattern.key_attributes.clone();
                for attr in &schema2.pattern.key_attributes {
                    if !attrs.contains(attr) {
                        attrs.push(attr.clone());
                    }
                }
                attrs
            },
            relation_patterns: {
                let mut patterns = schema1.pattern.relation_patterns.clone();
                patterns.extend(schema2.pattern.relation_patterns.clone());
                patterns
            },
            typical_examples: {
                let mut examples = schema1.pattern.typical_examples.clone();
                examples.extend(schema2.pattern.typical_examples.clone());
                examples
            },
            semantic_vector: None,
        };

        Ok(Schema {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{}_merged", schema1.name),
            description: format!("合并自 {} 和 {}", schema1.name, schema2.name),
            pattern: merged_pattern,
            memory_ids: merged_memory_ids,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: schema1.usage_count + schema2.usage_count,
            confidence: (schema1.confidence + schema2.confidence) / 2.0,
        })
    }

    /// 分裂Schema
    async fn split_schema(&self, schema: &Schema) -> Result<Vec<Schema>> {
        // 简化实现：将Schema分成两部分
        let mid = schema.memory_ids.len() / 2;
        let (first_half, second_half) = schema.memory_ids.split_at(mid);

        let schema1 = Schema {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{}_split_1", schema.name),
            description: format!("从 {} 分裂", schema.name),
            pattern: schema.pattern.clone(),
            memory_ids: first_half.to_vec(),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            confidence: schema.confidence * 0.8,
        };

        let schema2 = Schema {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("{}_split_2", schema.name),
            description: format!("从 {} 分裂", schema.name),
            pattern: schema.pattern.clone(),
            memory_ids: second_half.to_vec(),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            confidence: schema.confidence * 0.8,
        };

        Ok(vec![schema1, schema2])
    }

    /// 从记忆提取模式
    async fn extract_pattern_from_memories(&self, memory_ids: &[String]) -> Result<SchemaPattern> {
        // 简化实现：实际应该分析记忆内容提取模式
        Ok(SchemaPattern {
            core_concept: "extracted_concept".to_string(),
            key_attributes: vec!["attr1".to_string(), "attr2".to_string()],
            relation_patterns: Vec::new(),
            typical_examples: Vec::new(),
            semantic_vector: None,
        })
    }

    /// 获取Schema
    pub async fn get_schema(&self, schema_id: &str) -> Result<Option<Schema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas.get(schema_id).cloned())
    }

    /// 获取所有Schema
    pub async fn get_all_schemas(&self) -> Vec<Schema> {
        let schemas = self.schemas.read().await;
        schemas.values().cloned().collect()
    }

    /// 获取演化历史
    pub async fn get_evolution_history(&self, limit: usize) -> Vec<SchemaEvolutionHistory> {
        let history = self.evolution_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schema_evolution() {
        let engine = SchemaEvolutionEngine::with_defaults();

        // 创建Schema
        let schema = Schema {
            id: "schema1".to_string(),
            name: "Test Schema".to_string(),
            description: "Test".to_string(),
            pattern: SchemaPattern {
                core_concept: "concept1".to_string(),
                key_attributes: vec!["attr1".to_string()],
                relation_patterns: Vec::new(),
                typical_examples: Vec::new(),
                semantic_vector: None,
            },
            memory_ids: vec!["mem1".to_string(), "mem2".to_string()],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            confidence: 0.8,
        };

        engine.create_schema(schema).await?;

        // 更新Schema
        engine
            .update_schema_from_memories("schema1", vec!["mem3".to_string()])
            .await
            .unwrap();

        // 获取Schema
        let updated = engine.get_schema("schema1").await?;
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().memory_ids.len(), 3);
    }

    #[tokio::test]
    async fn test_schema_evolution_nonexistent() {
        let engine = SchemaEvolutionEngine::with_defaults();

        // 测试更新不存在的Schema
        let result = engine
            .update_schema_from_memories("nonexistent", vec!["mem1".to_string()])
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_schema_evolution_max_count() {
        let engine = SchemaEvolutionEngine::with_defaults();

        // 创建最大数量的Schema
        for i in 0..engine.config.max_schema_count {
            let schema = Schema {
                id: format!("schema{}", i),
                name: format!("Schema {}", i),
                description: "Test".to_string(),
                pattern: SchemaPattern {
                    core_concept: format!("concept{}", i),
                    key_attributes: vec![],
                    relation_patterns: Vec::new(),
                    typical_examples: Vec::new(),
                    semantic_vector: None,
                },
                memory_ids: vec![],
                version: 1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                usage_count: 0,
                confidence: 0.8,
            };

            engine.create_schema(schema).await?;
        }

        // 尝试创建超出限制的Schema
        let schema = Schema {
            id: "schema_overflow".to_string(),
            name: "Overflow Schema".to_string(),
            description: "Test".to_string(),
            pattern: SchemaPattern {
                core_concept: "concept".to_string(),
                key_attributes: vec![],
                relation_patterns: Vec::new(),
                typical_examples: Vec::new(),
                semantic_vector: None,
            },
            memory_ids: vec![],
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            confidence: 0.8,
        };

        let result = engine.create_schema(schema).await;
        assert!(result.is_err());
    }
}

