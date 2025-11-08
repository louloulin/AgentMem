//! Pipeline阶段实现
//!
//! 为Memory添加和查询定义具体的Pipeline stages

use crate::types::{
    Memory, Query, PipelineStage, PipelineContext, StageResult,
    AttributeKey, AttributeValue, Constraint, ComparisonOperator,
};
use async_trait::async_trait;
use std::collections::HashMap;

// ========== 记忆添加Pipeline Stages ==========

/// Stage 1: 内容预处理
pub struct ContentPreprocessStage {
    pub min_length: usize,
    pub max_length: usize,
}

#[async_trait]
impl PipelineStage for ContentPreprocessStage {
    type Input = Memory;
    type Output = Memory;
    
    fn name(&self) -> &str {
        "ContentPreprocess"
    }
    
    async fn execute(
        &self,
        mut input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // 获取文本内容
        let text = input.content.as_text();
        
        // 检查长度
        if text.len() < self.min_length {
            return Ok(StageResult::Abort(format!(
                "Content too short: {} < {}",
                text.len(),
                self.min_length
            )));
        }
        
        if text.len() > self.max_length {
            return Ok(StageResult::Abort(format!(
                "Content too long: {} > {}",
                text.len(),
                self.max_length
            )));
        }
        
        // 存储原始长度到context
        let _ = context.set("original_length", text.len());
        
        Ok(StageResult::Continue(input))
    }
}

/// Stage 2: 去重检测
pub struct DeduplicationStage {
    pub similarity_threshold: f32,
}

#[async_trait]
impl PipelineStage for DeduplicationStage {
    type Input = Memory;
    type Output = Memory;
    
    fn name(&self) -> &str {
        "Deduplication"
    }
    
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // TODO: 实现实际的去重逻辑（需要向量搜索）
        // 这里简化处理：检查content hash
        let content_hash = format!("{:x}", md5::compute(input.content.as_text()));
        
        // 存储hash到context
        let _ = context.set("content_hash", &content_hash);
        
        // 简化：假设不重复
        Ok(StageResult::Continue(input))
    }
    
    fn is_optional(&self) -> bool {
        true // 去重失败不应该中止pipeline
    }
}

/// Stage 3: 重要性评估
pub struct ImportanceEvaluationStage {
    pub default_importance: f32,
}

#[async_trait]
impl PipelineStage for ImportanceEvaluationStage {
    type Input = Memory;
    type Output = Memory;
    
    fn name(&self) -> &str {
        "ImportanceEvaluation"
    }
    
    async fn execute(
        &self,
        mut input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // 简化：使用默认重要性或从attributes中读取
        let importance = input.attributes
            .get(&AttributeKey::domain("importance"))
            .and_then(|v| v.as_number())
            .unwrap_or(self.default_importance as f64) as f32;
        
        // 存储importance到context
        let _ = context.set("importance", importance);
        
        Ok(StageResult::Continue(input))
    }
}

/// Stage 4: 实体提取
pub struct EntityExtractionStage;

#[async_trait]
impl PipelineStage for EntityExtractionStage {
    type Input = Memory;
    type Output = Memory;
    
    fn name(&self) -> &str {
        "EntityExtraction"
    }
    
    async fn execute(
        &self,
        mut input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // TODO: 实现实际的实体提取（需要NER模型）
        // 简化处理：提取简单的ID模式
        let text = input.content.as_text();
        let entity_pattern = regex::Regex::new(r"[A-Z]\d{6}").unwrap();
        let entities: Vec<String> = entity_pattern
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .collect();
        
        // 存储entities到attributes
        if !entities.is_empty() {
            input.attributes.set(
                AttributeKey::domain("entities"),
                AttributeValue::Array(
                    entities.iter()
                        .map(|e| AttributeValue::String(e.clone()))
                        .collect()
                ),
            );
            
            // 也存储到context
            let _ = context.set("entities", &entities);
        }
        
        Ok(StageResult::Continue(input))
    }
    
    fn is_optional(&self) -> bool {
        true
    }
}

// ========== 查询Pipeline Stages ==========

/// Stage 1: 查询理解
pub struct QueryUnderstandingStage;

#[async_trait]
impl PipelineStage for QueryUnderstandingStage {
    type Input = Query;
    type Output = Query;
    
    fn name(&self) -> &str {
        "QueryUnderstanding"
    }
    
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // 分析查询意图
        let intent_type = match &input.intent {
            crate::types::QueryIntent::Lookup { .. } => "lookup",
            crate::types::QueryIntent::SemanticSearch { .. } => "semantic",
            crate::types::QueryIntent::RelationQuery { .. } => "relation",
            crate::types::QueryIntent::Aggregation { .. } => "aggregation",
        };
        
        let _ = context.set("intent_type", intent_type);
        let _ = context.set("constraint_count", input.constraints.len());
        let _ = context.set("preference_count", input.preferences.len());
        
        Ok(StageResult::Continue(input))
    }
}

/// Stage 2: 查询扩展
pub struct QueryExpansionStage {
    pub enable_synonym: bool,
}

#[async_trait]
impl PipelineStage for QueryExpansionStage {
    type Input = Query;
    type Output = Query;
    
    fn name(&self) -> &str {
        "QueryExpansion"
    }
    
    async fn execute(
        &self,
        mut input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // TODO: 实现同义词扩展、关系扩展
        // 简化处理：记录是否需要扩展
        if self.enable_synonym {
            let _ = context.set("query_expanded", true);
        }
        
        Ok(StageResult::Continue(input))
    }
    
    fn is_optional(&self) -> bool {
        true
    }
}

/// Stage 3: 约束验证
pub struct ConstraintValidationStage;

#[async_trait]
impl PipelineStage for ConstraintValidationStage {
    type Input = Query;
    type Output = Query;
    
    fn name(&self) -> &str {
        "ConstraintValidation"
    }
    
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // 验证约束的合法性
        for constraint in &input.constraints {
            match constraint {
                Constraint::Limit(limit) => {
                    if *limit == 0 {
                        return Ok(StageResult::Abort("Limit cannot be zero".to_string()));
                    }
                    if *limit > 10000 {
                        return Ok(StageResult::Abort("Limit too large (>10000)".to_string()));
                    }
                }
                Constraint::MinScore(score) => {
                    if *score < 0.0 || *score > 1.0 {
                        return Ok(StageResult::Abort("MinScore must be between 0 and 1".to_string()));
                    }
                }
                _ => {}
            }
        }
        
        let _ = context.set("constraints_valid", true);
        
        Ok(StageResult::Continue(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Content, MemoryBuilder, QueryBuilder};
    
    #[tokio::test]
    async fn test_content_preprocess_stage() {
        let stage = ContentPreprocessStage {
            min_length: 5,
            max_length: 1000,
        };
        
        let memory = MemoryBuilder::new()
            .text("Test content")
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        assert!(matches!(result, StageResult::Continue(_)));
        assert_eq!(context.get::<usize>("original_length"), Some(12));
    }
    
    #[tokio::test]
    async fn test_content_too_short() {
        let stage = ContentPreprocessStage {
            min_length: 100,
            max_length: 1000,
        };
        
        let memory = MemoryBuilder::new()
            .text("Short")
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        assert!(matches!(result, StageResult::Abort(_)));
    }
    
    #[tokio::test]
    async fn test_entity_extraction_stage() {
        let stage = EntityExtractionStage;
        
        let memory = MemoryBuilder::new()
            .text("Product P000257 and P000123 are available")
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        if let StageResult::Continue(mem) = result {
            let entities = context.get::<Vec<String>>("entities").unwrap();
            assert_eq!(entities.len(), 2);
            assert_eq!(entities[0], "P000257");
            assert_eq!(entities[1], "P000123");
        } else {
            panic!("Expected Continue");
        }
    }
    
    #[tokio::test]
    async fn test_query_understanding_stage() {
        let stage = QueryUnderstandingStage;
        
        let query = QueryBuilder::new()
            .text("Test query")
            .limit(10)
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(query, &mut context).await.unwrap();
        
        assert!(matches!(result, StageResult::Continue(_)));
        assert_eq!(context.get::<String>("intent_type"), Some("semantic".to_string()));
        assert_eq!(context.get::<usize>("constraint_count"), Some(1));
    }
    
    #[tokio::test]
    async fn test_constraint_validation_stage() {
        let stage = ConstraintValidationStage;
        
        // Valid query
        let query = QueryBuilder::new()
            .text("Test")
            .limit(100)
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(query, &mut context).await.unwrap();
        assert!(matches!(result, StageResult::Continue(_)));
        
        // Invalid query (limit = 0)
        let invalid_query = Query {
            id: "test".to_string(),
            intent: crate::types::QueryIntent::SemanticSearch {
                text: "test".to_string(),
                semantic_vector: None,
            },
            constraints: vec![Constraint::Limit(0)],
            preferences: vec![],
            context: crate::types::QueryContext::default(),
        };
        
        let mut context2 = PipelineContext::new();
        let result2 = stage.execute(invalid_query, &mut context2).await.unwrap();
        assert!(matches!(result2, StageResult::Abort(_)));
    }
}

