//! # Agent Memory Traits
//!
//! Core traits and abstractions for the AgentMem memory platform.
//! This crate defines the fundamental interfaces that all components must implement.

pub mod batch;
pub mod cache;
pub mod embedder;
pub mod error;
pub mod intelligence;
pub mod llm;
pub mod memory;
pub mod memory_store;
pub mod session;
pub mod storage;
pub mod types;

// Re-export main traits
pub use batch::{
    AdvancedSearch, ArchiveCriteria, BatchMemoryOperations, ConfigurationProvider,
    HealthCheckProvider, MemoryLifecycle, MemoryStats, MemoryUpdate, RetryableOperations,
    TelemetryProvider,
};
pub use cache::{CacheStats, IntelligenceCache};
pub use embedder::Embedder;
pub use error::{AgentMemError, ErrorContext, ErrorSeverity, Result};
pub use intelligence::{
    DecisionEngine, ExtractedFact, FactExtractor, IntelligentMemoryProcessor,
    IntelligentProcessingResult, MemoryActionType, MemoryDecision,
};
pub use llm::{LLMProvider, ModelInfo};
pub use memory::MemoryProvider;
pub use memory_store::{
    CoreMemoryItem, CoreMemoryStore, EpisodicEvent, EpisodicMemoryStore, EpisodicQuery,
    ProceduralMemoryItem, ProceduralMemoryStore, ProceduralQuery, SemanticMemoryItem,
    SemanticMemoryStore, SemanticQuery, WorkingMemoryItem, WorkingMemoryStore,
};
pub use session::SessionManager;
pub use storage::{
    EmbeddingVectorStore, GraphResult, GraphStore, HistoryStore, KeyValueStore, LegacyVectorStore,
    VectorStore, VectorStoreStats,
};
pub use types::*;

// Re-export new Mem5 types
pub use types::{
    BatchResult, EnhancedAddRequest, EnhancedSearchRequest, FilterBuilder, HealthStatus,
    MemorySearchResult, Messages, MetadataBuilder, PerformanceReport, ProcessingOptions,
    ProcessingResult, SystemMetrics,
};
