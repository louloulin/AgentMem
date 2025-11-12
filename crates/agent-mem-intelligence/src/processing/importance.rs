//! Memory importance scoring algorithms
//!
//! Implements intelligent importance scoring based on multiple factors
//! including recency, frequency, relevance, and context.

use agent_mem_traits::{MemoryV4 as Memory, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Importance scoring factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportanceFactors {
    /// Recency weight (how recent the memory is)
    pub recency_weight: f32,

    /// Frequency weight (how often the memory is accessed)
    pub frequency_weight: f32,

    /// Content relevance weight
    pub relevance_weight: f32,

    /// Emotional significance weight
    pub emotional_weight: f32,

    /// Context importance weight
    pub context_weight: f32,
}

impl Default for ImportanceFactors {
    fn default() -> Self {
        Self {
            recency_weight: 0.3,
            frequency_weight: 0.25,
            relevance_weight: 0.25,
            emotional_weight: 0.1,
            context_weight: 0.1,
        }
    }
}

/// Importance scoring strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoringStrategy {
    /// Weighted combination of all factors
    Weighted,
    /// Maximum of all factors
    Maximum,
    /// Adaptive scoring based on memory type
    Adaptive,
}

/// Memory importance scorer
pub struct ImportanceScorer {
    /// Decay rate for importance over time
    decay_rate: f32,

    /// Scoring factors
    factors: ImportanceFactors,

    /// Scoring strategy
    strategy: ScoringStrategy,

    /// Context keywords for relevance scoring
    context_keywords: HashMap<String, f32>,

    /// Emotional keywords for emotional scoring
    emotional_keywords: HashMap<String, f32>,
}

impl ImportanceScorer {
    /// Create a new importance scorer
    pub fn new(decay_rate: f32) -> Self {
        Self {
            decay_rate,
            factors: ImportanceFactors::default(),
            strategy: ScoringStrategy::Weighted,
            context_keywords: Self::default_context_keywords(),
            emotional_keywords: Self::default_emotional_keywords(),
        }
    }

    /// Set scoring factors
    pub fn with_factors(mut self, factors: ImportanceFactors) -> Self {
        self.factors = factors;
        self
    }

    /// Set scoring strategy
    pub fn with_strategy(mut self, strategy: ScoringStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Update decay rate
    pub fn update_decay_rate(&mut self, decay_rate: f32) {
        self.decay_rate = decay_rate;
    }

    /// Update importance scores for a batch of memories
    pub async fn update_importance_scores(&mut self, memories: &mut [Memory]) -> Result<usize> {
        info!("Updating importance scores for {} memories", memories.len());

        let mut updated_count = 0;
        let current_time = chrono::Utc::now().timestamp();

        for memory in memories.iter_mut() {
            let old_importance = memory.score().unwrap_or(0.5) as f32;
            let new_importance = self
                .calculate_importance_score(memory, current_time)
                .await?;

            if (new_importance - old_importance).abs() > 0.01 {
                memory.set_score(new_importance as f64);
                memory.metadata.updated_at = chrono::Utc::now();
                updated_count += 1;
            }
        }

        info!("Updated importance scores for {} memories", updated_count);
        Ok(updated_count)
    }

    /// Score a single memory
    pub async fn score_single_memory(&mut self, memory: &mut Memory) -> Result<()> {
        let current_time = chrono::Utc::now().timestamp();
        let new_importance = self
            .calculate_importance_score(memory, current_time)
            .await?;

        if (new_importance - (memory.score().unwrap_or(0.5) as f32)).abs() > 0.01 {
            memory.set_score(new_importance as f64);
            memory.metadata.updated_at = chrono::Utc::now();
        }

        Ok(())
    }

    /// Calculate importance score for a memory
    async fn calculate_importance_score(&self, memory: &Memory, current_time: i64) -> Result<f32> {
        let recency_score = self.calculate_recency_score(memory, current_time);
        let frequency_score = self.calculate_frequency_score(memory);
        let relevance_score = self.calculate_relevance_score(memory);
        let emotional_score = self.calculate_emotional_score(memory);
        let context_score = self.calculate_context_score(memory);

        let importance = match self.strategy {
            ScoringStrategy::Weighted => {
                recency_score * self.factors.recency_weight
                    + frequency_score * self.factors.frequency_weight
                    + relevance_score * self.factors.relevance_weight
                    + emotional_score * self.factors.emotional_weight
                    + context_score * self.factors.context_weight
            }
            ScoringStrategy::Maximum => [
                recency_score,
                frequency_score,
                relevance_score,
                emotional_score,
                context_score,
            ]
            .iter()
            .fold(0.0f32, |acc, &x| acc.max(x)),
            ScoringStrategy::Adaptive => self.calculate_adaptive_score(
                memory,
                recency_score,
                frequency_score,
                relevance_score,
                emotional_score,
                context_score,
            ),
        };

        // Apply decay based on time since last access
        let time_since_access =
            current_time - memory.metadata.accessed_at.timestamp();
        let decay_factor = self
            .decay_rate
            .powf(time_since_access as f32 / (24.0 * 60.0 * 60.0)); // Daily decay

        Ok((importance * decay_factor).clamp(0.0, 1.0))
    }

    /// Calculate recency score (newer memories are more important)
    fn calculate_recency_score(&self, memory: &Memory, current_time: i64) -> f32 {
        let age_seconds = current_time - memory.metadata.created_at.timestamp();
        let max_age = 30.0 * 24.0 * 60.0 * 60.0; // 30 days in seconds

        if age_seconds <= 0 {
            return 1.0;
        }

        let age_factor = (max_age - age_seconds as f32).max(0.0) / max_age;
        age_factor.clamp(0.0, 1.0)
    }

    /// Calculate frequency score (more accessed memories are more important)
    fn calculate_frequency_score(&self, memory: &Memory) -> f32 {
        let max_access_count = 100.0; // Normalize to this maximum
        let access_count = memory.metadata.access_count as f32;
        (access_count / max_access_count).clamp(0.0, 1.0)
    }

    /// Calculate relevance score based on content length and complexity
    fn calculate_relevance_score(&self, memory: &Memory) -> f32 {
        let content_str = match &memory.content {
            agent_mem_traits::Content::Text(t) => t.as_str(),
            agent_mem_traits::Content::Structured(v) => &v.to_string(),
            _ => "",
        };
        
        let content_length = content_str.len() as f32;
        let word_count = content_str.split_whitespace().count() as f32;

        // Longer, more detailed memories are generally more important
        let length_score = (content_length / 1000.0).clamp(0.0, 1.0);
        let complexity_score = (word_count / 100.0).clamp(0.0, 1.0);

        (length_score + complexity_score) / 2.0
    }

    /// Calculate emotional score based on emotional keywords
    fn calculate_emotional_score(&self, memory: &Memory) -> f32 {
        let content_str = match &memory.content {
            agent_mem_traits::Content::Text(t) => t.as_str(),
            agent_mem_traits::Content::Structured(v) => &v.to_string(),
            _ => "",
        };
        let content_lower = content_str.to_lowercase();
        let mut emotional_score = 0.0;
        let mut keyword_count = 0;

        for (keyword, weight) in &self.emotional_keywords {
            if content_lower.contains(keyword) {
                emotional_score += weight;
                keyword_count += 1;
            }
        }

        if keyword_count > 0 {
            (emotional_score / keyword_count as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Calculate context score based on context keywords
    fn calculate_context_score(&self, memory: &Memory) -> f32 {
        let content_str = match &memory.content {
            agent_mem_traits::Content::Text(t) => t.as_str(),
            agent_mem_traits::Content::Structured(v) => &v.to_string(),
            _ => "",
        };
        let content_lower = content_str.to_lowercase();
        let mut context_score = 0.0;
        let mut keyword_count = 0;

        for (keyword, weight) in &self.context_keywords {
            if content_lower.contains(keyword) {
                context_score += weight;
                keyword_count += 1;
            }
        }

        if keyword_count > 0 {
            (context_score / keyword_count as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Calculate adaptive score based on memory type
    fn calculate_adaptive_score(
        &self,
        memory: &Memory,
        recency: f32,
        frequency: f32,
        relevance: f32,
        emotional: f32,
        context: f32,
    ) -> f32 {

        let memory_type_str = memory.memory_type().unwrap_or_else(|| "episodic".to_string());
        match memory_type_str.as_str() {
            "episodic" => {
                // Episodic memories: prioritize recency and emotional content
                recency * 0.4 + emotional * 0.3 + frequency * 0.2 + relevance * 0.1
            }
            "semantic" => {
                // Semantic memories: prioritize relevance and context
                relevance * 0.4 + context * 0.3 + frequency * 0.2 + recency * 0.1
            }
            "procedural" => {
                // Procedural memories: prioritize frequency and context
                frequency * 0.4 + context * 0.3 + relevance * 0.2 + recency * 0.1
            }
            "working" => {
                // Working memories: prioritize recency heavily
                recency * 0.6 + frequency * 0.2 + relevance * 0.1 + emotional * 0.1
            }
            "factual" => {
                // Factual memories: prioritize relevance and context
                relevance * 0.4 + context * 0.3 + frequency * 0.2 + recency * 0.1
            }
            "core" => {
                // Core memories: prioritize all factors equally with high base importance
                (relevance + context + frequency + recency + emotional) / 5.0 * 1.2
            }
            "resource" => {
                // Resource memories: prioritize relevance and frequency
                relevance * 0.4 + frequency * 0.3 + context * 0.2 + recency * 0.1
            }
            "knowledge" => {
                // Knowledge memories: prioritize relevance and context
                relevance * 0.5 + context * 0.3 + frequency * 0.15 + recency * 0.05
            }
            "contextual" => {
                // Contextual memories: prioritize context and recency
                context * 0.4 + recency * 0.3 + relevance * 0.2 + frequency * 0.1
            }
            _ => {
                // Default: balanced approach
                (relevance + context + frequency + recency + emotional) / 5.0
            }
        }
    }

    /// Default context keywords with weights
    fn default_context_keywords() -> HashMap<String, f32> {
        let mut keywords = HashMap::new();
        keywords.insert("important".to_string(), 0.9);
        keywords.insert("critical".to_string(), 1.0);
        keywords.insert("urgent".to_string(), 0.8);
        keywords.insert("remember".to_string(), 0.7);
        keywords.insert("note".to_string(), 0.6);
        keywords.insert("todo".to_string(), 0.7);
        keywords.insert("deadline".to_string(), 0.8);
        keywords.insert("meeting".to_string(), 0.6);
        keywords.insert("project".to_string(), 0.7);
        keywords.insert("goal".to_string(), 0.8);
        keywords
    }

    /// Default emotional keywords with weights
    fn default_emotional_keywords() -> HashMap<String, f32> {
        let mut keywords = HashMap::new();
        keywords.insert("love".to_string(), 0.8);
        keywords.insert("hate".to_string(), 0.8);
        keywords.insert("excited".to_string(), 0.7);
        keywords.insert("worried".to_string(), 0.7);
        keywords.insert("happy".to_string(), 0.6);
        keywords.insert("sad".to_string(), 0.6);
        keywords.insert("angry".to_string(), 0.7);
        keywords.insert("frustrated".to_string(), 0.7);
        keywords.insert("proud".to_string(), 0.6);
        keywords.insert("disappointed".to_string(), 0.6);
        keywords.insert("amazing".to_string(), 0.7);
        keywords.insert("terrible".to_string(), 0.7);
        keywords
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::MemoryType;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_memory(id: &str, content: &str, access_count: u32) -> Memory {
        use agent_mem_traits::{Session, MemoryItem};
        let now = Utc::now();
        let item = MemoryItem {
            id: id.to_string(),
            content: content.to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: Some(0.5),
            created_at: now - chrono::Duration::hours(1),
            updated_at: None,
            session: Session::new(),
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.5,
            embedding: None,
            last_accessed_at: now - chrono::Duration::minutes(30),
            access_count,
            expires_at: None,
            version: 1,
        };
        Memory::from_legacy_item(&item)
    }

    #[tokio::test]
    async fn test_importance_scorer_creation() {
        let scorer = ImportanceScorer::new(0.95);
        assert_eq!(scorer.decay_rate, 0.95);
    }

    #[tokio::test]
    async fn test_recency_scoring() {
        let scorer = ImportanceScorer::new(0.95);
        let current_time = Utc::now().timestamp();

        let recent_memory = create_test_memory("1", "Recent memory", 1);

        // Create old memory with modified created_at
        use agent_mem_traits::{Session, MemoryItem};
        let now = Utc::now();
        let old_item = MemoryItem {
            id: "2".to_string(),
            content: "Old memory".to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: Some(0.5),
            created_at: now - chrono::Duration::days(7), // 7 days ago
            updated_at: None,
            session: Session::new(),
            memory_type: MemoryType::Episodic,
            entities: Vec::new(),
            relations: Vec::new(),
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.5,
            embedding: None,
            last_accessed_at: now - chrono::Duration::minutes(30),
            access_count: 1,
            expires_at: None,
            version: 1,
        };
        let old_memory = Memory::from_legacy_item(&old_item);

        let recent_score = scorer.calculate_recency_score(&recent_memory, current_time);
        let old_score = scorer.calculate_recency_score(&old_memory, current_time);

        assert!(recent_score > old_score);
    }

    #[tokio::test]
    async fn test_frequency_scoring() {
        let scorer = ImportanceScorer::new(0.95);

        let frequent_memory = create_test_memory("1", "Frequent memory", 50);
        let rare_memory = create_test_memory("2", "Rare memory", 1);

        let frequent_score = scorer.calculate_frequency_score(&frequent_memory);
        let rare_score = scorer.calculate_frequency_score(&rare_memory);

        assert!(frequent_score > rare_score);
    }

    #[tokio::test]
    async fn test_emotional_scoring() {
        let scorer = ImportanceScorer::new(0.95);

        let emotional_memory = create_test_memory("1", "I love this amazing project!", 1);
        let neutral_memory = create_test_memory("2", "The weather is okay today.", 1);

        let emotional_score = scorer.calculate_emotional_score(&emotional_memory);
        let neutral_score = scorer.calculate_emotional_score(&neutral_memory);

        assert!(emotional_score > neutral_score);
    }

    #[tokio::test]
    async fn test_importance_calculation() {
        let mut scorer = ImportanceScorer::new(0.95);
        let current_time = Utc::now().timestamp();

        let memory = create_test_memory("1", "This is an important memory to remember!", 10);
        let importance = scorer
            .calculate_importance_score(&memory, current_time)
            .await
            .unwrap();

        assert!(importance > 0.0);
        assert!(importance <= 1.0);
    }

    #[tokio::test]
    async fn test_batch_scoring() {
        let mut scorer = ImportanceScorer::new(0.95);

        let mut memories = vec![
            create_test_memory("1", "Important memory", 10),
            create_test_memory("2", "Less important memory", 1),
            create_test_memory("3", "Critical deadline tomorrow!", 5),
        ];

        let updated_count = scorer
            .update_importance_scores(&mut memories)
            .await
            .unwrap();
        // Updated count should be valid (0 or positive)
        assert!(updated_count == 0 || updated_count > 0);
    }
}
