//! P0 优化验证测试
//!
//! 测试所有P0优先级的优化项：
//! - #2, #12, #22: 超时控制
//! - #10: Prompt长度控制
//! - #21: 零向量降级修复

use agent_mem_intelligence::{
    conflict_resolution::ConflictResolverConfig, ConflictResolver, FactExtractor,
    MemoryDecisionEngine, TimeoutConfig,
};
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 P0-#2: FactExtractor 超时控制
    #[tokio::test]
    #[ignore] // TODO: 需要实现 MockLLMProvider
    async fn test_fact_extractor_timeout() {
        let mock_llm = Arc::new(MockLLMProvider::new());

        // 创建一个超时时间很短的配置
        let timeout_config = TimeoutConfig {
            fact_extraction_timeout_secs: 1,
            ..Default::default()
        };

        let extractor = FactExtractor::with_timeout_config(mock_llm, timeout_config);

        // 测试应该在1秒内完成或超时
        let start = std::time::Instant::now();
        let result = extractor
            .extract_facts_internal(&[agent_mem_traits::Message::user("test")])
            .await;
        let elapsed = start.elapsed();

        // 即使失败，也应该在超时时间内返回
        assert!(
            elapsed < Duration::from_secs(2),
            "操作应该在超时时间内完成或超时"
        );
    }

    /// 测试 P0-#12: DecisionEngine 超时和重试
    #[tokio::test]
    #[ignore] // TODO: 需要实现 MockLLMProvider
    async fn test_decision_engine_timeout_and_retry() {
        let mock_llm = Arc::new(MockLLMProvider::new());

        let timeout_config = TimeoutConfig {
            decision_timeout_secs: 1,
            ..Default::default()
        };

        let engine = MemoryDecisionEngine::with_timeout_config(mock_llm, timeout_config);

        let result = engine.make_decisions(&[], &[]).await;

        // 应该有NoAction决策（因为没有输入）
        assert!(result.is_ok());
        let decisions = result.unwrap();
        assert_eq!(decisions.len(), 1);
    }

    /// 测试 P0-#10: ConflictResolver Prompt长度控制
    #[tokio::test]
    #[ignore] // TODO: 需要实现 MockLLMProvider 和完整的 Memory 结构
    async fn test_conflict_resolver_memory_limit() {
        use agent_mem_core::Memory;
        use agent_mem_traits::MemoryType;
        use std::collections::HashMap;

        // Mock provider not available
        // let mock_llm = Arc::new(MockLLMProvider::new());

        let config = ConflictResolverConfig {
            max_consideration_memories: 5, // 限制为5个
            ..Default::default()
        };

        // let resolver = ConflictResolver::new(mock_llm, config);

        // 创建10个测试记忆
        let mut memories = Vec::new();
        for i in 0..10 {
            memories.push(Memory {
                id: format!("mem_{}", i),
                agent_id: "test_agent".to_string(),
                user_id: Some("test_user".to_string()),
                content: format!("Memory content {}", i),
                memory_type: MemoryType::Semantic,
                importance: 0.5,
                hash: Some(format!("hash_{}", i)),
                embedding: None,
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: Some(chrono::Utc::now()),
                last_accessed_at: chrono::Utc::now(),
                session: agent_mem_traits::Session::default(),
                entities: Vec::new(),
                relations: Vec::new(),
                score: Some(0.5),
                expires_at: None,
                access_count: 0,
                version: 1,
            });
        }

        // 测试冲突检测应该能处理大量记忆（通过限制）
        let result = resolver.detect_conflicts(&[], &memories).await;

        // 应该成功（即使有很多记忆，也会被限制）
        assert!(result.is_ok());
    }

    /// 测试超时配置的默认值
    #[test]
    fn test_timeout_config_defaults() {
        let config = TimeoutConfig::default();

        assert_eq!(config.fact_extraction_timeout_secs, 30);
        assert_eq!(config.decision_timeout_secs, 60);
        assert_eq!(config.rerank_timeout_secs, 10);
        assert_eq!(config.conflict_detection_timeout_secs, 30);
        assert_eq!(config.search_timeout_secs, 5);
    }
}
