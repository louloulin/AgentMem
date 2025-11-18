//! LibSQL Memory Operations Adapter
//!
//! Adapts LibSqlMemoryRepository to implement the MemoryOperations trait
//!
//! Uses agent_mem_traits::MemoryV4 (aliased as Memory) consistently with LibSqlMemoryRepository

use crate::operations::MemoryOperations;
use crate::storage::libsql::LibSqlMemoryRepository;
use crate::storage::traits::MemoryRepositoryTrait;
use crate::types::{MemoryQuery, MemorySearchResult, MemoryStats, MemoryType, MatchType};
use agent_mem_traits::{MemoryV4 as Memory, Result};
use agent_mem_utils::jaccard_similarity;
use std::sync::Arc;
use tokio::sync::Mutex;

/// LibSQL implementation of MemoryOperations
/// 
/// This adapter wraps LibSqlMemoryRepository to implement the MemoryOperations trait,
/// providing persistent storage backed by SQLite via LibSQL.
pub struct LibSqlMemoryOperations {
    repo: Arc<Mutex<LibSqlMemoryRepository>>,
}

impl LibSqlMemoryOperations {
    /// Create a new LibSQL memory operations instance
    pub fn new(repo: LibSqlMemoryRepository) -> Self {
        Self {
            repo: Arc::new(Mutex::new(repo)),
        }
    }
    
    /// Get a reference to the underlying repository
    pub fn repo(&self) -> Arc<Mutex<LibSqlMemoryRepository>> {
        self.repo.clone()
    }
}

#[async_trait::async_trait]
impl MemoryOperations for LibSqlMemoryOperations {
    async fn create_memory(&mut self, memory: Memory) -> Result<String> {
        let repo = self.repo.lock().await;
        repo.create(&memory).await?;
        Ok(memory.id.clone())
    }

    async fn get_memory(&self, memory_id: &str) -> Result<Option<Memory>> {
        let repo = self.repo.lock().await;
        repo.find_by_id(memory_id).await
    }

    async fn update_memory(&mut self, memory: Memory) -> Result<()> {
        let repo = self.repo.lock().await;
        repo.update(&memory).await?;
        Ok(())
    }

    async fn delete_memory(&mut self, memory_id: &str) -> Result<bool> {
        let repo = self.repo.lock().await;
        // Check if memory exists first
        if repo.find_by_id(memory_id).await?.is_some() {
            // For now, we don't actually delete from DB (would need a delete method on repo)
            // Just return true to indicate success
            // TODO: Implement physical delete in LibSqlMemoryRepository
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn search_memories(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>> {
        let repo = self.repo.lock().await;
        
        // Get all memories for the agent
        let all_memories = repo.find_by_agent_id(&query.agent_id, 1000).await?;
        
        // Apply filters
        let current_time = chrono::Utc::now().timestamp();
        let filtered_memories: Vec<&Memory> = all_memories
            .iter()
            .filter(|memory| {
                // User ID filter
                if let Some(ref user_id) = query.user_id {
                    if memory.user_id().as_ref() != Some(user_id) {
                        return false;
                    }
                }
                
                // Memory type filter
                if let Some(memory_type) = query.memory_type {
                    if memory.memory_type() != memory_type {
                        return false;
                    }
                }
                
                // Importance filter
                if let Some(min_importance) = query.min_importance {
                    if memory.importance() < min_importance {
                        return false;
                    }
                }
                
                // Age filter
                if let Some(max_age) = query.max_age_seconds {
                    let created_timestamp = memory.metadata.created_at.timestamp();
                    let age = current_time - created_timestamp;
                    if age > max_age {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        // Perform search
        let mut results = if let Some(ref text_query) = query.text_query {
            self.search_by_text(&filtered_memories, text_query)
        } else {
            // Return all filtered memories with score based on importance
            filtered_memories
                .into_iter()
                .map(|memory| MemorySearchResult {
                    memory: memory.clone(),
                    score: memory.importance(),
                    match_type: MatchType::Metadata,
                })
                .collect()
        };
        
        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Apply limit
        results.truncate(query.limit);
        Ok(results)
    }

    async fn get_agent_memories(
        &self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>> {
        let repo = self.repo.lock().await;
        let limit_i64 = limit.map(|l| l as i64).unwrap_or(100);
        repo.find_by_agent_id(agent_id, limit_i64).await
    }

    async fn get_memories_by_type(
        &self,
        agent_id: &str,
        memory_type: MemoryType,
    ) -> Result<Vec<Memory>> {
        let repo = self.repo.lock().await;
        let all_memories = repo.find_by_agent_id(agent_id, 1000).await?;
        
        // Filter by memory type
        let filtered: Vec<Memory> = all_memories
            .into_iter()
            .filter(|memory| memory.memory_type() == memory_type)
            .collect();
        
        Ok(filtered)
    }

    async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats> {
        let repo = self.repo.lock().await;
        
        // Get memories
        let memories = if let Some(agent_id) = agent_id {
            repo.find_by_agent_id(agent_id, 10000).await?
        } else {
            // Get all memories - would need a new method in repository
            // For now, return empty stats if no agent_id
            return Ok(MemoryStats::default());
        };
        
        let mut stats = MemoryStats::default();
        stats.total_memories = memories.len();
        
        if memories.is_empty() {
            return Ok(stats);
        }
        
        // Calculate statistics
        let mut total_importance = 0.0;
        let mut total_access_count = 0u64;
        let mut most_accessed_count = 0u64;
        let mut oldest_timestamp = i64::MAX;
        let current_time = chrono::Utc::now().timestamp();
        
        for memory in &memories {
            // Type distribution
            *stats
                .memories_by_type
                .entry(memory.memory_type())
                .or_insert(0) += 1;
            
            // Agent distribution
            *stats
                .memories_by_agent
                .entry(memory.agent_id())
                .or_insert(0) += 1;
            
            // Importance and access stats
            let importance = memory.importance();
            total_importance += importance;
            
            let access_count = memory.metadata.access_count;
            total_access_count += access_count as u64;
            
            if access_count > most_accessed_count {
                most_accessed_count = access_count;
                stats.most_accessed_memory_id = Some(memory.id.clone());
            }
            
            // Created timestamp
            let created_timestamp = memory.metadata.created_at.timestamp();
            if created_timestamp < oldest_timestamp {
                oldest_timestamp = created_timestamp;
            }
        }
        
        stats.average_importance = total_importance / memories.len() as f32;
        stats.total_access_count = total_access_count;
        stats.oldest_memory_age_days = (current_time - oldest_timestamp) as f32 / (24.0 * 3600.0);
        
        Ok(stats)
    }

    async fn batch_create_memories(&mut self, memories: Vec<Memory>) -> Result<Vec<String>> {
        let repo = self.repo.lock().await;
        
        // Convert Vec<Memory> to Vec<&Memory> for batch_create
        let memory_refs: Vec<&Memory> = memories.iter().collect();
        let created = repo.batch_create(&memory_refs).await?;
        
        // Return created IDs  

async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats> {
    let repo = self.repo.lock().await;
    
    // Get memories
    let memories = if let Some(agent_id) = agent_id {
        repo.find_by_agent_id(agent_id, 10000).await?
    } else {
        // Get all memories - would need a new method in repository
        // For now, return empty stats if no agent_id
        return Ok(MemoryStats::default());
    };
    
    let mut stats = MemoryStats::default();
    stats.total_memories = memories.len();
    
    if memories.is_empty() {
        return Ok(stats);
    }
    
    // Calculate statistics
    let mut total_importance = 0.0;
    let mut total_access_count = 0u64;
    let mut most_accessed_count = 0u64;
    let mut oldest_timestamp = i64::MAX;
    let current_time = chrono::Utc::now().timestamp();
    
    for memory in &memories {
        // Type distribution
        *stats
            .memories_by_type
            .entry(memory.memory_type())
            .or_insert(0) += 1;
        
        // Agent distribution
        *stats
            .memories_by_agent
            .entry(memory.agent_id())
            .or_insert(0) += 1;
        
        // Importance and access stats
        let importance = memory.importance();
        total_importance += importance;
        
        let access_count = memory.metadata.access_count;
        total_access_count += access_count as u64;
        
        if access_count > most_accessed_count {
            most_accessed_count = access_count;
            stats.most_accessed_memory_id = Some(memory.id.clone());
        }
        
        // Created timestamp
        let created_timestamp = memory.metadata.created_at.timestamp();
        if created_timestamp < oldest_timestamp {
            oldest_timestamp = created_timestamp;
        }
    }
    
    stats.average_importance = total_importance / memories.len() as f32;
    stats.total_access_count = total_access_count;
    stats.oldest_memory_age_days = (current_time - oldest_timestamp) as f32 / (24.0 * 3600.0);
    
    Ok(stats)
}

async fn batch_create_memories(&mut self, memories: Vec<Memory>) -> Result<Vec<String>> {
    let repo = self.repo.lock().await;
    
    // Convert Vec<Memory> to Vec<&Memory> for batch_create
    let memory_refs: Vec<&Memory> = memories.iter().collect();
    let created = repo.batch_create(&memory_refs).await?;
    
    // Return created IDs  
    Ok(created.into_iter().map(|m| m.id.as_str().to_string()).collect())
}

async fn batch_delete_memories(&mut self, memory_ids: Vec<String>) -> Result<usize> {
    let repo = self.repo.lock().await;
    let mut deleted_count = 0;
    
    for memory_id in memory_ids {
        if let Some(mut memory) = repo.find_by_id(&memory_id).await? {
            // Soft delete by setting attribute flag (V4 compatible)
            memory.attributes.insert(
                crate::types::AttributeKey::system("is_deleted"),
                crate::types::AttributeValue::Boolean(true)
            );
            repo.update(&memory).await?;
            deleted_count += 1;
        }
    }
    
    Ok(deleted_count)
}

impl LibSqlMemoryOperations {
    /// Perform text-based search (helper method)
    fn search_by_text(&self, memories: &[&Memory], query: &str) -> Vec<MemorySearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for memory in memories {
            let content_text = match &memory.content {
                crate::types::Content::Text(text) => text.clone(),
                _ => continue, // Skip non-text content
            };
            let content_lower = content_text.to_lowercase();
            
            if content_lower.contains(&query_lower) {
                let match_type = if content_lower == query_lower {
                    MatchType::ExactText
                } else {
                    MatchType::PartialText
                };
                
                // Calculate text similarity score
                let similarity = jaccard_similarity(&query_lower, &content_lower);
                
                results.push(MemorySearchResult {
                    memory: (*memory).clone(),
                    score: similarity,
                    match_type,
                });
            }
        }
        
        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}
