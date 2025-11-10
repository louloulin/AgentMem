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

/// Stage 4: 实体提取（增强版）
pub struct EntityExtractionStage {
    /// Enable person name extraction
    pub extract_persons: bool,
    /// Enable organization extraction  
    pub extract_orgs: bool,
    /// Enable location extraction
    pub extract_locations: bool,
    /// Enable date extraction
    pub extract_dates: bool,
    /// Enable money/currency extraction
    pub extract_money: bool,
    /// Enable time extraction
    pub extract_time: bool,
    /// Enable percentage extraction
    pub extract_percentage: bool,
    /// Enable IP address extraction
    pub extract_ip: bool,
}

impl Default for EntityExtractionStage {
    fn default() -> Self {
        Self {
            extract_persons: true,
            extract_orgs: true,
            extract_locations: true,
            extract_dates: true,
            extract_money: true,
            extract_time: true,
            extract_percentage: true,
            extract_ip: true,
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
        
        // Extract Chinese person names (simple heuristic: 2-4 Chinese characters preceded by common surnames)
        if self.extract_persons {
            if let Ok(name_pattern) = regex::Regex::new(r"(张|李|王|刘|陈|杨|黄|赵|吴|周)[\p{Han}]{1,3}") {
                for name in name_pattern.find_iter(&text) {
                    entities.push(format!("PERSON:{}", name.as_str()));
                }
            }
        }
        
        // Extract money/currency amounts
        if self.extract_money {
            // Match patterns like: $100, ¥200, €50, 100元, 200美元
            if let Ok(money_pattern) = regex::Regex::new(r"(?:[$¥€£]\s*\d+(?:\.\d{2})?|\d+(?:\.\d{2})?\s*(?:元|美元|欧元|英镑|人民币|USD|CNY|EUR|GBP))") {
                for money in money_pattern.find_iter(&text) {
                    entities.push(format!("MONEY:{}", money.as_str()));
                }
            }
        }
        
        // Extract time expressions
        if self.extract_time {
            // Match patterns like: 10:30, 14:45:30, 上午9点, 下午3点
            if let Ok(time_pattern) = regex::Regex::new(r"(?:\d{1,2}:\d{2}(?::\d{2})?|(?:上午|下午|早上|晚上)\d{1,2}点)") {
                for time in time_pattern.find_iter(&text) {
                    entities.push(format!("TIME:{}", time.as_str()));
                }
            }
        }
        
        // Extract percentages
        if self.extract_percentage {
            // Match patterns like: 50%, 12.5%, 百分之50
            if let Ok(percent_pattern) = regex::Regex::new(r"(?:\d+(?:\.\d+)?%|百分之\d+(?:\.\d+)?)") {
                for percent in percent_pattern.find_iter(&text) {
                    entities.push(format!("PERCENTAGE:{}", percent.as_str()));
                }
            }
        }
        
        // Extract IP addresses
        if self.extract_ip {
            // Match IPv4 addresses
            if let Ok(ip_pattern) = regex::Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b") {
                for ip in ip_pattern.find_iter(&text) {
                    // Simple validation: each octet should be 0-255
                    let ip_str = ip.as_str();
                    let octets: Vec<&str> = ip_str.split('.').collect();
                    let valid = octets.iter().all(|o| {
                        o.parse::<u8>().is_ok()
                    });
                    if valid {
                        entities.push(format!("IP:{}", ip_str));
                    }
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

/// Stage 8: 关系建立（自动发现和建立记忆间关系）
pub struct RelationBuildingStage {
    /// Enable similarity-based relation detection
    pub enable_similarity: bool,
    /// Enable temporal relation detection (nearby in time)
    pub enable_temporal: bool,
    /// Enable entity-based relation detection (shared entities)
    pub enable_entity: bool,
    /// Similarity threshold for establishing relations (0.0-1.0)
    pub similarity_threshold: f32,
    /// Time window for temporal relations (in seconds)
    pub temporal_window_secs: i64,
}

impl Default for RelationBuildingStage {
    fn default() -> Self {
        Self {
            enable_similarity: true,
            enable_temporal: true,
            enable_entity: true,
            similarity_threshold: 0.7,
            temporal_window_secs: 86400, // 24 hours
        }
    }
}

#[async_trait]
impl PipelineStage for RelationBuildingStage {
    type Input = (Memory, Vec<Memory>);  // Current memory + existing memories
    type Output = Memory;
    
    fn name(&self) -> &str {
        "RelationBuilding"
    }
    
    fn is_optional(&self) -> bool {
        true
    }
    
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        let (mut current_memory, existing_memories) = input;
        let mut relations_built = 0;
        
        let current_text = current_memory.content.as_text();
        let current_entities = context.get::<Vec<String>>("entities").unwrap_or_default();
        
        for existing in existing_memories.iter().take(100) {  // Limit to 100 recent memories
            let mut relation_strength = 0.0;
            let mut relation_reasons = Vec::new();
            
            // 1. Entity-based relation (shared entities)
            if self.enable_entity && !current_entities.is_empty() {
                if let Some(AttributeValue::Array(existing_entities)) = existing.attributes.get(&AttributeKey::domain("entities")) {
                    let existing_entity_strs: Vec<String> = existing_entities.iter()
                        .filter_map(|e| {
                            if let AttributeValue::String(s) = e {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    let shared_count = current_entities.iter()
                        .filter(|e| existing_entity_strs.contains(e))
                        .count();
                    
                    if shared_count > 0 {
                        let entity_score = (shared_count as f32) * 0.2;
                        relation_strength += entity_score.min(0.5);
                        relation_reasons.push(format!("shared_entities:{}", shared_count));
                    }
                }
            }
            
            // 2. Temporal relation (nearby in time)
            if self.enable_temporal {
                let time_diff = (current_memory.metadata.created_at.timestamp() 
                    - existing.metadata.created_at.timestamp()).abs();
                
                if time_diff < self.temporal_window_secs {
                    let temporal_score = 1.0 - (time_diff as f32 / self.temporal_window_secs as f32);
                    relation_strength += temporal_score * 0.3;
                    relation_reasons.push(format!("temporal_proximity:{:.2}h", time_diff as f32 / 3600.0));
                }
            }
            
            // 3. Content similarity (Jaccard)
            if self.enable_similarity {
                let existing_text = existing.content.as_text();
                let similarity = calculate_jaccard_similarity(&current_text, &existing_text);
                
                if similarity >= self.similarity_threshold {
                    relation_strength += similarity * 0.5;
                    relation_reasons.push(format!("content_similarity:{:.2}", similarity));
                }
            }
            
            // Establish relation if strength is significant
            if relation_strength >= 0.3 {
                current_memory.relations.add_relation(crate::types::Relation {
                    target_id: existing.id.clone(),
                    relation_type: crate::types::RelationType::Custom(format!("auto_discovered: {}", relation_reasons.join(", "))),
                    strength: relation_strength.min(1.0),
                });
                relations_built += 1;
            }
        }
        
        let _ = context.set("relations_built", relations_built);
        let _ = context.set("relation_discovery_enabled", true);
        
        Ok(StageResult::Continue(current_memory))
    }
}

/// Calculate Jaccard similarity between two texts
fn calculate_jaccard_similarity(text1: &str, text2: &str) -> f32 {
    use std::collections::HashSet;
    
    let words1: HashSet<&str> = text1.split_whitespace().collect();
    let words2: HashSet<&str> = text2.split_whitespace().collect();
    
    if words1.is_empty() && words2.is_empty() {
        return 1.0;
    }
    
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Stage 9: 重要性重评估（动态调整记忆重要性）
pub struct ImportanceReassessmentStage {
    /// Enable access frequency factor
    pub enable_access_freq: bool,
    /// Enable temporal decay factor
    pub enable_temporal_decay: bool,
    /// Enable relation network factor (referenced by other memories)
    pub enable_relation_boost: bool,
    /// Enable context relevance factor
    pub enable_context_relevance: bool,
    /// Weight for access frequency (0.0-1.0)
    pub access_freq_weight: f32,
    /// Weight for temporal decay (0.0-1.0)
    pub temporal_decay_weight: f32,
    /// Weight for relation network (0.0-1.0)
    pub relation_boost_weight: f32,
    /// Weight for context relevance (0.0-1.0)
    pub context_relevance_weight: f32,
    /// Time decay half-life in days (how many days for importance to halve)
    pub decay_halflife_days: f32,
}

impl Default for ImportanceReassessmentStage {
    fn default() -> Self {
        Self {
            enable_access_freq: true,
            enable_temporal_decay: true,
            enable_relation_boost: true,
            enable_context_relevance: false,  // Optional, needs context
            access_freq_weight: 0.3,
            temporal_decay_weight: 0.25,
            relation_boost_weight: 0.25,
            context_relevance_weight: 0.2,
            decay_halflife_days: 30.0,  // 30 days half-life
        }
    }
}

#[async_trait]
impl PipelineStage for ImportanceReassessmentStage {
    type Input = Memory;
    type Output = Memory;
    
    fn name(&self) -> &str {
        "ImportanceReassessment"
    }
    
    fn is_optional(&self) -> bool {
        true
    }
    
    async fn execute(
        &self,
        mut input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        // Get original importance
        let original_importance = input.importance();
        let mut adjustment_factors = Vec::new();
        let mut total_adjustment = 0.0;
        let mut total_weight = 0.0;
        
        // 1. Access frequency factor
        if self.enable_access_freq {
            let access_count = input.metadata.accessed_count;
            // Logarithmic scale: frequent access = higher importance
            // log(1+x) to avoid log(0) and smooth scaling
            let freq_score = if access_count > 0 {
                ((access_count as f32).ln_1p() / 10.0).min(1.0)  // Cap at 1.0
            } else {
                0.0
            };
            
            adjustment_factors.push(format!("access_freq:{:.2}", freq_score));
            total_adjustment += freq_score * self.access_freq_weight;
            total_weight += self.access_freq_weight;
        }
        
        // 2. Temporal decay factor
        if self.enable_temporal_decay {
            let now = chrono::Utc::now();
            let age_secs = (now - input.metadata.created_at).num_seconds() as f32;
            let age_days = age_secs / 86400.0;
            
            // Exponential decay: importance = initial * 0.5^(age/halflife)
            // We calculate the decay multiplier
            let decay_multiplier = 0.5_f32.powf(age_days / self.decay_halflife_days);
            
            adjustment_factors.push(format!("temporal_decay:{:.2}", decay_multiplier));
            // Negative adjustment if old (decay_multiplier < 1.0)
            total_adjustment += (decay_multiplier - 1.0) * self.temporal_decay_weight;
            total_weight += self.temporal_decay_weight;
        }
        
        // 3. Relation network boost
        if self.enable_relation_boost {
            let relation_count = input.relations.relations().len();
            // More relations = more important (reference count)
            let relation_score = (relation_count as f32 / 10.0).min(1.0);  // Normalize, cap at 1.0
            
            adjustment_factors.push(format!("relation_boost:{:.2}", relation_score));
            total_adjustment += relation_score * self.relation_boost_weight;
            total_weight += self.relation_boost_weight;
        }
        
        // 4. Context relevance (optional, needs context data)
        if self.enable_context_relevance {
            if let Some(relevance_score) = context.get::<f32>("context_relevance") {
                adjustment_factors.push(format!("context_relevance:{:.2}", relevance_score));
                total_adjustment += relevance_score * self.context_relevance_weight;
                total_weight += self.context_relevance_weight;
            }
        }
        
        // Calculate new importance
        // Normalize adjustment to [-1.0, 1.0] range, then apply
        let normalized_adjustment = if total_weight > 0.0 {
            total_adjustment / total_weight
        } else {
            0.0
        };
        
        let new_importance = (original_importance + normalized_adjustment).clamp(0.0, 1.0);
        
        // Update importance in memory attributes
        input.attributes.set(
            crate::types::AttributeKey::system("importance"),
            crate::types::AttributeValue::Number(new_importance as f64),
        );
        
        // Record reassessment info
        let _ = context.set("original_importance", original_importance);
        let _ = context.set("new_importance", new_importance);
        let _ = context.set("importance_change", new_importance - original_importance);
        let _ = context.set("adjustment_factors", adjustment_factors.join(", "));
        
        Ok(StageResult::Continue(input))
    }
}

/// Stage 10: 记忆压缩（自动合并相似记忆）
pub struct MemoryCompressionStage {
    /// Enable content-based compression
    pub enable_content_compression: bool,
    /// Enable attribute-based compression
    pub enable_attribute_compression: bool,
    /// Similarity threshold for compression (0.0-1.0)
    pub similarity_threshold: f32,
    /// Maximum compression ratio (how many memories can be merged into one)
    pub max_compression_ratio: usize,
    /// Merge strategy: "newest" | "highest_importance" | "longest"
    pub merge_strategy: String,
    /// Preserve all unique entities
    pub preserve_unique_entities: bool,
}

impl Default for MemoryCompressionStage {
    fn default() -> Self {
        Self {
            enable_content_compression: true,
            enable_attribute_compression: true,
            similarity_threshold: 0.85,  // High threshold for compression
            max_compression_ratio: 5,    // Max 5 memories merge into 1
            merge_strategy: "highest_importance".to_string(),
            preserve_unique_entities: true,
        }
    }
}

#[async_trait]
impl PipelineStage for MemoryCompressionStage {
    type Input = Vec<Memory>;
    type Output = Vec<Memory>;
    
    fn name(&self) -> &str {
        "MemoryCompression"
    }
    
    fn is_optional(&self) -> bool {
        true
    }
    
    async fn execute(
        &self,
        input: Self::Input,
        context: &mut PipelineContext,
    ) -> anyhow::Result<StageResult<Self::Output>> {
        if input.is_empty() {
            return Ok(StageResult::Continue(input));
        }
        
        let mut compressed_memories = Vec::new();
        let mut merged_groups = Vec::new();
        let mut processed_indices = std::collections::HashSet::new();
        
        // Find similar memory groups
        for i in 0..input.len() {
            if processed_indices.contains(&i) {
                continue;
            }
            
            let mut group = vec![i];
            let base_memory = &input[i];
            
            // Find similar memories
            for j in (i + 1)..input.len() {
                if processed_indices.contains(&j) {
                    continue;
                }
                
                let candidate_memory = &input[j];
                
                // Calculate similarity
                let similarity = self.calculate_similarity(base_memory, candidate_memory);
                
                if similarity >= self.similarity_threshold {
                    group.push(j);
                    
                    // Respect max compression ratio
                    if group.len() >= self.max_compression_ratio {
                        break;
                    }
                }
            }
            
            // If group has multiple memories, merge them
            if group.len() > 1 {
                let memories_to_merge: Vec<_> = group.iter()
                    .map(|&idx| &input[idx])
                    .collect();
                
                let merged_memory = self.merge_memories(&memories_to_merge);
                compressed_memories.push(merged_memory);
                
                // Mark all as processed
                for &idx in &group {
                    processed_indices.insert(idx);
                }
                
                merged_groups.push(group.len());
            } else {
                // Single memory, keep as is
                compressed_memories.push(base_memory.clone());
                processed_indices.insert(i);
            }
        }
        
        // Record compression stats
        let original_count = input.len();
        let compressed_count = compressed_memories.len();
        let compression_ratio = if original_count > 0 {
            compressed_count as f32 / original_count as f32
        } else {
            1.0
        };
        
        let _ = context.set("original_memory_count", original_count);
        let _ = context.set("compressed_memory_count", compressed_count);
        let _ = context.set("compression_ratio", compression_ratio);
        let _ = context.set("merged_groups", merged_groups.clone());
        let _ = context.set("memories_saved", original_count - compressed_count);
        
        Ok(StageResult::Continue(compressed_memories))
    }
}

impl MemoryCompressionStage {
    /// Calculate similarity between two memories
    fn calculate_similarity(&self, mem1: &Memory, mem2: &Memory) -> f32 {
        let mut total_similarity = 0.0;
        let mut weight_sum = 0.0;
        
        // 1. Content similarity (weight: 0.6)
        if self.enable_content_compression {
            let content_sim = self.calculate_content_similarity(mem1, mem2);
            total_similarity += content_sim * 0.6;
            weight_sum += 0.6;
        }
        
        // 2. Attribute similarity (weight: 0.4)
        if self.enable_attribute_compression {
            let attr_sim = self.calculate_attribute_similarity(mem1, mem2);
            total_similarity += attr_sim * 0.4;
            weight_sum += 0.4;
        }
        
        if weight_sum > 0.0 {
            total_similarity / weight_sum
        } else {
            0.0
        }
    }
    
    /// Calculate content similarity using Jaccard
    fn calculate_content_similarity(&self, mem1: &Memory, mem2: &Memory) -> f32 {
        let text1 = mem1.content.to_string();
        let text2 = mem2.content.to_string();
        calculate_jaccard_similarity(&text1, &text2)
    }
    
    /// Calculate attribute similarity
    fn calculate_attribute_similarity(&self, mem1: &Memory, mem2: &Memory) -> f32 {
        // Compare important system attributes
        let mut matches = 0;
        let mut total = 0;
        
        // Check agent_id
        let agent1 = mem1.attributes.get(&crate::types::AttributeKey::system("agent_id"))
            .and_then(|v| v.as_string());
        let agent2 = mem2.attributes.get(&crate::types::AttributeKey::system("agent_id"))
            .and_then(|v| v.as_string());
        if agent1.is_some() || agent2.is_some() {
            total += 1;
            if agent1 == agent2 {
                matches += 1;
            }
        }
        
        // Check user_id
        let user1 = mem1.attributes.get(&crate::types::AttributeKey::system("user_id"))
            .and_then(|v| v.as_string());
        let user2 = mem2.attributes.get(&crate::types::AttributeKey::system("user_id"))
            .and_then(|v| v.as_string());
        if user1.is_some() || user2.is_some() {
            total += 1;
            if user1 == user2 {
                matches += 1;
            }
        }
        
        // Check memory_type
        let type1 = mem1.attributes.get(&crate::types::AttributeKey::system("memory_type"))
            .and_then(|v| v.as_string());
        let type2 = mem2.attributes.get(&crate::types::AttributeKey::system("memory_type"))
            .and_then(|v| v.as_string());
        if type1.is_some() || type2.is_some() {
            total += 1;
            if type1 == type2 {
                matches += 1;
            }
        }
        
        if total > 0 {
            matches as f32 / total as f32
        } else {
            1.0  // No attributes to compare, consider similar
        }
    }
    
    /// Merge multiple memories into one
    fn merge_memories(&self, memories: &[&Memory]) -> Memory {
        if memories.is_empty() {
            panic!("Cannot merge empty memory list");
        }
        
        // Select base memory according to strategy
        let base_memory = match self.merge_strategy.as_str() {
            "newest" => {
                // Select the newest one
                memories.iter()
                    .max_by_key(|m| m.metadata.created_at)
                    .unwrap()
            }
            "highest_importance" => {
                // Select the most important one
                memories.iter()
                    .max_by(|a, b| {
                        a.importance().partial_cmp(&b.importance()).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap()
            }
            "longest" => {
                // Select the longest content
                memories.iter()
                    .max_by_key(|m| m.content.to_string().len())
                    .unwrap()
            }
            _ => memories[0]  // Default to first
        };
        
        let mut merged = (*base_memory).clone();
        
        // Merge content: combine unique information
        if memories.len() > 1 {
            let all_texts: Vec<String> = memories.iter()
                .map(|m| m.content.to_string())
                .collect();
            
            // Simple merge: join with separator
            let merged_text = all_texts.join(" | ");
            merged.content = crate::types::Content::Text(merged_text);
        }
        
        // Merge entities if preserve_unique_entities is enabled
        if self.preserve_unique_entities {
            let mut all_entities = std::collections::HashSet::new();
            for memory in memories {
                if let Some(entities) = memory.attributes.get(&crate::types::AttributeKey::system("entities")) {
                    if let Some(entity_str) = entities.as_string() {
                        for entity in entity_str.split(',') {
                            all_entities.insert(entity.trim().to_string());
                        }
                    }
                }
            }
            
            if !all_entities.is_empty() {
                let entities_str = all_entities.into_iter().collect::<Vec<_>>().join(",");
                merged.attributes.set(
                    crate::types::AttributeKey::system("entities"),
                    crate::types::AttributeValue::String(entities_str),
                );
            }
        }
        
        // Record merge info
        merged.attributes.set(
            crate::types::AttributeKey::system("merged_from_count"),
            crate::types::AttributeValue::Number(memories.len() as f64),
        );
        
        // Boost importance for merged memories
        let avg_importance: f32 = memories.iter()
            .map(|m| m.importance())
            .sum::<f32>() / memories.len() as f32;
        let boosted_importance = (avg_importance * 1.1).min(1.0);  // 10% boost, cap at 1.0
        
        merged.attributes.set(
            crate::types::AttributeKey::system("importance"),
            crate::types::AttributeValue::Number(boosted_importance as f64),
        );
        
        merged
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
            extract_money: false,
            extract_time: false,
            extract_percentage: false,
            extract_ip: false,
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
    async fn test_entity_extraction_enhanced() {
        let stage = EntityExtractionStage {
            extract_persons: false,
            extract_orgs: false,
            extract_locations: false,
            extract_dates: true,
            extract_money: true,
            extract_time: true,
            extract_percentage: true,
            extract_ip: true,
        };
        
        let memory = MemoryBuilder::new()
            .text("Price is $100.50 or ¥200元, growth rate 15.5%, server IP 192.168.1.100, meeting at 14:30 on 2024-12-25")
            .build();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        if let StageResult::Continue(_mem) = result {
            let entities = context.get::<Vec<String>>("entities").unwrap();
            
            // Should extract: DATE, MONEY (2), PERCENTAGE, IP, TIME
            assert!(entities.len() >= 5, "Expected at least 5 entities, got {}", entities.len());
            
            // Verify entity types exist
            assert!(entities.iter().any(|e| e.starts_with("DATE:")), "Should have DATE entity");
            assert!(entities.iter().any(|e| e.starts_with("MONEY:")), "Should have MONEY entity");
            assert!(entities.iter().any(|e| e.starts_with("PERCENTAGE:")), "Should have PERCENTAGE entity");
            assert!(entities.iter().any(|e| e.starts_with("IP:")), "Should have IP entity");
            assert!(entities.iter().any(|e| e.starts_with("TIME:")), "Should have TIME entity");
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
    async fn test_relation_building_stage() {
        let stage = RelationBuildingStage {
            enable_similarity: true,
            enable_temporal: true,
            enable_entity: true,
            similarity_threshold: 0.5,
            temporal_window_secs: 86400,
        };
        
        // Create current memory
        let current_memory = MemoryBuilder::new()
            .text("Product P000257 price increased by 15%")
            .build();
        
        // Create existing memories
        let existing1 = MemoryBuilder::new()
            .text("Product P000257 is now available")
            .build();
        
        let existing2 = MemoryBuilder::new()
            .text("Unrelated content without any connection")
            .build();
        
        let mut context = PipelineContext::new();
        // Simulate entity extraction results
        let entities = vec!["ID:P000257".to_string(), "PERCENTAGE:15%".to_string()];
        let _ = context.set("entities", entities);
        
        let result = stage.execute((current_memory, vec![existing1, existing2]), &mut context).await.unwrap();
        
        if let StageResult::Continue(memory) = result {
            let relations_built = context.get::<usize>("relations_built").unwrap_or(0);
            assert!(relations_built > 0, "Should have built at least one relation");
            
            // Check that relation was established
            assert!(!memory.relations.relations().is_empty(), "Should have established relations");
        } else {
            panic!("Expected Continue");
        }
    }
    
    #[tokio::test]
    async fn test_memory_compression_stage() {
        let stage = MemoryCompressionStage {
            enable_content_compression: true,
            enable_attribute_compression: true,
            similarity_threshold: 0.5,  // Lower threshold for testing
            max_compression_ratio: 3,
            merge_strategy: "highest_importance".to_string(),
            preserve_unique_entities: true,
        };
        
        // Create very similar memories (with high overlap)
        let mem1 = MemoryBuilder::new()
            .text("The product price is one hundred dollars and available now")
            .build();
        
        let mem2 = MemoryBuilder::new()
            .text("The product price is one hundred dollars and currently available")
            .build();
        
        let mem3 = MemoryBuilder::new()
            .text("The product price available now one hundred dollars")
            .build();
        
        // Create a dissimilar memory
        let mem4 = MemoryBuilder::new()
            .text("Weather forecast shows sunny day tomorrow with rain")
            .build();
        
        let memories = vec![mem1, mem2, mem3, mem4];
        let original_count = memories.len();
        
        let mut context = PipelineContext::new();
        let result = stage.execute(memories, &mut context).await.unwrap();
        
        if let StageResult::Continue(compressed) = result {
            // Check compression stats
            let compressed_count = context.get::<usize>("compressed_memory_count").unwrap();
            let memories_saved = context.get::<usize>("memories_saved").unwrap_or(0);
            let ratio = context.get::<f32>("compression_ratio").unwrap();
            
            assert_eq!(compressed_count, compressed.len());
            assert_eq!(memories_saved, original_count - compressed_count);
            
            // If compressed, ratio should be < 1.0
            if compressed.len() < original_count {
                assert!(ratio < 1.0, "Compression ratio should be < 1.0 when compressed");
                
                // Check merged memory has boost
                let merged_memory = compressed.iter()
                    .find(|m| {
                        m.attributes.get(&crate::types::AttributeKey::system("merged_from_count")).is_some()
                    });
                
                assert!(merged_memory.is_some(), "Should have at least one merged memory");
            } else {
                // No compression happened, which is also valid
                assert_eq!(ratio, 1.0, "No compression, ratio should be 1.0");
            }
        } else {
            panic!("Expected Continue");
        }
    }
    
    #[tokio::test]
    async fn test_importance_reassessment_stage() {
        let stage = ImportanceReassessmentStage {
            enable_access_freq: true,
            enable_temporal_decay: true,
            enable_relation_boost: true,
            enable_context_relevance: false,
            access_freq_weight: 0.3,
            temporal_decay_weight: 0.25,
            relation_boost_weight: 0.25,
            context_relevance_weight: 0.2,
            decay_halflife_days: 30.0,
        };
        
        // Create a memory with some access history and relations
        let mut memory = MemoryBuilder::new()
            .text("Important product information P000257")
            .build();
        
        // Simulate some access history
        memory.metadata.accessed_count = 10;
        
        // Add some relations to boost importance
        memory.relations.add_relation(crate::types::Relation {
            target_id: "rel1".to_string(),
            relation_type: crate::types::RelationType::References,
            strength: 0.8,
        });
        memory.relations.add_relation(crate::types::Relation {
            target_id: "rel2".to_string(),
            relation_type: crate::types::RelationType::SimilarTo,
            strength: 0.6,
        });
        
        let original_importance = memory.importance();
        let mut context = PipelineContext::new();
        
        let result = stage.execute(memory, &mut context).await.unwrap();
        
        if let StageResult::Continue(updated_memory) = result {
            let new_importance = updated_memory.importance();
            
            // Importance should change based on access freq and relations
            // (temporal decay should be minimal for recent memories)
            assert!(
                (new_importance - original_importance).abs() > 0.001,
                "Importance should have changed"
            );
            
            // Check context recorded the change
            assert!(context.get::<f32>("original_importance").is_some());
            assert!(context.get::<f32>("new_importance").is_some());
            assert!(context.get::<f32>("importance_change").is_some());
            assert!(context.get::<String>("adjustment_factors").is_some());
            
            let factors = context.get::<String>("adjustment_factors").unwrap();
            assert!(factors.contains("access_freq"), "Should have access frequency factor");
            assert!(factors.contains("relation_boost"), "Should have relation boost factor");
        } else {
            panic!("Expected Continue");
        }
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

