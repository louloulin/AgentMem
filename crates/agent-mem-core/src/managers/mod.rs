//! Memory managers module
//!
//! 专门的记忆管理器，实现不同类型记忆的特化管理

#[cfg(feature = "postgres")]
pub mod association_manager;
pub mod contextual_memory;
pub mod core_memory;
pub mod deduplication;
#[cfg(feature = "postgres")]
pub mod episodic_memory;
#[cfg(feature = "postgres")]
pub mod knowledge_graph_manager;
pub mod knowledge_vault;
#[cfg(feature = "postgres")]
pub mod lifecycle_manager;
#[cfg(feature = "postgres")]
pub mod procedural_memory;
pub mod resource_memory;
#[cfg(feature = "postgres")]
pub mod semantic_memory;
#[cfg(feature = "postgres")]
pub mod tool_manager;

pub use core_memory::{
    CoreMemoryBlock, CoreMemoryBlockType, CoreMemoryConfig, CoreMemoryManager, CoreMemoryStats,
};

pub use resource_memory::{
    ResourceMemoryManager, ResourceMetadata, ResourceStorageConfig, ResourceStorageStats,
    ResourceType,
};

pub use knowledge_vault::{
    AccessPermission, AuditAction, AuditLogEntry, KnowledgeEntry, KnowledgeVaultConfig,
    KnowledgeVaultManager, KnowledgeVaultStats, SensitivityLevel, UserPermissions,
};

pub use contextual_memory::{
    ActivityState, ChangeType, ContextCorrelation, ContextState, ContextualMemoryConfig,
    ContextualMemoryManager, ContextualMemoryStats, CorrelationType, DeviceInfo,
    EnvironmentChangeEvent, EnvironmentType, LocationInfo, NetworkInfo, Season, TemporalInfo,
    TimeOfDay, UserState,
};

#[cfg(feature = "postgres")]
pub use episodic_memory::{EpisodicEvent, EpisodicMemoryManager, EpisodicQuery};

#[cfg(feature = "postgres")]
pub use semantic_memory::{SemanticMemoryItem, SemanticMemoryManager, SemanticQuery};

#[cfg(feature = "postgres")]
pub use procedural_memory::{ProceduralMemoryItem, ProceduralMemoryManager, ProceduralQuery};

#[cfg(feature = "postgres")]
pub use lifecycle_manager::{
    LifecycleEvent, LifecycleEventType, LifecycleManager, LifecycleManagerConfig, MemoryState,
    MemoryStateRecord,
};

pub use deduplication::{
    DeduplicationConfig, DeduplicationResult, DuplicateGroup, MemoryDeduplicator, MergeStrategy,
};

#[cfg(feature = "postgres")]
pub use association_manager::{
    AssociationManager, AssociationManagerConfig, AssociationStats, AssociationType,
    MemoryAssociation, TypeCount,
};

#[cfg(feature = "postgres")]
pub use knowledge_graph_manager::{
    Entity, EntityType, GraphPath, GraphQueryResult, KnowledgeGraphConfig,
    KnowledgeGraphManager, Relation, RelationType,
};

#[cfg(feature = "postgres")]
pub use tool_manager::{
    CreateToolRequest, ToolManager, ToolManagerConfig, ToolStats, ToolType, UpdateToolRequest,
};
