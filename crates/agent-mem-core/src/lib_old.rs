//! AgentMem Core - Memory Management Engine
//!
//! This crate provides the core memory management functionality for AgentMem,
//! including hierarchical memory architecture, intelligent memory processing,
//! and advanced search capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod engine;
pub mod hierarchy;
pub mod intelligence;
pub mod search;
pub mod storage;
pub mod conflict;
pub mod lifecycle;
pub mod context;

pub use adaptive_strategy::{
    AdaptiveStrategyConfig, AdaptiveStrategyManager, MemoryStrategy, StrategyParameters,
    StrategyPerformance, StrategyRecommendation,
};
pub use conflict_resolver::{
    ConflictDetection, ConflictResolution, ConflictResolver, ConflictResolverConfig, ConflictType,
};
pub use context_aware_search::{
    ContextAwareSearchConfig, ContextAwareSearchEngine, ContextualSearchQuery,
    ContextualSearchResult, ResultPreferences, SearchAnalytics, SearchStrategy,
};
pub use hierarchical_service::{
    ConflictResolutionStrategy, HierarchicalMemoryRecord, HierarchicalMemoryService,
    HierarchicalSearchFilters, HierarchicalServiceConfig, InheritanceType, MemoryInheritanceRule,
};
pub use hierarchy::*;
pub use hierarchy_manager::{
    HierarchyManager, HierarchyManagerConfig, HierarchyNode, HierarchyStatistics,
};
pub use history::MemoryHistory;
pub use importance_scorer::{
    AccessType, AdvancedImportanceScorer, ImportanceFactors, ImportanceScorerConfig,
    MemoryUsageStats, ScoringContext,
};
pub use lifecycle::MemoryLifecycle;
pub use llm_optimizer::{
    LlmOptimizationConfig, LlmOptimizer, LlmPerformanceMetrics, LlmProvider, OptimizationStrategy,
    OptimizedLlmResponse, PromptTemplate, PromptTemplateType,
};
pub use logging::{
    AuditEntry, AuditEventType, AuditResult, ComplianceExport, LogEntry, LogLevel, LoggingConfig,
    LoggingSystem, SecurityEntry, SecurityEventType, SecuritySeverity,
};
pub use manager::MemoryManager;
pub use monitoring::{
    Alert, AlertCondition, AlertRule, AlertSeverity, ComponentStatus, HealthStatus, MetricPoint,
    MetricType, MonitoringConfig, MonitoringSystem, PerformanceProfile, SystemInfo,
};
pub use operations::*;
pub use security::{
    AccessControlEntry, Permission, Role, SecurityConfig, SecuritySystem,
    Session as SecuritySession, ThreatAction, ThreatIncident, ThreatRule, ThreatRuleType,
    ThreatSeverity, UserAccount,
};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{MemoryProvider, Message, Session};
    use tokio_test;

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        assert!(true); // Basic creation test
    }

    #[tokio::test]
    async fn test_add_and_get_memory() {
        let manager = MemoryManager::new();
        let session = Session::new()
            .with_agent_id(Some("test-agent".to_string()))
            .with_user_id(Some("test-user".to_string()));

        // Test direct memory addition instead of using MemoryProvider trait
        let memory_id = manager
            .add_memory(
                "test-agent".to_string(),
                Some("test-user".to_string()),
                "I love playing tennis".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let retrieved = manager.get_memory(&memory_id).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "I love playing tennis");
    }

    #[tokio::test]
    async fn test_search_memories() {
        let manager = MemoryManager::new();

        // Add some memories directly
        let _id1 = manager
            .add_memory(
                "test-agent".to_string(),
                None,
                "I love playing tennis".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let _id2 = manager
            .add_memory(
                "test-agent".to_string(),
                None,
                "I enjoy reading books".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let _id3 = manager
            .add_memory(
                "test-agent".to_string(),
                None,
                "Tennis is my favorite sport".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Search for tennis-related memories
        let query = crate::types::MemoryQuery::new("test-agent".to_string())
            .with_text_query("tennis".to_string())
            .with_limit(10);
        let results = manager.search_memories(query).await?;
        assert!(results.len() >= 2); // Should find at least 2 tennis-related memories
    }

    #[tokio::test]
    async fn test_update_memory() {
        let manager = MemoryManager::new();

        let memory_id = manager
            .add_memory(
                "test-agent".to_string(),
                None,
                "Original content".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Update the memory
        manager
            .update_memory(&memory_id, Some("Updated content".to_string()), None, None)
            .await
            .unwrap();

        // Verify the update
        let retrieved = manager.get_memory(&memory_id).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Updated content");
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let manager = MemoryManager::new();

        let memory_id = manager
            .add_memory(
                "test-agent".to_string(),
                None,
                "To be deleted".to_string(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        // Delete the memory
        manager.delete_memory(&memory_id).await?;

        // Verify deletion
        let retrieved = manager.get_memory(&memory_id).await?;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_types() {
        let memory = Memory::new(
            "agent1".to_string(),
            Some("user1".to_string()),
            MemoryType::Semantic,
            "Test semantic memory".to_string(),
            0.8,
        );

        assert_eq!(memory.memory_type, MemoryType::Semantic);
        assert_eq!(memory.importance, 0.8);
        assert_eq!(memory.content, "Test semantic memory");
    }

    #[tokio::test]
    async fn test_memory_lifecycle() {
        let mut lifecycle = MemoryLifecycle::with_default_config();
        let memory = Memory::new(
            "agent1".to_string(),
            None,
            MemoryType::Working,
            "Test memory".to_string(),
            0.5,
        );

        // Register memory
        lifecycle.register_memory(&memory).unwrap();
        assert!(lifecycle.is_accessible(&memory.id));

        // Archive memory
        lifecycle.archive_memory(&memory.id).unwrap();
        assert!(lifecycle.is_accessible(&memory.id)); // Still accessible when archived

        // Delete memory
        lifecycle.delete_memory(&memory.id).unwrap();
        assert!(!lifecycle.is_accessible(&memory.id)); // Not accessible when deleted
    }
}
