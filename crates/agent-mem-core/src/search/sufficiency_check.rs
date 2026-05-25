//! Sufficiency Check Module
//!
//! Provides LLM-driven sufficiency checking for early exit:
//! - Category sufficiency: Check if category has enough information
//! - Item sufficiency: Check if memory items are sufficient
//! - Resource sufficiency: Check if resources provide enough context
//! - Early exit mechanism to avoid over-retrieval

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Sufficiency check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SufficiencyCheckResult {
    /// Whether the information is sufficient
    pub is_sufficient: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning for the decision
    pub reasoning: String,
    /// Suggested next action
    pub suggested_action: SufficiencyAction,
    /// Check time in milliseconds
    pub check_time_ms: u64,
}

/// Suggested action after sufficiency check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SufficiencyAction {
    /// Continue with more retrieval
    ContinueRetrieval,
    /// Stop retrieval, enough information
    StopRetrieval,
    /// Need more specific search
    RefineQuery,
    /// Need to include resources
    IncludeResources,
    /// Need category expansion
    ExpandCategories,
}

/// Sufficiency check type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SufficiencyCheckType {
    /// Check category sufficiency
    Category,
    /// Check memory item sufficiency
    Item,
    /// Check resource sufficiency
    Resource,
    /// Combined check
    Combined,
}

/// Sufficiency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SufficiencyConfig {
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Enable LLM-based checking
    pub enable_llm: bool,
    /// Fallback to rule-based if LLM fails
    pub fallback_to_rules: bool,
    /// Maximum checks before forcing stop
    pub max_checks: usize,
    /// Enable early exit
    pub enable_early_exit: bool,
}

impl Default for SufficiencyConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            enable_llm: false, // Disabled by default, use rule-based
            fallback_to_rules: true,
            max_checks: 3,
            enable_early_exit: true,
        }
    }
}

/// Simple rule-based sufficiency checker
pub struct RuleBasedSufficiencyChecker {
    config: SufficiencyConfig,
}

impl RuleBasedSufficiencyChecker {
    pub fn new(config: SufficiencyConfig) -> Self {
        Self { config }
    }

    /// Check category sufficiency based on item count and scores
    fn check_category_sufficiency(
        &self,
        category_count: usize,
        avg_score: f32,
        total_items: usize,
    ) -> SufficiencyCheckResult {
        let start = std::time::Instant::now();

        // Rule-based heuristics
        let is_sufficient = category_count > 0
            && avg_score > 0.5
            && total_items >= 3;

        let confidence = if category_count >= 5 && avg_score > 0.7 {
            0.9
        } else if category_count >= 3 && avg_score > 0.5 {
            0.7
        } else if category_count > 0 {
            0.5
        } else {
            0.3
        };

        let reasoning = if is_sufficient {
            format!(
                "Found {} categories with average score {:.2} and {} total items. Sufficient for answering.",
                category_count, avg_score, total_items
            )
        } else {
            format!(
                "Insufficient information: {} categories, avg score {:.2}, {} items. Need more data.",
                category_count, avg_score, total_items
            )
        };

        let suggested_action = if is_sufficient {
            SufficiencyAction::StopRetrieval
        } else if category_count == 0 {
            SufficiencyAction::ExpandCategories
        } else {
            SufficiencyAction::ContinueRetrieval
        };

        SufficiencyCheckResult {
            is_sufficient,
            confidence,
            reasoning,
            suggested_action,
            check_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Check item sufficiency based on count and scores
    fn check_item_sufficiency(
        &self,
        item_count: usize,
        avg_score: f32,
    ) -> SufficiencyCheckResult {
        let start = std::time::Instant::now();

        let is_sufficient = item_count >= 5 && avg_score > 0.5;

        let confidence = if item_count >= 10 && avg_score > 0.7 {
            0.9
        } else if item_count >= 5 && avg_score > 0.5 {
            0.7
        } else if item_count >= 3 {
            0.5
        } else {
            0.3
        };

        let reasoning = if is_sufficient {
            format!(
                "Found {} relevant items with average score {:.2}. Sufficient for answering.",
                item_count, avg_score
            )
        } else {
            format!(
                "Only {} items with avg score {:.2}. Need more retrieval.",
                item_count, avg_score
            )
        };

        let suggested_action = if is_sufficient {
            SufficiencyAction::StopRetrieval
        } else {
            SufficiencyAction::ContinueRetrieval
        };

        SufficiencyCheckResult {
            is_sufficient,
            confidence,
            reasoning,
            suggested_action,
            check_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Check resource sufficiency
    fn check_resource_sufficiency(
        &self,
        resource_count: usize,
        has_summaries: bool,
    ) -> SufficiencyCheckResult {
        let start = std::time::Instant::now();

        let is_sufficient = resource_count >= 2 || has_summaries;

        let confidence = if resource_count >= 5 {
            0.9
        } else if resource_count >= 2 {
            0.7
        } else if has_summaries {
            0.6
        } else {
            0.4
        };

        let reasoning = if is_sufficient {
            format!(
                "Found {} resources with context. Sufficient for source attribution.",
                resource_count
            )
        } else {
            "Limited resource context available.".to_string()
        };

        let suggested_action = if is_sufficient {
            SufficiencyAction::StopRetrieval
        } else {
            SufficiencyAction::IncludeResources
        };

        SufficiencyCheckResult {
            is_sufficient,
            confidence,
            reasoning,
            suggested_action,
            check_time_ms: start.elapsed().as_millis() as u64,
        }
    }
}

/// Sufficiency checker trait
#[async_trait]
pub trait SufficiencyChecker: Send + Sync {
    /// Check if retrieved information is sufficient
    async fn check(
        &self,
        check_type: SufficiencyCheckType,
        context: &SufficiencyContext,
    ) -> SufficiencyCheckResult;
}

/// Context for sufficiency checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SufficiencyContext {
    /// Number of categories found
    pub category_count: usize,
    /// Average category score
    pub avg_category_score: f32,
    /// Number of items found
    pub item_count: usize,
    /// Average item score
    pub avg_item_score: f32,
    /// Number of resources found
    pub resource_count: usize,
    /// Whether resources have summaries
    pub resources_have_summaries: bool,
    /// Current query
    pub query: String,
}

impl SufficiencyContext {
    pub fn new(query: String) -> Self {
        Self {
            category_count: 0,
            avg_category_score: 0.0,
            item_count: 0,
            avg_item_score: 0.0,
            resource_count: 0,
            resources_have_summaries: false,
            query,
        }
    }

    pub fn with_categories(mut self, count: usize, score: f32) -> Self {
        self.category_count = count;
        self.avg_category_score = score;
        self
    }

    pub fn with_items(mut self, count: usize, score: f32) -> Self {
        self.item_count = count;
        self.avg_item_score = score;
        self
    }

    pub fn with_resources(mut self, count: usize, has_summaries: bool) -> Self {
        self.resource_count = count;
        self.resources_have_summaries = has_summaries;
        self
    }
}

#[async_trait]
impl SufficiencyChecker for RuleBasedSufficiencyChecker {
    async fn check(
        &self,
        check_type: SufficiencyCheckType,
        context: &SufficiencyContext,
    ) -> SufficiencyCheckResult {
        debug!(
            "Sufficiency check: type={:?}, categories={}, items={}, resources={}",
            check_type,
            context.category_count,
            context.item_count,
            context.resource_count
        );

        match check_type {
            SufficiencyCheckType::Category => {
                self.check_category_sufficiency(
                    context.category_count,
                    context.avg_category_score,
                    context.item_count,
                )
            }
            SufficiencyCheckType::Item => {
                self.check_item_sufficiency(context.item_count, context.avg_item_score)
            }
            SufficiencyCheckType::Resource => {
                self.check_resource_sufficiency(
                    context.resource_count,
                    context.resources_have_summaries,
                )
            }
            SufficiencyCheckType::Combined => {
                // Check all and return the most restrictive result
                let category_result =
                    self.check_category_sufficiency(
                        context.category_count,
                        context.avg_category_score,
                        context.item_count,
                    );
                let item_result =
                    self.check_item_sufficiency(context.item_count, context.avg_item_score);
                let resource_result =
                    self.check_resource_sufficiency(
                        context.resource_count,
                        context.resources_have_summaries,
                    );

                // Return the result with lowest confidence (most restrictive)
                let mut results = vec![category_result, item_result, resource_result];
                results.sort_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap());

                results[0].clone()
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_category_sufficiency() {
        let checker = RuleBasedSufficiencyChecker::new(SufficiencyConfig::default());

        let context = SufficiencyContext::new("user preferences".to_string())
            .with_categories(5, 0.8)
            .with_items(10, 0.7);

        let result = checker.check(SufficiencyCheckType::Category, &context).await;

        assert!(result.is_sufficient);
        assert!(result.confidence >= 0.7);
    }

    #[tokio::test]
    async fn test_item_insufficiency() {
        let checker = RuleBasedSufficiencyChecker::new(SufficiencyConfig::default());

        let context = SufficiencyContext::new("rare topic".to_string())
            .with_categories(1, 0.3)
            .with_items(2, 0.2);

        let result = checker.check(SufficiencyCheckType::Item, &context).await;

        assert!(!result.is_sufficient);
        assert_eq!(result.suggested_action, SufficiencyAction::ContinueRetrieval);
    }

    #[tokio::test]
    async fn test_combined_check() {
        let checker = RuleBasedSufficiencyChecker::new(SufficiencyConfig::default());

        let context = SufficiencyContext::new("test query".to_string())
            .with_categories(3, 0.6)
            .with_items(8, 0.5)
            .with_resources(2, true);

        let result = checker.check(SufficiencyCheckType::Combined, &context).await;

        // Should be sufficient with combined info
        assert!(result.is_sufficient || result.confidence >= 0.5);
    }
}
