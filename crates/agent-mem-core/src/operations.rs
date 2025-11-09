//! Memory CRUD operations

use crate::types::{MatchType, Memory, MemoryQuery, MemorySearchResult, MemoryStats, MemoryType};
use agent_mem_traits::{AgentMemError, Result, Vector};
use agent_mem_utils::jaccard_similarity;

use std::collections::HashMap;

/// Memory operations interface
#[async_trait::async_trait]
pub trait MemoryOperations {
    /// Create a new memory
    async fn create_memory(&mut self, memory: Memory) -> Result<String>;

    /// Get a memory by ID
    async fn get_memory(&self, memory_id: &str) -> Result<Option<Memory>>;

    /// Update an existing memory
    async fn update_memory(&mut self, memory: Memory) -> Result<()>;

    /// Delete a memory
    async fn delete_memory(&mut self, memory_id: &str) -> Result<bool>;

    /// Search memories
    async fn search_memories(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>>;

    /// Get all memories for an agent
    async fn get_agent_memories(&self, agent_id: &str, limit: Option<usize>)
        -> Result<Vec<Memory>>;

    /// Get memories by type
    async fn get_memories_by_type(
        &self,
        agent_id: &str,
        memory_type: MemoryType,
    ) -> Result<Vec<Memory>>;

    /// Get memory statistics
    async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats>;

    /// Batch create memories
    async fn batch_create_memories(&mut self, memories: Vec<Memory>) -> Result<Vec<String>>;

    /// Batch delete memories
    async fn batch_delete_memories(&mut self, memory_ids: Vec<String>) -> Result<usize>;
}

/// In-memory implementation of memory operations
pub struct InMemoryOperations {
    memories: HashMap<String, Memory>,
    agent_index: HashMap<String, Vec<String>>,
    type_index: HashMap<MemoryType, Vec<String>>,
}

impl InMemoryOperations {
    pub fn new() -> Self {
        Self {
            memories: HashMap::new(),
            agent_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }

    /// Update indices when adding a memory
    fn update_indices(&mut self, memory: &Memory) {
        // Update agent index
        self.agent_index
            .entry(memory.agent_id())
            .or_default()
            .push(memory.id.clone());

        // Update type index
        self.type_index
            .entry(memory.memory_type())
            .or_default()
            .push(memory.id.clone());
    }

    /// Remove from indices when deleting a memory
    fn remove_from_indices(&mut self, memory: &Memory) {
        // Remove from agent index
        if let Some(agent_memories) = self.agent_index.get_mut(&memory.agent_id()) {
            agent_memories.retain(|id| id != &memory.id);
            if agent_memories.is_empty() {
                self.agent_index.remove(&memory.agent_id());
            }
        }

        // Remove from type index
        if let Some(type_memories) = self.type_index.get_mut(&memory.memory_type()) {
            type_memories.retain(|id| id != &memory.id);
            if type_memories.is_empty() {
                self.type_index.remove(&memory.memory_type());
            }
        }
    }

    /// Perform text-based search
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

    /// Perform vector-based semantic search
    fn search_by_vector(
        &self,
        memories: &[&Memory],
        query_vector: &Vector,
    ) -> Vec<MemorySearchResult> {
        let mut results = Vec::new();

        for memory in memories {
            // Try to get embedding from attributes
            if let Some(embedding_attr) = memory.attributes.get(&crate::types::AttributeKey::system("embedding")) {
                if let Some(embedding_values) = embedding_attr.as_array() {
                    // Convert AttributeValue array to Vec<f32>
                    let embedding: Vec<f32> = embedding_values.iter()
                        .filter_map(|v| v.as_number().map(|n| n as f32))
                        .collect();
                    
                    if !embedding.is_empty() {
                // Calculate cosine similarity
                        let similarity = self.cosine_similarity(&query_vector.values, &embedding);

                if similarity > 0.1 {
                    // Minimum similarity threshold
                    results.push(MemorySearchResult {
                        memory: (*memory).clone(),
                        score: similarity,
                        match_type: MatchType::Semantic,
                    });
                        }
                    }
                }
            }
        }

        // Sort by similarity descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Filter memories based on query criteria
    fn filter_memories(&self, query: &MemoryQuery) -> Vec<&Memory> {
        let current_time = chrono::Utc::now().timestamp();

        self.memories
            .values()
            .filter(|memory| {
                // Agent ID filter
                if memory.agent_id() != query.agent_id {
                    return false;
                }

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
                    let age = current_time - memory.created_at();
                    if age > max_age {
                        return false;
                    }
                }

                // Skip expired memories (check if memory has expiry attribute)
                if let Some(expiry_attr) = memory.attributes.get(&crate::types::AttributeKey::system("expires_at")) {
                    if let Some(expiry_ts) = expiry_attr.as_number() {
                        if current_time > expiry_ts as i64 {
                    return false;
                        }
                    }
                }

                true
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl MemoryOperations for InMemoryOperations {
    async fn create_memory(&mut self, memory: Memory) -> Result<String> {
        let memory_id = memory.id.clone();

        if self.memories.contains_key(&memory_id) {
            return Err(AgentMemError::memory_error("Memory already exists"));
        }

        self.update_indices(&memory);
        self.memories.insert(memory_id.clone(), memory);

        Ok(memory_id)
    }

    async fn get_memory(&self, memory_id: &str) -> Result<Option<Memory>> {
        Ok(self.memories.get(memory_id).cloned())
    }

    async fn update_memory(&mut self, memory: Memory) -> Result<()> {
        let memory_id = memory.id.clone();

        if let Some(existing) = self.memories.get(&memory_id) {
            // Check if indices need updating
            let needs_reindex =
                existing.agent_id() != memory.agent_id() || existing.memory_type() != memory.memory_type();

            if needs_reindex {
                // Clone the existing memory to avoid borrow issues
                let existing_clone = existing.clone();
                self.remove_from_indices(&existing_clone);
                self.update_indices(&memory);
            }
        } else {
            return Err(AgentMemError::memory_error("Memory not found"));
        }

        self.memories.insert(memory_id, memory);
        Ok(())
    }

    async fn delete_memory(&mut self, memory_id: &str) -> Result<bool> {
        if let Some(memory) = self.memories.remove(memory_id) {
            self.remove_from_indices(&memory);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn search_memories(&self, query: MemoryQuery) -> Result<Vec<MemorySearchResult>> {
        let filtered_memories = self.filter_memories(&query);

        let mut results = if let Some(ref text_query) = query.text_query {
            self.search_by_text(&filtered_memories, text_query)
        } else if let Some(ref vector_query) = query.vector_query {
            self.search_by_vector(&filtered_memories, vector_query)
        } else {
            // Return all filtered memories with default score
            filtered_memories
                .into_iter()
                .map(|memory| MemorySearchResult {
                    memory: memory.clone(),
                    score: memory.importance(),
                    match_type: MatchType::Metadata,
                })
                .collect()
        };

        // Apply limit
        results.truncate(query.limit);
        Ok(results)
    }

    async fn get_agent_memories(
        &self,
        agent_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>> {
        let memory_ids = self.agent_index.get(agent_id).cloned().unwrap_or_default();
        let mut memories: Vec<Memory> = memory_ids
            .iter()
            .filter_map(|id| self.memories.get(id))
            .cloned()
            .collect();

        // Sort by creation time (newest first)
        memories.sort_by(|a, b| b.created_at().cmp(&a.created_at()));

        if let Some(limit) = limit {
            memories.truncate(limit);
        }

        Ok(memories)
    }

    async fn get_memories_by_type(
        &self,
        agent_id: &str,
        memory_type: MemoryType,
    ) -> Result<Vec<Memory>> {
        let memory_ids = self
            .type_index
            .get(&memory_type)
            .cloned()
            .unwrap_or_default();
        let memories: Vec<Memory> = memory_ids
            .iter()
            .filter_map(|id| self.memories.get(id))
            .filter(|memory| memory.agent_id() == agent_id)
            .cloned()
            .collect();

        Ok(memories)
    }

    async fn get_memory_stats(&self, agent_id: Option<&str>) -> Result<MemoryStats> {
        let memories: Vec<&Memory> = if let Some(agent_id) = agent_id {
            self.memories
                .values()
                .filter(|memory| memory.agent_id() == agent_id)
                .collect()
        } else {
            self.memories.values().collect()
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
            // Type distribution - from attributes
            if let Some(memory_type_str) = memory.attributes.get(&crate::types::AttributeKey::system("memory_type"))
                .and_then(|v| v.as_string())
            {
                if let Ok(memory_type) = memory_type_str.parse::<crate::types::MemoryType>() {
                    *stats
                        .memories_by_type
                        .entry(memory_type)
                        .or_insert(0) += 1;
                }
            }

            // Agent distribution - from attributes
            if let Some(agent_id) = memory.attributes.get(&crate::types::AttributeKey::system("agent_id"))
                .and_then(|v| v.as_string())
            {
                *stats
                    .memories_by_agent
                    .entry(agent_id.to_string())
                    .or_insert(0) += 1;
            }

            // Importance and access stats - from attributes
            let importance = memory.attributes.get(&crate::types::AttributeKey::system("importance"))
                .and_then(|v| v.as_number())
                .unwrap_or(0.0) as f32;
            total_importance += importance;
            
            let access_count = memory.metadata.accessed_count;
            total_access_count += access_count as u64;

            if access_count > most_accessed_count {
                most_accessed_count = access_count;
                stats.most_accessed_memory_id = Some(memory.id.clone());
            }

            // created_at from metadata
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
        let mut created_ids = Vec::new();

        for memory in memories {
            let memory_id = memory.id.clone();

            if self.memories.contains_key(&memory_id) {
                return Err(AgentMemError::memory_error(format!(
                    "Memory {memory_id} already exists"
                )));
            }

            self.update_indices(&memory);
            self.memories.insert(memory_id.clone(), memory);
            created_ids.push(memory_id);
        }

        Ok(created_ids)
    }

    async fn batch_delete_memories(&mut self, memory_ids: Vec<String>) -> Result<usize> {
        let mut deleted_count = 0;

        for memory_id in memory_ids {
            if let Some(memory) = self.memories.remove(&memory_id) {
                self.remove_from_indices(&memory);
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }
}

impl Default for InMemoryOperations {
    fn default() -> Self {
        Self::new()
    }
}
