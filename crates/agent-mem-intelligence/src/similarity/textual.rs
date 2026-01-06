//! 文本相似度计算实现

use agent_mem_traits::Result;
use agent_mem_utils::text::extract_keywords;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 文本相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSimilarityResult {
    /// 相似度分数 (0.0 - 1.0)
    pub similarity: f32,
    /// 相似度类型
    pub similarity_type: String,
    /// 匹配的关键词
    pub matched_keywords: Vec<String>,
    /// 总关键词数
    pub total_keywords: usize,
    /// 是否相似
    pub is_similar: bool,
}

/// 文本相似度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSimilarityConfig {
    /// 相似度阈值
    pub threshold: f32,
    /// 最小关键词长度
    pub min_keyword_length: usize,
    /// 是否忽略大小写
    pub ignore_case: bool,
    /// 是否使用词干提取
    pub use_stemming: bool,
}

impl Default for TextSimilarityConfig {
    fn default() -> Self {
        Self {
            threshold: 0.3,
            min_keyword_length: 3,
            ignore_case: true,
            use_stemming: false,
        }
    }
}

/// 文本相似度计算器
pub struct TextualSimilarity {
    config: TextSimilarityConfig,
}

impl TextualSimilarity {
    /// 创建新的文本相似度计算器
    pub fn new(config: TextSimilarityConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(TextSimilarityConfig::default())
    }

    /// 计算两个文本的相似度
    pub fn calculate_similarity(&self, text1: &str, text2: &str) -> Result<TextSimilarityResult> {
        // 预处理文本
        let processed_text1 = self.preprocess_text(text1);
        let processed_text2 = self.preprocess_text(text2);

        // 提取关键词
        let keywords1 = self.extract_text_keywords(&processed_text1);
        let keywords2 = self.extract_text_keywords(&processed_text2);

        // 计算Jaccard相似度
        let similarity = self.calculate_jaccard_similarity(&keywords1, &keywords2);

        // 找到匹配的关键词
        let matched_keywords: Vec<String> = keywords1.intersection(&keywords2).cloned().collect();

        let total_keywords = keywords1.union(&keywords2).count();

        Ok(TextSimilarityResult {
            similarity,
            similarity_type: "jaccard".to_string(),
            matched_keywords,
            total_keywords,
            is_similar: similarity >= self.config.threshold,
        })
    }

    /// 计算编辑距离相似度
    pub fn calculate_edit_distance_similarity(
        &self,
        text1: &str,
        text2: &str,
    ) -> Result<TextSimilarityResult> {
        let distance = self.levenshtein_distance(text1, text2);
        let max_len = text1.len().max(text2.len());

        let similarity = if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f32 / max_len as f32)
        };

        Ok(TextSimilarityResult {
            similarity,
            similarity_type: "edit_distance".to_string(),
            matched_keywords: Vec::new(),
            total_keywords: 0,
            is_similar: similarity >= self.config.threshold,
        })
    }

    /// 计算N-gram相似度
    pub fn calculate_ngram_similarity(
        &self,
        text1: &str,
        text2: &str,
        n: usize,
    ) -> Result<TextSimilarityResult> {
        let ngrams1 = self.extract_ngrams(text1, n);
        let ngrams2 = self.extract_ngrams(text2, n);

        let similarity = self.calculate_jaccard_similarity(&ngrams1, &ngrams2);

        let matched_ngrams: Vec<String> = ngrams1.intersection(&ngrams2).cloned().collect();

        let total_ngrams = ngrams1.union(&ngrams2).count();

        Ok(TextSimilarityResult {
            similarity,
            similarity_type: format!("{n}-gram"),
            matched_keywords: matched_ngrams,
            total_keywords: total_ngrams,
            is_similar: similarity >= self.config.threshold,
        })
    }

    /// 预处理文本
    fn preprocess_text(&self, text: &str) -> String {
        let mut processed = text.to_string();

        if self.config.ignore_case {
            processed = processed.to_lowercase();
        }

        // 移除标点符号和多余空格
        processed = processed
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c.is_whitespace() {
                    c
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        processed
    }

    /// 提取文本关键词
    fn extract_text_keywords(&self, text: &str) -> HashSet<String> {
        let keywords = extract_keywords(text, self.config.min_keyword_length);
        keywords.into_iter().collect()
    }

    /// 计算Levenshtein距离
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // 初始化第一行和第一列
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// 提取N-gram
    fn extract_ngrams(&self, text: &str, n: usize) -> HashSet<String> {
        let processed = self.preprocess_text(text);
        let chars: Vec<char> = processed.chars().collect();
        let mut ngrams = HashSet::new();

        if chars.len() < n {
            ngrams.insert(processed);
            return ngrams;
        }

        for i in 0..=chars.len() - n {
            let ngram: String = chars[i..i + n].iter().collect();
            ngrams.insert(ngram);
        }

        ngrams
    }

    /// 批量文本相似度计算
    pub fn batch_similarity(
        &self,
        query: &str,
        texts: &[String],
    ) -> Result<Vec<TextSimilarityResult>> {
        let mut results = Vec::new();

        for text in texts {
            let result = self.calculate_similarity(query, text)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 找到最相似的文本
    pub fn find_most_similar(
        &self,
        query: &str,
        texts: &[String],
    ) -> Result<Option<(usize, TextSimilarityResult)>> {
        if texts.is_empty() {
            return Ok(None);
        }

        let results = self.batch_similarity(query, texts)?;

        let max_result = results.iter().enumerate().max_by(|(_, a), (_, b)| {
            a.similarity
                .partial_cmp(&b.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some((index, result)) = max_result {
            Ok(Some((index, result.clone())))
        } else {
            Ok(None)
        }
    }

    /// 更新配置
    pub fn update_config(&mut self, config: TextSimilarityConfig) {
        self.config = config;
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &TextSimilarityConfig {
        &self.config
    }

    /// 计算Jaccard相似度
    fn calculate_jaccard_similarity(&self, set1: &HashSet<String>, set2: &HashSet<String>) -> f32 {
        let intersection_size = set1.intersection(set2).count();
        let union_size = set1.union(set2).count();

        if union_size == 0 {
            0.0
        } else {
            intersection_size as f32 / union_size as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textual_similarity_creation() {
        let similarity = TextualSimilarity::default();
        assert_eq!(similarity.config.threshold, 0.3);
        assert_eq!(similarity.config.min_keyword_length, 3);
    }

    #[test]
    fn test_jaccard_similarity() {
        let similarity = TextualSimilarity::default();

        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "A quick brown fox leaps over a lazy dog";

        let result = similarity.calculate_similarity(text1, text2).unwrap();
        assert!(result.similarity > 0.5);
        assert_eq!(result.similarity_type, "jaccard");
        assert!(!result.matched_keywords.is_empty());
    }

    #[test]
    fn test_edit_distance_similarity() {
        let similarity = TextualSimilarity::default();

        let text1 = "hello world";
        let text2 = "hello word";

        let result = similarity
            .calculate_edit_distance_similarity(text1, text2)
            .unwrap();
        assert!(result.similarity > 0.8);
        assert_eq!(result.similarity_type, "edit_distance");
    }

    #[test]
    fn test_ngram_similarity() {
        let similarity = TextualSimilarity::default();

        let text1 = "hello world";
        let text2 = "hello word";

        let result = similarity
            .calculate_ngram_similarity(text1, text2, 3)
            .unwrap();
        assert!(result.similarity > 0.0);
        assert_eq!(result.similarity_type, "3-gram");
    }

    #[test]
    fn test_preprocess_text() {
        let similarity = TextualSimilarity::default();

        let text = "Hello, World! This is a TEST.";
        let processed = similarity.preprocess_text(text);
        assert_eq!(processed, "hello world this is a test");
    }

    #[test]
    fn test_levenshtein_distance() {
        let similarity = TextualSimilarity::default();

        assert_eq!(similarity.levenshtein_distance("", ""), 0);
        assert_eq!(similarity.levenshtein_distance("hello", ""), 5);
        assert_eq!(similarity.levenshtein_distance("", "world"), 5);
        assert_eq!(similarity.levenshtein_distance("hello", "hello"), 0);
        assert_eq!(similarity.levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(similarity.levenshtein_distance("hello", "world"), 4);
    }

    #[test]
    fn test_extract_ngrams() {
        let similarity = TextualSimilarity::default();

        let ngrams = similarity.extract_ngrams("hello", 2);
        assert!(ngrams.contains("he"));
        assert!(ngrams.contains("el"));
        assert!(ngrams.contains("ll"));
        assert!(ngrams.contains("lo"));
    }

    #[test]
    fn test_batch_similarity() {
        let similarity = TextualSimilarity::default();
        let query = "hello world";
        let texts = vec![
            "hello world".to_string(),
            "goodbye world".to_string(),
            "hello universe".to_string(),
        ];

        let results = similarity.batch_similarity(query, &texts).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].similarity > results[1].similarity);
    }

    #[test]
    fn test_find_most_similar() {
        let similarity = TextualSimilarity::default();
        let query = "hello world";
        let texts = vec![
            "goodbye world".to_string(),
            "hello universe".to_string(),
            "hello world".to_string(),
        ];

        let result = similarity.find_most_similar(query, &texts).unwrap();
        assert!(result.is_some());
        let (index, sim_result) = result.unwrap();
        assert_eq!(index, 2);
        assert!(sim_result.similarity > 0.9);
    }
}
