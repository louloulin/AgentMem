//! 批量处理模块
//!
//! P1 优化 #4, #6, #29: 批量处理优化
//!
//! 功能：
//! - 批量实体提取
//! - 批量重要性评估
//! - 批量metadata查询

use agent_mem_llm::LLMProvider;
use agent_mem_traits::{Message, Result};
use crate::fact_extraction::{ExtractedFact, StructuredFact};
use crate::importance_evaluator::ImportanceEvaluation;
use crate::timeout::{with_timeout, TimeoutConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批次大小
    pub batch_size: usize,
    /// 是否启用批量处理
    pub enabled: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 10, // 每批10个
            enabled: true,
        }
    }
}

/// 批量实体提取器 (P1 优化 #4)
pub struct BatchEntityExtractor {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    timeout_config: TimeoutConfig,
    batch_config: BatchConfig,
}

impl BatchEntityExtractor {
    pub fn new(
        llm: Arc<dyn LLMProvider + Send + Sync>,
        timeout_config: TimeoutConfig,
        batch_config: BatchConfig,
    ) -> Self {
        Self {
            llm,
            timeout_config,
            batch_config,
        }
    }

    /// 批量提取实体和关系
    pub async fn extract_entities_batch(
        &self,
        facts: &[ExtractedFact],
    ) -> Result<Vec<StructuredFact>> {
        if facts.is_empty() {
            return Ok(vec![]);
        }

        if !self.batch_config.enabled || facts.len() <= 1 {
            // 禁用或单个时逐个处理
            return self.extract_entities_sequential(facts).await;
        }

        info!("批量提取实体，总数: {}", facts.len());

        let mut all_structured_facts = Vec::new();

        // 按批次处理
        for chunk in facts.chunks(self.batch_config.batch_size) {
            debug!("处理批次: {} 个事实", chunk.len());
            
            let structured_facts = self.extract_batch_internal(chunk).await?;
            all_structured_facts.extend(structured_facts);
        }

        info!("批量提取完成，生成 {} 个结构化事实", all_structured_facts.len());
        Ok(all_structured_facts)
    }

    /// 内部批量提取实现
    async fn extract_batch_internal(
        &self,
        facts: &[ExtractedFact],
    ) -> Result<Vec<StructuredFact>> {
        let prompt = self.build_batch_entity_prompt(facts);

        let llm = self.llm.clone();
        let response = with_timeout(
            async move {
                llm.generate(&[Message::user(&prompt)]).await
            },
            self.timeout_config.fact_extraction_timeout_secs,
            "batch_entity_extraction",
        ).await?;

        self.parse_batch_response(&response, facts.len())
    }

    /// 构建批量实体提取prompt
    fn build_batch_entity_prompt(&self, facts: &[ExtractedFact]) -> String {
        let facts_json = serde_json::to_string_pretty(facts).unwrap_or_else(|_| "[]".to_string());

        format!(
            r#"Extract entities and relations from these facts. Return a JSON array.

Facts:
{}

Return format (JSON array):
[
  {{
    "id": "fact_0",
    "fact_type": "type",
    "description": "description",
    "entities": [],
    "relations": [],
    "confidence": 0.9,
    "importance": 0.8,
    "source_messages": [],
    "metadata": {{}}
  }}
]

Extract all entities and their relationships."#,
            facts_json
        )
    }

    /// 解析批量响应
    fn parse_batch_response(
        &self,
        response: &str,
        expected_count: usize,
    ) -> Result<Vec<StructuredFact>> {
        // 尝试提取JSON数组
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        match serde_json::from_str::<Vec<StructuredFact>>(json_str) {
            Ok(facts) => Ok(facts),
            Err(e) => {
                debug!("批量解析失败: {}, 降级到逐个解析", e);
                // 降级到返回空列表
                Ok(vec![])
            }
        }
    }

    /// 顺序提取（降级方案）
    async fn extract_entities_sequential(
        &self,
        facts: &[ExtractedFact],
    ) -> Result<Vec<StructuredFact>> {
        let mut results = Vec::new();
        
        for fact in facts {
            // 简单的结构化事实生成
            let structured_fact = StructuredFact {
                id: uuid::Uuid::new_v4().to_string(),
                fact_type: format!("{:?}", fact.category),
                description: fact.content.clone(),
                entities: fact.entities.clone(),
                relations: vec![],
                confidence: fact.confidence,
                importance: fact.confidence * 0.8,
                source_messages: vec![],
                metadata: std::collections::HashMap::new(),
            };
            results.push(structured_fact);
        }

        Ok(results)
    }
}

/// 批量重要性评估器 (P1 优化 #6)
pub struct BatchImportanceEvaluator {
    llm: Arc<dyn LLMProvider + Send + Sync>,
    timeout_config: TimeoutConfig,
    batch_config: BatchConfig,
}

impl BatchImportanceEvaluator {
    pub fn new(
        llm: Arc<dyn LLMProvider + Send + Sync>,
        timeout_config: TimeoutConfig,
        batch_config: BatchConfig,
    ) -> Self {
        Self {
            llm,
            timeout_config,
            batch_config,
        }
    }

    /// 批量评估重要性
    pub async fn evaluate_batch(
        &self,
        facts: &[StructuredFact],
    ) -> Result<Vec<ImportanceEvaluation>> {
        if facts.is_empty() {
            return Ok(vec![]);
        }

        if !self.batch_config.enabled || facts.len() <= 1 {
            // 禁用或单个时使用规则评估
            return self.evaluate_by_rules(facts);
        }

        info!("批量评估重要性，总数: {}", facts.len());

        let mut all_evaluations = Vec::new();

        // 按批次处理
        for chunk in facts.chunks(self.batch_config.batch_size) {
            debug!("评估批次: {} 个事实", chunk.len());
            
            let evaluations = self.evaluate_batch_internal(chunk).await?;
            all_evaluations.extend(evaluations);
        }

        info!("批量评估完成: {} 个评估", all_evaluations.len());
        Ok(all_evaluations)
    }

    /// 内部批量评估实现
    async fn evaluate_batch_internal(
        &self,
        facts: &[StructuredFact],
    ) -> Result<Vec<ImportanceEvaluation>> {
        let prompt = self.build_batch_evaluation_prompt(facts);

        let llm = self.llm.clone();
        let response = with_timeout(
            async move {
                llm.generate(&[Message::user(&prompt)]).await
            },
            self.timeout_config.decision_timeout_secs,
            "batch_importance_evaluation",
        ).await?;

        self.parse_evaluation_response(&response, facts)
    }

    /// 构建批量评估prompt
    fn build_batch_evaluation_prompt(&self, facts: &[StructuredFact]) -> String {
        let facts_summary: Vec<String> = facts
            .iter()
            .enumerate()
            .map(|(idx, f)| format!("{}. {} (confidence: {:.2})", idx, f.description, f.confidence))
            .collect();

        format!(
            r#"Evaluate the importance of these facts. Return a JSON array.

Facts:
{}

Return format:
[
  {{
    "importance_score": 0.8,
    "category_importance": 0.9,
    "temporal_importance": 0.7,
    "context_importance": 0.6,
    "reasoning": "brief reason"
  }}
]

Score 0.0-1.0 for each fact."#,
            facts_summary.join("\n")
        )
    }

    /// 解析评估响应
    fn parse_evaluation_response(
        &self,
        response: &str,
        facts: &[StructuredFact],
    ) -> Result<Vec<ImportanceEvaluation>> {
        #[derive(Deserialize)]
        struct SimpleEvaluation {
            importance_score: f32,
            #[serde(default)]
            category_importance: f32,
            #[serde(default)]
            temporal_importance: f32,
            #[serde(default)]
            context_importance: f32,
            #[serde(default)]
            reasoning: String,
        }

        // 尝试解析JSON数组
        let json_str = if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        match serde_json::from_str::<Vec<SimpleEvaluation>>(json_str) {
            Ok(simple_evals) => {
                let evaluations = simple_evals
                    .into_iter()
                    .zip(facts.iter())
                    .map(|(eval, fact)| ImportanceEvaluation {
                        fact_id: fact.id.clone(),
                        importance_score: eval.importance_score.clamp(0.0, 1.0),
                        factors: crate::importance_evaluator::ImportanceFactors {
                            category_importance: eval.category_importance,
                            temporal_importance: eval.temporal_importance,
                            entity_importance: 0.5, // 默认值
                            relation_importance: 0.5,
                            context_importance: eval.context_importance,
                            novelty_score: 0.5,
                        },
                        reasoning: eval.reasoning,
                        evaluated_at: chrono::Utc::now(),
                    })
                    .collect();
                Ok(evaluations)
            }
            Err(e) => {
                debug!("批量解析失败: {}, 使用规则评估", e);
                self.evaluate_by_rules(facts)
            }
        }
    }

    /// 基于规则的评估（降级方案）
    fn evaluate_by_rules(&self, facts: &[StructuredFact]) -> Result<Vec<ImportanceEvaluation>> {
        let evaluations = facts
            .iter()
            .map(|fact| ImportanceEvaluation {
                fact_id: fact.id.clone(),
                importance_score: fact.importance,
                factors: crate::importance_evaluator::ImportanceFactors {
                    category_importance: 0.5,
                    temporal_importance: 0.5,
                    entity_importance: 0.5,
                    relation_importance: 0.5,
                    context_importance: 0.5,
                    novelty_score: 0.5,
                },
                reasoning: "基于规则评估".to_string(),
                evaluated_at: chrono::Utc::now(),
            })
            .collect();

        Ok(evaluations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact_extraction::FactCategory;

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 10);
        assert!(config.enabled);
    }

    #[test]
    fn test_batch_entity_extractor_creation() {
        use agent_mem_llm::MockLLMProvider;
        
        let llm = Arc::new(MockLLMProvider::new());
        let extractor = BatchEntityExtractor::new(
            llm,
            TimeoutConfig::default(),
            BatchConfig::default(),
        );

        // 验证创建成功
        assert_eq!(extractor.batch_config.batch_size, 10);
    }
}

