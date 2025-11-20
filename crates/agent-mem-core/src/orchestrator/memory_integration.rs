//! 记忆集成模块 - 记忆检索和注入
//!
//! 参考 MIRIX 的记忆检索逻辑，实现智能记忆检索和 prompt 注入

use crate::{engine::MemoryEngine, Memory};
use agent_mem_traits::{MemoryType, Result};
use regex::Regex;
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};
use tracing::{debug, info};

/// 记忆集成器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryIntegratorConfig {
    /// 最大检索记忆数量
    pub max_memories: usize,
    /// 相关性阈值
    pub relevance_threshold: f32,
    /// 是否包含时间信息
    pub include_timestamp: bool,
    /// 是否按重要性排序
    pub sort_by_importance: bool,
    
    // 🆕 Phase 1.5: 认知架构权重配置（基于Adaptive Memory Framework）
    /// Episodic Memory权重（Long-term Memory优先，理论依据: Atkinson-Shiffrin）
    pub episodic_weight: f32,
    /// Working Memory权重（补充上下文，理论依据: Working Memory容量7±2）
    pub working_weight: f32,
    /// Semantic Memory权重（备选，理论依据: HCAM分层检索）
    pub semantic_weight: f32,
    
    // ⭐ Phase 5: 记忆压缩配置
    /// 启用记忆压缩
    pub enable_compression: bool,
    /// 压缩阈值（超过此数量启动压缩）
    pub compression_threshold: usize,
}

#[derive(Debug)]
struct CacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    size: AtomicU64,
}

impl CacheMetrics {
    fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            size: AtomicU64::new(0),
        }
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    fn set_size(&self, size: usize) {
        self.size.store(size as u64, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    fn snapshot(&self) -> (u64, u64, u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.evictions.load(Ordering::Relaxed),
            self.size.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_cache_key() {
        let engine = Arc::new(MemoryEngine::new(Default::default()));
        let integrator = MemoryIntegrator::with_default_config(engine);

        let key = integrator.normalize_cache_key(
            "  Hello World ",
            "agent-1",
            Some("user-1"),
            Some("session-1"),
        );

        assert_eq!(
            key,
            "agent-1::user-1::session-1::hello world".to_string()
        );
    }
}

impl Default for MemoryIntegratorConfig {
    fn default() -> Self {
        Self {
            max_memories: 3, // Phase 2/3优化
            relevance_threshold: 0.1,
            include_timestamp: true,
            sort_by_importance: true,
            
            // Phase 1.5: 认知架构权重
            episodic_weight: 1.2,
            working_weight: 1.0,
            semantic_weight: 0.9,
            
            // Phase 5: 记忆压缩
            enable_compression: true,
            compression_threshold: 10, // 超过10条启动压缩
        }
    }
}

/// ⭐ 简单缓存项
#[derive(Clone)]
struct CacheEntry {
    memories: Vec<Memory>,
    timestamp: std::time::Instant,
}

/// 记忆集成器
pub struct MemoryIntegrator {
    memory_engine: Arc<MemoryEngine>,
    config: MemoryIntegratorConfig,
    /// ⭐ 简单LRU缓存 (query -> memories)
    cache: Arc<RwLock<lru::LruCache<String, CacheEntry>>>,
    cache_metrics: CacheMetrics,
}

impl MemoryIntegrator {
    /// 创建新的记忆集成器
    pub fn new(memory_engine: Arc<MemoryEngine>, config: MemoryIntegratorConfig) -> Self {
        Self {
            memory_engine,
            config,
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                NonZeroUsize::new(100).unwrap(),
            ))),
            cache_metrics: CacheMetrics::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config(memory_engine: Arc<MemoryEngine>) -> Self {
        Self::new(memory_engine, MemoryIntegratorConfig::default())
    }
    
    /// ⭐ 检查缓存
    fn get_cached(&self, key: &str) -> Option<Vec<Memory>> {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(entry) = cache.get(key) {
                if entry.timestamp.elapsed().as_secs() < 300 {
                    debug!("🎯 Cache hit for key: {}", &key[..key.len().min(50)]);
                    self.cache_metrics.record_hit();
                    return Some(entry.memories.clone());
                }

                cache.pop(key);
            }
        }

        self.cache_metrics.record_miss();
        None
    }

    fn normalize_cache_key(
        &self,
        query: &str,
        agent_id: &str,
        user_id: Option<&str>,
        session_id: Option<&str>,
    ) -> String {
        let normalized_query = query.trim().to_lowercase();
        let user_part = user_id.unwrap_or("_global");
        let session_part = session_id.unwrap_or("_session");
        format!(
            "{}::{}::{}::{}",
            agent_id, user_part, session_part, normalized_query
        )
    }
    
    /// ⭐ 更新缓存
    fn update_cache(&self, key: String, memories: Vec<Memory>) {
        if let Ok(mut cache) = self.cache.write() {
            let evicted = cache.put(
                key,
                CacheEntry {
                memories: memories.clone(),
                timestamp: std::time::Instant::now(),
                },
            );

            if evicted.is_some() {
                self.cache_metrics.record_eviction();
            }

            self.cache_metrics.set_size(cache.len());
        }
    }

    /// 失效指定Agent/User的缓存
    pub fn invalidate_cache(&self, agent_id: &str, user_id: Option<&str>) {
        if let Ok(mut cache) = self.cache.write() {
            let prefix = match user_id {
                Some(uid) => format!("{}::{}::", agent_id, uid),
                None => format!("{}::", agent_id),
            };

            let keys: Vec<String> = cache
                .iter()
                .map(|(k, _)| k.clone())
                .filter(|k| k.starts_with(&prefix))
                .collect();

            for key in keys {
                cache.pop(&key);
            }

            self.cache_metrics.set_size(cache.len());
            info!("🗑️  Invalidated cache entries with prefix {}", prefix);
        }
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
            .filter(|m| m.score().unwrap_or(0.0) >= self.config.relevance_threshold as f64)
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
        // ⭐ 先检查缓存
        let cache_key = self.normalize_cache_key(query, agent_id, user_id, session_id);
        if let Some(cached) = self.get_cached(&cache_key) {
            info!("🎯 Cache hit, returning {} cached memories", cached.len());
            return Ok(cached.into_iter().take(max_count).collect());
        }
        
        use crate::hierarchy::MemoryScope;
        use std::collections::HashSet;
        use tracing::warn;

        let mut all_memories = Vec::new();
        let mut seen_ids = HashSet::new();

        info!(
            "🧠 Episodic-first检索 (理论指导): agent={}, user={:?}, session={:?}, target={}",
            agent_id, user_id, session_id, max_count
        );

        // 🔧 修复: 改进商品ID检测 - 从查询中提取商品ID（即使包含其他文本）
        let product_id_pattern = Regex::new(r"P\d{6}").unwrap(); // 不要求完全匹配，允许包含其他文本
        let extracted_product_id = product_id_pattern.find(query).map(|m| m.as_str());
        
        if let Some(product_id) = extracted_product_id {
            info!(
                "🎯 检测到商品ID查询，提取ID: {} (from query: {})",
                product_id, query
            );
            
            // 使用提取的商品ID进行查询（而不是完整查询）
            let global_scope = MemoryScope::Global;
            match self
                .memory_engine
                .search_memories(product_id, Some(global_scope), Some(max_count * 2))
                .await
            {
                Ok(memories) if !memories.is_empty() => {
                    info!(
                        "✅ Global Memory (商品ID查询) 找到 {} 条记忆",
                        memories.len()
                    );
                    
                    // 🔧 修复: 优先返回精确匹配的商品记忆，过滤工作记忆
                    let mut exact_product_memories = Vec::new();
                    let mut other_memories = Vec::new();
                    
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            // 检查是否是精确匹配的商品记忆
                            let content_str = match &memory.content {
                                agent_mem_traits::Content::Text(t) => t.as_str(),
                                agent_mem_traits::Content::Structured(v) => "",
                                _ => "",
                            };
                            let is_exact_product = {
                                content_str.contains(&format!("商品ID: {}", product_id))
                                    || memory
                                        .attributes
                                    .get(&agent_mem_traits::AttributeKey::core("product_id"))
                                    .and_then(|attr_val| attr_val.as_string())
                                    .map(|pid| pid == product_id)
                                    .unwrap_or(false)
                            };
                            
                            // 排除工作记忆
                            let mem_type_opt = memory.memory_type();
                            let is_working_memory = mem_type_opt
                                .as_ref()
                                .map(|t| t.to_lowercase() == "working")
                                .unwrap_or(false);
                            
                            if is_exact_product && !is_working_memory {
                                // 精确匹配的商品记忆，权重提升
                                if let Some(score) = memory.score() {
                                    memory.set_score(score * 2.0); // 大幅提升权重
                                }
                                exact_product_memories.push(memory);
                            } else if !is_working_memory {
                                // 其他相关记忆
                            if let Some(score) = memory.score() {
                                    memory.set_score(score * 1.2); // 适度提升权重
                                }
                                other_memories.push(memory);
                            }
                        }
                    }
                    
                    // 合并：精确匹配在前
                    let exact_count = exact_product_memories.len();
                    all_memories.extend(exact_product_memories);
                    all_memories.extend(other_memories);
                    
                    // 如果找到足够的结果，直接返回
                    if all_memories.len() >= max_count {
                        info!(
                            "✅ 商品ID查询完成，返回 {} 条结果 (精确匹配: {})",
                            all_memories.len(), 
                            exact_count
                        );
                        all_memories.sort_by(|a, b| {
                            b.score()
                                .unwrap_or(0.0)
                                .partial_cmp(&a.score().unwrap_or(0.0))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        return Ok(all_memories.into_iter().take(max_count).collect());
                    }
                }
                Ok(_) => {
                    warn!(
                        "⚠️  Global Memory查询返回0结果: product_id='{}'",
                        product_id
                    );
                }
                Err(e) => {
                    warn!("⚠️  Global Memory查询失败: {}, 继续其他scope查询", e);
                }
            }
        }

        // ========== ✅ Task 1.2: 并行查询 Priority 1 & 2 (优化) ==========
        let mut query_count = 0;

        // ✅ 优化1: 并行查询最重要的两层（Episodic + Working）
        if let Some(uid) = user_id {
            let episodic_scope = MemoryScope::User {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
            };

            let working_scope = session_id.map(|sid| MemoryScope::Session {
                agent_id: agent_id.to_string(),
                user_id: uid.to_string(),
                session_id: sid.to_string(),
            });

            info!("📚🔄 [1-2/4] Parallel querying Episodic + Working Memory");

            let episodic_query = self.memory_engine.search_memories(
                query,
                Some(episodic_scope),
                Some(max_count * 2),
            );

            let working_query = if let Some(ws) = working_scope {
                Some(
                    self.memory_engine
                        .search_memories(query, Some(ws), Some(max_count / 2)),
                )
            } else {
                None
            };

            // ✅ 并行执行
            let (episodic_result, working_result) = if let Some(wq) = working_query {
                let (e, w) = tokio::join!(episodic_query, wq);
                (e, Some(w))
            } else {
                (episodic_query.await, None)
            };

            // 处理 Episodic 结果
            match episodic_result {
                Ok(memories) => {
                    let count = memories.len();
                    query_count += 1;
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                        if let Some(score) = memory.score() {
                            memory.set_score(score * self.config.episodic_weight as f64);
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

            // 处理 Working 结果
            if let Some(Ok(memories)) = working_result {
                    let mut added = 0;
                query_count += 1;
                    for memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            all_memories.push(memory);
                            added += 1;
                        }
                    }
                info!("🔄 Working Memory added {} memories", added);
                }
        }

        // ✅ 优化2: 早停检查1 - Episodic + Working已足够
        if all_memories.len() >= max_count {
            let saved_queries = 2; // 节省了Semantic和Global查询
            info!(
                "✅ Early stop after Priority 1-2: {} >= target {}, saved {} queries",
                all_memories.len(),
                max_count,
                saved_queries
            );

            // 记录统计
            self.record_query_stats(query_count, saved_queries);

            // 排序、去重、限制数量
            all_memories.sort_by(|a, b| {
                b.score()
                    .unwrap_or(0.0)
                    .partial_cmp(&a.score().unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let result: Vec<Memory> = all_memories.into_iter().take(max_count).collect();

            // 更新缓存
            self.update_cache(cache_key.clone(), result.clone());

            return Ok(result);
        }

        // ========== Priority 3: Semantic Memory (Agent Scope) ==========
        // ✅ 优化3: 仅在需要时查询
        if all_memories.len() < max_count {
            let semantic_scope = MemoryScope::Agent(agent_id.to_string());

            let remaining = max_count.saturating_sub(all_memories.len());
            info!(
                "📖 [3/4] Querying Semantic Memory - need {} more",
                remaining
            );

            match self
                .memory_engine
                .search_memories(query, Some(semantic_scope), Some(remaining * 2))
                .await
            {
                Ok(memories) => {
                    let mut added = 0;
                    query_count += 1;
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            if let Some(score) = memory.score() {
                                memory.set_score(score * self.config.semantic_weight as f64);
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

            // ✅ 优化4: 早停检查2 - 加上Semantic已足够
            if all_memories.len() >= max_count {
                let saved_queries = 1; // 节省了Global查询
                info!(
                    "✅ Early stop after Priority 3: {} >= target {}, saved {} queries",
                    all_memories.len(),
                    max_count,
                    saved_queries
                );

                self.record_query_stats(query_count, saved_queries);

                all_memories.sort_by(|a, b| {
                    b.score()
                        .unwrap_or(0.0)
                        .partial_cmp(&a.score().unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let result: Vec<Memory> = all_memories.into_iter().take(max_count).collect();
                self.update_cache(cache_key.clone(), result.clone());

                return Ok(result);
            }
        }

        // ========== Priority 4: Global Memory (Global Scope) ==========
        // 理论依据: 全局知识库，包含通用知识、产品信息等
        // 修复: 支持global scope的商品记忆等全局知识
        if all_memories.len() < max_count {
            let global_scope = MemoryScope::Global;

            let remaining = max_count.saturating_sub(all_memories.len());
            info!(
                "🌍 Priority 4: Querying Global Memory (Global scope) - 需要 {} 更多",
                remaining
            );

            match self
                .memory_engine
                .search_memories(query, Some(global_scope), Some(remaining * 2))
                .await
            {
                Ok(memories) => {
                    let mut added = 0;
                    for mut memory in memories {
                        if seen_ids.insert(memory.id.clone()) {
                            // 🎯 Global Memory 权重 (可配置，降低因为范围最广)
                            if let Some(score) = memory.score() {
                                memory.set_score(score * self.config.semantic_weight as f64);
                            }
                            all_memories.push(memory);
                            added += 1;
                            if all_memories.len() >= max_count {
                                break;
                            }
                        }
                    }
                    info!("🌍 Global Memory added {} memories", added);
                }
                Err(e) => {
                    warn!("⚠️  Global Memory query failed: {}", e);
                }
            }
        }

        // 最终结果统计（认知架构分类）
        let final_count = all_memories.len();
        let episodic_count = all_memories
            .iter()
            .filter(|m| {
                // 简单判断：包含user_id但不包含session的是Episodic
                m.user_id().is_some() && !m.id.as_str().contains("session")
            })
            .count();
        let working_count = all_memories
            .iter()
            .filter(|m| m.id.as_str().contains("session"))
            .count();
        let semantic_count = final_count - episodic_count - working_count;

        info!(
            "✅ 检索完成 (认知架构): {} memories (Episodic: {}, Working: {}, Semantic: {})",
            final_count, episodic_count, working_count, semantic_count
        );

        // 按调整后的score排序
        all_memories.sort_by(|a, b| {
            b.score()
                .unwrap_or(0.0)
                .partial_cmp(&a.score().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 返回 top N（基于HCAM的两阶段检索结果）
        let result: Vec<Memory> = all_memories.into_iter().take(max_count).collect();
        
        // ⭐ 更新缓存
        self.update_cache(cache_key, result.clone());
        
        Ok(result)
    }

    /// ⭐ Phase 3: 极简记忆注入格式（token优化）
    /// 
    /// 优化：去除冗长说明，只保留核心信息
    pub fn inject_memories_to_prompt(&self, memories: &[Memory]) -> String {
        if memories.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        for (i, memory) in memories.iter().enumerate().take(5) {
            // 最多5条
            let content_str = match &memory.content {
                agent_mem_traits::Content::Text(t) => t.as_str(),
                _ => "[data]",
            };
            // 极简格式：序号 + 内容（最多80字符）
            let truncated = if content_str.len() > 80 {
                format!("{}...", &content_str[..80])
            } else {
                content_str.to_string()
            };
            lines.push(format!("{}. {}", i + 1, truncated));
        }
        
        lines.join("\n")
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

    /// ⭐ Phase 2: 综合评分系统 (relevance + importance + recency)
    /// 
    /// 借鉴mem0的最佳实践：相关性(50%) + 重要性(30%) + 时效性(20%)
    pub fn calculate_comprehensive_score(&self, memory: &Memory) -> f64 {
        let relevance = memory.score().unwrap_or(0.5); // 相似度分数
        let importance = memory.importance().unwrap_or(0.5);
        
        // 时效性衰减：使用指数衰减，半衰期为30天
        use chrono::Utc;
        let now = Utc::now();
        let age_seconds = (now - memory.metadata.created_at).num_seconds();
        let age_days = age_seconds as f64 / 86400.0;
        let recency = if age_days >= 0.0 {
            (-age_days / 30.0).exp() // 指数衰减，30天半衰期
        } else {
            1.0 // 未来时间（时钟偏差），默认1.0
        };
        
        // 综合评分：0.5 * relevance + 0.3 * importance + 0.2 * recency
        0.5 * relevance + 0.3 * importance + 0.2 * recency
    }
    
    /// 按综合评分排序记忆（Phase 2优化）
    pub fn sort_memories(&self, mut memories: Vec<Memory>) -> Vec<Memory> {
        if self.config.sort_by_importance {
            // Phase 2: 使用综合评分代替单一importance
            memories.sort_by(|a, b| {
                let score_a = self.calculate_comprehensive_score(a);
                let score_b = self.calculate_comprehensive_score(b);
                score_b
                    .partial_cmp(&score_a)
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
                let importance = m.importance().unwrap_or(0.0);
                let keep = importance >= self.config.relevance_threshold as f64;
                info!(
                    "  Memory importance={:.3}, threshold={:.3}, keep={}",
                    importance, self.config.relevance_threshold, keep
                );
                keep
            })
            .collect();

        info!("🔍 filter_by_relevance: output={} memories", filtered.len());
        filtered
    }
    
    /// ⭐ Phase 5: 记忆去重
    /// 移除内容相似的重复记忆
    pub fn deduplicate_memories(&self, memories: Vec<Memory>) -> Vec<Memory> {
        use std::collections::HashSet;
        
        let mut seen_content = HashSet::new();
        let mut dedup = Vec::new();
        
        for memory in memories {
            let content_key = match &memory.content {
                agent_mem_traits::Content::Text(t) => {
                    // 使用前100字符作为去重key，确保字符边界正确
                    if t.len() > 100 {
                        // 使用char_indices找到安全的字符边界
                        let mut char_count = 0;
                        let mut byte_index = 0;
                        for (i, _) in t.char_indices() {
                            if char_count >= 100 {
                                break;
                            }
                            char_count += 1;
                            byte_index = i;
                        }
                        &t[..byte_index]
                    } else {
                        t.as_str()
                    }
                }
                _ => continue,
            };
            
            if seen_content.insert(content_key.to_string()) {
                dedup.push(memory);
            } else {
                debug!("🔄 Deduplicate: skipping duplicate memory");
            }
        }
        
        info!(
            "🔄 Deduplicate: {} → {} memories",
            seen_content.len() + (dedup.len() - seen_content.len()),
            dedup.len()
        );
        dedup
    }
    
    /// ⭐ Phase 5: 记忆压缩（简化版）
    /// 当记忆数量过多时，只保留最重要的
    pub fn compress_memories(&self, memories: Vec<Memory>) -> Vec<Memory> {
        if !self.config.enable_compression || memories.len() <= self.config.compression_threshold {
            return memories;
        }
        
        info!(
            "📦 Compression: {} memories exceed threshold {}, keeping top {}",
            memories.len(),
            self.config.compression_threshold,
            self.config.compression_threshold / 2
        );
        
        // 简单策略：只保留最重要的前N条
        let keep_count = self.config.compression_threshold / 2;
        let mut result: Vec<Memory> = memories.into_iter().take(keep_count).collect();
        
        info!(
            "📦 Compressed: kept {} most important memories",
            result.len()
        );
        result
    }

    /// ✅ Task 1.2: 记录查询统计信息
    /// 用于监控早停优化效果
    fn record_query_stats(&self, actual_queries: usize, saved_queries: usize) {
        if saved_queries > 0 {
            info!(
                "📊 Query optimization: executed {} queries, saved {} queries ({:.1}% reduction)",
                actual_queries,
                saved_queries,
                (saved_queries as f64 / (actual_queries + saved_queries) as f64) * 100.0
            );
        }
    }
}
