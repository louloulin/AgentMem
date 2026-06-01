//! AgentMem Cognitive Memory Module
//! 
//! Implements the 8 types of cognitive memories for AgentMem with production-ready features.
//! 
//! Features:
//! - 8 types of cognitive memories (Episodic, Semantic, Procedural, Working, Core, Resource, Knowledge, Contextual)
//! - Memory hierarchy (层级记忆管理)
//! - Smart tiering (智能分层器)
//! - Archive memory manager (归档记忆管理)
//! - Intelligent review trigger (智能复习触发)
//! - Unified memory manager (统一记忆管理)
//! - Async support (异步支持)
//! - Error handling (错误处理)
//! - Storage backend (持久化存储)
//! - Configuration management (配置管理)
//! - Metrics (监控指标)

mod episodic;
mod semantic;
mod procedural;
mod working;
mod core;
mod resource;
mod knowledge;
mod contextual;
mod types;
mod forgetting;
mod consolidation;
mod hierarchy;
mod tiering;
mod archive;
mod review;
mod unified;
mod error;
mod async_unified;
mod storage;
mod config;
mod metrics;
mod lru;

pub use types::*;
pub use episodic::*;
pub use semantic::*;
pub use procedural::*;
pub use working::*;
pub use core::*;
pub use resource::*;
pub use knowledge::*;
pub use contextual::*;
pub use forgetting::{ForgettingCurve, DecayStatus};
pub use consolidation::{ConsolidationEngine, MemoryFusion};
pub use hierarchy::{MemoryTier, TieredMemoryItem, MemoryHierarchy, MemoryHierarchyStats};
pub use tiering::{SmartTiering, TieringConfig};
pub use archive::{ArchiveMemoryManager, ArchiveConfig, ArchiveStats, ArchivedItem};
pub use review::{ReviewTriggerManager, ReviewConfig, ReviewStats, ReviewTrigger, ReviewPriority};
pub use unified::{UnifiedMemoryManager, UnifiedConfig, UnifiedStats, SearchResult};
pub use error::{MemoryError, Result};
pub use async_unified::{AsyncUnifiedMemoryManager, AsyncUnifiedConfig, AsyncUnifiedStats, AsyncSearchResult};
pub use storage::{
    StorageBackend, InMemoryStorage, FileStorage, 
    StorageManager, InMemoryStorageManager, FileStorageManager,
    StoredMemory
};
pub use config::{ConfigManager, ConfigValidationError};
pub use metrics::{MemoryMetrics, MetricsCollector, OperationTimer};
pub use lru::{LruCache, LruTier};
mod integration;
mod resilience;
pub use resilience::{CircuitBreaker, RateLimiter, CircuitState};
