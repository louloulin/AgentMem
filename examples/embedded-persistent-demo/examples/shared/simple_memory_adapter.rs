use agent_mem_config::MemoryConfig;
use agent_mem_traits::{MemoryItem, MemoryType, Result, Session};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 轻量包装，兼容历史 SimpleMemory API（纯内存实现）
#[derive(Clone)]
pub struct SimpleMemory {
    memories: Arc<RwLock<Vec<MemoryItem>>>,
    default_user_id: Option<String>,
    default_agent_id: String,
    next_id: Arc<RwLock<u64>>,
    #[allow(dead_code)]
    config: Option<MemoryConfig>,
}

impl SimpleMemory {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            memories: Arc::new(RwLock::new(Vec::new())),
            default_user_id: None,
            default_agent_id: "default".to_string(),
            next_id: Arc::new(RwLock::new(1)),
            config: None,
        })
    }

    pub async fn with_config(config: MemoryConfig) -> Result<Self> {
        let mut instance = Self::new().await?;
        instance.config = Some(config);
        Ok(instance)
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.default_user_id = Some(user_id.into());
        self
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.default_agent_id = agent_id.into();
        self
    }

    pub async fn add(&self, content: impl Into<String>) -> Result<String> {
        self.add_with_metadata(content, None).await
    }

    pub async fn add_with_metadata(
        &self,
        content: impl Into<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let mut next_id = self.next_id.write().await;
        let id = format!("mem_{}", *next_id);
        *next_id += 1;
        drop(next_id);

        let now = Utc::now();
        let metadata_map = metadata
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, Value::String(v)))
            .collect();

        let session = Session::new()
            .with_user_id(self.default_user_id.clone())
            .with_agent_id(Some(self.default_agent_id.clone()));

        let memory = MemoryItem {
            id: id.clone(),
            content: content.into(),
            hash: None,
            metadata: metadata_map,
            score: None,
            created_at: now,
            updated_at: Some(now),
            session,
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: self.default_agent_id.clone(),
            user_id: self.default_user_id.clone(),
            importance: 0.5,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        };

        let mut memories = self.memories.write().await;
        memories.push(memory);

        Ok(id)
    }

    pub async fn search(&self, query: impl Into<String>) -> Result<Vec<MemoryItem>> {
        self.search_with_limit(query, 10).await
    }

    pub async fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<MemoryItem>> {
        let query = query.into().to_lowercase();
        let memories = self.memories.read().await;

        let mut results: Vec<MemoryItem> = memories
            .iter()
            .filter(|m| query.is_empty() || m.content.to_lowercase().contains(&query))
            .filter(|m| match &self.default_user_id {
                Some(user_id) => m.user_id.as_ref() == Some(user_id),
                None => true,
            })
            .filter(|m| m.agent_id == self.default_agent_id)
            .cloned()
            .collect();

        results.truncate(limit);
        Ok(results)
    }

    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        let memories = self.memories.read().await;
        Ok(memories.clone())
    }
}
