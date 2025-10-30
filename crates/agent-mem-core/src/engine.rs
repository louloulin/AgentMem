//! Memory Engine - Core orchestration and management

use crate::{hierarchy::MemoryScope, Memory};
use crate::{
    hierarchy::{DefaultHierarchyManager, HierarchyConfig, HierarchyManager, MemoryLevel},
    intelligence::{
        ConflictResolver, DefaultConflictResolver, DefaultImportanceScorer, ImportanceScorer,
        IntelligenceConfig,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Memory engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineConfig {
    /// Hierarchy configuration
    pub hierarchy: HierarchyConfig,

    /// Intelligence configuration
    pub intelligence: IntelligenceConfig,

    /// Enable automatic memory processing
    pub auto_processing: bool,

    /// Processing interval in seconds
    pub processing_interval_seconds: u64,

    /// Maximum memories to process in one batch
    pub max_batch_size: usize,
}

impl Default for MemoryEngineConfig {
    fn default() -> Self {
        Self {
            hierarchy: HierarchyConfig::default(),
            intelligence: IntelligenceConfig::default(),
            auto_processing: true,
            processing_interval_seconds: 300, // 5 minutes
            max_batch_size: 100,
        }
    }
}

impl From<&agent_mem_config::memory::MemoryConfig> for MemoryEngineConfig {
    fn from(config: &agent_mem_config::memory::MemoryConfig) -> Self {
        // Convert config IntelligenceConfig to core IntelligenceConfig
        let intelligence = IntelligenceConfig {
            importance_weights: crate::intelligence::ImportanceWeights::default(),
            conflict_sensitivity: if config.intelligence.enable_conflict_detection {
                0.8
            } else {
                0.0
            },
            auto_resolution_threshold: config.intelligence.similarity_threshold as f64,
        };

        Self {
            hierarchy: HierarchyConfig::default(),
            intelligence,
            auto_processing: config.intelligence.importance_scoring,
            processing_interval_seconds: 300,
            max_batch_size: 100,
        }
    }
}

/// Core memory engine
pub struct MemoryEngine {
    config: MemoryEngineConfig,
    hierarchy_manager: Arc<dyn HierarchyManager>,
    importance_scorer: Arc<dyn ImportanceScorer>,
    conflict_resolver: Arc<dyn ConflictResolver>,
    /// Optional LibSQL memory repository for persistent storage
    memory_repository: Option<Arc<dyn crate::storage::traits::MemoryRepositoryTrait>>,
}

impl MemoryEngine {
    /// Create new memory engine with default implementations
    pub fn new(config: MemoryEngineConfig) -> Self {
        let hierarchy_manager = Arc::new(DefaultHierarchyManager::new(config.hierarchy.clone()));
        let importance_scorer = Arc::new(DefaultImportanceScorer::new(config.intelligence.clone()));
        let conflict_resolver = Arc::new(DefaultConflictResolver::new(config.intelligence.clone()));

        Self {
            config,
            hierarchy_manager,
            importance_scorer,
            conflict_resolver,
            memory_repository: None,
        }
    }

    /// Create new memory engine with LibSQL repository for persistent storage
    pub fn with_repository(
        config: MemoryEngineConfig,
        memory_repository: Arc<dyn crate::storage::traits::MemoryRepositoryTrait>,
    ) -> Self {
        let hierarchy_manager = Arc::new(DefaultHierarchyManager::new(config.hierarchy.clone()));
        let importance_scorer = Arc::new(DefaultImportanceScorer::new(config.intelligence.clone()));
        let conflict_resolver = Arc::new(DefaultConflictResolver::new(config.intelligence.clone()));

        Self {
            config,
            hierarchy_manager,
            importance_scorer,
            conflict_resolver,
            memory_repository: Some(memory_repository),
        }
    }

    /// Add memory with full processing
    pub async fn add_memory(&self, mut memory: Memory) -> crate::CoreResult<String> {
        // Calculate importance if auto-processing is enabled
        if self.config.auto_processing {
            let importance_factors = self.importance_scorer.calculate_importance(&memory).await?;
            memory.score = Some(importance_factors.final_score as f32);

            debug!(
                "Calculated importance {} for memory {}",
                memory.score.unwrap_or(0.0),
                memory.id
            );
        }

        // Add to hierarchy
        let hierarchical_memory = self.hierarchy_manager.add_memory(memory).await?;

        info!("Added memory {} to engine", hierarchical_memory.memory.id);
        Ok(hierarchical_memory.memory.id)
    }

    /// Get memory by ID
    pub async fn get_memory(&self, id: &str) -> crate::CoreResult<Option<Memory>> {
        if let Some(hierarchical_memory) = self.hierarchy_manager.get_memory(id).await? {
            Ok(Some(hierarchical_memory.memory))
        } else {
            Ok(None)
        }
    }

    /// Update memory with reprocessing
    pub async fn update_memory(&self, mut memory: Memory) -> crate::CoreResult<Memory> {
        // Recalculate importance if auto-processing is enabled
        if self.config.auto_processing {
            let importance_factors = self.importance_scorer.calculate_importance(&memory).await?;
            memory.score = Some(importance_factors.final_score as f32);
        }

        // Get current hierarchical memory
        if let Some(mut hierarchical_memory) = self.hierarchy_manager.get_memory(&memory.id).await?
        {
            hierarchical_memory.memory = memory;

            // Update in hierarchy (may trigger level changes)
            let updated = self
                .hierarchy_manager
                .update_memory(hierarchical_memory)
                .await?;

            info!("Updated memory {}", updated.memory.id);
            Ok(updated.memory)
        } else {
            Err(crate::CoreError::Storage(format!(
                "Memory {} not found",
                memory.id
            )))
        }
    }

    /// Remove memory
    pub async fn remove_memory(&self, id: &str) -> crate::CoreResult<bool> {
        let removed = self.hierarchy_manager.remove_memory(id).await?;

        if removed {
            info!("Removed memory {}", id);
        }

        Ok(removed)
    }

    /// Search memories with intelligent ranking
    pub async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        limit: Option<usize>,
    ) -> crate::CoreResult<Vec<Memory>> {
        info!("Searching memories: query='{}', scope={:?}, limit={:?}", query, scope, limit);

        // ✅ 优先使用 LibSQL Repository（持久化存储）
        if let Some(memory_repo) = &self.memory_repository {
            info!("Using LibSQL memory repository for persistent search");
            
            // 从 LibSQL 读取记忆
            let agent_id = match &scope {
                Some(MemoryScope::Agent(id)) => Some(id.as_str()),
                _ => None,
            };
            
            let fetch_limit = limit.unwrap_or(100) as i64;
            let db_memories = if let Some(aid) = agent_id {
                memory_repo.find_by_agent_id(aid, fetch_limit).await
                    .map_err(|e| crate::CoreError::Storage(e.to_string()))?
            } else {
                memory_repo.list(0, fetch_limit).await
                    .map_err(|e| crate::CoreError::Storage(e.to_string()))?
            };
            
            info!("Found {} memories from LibSQL", db_memories.len());
            
            // 转换为 Memory (MemoryItem) 类型并计算相关性
            let mut scored_memories: Vec<(Memory, f64)> = db_memories
                .into_iter()
                .map(|db_mem| {
                    use std::collections::HashMap;
                    use chrono::Utc;
                    
                    // 解析 metadata JSON
                    let metadata: HashMap<String, serde_json::Value> = if let serde_json::Value::Object(map) = &db_mem.metadata {
                        map.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    };
                    
                    // 转换 storage::models::Memory -> MemoryItem
                    let memory = Memory {
                        id: db_mem.id.clone(),
                        agent_id: db_mem.agent_id.clone(),
                        user_id: Some(db_mem.user_id.clone()),
                        content: db_mem.content.clone(),
                        hash: db_mem.hash.clone(),
                        metadata,
                        score: db_mem.score,
                        created_at: db_mem.created_at,
                        updated_at: Some(db_mem.updated_at),
                        memory_type: db_mem.memory_type.parse().unwrap_or(agent_mem_traits::MemoryType::Semantic),
                        importance: db_mem.importance,
                        // 默认值字段
                        session: agent_mem_traits::Session {
                            id: format!("session-{}", db_mem.id),
                            user_id: Some(db_mem.user_id.clone()),
                            agent_id: Some(db_mem.agent_id.clone()),
                            run_id: None,
                            actor_id: None,
                            created_at: db_mem.created_at,
                            metadata: HashMap::new(),
                        },
                        entities: vec![],
                        relations: vec![],
                        embedding: None,
                        last_accessed_at: db_mem.last_accessed.unwrap_or_else(|| Utc::now()),
                        access_count: db_mem.access_count as u32,
                        expires_at: None,
                        version: 1,
                    };
                    
                    // 计算相关性分数
                    let score = self.calculate_relevance_score(&memory, query);
                    (memory, score)
                })
                .filter(|(_, score)| *score > 0.0)
                .collect();
            
            // 按分数排序
            scored_memories.sort_by(|(mem_a, score_a), (mem_b, score_b)| {
                let combined_a = score_a + (mem_a.importance as f64 * 0.3);
                let combined_b = score_b + (mem_b.importance as f64 * 0.3);
                combined_b.partial_cmp(&combined_a).unwrap_or(std::cmp::Ordering::Equal)
            });
            
            // 应用限制并设置分数
            let final_memories: Vec<Memory> = scored_memories
                .into_iter()
                .take(limit.unwrap_or(10))
                .map(|(mut mem, score)| {
                    mem.score = Some(score as f32);
                    mem
                })
                .collect();
            
            info!("Returning {} memories from LibSQL (after ranking and limit)", final_memories.len());
            return Ok(final_memories);
        }

        // ⚠️ Fallback: 使用内存层级管理器（当没有repository时）
        warn!("No LibSQL repository available, falling back to hierarchy_manager (may be empty!)");
        
        // Get all memories from hierarchy based on scope
        let mut all_memories = Vec::new();

        // Collect memories from all levels
        for level in [
            MemoryLevel::Strategic,
            MemoryLevel::Tactical,
            MemoryLevel::Operational,
            MemoryLevel::Contextual,
        ] {
            let level_memories = self.hierarchy_manager.get_memories_at_level(level).await?;
            all_memories.extend(level_memories.into_iter().map(|hm| hm.memory));
        }

        debug!("Found {} total memories before filtering", all_memories.len());

        // Apply scope filtering if specified
        let filtered_memories = if let Some(scope) = scope {
            all_memories
                .into_iter()
                .filter(|memory| self.matches_scope(memory, &scope))
                .collect()
        } else {
            all_memories
        };

        debug!("Found {} memories after scope filtering", filtered_memories.len());

        // Perform text-based search and ranking
        let mut scored_memories: Vec<(Memory, f64)> = filtered_memories
            .into_iter()
            .filter_map(|memory| {
                let score = self.calculate_relevance_score(&memory, query);
                if score > 0.0 {
                    Some((memory, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending) and importance
        scored_memories.sort_by(|(mem_a, score_a), (mem_b, score_b)| {
            let combined_a = score_a + (mem_a.score.unwrap_or(0.0) as f64 * 0.3);
            let combined_b = score_b + (mem_b.score.unwrap_or(0.0) as f64 * 0.3);
            combined_b
                .partial_cmp(&combined_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        let result_limit = limit.unwrap_or(10);
        let results: Vec<Memory> = scored_memories
            .into_iter()
            .take(result_limit)
            .map(|(memory, _)| memory)
            .collect();

        info!("Returning {} search results", results.len());
        Ok(results)
    }

    /// Check if a memory matches the given scope
    fn matches_scope(&self, memory: &Memory, scope: &MemoryScope) -> bool {
        match scope {
            MemoryScope::Global => true,
            MemoryScope::Agent(agent_id) => &memory.agent_id == agent_id,
            MemoryScope::User {
                agent_id,
                user_id,
            } => {
                &memory.agent_id == agent_id
                    && memory.user_id.as_ref().map(|uid| uid == user_id).unwrap_or(false)
            }
            MemoryScope::Session {
                agent_id,
                user_id,
                session_id,
            } => {
                &memory.agent_id == agent_id
                    && memory.user_id.as_ref().map(|uid| uid == user_id).unwrap_or(false)
                    && memory
                        .metadata
                        .get("session_id")
                        .map(|sid| sid == session_id)
                        .unwrap_or(false)
            }
        }
    }

    /// Calculate relevance score for a memory based on query
    fn calculate_relevance_score(&self, memory: &Memory, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let content_lower = memory.content.to_lowercase();

        // Exact match gets highest score
        if content_lower.contains(&query_lower) {
            return 1.0;
        }

        // Calculate word overlap score
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        if query_words.is_empty() || content_words.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for query_word in &query_words {
            if content_words.iter().any(|cw| cw.contains(query_word)) {
                matches += 1;
            }
        }

        (matches as f64) / (query_words.len() as f64)
    }

    /// Process memories for conflicts and optimization
    pub async fn process_memories(&self) -> crate::CoreResult<ProcessingReport> {
        let mut report = ProcessingReport::default();

        if !self.config.auto_processing {
            return Ok(report);
        }

        // Get all memories from all levels
        let mut all_memories = Vec::new();

        for level in [
            MemoryLevel::Strategic,
            MemoryLevel::Tactical,
            MemoryLevel::Operational,
            MemoryLevel::Contextual,
        ] {
            let level_memories = self.hierarchy_manager.get_memories_at_level(level).await?;
            all_memories.extend(level_memories.into_iter().map(|hm| hm.memory));
        }

        report.total_memories = all_memories.len();

        // Detect conflicts
        let conflicts = self
            .conflict_resolver
            .detect_conflicts(&all_memories)
            .await?;
        report.conflicts_detected = conflicts.len();

        // Auto-resolve conflicts
        if !conflicts.is_empty() {
            let resolved_memories = self
                .conflict_resolver
                .auto_resolve_conflicts(&conflicts, &all_memories)
                .await?;
            report.conflicts_resolved = all_memories.len() - resolved_memories.len();

            // Update resolved memories back to hierarchy
            for memory in resolved_memories {
                if let Err(e) = self.update_memory(memory).await {
                    warn!("Failed to update resolved memory: {}", e);
                    report.errors += 1;
                }
            }
        }

        info!("Processing completed: {:?}", report);
        Ok(report)
    }

    /// Get engine statistics
    pub async fn get_statistics(&self) -> crate::CoreResult<EngineStatistics> {
        let hierarchy_stats = self.hierarchy_manager.get_hierarchy_stats().await?;

        Ok(EngineStatistics {
            total_memories: hierarchy_stats.memories_by_level.values().sum(),
            memories_by_level: hierarchy_stats.memories_by_level,
            avg_importance_by_level: hierarchy_stats.avg_importance_by_level,
            inheritance_relationships: hierarchy_stats.inheritance_relationships,
            level_utilization: hierarchy_stats.level_utilization,
        })
    }
}

/// Memory processing report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessingReport {
    /// Total memories processed
    pub total_memories: usize,

    /// Conflicts detected
    pub conflicts_detected: usize,

    /// Conflicts resolved
    pub conflicts_resolved: usize,

    /// Memories promoted
    pub memories_promoted: usize,

    /// Memories demoted
    pub memories_demoted: usize,

    /// Processing errors
    pub errors: usize,

    /// Processing duration in milliseconds
    pub duration_ms: u64,
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatistics {
    /// Total number of memories
    pub total_memories: usize,

    /// Memories by hierarchy level
    pub memories_by_level: std::collections::HashMap<MemoryLevel, usize>,

    /// Average importance by level
    pub avg_importance_by_level: std::collections::HashMap<MemoryLevel, f64>,

    /// Number of inheritance relationships
    pub inheritance_relationships: usize,

    /// Level utilization ratios
    pub level_utilization: std::collections::HashMap<MemoryLevel, f64>,
}
