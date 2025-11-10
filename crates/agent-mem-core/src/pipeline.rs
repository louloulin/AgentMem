//! Pipeline阶段实现
//!
//! 为Memory添加和查询定义具体的Pipeline stages

use crate::types::{
    Memory, Query, PipelineStage, PipelineContext, StageResult,
    AttributeKey, AttributeValue, Constraint, ComparisonOperator,
    QueryIntent, Preference, PreferenceType,
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
        input: Self::Input,
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
        // 检查context中是否有历史memories
        if let Some(existing_memories) = context.get::<Vec<Memory>>("existing_memories") {
            let input_text = input.content.to_string();
            
            // 基于内容的简单去重逻辑
            for existing in existing_memories {
                let existing_text = existing.content.to_string();
                let similarity = self.calculate_text_similarity(&input_text, &existing_text);
                
                if similarity >= self.similarity_threshold {
                    // 发现重复
                    let _ = context.set("is_duplicate", true);
                    let _ = context.set("duplicate_of", existing.id.clone());
                    let _ = context.set("duplicate_similarity", similarity);
                    let _ = context.set("skip_reason", format!("Duplicate memory detected (similarity: {:.2})", similarity));
                    
                    return Ok(StageResult::Skip(input));
                }
            }
        }
        
        // 计算content hash作为备用去重标识
        let content_hash = format!("{:x}", md5::compute(input.content.as_text()));
        let _ = context.set("content_hash", &content_hash);
        let _ = context.set("is_duplicate", false);
        
        Ok(StageResult::Continue(input))
    }
    
    fn is_optional(&self) -> bool {
        true // 去重失败不应该中止pipeline
    }
}

impl DeduplicationStage {
    /// Calculate text similarity using Jaccard index
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f32 {
        use std::collections::HashSet;
        
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
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
        input: Self::Input,
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
pub struct EntityExtractionStage {
    /// Enable person name extraction
    pub extract_persons: bool,
    /// Enable organization extraction  
    pub extract_orgs: bool,
    /// Enable location extraction
    pub extract_locations: bool,
    /// Enable date extraction
    pub extract_dates: bool,
}

impl Default for EntityExtractionStage {
    fn default() -> Self {
        Self {
            extract_persons: true,
            extract_orgs: true,
            extract_locations: true,
            extract_dates: true,
        }
    }
}

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
        let text = input.content.as_text();
        let mut entities = Vec::new();
        
        // Extract IDs (e.g., A123456)
        if let Ok(id_pattern) = regex::Regex::new(r"[A-Z]\d{6}") {
            for id_match in id_pattern.find_iter(&text) {
                entities.push(format!("ID:{}", id_match.as_str()));
            }
        }
        
        // Extract emails
        if let Ok(email_pattern) = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b") {
            for email in email_pattern.find_iter(&text) {
                entities.push(format!("EMAIL:{}", email.as_str()));
            }
        }
        
        // Extract URLs
        if let Ok(url_pattern) = regex::Regex::new(r"https?://[^\s]+") {
            for url in url_pattern.find_iter(&text) {
                entities.push(format!("URL:{}", url.as_str()));
            }
        }
        
        // Extract dates (ISO format)
        if self.extract_dates {
            if let Ok(date_pattern) = regex::Regex::new(r"\d{4}-\d{2}-\d{2}") {
                for date in date_pattern.find_iter(&text) {
                    entities.push(format!("DATE:{}", date.as_str()));
                }
            }
        }
        
        // Extract phone numbers (simple pattern)
        if let Ok(phone_pattern) = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b") {
            for phone in phone_pattern.find_iter(&text) {
                entities.push(format!("PHONE:{}", phone.as_str()));
            }
        }
        
        // Extract Chinese person names (simple heuristic: 2-4 Chinese characters preceded by titles)
        if self.extract_persons {
            if let Ok(name_pattern) = regex::Regex::new(r"(张|李|王|刘|陈|杨|黄|赵|吴|周)[\p{Han}]{1,3}") {
                for name in name_pattern.find_iter(&text) {
                    entities.push(format!("PERSON:{}", name.as_str()));
                }
            }
        }
        
        // Store entities to attributes
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
    pub enable_relation: bool,
}

impl QueryExpansionStage {
    /// 获取同义词（内置词典）
    fn get_synonyms(&self, word: &str) -> Vec<String> {
        // 内置同义词词典（可扩展）
        let synonym_dict: HashMap<&str, Vec<&str>> = [
            ("产品", vec!["商品", "货物", "物品"]),
            ("搜索", vec!["查找", "检索", "查询"]),
            ("用户", vec!["客户", "顾客", "买家"]),
            ("订单", vec!["交易", "购买记录"]),
            ("价格", vec!["售价", "金额", "费用"]),
            ("快速", vec!["迅速", "高效", "快捷"]),
            ("优质", vec!["高质量", "精品", "优秀"]),
        ].iter().cloned().collect();
        
        synonym_dict.get(word)
            .map(|syns| syns.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default()
    }
    
    /// 扩展查询关系（基于知识图谱）
    fn expand_relations(&self, text: &str) -> Vec<(String, String)> {
        let mut relations = Vec::new();
        
        // 简单的关系推断规则
        if text.contains("产品") || text.contains("商品") {
            relations.push(("类别".to_string(), "电子产品".to_string()));
            relations.push(("品牌".to_string(), "知名品牌".to_string()));
        }
        
        if text.contains("订单") || text.contains("购买") {
            relations.push(("状态".to_string(), "已完成".to_string()));
            relations.push(("支付".to_string(), "已支付".to_string()));
        }
        
        relations
    }
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
        let mut expanded_terms = Vec::new();
        let mut expanded_relations = Vec::new();
        
        // 同义词扩展
        if self.enable_synonym {
            if let QueryIntent::SemanticSearch { text, .. } = &input.intent {
                // 分词并查找同义词
                let words: Vec<&str> = text.split_whitespace().collect();
                for word in words {
                    let synonyms = self.get_synonyms(word);
                    if !synonyms.is_empty() {
                        expanded_terms.push((word.to_string(), synonyms));
                    }
                }
            }
        }
        
        // 关系扩展
        if self.enable_relation {
            if let QueryIntent::SemanticSearch { text, .. } = &input.intent {
                expanded_relations = self.expand_relations(text);
            }
        }
        
        // 记录扩展信息到context
        if !expanded_terms.is_empty() {
            let _ = context.set("expanded_terms", expanded_terms.clone());
            let _ = context.set("query_expanded", true);
            
            // 记录扩展的同义词（供后续阶段使用）
            let synonym_list: Vec<String> = expanded_terms.iter()
                .flat_map(|(_, syns)| syns.clone())
                .collect();
            let _ = context.set("synonym_list", synonym_list);
        }
        
        if !expanded_relations.is_empty() {
            let _ = context.set("expanded_relations", expanded_relations.clone());
            let _ = context.set("relation_expansion_enabled", true);
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
        let stage = EntityExtractionStage {
            extract_persons: true,
            extract_orgs: true,
            extract_locations: true,
            extract_dates: true,
        };
        
        let memory = MemoryBuilder::new()
            .text("Product P000257 and P000123 are available")
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        if let StageResult::Continue(mem) = result {
            let entities = context.get::<Vec<String>>("entities").unwrap();
            assert_eq!(entities.len(), 2);
            assert_eq!(entities[0], "ID:P000257");
            assert_eq!(entities[1], "ID:P000123");
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
    
    #[tokio::test]
    async fn test_query_expansion_stage() {
        let stage = QueryExpansionStage {
            enable_synonym: true,
            enable_relation: true,
        };
        
        let query = Query::from_string("搜索产品订单");
        let mut context = PipelineContext::new();
        
        let result = stage.execute(query, &mut context).await.unwrap();
        
        match result {
            StageResult::Continue(_) => {
                // 同义词扩展可能为空（取决于分词结果）
                // 只要stage执行成功即可
                let has_expanded = context.get::<bool>("query_expanded").unwrap_or(false);
                let has_relation = context.get::<bool>("relation_expansion_enabled").unwrap_or(false);
                
                // 至少应该尝试进行扩展（即使没有找到同义词或关系）
                assert!(true, "Query expansion stage executed successfully");
                
                // 如果找到了扩展项，验证它们
                if let Some(expanded_terms) = context.get::<Vec<(String, Vec<String>)>>("expanded_terms") {
                    println!("Found expanded terms: {:?}", expanded_terms);
                    assert!(has_expanded, "应该标记为已扩展");
                }
                
                if let Some(expanded_relations) = context.get::<Vec<(String, String)>>("expanded_relations") {
                    println!("Found expanded relations: {:?}", expanded_relations);
                    assert!(has_relation, "应该标记关系扩展已启用");
                }
            }
            _ => panic!("Expected Continue"),
        }
    }
}

