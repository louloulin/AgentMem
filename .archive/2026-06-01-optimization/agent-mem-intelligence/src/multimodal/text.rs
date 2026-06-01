//! 文本内容处理模块

use super::{ContentType, MultimodalContent, MultimodalProcessor};
use agent_mem_traits::{AgentMemError, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 文本处理器
#[derive(Debug)]
pub struct TextProcessor {
    /// 是否启用关键词提取
    pub enable_keyword_extraction: bool,
    /// 是否启用情感分析
    pub enable_sentiment_analysis: bool,
    /// 是否启用语言检测
    pub enable_language_detection: bool,
    /// 是否启用实体识别
    pub enable_entity_recognition: bool,
}

impl TextProcessor {
    /// 创建新的文本处理器
    pub fn new() -> Self {
        Self {
            enable_keyword_extraction: true,
            enable_sentiment_analysis: true,
            enable_language_detection: true,
            enable_entity_recognition: true,
        }
    }

    /// 配置关键词提取
    pub fn with_keyword_extraction(mut self, enable: bool) -> Self {
        self.enable_keyword_extraction = enable;
        self
    }

    /// 配置情感分析
    pub fn with_sentiment_analysis(mut self, enable: bool) -> Self {
        self.enable_sentiment_analysis = enable;
        self
    }

    /// 配置语言检测
    pub fn with_language_detection(mut self, enable: bool) -> Self {
        self.enable_language_detection = enable;
        self
    }

    /// 配置实体识别
    pub fn with_entity_recognition(mut self, enable: bool) -> Self {
        self.enable_entity_recognition = enable;
        self
    }

    /// 提取关键词
    async fn extract_keywords(&self, text: &str) -> Result<Vec<Keyword>> {
        if !self.enable_keyword_extraction {
            return Ok(vec![]);
        }

        // 简化的关键词提取算法
        let words: Vec<&str> = text
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .collect();

        let mut keyword_counts: HashMap<String, usize> = HashMap::new();
        for word in words {
            let clean_word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if !clean_word.is_empty() && !is_stop_word(&clean_word) {
                *keyword_counts.entry(clean_word).or_insert(0) += 1;
            }
        }

        let mut keywords: Vec<Keyword> = keyword_counts
            .into_iter()
            .map(|(word, count)| Keyword {
                word,
                frequency: count,
                relevance: (count as f32) / (text.len() as f32) * 1000.0,
            })
            .collect();

        keywords.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
        keywords.truncate(10); // 只保留前10个关键词

        Ok(keywords)
    }

    /// 分析情感
    async fn analyze_sentiment(&self, text: &str) -> Result<SentimentAnalysis> {
        if !self.enable_sentiment_analysis {
            return Ok(SentimentAnalysis::default());
        }

        // 简化的情感分析
        let positive_words = [
            "good",
            "great",
            "excellent",
            "amazing",
            "wonderful",
            "fantastic",
            "love",
            "like",
            "happy",
            "joy",
        ];
        let negative_words = [
            "bad",
            "terrible",
            "awful",
            "hate",
            "dislike",
            "sad",
            "angry",
            "disappointed",
            "frustrated",
        ];

        let text_lower = text.to_lowercase();
        let mut positive_score = 0;
        let mut negative_score = 0;

        for word in positive_words.iter() {
            positive_score += text_lower.matches(word).count();
        }

        for word in negative_words.iter() {
            negative_score += text_lower.matches(word).count();
        }

        let total_score = positive_score + negative_score;
        let sentiment = if total_score == 0 {
            Sentiment::Neutral
        } else if positive_score > negative_score {
            Sentiment::Positive
        } else {
            Sentiment::Negative
        };

        let confidence = if total_score == 0 {
            0.5
        } else {
            (positive_score.max(negative_score) as f32) / (total_score as f32)
        };

        Ok(SentimentAnalysis {
            sentiment,
            confidence,
            positive_score: positive_score as f32,
            negative_score: negative_score as f32,
        })
    }

    /// 检测语言
    async fn detect_language(&self, text: &str) -> Result<LanguageDetection> {
        if !self.enable_language_detection {
            return Ok(LanguageDetection::default());
        }

        // 简化的语言检测
        let english_indicators = [
            "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        ];
        let chinese_chars = text.chars().filter(|c| is_chinese_char(*c)).count();
        let total_chars = text.chars().count();

        let language = if chinese_chars > total_chars / 4 {
            "zh".to_string()
        } else if english_indicators
            .iter()
            .any(|&word| text.to_lowercase().contains(word))
        {
            "en".to_string()
        } else {
            "unknown".to_string()
        };

        let confidence = if language == "unknown" { 0.3 } else { 0.8 };

        Ok(LanguageDetection {
            language,
            confidence,
        })
    }

    /// 识别实体
    async fn recognize_entities(&self, text: &str) -> Result<Vec<Entity>> {
        if !self.enable_entity_recognition {
            return Ok(vec![]);
        }

        let mut entities = Vec::new();

        // 简化的实体识别
        // 识别邮箱
        let email_regex =
            regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        for mat in email_regex.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: EntityType::Email,
                start_pos: mat.start(),
                end_pos: mat.end(),
                confidence: 0.95,
            });
        }

        // 识别URL
        let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
        for mat in url_regex.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: EntityType::Url,
                start_pos: mat.start(),
                end_pos: mat.end(),
                confidence: 0.9,
            });
        }

        // 识别电话号码（简化版）
        let phone_regex = regex::Regex::new(r"\b\d{3}-\d{3}-\d{4}\b|\b\d{10}\b").unwrap();
        for mat in phone_regex.find_iter(text) {
            entities.push(Entity {
                text: mat.as_str().to_string(),
                entity_type: EntityType::Phone,
                start_pos: mat.start(),
                end_pos: mat.end(),
                confidence: 0.85,
            });
        }

        Ok(entities)
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MultimodalProcessor for TextProcessor {
    async fn process(&self, content: &mut MultimodalContent) -> Result<()> {
        // 验证内容类型
        if content.content_type != ContentType::Text {
            return Err(AgentMemError::ProcessingError(
                "TextProcessor can only process text content".to_string(),
            ));
        }

        // 获取文本内容
        let text = if let Some(extracted_text) = &content.extracted_text {
            extracted_text.clone()
        } else if let Some(data) = &content.data {
            // 假设是 Base64 编码的文本
            let decoded = general_purpose::STANDARD.decode(data).map_err(|e| {
                AgentMemError::ProcessingError(format!("Failed to decode text data: {e}"))
            })?;
            String::from_utf8(decoded)
                .map_err(|e| AgentMemError::ProcessingError(format!("Invalid UTF-8 text: {e}")))?
        } else {
            return Err(AgentMemError::ProcessingError(
                "No text content found".to_string(),
            ));
        };

        // 设置提取的文本
        content.set_extracted_text(text.clone());

        // 提取关键词
        if let Ok(keywords) = self.extract_keywords(&text).await {
            let keywords_json = serde_json::to_value(keywords).map_err(|e| {
                AgentMemError::ProcessingError(format!("Failed to serialize keywords: {e}"))
            })?;
            content.set_metadata("keywords".to_string(), keywords_json);
        }

        // 分析情感
        if let Ok(sentiment) = self.analyze_sentiment(&text).await {
            let sentiment_json = serde_json::to_value(sentiment).map_err(|e| {
                AgentMemError::ProcessingError(format!("Failed to serialize sentiment: {e}"))
            })?;
            content.set_metadata("sentiment".to_string(), sentiment_json);
        }

        // 检测语言
        if let Ok(language) = self.detect_language(&text).await {
            let language_json = serde_json::to_value(language).map_err(|e| {
                AgentMemError::ProcessingError(format!("Failed to serialize language: {e}"))
            })?;
            content.set_metadata("language".to_string(), language_json);
        }

        // 识别实体
        if let Ok(entities) = self.recognize_entities(&text).await {
            let entities_json = serde_json::to_value(entities).map_err(|e| {
                AgentMemError::ProcessingError(format!("Failed to serialize entities: {e}"))
            })?;
            content.set_metadata("entities".to_string(), entities_json);
        }

        Ok(())
    }

    fn supported_types(&self) -> Vec<ContentType> {
        vec![ContentType::Text]
    }

    async fn extract_text(&self, content: &MultimodalContent) -> Result<Option<String>> {
        Ok(content.extracted_text.clone())
    }

    async fn generate_summary(&self, content: &MultimodalContent) -> Result<Option<String>> {
        if let Some(text) = &content.extracted_text {
            // 简化的摘要生成：取前100个字符（安全处理多字节字符）
            let summary = if text.chars().count() > 100 {
                let truncated: String = text.chars().take(100).collect();
                format!("{truncated}...")
            } else {
                text.clone()
            };
            Ok(Some(summary))
        } else {
            Ok(None)
        }
    }
}

/// 关键词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    /// 关键词
    pub word: String,
    /// 频率
    pub frequency: usize,
    /// 相关性分数
    pub relevance: f32,
}

/// 情感分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    /// 情感类型
    pub sentiment: Sentiment,
    /// 置信度
    pub confidence: f32,
    /// 积极分数
    pub positive_score: f32,
    /// 消极分数
    pub negative_score: f32,
}

impl Default for SentimentAnalysis {
    fn default() -> Self {
        Self {
            sentiment: Sentiment::Neutral,
            confidence: 0.0,
            positive_score: 0.0,
            negative_score: 0.0,
        }
    }
}

/// 情感类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

/// 语言检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDetection {
    /// 语言代码
    pub language: String,
    /// 置信度
    pub confidence: f32,
}

impl Default for LanguageDetection {
    fn default() -> Self {
        Self {
            language: "unknown".to_string(),
            confidence: 0.0,
        }
    }
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体文本
    pub text: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 开始位置
    pub start_pos: usize,
    /// 结束位置
    pub end_pos: usize,
    /// 置信度
    pub confidence: f32,
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Email,
    Phone,
    Url,
    Date,
    Time,
    Money,
    Other,
}

/// 检查是否为停用词
fn is_stop_word(word: &str) -> bool {
    let stop_words = [
        "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "from", "up",
        "about", "into", "through", "during", "before", "after", "above", "below", "between",
        "among", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
        "does", "did", "will", "would", "could", "should", "may", "might", "must", "can", "this",
        "that", "these", "those", "a", "an",
    ];
    stop_words.contains(&word)
}

/// 检查是否为中文字符
fn is_chinese_char(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9fff}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_text_processor_creation() {
        let processor = TextProcessor::new();
        assert!(processor.enable_keyword_extraction);
        assert!(processor.enable_sentiment_analysis);
        assert!(processor.enable_language_detection);
        assert!(processor.enable_entity_recognition);
    }

    #[tokio::test]
    async fn test_keyword_extraction() {
        let processor = TextProcessor::new();
        let keywords = processor
            .extract_keywords("This is a great example of keyword extraction technology")
            .await
            .unwrap();

        assert!(!keywords.is_empty());
        // 应该包含一些关键词，但不包含停用词
        let keyword_words: Vec<&str> = keywords.iter().map(|k| k.word.as_str()).collect();
        assert!(!keyword_words.contains(&"this"));
        assert!(!keyword_words.contains(&"is"));
    }

    #[tokio::test]
    async fn test_sentiment_analysis() {
        let processor = TextProcessor::new();

        let positive_sentiment = processor
            .analyze_sentiment("This is a great and wonderful day!")
            .await
            .unwrap();
        assert_eq!(positive_sentiment.sentiment, Sentiment::Positive);

        let negative_sentiment = processor
            .analyze_sentiment("This is terrible and awful")
            .await
            .unwrap();
        assert_eq!(negative_sentiment.sentiment, Sentiment::Negative);
    }

    #[tokio::test]
    async fn test_language_detection() {
        let processor = TextProcessor::new();

        let english_detection = processor
            .detect_language("This is an English sentence with common words")
            .await
            .unwrap();
        assert_eq!(english_detection.language, "en");

        let chinese_detection = processor
            .detect_language("这是一个中文句子，包含很多中文字符")
            .await
            .unwrap();
        assert_eq!(chinese_detection.language, "zh");
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("and"));
        assert!(!is_stop_word("technology"));
        assert!(!is_stop_word("example"));
    }

    #[test]
    fn test_is_chinese_char() {
        assert!(is_chinese_char('中'));
        assert!(is_chinese_char('文'));
        assert!(!is_chinese_char('a'));
        assert!(!is_chinese_char('1'));
    }
}
