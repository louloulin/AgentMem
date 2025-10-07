//! Memory managers module
//!
//! 专门的记忆管理器，实现不同类型记忆的特化管理

pub mod association_manager;
pub mod contextual_memory;
pub mod core_memory;
pub mod deduplication;
pub mod episodic_memory;
pub mod knowledge_graph_manager;
pub mod knowledge_vault;
pub mod lifecycle_manager;
pub mod procedural_memory;
pub mod resource_memory;
pub mod semantic_memory;

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

pub use episodic_memory::{EpisodicEvent, EpisodicMemoryManager, EpisodicQuery};

pub use semantic_memory::{SemanticMemoryItem, SemanticMemoryManager, SemanticQuery};

pub use procedural_memory::{ProceduralMemoryItem, ProceduralMemoryManager, ProceduralQuery};

pub use lifecycle_manager::{
    LifecycleEvent, LifecycleEventType, LifecycleManager, LifecycleManagerConfig, MemoryState,
    MemoryStateRecord,
};

pub use deduplication::{
    DeduplicationConfig, DeduplicationResult, DuplicateGroup, MemoryDeduplicator, MergeStrategy,
};

pub use association_manager::{
    AssociationManager, AssociationManagerConfig, AssociationStats, AssociationType,
    MemoryAssociation, TypeCount,
};

pub use knowledge_graph_manager::{
    Entity, EntityType, GraphPath, GraphQueryResult, KnowledgeGraphConfig,
    KnowledgeGraphManager, Relation, RelationType,
};
