//! 适配器模块 - 桥接现有AgentMem组件与增强混合搜索
//!
//! 这个模块展示如何以**最小改造**的方式集成新功能

use agent_mem_core::search::{enhanced_hybrid_v2::*, SearchResult};
use agent_mem_traits::{AgentMemError, Embedder, Result, VectorStore};
use std::sync::Arc;
use tracing::debug;

/// 向量搜索适配器
/// 
/// 桥接现有的 VectorStore trait 到新的 VectorSearcher trait
pub struct VectorSearcherAdapter {
    vector_store: Arc<dyn VectorStore + Send + Sync>,
    embedder: Arc<dyn Embedder + Send + Sync>,
}

impl VectorSearcherAdapter {
    pub fn new(
        vector_store: Arc<dyn VectorStore + Send + Sync>,
        embedder: Arc<dyn Embedder + Send + Sync>,
    ) -> Self {
        Self {
            vector_store,
            embedder,
        }
    }
}

#[async_trait::async_trait]
impl VectorSearcher for VectorSearcherAdapter {
    async fn search(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        debug!("VectorSearcher: query='{}', limit={}, threshold={}", query, limit, threshold);
        
        // 1. 使用现有embedder生成向量
        let query_vector = self.embedder.embed(query).await?;
        
        // 2. 调用现有vector_store搜索
        let vector_results = self.vector_store
            .search_vectors(query_vector, limit)
            .await?;
        
        // 3. 转换为SearchResult格式并应用阈值
        let results: Vec<SearchResult> = vector_results
            .into_iter()
            .filter(|r| r.score >= threshold)
            .map(|r| {
                let content = r.metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                SearchResult {
                    id: r.id,
                    content,
                    score: r.score,
                    vector_score: Some(r.score),
                    fulltext_score: None,
                    metadata: Some(serde_json::to_value(&r.metadata).unwrap_or_default()),
                }
            })
            .collect();
        
        debug!("VectorSearcher: returned {} results", results.len());
        Ok(results)
    }
}

/// BM25搜索适配器
///
/// 桥接 LibSQLFTS5Store 到 BM25Searcher trait
pub struct BM25SearcherAdapter {
    store: Arc<agent_mem_storage::backends::LibSQLFTS5Store>,
}

impl BM25SearcherAdapter {
    pub fn new(store: Arc<agent_mem_storage::backends::LibSQLFTS5Store>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl BM25Searcher for BM25SearcherAdapter {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        debug!("BM25Searcher: query='{}', limit={}", query, limit);
        
        // 调用FTS5的BM25搜索
        let fts_results = self.store.search_bm25(query, limit, None).await?;
        
        // 转换为SearchResult格式
        let results: Vec<SearchResult> = fts_results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: None,
                fulltext_score: Some(r.score),
                metadata: Some(serde_json::to_value(&r.metadata).unwrap_or_default()),
            })
            .collect();
        
        debug!("BM25Searcher: returned {} results", results.len());
        Ok(results)
    }
}

/// 精确匹配适配器
///
/// 桥接 LibSQLFTS5Store 的精确匹配功能到 ExactMatcher trait
pub struct ExactMatcherAdapter {
    store: Arc<agent_mem_storage::backends::LibSQLFTS5Store>,
}

impl ExactMatcherAdapter {
    pub fn new(store: Arc<agent_mem_storage::backends::LibSQLFTS5Store>) -> Self {
        Self { store }
    }
}

#[async_trait::async_trait]
impl ExactMatcher for ExactMatcherAdapter {
    async fn match_exact(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        debug!("ExactMatcher: query='{}', limit={}", query, limit);
        
        // 调用精确匹配
        let exact_results = self.store.exact_match(query, limit, None).await?;
        
        // 转换为SearchResult格式
        let results: Vec<SearchResult> = exact_results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                content: r.content,
                score: 1.0, // 精确匹配得分为1.0
                vector_score: None,
                fulltext_score: None,
                metadata: Some(serde_json::to_value(&r.metadata).unwrap_or_default()),
            })
            .collect();
        
        debug!("ExactMatcher: returned {} results", results.len());
        Ok(results)
    }
}

/// MemoryOrchestrator扩展
/// 
/// 为现有的MemoryOrchestrator添加增强搜索功能
/// 
/// 使用方式：
/// ```rust
/// use enhanced_search::OrchestratorExt;
/// 
/// let enhanced_results = orchestrator_ext
///     .search_enhanced(&orchestrator, query, limit)
///     .await?;
/// ```
pub struct OrchestratorExt;

impl OrchestratorExt {
    /// 增强搜索（不修改原有orchestrator）
    /// 
    /// 这个方法展示如何在不修改MemoryOrchestrator的情况下
    /// 使用增强的混合搜索功能
    pub async fn search_enhanced(
        orchestrator: &agent_mem::MemoryOrchestrator,
        query: &str,
        limit: usize,
    ) -> Result<EnhancedSearchResult> {
        use agent_mem_core::search::{EnhancedHybridSearchEngineV2, EnhancedHybridConfig};
        
        // 1. 创建配置
        let config = EnhancedHybridConfig {
            enable_query_classification: true,
            enable_adaptive_threshold: true,
            enable_parallel: true,
            enable_metrics: true,
            enable_cache: false,
            rrf_k: 60.0,
        };
        
        // 2. 创建引擎
        let mut engine = EnhancedHybridSearchEngineV2::new(config);
        
        // 3. 如果orchestrator有向量存储，添加向量搜索器
        // 注意：这里需要访问orchestrator的私有字段
        // 在实际使用中，可以通过添加getter方法或使用公开的API
        
        // 示例：假设我们可以获取vector_store和embedder
        // if let (Some(vector_store), Some(embedder)) = 
        //     (orchestrator.get_vector_store(), orchestrator.get_embedder()) 
        // {
        //     let adapter = VectorSearcherAdapter::new(vector_store, embedder);
        //     engine = engine.with_vector_searcher(Arc::new(adapter));
        // }
        
        // 4. 执行搜索
        engine.search(query, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adapter_creation() {
        // 测试适配器可以正常创建
        // 实际测试需要mock或真实的存储实例
    }
}

