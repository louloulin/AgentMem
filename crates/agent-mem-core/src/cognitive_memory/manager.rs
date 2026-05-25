//! CognitiveMemoryManager - 统一认知记忆管理器

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    managers::{
        ContextualMemoryManager, CoreMemoryManager, KnowledgeVaultManager, ResourceMemoryManager,
    },
    types::{Content, Memory, MemoryType},
    CoreResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitiveOperation {
    Add {
        content: String,
        memory_type: MemoryType,
        importance: Option<f32>,
        metadata: Option<HashMap<String, String>>,
    },
    Retrieve {
        query: String,
        memory_types: Option<Vec<MemoryType>>,
        limit: Option<usize>,
    },
    Update {
        id: String,
        content: Option<String>,
        importance: Option<f32>,
    },
    Delete {
        id: String,
    },
    ExtractPattern {
        memory_type: Option<MemoryType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveResult {
    pub memories: Vec<Memory>,
    pub stats: CognitiveStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CognitiveStats {
    pub total_memories: usize,
    pub by_type: HashMap<String, usize>,
    pub operation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMemoryConfig {
    pub enable_core_memory: bool,
    pub enable_contextual_memory: bool,
    pub enable_resource_memory: bool,
    pub enable_knowledge_vault: bool,
    pub default_importance: f32,
    pub max_memories: usize,
    pub enable_text_search: bool,
    pub search_threshold: f32,
}

impl Default for CognitiveMemoryConfig {
    fn default() -> Self {
        Self {
            enable_core_memory: true,
            enable_contextual_memory: true,
            enable_resource_memory: true,
            enable_knowledge_vault: true,
            default_importance: 0.5,
            max_memories: 10000,
            enable_text_search: true,
            search_threshold: 0.3,
        }
    }
}

pub struct CognitiveMemoryManager {
    config: CognitiveMemoryConfig,
    core_memory: Arc<RwLock<CoreMemoryManager>>,
    contextual_memory: Arc<RwLock<ContextualMemoryManager>>,
    resource_memory: Arc<RwLock<ResourceMemoryManager>>,
    knowledge_vault: Arc<RwLock<KnowledgeVaultManager>>,
    memories: Arc<RwLock<HashMap<String, Memory>>>,
}

impl CognitiveMemoryManager {
    pub async fn new(config: CognitiveMemoryConfig) -> CoreResult<Self> {
        let core_memory = Arc::new(RwLock::new(CoreMemoryManager::new()));
        let contextual_memory = Arc::new(RwLock::new(ContextualMemoryManager::new(
            crate::managers::ContextualMemoryConfig::default(),
        )));
        let resource_memory = Arc::new(RwLock::new(
            ResourceMemoryManager::new()
                .map_err(|e| crate::CoreError::Internal("{e}".to_string()))?,
        ));
        let knowledge_vault = Arc::new(RwLock::new(
            KnowledgeVaultManager::new(crate::managers::KnowledgeVaultConfig::default())
                .map_err(|e| crate::CoreError::Internal("{e}".to_string()))?,
        ));

        Ok(Self {
            config,
            core_memory,
            contextual_memory,
            resource_memory,
            knowledge_vault,
            memories: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn with_default_config() -> CoreResult<Self> {
        Self::new(CognitiveMemoryConfig::default()).await
    }

    pub async fn add_memory(&self, memory: Memory) -> CoreResult<String> {
        let id = Uuid::new_v4().to_string();
        let mut memories = self.memories.write().await;
        memories.insert(id.clone(), memory);
        Ok(id)
    }

    pub async fn add_memories(&self, memories: Vec<Memory>) -> CoreResult<Vec<String>> {
        let mut result = Vec::new();
        for memory in memories {
            let id = self.add_memory(memory).await?;
            result.push(id);
        }
        Ok(result)
    }

    pub async fn get_memory(&self, id: &str) -> CoreResult<Option<Memory>> {
        let memories = self.memories.read().await;
        Ok(memories.get(id).cloned())
    }

    fn get_text_content(content: &Content) -> String {
        match content {
            Content::Text(s) => s.clone(),
            Content::Image { url, .. } => url.clone(),
            Content::Audio { url, transcript } => {
                let mut s = url.clone();
                if let Some(t) = transcript {
                    s.push(' ');
                    s.push_str(t);
                }
                s
            }
            Content::Video { url, summary } => {
                let mut s = url.clone();
                if let Some(sm) = summary {
                    s.push(' ');
                    s.push_str(sm);
                }
                s
            }
            Content::Structured(v) => v.to_string(),
            Content::Mixed(items) => items
                .iter()
                .map(|i| Self::get_text_content(i))
                .collect::<Vec<_>>()
                .join(" "),
        }
        .to_lowercase()
    }

    pub async fn retrieve(
        &self,
        query: &str,
        memory_types: Option<Vec<MemoryType>>,
        limit: usize,
    ) -> CoreResult<Vec<Memory>> {
        let memories = self.memories.read().await;

        // 如果没有查询文本且没有类型过滤，返回所有记忆
        if query.is_empty() && memory_types.is_none() {
            let mut results: Vec<Memory> = memories.values().cloned().collect();
            results.sort_by(|a, b| {
                b.importance()
                    .partial_cmp(&a.importance())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(limit);
            return Ok(results);
        }

        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<(Memory, f32)> = memories
            .values()
            .filter(|m| {
                if let Some(ref types) = memory_types {
                    let mem_type = m.memory_type();
                    return types.iter().any(|t| *t == mem_type);
                }
                true
            })
            .filter_map(|m| {
                if self.config.enable_text_search && !query.is_empty() {
                    let content_str = Self::get_text_content(&m.content);

                    let mut score = 0.0f32;

                    for word in &query_words {
                        if content_str.contains(word) {
                            score += 0.3;
                            if content_str.contains(&format!(" {} ", word))
                                || content_str.starts_with(&format!("{} ", word))
                                || content_str.ends_with(&format!(" {}", word))
                            {
                                score += 0.2;
                            }
                            for w in content_str.split_whitespace() {
                                if w.starts_with(word) && w.len() > word.len() {
                                    score += 0.1;
                                }
                            }
                        }
                    }

                    score += m.importance() * 0.2;

                    if score >= self.config.search_threshold {
                        return Some((m.clone(), score));
                    }
                }

                if query.is_empty() {
                    return Some((m.clone(), m.importance()));
                }

                None
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let final_results: Vec<Memory> = results.into_iter().take(limit).map(|(m, _)| m).collect();

        Ok(final_results)
    }

    #[allow(unused_variables)]
    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<String>,
        importance: Option<f32>,
    ) -> CoreResult<bool> {
        let mut memories = self.memories.write().await;
        Ok(memories.contains_key(id))
    }

    pub async fn delete_memory(&self, id: &str) -> CoreResult<bool> {
        let mut memories = self.memories.write().await;
        Ok(memories.remove(id).is_some())
    }

    pub async fn get_stats(&self) -> CoreResult<CognitiveStats> {
        let memories = self.memories.read().await;
        let mut by_type: HashMap<String, usize> = HashMap::new();

        for memory in memories.values() {
            let mem_type = memory.memory_type();
            let type_str = mem_type.as_str();
            *by_type.entry(type_str.to_string()).or_insert(0) += 1;
        }

        Ok(CognitiveStats {
            total_memories: memories.len(),
            by_type,
            operation_time_ms: 0,
        })
    }

    pub fn core_memory_manager(&self) -> Arc<RwLock<CoreMemoryManager>> {
        self.core_memory.clone()
    }

    pub fn contextual_memory_manager(&self) -> Arc<RwLock<ContextualMemoryManager>> {
        self.contextual_memory.clone()
    }

    pub fn resource_memory_manager(&self) -> Arc<RwLock<ResourceMemoryManager>> {
        self.resource_memory.clone()
    }

    pub fn knowledge_vault_manager(&self) -> Arc<RwLock<KnowledgeVaultManager>> {
        self.knowledge_vault.clone()
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cognitive_memory_manager_creation() {
        let manager = CognitiveMemoryManager::with_default_config().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_retrieve_memory() {
        let manager = CognitiveMemoryManager::with_default_config().await.unwrap();

        let memory = Memory::new(
            "test-agent".to_string(),
            None,
            MemoryType::Semantic,
            "Test content".to_string(),
            0.5,
        );

        let id = manager.add_memory(memory.clone()).await.unwrap();
        assert!(!id.is_empty());

        let retrieved = manager.get_memory(&id).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_retrieve_with_filter() {
        let manager = CognitiveMemoryManager::with_default_config().await.unwrap();

        for i in 0..5 {
            let memory = Memory::new(
                "test-agent".to_string(),
                None,
                MemoryType::Semantic,
                format!("Content {}", i),
                0.5,
            );
            let _ = manager.add_memory(memory).await;
        }

        let results = manager.retrieve("Content", None, 10).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let manager = CognitiveMemoryManager::with_default_config().await.unwrap();

        let memory = Memory::new(
            "test-agent".to_string(),
            None,
            MemoryType::Episodic,
            "To be deleted".to_string(),
            0.5,
        );
        let id = manager.add_memory(memory).await.unwrap();

        let deleted = manager.delete_memory(&id).await.unwrap();
        assert!(deleted);

        let result = manager.get_memory(&id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = CognitiveMemoryManager::with_default_config().await.unwrap();

        for i in 0..3 {
            let memory = Memory::new(
                "test-agent".to_string(),
                None,
                MemoryType::Semantic,
                format!("Stats {}", i),
                0.5,
            );
            let _ = manager.add_memory(memory).await;
        }

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_memories, 3);
    }

    #[tokio::test]
    async fn test_text_search() {
        let manager = CognitiveMemoryManager::with_default_config().await.unwrap();

        let memories = vec![
            ("Rust programming language", MemoryType::Semantic, 0.9),
            ("Python for data science", MemoryType::Semantic, 0.8),
            ("JavaScript web development", MemoryType::Semantic, 0.7),
        ];

        for (content, mem_type, importance) in memories {
            let memory = Memory::new(
                "test-agent".to_string(),
                None,
                mem_type,
                content.to_string(),
                importance,
            );
            let _ = manager.add_memory(memory).await;
        }

        let results = manager.retrieve("rust", None, 10).await.unwrap();
        assert!(
            results.len() >= 1,
            "Should find at least 1 result for 'rust'"
        );
    }
}
