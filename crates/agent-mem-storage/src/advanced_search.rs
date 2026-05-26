//! Advanced Search Module for AgentMem
//!
//! Provides advanced search capabilities:
//! - Time-weighted retrieval
//! - Query expansion
//! - Semantic reranking

use agent_mem_traits::{Result, VectorSearchResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for time-weighted search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWeightedConfig {
    /// Enable time weighting
    pub enabled: bool,
    /// Decay rate per day (0.0 - 1.0)
    pub decay_rate_per_day: f32,
    /// Minimum time weight
    pub min_time_weight: f32,
    /// Maximum time weight
    pub max_time_weight: f32,
    /// Reference timestamp (None = now)
    pub reference_timestamp: Option<i64>,
}

impl Default for TimeWeightedConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            decay_rate_per_day: 0.05,
            min_time_weight: 0.1,
            max_time_weight: 1.0,
            reference_timestamp: None,
        }
    }
}

/// Configuration for query expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExpansionConfig {
    /// Enable query expansion
    pub enabled: bool,
    /// Number of expanded terms
    pub num_expansions: usize,
    /// Expansion threshold
    pub expansion_threshold: f32,
    /// Synonym dictionary (simplified)
    pub use_synonyms: bool,
}

impl Default for QueryExpansionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            num_expansions: 3,
            expansion_threshold: 0.8,
            use_synonyms: true,
        }
    }
}

/// Configuration for reranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankConfig {
    /// Enable reranking
    pub enabled: bool,
    /// Number of results to return
    pub top_n: usize,
    /// Rerank weight for vector score
    pub vector_weight: f32,
    /// Rerank weight for time score
    pub time_weight: f32,
    /// Rerank weight for importance score
    pub importance_weight: f32,
}

impl Default for RerankConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            top_n: 10,
            vector_weight: 0.5,
            time_weight: 0.3,
            importance_weight: 0.2,
        }
    }
}

/// Search result with metadata
#[derive(Debug, Clone)]
pub struct EnhancedSearchResult {
    pub id: String,
    pub content: String,
    pub vector_score: f32,
    pub time_weight: f32,
    pub importance_score: f32,
    pub combined_score: f32,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

/// Time-weighted search calculator
pub struct TimeWeightedSearch {
    config: TimeWeightedConfig,
}

impl TimeWeightedSearch {
    pub fn new(config: TimeWeightedConfig) -> Self {
        Self { config }
    }

    /// Calculate time weight for a result
    pub fn calculate_weight(&self, timestamp: i64) -> f32 {
        if !self.config.enabled {
            return 1.0;
        }

        let now = self.config.reference_timestamp.unwrap_or_else(|| {
            chrono::Utc::now().timestamp()
        });

        let age_days = (now - timestamp) as f32 / 86400.0;
        let decay = (-self.config.decay_rate_per_day * age_days).exp();
        
        decay.clamp(self.config.min_time_weight, self.config.max_time_weight)
    }

    /// Apply time weighting to search results
    pub fn apply_weights(
        &self,
        results: Vec<VectorSearchResult>,
        timestamps: &HashMap<String, i64>,
    ) -> Vec<EnhancedSearchResult> {
        results
            .into_iter()
            .map(|r| {
                let timestamp = timestamps.get(&r.id).copied().unwrap_or(0);
                let time_weight = self.calculate_weight(timestamp);
                
                EnhancedSearchResult {
                    id: r.id,
                    content: String::new(),
                    vector_score: r.similarity,
                    time_weight,
                    importance_score: 0.5,
                    combined_score: r.similarity * time_weight,
                    timestamp,
                    metadata: r.metadata,
                }
            })
            .collect()
    }
}

/// Query expansion module
pub struct QueryExpansion {
    config: QueryExpansionConfig,
    synonyms: HashMap<String, Vec<String>>,
}

impl QueryExpansion {
    pub fn new(config: QueryExpansionConfig) -> Self {
        let synonyms = Self::build_default_synonyms();
        Self { config, synonyms }
    }

    /// Build default synonym dictionary
    fn build_default_synonyms() -> HashMap<String, Vec<String>> {
        let mut synonyms = HashMap::new();
        
        // Common programming synonyms
        synonyms.insert("bug".to_string(), vec!["error".to_string(), "issue".to_string()]);
        synonyms.insert("function".to_string(), vec!["method".to_string(), "procedure".to_string()]);
        synonyms.insert("variable".to_string(), vec!["var".to_string(), "parameter".to_string()]);
        synonyms.insert("class".to_string(), vec!["type".to_string(), "struct".to_string()]);
        synonyms.insert("test".to_string(), vec!["verify".to_string(), "check".to_string()]);
        
        synonyms
    }

    /// Expand a query with synonyms
    pub fn expand(&self, query: &str) -> Vec<String> {
        if !self.config.enabled {
            return vec![query.to_string()];
        }

        let mut expanded = vec![query.to_string()];
        let words: Vec<&str> = query.split_whitespace().collect();
        
        for word in words {
            let word_lower = word.to_lowercase();
            if let Some(syns) = self.synonyms.get(&word_lower) {
                for syn in syns.iter().take(self.config.num_expansions) {
                    let expanded_query = query.replace(word, syn);
                    if !expanded.contains(&expanded_query) {
                        expanded.push(expanded_query);
                    }
                }
            }
        }
        
        expanded
    }

    /// Generate expanded queries with variations
    pub fn generate_variations(&self, query: &str) -> Vec<String> {
        let mut variations = self.expand(query);
        
        // Add common variations
        if query.contains("how") {
            variations.push(query.replace("how", "what"));
        }
        if query.contains("why") {
            variations.push(query.replace("why", "reason"));
        }
        
        variations.truncate(self.config.num_expansions * 2);
        variations
    }
}

/// Semantic reranker
pub struct SemanticReranker {
    config: RerankConfig,
}

impl SemanticReranker {
    pub fn new(config: RerankConfig) -> Self {
        Self { config }
    }

    /// Rerank results based on combined signals
    pub fn rerank(&self, results: Vec<EnhancedSearchResult>) -> Vec<EnhancedSearchResult> {
        if !self.config.enabled {
            return results;
        }

        let mut scored: Vec<_> = results
            .into_iter()
            .map(|mut r| {
                r.combined_score = 
                    r.vector_score * self.config.vector_weight +
                    r.time_weight * self.config.time_weight +
                    r.importance_score * self.config.importance_weight;
                r
            })
            .collect();
        
        // Sort by combined score descending
        scored.sort_by(|a, b| {
            b.combined_score.partial_cmp(&a.combined_score).unwrap()
        });
        
        // Return top N
        scored.into_iter().take(self.config.top_n).collect()
    }

    /// Update importance scores for reranking
    pub fn with_importance(
        &self,
        results: Vec<EnhancedSearchResult>,
        importance_scores: &HashMap<String, f32>,
    ) -> Vec<EnhancedSearchResult> {
        let updated: Vec<_> = results
            .into_iter()
            .map(|mut r| {
                if let Some(&score) = importance_scores.get(&r.id) {
                    r.importance_score = score;
                }
                r
            })
            .collect();
        
        self.rerank(updated)
    }
}

/// Search analytics
#[derive(Debug, Clone, Default)]
pub struct SearchAnalytics {
    pub total_queries: u64,
    pub avg_results: f32,
    pub avg_time_weight: f32,
    pub reranked_queries: u64,
}

impl SearchAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_query(&mut self, num_results: usize, avg_time_weight: f32, reranked: bool) {
        self.total_queries += 1;
        self.avg_results = (self.avg_results * (self.total_queries - 1) as f32 + num_results as f32) 
            / self.total_queries as f32;
        self.avg_time_weight = (self.avg_time_weight * (self.total_queries - 1) as f32 + avg_time_weight) 
            / self.total_queries as f32;
        if reranked {
            self.reranked_queries += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_weighted_search() {
        let config = TimeWeightedConfig::default();
        let search = TimeWeightedSearch::new(config);
        
        let now = chrono::Utc::now().timestamp();
        let weight = search.calculate_weight(now - 86400); // 1 day ago
        
        assert!(weight > 0.0);
        assert!(weight <= 1.0);
    }

    #[test]
    fn test_query_expansion() {
        let config = QueryExpansionConfig::default();
        let expansion = QueryExpansion::new(config);
        
        let expanded = expansion.expand("fix bug");
        
        assert!(expanded.len() >= 1);
        assert!(expanded.contains(&"fix bug".to_string()));
    }

    #[test]
    fn test_reranking() {
        let config = RerankConfig::default();
        let reranker = SemanticReranker::new(config);
        
        let results = vec![
            EnhancedSearchResult {
                id: "1".to_string(),
                content: "Test".to_string(),
                vector_score: 0.9,
                time_weight: 0.5,
                importance_score: 0.8,
                combined_score: 0.0,
                timestamp: 0,
                metadata: HashMap::new(),
            },
            EnhancedSearchResult {
                id: "2".to_string(),
                content: "Test".to_string(),
                vector_score: 0.7,
                time_weight: 0.9,
                importance_score: 0.6,
                combined_score: 0.0,
                timestamp: 0,
                metadata: HashMap::new(),
            },
        ];
        
        let reranked = reranker.rerank(results);
        
        assert_eq!(reranked.len(), 2);
        // Higher combined score should be first
        assert!(reranked[0].combined_score >= reranked[1].combined_score);
    }
}
