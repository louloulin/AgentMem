//! Context-Aware Memory Search System
//!
//! Advanced search capabilities that understand and adapt to current context,
//! providing intelligent memory retrieval and recommendations.

use crate::adaptive_strategy::MemoryStrategy;
use crate::hierarchical_service::{HierarchicalMemoryRecord, HierarchicalSearchFilters};
use crate::hierarchy::{MemoryLevel, MemoryScope};
use crate::importance_scorer::{ImportanceFactors, ScoringContext};
use crate::types::{ImportanceLevel, MemoryType};
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

/// Configuration for context-aware search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareSearchConfig {
    /// Enable semantic similarity search
    pub enable_semantic_search: bool,
    /// Semantic similarity threshold (0.0-1.0)
    pub semantic_threshold: f64,
    /// Enable temporal context weighting
    pub enable_temporal_weighting: bool,
    /// Temporal decay factor
    pub temporal_decay_factor: f64,
    /// Enable user preference learning
    pub enable_preference_learning: bool,
    /// Maximum search results to return
    pub max_results: usize,
    /// Enable search result caching
    pub enable_result_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable query expansion
    pub enable_query_expansion: bool,
    /// Enable contextual ranking
    pub enable_contextual_ranking: bool,
}

impl Default for ContextAwareSearchConfig {
    fn default() -> Self {
        Self {
            enable_semantic_search: true,
            semantic_threshold: 0.3,
            enable_temporal_weighting: true,
            temporal_decay_factor: 0.1,
            enable_preference_learning: true,
            max_results: 50,
            enable_result_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            enable_query_expansion: true,
            enable_contextual_ranking: true,
        }
    }
}

/// Search query with context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualSearchQuery {
    pub query_text: String,
    pub context: ScoringContext,
    pub filters: Option<HierarchicalSearchFilters>,
    pub search_strategy: SearchStrategy,
    pub result_preferences: ResultPreferences,
}

/// Search strategy options
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum SearchStrategy {
    /// Exact text matching
    Exact,
    /// Fuzzy text matching
    Fuzzy,
    /// Semantic similarity matching
    Semantic,
    /// Hybrid approach combining multiple strategies
    Hybrid,
    /// Context-driven adaptive search
    Adaptive,
}

/// Result preferences for search customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultPreferences {
    /// Prefer recent memories
    pub prefer_recent: bool,
    /// Prefer frequently accessed memories
    pub prefer_frequent: bool,
    /// Prefer high-importance memories
    pub prefer_important: bool,
    /// Prefer memories from specific scopes
    pub preferred_scopes: Vec<MemoryScope>,
    /// Prefer memories from specific levels
    pub preferred_levels: Vec<MemoryLevel>,
    /// Custom ranking weights
    pub custom_weights: HashMap<String, f64>,
}

impl Default for ResultPreferences {
    fn default() -> Self {
        Self {
            prefer_recent: true,
            prefer_frequent: true,
            prefer_important: true,
            preferred_scopes: Vec::new(),
            preferred_levels: Vec::new(),
            custom_weights: HashMap::new(),
        }
    }
}

/// Search result with context-aware scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualSearchResult {
    pub memory: HierarchicalMemoryRecord,
    pub relevance_score: f64,
    pub context_score: f64,
    pub importance_factors: Option<ImportanceFactors>,
    pub match_reasons: Vec<String>,
    pub snippet: String,
    pub rank: usize,
}

/// Search analytics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnalytics {
    pub query_count: u64,
    pub average_results_per_query: f64,
    pub average_response_time_ms: u64,
    pub cache_hit_rate: f64,
    pub user_satisfaction_score: f64,
    pub most_common_queries: Vec<(String, u64)>,
    pub search_patterns: HashMap<String, u64>,
}

/// Context-aware search engine
pub struct ContextAwareSearchEngine {
    config: ContextAwareSearchConfig,
    search_cache: HashMap<String, (Vec<ContextualSearchResult>, DateTime<Utc>)>,
    query_history: Vec<(String, DateTime<Utc>, usize)>, // query, timestamp, result_count
    user_preferences: HashMap<String, HashMap<String, f64>>,
    search_analytics: SearchAnalytics,
    expanded_queries: HashMap<String, Vec<String>>,
}

impl ContextAwareSearchEngine {
    /// Create a new context-aware search engine
    pub fn new(config: ContextAwareSearchConfig) -> Self {
        Self {
            config,
            search_cache: HashMap::new(),
            query_history: Vec::new(),
            user_preferences: HashMap::new(),
            search_analytics: SearchAnalytics {
                query_count: 0,
                average_results_per_query: 0.0,
                average_response_time_ms: 0,
                cache_hit_rate: 0.0,
                user_satisfaction_score: 0.0,
                most_common_queries: Vec::new(),
                search_patterns: HashMap::new(),
            },
            expanded_queries: HashMap::new(),
        }
    }

    /// Perform context-aware search
    pub async fn search(
        &mut self,
        query: ContextualSearchQuery,
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        let start_time = std::time::Instant::now();

        // Check cache first
        if self.config.enable_result_caching {
            let cache_key = self.generate_cache_key(&query);
            if let Some((cached_results, cached_at)) = self.search_cache.get(&cache_key) {
                let cache_age = Utc::now().signed_duration_since(*cached_at);
                if cache_age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                    self.search_analytics.cache_hit_rate += 1.0;
                    return Ok(cached_results.clone());
                }
            }
        }

        // Expand query if enabled
        let expanded_query = if self.config.enable_query_expansion {
            self.expand_query(&query.query_text).await?
        } else {
            vec![query.query_text.clone()]
        };

        // Perform search based on strategy
        let mut results = match query.search_strategy {
            SearchStrategy::Exact => self.exact_search(&expanded_query, memories).await?,
            SearchStrategy::Fuzzy => self.fuzzy_search(&expanded_query, memories).await?,
            SearchStrategy::Semantic => self.semantic_search(&expanded_query, memories).await?,
            SearchStrategy::Hybrid => self.hybrid_search(&expanded_query, memories).await?,
            SearchStrategy::Adaptive => self.adaptive_search(&query, memories).await?,
        };

        // Apply context-aware filtering
        results = self
            .apply_context_filtering(results, &query.context)
            .await?;

        // Apply hierarchical filters if provided
        if let Some(ref filters) = query.filters {
            results = self.apply_hierarchical_filters(results, filters).await?;
        }

        // Rank results based on context and preferences
        if self.config.enable_contextual_ranking {
            results = self.rank_results_contextually(results, &query).await?;
        }

        // Limit results
        results.truncate(self.config.max_results);

        // Add ranking information
        for (i, result) in results.iter_mut().enumerate() {
            result.rank = i + 1;
        }

        // Cache results
        if self.config.enable_result_caching {
            let cache_key = self.generate_cache_key(&query);
            self.search_cache
                .insert(cache_key, (results.clone(), Utc::now()));
        }

        // Update analytics
        let response_time = start_time.elapsed().as_millis() as u64;
        self.update_search_analytics(&query.query_text, results.len(), response_time);

        // Learn from user preferences if enabled
        if self.config.enable_preference_learning {
            self.update_user_preferences(&query, &results).await?;
        }

        Ok(results)
    }

    /// Exact text search
    async fn exact_search(
        &self,
        queries: &[String],
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        let mut results = Vec::new();

        for memory in memories {
            for query in queries {
                if memory
                    .content
                    .to_lowercase()
                    .contains(&query.to_lowercase())
                {
                    let snippet = self.extract_snippet(&memory.content, query);
                    results.push(ContextualSearchResult {
                        memory: memory.clone(),
                        relevance_score: 1.0, // Exact match
                        context_score: 0.0,   // Will be calculated later
                        importance_factors: None,
                        match_reasons: vec!["Exact text match".to_string()],
                        snippet,
                        rank: 0,
                    });
                    break; // Only add once per memory
                }
            }
        }

        Ok(results)
    }

    /// Fuzzy text search
    async fn fuzzy_search(
        &self,
        queries: &[String],
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        let mut results = Vec::new();

        for memory in memories {
            let mut best_score = 0.0;
            let mut best_query = String::new();

            for query in queries {
                let score = self.calculate_fuzzy_similarity(&memory.content, query);
                if score > best_score {
                    best_score = score;
                    best_query = query.clone();
                }
            }

            if best_score > 0.3 {
                // Minimum fuzzy threshold
                let snippet = self.extract_snippet(&memory.content, &best_query);
                results.push(ContextualSearchResult {
                    memory: memory.clone(),
                    relevance_score: best_score,
                    context_score: 0.0,
                    importance_factors: None,
                    match_reasons: vec![format!("Fuzzy match (score: {:.2})", best_score)],
                    snippet,
                    rank: 0,
                });
            }
        }

        Ok(results)
    }

    /// Semantic similarity search
    async fn semantic_search(
        &self,
        queries: &[String],
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        let mut results = Vec::new();

        for memory in memories {
            let mut best_score = 0.0;
            let mut best_query = String::new();

            for query in queries {
                let score = self
                    .calculate_semantic_similarity(&memory.content, query)
                    .await?;
                if score > best_score {
                    best_score = score;
                    best_query = query.clone();
                }
            }

            if best_score > self.config.semantic_threshold {
                let snippet = self.extract_snippet(&memory.content, &best_query);
                results.push(ContextualSearchResult {
                    memory: memory.clone(),
                    relevance_score: best_score,
                    context_score: 0.0,
                    importance_factors: None,
                    match_reasons: vec![format!("Semantic similarity (score: {:.2})", best_score)],
                    snippet,
                    rank: 0,
                });
            }
        }

        Ok(results)
    }

    /// Hybrid search combining multiple strategies
    async fn hybrid_search(
        &self,
        queries: &[String],
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        // Combine results from different search strategies
        let exact_results = self.exact_search(queries, memories).await?;
        let fuzzy_results = self.fuzzy_search(queries, memories).await?;
        let semantic_results = self.semantic_search(queries, memories).await?;

        // Merge and deduplicate results
        let mut combined_results = HashMap::new();

        // Add exact results with highest weight
        for result in exact_results {
            combined_results.insert(result.memory.id.clone(), result);
        }

        // Add fuzzy results, combining scores if memory already exists
        for result in fuzzy_results {
            if let Some(existing) = combined_results.get_mut(&result.memory.id) {
                existing.relevance_score =
                    (existing.relevance_score + result.relevance_score * 0.7) / 2.0;
                existing.match_reasons.extend(result.match_reasons);
            } else {
                combined_results.insert(result.memory.id.clone(), result);
            }
        }

        // Add semantic results
        for result in semantic_results {
            if let Some(existing) = combined_results.get_mut(&result.memory.id) {
                existing.relevance_score =
                    (existing.relevance_score + result.relevance_score * 0.8) / 2.0;
                existing.match_reasons.extend(result.match_reasons);
            } else {
                combined_results.insert(result.memory.id.clone(), result);
            }
        }

        Ok(combined_results.into_values().collect())
    }

    /// Adaptive search that adjusts strategy based on context
    async fn adaptive_search(
        &self,
        query: &ContextualSearchQuery,
        memories: &[HierarchicalMemoryRecord],
    ) -> Result<Vec<ContextualSearchResult>> {
        // Choose strategy based on query characteristics and context
        let strategy = self.determine_optimal_strategy(query);

        match strategy {
            SearchStrategy::Exact => {
                self.exact_search(&[query.query_text.clone()], memories)
                    .await
            }
            SearchStrategy::Fuzzy => {
                self.fuzzy_search(&[query.query_text.clone()], memories)
                    .await
            }
            SearchStrategy::Semantic => {
                self.semantic_search(&[query.query_text.clone()], memories)
                    .await
            }
            SearchStrategy::Hybrid => {
                self.hybrid_search(&[query.query_text.clone()], memories)
                    .await
            }
            SearchStrategy::Adaptive => {
                // Fallback to hybrid if we end up here recursively
                self.hybrid_search(&[query.query_text.clone()], memories)
                    .await
            }
        }
    }

    /// Determine optimal search strategy based on query and context
    fn determine_optimal_strategy(&self, query: &ContextualSearchQuery) -> SearchStrategy {
        // Simple heuristics for strategy selection
        let query_len = query.query_text.len();
        let word_count = query.query_text.split_whitespace().count();

        if query_len < 10 && word_count <= 2 {
            SearchStrategy::Exact
        } else if query_len < 50 && word_count <= 5 {
            SearchStrategy::Fuzzy
        } else if word_count > 5 {
            SearchStrategy::Semantic
        } else {
            SearchStrategy::Hybrid
        }
    }

    /// Apply context-aware filtering to results
    async fn apply_context_filtering(
        &self,
        mut results: Vec<ContextualSearchResult>,
        context: &ScoringContext,
    ) -> Result<Vec<ContextualSearchResult>> {
        for result in &mut results {
            // Calculate context score based on various factors
            let mut context_score = 0.0;

            // Time-based context
            if self.config.enable_temporal_weighting {
                let time_since_access = context
                    .current_time
                    .signed_duration_since(result.memory.accessed_at)
                    .num_hours() as f64;
                let temporal_score =
                    (-self.config.temporal_decay_factor * time_since_access / 24.0).exp();
                context_score += temporal_score * 0.3;
            }

            // User context
            if let Some(ref user_id) = context.user_id {
                if result.memory.metadata.get("user_id") == Some(user_id) {
                    context_score += 0.4;
                }
            }

            // Session context
            if let Some(ref session_id) = context.session_id {
                if result.memory.metadata.get("session_id") == Some(session_id) {
                    context_score += 0.2;
                }
            }

            // Task context
            if let Some(ref task) = context.current_task {
                if result
                    .memory
                    .content
                    .to_lowercase()
                    .contains(&task.to_lowercase())
                {
                    context_score += 0.1;
                }
            }

            result.context_score = context_score.min(1.0);
        }

        Ok(results)
    }

    /// Apply hierarchical filters to results
    async fn apply_hierarchical_filters(
        &self,
        results: Vec<ContextualSearchResult>,
        filters: &HierarchicalSearchFilters,
    ) -> Result<Vec<ContextualSearchResult>> {
        let filtered_results = results
            .into_iter()
            .filter(|result| {
                // Apply scope filters
                if let Some(ref scopes) = filters.scopes {
                    if !scopes.contains(&result.memory.scope) {
                        return false;
                    }
                }

                // Apply level filters
                if let Some(ref levels) = filters.levels {
                    if !levels.contains(&result.memory.level) {
                        return false;
                    }
                }

                // Apply importance filter
                if let Some(min_importance) = filters.importance_min {
                    if result.memory.importance < min_importance {
                        return false;
                    }
                }

                // Apply quality filter
                if let Some(min_quality) = filters.quality_min {
                    if result.memory.quality_score < min_quality {
                        return false;
                    }
                }

                // Apply date filters
                if let Some(created_after) = filters.created_after {
                    if result.memory.created_at < created_after {
                        return false;
                    }
                }

                if let Some(created_before) = filters.created_before {
                    if result.memory.created_at > created_before {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered_results)
    }

    /// Rank results based on context and user preferences
    async fn rank_results_contextually(
        &self,
        mut results: Vec<ContextualSearchResult>,
        query: &ContextualSearchQuery,
    ) -> Result<Vec<ContextualSearchResult>> {
        // Calculate composite scores for ranking
        for result in &mut results {
            let mut composite_score = result.relevance_score * 0.4 + result.context_score * 0.3;

            // Apply preference weights
            if query.result_preferences.prefer_recent {
                let recency_score = self.calculate_recency_score(&result.memory);
                composite_score += recency_score * 0.1;
            }

            if query.result_preferences.prefer_frequent {
                let frequency_score = (result.memory.access_count as f64).ln() / 10.0;
                composite_score += frequency_score.min(0.1);
            }

            if query.result_preferences.prefer_important {
                let importance_score = match result.memory.importance {
                    ImportanceLevel::Critical => 1.0,
                    ImportanceLevel::High => 0.8,
                    ImportanceLevel::Medium => 0.6,
                    ImportanceLevel::Low => 0.4,
                };
                composite_score += importance_score * 0.1;
            }

            // Apply custom weights
            for (key, weight) in &query.result_preferences.custom_weights {
                if let Some(value) = result.memory.metadata.get(key) {
                    if let Ok(numeric_value) = value.parse::<f64>() {
                        composite_score += numeric_value * weight;
                    }
                }
            }

            result.relevance_score = composite_score;
        }

        // Sort by composite score
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Calculate recency score for a memory
    fn calculate_recency_score(&self, memory: &HierarchicalMemoryRecord) -> f64 {
        let hours_since_access = Utc::now()
            .signed_duration_since(memory.accessed_at)
            .num_hours() as f64;

        (-0.01 * hours_since_access).exp()
    }

    /// Calculate fuzzy similarity between two texts
    fn calculate_fuzzy_similarity(&self, text1: &str, text2: &str) -> f64 {
        // Simple word overlap similarity
        let words1: HashSet<&str> = text1.split_whitespace().collect();
        let words2: HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Calculate semantic similarity (simplified implementation)
    async fn calculate_semantic_similarity(&self, text1: &str, text2: &str) -> Result<f64> {
        // This is a simplified implementation
        // In production, would use embedding-based similarity
        Ok(self.calculate_fuzzy_similarity(text1, text2))
    }

    /// Expand query with related terms
    async fn expand_query(&mut self, query: &str) -> Result<Vec<String>> {
        let mut expanded = vec![query.to_string()];

        // Check cache first
        if let Some(cached_expansions) = self.expanded_queries.get(query) {
            expanded.extend(cached_expansions.clone());
            return Ok(expanded);
        }

        // Simple query expansion (in production, would use more sophisticated methods)
        let words: Vec<&str> = query.split_whitespace().collect();

        // Add synonyms and related terms (simplified)
        let synonyms = HashMap::from([
            ("important", vec!["critical", "significant", "vital"]),
            ("task", vec!["job", "work", "assignment"]),
            ("memory", vec!["information", "data", "knowledge"]),
        ]);

        for word in &words {
            if let Some(word_synonyms) = synonyms.get(word) {
                for synonym in word_synonyms {
                    let expanded_query = query.replace(word, synonym);
                    expanded.push(expanded_query);
                }
            }
        }

        // Cache the expansions
        self.expanded_queries
            .insert(query.to_string(), expanded[1..].to_vec());

        Ok(expanded)
    }

    /// Extract relevant snippet from content
    fn extract_snippet(&self, content: &str, query: &str) -> String {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_words: Vec<&str> = content.split_whitespace().collect();

        // Find the best matching position
        let mut best_start = 0;
        let mut best_matches = 0;

        for i in 0..content_words.len() {
            let mut matches = 0;
            for j in 0..query_words.len().min(content_words.len() - i) {
                if content_words[i + j]
                    .to_lowercase()
                    .contains(&query_words[j].to_lowercase())
                {
                    matches += 1;
                }
            }
            if matches > best_matches {
                best_matches = matches;
                best_start = i;
            }
        }

        // Extract snippet around best match
        let snippet_start = best_start.saturating_sub(5);
        let snippet_end = (best_start + 15).min(content_words.len());

        let snippet_words = &content_words[snippet_start..snippet_end];
        let mut snippet = snippet_words.join(" ");

        if snippet_start > 0 {
            snippet = format!("...{}", snippet);
        }
        if snippet_end < content_words.len() {
            snippet = format!("{}...", snippet);
        }

        snippet
    }

    /// Generate cache key for query
    fn generate_cache_key(&self, query: &ContextualSearchQuery) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.query_text.hash(&mut hasher);
        query.search_strategy.hash(&mut hasher);
        format!("search_cache_{}", hasher.finish())
    }

    /// Update search analytics
    fn update_search_analytics(&mut self, query: &str, result_count: usize, response_time_ms: u64) {
        self.search_analytics.query_count += 1;

        // Update average results per query
        let old_avg = self.search_analytics.average_results_per_query;
        let count = self.search_analytics.query_count as f64;
        self.search_analytics.average_results_per_query =
            (old_avg * (count - 1.0) + result_count as f64) / count;

        // Update average response time
        let old_time = self.search_analytics.average_response_time_ms;
        self.search_analytics.average_response_time_ms =
            ((old_time as f64 * (count - 1.0) + response_time_ms as f64) / count) as u64;

        // Track query patterns
        *self
            .search_analytics
            .search_patterns
            .entry(query.to_string())
            .or_insert(0) += 1;

        // Update query history
        self.query_history
            .push((query.to_string(), Utc::now(), result_count));

        // Limit history size
        if self.query_history.len() > 1000 {
            self.query_history.remove(0);
        }
    }

    /// Update user preferences based on search results
    async fn update_user_preferences(
        &mut self,
        query: &ContextualSearchQuery,
        results: &[ContextualSearchResult],
    ) -> Result<()> {
        if let Some(ref user_id) = query.context.user_id {
            let user_prefs = self
                .user_preferences
                .entry(user_id.clone())
                .or_insert_with(HashMap::new);

            // Learn from result characteristics
            for result in results.iter().take(5) {
                // Top 5 results
                // Update scope preferences
                let scope_key = format!("scope_{:?}", result.memory.scope);
                *user_prefs.entry(scope_key).or_insert(0.0) += 0.1;

                // Update level preferences
                let level_key = format!("level_{:?}", result.memory.level);
                *user_prefs.entry(level_key).or_insert(0.0) += 0.1;

                // Update importance preferences
                let importance_key = format!("importance_{:?}", result.memory.importance);
                *user_prefs.entry(importance_key).or_insert(0.0) += 0.1;
            }
        }

        Ok(())
    }

    /// Get search analytics
    pub fn get_analytics(&self) -> &SearchAnalytics {
        &self.search_analytics
    }

    /// Clear search cache
    pub fn clear_cache(&mut self) {
        self.search_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory(content: &str) -> HierarchicalMemoryRecord {
        HierarchicalMemoryRecord {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            scope: MemoryScope::Global,
            level: MemoryLevel::Operational,
            importance: ImportanceLevel::Medium,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            accessed_at: Utc::now(),
            access_count: 0,
            metadata: HashMap::new(),
            tags: Vec::new(),
            parent_memory_id: None,
            child_memory_ids: Vec::new(),
            conflict_resolution_strategy:
                crate::hierarchical_service::ConflictResolutionStrategy::ImportanceBased,
            quality_score: 1.0,
            source_reliability: 1.0,
        }
    }

    #[tokio::test]
    async fn test_context_aware_search_engine_creation() {
        let config = ContextAwareSearchConfig::default();
        let engine = ContextAwareSearchEngine::new(config);

        assert_eq!(engine.search_cache.len(), 0);
        assert_eq!(engine.query_history.len(), 0);
    }

    #[tokio::test]
    async fn test_exact_search() {
        let config = ContextAwareSearchConfig::default();
        let engine = ContextAwareSearchEngine::new(config);

        let memories = vec![
            create_test_memory("This is a test memory"),
            create_test_memory("Another memory for testing"),
            create_test_memory("Unrelated content"),
        ];

        let results = engine
            .exact_search(&["test".to_string()], &memories)
            .await
            .unwrap();
        assert_eq!(results.len(), 2); // Two memories contain "test"
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let config = ContextAwareSearchConfig::default();
        let engine = ContextAwareSearchEngine::new(config);

        let memories = vec![
            create_test_memory("This is a test memory"),
            create_test_memory("Testing fuzzy search"),
            create_test_memory("Completely different content"),
        ];

        let results = engine
            .fuzzy_search(&["test memory".to_string()], &memories)
            .await
            .unwrap();
        assert!(!results.is_empty());
        assert!(results[0].relevance_score > 0.3);
    }

    #[tokio::test]
    async fn test_contextual_search() -> anyhow::Result<()> {
        let config = ContextAwareSearchConfig::default();
        let mut engine = ContextAwareSearchEngine::new(config);

        let memories = vec![
            create_test_memory("Important task information"),
            create_test_memory("Regular memory content"),
        ];

        let query = ContextualSearchQuery {
            query_text: "task".to_string(),
            context: ScoringContext::default(),
            filters: None,
            search_strategy: SearchStrategy::Exact,
            result_preferences: ResultPreferences::default(),
        Ok(())
        };

        let results = engine.search(query, &memories).await?;
        assert!(!results.is_empty());
        assert_eq!(results[0].rank, 1);
    }
}
