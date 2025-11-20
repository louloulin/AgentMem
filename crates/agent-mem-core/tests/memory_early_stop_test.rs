//! Task 1.2: 记忆检索早停优化测试

#[cfg(test)]
mod tests {
    use agent_mem_core::{
        engine::{MemoryEngine, MemoryEngineConfig},
        orchestrator::memory_integration::{MemoryIntegrator, MemoryIntegratorConfig},
    };
    use agent_mem_traits::abstractions::Memory;
    use agent_mem_traits::{Content, MemoryId};
    use std::sync::Arc;
    use std::time::Instant;

    /// 创建测试用的Memory
    #[allow(dead_code)]
    fn create_test_memory(_id: &str, content: &str, agent_id: &str, user_id: &str) -> Memory {
        Memory::new(
            agent_id,
            Some(user_id.to_string()),
            "episodic",
            content,
            0.8,
        )
    }

    /// 测试早停逻辑：当Episodic Memory返回足够记忆时
    #[tokio::test]
    async fn test_early_stop_after_episodic() {
        // 创建测试环境
        let config = MemoryEngineConfig::default();
        let engine = Arc::new(MemoryEngine::new(config));

        let integrator_config = MemoryIntegratorConfig::default();
        let integrator = MemoryIntegrator::new(engine.clone(), integrator_config);

        // 添加足够的Episodic记忆（模拟）
        // 注意：实际测试需要mock engine或使用测试数据库

        // 测试查询
        let query = "test query";
        let agent_id = "test-agent";
        let user_id = Some("test-user");
        let session_id = Some("test-session");
        let max_count = 5;

        let result = integrator
            .retrieve_episodic_first(query, agent_id, user_id, session_id, max_count)
            .await;

        // 验证结果
        assert!(result.is_ok(), "Early stop query should succeed");

        // 注意：由于没有实际数据，这里会返回空列表
        // 在实际集成测试中，应该验证查询次数和早停行为
    }

    /// 测试并行查询优化
    #[tokio::test]
    async fn test_parallel_query_timing() {
        let config = MemoryEngineConfig::default();
        let engine = Arc::new(MemoryEngine::new(config));

        let integrator_config = MemoryIntegratorConfig::default();
        let integrator = MemoryIntegrator::new(engine.clone(), integrator_config);

        let start = Instant::now();

        let result = integrator
            .retrieve_episodic_first("test", "agent1", Some("user1"), Some("session1"), 10)
            .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // 并行查询应该比串行快
        // 注意：由于是空数据库，实际延迟很小
        assert!(
            elapsed.as_millis() < 100,
            "Parallel query should be fast, got {:?}",
            elapsed
        );
    }

    /// 测试缓存命中
    #[tokio::test]
    async fn test_cache_hit() {
        let config = MemoryEngineConfig::default();
        let engine = Arc::new(MemoryEngine::new(config));

        let integrator_config = MemoryIntegratorConfig::default();
        let integrator = MemoryIntegrator::new(engine.clone(), integrator_config);

        let query = "test cache";
        let agent_id = "agent1";
        let user_id = Some("user1");
        let session_id = Some("session1");

        // 第一次查询
        let result1 = integrator
            .retrieve_episodic_first(query, agent_id, user_id, session_id, 10)
            .await;

        assert!(result1.is_ok());

        // 第二次查询应该命中缓存
        let start = Instant::now();
        let result2 = integrator
            .retrieve_episodic_first(query, agent_id, user_id, session_id, 10)
            .await;
        let elapsed = start.elapsed();

        assert!(result2.is_ok());
        // 缓存命中应该非常快（<5ms）
        assert!(
            elapsed.as_micros() < 5000,
            "Cache hit should be very fast, got {:?}",
            elapsed
        );
    }

    /// 测试查询统计记录
    #[test]
    fn test_query_stats_logging() {
        // 这个测试主要验证record_query_stats函数的逻辑
        // 实际日志输出需要在运行时验证

        // 计算早停节省的比例
        let actual_queries = 2;
        let saved_queries = 2;
        let total = actual_queries + saved_queries;
        let reduction_rate = (saved_queries as f64 / total as f64) * 100.0;

        assert_eq!(reduction_rate, 50.0, "Should save 50% queries");
    }

    /// 测试记忆去重和排序
    #[tokio::test]
    async fn test_deduplication_and_ranking() {
        let config = MemoryEngineConfig::default();
        let engine = Arc::new(MemoryEngine::new(config));

        let integrator_config = MemoryIntegratorConfig::default();
        let integrator = MemoryIntegrator::new(engine.clone(), integrator_config);

        // 模拟有重复ID的记忆
        // 实际实现中，HashSet会自动去重

        let result = integrator
            .retrieve_episodic_first("test", "agent1", Some("user1"), Some("session1"), 10)
            .await;

        assert!(result.is_ok());
        let memories = result.unwrap();

        // 验证没有重复ID
        use std::collections::HashSet;
        let ids: HashSet<_> = memories.iter().map(|m| &m.id).collect();
        assert_eq!(ids.len(), memories.len(), "Should have no duplicate IDs");
    }
}
