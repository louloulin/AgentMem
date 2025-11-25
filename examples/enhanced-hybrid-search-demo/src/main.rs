//! Enhanced Hybrid Search Demo
//!
//! 展示如何使用增强的混合检索系统

use agent_mem_core::search::{EnhancedHybridConfig, EnhancedHybridSearchEngineV2, SearchResult};
use agent_mem_storage::backends::LibSQLFTS5Store;
use agent_mem_traits::Result;
use std::sync::Arc;
use tracing::{info, warn};

// 实现BM25搜索器的适配器
struct BM25SearcherAdapter {
    store: Arc<LibSQLFTS5Store>,
}

#[async_trait::async_trait]
impl agent_mem_core::search::enhanced_hybrid_v2::BM25Searcher for BM25SearcherAdapter {
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results = self.store.search_bm25(query, limit, None).await?;
        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                content: r.content,
                score: r.score,
                vector_score: None,
                fulltext_score: Some(r.score),
                metadata: Some(serde_json::to_value(&r.metadata).unwrap()),
            })
            .collect())
    }
}

// 实现精确匹配器的适配器
struct ExactMatcherAdapter {
    store: Arc<LibSQLFTS5Store>,
}

#[async_trait::async_trait]
impl agent_mem_core::search::enhanced_hybrid_v2::ExactMatcher for ExactMatcherAdapter {
    async fn match_exact(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let results = self.store.exact_match(query, limit, None).await?;
        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                id: r.id,
                content: r.content,
                score: 1.0,
                vector_score: None,
                fulltext_score: None,
                metadata: Some(serde_json::to_value(&r.metadata).unwrap()),
            })
            .collect())
    }
}

// 模拟向量搜索器（实际使用时应该连接到LanceDB）
struct MockVectorSearcher;

#[async_trait::async_trait]
impl agent_mem_core::search::enhanced_hybrid_v2::VectorSearcher for MockVectorSearcher {
    async fn search(
        &self,
        query: &str,
        _limit: usize,
        _threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        // 这里应该实际调用LanceDB
        // 现在返回模拟数据用于演示
        Ok(vec![SearchResult {
            id: format!("vec_{}", uuid::Uuid::new_v4()),
            content: format!("Vector search result for: {}", query),
            score: 0.85,
            vector_score: Some(0.85),
            fulltext_score: None,
            metadata: None,
        }])
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Enhanced Hybrid Search Demo Starting...");

    // 1. 初始化LibSQL FTS5存储
    info!("📦 Initializing LibSQL FTS5 Store...");
    let store = Arc::new(LibSQLFTS5Store::new(":memory:").await?);

    // 2. 插入测试数据
    info!("📝 Inserting test data...");
    insert_test_data(&store).await?;

    // 3. 创建增强混合搜索引擎
    info!("🔧 Creating Enhanced Hybrid Search Engine...");
    let config = EnhancedHybridConfig {
        enable_query_classification: true,
        enable_adaptive_threshold: true,
        enable_parallel: true,
        enable_metrics: true,
        enable_cache: false,
        rrf_k: 60.0,
        vector_weight: 0.7,
        fulltext_weight: 0.3,
    };

    let engine = EnhancedHybridSearchEngineV2::new(config)
        .with_vector_searcher(Arc::new(MockVectorSearcher))
        .with_bm25_searcher(Arc::new(BM25SearcherAdapter {
            store: store.clone(),
        }))
        .with_exact_matcher(Arc::new(ExactMatcherAdapter {
            store: store.clone(),
        }));

    info!("✅ Engine created successfully\n");

    // 4. 运行测试查询
    run_test_queries(&engine).await?;

    // 5. 显示统计信息
    let metrics = engine.get_metrics().await;
    info!("\n📊 Search Metrics:");
    info!("  Total Queries: {}", metrics.total_queries);
    info!("  Avg Latency: {:.2}ms", metrics.avg_latency_ms);
    info!("  P99 Latency: {}ms", metrics.p99_latency_ms);
    info!("  Queries by Type: {:?}", metrics.queries_by_type);

    info!("\n✨ Demo completed successfully!");

    Ok(())
}

async fn insert_test_data(_store: &LibSQLFTS5Store) -> anyhow::Result<()> {
    // 这里应该调用store的insert方法
    // 由于我们使用的是内存数据库，这里只是演示
    info!("  ℹ️  Note: Using in-memory database for demo");
    info!("  ℹ️  In production, insert actual memory records here");
    Ok(())
}

async fn run_test_queries(engine: &EnhancedHybridSearchEngineV2) -> anyhow::Result<()> {
    let test_queries = vec![
        ("P000001", "Exact ID Query"),
        ("Apple", "Short Keyword Query"),
        ("推荐一款手机", "Natural Language Query (Chinese)"),
        (
            "What is artificial intelligence?",
            "Semantic Query (English)",
        ),
        ("iPhone 15 Pro Max", "Product Query"),
    ];

    for (query, description) in test_queries {
        info!("\n🔍 Testing: {} - \"{}\"", description, query);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let result = engine.search(query, 5).await?;

        info!("  Query Type: {:?}", result.query_type);
        info!("  Strategy:");
        info!("    - Use Vector: {}", result.strategy.use_vector);
        info!("    - Use BM25: {}", result.strategy.use_bm25);
        info!("    - Use Exact Match: {}", result.strategy.use_exact_match);
        info!(
            "    - Weights: Vector={:.1}, BM25={:.1}",
            result.strategy.vector_weight, result.strategy.bm25_weight
        );
        info!("    - Threshold: {:.3}", result.strategy.threshold);

        info!("  Stats:");
        info!("    - Total Time: {}ms", result.stats.total_time_ms);
        info!(
            "    - Classification Time: {}ms",
            result.stats.classification_time_ms
        );
        info!(
            "    - Vector Search Time: {}ms",
            result.stats.vector_search_time_ms
        );
        info!(
            "    - BM25 Search Time: {}ms",
            result.stats.bm25_search_time_ms
        );
        info!(
            "    - Exact Match Time: {}ms",
            result.stats.exact_match_time_ms
        );
        info!("    - Fusion Time: {}ms", result.stats.fusion_time_ms);

        info!("  Results: {} found", result.results.len());
        for (i, item) in result.results.iter().enumerate().take(3) {
            info!("    {}. [Score: {:.3}] {}", i + 1, item.score, item.content);
            if let Some(vs) = item.vector_score {
                info!("       Vector Score: {:.3}", vs);
            }
            if let Some(fs) = item.fulltext_score {
                info!("       BM25 Score: {:.3}", fs);
            }
        }

        if result.results.is_empty() {
            warn!("    ⚠️  No results found!");
        }
    }

    Ok(())
}
