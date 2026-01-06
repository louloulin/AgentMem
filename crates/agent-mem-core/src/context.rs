//! Context analysis and management
//!
//! 提供上下文分析功能，用于理解查询的上下文环境，
//! 提取关键信息，并为检索提供更好的上下文理解。

use agent_mem_traits::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上下文分析器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalyzerConfig {
    /// 是否启用时间上下文分析
    pub enable_temporal_analysis: bool,
    /// 是否启用实体识别
    pub enable_entity_recognition: bool,
    /// 是否启用情感分析
    pub enable_sentiment_analysis: bool,
    /// 最大上下文窗口大小
    pub max_context_window: usize,
    /// 相关性阈值
    pub relevance_threshold: f64,
}

impl Default for ContextAnalyzerConfig {
    fn default() -> Self {
        Self {
            enable_temporal_analysis: true,
            enable_entity_recognition: true,
            enable_sentiment_analysis: false,
            max_context_window: 10,
            relevance_threshold: 0.5,
        }
    }
}

/// 上下文分析器
pub struct ContextAnalyzer {
    /// 配置
    config: ContextAnalyzerConfig,
    /// 历史上下文缓存
    context_cache: HashMap<String, Vec<MemoryContext>>,
}

impl ContextAnalyzer {
    /// 创建新的上下文分析器
    pub fn new(config: ContextAnalyzerConfig) -> Self {
        Self {
            config,
            context_cache: HashMap::new(),
        }
    }

    /// 分析查询上下文
    pub fn analyze_query_context(&self, query: &str) -> Result<AnalyzedContext> {
        let mut context = AnalyzedContext {
            query: query.to_string(),
            entities: Vec::new(),
            temporal_info: None,
            sentiment: None,
            keywords: Vec::new(),
            context_type: ContextType::General,
            confidence: 0.0,
        };

        // 1. 提取关键词
        context.keywords = self.extract_keywords(query);

        // 2. 识别实体
        if self.config.enable_entity_recognition {
            context.entities = self.recognize_entities(query);
        }

        // 3. 分析时间信息
        if self.config.enable_temporal_analysis {
            context.temporal_info = self.analyze_temporal_info(query);
        }

        // 4. 情感分析
        if self.config.enable_sentiment_analysis {
            context.sentiment = self.analyze_sentiment(query);
        }

        // 5. 确定上下文类型
        context.context_type = self.determine_context_type(&context);

        // 6. 计算置信度
        context.confidence = self.calculate_confidence(&context);

        Ok(context)
    }

    /// 提取关键词
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        // 简单的关键词提取：分词并过滤停用词
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "will", "would", "should", "could", "may", "might", "can", "what",
            "when", "where", "who", "why", "how",
        ];

        query
            .to_lowercase()
            .split_whitespace()
            .filter(|word| {
                word.len() > 2
                    && !stop_words.contains(&word.trim_matches(|c: char| !c.is_alphanumeric()))
            })
            .map(|s| s.to_string())
            .collect()
    }

    /// 识别实体
    fn recognize_entities(&self, query: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        // 简单的实体识别：查找大写开头的词组
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut i = 0;
        while i < words.len() {
            if words[i].chars().next().is_some_and(|c| c.is_uppercase()) {
                let mut entity_text = words[i].to_string();
                let _start = i;
                i += 1;

                // 继续收集连续的大写词
                while i < words.len() && words[i].chars().next().is_some_and(|c| c.is_uppercase()) {
                    entity_text.push(' ');
                    entity_text.push_str(words[i]);
                    i += 1;
                }

                entities.push(Entity {
                    text: entity_text,
                    entity_type: EntityType::Unknown,
                    confidence: 0.7,
                });
            } else {
                i += 1;
            }
        }

        entities
    }

    /// 分析时间信息
    fn analyze_temporal_info(&self, query: &str) -> Option<TemporalInfo> {
        let query_lower = query.to_lowercase();

        // 检测时间相关关键词
        let temporal_keywords = vec![
            ("today", TemporalScope::Today),
            ("yesterday", TemporalScope::Yesterday),
            ("last week", TemporalScope::LastWeek),
            ("last month", TemporalScope::LastMonth),
            ("this week", TemporalScope::ThisWeek),
            ("this month", TemporalScope::ThisMonth),
            ("recent", TemporalScope::Recent),
        ];

        for (keyword, scope) in temporal_keywords {
            if query_lower.contains(keyword) {
                return Some(TemporalInfo {
                    scope,
                    reference_time: chrono::Utc::now(),
                    confidence: 0.8,
                });
            }
        }

        None
    }

    /// 分析情感
    fn analyze_sentiment(&self, _query: &str) -> Option<Sentiment> {
        // 简单的情感分析（可以后续集成更复杂的模型）
        Some(Sentiment {
            polarity: 0.0, // -1.0 (负面) 到 1.0 (正面)
            confidence: 0.5,
        })
    }

    /// 确定上下文类型
    fn determine_context_type(&self, context: &AnalyzedContext) -> ContextType {
        // 基于关键词和实体判断上下文类型
        if context.temporal_info.is_some() {
            return ContextType::Temporal;
        }

        if !context.entities.is_empty() {
            return ContextType::EntityFocused;
        }

        if context.keywords.iter().any(|k| {
            k.contains("how") || k.contains("what") || k.contains("why") || k.contains("when")
        }) {
            return ContextType::Question;
        }

        ContextType::General
    }

    /// 计算置信度
    fn calculate_confidence(&self, context: &AnalyzedContext) -> f64 {
        let mut confidence = 0.5;

        // 有关键词增加置信度
        if !context.keywords.is_empty() {
            confidence += 0.1 * (context.keywords.len() as f64).min(3.0) / 3.0;
        }

        // 有实体增加置信度
        if !context.entities.is_empty() {
            confidence += 0.2;
        }

        // 有时间信息增加置信度
        if context.temporal_info.is_some() {
            confidence += 0.2;
        }

        confidence.min(1.0)
    }

    /// 合并多个上下文
    pub fn merge_contexts(&self, contexts: Vec<MemoryContext>) -> Result<MemoryContext> {
        if contexts.is_empty() {
            return Ok(MemoryContext {
                context_type: "empty".to_string(),
                data: HashMap::new(),
                relevance: 0.0,
            });
        }

        let mut merged_data = HashMap::new();
        let mut total_relevance = 0.0;

        for context in &contexts {
            for (key, value) in &context.data {
                merged_data.insert(key.clone(), value.clone());
            }
            total_relevance += context.relevance;
        }

        Ok(MemoryContext {
            context_type: "merged".to_string(),
            data: merged_data,
            relevance: total_relevance / contexts.len() as f64,
        })
    }
}

impl Default for ContextAnalyzer {
    fn default() -> Self {
        Self::new(ContextAnalyzerConfig::default())
    }
}

/// 分析后的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedContext {
    /// 原始查询
    pub query: String,
    /// 识别的实体
    pub entities: Vec<Entity>,
    /// 时间信息
    pub temporal_info: Option<TemporalInfo>,
    /// 情感信息
    pub sentiment: Option<Sentiment>,
    /// 关键词
    pub keywords: Vec<String>,
    /// 上下文类型
    pub context_type: ContextType,
    /// 置信度
    pub confidence: f64,
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体文本
    pub text: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 置信度
    pub confidence: f64,
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    /// 人名
    Person,
    /// 组织
    Organization,
    /// 地点
    Location,
    /// 日期
    Date,
    /// 未知
    Unknown,
}

/// 时间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalInfo {
    /// 时间范围
    pub scope: TemporalScope,
    /// 参考时间
    pub reference_time: chrono::DateTime<chrono::Utc>,
    /// 置信度
    pub confidence: f64,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemporalScope {
    /// 今天
    Today,
    /// 昨天
    Yesterday,
    /// 本周
    ThisWeek,
    /// 上周
    LastWeek,
    /// 本月
    ThisMonth,
    /// 上月
    LastMonth,
    /// 最近
    Recent,
}

/// 情感
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    /// 极性 (-1.0 到 1.0)
    pub polarity: f64,
    /// 置信度
    pub confidence: f64,
}

/// 上下文类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    /// 一般
    General,
    /// 问题
    Question,
    /// 时间相关
    Temporal,
    /// 实体相关
    EntityFocused,
}

/// 记忆上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    /// 上下文类型
    pub context_type: String,
    /// 上下文数据
    pub data: HashMap<String, String>,
    /// 相关性分数
    pub relevance: f64,
}
