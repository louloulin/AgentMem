//! Context-Aware Memory Management
//!
//! This module provides intelligent context understanding and application for memory management.
//! It implements advanced context-aware features including:
//! - Intelligent context extraction and analysis
//! - Context-based memory retrieval and ranking
//! - Adaptive context learning and pattern recognition
//! - Context-aware memory consolidation and organization

use crate::error::Result;
use crate::types::Memory;
use agent_mem_traits::{Entity, MemoryType, Relation, Session};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Context-aware memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareConfig {
    /// Enable context extraction from memory content
    pub enable_context_extraction: bool,
    /// Enable context-based memory ranking
    pub enable_context_ranking: bool,
    /// Enable adaptive context learning
    pub enable_adaptive_learning: bool,
    /// Context similarity threshold for grouping
    pub context_similarity_threshold: f32,
    /// Maximum context history to maintain
    pub max_context_history: usize,
    /// Context decay factor for temporal weighting
    pub context_decay_factor: f32,
    /// Enable context-based memory consolidation
    pub enable_context_consolidation: bool,
    /// Context pattern recognition threshold
    pub pattern_recognition_threshold: f32,
}

impl Default for ContextAwareConfig {
    fn default() -> Self {
        Self {
            enable_context_extraction: true,
            enable_context_ranking: true,
            enable_adaptive_learning: true,
            context_similarity_threshold: 0.7,
            max_context_history: 1000,
            context_decay_factor: 0.1,
            enable_context_consolidation: true,
            pattern_recognition_threshold: 0.8,
        }
    }
}

/// Context information extracted from memory or environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    /// Context ID
    pub id: String,
    /// Context type (e.g., "task", "location", "time", "mood", "topic")
    pub context_type: String,
    /// Context value or description
    pub value: String,
    /// Context confidence score
    pub confidence: f32,
    /// Context metadata
    pub metadata: HashMap<String, String>,
    /// Context timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Related entities
    pub entities: Vec<Entity>,
    /// Context relationships
    pub relations: Vec<Relation>,
}

/// Context pattern for learning and recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPattern {
    /// Pattern ID
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Context types involved in this pattern
    pub context_types: Vec<String>,
    /// Pattern frequency (how often this pattern occurs)
    pub frequency: u32,
    /// Pattern confidence score
    pub confidence: f32,
    /// Associated memory types
    pub memory_types: Vec<MemoryType>,
    /// Pattern triggers (conditions that activate this pattern)
    pub triggers: Vec<String>,
    /// Pattern outcomes (what happens when this pattern is active)
    pub outcomes: Vec<String>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Context-aware search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareSearchRequest {
    /// Search query
    pub query: String,
    /// Current context information
    pub current_context: Vec<ContextInfo>,
    /// Session information
    pub session: Session,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Minimum relevance score
    pub min_relevance: Option<f32>,
    /// Context weight in ranking (0.0 to 1.0)
    pub context_weight: Option<f32>,
    /// Enable context pattern matching
    pub enable_pattern_matching: bool,
}

/// Context-aware search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareSearchResult {
    /// Base search result
    pub memory: Memory,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f32,
    /// Context relevance score (0.0 to 1.0)
    pub context_score: f32,
    /// Combined score (relevance + context)
    pub combined_score: f32,
    /// Matching context information
    pub matching_contexts: Vec<ContextInfo>,
    /// Matching patterns
    pub matching_patterns: Vec<ContextPattern>,
    /// Context explanation
    pub context_explanation: String,
}

/// Context learning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLearningResult {
    /// Newly discovered patterns
    pub new_patterns: Vec<ContextPattern>,
    /// Updated patterns
    pub updated_patterns: Vec<ContextPattern>,
    /// Context insights
    pub insights: Vec<String>,
    /// Learning confidence
    pub confidence: f32,
}

/// Context-aware memory manager
pub struct ContextAwareManager {
    /// Configuration
    config: ContextAwareConfig,
    /// Context history
    context_history: Arc<RwLock<VecDeque<ContextInfo>>>,
    /// Learned patterns
    patterns: Arc<RwLock<HashMap<String, ContextPattern>>>,
    /// Context type frequencies
    context_frequencies: Arc<RwLock<HashMap<String, u32>>>,
    /// Memory-context associations
    memory_contexts: Arc<RwLock<HashMap<String, Vec<ContextInfo>>>>,
}

impl ContextAwareManager {
    /// Create a new context-aware manager
    pub async fn new(config: ContextAwareConfig) -> Result<Self> {
        Ok(Self {
            config,
            context_history: Arc::new(RwLock::new(VecDeque::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            context_frequencies: Arc::new(RwLock::new(HashMap::new())),
            memory_contexts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Extract context information from memory content
    pub async fn extract_context(
        &self,
        content: &str,
        session: &Session,
    ) -> Result<Vec<ContextInfo>> {
        if !self.config.enable_context_extraction {
            return Ok(Vec::new());
        }

        let mut contexts = Vec::new();

        // Extract temporal context
        if let Some(time_context) = self.extract_temporal_context(content).await? {
            contexts.push(time_context);
        }

        // Extract topic context
        if let Some(topic_context) = self.extract_topic_context(content).await? {
            contexts.push(topic_context);
        }

        // Extract emotional context
        if let Some(emotional_context) = self.extract_emotional_context(content).await? {
            contexts.push(emotional_context);
        }

        // Extract task context
        if let Some(task_context) = self.extract_task_context(content).await? {
            contexts.push(task_context);
        }

        // Extract location context
        if let Some(location_context) = self.extract_location_context(content).await? {
            contexts.push(location_context);
        }

        // Add session context
        let session_context = ContextInfo {
            id: Uuid::new_v4().to_string(),
            context_type: "session".to_string(),
            value: session.id.clone(),
            confidence: 1.0,
            metadata: {
                let mut meta = HashMap::new();
                if let Some(ref user_id) = session.user_id {
                    meta.insert("user_id".to_string(), user_id.clone());
                }
                if let Some(ref agent_id) = session.agent_id {
                    meta.insert("agent_id".to_string(), agent_id.clone());
                }
                meta
            },
            timestamp: chrono::Utc::now(),
            entities: Vec::new(),
            relations: Vec::new(),
        };
        contexts.push(session_context);

        Ok(contexts)
    }

    /// Perform context-aware search
    pub async fn search_with_context(
        &self,
        request: ContextAwareSearchRequest,
    ) -> Result<Vec<ContextAwareSearchResult>> {
        // Store current context in history
        self.update_context_history(&request.current_context)
            .await?;

        // Learn from current context
        if self.config.enable_adaptive_learning {
            self.learn_from_context(&request.current_context).await?;
        }

        // Perform basic search (this would integrate with the main search system)
        let basic_results = self
            .perform_basic_search(&request.query, &request.session)
            .await?;

        // Enhance results with context awareness
        let mut context_aware_results = Vec::new();
        for memory in basic_results {
            let context_score = self
                .calculate_context_score(&memory, &request.current_context)
                .await?;
            let matching_contexts = self
                .find_matching_contexts(&memory, &request.current_context)
                .await?;
            let matching_patterns = self
                .find_matching_patterns(&request.current_context)
                .await?;

            let relevance_score = memory.score.unwrap_or(0.5);
            let context_weight = request.context_weight.unwrap_or(0.3);
            let combined_score =
                relevance_score * (1.0 - context_weight) + context_score * context_weight;

            let context_explanation = self
                .generate_context_explanation(&matching_contexts, &matching_patterns)
                .await?;

            context_aware_results.push(ContextAwareSearchResult {
                memory,
                relevance_score,
                context_score,
                combined_score,
                matching_contexts,
                matching_patterns,
                context_explanation,
            });
        }

        // Sort by combined score
        context_aware_results.sort_by(|a, b| {
            b.combined_score
                .partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = request.limit {
            context_aware_results.truncate(limit);
        }

        Ok(context_aware_results)
    }

    /// Learn patterns from context
    pub async fn learn_from_context(
        &self,
        contexts: &[ContextInfo],
    ) -> Result<ContextLearningResult> {
        let mut new_patterns = Vec::new();
        let mut updated_patterns = Vec::new();
        let mut insights = Vec::new();

        // Update context frequencies
        {
            let mut frequencies = self.context_frequencies.write().await;
            for context in contexts {
                *frequencies.entry(context.context_type.clone()).or_insert(0) += 1;
            }
        }

        // Detect new patterns
        let detected_patterns = self.detect_context_patterns(contexts).await?;

        for pattern in detected_patterns {
            let mut patterns = self.patterns.write().await;
            if let Some(existing_pattern) = patterns.get_mut(&pattern.id) {
                // Update existing pattern
                existing_pattern.frequency += 1;
                existing_pattern.last_seen = chrono::Utc::now();
                existing_pattern.confidence =
                    (existing_pattern.confidence + pattern.confidence) / 2.0;
                updated_patterns.push(existing_pattern.clone());
            } else {
                // Add new pattern
                patterns.insert(pattern.id.clone(), pattern.clone());
                new_patterns.push(pattern);
            }
        }

        // Generate insights
        if !new_patterns.is_empty() {
            insights.push(format!(
                "Discovered {} new context patterns",
                new_patterns.len()
            ));
        }
        if !updated_patterns.is_empty() {
            insights.push(format!(
                "Updated {} existing patterns",
                updated_patterns.len()
            ));
        }

        let confidence = if new_patterns.is_empty() && updated_patterns.is_empty() {
            0.0
        } else {
            0.8
        };

        Ok(ContextLearningResult {
            new_patterns,
            updated_patterns,
            insights,
            confidence,
        })
    }

    /// Get context patterns
    pub async fn get_patterns(&self) -> Result<Vec<ContextPattern>> {
        let patterns = self.patterns.read().await;
        Ok(patterns.values().cloned().collect())
    }

    /// Get context history
    pub async fn get_context_history(&self, limit: Option<usize>) -> Result<Vec<ContextInfo>> {
        let history = self.context_history.read().await;
        let contexts: Vec<_> = history.iter().cloned().collect();

        if let Some(limit) = limit {
            Ok(contexts.into_iter().rev().take(limit).collect())
        } else {
            Ok(contexts.into_iter().rev().collect())
        }
    }

    /// Clear context history
    pub async fn clear_context_history(&self) -> Result<()> {
        let mut history = self.context_history.write().await;
        history.clear();
        Ok(())
    }

    // Private helper methods

    /// Extract temporal context from content
    async fn extract_temporal_context(&self, content: &str) -> Result<Option<ContextInfo>> {
        // Simple temporal context extraction
        let temporal_keywords = [
            "today",
            "yesterday",
            "tomorrow",
            "morning",
            "afternoon",
            "evening",
            "night",
        ];

        for keyword in &temporal_keywords {
            if content.to_lowercase().contains(keyword) {
                return Ok(Some(ContextInfo {
                    id: Uuid::new_v4().to_string(),
                    context_type: "temporal".to_string(),
                    value: keyword.to_string(),
                    confidence: 0.8,
                    metadata: HashMap::new(),
                    timestamp: chrono::Utc::now(),
                    entities: Vec::new(),
                    relations: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    /// Extract topic context from content
    async fn extract_topic_context(&self, content: &str) -> Result<Option<ContextInfo>> {
        // Simple topic extraction based on keywords
        let topics = [
            (
                "programming",
                vec![
                    "code",
                    "programming",
                    "software",
                    "development",
                    "bug",
                    "feature",
                ],
            ),
            (
                "work",
                vec!["meeting", "project", "deadline", "task", "work", "office"],
            ),
            (
                "personal",
                vec!["family", "friend", "hobby", "vacation", "personal", "home"],
            ),
            (
                "learning",
                vec!["study", "learn", "course", "book", "tutorial", "education"],
            ),
        ];

        for (topic, keywords) in &topics {
            for keyword in keywords {
                if content.to_lowercase().contains(keyword) {
                    return Ok(Some(ContextInfo {
                        id: Uuid::new_v4().to_string(),
                        context_type: "topic".to_string(),
                        value: topic.to_string(),
                        confidence: 0.7,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("keyword".to_string(), keyword.to_string());
                            meta
                        },
                        timestamp: chrono::Utc::now(),
                        entities: Vec::new(),
                        relations: Vec::new(),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Extract emotional context from content
    async fn extract_emotional_context(&self, content: &str) -> Result<Option<ContextInfo>> {
        let emotions = [
            (
                "positive",
                vec!["happy", "excited", "great", "awesome", "love", "enjoy"],
            ),
            (
                "negative",
                vec!["sad", "angry", "frustrated", "hate", "terrible", "awful"],
            ),
            (
                "neutral",
                vec!["okay", "fine", "normal", "regular", "standard"],
            ),
        ];

        for (emotion, keywords) in &emotions {
            for keyword in keywords {
                if content.to_lowercase().contains(keyword) {
                    return Ok(Some(ContextInfo {
                        id: Uuid::new_v4().to_string(),
                        context_type: "emotional".to_string(),
                        value: emotion.to_string(),
                        confidence: 0.6,
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("keyword".to_string(), keyword.to_string());
                            meta
                        },
                        timestamp: chrono::Utc::now(),
                        entities: Vec::new(),
                        relations: Vec::new(),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Extract task context from content
    async fn extract_task_context(&self, content: &str) -> Result<Option<ContextInfo>> {
        let task_indicators = [
            "need to", "should", "must", "have to", "plan to", "going to",
        ];

        for indicator in &task_indicators {
            if content.to_lowercase().contains(indicator) {
                return Ok(Some(ContextInfo {
                    id: Uuid::new_v4().to_string(),
                    context_type: "task".to_string(),
                    value: "action_required".to_string(),
                    confidence: 0.7,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("indicator".to_string(), indicator.to_string());
                        meta
                    },
                    timestamp: chrono::Utc::now(),
                    entities: Vec::new(),
                    relations: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    /// Extract location context from content
    async fn extract_location_context(&self, content: &str) -> Result<Option<ContextInfo>> {
        let locations = [
            "home",
            "office",
            "work",
            "school",
            "restaurant",
            "store",
            "park",
        ];

        for location in &locations {
            if content.to_lowercase().contains(location) {
                return Ok(Some(ContextInfo {
                    id: Uuid::new_v4().to_string(),
                    context_type: "location".to_string(),
                    value: location.to_string(),
                    confidence: 0.6,
                    metadata: HashMap::new(),
                    timestamp: chrono::Utc::now(),
                    entities: Vec::new(),
                    relations: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    /// Update context history
    async fn update_context_history(&self, contexts: &[ContextInfo]) -> Result<()> {
        let mut history = self.context_history.write().await;

        for context in contexts {
            history.push_back(context.clone());

            // Maintain maximum history size
            while history.len() > self.config.max_context_history {
                history.pop_front();
            }
        }

        Ok(())
    }

    /// Perform basic search (placeholder - would integrate with main search system)
    async fn perform_basic_search(&self, _query: &str, _session: &Session) -> Result<Vec<Memory>> {
        // This is a placeholder implementation
        // In a real implementation, this would call the main search system
        Ok(Vec::new())
    }

    /// Calculate context score for a memory
    async fn calculate_context_score(
        &self,
        memory: &Memory,
        current_contexts: &[ContextInfo],
    ) -> Result<f32> {
        let memory_contexts = {
            let contexts = self.memory_contexts.read().await;
            contexts.get(&memory.id).cloned().unwrap_or_default()
        };

        if memory_contexts.is_empty() || current_contexts.is_empty() {
            return Ok(0.0);
        }

        let mut total_score = 0.0;
        let mut matches = 0;

        for current_context in current_contexts {
            for memory_context in &memory_contexts {
                if current_context.context_type == memory_context.context_type {
                    let similarity = self
                        .calculate_context_similarity(current_context, memory_context)
                        .await?;
                    total_score += similarity;
                    matches += 1;
                }
            }
        }

        if matches > 0 {
            Ok(total_score / matches as f32)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate similarity between two contexts
    async fn calculate_context_similarity(
        &self,
        context1: &ContextInfo,
        context2: &ContextInfo,
    ) -> Result<f32> {
        if context1.context_type != context2.context_type {
            return Ok(0.0);
        }

        // Simple string similarity for now
        let similarity = if context1.value == context2.value {
            1.0
        } else if context1
            .value
            .to_lowercase()
            .contains(&context2.value.to_lowercase())
            || context2
                .value
                .to_lowercase()
                .contains(&context1.value.to_lowercase())
        {
            0.7
        } else {
            0.0
        };

        Ok(similarity)
    }

    /// Find matching contexts for a memory
    async fn find_matching_contexts(
        &self,
        memory: &Memory,
        current_contexts: &[ContextInfo],
    ) -> Result<Vec<ContextInfo>> {
        let memory_contexts = {
            let contexts = self.memory_contexts.read().await;
            contexts.get(&memory.id).cloned().unwrap_or_default()
        };

        let mut matching = Vec::new();

        for current_context in current_contexts {
            for memory_context in &memory_contexts {
                let similarity = self
                    .calculate_context_similarity(current_context, memory_context)
                    .await?;
                if similarity >= self.config.context_similarity_threshold {
                    matching.push(current_context.clone());
                    break;
                }
            }
        }

        Ok(matching)
    }

    /// Find matching patterns for current contexts
    async fn find_matching_patterns(
        &self,
        current_contexts: &[ContextInfo],
    ) -> Result<Vec<ContextPattern>> {
        let patterns = self.patterns.read().await;
        let mut matching = Vec::new();

        for pattern in patterns.values() {
            let mut pattern_score = 0.0;
            let mut matches = 0;

            for context in current_contexts {
                if pattern.context_types.contains(&context.context_type) {
                    pattern_score += context.confidence;
                    matches += 1;
                }
            }

            if matches > 0 {
                pattern_score /= matches as f32;
                if pattern_score >= self.config.pattern_recognition_threshold {
                    matching.push(pattern.clone());
                }
            }
        }

        Ok(matching)
    }

    /// Detect context patterns from current contexts
    async fn detect_context_patterns(
        &self,
        contexts: &[ContextInfo],
    ) -> Result<Vec<ContextPattern>> {
        let mut patterns = Vec::new();

        // Simple pattern detection: group contexts by type combinations
        if contexts.len() >= 2 {
            let context_types: Vec<String> =
                contexts.iter().map(|c| c.context_type.clone()).collect();
            let pattern_id = format!("pattern_{}", Uuid::new_v4());

            let pattern = ContextPattern {
                id: pattern_id,
                name: format!("Pattern: {}", context_types.join(" + ")),
                context_types,
                frequency: 1,
                confidence: 0.7,
                memory_types: vec![MemoryType::Episodic], // Default to episodic
                triggers: contexts.iter().map(|c| c.value.clone()).collect(),
                outcomes: Vec::new(),
                last_seen: chrono::Utc::now(),
            };

            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Generate context explanation
    async fn generate_context_explanation(
        &self,
        matching_contexts: &[ContextInfo],
        matching_patterns: &[ContextPattern],
    ) -> Result<String> {
        let mut explanation = String::new();

        if !matching_contexts.is_empty() {
            explanation.push_str("Relevant contexts: ");
            let context_descriptions: Vec<String> = matching_contexts
                .iter()
                .map(|c| format!("{}: {}", c.context_type, c.value))
                .collect();
            explanation.push_str(&context_descriptions.join(", "));
        }

        if !matching_patterns.is_empty() {
            if !explanation.is_empty() {
                explanation.push_str(". ");
            }
            explanation.push_str("Matching patterns: ");
            let pattern_descriptions: Vec<String> =
                matching_patterns.iter().map(|p| p.name.clone()).collect();
            explanation.push_str(&pattern_descriptions.join(", "));
        }

        if explanation.is_empty() {
            explanation = "No specific context relevance detected".to_string();
        }

        Ok(explanation)
    }

    /// Associate contexts with a memory
    pub async fn associate_contexts_with_memory(
        &self,
        memory_id: &str,
        contexts: Vec<ContextInfo>,
    ) -> Result<()> {
        let mut memory_contexts = self.memory_contexts.write().await;
        memory_contexts.insert(memory_id.to_string(), contexts);
        Ok(())
    }

    /// Get contexts associated with a memory
    pub async fn get_memory_contexts(&self, memory_id: &str) -> Result<Vec<ContextInfo>> {
        let memory_contexts = self.memory_contexts.read().await;
        Ok(memory_contexts.get(memory_id).cloned().unwrap_or_default())
    }

    /// Get context statistics
    pub async fn get_context_statistics(&self) -> Result<HashMap<String, u32>> {
        let frequencies = self.context_frequencies.read().await;
        Ok(frequencies.clone())
    }

    /// Reset all context data
    pub async fn reset(&self) -> Result<()> {
        {
            let mut history = self.context_history.write().await;
            history.clear();
        }
        {
            let mut patterns = self.patterns.write().await;
            patterns.clear();
        }
        {
            let mut frequencies = self.context_frequencies.write().await;
            frequencies.clear();
        }
        {
            let mut memory_contexts = self.memory_contexts.write().await;
            memory_contexts.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_aware_manager_creation() {
        let config = ContextAwareConfig::default();
        let manager = ContextAwareManager::new(config).await.unwrap();

        let patterns = manager.get_patterns().await.unwrap();
        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_context_extraction() {
        let config = ContextAwareConfig::default();
        let manager = ContextAwareManager::new(config).await.unwrap();

        let session = Session {
            id: "test_session".to_string(),
            user_id: Some("test_user".to_string()),
            agent_id: Some("test_agent".to_string()),
            run_id: None,
            actor_id: None,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let contexts = manager
            .extract_context("I need to finish my programming project today", &session)
            .await
            .unwrap();

        // Should extract multiple contexts
        assert!(!contexts.is_empty());

        // Should include session context
        assert!(contexts.iter().any(|c| c.context_type == "session"));
    }

    #[tokio::test]
    async fn test_context_learning() {
        let config = ContextAwareConfig::default();
        let manager = ContextAwareManager::new(config).await.unwrap();

        let contexts = vec![
            ContextInfo {
                id: Uuid::new_v4().to_string(),
                context_type: "topic".to_string(),
                value: "programming".to_string(),
                confidence: 0.8,
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                entities: Vec::new(),
                relations: Vec::new(),
            },
            ContextInfo {
                id: Uuid::new_v4().to_string(),
                context_type: "temporal".to_string(),
                value: "morning".to_string(),
                confidence: 0.7,
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
                entities: Vec::new(),
                relations: Vec::new(),
            },
        ];

        let result = manager.learn_from_context(&contexts).await.unwrap();

        assert!(!result.new_patterns.is_empty());
        assert!(result.confidence > 0.0);
    }
}
