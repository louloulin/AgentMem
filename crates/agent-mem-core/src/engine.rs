//! Memory Engine - Core orchestration and management

use crate::hierarchy::MemoryScope;
use crate::{
    hierarchy::{DefaultHierarchyManager, HierarchyConfig, HierarchyManager, MemoryLevel},
    intelligence::{
        ConflictResolver, DefaultConflictResolver, DefaultImportanceScorer, ImportanceScorer,
        IntelligenceConfig,
    },
    storage::conversion::{db_to_memory, legacy_to_v4, memory_to_db, v4_to_legacy},
};
use agent_mem_traits::{MemoryItem as LegacyMemory, MemoryV4 as Memory};
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
            memory.set_score(importance_factors.final_score);

            debug!(
                "Calculated importance {} for memory {}",
                memory.score().unwrap_or(0.0),
                memory.id.as_str()
            );
        }

        // Add to hierarchy
        let hierarchical_memory = self.hierarchy_manager.add_memory(memory).await?;

        info!(
            "Added memory {} to engine",
            hierarchical_memory.memory.id.as_str()
        );
        Ok(hierarchical_memory.memory.id.as_str().to_string())
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
            memory.set_score(importance_factors.final_score);
        }

        // Get current hierarchical memory
        let memory_id = memory.id.as_str().to_string();
        if let Some(mut hierarchical_memory) = self.hierarchy_manager.get_memory(&memory_id).await?
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

    /// Search memories with intelligent ranking (session-aware with temporal decay)
    pub async fn search_memories(
        &self,
        query: &str,
        scope: Option<MemoryScope>,
        limit: Option<usize>,
    ) -> crate::CoreResult<Vec<Memory>> {
        info!(
            "Searching memories: query='{}', scope={:?}, limit={:?}",
            query, scope, limit
        );

        // ✅ 优先使用 LibSQL Repository（持久化存储）
        if let Some(memory_repo) = &self.memory_repository {
            info!("Using LibSQL memory repository for persistent search");

            // 提取scope信息用于过滤和加权
            let (agent_id, target_user_id, target_session_id) = match &scope {
                Some(MemoryScope::Agent(id)) => (Some(id.as_str()), None, None),
                Some(MemoryScope::User { agent_id, user_id }) => {
                    (Some(agent_id.as_str()), Some(user_id.as_str()), None)
                }
                Some(MemoryScope::Session {
                    agent_id,
                    user_id,
                    session_id,
                }) => (
                    Some(agent_id.as_str()),
                    Some(user_id.as_str()),
                    Some(session_id.as_str()),
                ),
                _ => (None, None, None),
            };

            let fetch_limit = limit.unwrap_or(100) as i64;

            // 🔧 修复: 根据scope获取记忆，对于Global Scope使用search方法
            let db_memories = match &scope {
                Some(MemoryScope::Global) => {
                    // 🔧 修复: Global Scope使用search方法进行LIKE查询，而不是list()
                    info!("🔍 Global Scope: 使用search方法查询: '{}'", query);
                    memory_repo
                        .search(query, fetch_limit)
                        .await
                        .map_err(|e| crate::CoreError::Storage(e.to_string()))?
                }
                _ => {
                    // 其他scope使用原有逻辑
                    if let Some(uid) = target_user_id {
                        // 优先按user_id过滤（同一用户的记忆）
                        // 🔧 修复: 对于User scope，先按user_id过滤，然后搜索
                        let user_memories = memory_repo
                            .find_by_user_id(uid, fetch_limit * 2)
                            .await
                            .map_err(|e| crate::CoreError::Storage(e.to_string()))?;

                        // 如果查询不为空，过滤包含查询的记忆
                        if !query.trim().is_empty() {
                            user_memories
                                .into_iter()
                                .filter(|m| match &m.content {
                                    agent_mem_traits::Content::Text(t) => t.contains(query),
                                    _ => false,
                                })
                                .take(fetch_limit as usize)
                                .collect()
                        } else {
                            user_memories
                                .into_iter()
                                .take(fetch_limit as usize)
                                .collect()
                        }
                    } else if let Some(aid) = agent_id {
                        // 回退到agent_id过滤
                        memory_repo
                            .find_by_agent_id(aid, fetch_limit)
                            .await
                            .map_err(|e| crate::CoreError::Storage(e.to_string()))?
                    } else {
                        // 无scope限制，使用search方法
                        memory_repo
                            .search(query, fetch_limit)
                            .await
                            .map_err(|e| crate::CoreError::Storage(e.to_string()))?
                    }
                }
            };

            info!(
                "Found {} memories from LibSQL (agent={:?}, user={:?}, session={:?})",
                db_memories.len(),
                agent_id,
                target_user_id,
                target_session_id
            );

            // 🔧 修复: 检测商品ID查询，用于过滤工作记忆
            let is_product_query = {
                use regex::Regex;
                Regex::new(r"P\d{6}").unwrap().is_match(query)
            };

            // 🆕 V4: db_memories already are Vec<Memory> (V4)
            let mut scored_memories: Vec<(Memory, f64)> = db_memories
                .into_iter()
                .filter(|memory| {
                    // 🔧 修复: 对于商品ID查询，过滤工作记忆
                    if is_product_query {
                        let mem_type = memory.memory_type().unwrap_or_else(|| "episodic".to_string());
                        !matches!(
                            mem_type.as_str(),
                            "working" | "Working"
                        )
                    } else {
                        true  // 非商品查询，不过滤
                    }
                })
                .map(|memory| {
                    // 计算内容相关性分数
                    let legacy = v4_to_legacy(&memory);
                    let relevance_score = self.calculate_relevance_score(&legacy, query);
                    
                    // ✅ 计算时间衰减权重（指数衰减，半衰期24小时）
                    let now = chrono::Utc::now();
                    let age_hours = (now - memory.metadata.created_at).num_hours() as f64;
                    
                    let memory_type_str = memory.attributes
                        .get(&agent_mem_traits::AttributeKey::core("memory_type"))
                        .and_then(|v| v.as_string())
                        .unwrap_or(&String::from(""))
                        .clone();
                    
                    let time_decay = if memory_type_str == "working" || memory_type_str == "Working" {
                        1.0  // Working memory不衰减
                    } else {
                        (-age_hours / 24.0).exp()  // 长期记忆：e^(-t/24)
                    };
                    
                    // ✅ 计算用户匹配权重 (V4: 使用属性访问)
                    let mem_user_id = memory.attributes
                        .get(&agent_mem_traits::AttributeKey::core("user_id"))
                        .and_then(|v| v.as_string());
                    
                    let user_match_boost = if let Some(ref mem_user_id) = mem_user_id {
                        if let Some(target_uid) = target_user_id {
                            if mem_user_id.as_str() == target_uid {
                                2.0  // 同一用户：加倍权重
                            } else {
                                0.3  // 不同用户：大幅降权
                            }
                        } else {
                            1.0  // 无user_id过滤：保持原权重
                        }
                    } else {
                        1.0
                    };
                    
                    // ✅ 综合权重计算 (V4: 使用 importance 属性)
                    let importance = memory.attributes
                        .get(&agent_mem_traits::AttributeKey::system("importance"))
                        .and_then(|v| v.as_number())
                        .unwrap_or(0.5);
                    
                    let final_score = relevance_score * time_decay * user_match_boost * (0.5 + 0.5 * importance);
                    
                    // 日志（安全截取字符串） (V4: 使用 content 访问)
                    let content_text = match &memory.content {
                        agent_mem_traits::Content::Text(t) => t.clone(),
                        agent_mem_traits::Content::Structured(v) => v.to_string(),
                        _ => String::new(),
                    };
                    let content_preview: String = content_text.chars().take(30).collect();
                    
                    info!("🔍 Memory: user={:?} age={}h relevance={:.2} decay={:.2} user_boost={:.1} importance={:.2} → final={:.3} | '{}'", 
                          mem_user_id.as_ref().map(|s| s.chars().take(8).collect::<String>()), 
                          age_hours,
                          relevance_score,
                          time_decay,
                          user_match_boost,
                          importance,
                          final_score,
                          content_preview);
                    
                    (memory, final_score)
                })
                .collect();

            info!(
                "📊 Collected {} memories with weighted scores",
                scored_memories.len()
            );

            // 🔧 修复: 改进排序逻辑 - 精确匹配优先，工作记忆降权
            let is_product_query = {
                use regex::Regex;
                Regex::new(r"P\d{6}").unwrap().is_match(query)
            };

            scored_memories.sort_by(|(mem_a, score_a), (mem_b, score_b)| {
                // 辅助函数：检查是否是精确商品匹配 (V4: 使用 Content 和 attributes 访问)
                let is_exact_product_match = |mem: &Memory, q: &str| -> bool {
                    if let Some(product_id) = {
                        use regex::Regex;
                        Regex::new(r"P\d{6}").unwrap().find(q).map(|m| m.as_str())
                    } {
                        // V4: 检查 content
                        let content_text = match &mem.content {
                            agent_mem_traits::Content::Text(t) => t.clone(),
                            agent_mem_traits::Content::Structured(v) => v.to_string(),
                            _ => String::new(),
                        };

                        content_text.contains(&format!("商品ID: {}", product_id))
                            || mem
                                .attributes
                                .get(&agent_mem_traits::AttributeKey::user("product_id"))
                                .and_then(|v| v.as_string())
                                .map(|pid| pid == product_id)
                                .unwrap_or(false)
                    } else {
                        false
                    }
                };

                if is_product_query {
                    // 1. 精确匹配优先
                    let a_exact = is_exact_product_match(mem_a, query);
                    let b_exact = is_exact_product_match(mem_b, query);

                    match (a_exact, b_exact) {
                        (true, false) => return std::cmp::Ordering::Less, // a 排在前面
                        (false, true) => return std::cmp::Ordering::Greater, // b 排在前面
                        _ => {}
                    }

                    // 2. 工作记忆降权（虽然已经过滤，但保留逻辑以防万一）(V4: 使用属性访问)
                    let get_memory_type = |mem: &Memory| -> String {
                        mem.attributes
                            .get(&agent_mem_traits::AttributeKey::core("memory_type"))
                            .and_then(|v| v.as_string())
                            .unwrap_or(&String::from("episodic"))
                            .clone()
                    };

                    let a_working = get_memory_type(mem_a).to_lowercase() == "working";
                    let b_working = get_memory_type(mem_b).to_lowercase() == "working";

                    match (a_working, b_working) {
                        (true, false) => return std::cmp::Ordering::Greater, // a 排在后面
                        (false, true) => return std::cmp::Ordering::Less,    // b 排在后面
                        _ => {}
                    }
                }

                // 3. 按分数排序
                score_b
                    .partial_cmp(score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // 应用限制并设置分数 (V4: 使用属性设置 score)
            let final_memories: Vec<Memory> = scored_memories
                .into_iter()
                .take(limit.unwrap_or(10))
                .map(|(mut mem, score)| {
                    mem.attributes.insert(
                        agent_mem_traits::AttributeKey::system("score"),
                        agent_mem_traits::AttributeValue::Number(score),
                    );
                    mem
                })
                .collect();

            info!(
                "Returning {} memories from LibSQL (after ranking and limit)",
                final_memories.len()
            );
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

        debug!(
            "Found {} total memories before filtering",
            all_memories.len()
        );

        // Apply scope filtering if specified
        let filtered_memories = if let Some(scope) = scope {
            all_memories
                .into_iter()
                .filter(|memory| self.matches_scope(memory, &scope))
                .collect()
        } else {
            all_memories
        };

        debug!(
            "Found {} memories after scope filtering",
            filtered_memories.len()
        );

        // Perform text-based search and ranking
        let mut scored_memories: Vec<(Memory, f64)> = filtered_memories
            .into_iter()
            .filter_map(|memory| {
                let legacy = v4_to_legacy(&memory);
                let score = self.calculate_relevance_score(&legacy, query);
                if score > 0.0 {
                    Some((memory, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending) and importance
        scored_memories.sort_by(|(mem_a, score_a), (mem_b, score_b)| {
            let combined_a = score_a + (mem_a.score().unwrap_or(0.0) * 0.3);
            let combined_b = score_b + (mem_b.score().unwrap_or(0.0) * 0.3);
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
            MemoryScope::Agent(agent_id) => memory.agent_id().as_ref() == Some(agent_id),
            MemoryScope::User { agent_id, user_id } => {
                memory.agent_id().as_ref() == Some(agent_id)
                    && memory
                        .user_id()
                        .as_ref()
                        .map(|uid| uid == user_id)
                        .unwrap_or(false)
            }
            MemoryScope::Session {
                agent_id,
                user_id,
                session_id,
            } => {
                memory.agent_id().as_ref() == Some(agent_id)
                    && memory
                        .user_id()
                        .as_ref()
                        .map(|uid| uid == user_id)
                        .unwrap_or(false)
                    && memory
                        .attributes
                        .get(&agent_mem_traits::AttributeKey::core("session_id"))
                        .and_then(|v| v.as_string())
                        .map(|sid| sid == session_id)
                        .unwrap_or(false)
            }
        }
    }

    /// Calculate relevance score for a memory based on query
    fn calculate_relevance_score(&self, memory: &LegacyMemory, query: &str) -> f64 {
        use regex::Regex;

        // 🔧 修复: 检测商品ID查询，优先处理精确ID匹配
        let product_id_pattern = Regex::new(r"P\d{6}").unwrap();
        if let Some(product_id) = product_id_pattern.find(query) {
            let product_id = product_id.as_str();

            // 1. 精确ID匹配（最高分）
            if memory.content.contains(&format!("商品ID: {}", product_id))
                || memory
                    .metadata
                    .get("product_id")
                    .and_then(|v| v.as_str())
                    .map(|pid| pid == product_id)
                    .unwrap_or(false)
            {
                info!("✅ 精确商品ID匹配: product_id={}", product_id);
                return 2.0; // 精确匹配：最高分
            }

            // 2. 包含ID但不精确（中等分）
            if memory.content.contains(product_id) {
                info!("✅ 包含商品ID: product_id={}", product_id);
                return 1.5;
            }
        }

        let query_lower = query.to_lowercase();
        let content_lower = memory.content.to_lowercase();

        // 3. 完全匹配（高分）
        if content_lower.contains(&query_lower) {
            info!("✅ 完全匹配");
            return 1.0;
        }

        // ✅ 改进：支持中文和英文的混合匹配
        // 对于中文，按字符匹配；对于英文，按单词匹配

        // 方法1: 简单字符重叠（适用于中文）
        let query_chars: Vec<char> = query_lower.chars().filter(|c| !c.is_whitespace()).collect();
        if query_chars.is_empty() {
            return 0.5; // 空查询给个默认分数
        }

        let mut char_matches = 0;
        for query_char in &query_chars {
            if content_lower.contains(*query_char) {
                char_matches += 1;
            }
        }

        let char_score = (char_matches as f64) / (query_chars.len() as f64);

        // 方法2: 单词重叠（适用于英文）
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();

        let word_score = if !query_words.is_empty() && !content_words.is_empty() {
            let mut word_matches = 0;
            for query_word in &query_words {
                if content_words.iter().any(|cw| cw.contains(query_word)) {
                    word_matches += 1;
                }
            }
            (word_matches as f64) / (query_words.len() as f64)
        } else {
            0.0
        };

        // 返回两种方法的最大值（兼容中英文）
        let final_score = char_score.max(word_score);
        info!(
            "📊 Scoring: char_score={:.3}, word_score={:.3}, final={:.3}",
            char_score, word_score, final_score
        );
        final_score
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
