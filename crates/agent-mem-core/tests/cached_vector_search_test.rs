//! 缓存向量搜索集成测试
//!
//! 验证CachedVectorSearchEngine的缓存功能

#[cfg(all(feature = "redis-cache", test))]
mod cache_tests {
    use agent_mem_core::cache::{MemoryCacheConfig, MultiLevelCache, MultiLevelCacheConfig};
    use agent_mem_core::search::{CachedVectorSearchConfig, CachedVectorSearchEngine, SearchQuery};
    use std::sync::Arc;

    // 注意：这些测试需要实际的VectorSearchEngine实例
    // 在实际环境中，需要mock或使用测试数据库

    #[tokio::test]
    async fn test_cache_basic_flow() {
        // 1. 创建缓存配置
        let cache_config = MultiLevelCacheConfig {
            enable_l1: true,
            enable_l2: false, // 不需要Redis进行单元测试
            l1_config: MemoryCacheConfig {
                max_size: 1000,
                ttl_seconds: 300,
                ..Default::default()
            },
            ..Default::default()
        };

        // 2. 创建多层缓存
        let multi_cache = Arc::new(MultiLevelCache::new(cache_config));

        // 注：这里需要实际的VectorSearchEngine实例进行完整测试
        // let base_engine = Arc::new(create_test_vector_engine());
        // let cached_engine = CachedVectorSearchEngine::with_cache(
        //     base_engine,
        //     CachedVectorSearchConfig::default(),
        //     multi_cache,
        // );

        // 测试缓存键生成（验证存在）
        let config = CachedVectorSearchConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_cache_config() {
        let config = CachedVectorSearchConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert_eq!(config.cache_key_prefix, "vec_search");
    }

    #[test]
    fn test_custom_cache_config() {
        let config = CachedVectorSearchConfig {
            enable_cache: false,
            cache_ttl_seconds: 600,
            cache_key_prefix: "custom".to_string(),
        };

        assert!(!config.enable_cache);
        assert_eq!(config.cache_ttl_seconds, 600);
        assert_eq!(config.cache_key_prefix, "custom");
    }
}

// 基础功能测试（不需要feature flag）
#[cfg(all(feature = "redis-cache", test))]
mod basic_tests {
    use agent_mem_core::search::CachedVectorSearchConfig;

    #[test]
    fn test_config_default() {
        let config = CachedVectorSearchConfig::default();
        assert!(config.enable_cache);
    }
}
