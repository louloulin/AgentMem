//! BM25 搜索引擎
//!
//! 实现 BM25 (Best Matching 25) 算法，这是一种基于概率的信息检索算法，
//! 广泛用于全文搜索和文档排序。

use super::{SearchQuery, SearchResult};
use agent_mem_traits::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// BM25 参数
#[derive(Debug, Clone)]
pub struct BM25Params {
    /// k1 参数：控制词频饱和度 (通常 1.2-2.0)
    pub k1: f32,
    /// b 参数：控制文档长度归一化 (通常 0.75)
    pub b: f32,
    /// 最小 IDF 值
    pub min_idf: f32,
}

impl Default for BM25Params {
    fn default() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
            min_idf: 0.0,
        }
    }
}

/// 文档统计信息
#[derive(Debug, Clone)]
struct DocumentStats {
    /// 文档 ID
    id: String,
    /// 文档内容
    content: String,
    /// 文档长度（词数）
    length: usize,
    /// 词频统计
    term_frequencies: HashMap<String, usize>,
}

/// BM25 搜索引擎
pub struct BM25SearchEngine {
    /// BM25 参数
    params: BM25Params,
    /// 文档集合
    documents: Arc<RwLock<Vec<DocumentStats>>>,
    /// 平均文档长度
    avg_doc_length: Arc<RwLock<f32>>,
    /// 逆文档频率 (IDF) 缓存
    idf_cache: Arc<RwLock<HashMap<String, f32>>>,
}

impl BM25SearchEngine {
    /// 创建新的 BM25 搜索引擎
    pub fn new(params: BM25Params) -> Self {
        Self {
            params,
            documents: Arc::new(RwLock::new(Vec::new())),
            avg_doc_length: Arc::new(RwLock::new(0.0)),
            idf_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 使用默认参数创建
    pub fn with_defaults() -> Self {
        Self::new(BM25Params::default())
    }

    /// 添加文档到索引
    pub async fn add_document(&self, id: String, content: String) -> Result<()> {
        let stats = self.compute_document_stats(id, content);

        let mut documents = self.documents.write().await;
        documents.push(stats);

        // 更新平均文档长度
        self.update_avg_doc_length(&documents).await;

        // 清空 IDF 缓存（需要重新计算）
        self.idf_cache.write().await.clear();

        Ok(())
    }

    /// 批量添加文档
    pub async fn add_documents(&self, docs: Vec<(String, String)>) -> Result<()> {
        let mut documents = self.documents.write().await;

        for (id, content) in docs {
            let stats = self.compute_document_stats(id, content);
            documents.push(stats);
        }

        // 更新平均文档长度
        self.update_avg_doc_length(&documents).await;

        // 清空 IDF 缓存
        self.idf_cache.write().await.clear();

        Ok(())
    }

    /// 执行 BM25 搜索
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let query_terms = self.tokenize(&query.query);

        if query_terms.is_empty() {
            return Ok(Vec::new());
        }

        let documents = self.documents.read().await;
        let avg_doc_len = *self.avg_doc_length.read().await;

        // 计算每个文档的 BM25 分数
        let mut scored_docs: Vec<(String, String, f32)> = Vec::new();

        for doc in documents.iter() {
            let score = self
                .calculate_bm25_score(&query_terms, doc, avg_doc_len, documents.len())
                .await;

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

    /// 计算文档的 BM25 分数
    async fn calculate_bm25_score(
        &self,
        query_terms: &[String],
        doc: &DocumentStats,
        avg_doc_len: f32,
        total_docs: usize,
    ) -> f32 {
        let mut score = 0.0;

        for term in query_terms {
            // 获取词频
            let tf = *doc.term_frequencies.get(term).unwrap_or(&0) as f32;

            if tf == 0.0 {
                continue;
            }

            // 获取或计算 IDF
            let idf = self.get_or_compute_idf(term, total_docs).await;

            // BM25 公式
            let doc_len = doc.length as f32;
            let normalized_tf = tf * (self.params.k1 + 1.0)
                / (tf
                    + self.params.k1
                        * (1.0 - self.params.b + self.params.b * doc_len / avg_doc_len));

            score += idf * normalized_tf;
        }

        score
    }

    /// 获取或计算 IDF
    async fn get_or_compute_idf(&self, term: &str, total_docs: usize) -> f32 {
        // 先检查缓存
        {
            let cache = self.idf_cache.read().await;
            if let Some(&idf) = cache.get(term) {
                return idf;
            }
        }

        // 计算 IDF
        let documents = self.documents.read().await;
        let doc_freq = documents
            .iter()
            .filter(|doc| doc.term_frequencies.contains_key(term))
            .count();

        let idf = if doc_freq > 0 {
            let idf_value =
                ((total_docs as f32 - doc_freq as f32 + 0.5) / (doc_freq as f32 + 0.5) + 1.0).ln();
            idf_value.max(self.params.min_idf)
        } else {
            self.params.min_idf
        };

        // 缓存 IDF
        self.idf_cache.write().await.insert(term.to_string(), idf);

        idf
    }

    /// 计算文档统计信息
    fn compute_document_stats(&self, id: String, content: String) -> DocumentStats {
        let tokens = self.tokenize(&content);
        let length = tokens.len();

        let mut term_frequencies = HashMap::new();
        for token in tokens {
            *term_frequencies.entry(token).or_insert(0) += 1;
        }

        DocumentStats {
            id,
            content,
            length,
            term_frequencies,
        }
    }

    /// 更新平均文档长度
    async fn update_avg_doc_length(&self, documents: &[DocumentStats]) {
        if documents.is_empty() {
            *self.avg_doc_length.write().await = 0.0;
            return;
        }

        let total_length: usize = documents.iter().map(|doc| doc.length).sum();
        let avg = total_length as f32 / documents.len() as f32;
        *self.avg_doc_length.write().await = avg;
    }

    /// 分词
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|word| word.len() > 2)
            .map(|word| {
                // 移除标点符号
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// 获取文档数量
    pub async fn document_count(&self) -> usize {
        self.documents.read().await.len()
    }

    /// 清空索引
    pub async fn clear(&self) {
        self.documents.write().await.clear();
        *self.avg_doc_length.write().await = 0.0;
        self.idf_cache.write().await.clear();
    }
}

// ============================================================================
// SearchEngine Trait 实现 (V4)
// ============================================================================

use agent_mem_traits::{SearchEngine, Query, QueryIntent, QueryIntentType};
use async_trait::async_trait;

#[async_trait]
impl SearchEngine for BM25SearchEngine {
    /// 执行搜索查询（V4 Query 接口）
    async fn search(&self, query: &Query) -> Result<Vec<agent_mem_traits::SearchResultV4>> {
        // 1. 提取查询文本
        let query_text = match &query.intent {
            QueryIntent::NaturalLanguage { text, .. } => text.clone(),
            QueryIntent::Hybrid { intents, .. } => {
                // 从混合查询中提取自然语言意图
                intents.iter()
                    .find_map(|intent| {
                        if let QueryIntent::NaturalLanguage { text, .. } = intent {
                            Some(text.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            }
            _ => {
                return Err(agent_mem_traits::AgentMemError::validation_error(
                    format!("Unsupported query intent for BM25SearchEngine: {:?}. Use NaturalLanguage or Hybrid intent.", query.intent)
                ));
            }
        };

        // 2. 转换 Query V4 到 SearchQuery
        let mut search_query = SearchQuery::from_query_v4(query);
        search_query.query = query_text;

        // 3. 执行 BM25 搜索（使用现有的 search 方法）
        let results = self.search(&search_query).await?;

        // 4. 转换 SearchResult 到 SearchResultV4
        let v4_results = results.into_iter()
            .map(|r| agent_mem_traits::SearchResultV4 {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: r.vector_score,
                fulltext_score: r.fulltext_score,
                metadata: r.metadata,
            })
            .collect();

        Ok(v4_results)
    }

    /// 获取引擎名称
    fn name(&self) -> &str {
        "BM25SearchEngine"
    }

    /// 获取支持的查询意图类型
    fn supported_intents(&self) -> Vec<QueryIntentType> {
        vec![
            QueryIntentType::NaturalLanguage, // 主要支持自然语言查询
            QueryIntentType::Hybrid, // 也支持混合查询（提取文本部分）
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bm25_basic() {
        let engine = BM25SearchEngine::with_defaults();

        // 添加文档
        engine
            .add_document("doc1".to_string(), "the quick brown fox".to_string())
            .await
            .unwrap();
        engine
            .add_document("doc2".to_string(), "the lazy dog".to_string())
            .await
            .unwrap();
        engine
            .add_document("doc3".to_string(), "quick brown dog".to_string())
            .await
            .unwrap();

        // 搜索
        let query = SearchQuery {
            query: "quick brown".to_string(),
            limit: 10,
            ..Default::default()
        };

        let results = engine.search(&query).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc3"); // doc3 应该得分最高
    }

    #[tokio::test]
    async fn test_bm25_empty_query() {
        let engine = BM25SearchEngine::with_defaults();

        let query = SearchQuery {
            query: "".to_string(),
            limit: 10,
            ..Default::default()
        };

        let results = engine.search(&query).await.unwrap();
        assert!(results.is_empty());
    }
}
