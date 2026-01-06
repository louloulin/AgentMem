//! 混合搜索集成测试
//!
//! 测试向量搜索 + 全文搜索的混合搜索功能

#![cfg(feature = "postgres")]

use agent_mem_core::search::{HybridSearchConfig, SearchFilters, SearchQuery};

/// 测试混合搜索配置
#[test]
fn test_hybrid_search_config() {
    let config = HybridSearchConfig::default();
    assert_eq!(config.vector_weight, 0.7);
    assert_eq!(config.fulltext_weight, 0.3);
    assert_eq!(config.rrf_k, 60.0);
    assert!(config.enable_parallel);
    assert!(!config.enable_cache);
}

/// 测试自定义混合搜索配置
#[test]
fn test_custom_hybrid_search_config() {
    let config = HybridSearchConfig {
        vector_weight: 0.5,
        fulltext_weight: 0.5,
        rrf_k: 80.0,
        enable_parallel: false,
        enable_cache: true,
    };

    assert_eq!(config.vector_weight, 0.5);
    assert_eq!(config.fulltext_weight, 0.5);
    assert_eq!(config.rrf_k, 80.0);
    assert!(!config.enable_parallel);
    assert!(config.enable_cache);
}

/// 测试搜索查询构建
#[test]
fn test_search_query_builder() {
    let query = SearchQuery {
        query: "test query".to_string(),
        limit: 20,
        threshold: Some(0.8),
        vector_weight: 0.6,
        fulltext_weight: 0.4,
        filters: None,
    };

    assert_eq!(query.query, "test query");
    assert_eq!(query.limit, 20);
    assert_eq!(query.threshold, Some(0.8));
    assert_eq!(query.vector_weight, 0.6);
    assert_eq!(query.fulltext_weight, 0.4);
}

/// 测试搜索过滤器
#[test]
fn test_search_filters() {
    let filters = SearchFilters {
        user_id: Some("user456".to_string()),
        organization_id: Some("org789".to_string()),
        agent_id: Some("agent123".to_string()),
        start_time: None,
        end_time: None,
        tags: Some(vec!["important".to_string(), "urgent".to_string()]),
    };

    assert_eq!(filters.user_id, Some("user456".to_string()));
    assert_eq!(filters.organization_id, Some("org789".to_string()));
    assert_eq!(filters.agent_id, Some("agent123".to_string()));
    assert_eq!(
        filters.tags,
        Some(vec!["important".to_string(), "urgent".to_string()])
    );
}

/// 测试权重归一化
#[test]
fn test_weight_normalization() {
    let config = HybridSearchConfig {
        vector_weight: 0.7,
        fulltext_weight: 0.3,
        rrf_k: 60.0,
        enable_parallel: true,
        enable_cache: false,
    };

    let total = config.vector_weight + config.fulltext_weight;
    assert!((total - 1.0).abs() < 0.001, "Weights should sum to 1.0");
}

/// 测试RRF常数
#[test]
fn test_rrf_constant() {
    let config = HybridSearchConfig::default();
    assert_eq!(config.rrf_k, 60.0, "Default RRF k should be 60.0");

    let custom_config = HybridSearchConfig {
        rrf_k: 80.0,
        ..Default::default()
    };
    assert_eq!(custom_config.rrf_k, 80.0);
}
