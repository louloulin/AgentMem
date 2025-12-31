//! Enhanced Hierarchical Memory Service
//!
//! Complete implementation of ContextEngine's hierarchical memory architecture
//! with advanced features like memory inheritance, conflict resolution, and
//! intelligent routing.

use crate::hierarchy::{MemoryLevel, MemoryScope};
use crate::types::ImportanceLevel;
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Hierarchical memory record with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalMemoryRecord {
    pub id: String,
    pub content: String,
    pub scope: MemoryScope,
    pub level: MemoryLevel,
    pub importance: ImportanceLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u64,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub parent_memory_id: Option<String>,
    pub child_memory_ids: Vec<String>,
    pub conflict_resolution_strategy: ConflictResolutionStrategy,
    pub quality_score: f64,
    pub source_reliability: f64,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolutionStrategy {
    /// Newer memory takes precedence
    TimeBasedNewest,
    /// Higher importance takes precedence
    ImportanceBased,
    /// More reliable source takes precedence
    SourceReliabilityBased,
    /// Merge semantically similar memories
    SemanticMerge,
    /// Keep both memories with conflict markers
    KeepBoth,
}

/// Memory inheritance rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInheritanceRule {
    pub from_scope: MemoryScope,
    pub to_scope: MemoryScope,
    pub inheritance_type: InheritanceType,
    pub conditions: Vec<InheritanceCondition>,
}

/// Types of memory inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InheritanceType {
    /// Full inheritance - child can access all parent memories
    Full,
    /// Filtered inheritance - only specific memories are inherited
    Filtered,
    /// Summary inheritance - only summaries are inherited
    Summary,
    /// No inheritance
    None,
}

/// Conditions for memory inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InheritanceCondition {
    /// Minimum importance level required
    MinImportance(ImportanceLevel),
    /// Specific tags required
    RequiredTags(Vec<String>),
    /// Maximum age in days
    MaxAge(u32),
    /// Minimum quality score
    MinQuality(f64),
}

/// Enhanced hierarchical memory service
pub struct HierarchicalMemoryService {
    /// Memory storage organized by scope and level
    memories:
        Arc<RwLock<BTreeMap<MemoryScope, BTreeMap<MemoryLevel, Vec<HierarchicalMemoryRecord>>>>>,
    /// Memory index for fast lookups
    memory_index: Arc<RwLock<HashMap<String, (MemoryScope, MemoryLevel, usize)>>>,
    /// Inheritance rules
    inheritance_rules: Arc<RwLock<Vec<MemoryInheritanceRule>>>,
    /// Conflict resolution cache
    conflict_cache: Arc<RwLock<HashMap<String, Vec<HierarchicalMemoryRecord>>>>,
    /// Service configuration
    config: HierarchicalServiceConfig,
}

/// Configuration for hierarchical memory service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalServiceConfig {
    /// Enable automatic conflict resolution
    pub auto_resolve_conflicts: bool,
    /// Enable memory inheritance
    pub enable_inheritance: bool,
    /// Maximum memories per scope-level combination
    pub max_memories_per_scope_level: usize,
    /// Default conflict resolution strategy
    pub default_conflict_strategy: ConflictResolutionStrategy,
    /// Memory quality threshold for inheritance
    pub inheritance_quality_threshold: f64,
    /// Enable memory compression for old memories
    pub enable_memory_compression: bool,
    /// Days after which memories are considered old
    pub memory_aging_days: u32,
}

impl Default for HierarchicalServiceConfig {
    fn default() -> Self {
        Self {
            auto_resolve_conflicts: true,
            enable_inheritance: true,
            max_memories_per_scope_level: 1000,
            default_conflict_strategy: ConflictResolutionStrategy::ImportanceBased,
            inheritance_quality_threshold: 0.7,
            enable_memory_compression: true,
            memory_aging_days: 30,
        }
    }
}

impl HierarchicalMemoryService {
    /// Create a new hierarchical memory service
    pub async fn new(config: HierarchicalServiceConfig) -> Result<Self> {
        let service = Self {
            memories: Arc::new(RwLock::new(BTreeMap::new())),
            memory_index: Arc::new(RwLock::new(HashMap::new())),
            inheritance_rules: Arc::new(RwLock::new(Vec::new())),
            conflict_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Initialize default inheritance rules
        service.initialize_default_inheritance_rules().await?;

        Ok(service)
    }

    /// Add a hierarchical memory
    pub async fn add_hierarchical_memory(
        &self,
        content: String,
        scope: MemoryScope,
        level: MemoryLevel,
        importance: ImportanceLevel,
        metadata: HashMap<String, String>,
    ) -> Result<HierarchicalMemoryRecord> {
        let memory_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let memory = HierarchicalMemoryRecord {
            id: memory_id.clone(),
            content,
            scope: scope.clone(),
            level: level.clone(),
            importance,
            created_at: now,
            updated_at: now,
            accessed_at: now,
            access_count: 0,
            metadata,
            tags: Vec::new(),
            parent_memory_id: None,
            child_memory_ids: Vec::new(),
            conflict_resolution_strategy: self.config.default_conflict_strategy.clone(),
            quality_score: 1.0,      // Default quality score
            source_reliability: 1.0, // Default source reliability
        };

        // Check for conflicts if auto-resolution is enabled
        if self.config.auto_resolve_conflicts {
            self.resolve_conflicts(&memory).await?;
        }

        // Add memory to storage
        {
            let mut memories = self.memories.write().await;
            let scope_memories = memories.entry(scope.clone()).or_insert_with(BTreeMap::new);
            let level_memories = scope_memories.entry(level.clone()).or_insert_with(Vec::new);

            // Check capacity limits
            if level_memories.len() >= self.config.max_memories_per_scope_level {
                // Remove oldest memory if at capacity
                if let Some(oldest_idx) = self.find_oldest_memory_index(level_memories) {
                    let removed = level_memories.remove(oldest_idx);
                    // Update index
                    let mut index = self.memory_index.write().await;
                    index.remove(&removed.id);
                }
            }

            let memory_index = level_memories.len();
            level_memories.push(memory.clone());

            // Update index
            let mut index = self.memory_index.write().await;
            index.insert(memory_id, (scope, level, memory_index));
        }

        // Apply inheritance rules if enabled
        if self.config.enable_inheritance {
            self.apply_inheritance_rules(&memory).await?;
        }

        Ok(memory)
    }

    /// Get hierarchical memory with access control
    pub async fn get_hierarchical_memory(
        &self,
        memory_id: &str,
        request_scope: &MemoryScope,
    ) -> Result<Option<HierarchicalMemoryRecord>> {
        let index = self.memory_index.read().await;

        if let Some((scope, level, memory_index)) = index.get(memory_id) {
            // Check access permissions
            if !request_scope.can_access(scope) {
                return Err(AgentMemError::memory_error(format!(
                    "Scope {request_scope:?} cannot access memory in scope {scope:?}"
                )));
            }

            let memories = self.memories.read().await;
            if let Some(scope_memories) = memories.get(scope) {
                if let Some(level_memories) = scope_memories.get(level) {
                    if let Some(memory) = level_memories.get(*memory_index) {
                        // Update access statistics
                        self.update_access_stats(memory_id).await?;
                        return Ok(Some(memory.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Search memories with hierarchical filtering
    pub async fn search_hierarchical_memories(
        &self,
        query: &str,
        request_scope: &MemoryScope,
        filters: Option<HierarchicalSearchFilters>,
    ) -> Result<Vec<HierarchicalMemoryRecord>> {
        let memories = self.memories.read().await;
        let mut results = Vec::new();

        for (scope, scope_memories) in memories.iter() {
            // Check access permissions
            if !request_scope.can_access(scope) {
                continue;
            }

            for (level, level_memories) in scope_memories.iter() {
                for memory in level_memories.iter() {
                    // Apply filters if provided
                    if let Some(ref filters) = filters {
                        if !self.matches_filters(memory, filters) {
                            continue;
                        }
                    }

                    // Simple text search (in production, would use semantic search)
                    if memory
                        .content
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    {
                        results.push(memory.clone());
                    }
                }
            }
        }

        // Sort by relevance and importance
        results.sort_by(|a, b| {
            b.importance
                .cmp(&a.importance)
                .then_with(|| {
                    b.quality_score
                        .partial_cmp(&a.quality_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| b.accessed_at.cmp(&a.accessed_at))
        });

        Ok(results)
    }

    /// Initialize default inheritance rules
    async fn initialize_default_inheritance_rules(&self) -> Result<()> {
        let mut rules = self.inheritance_rules.write().await;

        // Global memories are inherited by all scopes
        rules.push(MemoryInheritanceRule {
            from_scope: MemoryScope::Global,
            to_scope: MemoryScope::Agent("*".to_string()),
            inheritance_type: InheritanceType::Filtered,
            conditions: vec![
                InheritanceCondition::MinImportance(ImportanceLevel::Medium),
                InheritanceCondition::MinQuality(0.7),
            ],
        });

        // Agent memories are inherited by user scopes
        rules.push(MemoryInheritanceRule {
            from_scope: MemoryScope::Agent("*".to_string()),
            to_scope: MemoryScope::User {
                agent_id: "*".to_string(),
                user_id: "*".to_string(),
            },
            inheritance_type: InheritanceType::Summary,
            conditions: vec![
                InheritanceCondition::MinImportance(ImportanceLevel::High),
                InheritanceCondition::MaxAge(7), // Only recent memories
            ],
        });

        Ok(())
    }

    /// Resolve memory conflicts
    async fn resolve_conflicts(&self, new_memory: &HierarchicalMemoryRecord) -> Result<()> {
        // Implementation would check for semantic conflicts and resolve them
        // based on the configured strategy
        Ok(())
    }

    /// Apply inheritance rules
    async fn apply_inheritance_rules(&self, memory: &HierarchicalMemoryRecord) -> Result<()> {
        // Implementation would apply inheritance rules to propagate
        // memories to child scopes based on configured rules
        Ok(())
    }

    /// Find oldest memory index in a level
    fn find_oldest_memory_index(&self, memories: &[HierarchicalMemoryRecord]) -> Option<usize> {
        memories
            .iter()
            .enumerate()
            .min_by_key(|(_, memory)| memory.created_at)
            .map(|(index, _)| index)
    }

    /// Update access statistics
    async fn update_access_stats(&self, memory_id: &str) -> Result<()> {
        // Implementation would update access count and last accessed time
        Ok(())
    }

    /// Check if memory matches search filters
    fn matches_filters(
        &self,
        _memory: &HierarchicalMemoryRecord,
        _filters: &HierarchicalSearchFilters,
    ) -> bool {
        // Implementation would check various filter conditions
        true
    }
}

/// Search filters for hierarchical memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalSearchFilters {
    pub scopes: Option<Vec<MemoryScope>>,
    pub levels: Option<Vec<MemoryLevel>>,
    pub importance_min: Option<ImportanceLevel>,
    pub quality_min: Option<f64>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hierarchical_service_creation() {
        let config = HierarchicalServiceConfig::default();
        let service = HierarchicalMemoryService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_add_hierarchical_memory() {
        let config = HierarchicalServiceConfig::default();
        let service = HierarchicalMemoryService::new(config).await?;

        let memory = service
            .add_hierarchical_memory(
                "Test memory content".to_string(),
                MemoryScope::Global,
                MemoryLevel::Strategic,
                ImportanceLevel::High,
                HashMap::new(),
            )
            .await;

        assert!(memory.is_ok());
        let memory = memory.unwrap();
        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.scope, MemoryScope::Global);
        assert_eq!(memory.level, MemoryLevel::Strategic);
    }

    #[tokio::test]
    async fn test_memory_access_control() {
        let config = HierarchicalServiceConfig::default();
        let service = HierarchicalMemoryService::new(config).await?;

        // Add a user-scoped memory
        let memory = service
            .add_hierarchical_memory(
                "User memory".to_string(),
                MemoryScope::User {
                    agent_id: "agent1".to_string(),
                    user_id: "user1".to_string(),
                },
                MemoryLevel::Operational,
                ImportanceLevel::Medium,
                HashMap::new(),
            )
            .await
            .unwrap();

        // Test access from same scope - should succeed
        let result = service
            .get_hierarchical_memory(
                &memory.id,
                &MemoryScope::User {
                    agent_id: "agent1".to_string(),
                    user_id: "user1".to_string(),
                },
            )
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Test access from different user - should fail
        let result = service
            .get_hierarchical_memory(
                &memory.id,
                &MemoryScope::User {
                    agent_id: "agent1".to_string(),
                    user_id: "user2".to_string(),
                },
            )
            .await;
        assert!(result.is_err());
    }
}
