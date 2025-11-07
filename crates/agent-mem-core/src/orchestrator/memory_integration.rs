//! 记忆集成模块 - 记忆检索和注入
//!
//! 参考 MIRIX 的记忆检索逻辑，实现智能记忆检索和 prompt 注入

use crate::{engine::MemoryEngine, Memory};
use agent_mem_traits::{MemoryType, Result};
use std::sync::Arc;
use tracing::{debug, info};

/// 记忆集成器配置
#[derive(Debug, Clone)]
pub struct MemoryIntegratorConfig {
    /// 最大检索记忆数量
    pub max_memories: usize,
    /// 相关性阈值
    pub relevance_threshold: f32,
    /// 是否包含时间信息
    pub include_timestamp: bool,
    /// 是否按重要性排序
    pub sort_by_importance: bool,
}

impl Default for MemoryIntegratorConfig {
    fn default() -> Self {
        Self {
            max_memories: 10,
            relevance_threshold: 0.1, // ✅ 降低阈值以支持更宽泛的匹配
            include_timestamp: true,
            sort_by_importance: true,
        }
    }
}

/// 记忆集成器
pub struct MemoryIntegrator {
    memory_engine: Arc<MemoryEngine>,
    config: MemoryIntegratorConfig,
}

impl MemoryIntegrator {
    /// 创建新的记忆集成器
    pub fn new(memory_engine: Arc<MemoryEngine>, config: MemoryIntegratorConfig) -> Self {
        Self {
            memory_engine,
            config,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(memory_engine: Arc<MemoryEngine>) -> Self {
        Self::new(memory_engine, MemoryIntegratorConfig::default())
    }

    /// 从对话中检索相关记忆（支持session隔离）
    ///
    /// 参考 MIRIX 的 _retrieve_memories 方法，增加session_id支持
    pub async fn retrieve_relevant_memories(
        &self,
        query: &str,
        agent_id: &str,
        max_count: usize,
    ) -> Result<Vec<Memory>> {
        self.retrieve_relevant_memories_with_session(query, agent_id, None, None, max_count)
            .await
    }

    /// 检索相关记忆（支持session和user过滤）
    pub async fn retrieve_relevant_memories_with_session(
        &self,
        query: &str,
        agent_id: &str,
        user_id: Option<&str>,
        session_id: Option<&str>,
        max_count: usize,
    ) -> Result<Vec<Memory>> {
        debug!(
            "Retrieving memories for agent_id={}, user_id={:?}, session_id={:?}, query={}",
            agent_id, user_id, session_id, query
        );

        // 使用 MemoryEngine 的搜索功能
        use crate::hierarchy::MemoryScope;

        // 根据参数创建最精确的 scope
        let scope = if let (Some(uid), Some(sid)) = (user_id, session_id) {
            // 最高优先级：Session scope（会话级别）
            Some(MemoryScope::Session {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
                session_id: sid.to_string(),
            })
        } else if let Some(uid) = user_id {
            // 中优先级：User scope（用户级别）
            Some(MemoryScope::User {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
            })
        } else {
            // 低优先级：Agent scope（仅按agent过滤）
            Some(MemoryScope::Agent(agent_id.to_string()))
        };

        // 调用 MemoryEngine 进行搜索
        let scope_str = format!("{:?}", scope); // Clone scope info for logging
        let memories = self
            .memory_engine
            .search_memories(query, scope, Some(max_count))
            .await
            .map_err(|e| agent_mem_traits::AgentMemError::storage_error(e.to_string()))?;

        // 过滤低相关性记忆（基于 importance score）
        let filtered_memories: Vec<Memory> = memories
            .into_iter()
            .filter(|m| m.score.unwrap_or(0.0) >= self.config.relevance_threshold)
            .collect();

        info!(
            "Retrieved {} relevant memories (filtered from search results, scope={})",
            filtered_memories.len(),
            scope_str
        );
        Ok(filtered_memories)
    }

    /// 🆕 Phase 1: Episodic-first记忆检索（基于认知理论）
    ///
    /// ## 理论依据
    /// - **Atkinson-Shiffrin模型**: Long-term Memory应该是主要检索源
    /// - **HCAM**: 分层检索（粗略→精细）
    /// - **Adaptive Framework**: 动态权重调整
    ///
    /// ## 检索策略（符合认知模型）
    /// 1. **Priority 1**: Episodic Memory (Agent/User scope) - 主要来源（90%）
    /// 2. **Priority 2**: Working Memory (Session scope) - 补充上下文（10%）
    /// 3. **Priority 3**: Semantic Memory (Agent scope) - 备选
    ///
    /// ## 权重调整（基于Adaptive Framework）
    /// - Episodic Memory: 权重 1.2（提升主要来源）
    /// - Working Memory: 权重 1.0（正常，因为新鲜）
    /// - Semantic Memory: 权重 0.9（降低，因为范围更广）
    pub async fn retrieve_episodic_first(
        &self,
        query: &str,
        agent_id: &str,
        user_id: Option<&str>,
        session_id: Option<&str>,
        max_count: usize,
    ) -> Result<Vec<Memory>> {
        use crate::hierarchy::MemoryScope;
        use std::collections::HashSet;
        use tracing::warn;

        let mut all_memories = Vec::new();
        let mut seen_ids = HashSet::new();

        info!(
            "🧠 Episodic-first检索 (理论指导): agent={}, user={:?}, session={:?}, target={}",
            agent_id, user_id, session_id, max_count
        );

        // ========== Priority 1: Episodic Memory (Agent/User Scope) ==========
        // 理论依据: Atkinson-Shiffrin模型 - Long-term Memory是主要来源
        if let Some(uid) = user_id {
            let episodic_scope = MemoryScope::User {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
            };

            info!("📚 Priority 1: Querying Episodic Memory (Agent/User scope) - 主要来源");

            // 查询更多数量（max_count * 2），因为这是主要来源
            match self
                .memory_engine
                .search_memories(query, Some(episodic_scope), Some(max_count * 2))
                .await
            {
                Ok(memories) => {
                    let count = memories.len();
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            // 🎯 Episodic Memory 权重: 1.2 (基于Adaptive Framework)
                            if let Some(score) = memory.score {
                                memory.score = Some(score * 1.2);
                            }
                            all_memories.push(memory);
                        }
                    }
                    info!("📚 Episodic Memory returned {} memories", count);
                }
                Err(e) => {
                    warn!("⚠️  Episodic Memory query failed: {}", e);
                }
            }
        }

        // ========== Priority 2: Working Memory (Session Scope) ==========
        // 理论依据: Working Memory作为补充上下文（容量7±2项）
        if let (Some(uid), Some(sid)) = (user_id, session_id) {
            let working_scope = MemoryScope::Session {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
                session_id: sid.to_string(),
            };

            info!("🔄 Priority 2: Querying Working Memory (Session scope) - 补充上下文");

            // 只查询少量（max_count / 2），因为只是补充
            match self
                .memory_engine
                .search_memories(query, Some(working_scope), Some(max_count / 2))
                .await
            {
                Ok(memories) => {
                    let mut added = 0;
                    for memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            // 🎯 Working Memory 权重: 1.0（正常，因为新鲜且相关）
                            all_memories.push(memory);
                            added += 1;
                        }
                    }
                    info!("🔄 Working Memory added {} memories as context", added);
                }
                Err(e) => {
                    warn!("⚠️  Working Memory query failed: {}", e);
                }
            }
        }

        // ========== Priority 3: Semantic Memory (Agent Scope) ==========
        // 理论依据: 备选，如果前面不够则查询更广范围
        if all_memories.len() < max_count {
            let semantic_scope = MemoryScope::Agent(agent_id.to_string());

            let remaining = max_count.saturating_sub(all_memories.len());
            info!(
                "📖 Priority 3: Querying Semantic Memory (Agent scope) - 需要 {} 更多",
                remaining
            );

            match self
                .memory_engine
                .search_memories(query, Some(semantic_scope), Some(remaining * 2))
                .await
            {
                Ok(memories) => {
                    let mut added = 0;
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            // 🎯 Semantic Memory 权重: 0.9（降低，因为范围更广）
                            if let Some(score) = memory.score {
                                memory.score = Some(score * 0.9);
                            }
                            all_memories.push(memory);
                            added += 1;
                            if all_memories.len() >= max_count {
                                break;
                            }
                        }
                    }
                    info!("📖 Semantic Memory added {} memories", added);
                }
                Err(e) => {
                    warn!("⚠️  Semantic Memory query failed: {}", e);
                }
            }
        }

        // 最终结果统计（认知架构分类）
        let final_count = all_memories.len();
        let episodic_count = all_memories
            .iter()
            .filter(|m| {
                // 简单判断：包含user_id但不包含session的是Episodic
                m.metadata.contains_key("user_id") && !m.id.contains("session")
            })
            .count();
        let working_count = all_memories
            .iter()
            .filter(|m| m.id.contains("session"))
            .count();
        let semantic_count = final_count - episodic_count - working_count;

        info!(
            "✅ 检索完成 (认知架构): {} memories (Episodic: {}, Working: {}, Semantic: {})",
            final_count, episodic_count, working_count, semantic_count
        );

        // 按调整后的score排序
        all_memories.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回 top N（基于HCAM的两阶段检索结果）
        Ok(all_memories.into_iter().take(max_count).collect())
    }

    /// 将记忆注入到 prompt
    ///
    /// 参考 MIRIX 的记忆格式化方式
    pub fn inject_memories_to_prompt(&self, memories: &[Memory]) -> String {
        if memories.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("## Relevant Memories\n\n");
        prompt.push_str("The following memories may be relevant to the current conversation:\n\n");

        for (i, memory) in memories.iter().enumerate() {
            // 格式化记忆
            prompt.push_str(&format!("{}. ", i + 1));

            // 添加记忆类型标签
            prompt.push_str(&format!(
                "[{}] ",
                self.format_memory_type(&memory.memory_type)
            ));

            // 添加记忆内容
            prompt.push_str(&memory.content);

            // 添加时间戳（如果启用）
            if self.config.include_timestamp {
                let time_str = memory.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
                prompt.push_str(&format!(" ({time_str})"));
            }

            // 添加重要性分数（如果有）
            if memory.importance > 0.7 {
                prompt.push_str(" [Important]");
            }

            prompt.push('\n');
        }

        prompt.push_str(
            "\nPlease use these memories to provide more contextual and personalized responses.\n",
        );
        prompt
    }

    /// 格式化记忆类型
    fn format_memory_type(&self, memory_type: &MemoryType) -> &str {
        match memory_type {
            MemoryType::Episodic => "Episodic",
            MemoryType::Semantic => "Semantic",
            MemoryType::Procedural => "Procedural",
            MemoryType::Working => "Working",
            MemoryType::Core => "Core",
            MemoryType::Resource => "Resource",
            MemoryType::Knowledge => "Knowledge",
            MemoryType::Contextual => "Contextual",
            MemoryType::Factual => "Factual",
        }
    }

    /// 按重要性和相关性排序记忆
    pub fn sort_memories(&self, mut memories: Vec<Memory>) -> Vec<Memory> {
        if self.config.sort_by_importance {
            memories.sort_by(|a, b| {
                b.importance
                    .partial_cmp(&a.importance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        memories
    }

    /// 过滤低相关性记忆
    pub fn filter_by_relevance(&self, memories: Vec<Memory>) -> Vec<Memory> {
        info!(
            "🔍 filter_by_relevance: input={} memories, threshold={}",
            memories.len(),
            self.config.relevance_threshold
        );

        let filtered: Vec<Memory> = memories
            .into_iter()
            .filter(|m| {
                let keep = m.importance >= self.config.relevance_threshold;
                info!(
                    "  Memory importance={:.3}, threshold={:.3}, keep={}",
                    m.importance, self.config.relevance_threshold, keep
                );
                keep
            })
            .collect();

        info!("🔍 filter_by_relevance: output={} memories", filtered.len());
        filtered
    }
}
