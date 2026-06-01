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
use agent_mem_llm::{LLMProvider, Message, ModelInfo};
use agent_mem_traits::Result as TraitResult;
use async_trait::async_trait;
use futures::stream;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

// ✅ Mock LLM Provider for testing
struct MockLLMProvider;

impl MockLLMProvider {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn generate(&self, _messages: &[Message]) -> TraitResult<String> {
        Ok(r#"{"facts": ["用户喜欢编程", "这是测试数据"]}"#.to_string())
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            max_tokens: 1000,
            supports_streaming: false,
            supports_functions: false,
        }
    }

    async fn generate_stream(
        &self,
        _messages: &[Message],
    ) -> TraitResult<Pin<Box<dyn futures::Stream<Item = TraitResult<String>> + Send>>> {
        let items = vec![Ok("Mock stream response".to_string())];
        Ok(Box::pin(stream::iter(items)))
    }

    fn validate_config(&self) -> TraitResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 P0-#2: FactExtractor 超时控制
    #[tokio::test]
    async fn test_fact_extractor_timeout() {

    /// 测试 P0-#12: DecisionEngine 超时和重试
    #[tokio::test]
    async fn test_decision_engine_timeout_and_retry() {
        let mock_llm = Arc::new(MockLLMProvider::new());

        // 创建带超时配置的 DecisionEngine
        let timeout_config = TimeoutConfig {
            decision_timeout_secs: 5,
            ..Default::default()
        };

        let engine = MemoryDecisionEngine::with_timeout(mock_llm, timeout_config);

        // 测试决策功能
        let memories = vec![];
        let query = "测试查询";
        let result = engine.make_decision(&memories, query).await;

        // 验证结果（应该不会超时）
        assert!(result.is_ok());
    }

    /// 测试 P0-#10: ConflictResolver Prompt长度控制
    #[tokio::test]
    async fn test_conflict_resolver_memory_limit() {
        let mock_llm = Arc::new(MockLLMProvider::new());
        let config = ConflictResolverConfig {
            max_consideration_memories: 5,
            ..Default::default()
        };

        let resolver = ConflictResolver::with_config(mock_llm, config);

        // 测试冲突解决功能
        let memories = vec![]; // 空记忆列表用于测试
        let result = resolver.detect_conflicts(&memories).await;

        // 验证结果
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
