//! AgentMem Core - Memory Management Engine
//!
//! This crate provides the core memory management functionality for AgentMem,
//! including hierarchical memory architecture, intelligent memory processing,
//! and advanced search capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Agent state management and lifecycle
pub mod agent_state;
/// V4 migration utilities
// pub mod v4_migration;  // 临时禁用，等核心功能完成后再修复
/// Specialized memory agents for different cognitive memory types
pub mod agents;
/// Background agent processing system
pub mod background_agent;
/// Multi-level caching system with warming strategies
pub mod cache;
pub mod client;
pub mod collaboration;
pub mod compression;
/// Configuration loader (Week 3-4: eliminate hardcoding)
pub mod config;
/// Environment-based configuration
pub mod config_env;
pub mod conflict;
pub mod context;
/// Multi-agent coordination and orchestration
pub mod coordination;
/// Core Memory system with Block management, template engine, and auto-rewriter
pub mod core_memory;
pub mod embeddings_batch;
pub mod engine;
/// Entity and relation extraction for knowledge graph construction
pub mod extraction;
/// Graph-based memory management and reasoning capabilities
pub mod graph_memory;
/// Graph optimization with compression and query optimization
pub mod graph_optimization;
/// Hierarchical memory service with enhanced features
pub mod hierarchical_service;
pub mod hierarchy;
/// Hierarchy manager for advanced memory structure management
pub mod hierarchy_manager;
pub mod history;
pub mod importance_scorer;
pub mod integration;
pub mod intelligence;
pub mod lifecycle;
pub mod manager;
/// Specialized memory managers for different memory types
pub mod managers;
/// Message queue system for agent communication
pub mod message_queue;
pub mod operations;
/// Agent orchestrator for conversation loop and memory integration
pub mod orchestrator;
/// Performance optimization system with caching, batching, and benchmarking
pub mod performance;
/// Pipeline framework for memory operations
pub mod pipeline;
/// Prompt processing and optimization
pub mod prompt;
/// Query abstraction (V4.0: replace String queries)
pub mod query;
/// Active retrieval system with topic extraction, intelligent routing, and context synthesis
pub mod retrieval;
pub mod search;
pub mod security;
/// Simplified Memory API (Mem0-style)
// simple_memory模块已删除，统一使用Memory V4架构
pub mod storage;
/// Temporal graph memory with time-aware knowledge graphs
pub mod temporal_graph;
/// Temporal reasoning engine with causal inference and multi-hop reasoning
pub mod temporal_reasoning;
pub mod tenant;
pub mod types;
/// Vector storage ecosystem with capability detection and auto-selection
pub mod vector_ecosystem;

// Re-export core types
pub use agent_state::{AgentState, AgentStateMachine};
pub use background_agent::BackgroundAgentManager;
pub use engine::{MemoryEngine, MemoryEngineConfig};
pub use extraction::{
    Entity, EntityExtractor, EntityType, ExtractionResult, Relation, RelationExtractor,
    RelationType, RuleBasedExtractor, RuleBasedRelationExtractor,
};
pub use hierarchy::{HierarchyManager, MemoryLevel};
pub use managers::{
    ActivityState, ChangeType, ContextCorrelation, ContextState, ContextualMemoryConfig,
    ContextualMemoryManager, ContextualMemoryStats, CoreMemoryBlock, CoreMemoryBlockType,
    CoreMemoryConfig, CoreMemoryManager, CoreMemoryStats, CorrelationType, DeviceInfo,
    EnvironmentChangeEvent, EnvironmentType, LocationInfo, NetworkInfo, ResourceMemoryManager,
    ResourceMetadata, ResourceStorageConfig, ResourceStorageStats, ResourceType, Season,
    TemporalInfo, TimeOfDay, UserState,
};
pub use message_queue::{AgentMessage as QueueMessage, MessageAccumulator, MessageQueue};

// Re-export coordination and agents modules
pub use agents::{
    AgentConfig, AgentStats, BaseAgent, ContextualAgent, CoreAgent, EpisodicAgent, KnowledgeAgent,
    MemoryAgent, ProceduralAgent, ResourceAgent, SemanticAgent, WorkingAgent,
};

// Re-export Query abstraction
pub use coordination::{
    AgentMessage, AgentStatus, CoordinationError, CoordinationResult, CoordinationStats,
    LoadBalancingStrategy, MessageType, MetaMemoryConfig, MetaMemoryManager, TaskRequest,
    TaskResponse,
};
pub use query::{
    AggregationOp, Constraint, Preference, Query, QueryBuilder, QueryComplexity, QueryContext,
    QueryFeatures, QueryId, QueryIntent,
};

// Re-export retrieval modules
pub use retrieval::{
    ActiveRetrievalConfig, ActiveRetrievalSystem, ConflictResolution, ContextSynthesizer,
    ContextSynthesizerConfig, ExtractedTopic, RetrievalRequest, RetrievalResponse, RetrievalRouter,
    RetrievalRouterConfig, RetrievalStats, RetrievalStrategy, RetrievedMemory, RouteDecision,
    RoutingResult, SynthesisResult, TopicCategory, TopicExtractor, TopicExtractorConfig,
    TopicHierarchy,
};

// Re-export integration modules
pub use integration::{
    ComponentHealth, HealthStatus, SystemConfig, SystemIntegrationManager, SystemState,
    SystemStatus,
};

// Re-export orchestrator modules
#[cfg(feature = "postgres")]
pub use orchestrator::{
    AgentOrchestrator, ChatRequest, ChatResponse, OrchestratorConfig, ToolCallInfo,
};

// Re-export prompt modules
pub use prompt::summarizer::{MemorySummarizer, SummarizationStrategy};

// Re-export cache modules
pub use cache::{
    Cache, CacheConfig, CacheKey, CacheLevel, CacheMetadata, CacheStats, CacheValue, CacheWarmer,
    CacheWarmingConfig, DataLoader, EvictionPolicy, InvalidationStrategy, MemoryCache,
    MemoryCacheConfig, MemoryCacheStats, MultiLevelCache, MultiLevelCacheConfig, WarmingStats,
    WarmingStrategy,
};

// Re-export from traits
// V4 Architecture: Memory now points to the new V4 abstraction
pub use agent_mem_traits::{
    AgentMemError,
    MemoryItem, // Legacy: Kept for backward compatibility
    MemoryType,
    MemoryV4 as Memory, // V4: Primary Memory type
    Result as MemoryResult,
    Session,
};

// SimpleMemory已删除，统一使用Memory V4架构
// 迁移路径：使用 agent_mem::Memory 代替 agent_mem_core::SimpleMemory

/// Core error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum CoreError {
    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Search error
    #[error("Search error: {0}")]
    Search(String),

    /// Hierarchy error
    #[error("Hierarchy error: {0}")]
    Hierarchy(String),

    /// Intelligence error
    #[error("Intelligence error: {0}")]
    Intelligence(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Core result type
pub type CoreResult<T> = Result<T, CoreError>;
