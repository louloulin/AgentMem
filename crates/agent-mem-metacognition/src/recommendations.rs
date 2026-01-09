//! Recommendation Engine
//!
//! Provides intelligent recommendations for memory optimization.
//!
//! # Theory
//!
//! Recommendations are generated based on:
//! - Memory health metrics
//! - Usage patterns
//! - Consolidation history
//! - Performance bottlenecks
//!
//! # Example
//!
//! ```no_run
//! use agent_mem_metacognition::RecommendationEngine;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let engine = RecommendationEngine::new();
//!
//!     let recommendations = engine.generate_recommendations().await?;
//!     for rec in recommendations {
//!         println!("{}: {}", rec.recommendation_type, rec.description);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::metacognition::{MetacognitionReport, MemoryHealthMetrics};
use agent_mem_traits::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Recommendation type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Consolidation recommended
    Consolidation,
    
    /// Memory cleanup recommended
    Cleanup,
    
    /// Performance optimization
    Performance,
    
    /// Storage optimization
    Storage,
    
    /// Index optimization
    Indexing,
    
    /// General advice
    General,
}

/// Recommendation priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Single recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    
    /// Priority level
    pub priority: RecommendationPriority,
    
    /// Human-readable description
    pub description: String,
    
    /// Expected impact (0-100)
    pub expected_impact: u8,
    
    /// Estimated effort (low/medium/high)
    pub effort: String,
    
    /// Actionable steps
    pub steps: Vec<String>,
    
    /// Generated timestamp
    pub generated_at: DateTime<Utc>,
}

impl Recommendation {
    /// Create new recommendation
    pub fn new(
        recommendation_type: RecommendationType,
        priority: RecommendationPriority,
        description: String,
    ) -> Self {
        Self {
            recommendation_type,
            priority,
            description,
            expected_impact: 50,
            effort: "medium".to_string(),
            steps: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    /// Set expected impact
    pub fn with_impact(mut self, impact: u8) -> Self {
        self.expected_impact = impact.min(100);
        self
    }

    /// Set effort
    pub fn with_effort(mut self, effort: &str) -> Self {
        self.effort = effort.to_string();
        self
    }

    /// Add step
    pub fn add_step(mut self, step: &str) -> Self {
        self.steps.push(step.to_string());
        self
    }
}

/// Recommendation engine
///
/// Generates recommendations for memory optimization.
pub struct RecommendationEngine {
    /// Minimum urgency threshold for generating recommendations
    urgency_threshold: f64,
}

impl RecommendationEngine {
    /// Create new recommendation engine
    pub fn new() -> Self {
        Self {
            urgency_threshold: 50.0,
        }
    }

    /// Set urgency threshold
    pub fn with_urgency_threshold(mut self, threshold: f64) -> Self {
        self.urgency_threshold = threshold;
        self
    }

    /// Generate recommendations based on metacognitive report
    pub async fn generate_recommendations(
        &self,
        report: &MetacognitionReport,
    ) -> Result<Vec<Recommendation>> {
        info!("Generating recommendations based on metacognitive report");

        let mut recommendations = Vec::new();

        // Check consolidation urgency
        if report.health.consolidation_urgency > self.urgency_threshold {
            let priority = if report.health.consolidation_urgency > 80.0 {
                RecommendationPriority::Critical
            } else if report.health.consolidation_urgency > 60.0 {
                RecommendationPriority::High
            } else {
                RecommendationPriority::Medium
            };

            let impact = ((report.health.consolidation_urgency / 100.0) * 100.0) as u8;

            recommendations.push(
                Recommendation::new(
                    RecommendationType::Consolidation,
                    priority,
                    format!(
                        "Consolidation recommended: {} memories show high fragmentation (urgency: {:.1}%)",
                        report.health.total_memories,
                        report.health.consolidation_urgency
                    ),
                )
                .with_impact(impact)
                .with_effort("low")
                .add_step("Run automatic consolidation")
                .add_step("Review merge candidates")
                .add_step("Verify merged memories"),
            );
        }

        // Check health score
        if report.health.health_score < 60.0 {
            recommendations.push(
                Recommendation::new(
                    RecommendationType::Cleanup,
                    RecommendationPriority::High,
                    format!(
                        "Memory health is low ({:.1}/100). Consider cleanup operations.",
                        report.health.health_score
                    ),
                )
                .with_impact(70)
                .with_effort("medium")
                .add_step("Identify dormant memories")
                .add_step("Archive or delete old memories")
                .add_step("Rebuild indexes"),
            );
        }

        // Check dormant memories
        if report.health.dormant_memories > report.health.total_memories / 4 {
            recommendations.push(
                Recommendation::new(
                    RecommendationType::Storage,
                    RecommendationPriority::Medium,
                    format!(
                        "High number of dormant memories detected ({}). Consider archival.",
                        report.health.dormant_memories
                    ),
                )
                .with_impact(60)
                .with_effort("low")
                .add_step("Move dormant memories to cold storage")
                .add_step("Update archival policy"),
            );
        }

        // Performance recommendations
        if report.performance.avg_retrieval_time_ms > 100.0 {
            recommendations.push(
                Recommendation::new(
                    RecommendationType::Performance,
                    RecommendationPriority::Medium,
                    format!(
                        "Retrieval performance below optimal ({:.1}ms avg). Consider optimization.",
                        report.performance.avg_retrieval_time_ms
                    ),
                )
                .with_impact(65)
                .with_effort("medium")
                .add_step("Review indexing strategy")
                .add_step("Consider cache warming")
                .add_step("Optimize database queries"),
            );
        }

        // Cache recommendations
        if report.performance.cache_hit_rate < 0.7 {
            recommendations.push(
                Recommendation::new(
                    RecommendationType::Performance,
                    RecommendationPriority::Medium,
                    format!(
                        "Low cache hit rate ({:.1}%). Consider cache optimization.",
                        report.performance.cache_hit_rate * 100.0
                    ),
                )
                .with_impact(55)
                .with_effort("low")
                .add_step("Increase cache size")
                .add_step("Review cache eviction policy")
                .add_step("Preload frequently accessed memories"),
            );
        }

        // General recommendations if no critical issues
        if recommendations.is_empty() {
            recommendations.push(
                Recommendation::new(
                    RecommendationType::General,
                    RecommendationPriority::Low,
                    "Memory system is healthy. Continue monitoring.".to_string(),
                )
                .with_impact(10)
                .with_effort("none")
                .add_step("Schedule regular health checks")
                .add_step("Review metacognitive reports monthly"),
            );
        }

        // Sort by priority
        recommendations.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!("Generated {} recommendations", recommendations.len());
        Ok(recommendations)
    }

    /// Generate simple recommendations from health metrics
    pub async fn generate_from_health(
        &self,
        health: &MemoryHealthMetrics,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        if health.consolidation_urgency > self.urgency_threshold {
            recommendations.push(Recommendation::new(
                RecommendationType::Consolidation,
                RecommendationPriority::High,
                format!("Consolidation needed (urgency: {:.1}%)", health.consolidation_urgency),
            ));
        }

        if health.health_score < 70.0 {
            recommendations.push(Recommendation::new(
                RecommendationType::Cleanup,
                RecommendationPriority::Medium,
                format!("Health score below optimal ({:.1}/100)", health.health_score),
            ));
        }

        Ok(recommendations)
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metacognition::{MemoryHealthMetrics, MemoryUsageStats, PerformanceMetrics, MetacognitionReport, MergeStatisticsSummary};

    #[tokio::test]
    async fn test_recommendation_creation() {
        let rec = Recommendation::new(
            RecommendationType::Consolidation,
            RecommendationPriority::High,
            "Test recommendation".to_string(),
        );

        assert_eq!(rec.recommendation_type, RecommendationType::Consolidation);
        assert_eq!(rec.priority, RecommendationPriority::High);
    }

    #[tokio::test]
    async fn test_recommendation_builder() {
        let rec = Recommendation::new(
            RecommendationType::Cleanup,
            RecommendationPriority::Medium,
            "Test".to_string(),
        )
        .with_impact(80)
        .with_effort("low")
        .add_step("Step 1")
        .add_step("Step 2");

        assert_eq!(rec.expected_impact, 80);
        assert_eq!(rec.effort, "low");
        assert_eq!(rec.steps.len(), 2);
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = RecommendationEngine::new();
        assert_eq!(engine.urgency_threshold, 50.0);
    }

    #[tokio::test]
    async fn test_generate_recommendations() {
        let engine = RecommendationEngine::new();

        let report = MetacognitionReport {
            generated_at: Utc::now(),
            health: MemoryHealthMetrics {
                total_memories: 1000,
                active_memories: 700,
                dormant_memories: 200,
                fragmented_memories: 100,
                health_score: 50.0,
                consolidation_urgency: 85.0,
            },
            usage: MemoryUsageStats {
                total_accesses: 10000,
                avg_accesses_per_memory: 10.0,
                most_accessed_memory_id: Some("mem-1".to_string()),
                access_distribution: AccessDistribution {
                    q1: 5,
                    q2: 10,
                    q3: 15,
                    max: 50,
                },
            },
            performance: PerformanceMetrics {
                avg_retrieval_time_ms: 50.0,
                avg_consolidation_time_ms: 200.0,
                cache_hit_rate: 0.85,
                throughput_ops_per_sec: 1000.0,
            },
            merge_stats: MergeStatisticsSummary {
                total_merges: 100,
                merges_last_24h: 5,
                avg_memories_per_merge: 2.5,
                consolidation_rate_per_day: 5.0,
            },
            recommendations_count: 0,
            health_score: 50.0,
        };

        // AccessDistribution needs to be in scope
        use crate::metacognition::AccessDistribution;

        let recommendations = engine.generate_recommendations(&report).await.unwrap();
        assert!(!recommendations.is_empty());
    }
}
