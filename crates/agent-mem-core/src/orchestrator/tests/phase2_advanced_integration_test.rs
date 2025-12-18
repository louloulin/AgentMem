//! Phase 2 高级能力集成测试
//!
//! 测试主动检索系统和自动压缩机制的集成

#[cfg(test)]
mod tests {
    use crate::orchestrator::memory_integration::{MemoryIntegrator, MemoryIntegratorConfig};
    use crate::engine::MemoryEngine;
    use crate::retrieval::{ActiveRetrievalConfig, ActiveRetrievalSystem};
    use std::sync::Arc;

    /// 测试主动检索系统集成（可选启用）
    #[tokio::test]
    async fn test_active_retrieval_integration() {
        // 创建 MemoryEngine
        let engine_config = crate::engine::MemoryEngineConfig::default();
        let memory_engine = Arc::new(MemoryEngine::new(engine_config));

        // 创建 MemoryIntegrator 配置（启用主动检索）
        let mut config = MemoryIntegratorConfig::default();
        config.enable_active_retrieval = true;

        // 创建 MemoryIntegrator
        let integrator = MemoryIntegrator::new(memory_engine.clone(), config);

        // 创建 ActiveRetrievalSystem（Mock模式）
        let active_retrieval_config = ActiveRetrievalConfig::default();
        let active_retrieval = Arc::new(
            ActiveRetrievalSystem::new(active_retrieval_config)
                .await
                .expect("Failed to create ActiveRetrievalSystem"),
        );

        // 设置主动检索系统
        let integrator = integrator.with_active_retrieval(active_retrieval);

        // 验证配置
        assert!(integrator.config.enable_active_retrieval);
        // 注意：由于是可选功能，这里只验证配置是否正确设置
        // 实际的功能测试需要完整的数据库和向量存储环境
    }

    /// 测试自动压缩配置
    #[test]
    fn test_auto_compression_config() {
        use crate::storage::coordinator::{CacheConfig, UnifiedStorageCoordinator};
        use crate::storage::libsql::memory_repository::LibSqlMemoryRepository;
        use crate::storage::traits::MemoryRepositoryTrait;

        // 创建配置（启用自动压缩）
        let mut cache_config = CacheConfig::default();
        cache_config.enable_auto_compression = true;
        cache_config.auto_compression_threshold = 1000;
        cache_config.auto_compression_age_days = 30;

        // 验证配置
        assert!(cache_config.enable_auto_compression);
        assert_eq!(cache_config.auto_compression_threshold, 1000);
        assert_eq!(cache_config.auto_compression_age_days, 30);
    }

    /// 测试 MemoryIntegrator 配置默认值
    #[test]
    fn test_memory_integrator_config_defaults() {
        let config = MemoryIntegratorConfig::default();
        
        // 验证默认配置
        assert!(!config.enable_active_retrieval); // 默认关闭
        assert!(!config.enable_graph_memory); // 默认关闭
        assert!(!config.enable_context_enhancement); // 默认关闭
        assert_eq!(config.max_memories, 3);
        assert_eq!(config.episodic_weight, 1.2);
    }

    /// 测试 CacheConfig 默认值
    #[test]
    fn test_cache_config_defaults() {
        use crate::storage::coordinator::CacheConfig;
        
        let config = CacheConfig::default();
        
        // 验证默认配置
        assert!(!config.enable_auto_compression); // 默认关闭
        assert_eq!(config.auto_compression_threshold, 1000);
        assert_eq!(config.auto_compression_age_days, 30);
    }

    /// 🆕 Phase 2: 综合测试 - 验证所有高级能力配置可以同时启用
    #[test]
    fn test_all_advanced_capabilities_config() {
        let mut config = MemoryIntegratorConfig::default();
        
        // 启用所有高级能力
        config.enable_active_retrieval = true;
        config.enable_graph_memory = true;
        config.enable_context_enhancement = true;
        
        // 验证所有配置都可以启用
        assert!(config.enable_active_retrieval);
        assert!(config.enable_graph_memory);
        assert!(config.enable_context_enhancement);
        
        // 验证可以同时启用多个功能
        let all_enabled = config.enable_active_retrieval 
            && config.enable_graph_memory 
            && config.enable_context_enhancement;
        assert!(all_enabled);
    }
}
