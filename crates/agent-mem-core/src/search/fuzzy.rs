//! 模糊匹配搜索引擎
//!
//! 实现基于编辑距离（Levenshtein Distance）的模糊匹配搜索，
//! 用于处理拼写错误、变体等情况。

use super::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 模糊匹配参数
#[derive(Debug, Clone)]
pub struct FuzzyMatchParams {
    /// 最大编辑距离
    pub max_edit_distance: usize,
    /// 最小匹配长度
    pub min_match_length: usize,
    /// 是否区分大小写
    pub case_sensitive: bool,
    /// N-gram 大小（用于优化）
    pub ngram_size: usize,
}

impl Default for FuzzyMatchParams {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            min_match_length: 3,
            case_sensitive: false,
            ngram_size: 3,
        }
    }
}

/// 文档信息
#[derive(Debug, Clone)]
struct FuzzyDocument {
    /// 文档 ID
    id: String,
    /// 文档内容
    content: String,
    /// N-gram 索引
    ngrams: HashMap<String, Vec<usize>>,
}

/// 模糊匹配搜索引擎
pub struct FuzzyMatchEngine {
    /// 参数
    params: FuzzyMatchParams,
    /// 文档集合
    documents: Arc<RwLock<Vec<FuzzyDocument>>>,
}

impl FuzzyMatchEngine {
    /// 创建新的模糊匹配引擎
    pub fn new(params: FuzzyMatchParams) -> Self {
        Self {
            params,
            documents: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 使用默认参数创建
    pub fn with_defaults() -> Self {
        Self::new(FuzzyMatchParams::default())
    }

    /// 添加文档
    pub async fn add_document(&self, id: String, content: String) -> Result<()> {
        let ngrams = self.build_ngram_index(&content);

        let doc = FuzzyDocument {
            id,
            content,
            ngrams,
        };

        self.documents.write().await.push(doc);
        Ok(())
    }

    /// 批量添加文档
    pub async fn add_documents(&self, docs: Vec<(String, String)>) -> Result<()> {
        let mut documents = self.documents.write().await;

        for (id, content) in docs {
            let ngrams = self.build_ngram_index(&content);
            documents.push(FuzzyDocument {
                id,
                content,
                ngrams,
            });
        }

        Ok(())
    }

    /// 执行模糊搜索
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let query_text = if self.params.case_sensitive {
            query.query.clone()
        } else {
            query.query.to_lowercase()
        };

        if query_text.len() < self.params.min_match_length {
            return Ok(Vec::new());
        }

        let documents = self.documents.read().await;
        let mut scored_docs: Vec<(String, String, f32)> = Vec::new();

        for doc in documents.iter() {
            let doc_text = if self.params.case_sensitive {
                doc.content.clone()
            } else {
                doc.content.to_lowercase()
            };

            // 计算模糊匹配分数
            let score = self.calculate_fuzzy_score(&query_text, &doc_text);

            if score > 0.0 {
                scored_docs.push((doc.id.clone(), doc.content.clone(), score));
            }
        }

        // 按分数降序排序
        scored_docs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // 转换为搜索结果
        let results = scored_docs
            .into_iter()
            .take(query.limit)
            .map(|(id, content, score)| SearchResult {
                id,
                content,
                score,
                vector_score: None,
                fulltext_score: Some(score),
                metadata: None,
            })
            .collect();

        Ok(results)
    }

    /// 计算模糊匹配分数
    fn calculate_fuzzy_score(&self, query: &str, text: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let text_words: Vec<&str> = text.split_whitespace().collect();

        if query_words.is_empty() || text_words.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        let mut matched_words = 0;

        for query_word in &query_words {
            let mut best_match_score = 0.0;

            for text_word in &text_words {
                let distance = self.levenshtein_distance(query_word, text_word);

                if distance <= self.params.max_edit_distance {
                    // 计算相似度分数 (1.0 - 归一化距离)
                    let max_len = query_word.len().max(text_word.len()) as f32;
                    let similarity = 1.0 - (distance as f32 / max_len);

                    if similarity > best_match_score {
                        best_match_score = similarity;
                    }
                }
            }

            if best_match_score > 0.0 {
                total_score += best_match_score;
                matched_words += 1;
            }
        }

        if matched_words == 0 {
            return 0.0;
        }

        // 归一化分数
        total_score / query_words.len() as f32
    }

    /// 计算 Levenshtein 距离（编辑距离）
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        // 创建距离矩阵
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // 初始化第一行和第一列
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // 填充矩阵
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };

                matrix[i][j] = min(
                    min(
                        matrix[i - 1][j] + 1, // 删除
                        matrix[i][j - 1] + 1, // 插入
                    ),
                    matrix[i - 1][j - 1] + cost, // 替换
                );
            }
        }

        matrix[len1][len2]
    }

    /// 构建 N-gram 索引
    fn build_ngram_index(&self, text: &str) -> HashMap<String, Vec<usize>> {
        let mut ngrams = HashMap::new();
        let text_lower = text.to_lowercase();
        let chars: Vec<char> = text_lower.chars().collect();

        if chars.len() < self.params.ngram_size {
            return ngrams;
        }

        for i in 0..=chars.len() - self.params.ngram_size {
            let ngram: String = chars[i..i + self.params.ngram_size].iter().collect();
            ngrams.entry(ngram).or_insert_with(Vec::new).push(i);
        }

        ngrams
    }

    /// 获取文档数量
    pub async fn document_count(&self) -> usize {
        self.documents.read().await.len()
    }

    /// 清空索引
    pub async fn clear(&self) {
        self.documents.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fuzzy_match_basic() {
        let engine = FuzzyMatchEngine::with_defaults();

        // 添加文档
        engine
            .add_document("doc1".to_string(), "the quick brown fox".to_string())
            .await
            .unwrap();
        engine
            .add_document("doc2".to_string(), "the quik brwn fox".to_string())
            .await
            .unwrap(); // 拼写错误
        engine
            .add_document("doc3".to_string(), "lazy dog".to_string())
            .await
            .unwrap();

        // 搜索（包含拼写错误）
        let query = SearchQuery {
            query: "quick brown".to_string(),
            limit: 10,
            ..Default::default()
        };

        let results = engine.search(&query).await.unwrap();

        assert!(!results.is_empty());
        // doc1 和 doc2 都应该被找到
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_levenshtein_distance() {
        let engine = FuzzyMatchEngine::with_defaults();

        assert_eq!(engine.levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(engine.levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(engine.levenshtein_distance("hello", "hello"), 0);
        assert_eq!(engine.levenshtein_distance("", "hello"), 5);
    }

    #[tokio::test]
    async fn test_fuzzy_match_case_insensitive() {
        let engine = FuzzyMatchEngine::with_defaults();

        engine
            .add_document("doc1".to_string(), "Hello World".to_string())
            .await
            .unwrap();

        let query = SearchQuery {
            query: "hello world".to_string(),
            limit: 10,
            ..Default::default()
        };

        let results = engine.search(&query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].score > 0.9); // 应该是高分匹配
    }
}
