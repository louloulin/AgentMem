//! Adaptive Memory Strategy System
//!
//! Automatically adjusts memory management strategies based on usage patterns,
//! context, and performance metrics for optimal memory system behavior.

use crate::hierarchical_service::{ConflictResolutionStrategy, HierarchicalMemoryRecord};
use crate::importance_scorer::{ImportanceFactors, ScoringContext};
use crate::types::{ImportanceLevel, MemoryType};
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Configuration for adaptive strategy system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStrategyConfig {
    /// Enable automatic strategy adaptation
    pub enable_adaptation: bool,
    /// Adaptation learning rate (0.0-1.0)
    pub learning_rate: f64,
    /// Minimum samples needed for adaptation
    pub min_samples_for_adaptation: usize,
    /// Strategy evaluation window in hours
    pub evaluation_window_hours: u64,
    /// Performance threshold for strategy changes
    pub performance_threshold: f64,
    /// Maximum number of strategies to track
    pub max_strategy_history: usize,
    /// Enable predictive strategy selection
    pub enable_predictive_selection: bool,
}

impl Default for AdaptiveStrategyConfig {
    fn default() -> Self {
        Self {
            enable_adaptation: true,
            learning_rate: 0.1,
            min_samples_for_adaptation: 50,
            evaluation_window_hours: 24,
            performance_threshold: 0.7,
            max_strategy_history: 100,
            enable_predictive_selection: true,
        }
    }
}

/// Memory management strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryStrategy {
    /// Conservative strategy - prioritize data integrity
    Conservative,
    /// Aggressive strategy - prioritize performance
    Aggressive,
    /// Balanced strategy - balance between integrity and performance
    Balanced,
    /// Context-aware strategy - adapt based on current context
    ContextAware,
    /// User-centric strategy - prioritize user preferences
    UserCentric,
    /// Task-oriented strategy - optimize for current task
    TaskOriented,
}

/// Strategy performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy: MemoryStrategy,
    pub success_rate: f64,
    pub average_response_time: Duration,
    pub memory_efficiency: f64,
    pub user_satisfaction: f64,
    pub conflict_resolution_rate: f64,
    pub sample_count: usize,
    pub last_updated: DateTime<Utc>,
}

/// Context pattern for strategy selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContextPattern {
    pub user_type: Option<String>,
    pub task_category: Option<String>,
    pub time_of_day: Option<u32>,
    pub memory_load: Option<String>, // "low", "medium", "high"
    pub interaction_frequency: Option<String>, // "rare", "normal", "frequent"
}

/// Strategy recommendation with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendation {
    pub recommended_strategy: MemoryStrategy,
    pub confidence: f64,
    pub reasoning: Vec<String>,
    pub expected_performance: f64,
    pub alternative_strategies: Vec<(MemoryStrategy, f64)>,
}

/// Adaptive memory strategy manager
pub struct AdaptiveStrategyManager {
    config: AdaptiveStrategyConfig,
    current_strategy: MemoryStrategy,
    strategy_performance: HashMap<MemoryStrategy, StrategyPerformance>,
    context_patterns: HashMap<ContextPattern, MemoryStrategy>,
    performance_history: VecDeque<(DateTime<Utc>, MemoryStrategy, f64)>,
    adaptation_log: VecDeque<(DateTime<Utc>, MemoryStrategy, MemoryStrategy, String)>,
}

impl AdaptiveStrategyManager {
    /// Create a new adaptive strategy manager
    pub fn new(config: AdaptiveStrategyConfig) -> Self {
        let mut manager = Self {
            config,
            current_strategy: MemoryStrategy::Balanced,
            strategy_performance: HashMap::new(),
            context_patterns: HashMap::new(),
            performance_history: VecDeque::new(),
            adaptation_log: VecDeque::new(),
        };

        // Initialize default strategy performance metrics
        manager.initialize_default_strategies();
        manager
    }

    /// Get recommended strategy for current context
    pub async fn recommend_strategy(
        &mut self,
        context: &ScoringContext,
        recent_performance: Option<f64>,
    ) -> Result<StrategyRecommendation> {
        // Update performance if provided
        if let Some(performance) = recent_performance {
            self.record_performance(self.current_strategy.clone(), performance)
                .await?;
        }

        // Extract context pattern
        let pattern = self.extract_context_pattern(context);

        // Check for existing pattern-based strategy
        if let Some(strategy) = self.context_patterns.get(&pattern) {
            return Ok(StrategyRecommendation {
                recommended_strategy: strategy.clone(),
                confidence: 0.8,
                reasoning: vec!["Based on similar context patterns".to_string()],
                expected_performance: self.get_expected_performance(strategy),
                alternative_strategies: self.get_alternative_strategies(strategy),
            });
        }

        // Use predictive selection if enabled
        if self.config.enable_predictive_selection {
            return self.predict_optimal_strategy(context).await;
        }

        // Fall back to performance-based selection
        self.select_best_performing_strategy().await
    }

    /// Apply recommended strategy
    pub async fn apply_strategy(
        &mut self,
        strategy: MemoryStrategy,
        context: &ScoringContext,
    ) -> Result<()> {
        let old_strategy = self.current_strategy.clone();
        self.current_strategy = strategy.clone();

        // Log the strategy change
        let reason = format!("Strategy adapted from {:?} to {:?}", old_strategy, strategy);
        self.adaptation_log
            .push_back((Utc::now(), old_strategy, strategy.clone(), reason));

        // Limit adaptation log size
        if self.adaptation_log.len() > self.config.max_strategy_history {
            self.adaptation_log.pop_front();
        }

        // Update context pattern mapping
        let pattern = self.extract_context_pattern(context);
        self.context_patterns.insert(pattern, strategy);

        Ok(())
    }

    /// Get strategy-specific parameters for memory operations
    pub fn get_strategy_parameters(&self, strategy: &MemoryStrategy) -> StrategyParameters {
        match strategy {
            MemoryStrategy::Conservative => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::KeepBoth,
                importance_threshold: 0.3,
                cache_aggressiveness: 0.2,
                compression_level: 0.1,
                retention_period_days: 365,
                max_memory_per_scope: 10000,
            },
            MemoryStrategy::Aggressive => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::ImportanceBased,
                importance_threshold: 0.7,
                cache_aggressiveness: 0.9,
                compression_level: 0.8,
                retention_period_days: 30,
                max_memory_per_scope: 1000,
            },
            MemoryStrategy::Balanced => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::SemanticMerge,
                importance_threshold: 0.5,
                cache_aggressiveness: 0.5,
                compression_level: 0.5,
                retention_period_days: 90,
                max_memory_per_scope: 5000,
            },
            MemoryStrategy::ContextAware => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::TimeBasedNewest,
                importance_threshold: 0.4,
                cache_aggressiveness: 0.6,
                compression_level: 0.3,
                retention_period_days: 180,
                max_memory_per_scope: 7500,
            },
            MemoryStrategy::UserCentric => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::SourceReliabilityBased,
                importance_threshold: 0.3,
                cache_aggressiveness: 0.4,
                compression_level: 0.2,
                retention_period_days: 270,
                max_memory_per_scope: 8000,
            },
            MemoryStrategy::TaskOriented => StrategyParameters {
                conflict_resolution: ConflictResolutionStrategy::ImportanceBased,
                importance_threshold: 0.6,
                cache_aggressiveness: 0.8,
                compression_level: 0.6,
                retention_period_days: 60,
                max_memory_per_scope: 3000,
            },
        }
    }

    /// Record performance for a strategy
    async fn record_performance(
        &mut self,
        strategy: MemoryStrategy,
        performance: f64,
    ) -> Result<()> {
        // Update strategy performance metrics
        let perf = self
            .strategy_performance
            .entry(strategy.clone())
            .or_insert_with(|| StrategyPerformance {
                strategy: strategy.clone(),
                success_rate: 0.0,
                average_response_time: Duration::milliseconds(0),
                memory_efficiency: 0.0,
                user_satisfaction: 0.0,
                conflict_resolution_rate: 0.0,
                sample_count: 0,
                last_updated: Utc::now(),
            });

        // Update running averages
        let old_count = perf.sample_count as f64;
        let new_count = old_count + 1.0;

        perf.success_rate = (perf.success_rate * old_count + performance) / new_count;
        perf.sample_count += 1;
        perf.last_updated = Utc::now();

        // Add to performance history
        self.performance_history
            .push_back((Utc::now(), strategy, performance));

        // Limit history size
        if self.performance_history.len() > self.config.max_strategy_history {
            self.performance_history.pop_front();
        }

        // Check if adaptation is needed
        if self.config.enable_adaptation
            && perf.sample_count >= self.config.min_samples_for_adaptation
        {
            self.evaluate_adaptation_need().await?;
        }

        Ok(())
    }

    /// Extract context pattern from scoring context
    fn extract_context_pattern(&self, context: &ScoringContext) -> ContextPattern {
        ContextPattern {
            user_type: context.user_id.clone(),
            task_category: context.current_task.clone(),
            time_of_day: Some(context.current_time.hour()),
            memory_load: Some("medium".to_string()), // Simplified
            interaction_frequency: Some("normal".to_string()), // Simplified
        }
    }

    /// Predict optimal strategy using historical data
    async fn predict_optimal_strategy(
        &self,
        context: &ScoringContext,
    ) -> Result<StrategyRecommendation> {
        let mut strategy_scores: HashMap<MemoryStrategy, f64> = HashMap::new();

        // Score strategies based on historical performance
        for (strategy, performance) in &self.strategy_performance {
            let mut score = performance.success_rate;

            // Adjust score based on context similarity
            score *= self.calculate_context_similarity(context, strategy);

            // Adjust score based on recent performance
            score *= self.calculate_recency_weight(strategy);

            strategy_scores.insert(strategy.clone(), score);
        }

        // Find best strategy
        let best_strategy = strategy_scores
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(strategy, score)| (strategy.clone(), *score))
            .unwrap_or((MemoryStrategy::Balanced, 0.5));

        // Generate alternatives
        let mut alternatives: Vec<_> = strategy_scores
            .into_iter()
            .filter(|(s, _)| s != &best_strategy.0)
            .collect();
        alternatives
            .sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        alternatives.truncate(3);

        Ok(StrategyRecommendation {
            recommended_strategy: best_strategy.0,
            confidence: best_strategy.1,
            reasoning: vec!["Based on predictive analysis of historical performance".to_string()],
            expected_performance: best_strategy.1,
            alternative_strategies: alternatives,
        })
    }

    /// Select best performing strategy
    async fn select_best_performing_strategy(&self) -> Result<StrategyRecommendation> {
        let best_strategy = self
            .strategy_performance
            .iter()
            .max_by(|(_, a), (_, b)| {
                a.success_rate
                    .partial_cmp(&b.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(strategy, performance)| (strategy.clone(), performance.success_rate))
            .unwrap_or((MemoryStrategy::Balanced, 0.5));

        Ok(StrategyRecommendation {
            recommended_strategy: best_strategy.0,
            confidence: best_strategy.1,
            reasoning: vec!["Based on historical success rate".to_string()],
            expected_performance: best_strategy.1,
            alternative_strategies: Vec::new(),
        })
    }

    /// Calculate context similarity for strategy scoring
    fn calculate_context_similarity(
        &self,
        _context: &ScoringContext,
        _strategy: &MemoryStrategy,
    ) -> f64 {
        // Simplified implementation
        // In a full implementation, this would analyze context patterns
        1.0
    }

    /// Calculate recency weight for strategy scoring
    fn calculate_recency_weight(&self, strategy: &MemoryStrategy) -> f64 {
        if let Some(performance) = self.strategy_performance.get(strategy) {
            let hours_since_update = Utc::now()
                .signed_duration_since(performance.last_updated)
                .num_hours() as f64;

            // Exponential decay with 24-hour half-life
            0.5_f64.powf(hours_since_update / 24.0)
        } else {
            0.5 // Default weight for unknown strategies
        }
    }

    /// Get expected performance for a strategy
    fn get_expected_performance(&self, strategy: &MemoryStrategy) -> f64 {
        self.strategy_performance
            .get(strategy)
            .map(|p| p.success_rate)
            .unwrap_or(0.5)
    }

    /// Get alternative strategies with scores
    fn get_alternative_strategies(&self, current: &MemoryStrategy) -> Vec<(MemoryStrategy, f64)> {
        self.strategy_performance
            .iter()
            .filter(|(s, _)| *s != current)
            .map(|(s, p)| (s.clone(), p.success_rate))
            .collect()
    }

    /// Evaluate if adaptation is needed
    async fn evaluate_adaptation_need(&mut self) -> Result<()> {
        let current_performance = self.get_expected_performance(&self.current_strategy);

        if current_performance < self.config.performance_threshold {
            // Find better strategy
            if let Some((better_strategy, better_performance)) = self
                .strategy_performance
                .iter()
                .filter(|(s, _)| *s != &self.current_strategy)
                .max_by(|(_, a), (_, b)| {
                    a.success_rate
                        .partial_cmp(&b.success_rate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(s, p)| (s.clone(), p.success_rate))
            {
                if better_performance > current_performance + 0.1 {
                    // Significant improvement available
                    let reason = format!(
                        "Performance below threshold ({:.2} < {:.2}), switching to better strategy",
                        current_performance, self.config.performance_threshold
                    );

                    self.adaptation_log.push_back((
                        Utc::now(),
                        self.current_strategy.clone(),
                        better_strategy.clone(),
                        reason,
                    ));

                    self.current_strategy = better_strategy;
                }
            }
        }

        Ok(())
    }

    /// Initialize default strategy performance metrics
    fn initialize_default_strategies(&mut self) {
        let strategies = vec![
            MemoryStrategy::Conservative,
            MemoryStrategy::Aggressive,
            MemoryStrategy::Balanced,
            MemoryStrategy::ContextAware,
            MemoryStrategy::UserCentric,
            MemoryStrategy::TaskOriented,
        ];

        for strategy in strategies {
            self.strategy_performance.insert(
                strategy.clone(),
                StrategyPerformance {
                    strategy: strategy.clone(),
                    success_rate: 0.5, // Default neutral performance
                    average_response_time: Duration::milliseconds(100),
                    memory_efficiency: 0.5,
                    user_satisfaction: 0.5,
                    conflict_resolution_rate: 0.5,
                    sample_count: 0,
                    last_updated: Utc::now(),
                },
            );
        }
    }

    /// Get current strategy
    pub fn get_current_strategy(&self) -> &MemoryStrategy {
        &self.current_strategy
    }

    /// Get strategy performance metrics
    pub fn get_strategy_performance(&self) -> &HashMap<MemoryStrategy, StrategyPerformance> {
        &self.strategy_performance
    }

    /// Get adaptation history
    pub fn get_adaptation_log(
        &self,
    ) -> &VecDeque<(DateTime<Utc>, MemoryStrategy, MemoryStrategy, String)> {
        &self.adaptation_log
    }
}

/// Strategy-specific parameters for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    pub conflict_resolution: ConflictResolutionStrategy,
    pub importance_threshold: f64,
    pub cache_aggressiveness: f64,
    pub compression_level: f64,
    pub retention_period_days: u32,
    pub max_memory_per_scope: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_strategy_manager_creation() {
        let config = AdaptiveStrategyConfig::default();
        let manager = AdaptiveStrategyManager::new(config);

        assert_eq!(manager.current_strategy, MemoryStrategy::Balanced);
        assert_eq!(manager.strategy_performance.len(), 6); // All default strategies
    }

    #[tokio::test]
    async fn test_strategy_recommendation() -> anyhow::Result<()> {
        let config = AdaptiveStrategyConfig::default();
        let mut manager = AdaptiveStrategyManager::new(config);
        let context = ScoringContext::default();

        let recommendation = manager.recommend_strategy(&context, None).await?;

        // Check that we got a valid strategy
        match recommendation.recommended_strategy {
            MemoryStrategy::Conservative
            | MemoryStrategy::Aggressive
            | MemoryStrategy::Balanced
            | MemoryStrategy::ContextAware
            | MemoryStrategy::UserCentric
            | MemoryStrategy::TaskOriented => {
                // Valid strategy
            }
        Ok(())
        }
        assert!(recommendation.confidence >= 0.0 && recommendation.confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_performance_recording() {
        let config = AdaptiveStrategyConfig::default();
        let mut manager = AdaptiveStrategyManager::new(config);

        manager
            .record_performance(MemoryStrategy::Balanced, 0.8)
            .await
            .unwrap();

        let performance = manager
            .strategy_performance
            .get(&MemoryStrategy::Balanced)
            .unwrap();
        assert_eq!(performance.sample_count, 1);
        assert_eq!(performance.success_rate, 0.8);
    }

    #[test]
    fn test_strategy_parameters() {
        let config = AdaptiveStrategyConfig::default();
        let manager = AdaptiveStrategyManager::new(config);

        let params = manager.get_strategy_parameters(&MemoryStrategy::Conservative);
        assert_eq!(
            params.conflict_resolution,
            ConflictResolutionStrategy::KeepBoth
        );
        assert_eq!(params.importance_threshold, 0.3);

        let params = manager.get_strategy_parameters(&MemoryStrategy::Aggressive);
        assert_eq!(
            params.conflict_resolution,
            ConflictResolutionStrategy::ImportanceBased
        );
        assert_eq!(params.importance_threshold, 0.7);
    }
}
