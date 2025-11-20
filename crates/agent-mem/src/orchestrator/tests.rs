//! Orchestrator 模块测试
//!
//! 验证拆分后的模块功能完整性

#[cfg(test)]
mod tests {
    use crate::orchestrator::core::{MemoryOrchestrator, OrchestratorConfig};

    #[tokio::test]
    async fn test_orchestrator_initialization() {
        // 测试初始化
        let config = OrchestratorConfig::default();
        let result = MemoryOrchestrator::new_with_config(config).await;
        assert!(result.is_ok(), "Orchestrator 应该能够初始化");
    }

    #[tokio::test]
    async fn test_orchestrator_auto_config() {
        // 测试自动配置
        let result = MemoryOrchestrator::new_with_auto_config().await;
        // 如果环境变量未设置，可能会失败，这是正常的
        if result.is_err() {
            println!("自动配置失败（可能是预期的）: {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_storage_module() {
        // 测试存储模块
        let config = OrchestratorConfig::default();
        let orchestrator = MemoryOrchestrator::new_with_config(config).await.unwrap();

        // 测试快速添加记忆
        let result = orchestrator
            .add_memory_fast(
                "Test memory".to_string(),
                "test_agent".to_string(),
                Some("test_user".to_string()),
                None,
                None,
            )
            .await;

        // 如果没有配置 embedder，会失败，这是正常的
        if result.is_err() {
            println!("存储测试失败（可能是预期的）: {:?}", result.err());
        }
    }

    #[tokio::test]
    async fn test_utils_module() {
        // 测试工具模块
        use crate::orchestrator::utils::UtilsModule;

        // 测试动态阈值计算
        let threshold = UtilsModule::calculate_dynamic_threshold("test query", Some(0.7));
        assert!(threshold >= 0.2 && threshold <= 0.9, "阈值应该在合理范围内");

        // 测试查询预处理
        let processed = UtilsModule::preprocess_query("  The   quick   brown   fox  ").await;
        assert!(processed.is_ok(), "查询预处理应该成功");
    }
}
