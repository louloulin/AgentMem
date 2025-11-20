//! Query Classifier - 智能查询分类器
//!
//! 根据查询特征自动分类，为不同类型的查询选择最优检索策略

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    /// 精确ID查询 (如: P000001, SKU-123)
    ExactId,
    /// 短关键词查询 (如: Apple, 手机)
    ShortKeyword,
    /// 自然语言查询 (如: "推荐一款手机")
    NaturalLanguage,
    /// 语义查询/问题 (如: "What is...?", "How to...?")
    Semantic,
    /// 时间范围查询 (如: "last week", "2024-01-01")
    Temporal,
}

/// 检索策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStrategy {
    /// 是否使用向量搜索
    pub use_vector: bool,
    /// 是否使用BM25搜索
    pub use_bm25: bool,
    /// 是否使用精确匹配
    pub use_exact_match: bool,

    /// 向量搜索权重 (0.0 - 1.0)
    pub vector_weight: f32,
    /// BM25搜索权重 (0.0 - 1.0)
    pub bm25_weight: f32,

    /// 相似度阈值
    pub threshold: f32,

    /// 是否需要重排序
    pub need_rerank: bool,
}

impl Default for SearchStrategy {
    fn default() -> Self {
        Self {
            use_vector: true,
            use_bm25: true,
            use_exact_match: false,
            vector_weight: 0.7,
            bm25_weight: 0.3,
            threshold: 0.3,
            need_rerank: false,
        }
    }
}

/// 查询特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFeatures {
    /// 查询长度
    pub length: usize,
    /// 词数
    pub word_count: usize,
    /// 是否包含特殊字符
    pub has_special_chars: bool,
    /// 是否是问题（包含?或疑问词）
    pub is_question: bool,
    /// 是否包含数字
    pub has_numbers: bool,
    /// 是否全大写
    pub is_uppercase: bool,
    /// 语言（简单判断）
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    Chinese,
    Mixed,
    Unknown,
}

/// 查询分类器配置
#[derive(Debug, Clone)]
pub struct QueryClassifierConfig {
    /// 短查询最大长度
    pub max_short_query_length: usize,
    /// 语义查询最小长度
    pub min_semantic_length: usize,
    /// 是否启用正则匹配
    pub enable_regex: bool,
}

impl Default for QueryClassifierConfig {
    fn default() -> Self {
        Self {
            max_short_query_length: 5, // 降低阈值，单个词或短语才算ShortKeyword
            min_semantic_length: 30,
            enable_regex: true,
        }
    }
}

/// 查询分类器
pub struct QueryClassifier {
    config: QueryClassifierConfig,

    // 正则模式
    exact_id_pattern: Option<Regex>,
    sku_pattern: Option<Regex>,
    date_pattern: Option<Regex>,
    uuid_pattern: Option<Regex>,

    // 问题词
    question_words: Vec<String>,
}

impl QueryClassifier {
    /// 创建新的查询分类器
    pub fn new(config: QueryClassifierConfig) -> Self {
        let exact_id_pattern = if config.enable_regex {
            Regex::new(r"^P\d{6}$").ok()
        } else {
            None
        };

        let sku_pattern = if config.enable_regex {
            Regex::new(r"^[A-Z]+-\d+$").ok()
        } else {
            None
        };

        let date_pattern = if config.enable_regex {
            Regex::new(r"\d{4}-\d{2}-\d{2}").ok()
        } else {
            None
        };

        let uuid_pattern = if config.enable_regex {
            Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").ok()
        } else {
            None
        };

        let question_words = vec![
            "what",
            "why",
            "how",
            "when",
            "where",
            "who",
            "which",
            "什么",
            "为什么",
            "怎么",
            "如何",
            "哪里",
            "谁",
            "哪个",
        ]
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();

        Self {
            config,
            exact_id_pattern,
            sku_pattern,
            date_pattern,
            uuid_pattern,
            question_words,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(QueryClassifierConfig::default())
    }

    /// 提取查询特征
    pub fn extract_features(&self, query: &str) -> QueryFeatures {
        let trimmed = query.trim();
        let length = trimmed.len();
        let word_count = trimmed.split_whitespace().count();

        let has_special_chars = trimmed
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace() && c != '-' && c != '_');

        let is_question = trimmed.contains('?')
            || trimmed.contains('？')
            || self
                .question_words
                .iter()
                .any(|word| trimmed.to_lowercase().starts_with(word));

        let has_numbers = trimmed.chars().any(|c| c.is_numeric());
        let is_uppercase = trimmed
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase());

        let language = self.detect_language(trimmed);

        QueryFeatures {
            length,
            word_count,
            has_special_chars,
            is_question,
            has_numbers,
            is_uppercase,
            language,
        }
    }

    /// 检测语言
    fn detect_language(&self, query: &str) -> Language {
        let has_chinese = query
            .chars()
            .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));

        let has_english = query
            .chars()
            .any(|c| ('a'..='z').contains(&c) || ('A'..='Z').contains(&c));

        match (has_chinese, has_english) {
            (true, true) => Language::Mixed,
            (true, false) => Language::Chinese,
            (false, true) => Language::English,
            (false, false) => Language::Unknown,
        }
    }

    /// 分类查询
    pub fn classify(&self, query: &str) -> QueryType {
        let trimmed = query.trim();

        // 1. 检测精确ID模式
        if let Some(pattern) = &self.exact_id_pattern {
            if pattern.is_match(trimmed) {
                return QueryType::ExactId;
            }
        }

        if let Some(pattern) = &self.sku_pattern {
            if pattern.is_match(trimmed) {
                return QueryType::ExactId;
            }
        }

        if let Some(pattern) = &self.uuid_pattern {
            if pattern.is_match(trimmed) {
                return QueryType::ExactId;
            }
        }

        // 2. 检测时间查询
        if let Some(pattern) = &self.date_pattern {
            if pattern.is_match(trimmed) {
                return QueryType::Temporal;
            }
        }

        let features = self.extract_features(trimmed);

        // 3. 检测问题型查询
        if features.is_question {
            return QueryType::Semantic;
        }

        // 4. 根据长度分类
        // 对于中文，使用字符数而不是字节数
        let effective_length =
            if features.language == Language::Chinese || features.language == Language::Mixed {
                trimmed.chars().count()
            } else {
                features.length
            };

        if effective_length <= self.config.max_short_query_length {
            return QueryType::ShortKeyword;
        }

        if effective_length >= self.config.min_semantic_length {
            return QueryType::Semantic;
        }

        // 5. 默认为自然语言查询
        QueryType::NaturalLanguage
    }

    /// 获取检索策略
    pub fn get_strategy(&self, query_type: &QueryType) -> SearchStrategy {
        match query_type {
            QueryType::ExactId => SearchStrategy {
                use_vector: false,
                use_bm25: true,
                use_exact_match: true,
                vector_weight: 0.0,
                bm25_weight: 1.0,
                threshold: 0.0,
                need_rerank: false,
            },

            QueryType::ShortKeyword => SearchStrategy {
                use_vector: true,
                use_bm25: true,
                use_exact_match: false,
                vector_weight: 0.5,
                bm25_weight: 0.5,
                threshold: 0.1,
                need_rerank: true,
            },

            QueryType::NaturalLanguage => SearchStrategy {
                use_vector: true,
                use_bm25: true,
                use_exact_match: false,
                vector_weight: 0.7,
                bm25_weight: 0.3,
                threshold: 0.3,
                need_rerank: false,
            },

            QueryType::Semantic => SearchStrategy {
                use_vector: true,
                use_bm25: false,
                use_exact_match: false,
                vector_weight: 0.9,
                bm25_weight: 0.1,
                threshold: 0.5,
                need_rerank: false,
            },

            QueryType::Temporal => SearchStrategy {
                use_vector: false,
                use_bm25: true,
                use_exact_match: true,
                vector_weight: 0.2,
                bm25_weight: 0.8,
                threshold: 0.0,
                need_rerank: false,
            },
        }
    }

    /// 获取自适应策略（结合查询特征）
    pub fn get_adaptive_strategy(&self, query: &str) -> (QueryType, SearchStrategy) {
        let query_type = self.classify(query);
        let mut strategy = self.get_strategy(&query_type);

        // 根据查询特征微调策略
        let features = self.extract_features(query);

        // 如果查询很长，增加向量权重
        if features.length > 50 {
            strategy.vector_weight = (strategy.vector_weight + 0.1).min(1.0);
            strategy.bm25_weight = 1.0 - strategy.vector_weight;
        }

        // 如果包含很多数字，增加BM25权重
        let digit_ratio =
            query.chars().filter(|c| c.is_numeric()).count() as f32 / query.len() as f32;
        if digit_ratio > 0.3 {
            strategy.bm25_weight = (strategy.bm25_weight + 0.2).min(1.0);
            strategy.vector_weight = 1.0 - strategy.bm25_weight;
        }

        (query_type, strategy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_classifier_exact_id() {
        let classifier = QueryClassifier::with_default_config();

        assert_eq!(classifier.classify("P000001"), QueryType::ExactId);
        assert_eq!(classifier.classify("ABC-123"), QueryType::ExactId);
    }

    #[test]
    fn test_query_classifier_short_keyword() {
        let classifier = QueryClassifier::with_default_config();

        assert_eq!(classifier.classify("Apple"), QueryType::ShortKeyword);
        assert_eq!(classifier.classify("手机"), QueryType::ShortKeyword);
    }

    #[test]
    fn test_query_classifier_natural_language() {
        let classifier = QueryClassifier::with_default_config();

        assert_eq!(
            classifier.classify("推荐一款手机"),
            QueryType::NaturalLanguage
        );
    }

    #[test]
    fn test_query_classifier_semantic() {
        let classifier = QueryClassifier::with_default_config();

        assert_eq!(
            classifier.classify("What is the best phone for photography?"),
            QueryType::Semantic
        );
        assert_eq!(classifier.classify("什么是人工智能？"), QueryType::Semantic);
    }

    #[test]
    fn test_extract_features() {
        let classifier = QueryClassifier::with_default_config();

        let features = classifier.extract_features("What is AI?");
        assert!(features.is_question);
        assert_eq!(features.language, Language::English);

        let features = classifier.extract_features("人工智能");
        assert_eq!(features.language, Language::Chinese);
    }

    #[test]
    fn test_get_strategy() {
        let classifier = QueryClassifier::with_default_config();

        let strategy = classifier.get_strategy(&QueryType::ExactId);
        assert!(!strategy.use_vector);
        assert!(strategy.use_exact_match);
        assert_eq!(strategy.bm25_weight, 1.0);

        let strategy = classifier.get_strategy(&QueryType::Semantic);
        assert!(strategy.use_vector);
        assert_eq!(strategy.vector_weight, 0.9);
    }

    #[test]
    fn test_adaptive_strategy() {
        let classifier = QueryClassifier::with_default_config();

        let (query_type, strategy) = classifier.get_adaptive_strategy("P000001");
        assert_eq!(query_type, QueryType::ExactId);
        assert!(!strategy.use_vector);

        let (query_type, _) = classifier
            .get_adaptive_strategy("What is the meaning of life, the universe, and everything?");
        assert_eq!(query_type, QueryType::Semantic);
    }
}
