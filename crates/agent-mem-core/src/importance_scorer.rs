//! Advanced Importance Scoring System
//!
//! Multi-dimensional importance scoring with dynamic weight adjustment
//! and context-aware evaluation for memory prioritization.

use crate::hierarchical_service::HierarchicalMemoryRecord;
use crate::types::ImportanceLevel;
use agent_mem_traits::Result;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for importance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceScorerConfig {
    /// Weight for recency factor (0.0-1.0)
    pub recency_weight: f64,
    /// Weight for frequency factor (0.0-1.0)
    pub frequency_weight: f64,
    /// Weight for relevance factor (0.0-1.0)
    pub relevance_weight: f64,
    /// Weight for emotional factor (0.0-1.0)
    pub emotional_weight: f64,
    /// Weight for context factor (0.0-1.0)
    pub context_weight: f64,
    /// Weight for user interaction factor (0.0-1.0)
    pub interaction_weight: f64,
    /// Enable dynamic weight adjustment
    pub enable_dynamic_weights: bool,
    /// Learning rate for weight adjustment
    pub learning_rate: f64,
    /// Minimum score threshold
    pub min_score_threshold: f64,
    /// Maximum score cap
    pub max_score_cap: f64,
}

impl Default for ImportanceScorerConfig {
    fn default() -> Self {
        Self {
            recency_weight: 0.25,
            frequency_weight: 0.20,
            relevance_weight: 0.25,
            emotional_weight: 0.15,
            context_weight: 0.10,
            interaction_weight: 0.05,
            enable_dynamic_weights: true,
            learning_rate: 0.01,
            min_score_threshold: 0.0,
            max_score_cap: 10.0,
        }
    }
}

/// Importance scoring factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceFactors {
    pub recency_score: f64,
    pub frequency_score: f64,
    pub relevance_score: f64,
    pub emotional_score: f64,
    pub context_score: f64,
    pub interaction_score: f64,
    pub composite_score: f64,
    pub calculated_at: DateTime<Utc>,
}

/// Memory usage statistics for scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub creation_time: DateTime<Utc>,
    pub modification_count: u64,
    pub reference_count: u64,
    pub user_interactions: u64,
    pub context_matches: u64,
}

impl Default for MemoryUsageStats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            access_count: 0,
            last_accessed: now,
            creation_time: now,
            modification_count: 0,
            reference_count: 0,
            user_interactions: 0,
            context_matches: 0,
        }
    }
}

/// Context information for scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringContext {
    pub current_time: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub current_task: Option<String>,
    pub recent_queries: Vec<String>,
    pub user_preferences: HashMap<String, f64>,
    pub domain_context: Option<String>,
}

impl Default for ScoringContext {
    fn default() -> Self {
        Self {
            current_time: Utc::now(),
            user_id: None,
            session_id: None,
            current_task: None,
            recent_queries: Vec::new(),
            user_preferences: HashMap::new(),
            domain_context: None,
        }
    }
}

/// Advanced importance scorer with multi-dimensional evaluation
pub struct AdvancedImportanceScorer {
    config: ImportanceScorerConfig,
    usage_stats: HashMap<String, MemoryUsageStats>,
    weight_history: Vec<ImportanceScorerConfig>,
    performance_metrics: HashMap<String, f64>,
}

impl AdvancedImportanceScorer {
    /// Create a new importance scorer
    pub fn new(config: ImportanceScorerConfig) -> Self {
        Self {
            config: config.clone(),
            usage_stats: HashMap::new(),
            weight_history: vec![config],
            performance_metrics: HashMap::new(),
        }
    }

    /// Calculate comprehensive importance score for a memory
    pub async fn calculate_importance(
        &mut self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
    ) -> Result<ImportanceFactors> {
        // Get or create usage stats and clone them to avoid borrow checker issues
        let stats = self.get_or_create_stats(&memory.id).clone();

        // Calculate individual factor scores
        let recency_score = self
            .calculate_recency_score(memory, context, &stats)
            .await?;
        let frequency_score = self
            .calculate_frequency_score(memory, context, &stats)
            .await?;
        let relevance_score = self
            .calculate_relevance_score(memory, context, &stats)
            .await?;
        let emotional_score = self
            .calculate_emotional_score(memory, context, &stats)
            .await?;
        let context_score = self
            .calculate_context_score(memory, context, &stats)
            .await?;
        let interaction_score = self
            .calculate_interaction_score(memory, context, &stats)
            .await?;

        // Calculate composite score with current weights
        let composite_score = self.calculate_composite_score(
            recency_score,
            frequency_score,
            relevance_score,
            emotional_score,
            context_score,
            interaction_score,
        );

        // Apply dynamic weight adjustment if enabled
        if self.config.enable_dynamic_weights {
            self.adjust_weights_based_on_performance(memory, composite_score)
                .await?;
        }

        Ok(ImportanceFactors {
            recency_score,
            frequency_score,
            relevance_score,
            emotional_score,
            context_score,
            interaction_score,
            composite_score,
            calculated_at: Utc::now(),
        })
    }

    /// Calculate recency score based on how recently the memory was accessed
    async fn calculate_recency_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let time_since_access = context
            .current_time
            .signed_duration_since(stats.last_accessed);
        let hours_since_access = time_since_access.num_hours() as f64;

        // Exponential decay function
        let decay_rate = match memory.importance {
            ImportanceLevel::Critical => 0.01, // Slower decay for critical memories
            ImportanceLevel::High => 0.02,
            ImportanceLevel::Medium => 0.05,
            ImportanceLevel::Low => 0.1, // Faster decay for low importance
        };

        let recency_score = (-decay_rate * hours_since_access).exp();
        Ok(recency_score.max(0.0).min(1.0))
    }

    /// Calculate frequency score based on access patterns
    async fn calculate_frequency_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let days_since_creation = context
            .current_time
            .signed_duration_since(stats.creation_time)
            .num_days()
            .max(1) as f64;

        // Access frequency per day
        let access_frequency = stats.access_count as f64 / days_since_creation;

        // Normalize using logarithmic scale
        let frequency_score = (1.0 + access_frequency).ln() / (1.0 + 100.0_f64).ln();
        Ok(frequency_score.max(0.0).min(1.0))
    }

    /// Calculate relevance score based on content similarity to current context
    async fn calculate_relevance_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        _stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let mut relevance_score: f64 = 0.0;

        // Check relevance to recent queries
        for query in &context.recent_queries {
            let similarity = self.calculate_text_similarity(&memory.content, query);
            relevance_score = relevance_score.max(similarity);
        }

        // Check relevance to current task
        if let Some(ref task) = context.current_task {
            let task_similarity = self.calculate_text_similarity(&memory.content, task);
            relevance_score = relevance_score.max(task_similarity);
        }

        // Check domain context relevance
        if let Some(ref domain) = context.domain_context {
            if memory.metadata.contains_key("domain") {
                if let Some(memory_domain) = memory.metadata.get("domain") {
                    if memory_domain == domain {
                        relevance_score = relevance_score.max(0.8);
                    }
                }
            }
        }

        Ok(relevance_score.max(0.0).min(1.0))
    }

    /// Calculate emotional score based on emotional content and user reactions
    async fn calculate_emotional_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let mut emotional_score: f64 = 0.0;

        // Analyze emotional keywords in content
        let emotional_keywords = vec![
            ("important", 0.8),
            ("critical", 0.9),
            ("urgent", 0.7),
            ("love", 0.6),
            ("hate", 0.6),
            ("excited", 0.5),
            ("worried", 0.4),
            ("happy", 0.5),
            ("sad", 0.4),
        ];

        let content_lower = memory.content.to_lowercase();
        for (keyword, weight) in emotional_keywords {
            if content_lower.contains(keyword) {
                emotional_score = emotional_score.max(weight);
            }
        }

        // Factor in user interaction frequency
        let interaction_boost = (stats.user_interactions as f64 / 10.0).min(0.3);
        emotional_score += interaction_boost;

        // Check user preferences
        if let Some(ref user_id) = context.user_id {
            if let Some(preference) = context.user_preferences.get(user_id) {
                emotional_score *= preference;
            }
        }

        Ok(emotional_score.max(0.0).min(1.0))
    }

    /// Calculate context score based on current situational context
    async fn calculate_context_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let mut context_score = 0.0;

        // Session context matching
        if let Some(ref session_id) = context.session_id {
            if memory.metadata.get("session_id") == Some(session_id) {
                context_score += 0.4;
            }
        }

        // User context matching
        if let Some(ref user_id) = context.user_id {
            if memory.metadata.get("user_id") == Some(user_id) {
                context_score += 0.3;
            }
        }

        // Time-based context (e.g., work hours, weekends)
        let hour = context.current_time.hour();
        let is_work_hours = (9..=17).contains(&hour);

        if let Some(memory_context) = memory.metadata.get("context_type") {
            match memory_context.as_str() {
                "work" if is_work_hours => context_score += 0.2,
                "personal" if !is_work_hours => context_score += 0.2,
                _ => {}
            }
        }

        // Context match history
        let context_match_ratio = stats.context_matches as f64 / stats.access_count.max(1) as f64;
        context_score += context_match_ratio * 0.1;

        Ok(context_score.max(0.0).min(1.0))
    }

    /// Calculate interaction score based on user engagement
    async fn calculate_interaction_score(
        &self,
        memory: &HierarchicalMemoryRecord,
        context: &ScoringContext,
        stats: &MemoryUsageStats,
    ) -> Result<f64> {
        let mut interaction_score = 0.0;

        // Direct user interactions
        let interaction_ratio = stats.user_interactions as f64 / stats.access_count.max(1) as f64;
        interaction_score += interaction_ratio * 0.5;

        // Reference count (how often this memory is referenced by others)
        let reference_boost = (stats.reference_count as f64 / 5.0).min(0.3);
        interaction_score += reference_boost;

        // Modification frequency (indicates active use)
        let modification_ratio = stats.modification_count as f64 / stats.access_count.max(1) as f64;
        interaction_score += modification_ratio * 0.2;

        Ok(interaction_score.max(0.0).min(1.0))
    }

    /// Calculate composite score using current weights
    fn calculate_composite_score(
        &self,
        recency: f64,
        frequency: f64,
        relevance: f64,
        emotional: f64,
        context: f64,
        interaction: f64,
    ) -> f64 {
        let weighted_score = recency * self.config.recency_weight
            + frequency * self.config.frequency_weight
            + relevance * self.config.relevance_weight
            + emotional * self.config.emotional_weight
            + context * self.config.context_weight
            + interaction * self.config.interaction_weight;

        // Apply bounds
        weighted_score
            .max(self.config.min_score_threshold)
            .min(self.config.max_score_cap)
    }

    /// Adjust weights based on performance feedback
    async fn adjust_weights_based_on_performance(
        &mut self,
        memory: &HierarchicalMemoryRecord,
        score: f64,
    ) -> Result<()> {
        // This is a simplified version of dynamic weight adjustment
        // In a full implementation, this would use machine learning techniques

        // Track performance metrics based on importance level instead of memory type
        let importance_key = format!("{:?}", memory.importance);
        self.performance_metrics.insert(importance_key, score);

        // Adjust weights based on importance level performance
        match memory.importance {
            ImportanceLevel::Critical => {
                // Critical memories benefit more from recency and emotional factors
                self.config.recency_weight += self.config.learning_rate * 0.1;
                self.config.emotional_weight += self.config.learning_rate * 0.1;
            }
            ImportanceLevel::High => {
                // High importance memories benefit more from relevance and frequency
                self.config.relevance_weight += self.config.learning_rate * 0.1;
                self.config.frequency_weight += self.config.learning_rate * 0.1;
            }
            ImportanceLevel::Medium => {
                // Medium importance memories benefit more from context and interaction
                self.config.context_weight += self.config.learning_rate * 0.1;
                self.config.interaction_weight += self.config.learning_rate * 0.1;
            }
            ImportanceLevel::Low => {
                // Low importance memories get balanced adjustment
                self.config.recency_weight += self.config.learning_rate * 0.05;
                self.config.relevance_weight += self.config.learning_rate * 0.05;
            }
        }

        // Normalize weights to ensure they sum to 1.0
        self.normalize_weights();

        Ok(())
    }

    /// Normalize weights to sum to 1.0
    fn normalize_weights(&mut self) {
        let total_weight = self.config.recency_weight
            + self.config.frequency_weight
            + self.config.relevance_weight
            + self.config.emotional_weight
            + self.config.context_weight
            + self.config.interaction_weight;

        if total_weight > 0.0 {
            self.config.recency_weight /= total_weight;
            self.config.frequency_weight /= total_weight;
            self.config.relevance_weight /= total_weight;
            self.config.emotional_weight /= total_weight;
            self.config.context_weight /= total_weight;
            self.config.interaction_weight /= total_weight;
        }
    }

    /// Calculate text similarity using simple word overlap
    fn calculate_text_similarity(&self, text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Get or create usage statistics for a memory
    fn get_or_create_stats(&mut self, memory_id: &str) -> &mut MemoryUsageStats {
        self.usage_stats
            .entry(memory_id.to_string())
            .or_default()
    }

    /// Update usage statistics for a memory
    pub fn update_usage_stats(&mut self, memory_id: &str, access_type: AccessType) {
        let stats = self.get_or_create_stats(memory_id);

        match access_type {
            AccessType::Read => {
                stats.access_count += 1;
                stats.last_accessed = Utc::now();
            }
            AccessType::Write => {
                stats.modification_count += 1;
                stats.last_accessed = Utc::now();
            }
            AccessType::UserInteraction => {
                stats.user_interactions += 1;
                stats.last_accessed = Utc::now();
            }
            AccessType::Reference => {
                stats.reference_count += 1;
            }
            AccessType::ContextMatch => {
                stats.context_matches += 1;
            }
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ImportanceScorerConfig {
        &self.config
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &HashMap<String, f64> {
        &self.performance_metrics
    }
}

/// Types of memory access for statistics tracking
#[derive(Debug, Clone, Copy)]
pub enum AccessType {
    Read,
    Write,
    UserInteraction,
    Reference,
    ContextMatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::{MemoryLevel, MemoryScope};

    fn create_test_memory() -> HierarchicalMemoryRecord {
        HierarchicalMemoryRecord {
            id: Uuid::new_v4().to_string(),
            content: "Test memory content".to_string(),
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
    async fn test_importance_scorer_creation() {
        let config = ImportanceScorerConfig::default();
        let scorer = AdvancedImportanceScorer::new(config);
        assert_eq!(scorer.usage_stats.len(), 0);
    }

    #[tokio::test]
    async fn test_importance_calculation() {
        let config = ImportanceScorerConfig::default();
        let mut scorer = AdvancedImportanceScorer::new(config);
        let memory = create_test_memory();
        let context = ScoringContext::default();

        let factors = scorer
            .calculate_importance(&memory, &context)
            .await
            .unwrap();

        assert!(factors.composite_score >= 0.0);
        assert!(factors.composite_score <= 10.0);
        assert!(factors.recency_score >= 0.0 && factors.recency_score <= 1.0);
        assert!(factors.frequency_score >= 0.0 && factors.frequency_score <= 1.0);
    }

    #[tokio::test]
    async fn test_usage_stats_update() {
        let config = ImportanceScorerConfig::default();
        let mut scorer = AdvancedImportanceScorer::new(config);
        let memory_id = "test-memory-id";

        scorer.update_usage_stats(memory_id, AccessType::Read);
        scorer.update_usage_stats(memory_id, AccessType::UserInteraction);

        let stats = scorer.usage_stats.get(memory_id).unwrap();
        assert_eq!(stats.access_count, 1);
        assert_eq!(stats.user_interactions, 1);
    }
}
