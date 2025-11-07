//! Core memory manager implementation

use crate::{
    history::{HistoryConfig, MemoryHistory},
    lifecycle::{LifecycleConfig, MemoryLifecycle},
    managers::deduplication::MemoryDeduplicator,
    operations::{InMemoryOperations, MemoryOperations},
    types::{Memory, MemoryQuery, MemorySearchResult, MemoryStats, MemoryType},
};
use agent_mem_config::MemoryConfig;
use agent_mem_llm::LLMProvider;
use agent_mem_traits::{
    AgentMemError, DecisionEngine, ExtractedFact, FactExtractor, HistoryEntry, MemoryActionType,
    MemoryEvent, MemoryItem, MemoryProvider, Message, Result, Session,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Core memory manager
pub struct MemoryManager {
    /// Memory operations backend
    operations: Arc<RwLock<Box<dyn MemoryOperations + Send + Sync>>>,
    /// Memory lifecycle manager
    lifecycle: Arc<RwLock<MemoryLifecycle>>,
    /// Memory history tracker
    history: Arc<RwLock<MemoryHistory>>,
    /// Configuration
    config: MemoryConfig,

    // 智能组件 (可选)
    /// 事实提取器 (用于从内容中提取结构化事实)
    fact_extractor: Option<Arc<dyn FactExtractor>>,
    /// 决策引擎 (用于智能决定记忆操作)
    decision_engine: Option<Arc<dyn DecisionEngine>>,
    /// 去重器 (用于检测和合并重复记忆)
    deduplicator: Option<Arc<MemoryDeduplicator>>,
    /// LLM 提供商 (用于智能功能)
    llm_provider: Option<Arc<dyn LLMProvider>>,
}

impl MemoryManager {
    /// Create a new memory manager with default configuration
    ///
    /// **Note**: This uses in-memory storage which is not persistent.
    /// For production use with persistent storage, use `with_operations()` or use
    /// the Agent-based API (CoreAgent, EpisodicAgent, etc.) which supports persistent storage.
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    /// Create a new memory manager with custom configuration
    ///
    /// **Note**: This uses in-memory storage which is not persistent.
    /// For production use with persistent storage, use `with_operations()` or use
    /// the Agent-based API (CoreAgent, EpisodicAgent, etc.) which supports persistent storage.
    pub fn with_config(config: MemoryConfig) -> Self {
        let operations: Box<dyn MemoryOperations + Send + Sync> =
            Box::new(InMemoryOperations::new());
        let lifecycle = MemoryLifecycle::with_default_config();
        let history = MemoryHistory::with_default_config();

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config,
            fact_extractor: None,
            decision_engine: None,
            deduplicator: None,
            llm_provider: None,
        }
    }

    /// Create a new memory manager with custom operations backend
    ///
    /// This allows you to provide your own MemoryOperations implementation,
    /// such as one backed by a persistent database.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use agent_mem_core::manager::MemoryManager;
    /// use agent_mem_config::MemoryConfig;
    ///
    /// // Create your custom operations implementation
    /// let operations = MyPersistentOperations::new(db_connection);
    ///
    /// // Create manager with custom operations
    /// let manager = MemoryManager::with_operations(
    ///     MemoryConfig::default(),
    ///     Box::new(operations),
    /// );
    /// ```
    pub fn with_operations(
        config: MemoryConfig,
        operations: Box<dyn MemoryOperations + Send + Sync>,
    ) -> Self {
        let lifecycle = MemoryLifecycle::with_default_config();
        let history = MemoryHistory::with_default_config();

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config,
            fact_extractor: None,
            decision_engine: None,
            deduplicator: None,
            llm_provider: None,
        }
    }

    /// Create a new memory manager with custom lifecycle and history configs
    pub fn with_custom_configs(
        memory_config: MemoryConfig,
        lifecycle_config: LifecycleConfig,
        history_config: HistoryConfig,
    ) -> Self {
        let operations: Box<dyn MemoryOperations + Send + Sync> =
            Box::new(InMemoryOperations::new());
        let lifecycle = MemoryLifecycle::new(lifecycle_config);
        let history = MemoryHistory::new(history_config);

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config: memory_config,
            fact_extractor: None,
            decision_engine: None,
            deduplicator: None,
            llm_provider: None,
        }
    }

    /// Create a new memory manager with intelligent components
    pub fn with_intelligent_components(
        config: MemoryConfig,
        fact_extractor: Option<Arc<dyn FactExtractor>>,
        decision_engine: Option<Arc<dyn DecisionEngine>>,
        llm_provider: Option<Arc<dyn LLMProvider>>,
    ) -> Self {
        let operations: Box<dyn MemoryOperations + Send + Sync> =
            Box::new(InMemoryOperations::new());
        let lifecycle = MemoryLifecycle::with_default_config();
        let history = MemoryHistory::with_default_config();

        // 初始化去重器 (如果启用)
        let deduplicator = if config.intelligence.enable_deduplication {
            use crate::managers::deduplication::DeduplicationConfig as DedupConfig;

            let dedup_config = DedupConfig {
                similarity_threshold: config.intelligence.deduplication.similarity_threshold,
                time_window_seconds: config
                    .intelligence
                    .deduplication
                    .time_window_seconds
                    .unwrap_or(3600),
                batch_size: 100,
                enable_intelligent_merge: true,
                preserve_history: true,
            };

            Some(Arc::new(MemoryDeduplicator::new(dedup_config)))
        } else {
            None
        };

        Self {
            operations: Arc::new(RwLock::new(operations)),
            lifecycle: Arc::new(RwLock::new(lifecycle)),
            history: Arc::new(RwLock::new(history)),
            config,
            fact_extractor,
            decision_engine,
            deduplicator,
            llm_provider,
        }
    }

    /// Add a new memory (智能版本 - 支持事实提取和决策引擎)
    pub async fn add_memory(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        // 检查是否启用智能提取
        if self.config.intelligence.enable_intelligent_extraction
            && self.fact_extractor.is_some()
            && self.decision_engine.is_some()
        {
            info!("Using intelligent memory addition with fact extraction and decision engine");
            return self
                .add_memory_intelligent(
                    agent_id,
                    user_id,
                    content,
                    memory_type,
                    importance,
                    metadata,
                )
                .await;
        }

        // 降级到简单流程
        debug!("Using simple memory addition (intelligent features disabled)");
        self.add_memory_simple(
            agent_id,
            user_id,
            content,
            memory_type,
            importance,
            metadata,
        )
        .await
    }

    /// 简单的记忆添加流程 (原始逻辑)
    async fn add_memory_simple(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String> {
        let mut memory = Memory::new(
            agent_id,
            user_id,
            memory_type.unwrap_or(MemoryType::Episodic),
            content,
            importance.unwrap_or(0.5),
        );

        if let Some(metadata) = metadata {
            for (key, value) in metadata {
                memory.add_metadata(key, value);
            }
        }

        // Register with lifecycle manager
        {
            let mut lifecycle = self.lifecycle.write().await;
            let memory_item = agent_mem_traits::MemoryItem::from(memory.clone());
            lifecycle.register_memory(&memory_item)?;
        }

        // Record creation in history
        {
            let mut history = self.history.write().await;
            history.record_creation(&memory)?;
        }

        // Store the memory
        let mut operations = self.operations.write().await;
        operations.create_memory(memory).await
    }

    /// 智能记忆添加流程 (使用事实提取和决策引擎)
    async fn add_memory_intelligent(
        &self,
        agent_id: String,
        user_id: Option<String>,
        content: String,
        memory_type: Option<MemoryType>,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        info!(
            "Starting intelligent memory addition for agent: {}",
            agent_id
        );

        // 步骤 1: 提取事实
        let facts = self.extract_facts_from_content(&content).await?;
        info!("Extracted {} facts from content", facts.len());

        if facts.is_empty() {
            warn!("No facts extracted, falling back to simple addition");
            return self
                .add_memory_simple(
                    agent_id,
                    user_id,
                    content,
                    memory_type,
                    importance,
                    metadata,
                )
                .await;
        }

        // 步骤 2: 对每个事实进行决策和执行
        let mut memory_ids = Vec::new();
        for (idx, fact) in facts.iter().enumerate() {
            debug!(
                "Processing fact {}/{}: {}",
                idx + 1,
                facts.len(),
                fact.content
            );

            // 查找相似记忆
            let similar_memories = self
                .find_similar_memories_for_fact(fact, &agent_id, &user_id)
                .await?;

            // 决策
            let decision = self.make_decision_for_fact(fact, &similar_memories).await?;

            // 执行决策
            let memory_id = self
                .execute_memory_action(
                    decision,
                    &agent_id,
                    &user_id,
                    &memory_type,
                    &importance,
                    &metadata,
                )
                .await?;

            if let Some(id) = memory_id {
                memory_ids.push(id);
            }
        }

        // 返回第一个记忆ID (主记忆)
        Ok(memory_ids.first().cloned().unwrap_or_else(|| {
            warn!("No memories created, returning empty ID");
            String::new()
        }))
    }

    /// Get a memory by ID
    pub async fn get_memory(&self, memory_id: &str) -> Result<Option<Memory>> {
        // Check if memory is accessible
        {
            let lifecycle = self.lifecycle.read().await;
            if !lifecycle.is_accessible(memory_id) {
                return Ok(None);
            }
        }

        // Get the memory first
        let mut memory = {
            let operations = self.operations.read().await;
            operations.get_memory(memory_id).await?
        };

        if let Some(ref mut mem) = memory {
            // Record access
            mem.access();

            // Update lifecycle
            {
                let mut lifecycle = self.lifecycle.write().await;
                lifecycle.record_access(memory_id)?;
            }

            // Record in history (if enabled)
            {
                let mut history = self.history.write().await;
                history.record_access(mem)?;
            }

            // Update the memory in storage
            {
                let mut operations = self.operations.write().await;
                operations.update_memory(mem.clone()).await?;
            }
        }

        Ok(memory)
    }

    /// Update an existing memory
    pub async fn update_memory(
        &self,
        memory_id: &str,
        new_content: Option<String>,
        new_importance: Option<f32>,
        new_metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<()> {
        let operations = self.operations.read().await;
        let mut memory = operations
            .get_memory(memory_id)
            .await?
            .ok_or_else(|| AgentMemError::memory_error("Memory not found"))?;

        let old_content = memory.content.clone();
        let old_importance = memory.importance;
        let old_version = memory.version;

        // Update fields
        if let Some(content) = new_content {
            memory.update_content(content);
        }

        if let Some(importance) = new_importance {
            memory.importance = importance.clamp(0.0, 1.0);
        }

        if let Some(metadata) = new_metadata {
            for (key, value) in metadata {
                memory.add_metadata(key, value);
            }
        }

        // Record changes in history
        {
            let mut history = self.history.write().await;

            if memory.content != old_content {
                history.record_content_update(&memory, &old_content, None)?;
            }

            if memory.importance != old_importance {
                history.record_importance_change(&memory, old_importance)?;
            }
        }

        // Record lifecycle update
        {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.record_update(memory_id, old_version, memory.version)?;
        }

        // Update in storage
        drop(operations);
        let mut operations = self.operations.write().await;
        operations.update_memory(memory).await
    }

    /// Delete a memory
    pub async fn delete_memory(&self, memory_id: &str) -> Result<bool> {
        // Mark as deleted in lifecycle
        {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.delete_memory(memory_id)?;
        }

        // Delete from storage
        let mut operations = self.operations.write().await;
        operations.delete_memory(memory_id).await
    }

    /// Search memories
    pub async fn search_memories(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>> {
        let operations = self.operations.read().await;
        operations.search_memories(query).await
    }

    /// Get all memories for an agent
    pub async fn get_agent_memories(
        &self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>> {
        let operations = self.operations.read().await;
        operations.get_agent_memories(agent_id, limit).await
    }

    /// Get memories by type
    pub async fn get_memories_by_type(
        &self,
        agent_id: &str,
        memory_type: MemoryType,
    ) -> Result<Vec<Memory>> {
        let operations = self.operations.read().await;
        operations.get_memories_by_type(agent_id, memory_type).await
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats> {
        let operations = self.operations.read().await;
        operations.get_memory_stats(agent_id).await
    }

    /// Apply automatic lifecycle policies
    pub async fn apply_auto_policies(&self) -> Result<Vec<String>> {
        let operations = self.operations.read().await;
        let all_memories = operations.get_agent_memories("", None).await?; // Get all memories
        drop(operations);

        let mut lifecycle = self.lifecycle.write().await;
        let memory_items: Vec<agent_mem_traits::MemoryItem> = all_memories
            .iter()
            .map(|m| agent_mem_traits::MemoryItem::from(m.clone()))
            .collect();
        lifecycle.apply_auto_policies(&memory_items)
    }

    /// Clean up expired memories and old history
    pub async fn cleanup(&self) -> Result<(usize, usize)> {
        // Clean up history
        let history_cleaned = {
            let mut history = self.history.write().await;
            history.cleanup_old_entries()
        };

        // Clean up lifecycle events
        let lifecycle_cleaned = {
            let mut lifecycle = self.lifecycle.write().await;
            lifecycle.cleanup_old_events(30 * 24 * 3600); // 30 days
            0 // Return 0 for now, could implement actual cleanup count
        };

        Ok((history_cleaned, lifecycle_cleaned))
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoryProvider for MemoryManager {
    async fn add(&self, messages: &[Message], session: &Session) -> Result<Vec<MemoryItem>> {
        let mut results = Vec::new();

        for message in messages {
            let memory_id = self
                .add_memory(
                    session
                        .agent_id
                        .clone()
                        .unwrap_or_else(|| "default".to_string()),
                    session.user_id.clone(),
                    message.content.clone(),
                    None, // Use default memory type
                    None, // Use default importance
                    None, // No additional metadata
                )
                .await?;

            if let Some(memory) = self.get_memory(&memory_id).await? {
                results.push(memory.into());
            }
        }

        Ok(results)
    }

    async fn get(&self, memory_id: &str) -> Result<Option<MemoryItem>> {
        let memory = self.get_memory(memory_id).await?;
        Ok(memory.map(|m| m.into()))
    }

    async fn search(
        &self,
        query: &str,
        session: &Session,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        let mut memory_query = MemoryQuery::new(
            session
                .agent_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        )
        .with_text_query(query.to_string())
        .with_limit(limit);

        if let Some(ref user_id) = session.user_id {
            memory_query = memory_query.with_user_id(user_id.clone());
        }

        let results = self.search_memories(memory_query).await?;
        Ok(results.into_iter().map(|r| r.memory.into()).collect())
    }

    async fn update(&self, memory_id: &str, data: &str) -> Result<()> {
        self.update_memory(
            memory_id,
            Some(data.to_string()),
            None, // Don't update importance through this interface
            None, // No metadata updates
        )
        .await
    }

    async fn delete(&self, memory_id: &str) -> Result<()> {
        self.delete_memory(memory_id).await?;
        Ok(())
    }

    async fn history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>> {
        let history = self.history.read().await;
        if let Some(entries) = history.get_memory_history(memory_id) {
            let items: Vec<HistoryEntry> = entries
                .iter()
                .map(|entry| {
                    let event = match entry.change_type {
                        crate::history::ChangeType::Created => MemoryEvent::Create,
                        crate::history::ChangeType::ContentUpdated
                        | crate::history::ChangeType::ImportanceChanged
                        | crate::history::ChangeType::MetadataUpdated => MemoryEvent::Update,
                        crate::history::ChangeType::Deprecated => MemoryEvent::Delete,
                        _ => MemoryEvent::Update,
                    };

                    HistoryEntry {
                        id: format!("{}_{}", entry.memory_id, entry.version),
                        memory_id: entry.memory_id.clone(),
                        event,
                        timestamp: chrono::DateTime::from_timestamp(entry.timestamp, 0)
                            .unwrap_or_else(chrono::Utc::now),
                        data: Some(serde_json::json!({
                            "content": entry.content,
                            "change_type": entry.change_type.to_string(),
                            "version": entry.version
                        })),
                    }
                })
                .collect();
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_all(&self, session: &Session) -> Result<Vec<MemoryItem>> {
        let agent_id = session.agent_id.as_deref().unwrap_or("default");
        let memories = self.get_agent_memories(agent_id, None).await?;
        Ok(memories.into_iter().map(|m| m.into()).collect())
    }

    async fn reset(&self) -> Result<()> {
        // This is a destructive operation, typically used for testing
        // For now, we'll just return Ok since we don't have a reset implementation
        // TODO: Implement actual reset functionality if needed
        Ok(())
    }
}

// ============================================================================
// 智能功能辅助方法
// ============================================================================

impl MemoryManager {
    /// 从内容中提取事实
    async fn extract_facts_from_content(&self, content: &str) -> Result<Vec<ExtractedFact>> {
        let fact_extractor = self.fact_extractor.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Fact extractor not initialized".to_string())
        })?;

        // 创建消息
        let message = agent_mem_traits::Message {
            role: agent_mem_traits::MessageRole::User,
            content: content.to_string(),
            timestamp: Some(chrono::Utc::now()),
        };

        // 提取结构化事实
        match fact_extractor.extract_facts(&[message]).await {
            Ok(facts) => {
                // 过滤低置信度事实
                let min_confidence = self.config.intelligence.fact_extraction.min_confidence;
                let filtered_facts: Vec<_> = facts
                    .into_iter()
                    .filter(|f| f.confidence >= min_confidence)
                    .collect();

                debug!(
                    "Extracted {} facts (filtered from total with min confidence {})",
                    filtered_facts.len(),
                    min_confidence
                );
                Ok(filtered_facts)
            }
            Err(e) => {
                warn!(
                    "Failed to extract facts: {}, falling back to simple extraction",
                    e
                );
                // 降级: 将整个内容作为一个事实
                Ok(vec![ExtractedFact {
                    content: content.to_string(),
                    confidence: 0.5,
                    category: "knowledge".to_string(),
                    metadata: HashMap::new(),
                }])
            }
        }
    }

    /// 查找与事实相似的记忆
    async fn find_similar_memories_for_fact(
        &self,
        fact: &ExtractedFact,
        agent_id: &str,
        user_id: &Option<String>,
    ) -> Result<Vec<MemoryItem>> {
        let max_similar = self
            .config
            .intelligence
            .decision_engine
            .max_similar_memories;

        // 构建查询
        let query = MemoryQuery {
            agent_id: agent_id.to_string(),
            user_id: user_id.clone(),
            memory_type: None,
            text_query: Some(fact.content.clone()),
            vector_query: None,
            min_importance: None,
            max_age_seconds: None,
            limit: max_similar,
        };

        // 搜索相似记忆
        let results = self.search_memories(query).await?;

        // 转换为 MemoryItem 格式
        let existing_memories: Vec<MemoryItem> = results
            .into_iter()
            .map(|result| {
                let memory = &result.memory;
                // 转换 metadata: HashMap<String, String> -> HashMap<String, serde_json::Value>
                let metadata: HashMap<String, serde_json::Value> = memory
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                    .collect();

                MemoryItem {
                    id: memory.id.clone(),
                    content: memory.content.clone(),
                    hash: None,
                    metadata,
                    score: Some(result.score),
                    created_at: chrono::DateTime::from_timestamp(memory.created_at, 0)
                        .unwrap_or_else(chrono::Utc::now),
                    updated_at: Some(
                        chrono::DateTime::from_timestamp(memory.last_accessed_at, 0)
                            .unwrap_or_else(chrono::Utc::now),
                    ),
                    session: Session {
                        id: memory.agent_id.clone(),
                        user_id: memory.user_id.clone(),
                        agent_id: Some(memory.agent_id.clone()),
                        run_id: None,
                        actor_id: None,
                        created_at: chrono::Utc::now(),
                        metadata: HashMap::new(),
                    },
                    memory_type: agent_mem_traits::MemoryType::Episodic, // 默认类型
                    entities: vec![],
                    relations: vec![],
                    agent_id: memory.agent_id.clone(),
                    user_id: memory.user_id.clone(),
                    importance: memory.importance,
                    embedding: None,
                    last_accessed_at: chrono::DateTime::from_timestamp(memory.last_accessed_at, 0)
                        .unwrap_or_else(chrono::Utc::now),
                    access_count: 0,
                    expires_at: None,
                    version: 1,
                }
            })
            .collect();

        debug!(
            "Found {} similar memories for fact",
            existing_memories.len()
        );
        Ok(existing_memories)
    }

    /// 为事实做出决策
    async fn make_decision_for_fact(
        &self,
        fact: &ExtractedFact,
        similar_memories: &[MemoryItem],
    ) -> Result<MemoryActionType> {
        let decision_engine = self.decision_engine.as_ref().ok_or_else(|| {
            AgentMemError::ConfigError("Decision engine not initialized".to_string())
        })?;

        // 调用决策引擎
        match decision_engine.decide(fact, similar_memories).await {
            Ok(decision) => {
                info!(
                    "Decision made: {:?} with confidence {}",
                    decision.action, decision.confidence
                );

                // 检查决策置信度
                let min_confidence = self
                    .config
                    .intelligence
                    .decision_engine
                    .min_decision_confidence;
                if decision.confidence < min_confidence {
                    warn!(
                        "Decision confidence {} below threshold {}, defaulting to Add",
                        decision.confidence, min_confidence
                    );
                    return Ok(MemoryActionType::Add {
                        content: fact.content.clone(),
                        importance: fact.confidence,
                        metadata: fact.metadata.clone(),
                    });
                }

                Ok(decision.action)
            }
            Err(e) => {
                warn!("Decision engine failed: {}, defaulting to Add", e);
                // 降级: 默认添加
                Ok(MemoryActionType::Add {
                    content: fact.content.clone(),
                    importance: fact.confidence,
                    metadata: fact.metadata.clone(),
                })
            }
        }
    }

    /// 执行记忆操作
    async fn execute_memory_action(
        &self,
        action: MemoryActionType,
        agent_id: &str,
        user_id: &Option<String>,
        memory_type: &Option<MemoryType>,
        importance: &Option<f32>,
        metadata: &Option<HashMap<String, String>>,
    ) -> Result<Option<String>> {
        match action {
            MemoryActionType::Add {
                content,
                importance: fact_importance,
                metadata: fact_metadata,
            } => {
                info!("Executing ADD action");

                // 合并元数据
                let mut combined_metadata = metadata.clone().unwrap_or_default();
                combined_metadata.extend(fact_metadata);

                // 使用事实的重要性或默认值
                let final_importance = importance.unwrap_or(fact_importance);

                let memory_id = self
                    .add_memory_simple(
                        agent_id.to_string(),
                        user_id.clone(),
                        content,
                        *memory_type,
                        Some(final_importance),
                        Some(combined_metadata),
                    )
                    .await?;

                Ok(Some(memory_id))
            }

            MemoryActionType::Update {
                memory_id,
                new_content,
                merge_strategy,
            } => {
                info!(
                    "Executing UPDATE action for memory {} with strategy {}",
                    memory_id, merge_strategy
                );

                // 获取现有记忆
                if let Some(memory) = self.get_memory(&memory_id).await? {
                    // 根据合并策略更新内容
                    let updated_content = match merge_strategy.as_str() {
                        "replace" => new_content,
                        "append" => format!("{}\n\n{}", memory.content, new_content),
                        "merge" => format!("{}\n{}", memory.content, new_content),
                        _ => new_content, // 默认替换
                    };

                    // 更新记忆
                    self.update_memory(&memory_id, Some(updated_content), None, None)
                        .await?;
                    Ok(Some(memory_id))
                } else {
                    warn!("Memory {} not found for update, skipping", memory_id);
                    Ok(None)
                }
            }

            MemoryActionType::Delete { memory_id, reason } => {
                info!(
                    "Executing DELETE action for memory {}: {}",
                    memory_id, reason
                );
                self.delete_memory(&memory_id).await?;
                Ok(None)
            }

            MemoryActionType::Merge {
                primary_memory_id,
                secondary_memory_ids,
                merged_content,
            } => {
                info!(
                    "Executing MERGE action: merging {} memories into {}",
                    secondary_memory_ids.len(),
                    primary_memory_id
                );

                // 更新主记忆
                if self.get_memory(&primary_memory_id).await?.is_some() {
                    self.update_memory(&primary_memory_id, Some(merged_content), None, None)
                        .await?;
                }

                // 删除次要记忆
                for secondary_id in secondary_memory_ids {
                    self.delete_memory(&secondary_id).await?;
                }

                Ok(Some(primary_memory_id))
            }

            MemoryActionType::NoAction { reason } => {
                debug!("No action taken: {}", reason);
                Ok(None)
            }
        }
    }
}
