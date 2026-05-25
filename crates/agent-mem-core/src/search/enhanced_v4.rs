//! Enhanced Search V4 Module
//!
//! Integrates all enhanced search features:
//! - Category recall
//! - Resource recall
//! - Sufficiency checking
//! - 7-stage retrieval process
//!
//! The 7-stage retrieval process:
//! 1. Route intention - determine query type
//! 2. Category recall - find relevant categories
//! 3. Sufficiency check - determine if more info needed
//! 4. Item recall - search memory items
//! 5. Resource recall - include source resources
//! 6. Sufficiency check - final check
//! 7. Build response - combine all results

use crate::search::category_recall::{
    CategoryFilter, CategoryRecallConfig, CategoryRecallEngine, CategoryRecallResult, CategoryScope,
    CategorySearchResult,
};
use crate::search::resource_recall::{ResourceRecallConfig, ResourceRecallEngine, ResourceRecallResult};
use crate::search::sufficiency_check::{
    SufficiencyAction, SufficiencyCheckResult, SufficiencyCheckType, SufficiencyChecker,
    SufficiencyConfig, SufficiencyContext,
};
use crate::search::SearchResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Enhanced search V4 result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchV4Result {
    /// Memory items
    pub items: Vec<SearchResult>,
    /// Categories found
    pub categories: Vec<CategorySearchResult>,
    /// Resources found
    pub resources: Vec<crate::search::resource_recall::ResourceContext>,
    /// Whether to continue retrieval
    pub should_continue: bool,
    /// Final sufficiency check result
    pub sufficiency: Option<SufficiencyCheckResult>,
    /// Search statistics
    pub stats: EnhancedSearchV4Stats,
    /// Error message if failed
    pub error: Option<String>,
}

/// Enhanced search V4 statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchV4Stats {
    /// Total search time in milliseconds
    pub total_time_ms: u64,
    /// Category recall time
    pub category_recall_time_ms: u64,
    /// Item recall time
    pub item_recall_time_ms: u64,
    /// Resource recall time
    pub resource_recall_time_ms: u64,
    /// Sufficiency check time
    pub sufficiency_check_time_ms: u64,
    /// Number of categories found
    pub category_count: usize,
    /// Number of items found
    pub item_count: usize,
    /// Number of resources found
    pub resource_count: usize,
}

/// Enhanced search V4 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSearchV4Config {
    /// Category recall config
    pub category_config: CategoryRecallConfig,
    /// Resource recall config
    pub resource_config: ResourceRecallConfig,
    /// Sufficiency config
    pub sufficiency_config: SufficiencyConfig,
    /// Maximum items to return
    pub max_items: usize,
    /// Enable early exit
    pub enable_early_exit: bool,
    /// Enable parallel execution
    pub enable_parallel: bool,
}

impl Default for EnhancedSearchV4Config {
    fn default() -> Self {
        Self {
            category_config: CategoryRecallConfig::default(),
            resource_config: ResourceRecallConfig::default(),
            sufficiency_config: SufficiencyConfig::default(),
            max_items: 20,
            enable_early_exit: true,
            enable_parallel: true,
        }
    }
}

/// Query type for routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnhancedQueryType {
    /// General knowledge query
    General,
    /// Preference query (user preferences)
    Preference,
    /// Skill query (user skills)
    Skill,
    /// Knowledge query (factual knowledge)
    Knowledge,
    /// Procedure query (how-to)
    Procedure,
    /// Context query (conversation context)
    Context,
}

/// Enhanced search V4 engine
pub struct EnhancedSearchV4 {
    category_engine: Arc<dyn CategoryRecallEngine>,
    resource_engine: Arc<dyn ResourceRecallEngine>,
    sufficiency_checker: Arc<dyn SufficiencyChecker>,
    config: EnhancedSearchV4Config,
}

impl EnhancedSearchV4 {
    pub fn new(
        category_engine: Arc<dyn CategoryRecallEngine>,
        resource_engine: Arc<dyn ResourceRecallEngine>,
        config: EnhancedSearchV4Config,
    ) -> Self {
        let sufficiency_checker = Arc::new(crate::search::sufficiency_check::RuleBasedSufficiencyChecker::new(
            config.sufficiency_config.clone(),
        ));

        Self {
            category_engine,
            resource_engine,
            sufficiency_checker,
            config,
        }
    }

    /// Route query to determine type
    fn route_intention(&self, query: &str) -> EnhancedQueryType {
        let query_lower = query.to_lowercase();

        // Simple keyword-based routing
        if query_lower.contains("偏好")
            || query_lower.contains("喜欢")
            || query_lower.contains("想要")
            || query_lower.contains("prefer")
            || query_lower.contains("like")
            || query_lower.contains("want")
        {
            EnhancedQueryType::Preference
        } else if query_lower.contains("技能")
            || query_lower.contains("能力")
            || query_lower.contains("擅长")
            || query_lower.contains("skill")
            || query_lower.contains("can")
            || query_lower.contains("good at")
        {
            EnhancedQueryType::Skill
        } else if query_lower.contains("知识")
            || query_lower.contains("知道")
            || query_lower.contains("什么是")
            || query_lower.contains("knowledge")
            || query_lower.contains("what is")
        {
            EnhancedQueryType::Knowledge
        } else if query_lower.contains("如何")
            || query_lower.contains("怎么做")
            || query_lower.contains("步骤")
            || query_lower.contains("how to")
            || query_lower.contains("procedure")
        {
            EnhancedQueryType::Procedure
        } else if query_lower.contains("对话")
            || query_lower.contains("之前")
            || query_lower.contains("context")
            || query_lower.contains("earlier")
        {
            EnhancedQueryType::Context
        } else {
            EnhancedQueryType::General
        }
    }

    /// Determine category paths based on query type
    fn get_category_paths_for_type(&self, query_type: &EnhancedQueryType) -> Vec<String> {
        match query_type {
            EnhancedQueryType::Preference => vec!["/preferences".to_string()],
            EnhancedQueryType::Skill => vec!["/skills".to_string()],
            EnhancedQueryType::Knowledge => vec!["/knowledge".to_string()],
            EnhancedQueryType::Procedure => vec!["/skills".to_string()],
            EnhancedQueryType::Context => vec!["/context".to_string()],
            EnhancedQueryType::General => vec![],
        }
    }

    /// Execute the 7-stage retrieval process
    pub async fn search(&self, query: &str, scope: &CategoryScope) -> EnhancedSearchV4Result {
        let total_start = std::time::Instant::now();
        info!("Starting enhanced search V4 for query: {}", query);

        // Stage 1: Route intention
        let query_type = self.route_intention(query);
        debug!("Query routed to type: {:?}", query_type);

        // Stage 2: Category recall
        let category_start = std::time::Instant::now();
        let category_result = self
            .category_engine
            .search_categories(query, scope, self.config.category_config.max_categories)
            .await
            .unwrap_or(CategoryRecallResult {
                categories: vec![],
                search_time_ms: 0,
                success: false,
                error: Some("Category search failed".to_string()),
            });
        let category_time_ms = category_start.elapsed().as_millis() as u64;

        debug!(
            "Category recall: found {} categories in {}ms",
            category_result.categories.len(),
            category_time_ms
        );

        // Stage 3: First sufficiency check (after category recall)
        let sufficiency_start = std::time::Instant::now();
        let initial_context = SufficiencyContext::new(query.to_string())
            .with_categories(category_result.categories.len(), 0.7)
            .with_items(0, 0.0)
            .with_resources(0, false);

        let initial_sufficiency = self
            .sufficiency_checker
            .check(SufficiencyCheckType::Category, &initial_context)
            .await;
        let sufficiency_check_1_time_ms = sufficiency_start.elapsed().as_millis() as u64;

        // Early exit if categories are sufficient
        if self.config.enable_early_exit
            && initial_sufficiency.is_sufficient
            && initial_sufficiency.suggested_action == SufficiencyAction::StopRetrieval
        {
            let total_time_ms = total_start.elapsed().as_millis() as u64;
            let categories = category_result.categories.clone();
            let category_count = categories.len();
            info!(
                "Early exit after category recall: sufficient={}, confidence={}",
                initial_sufficiency.is_sufficient, initial_sufficiency.confidence
            );

            return EnhancedSearchV4Result {
                items: vec![],
                categories,
                resources: vec![],
                should_continue: false,
                sufficiency: Some(initial_sufficiency),
                stats: EnhancedSearchV4Stats {
                    total_time_ms,
                    category_recall_time_ms: category_time_ms,
                    item_recall_time_ms: 0,
                    resource_recall_time_ms: 0,
                    sufficiency_check_time_ms: sufficiency_check_1_time_ms,
                    category_count,
                    item_count: 0,
                    resource_count: 0,
                },
                error: None,
            };
        }

        // Stage 4: Item recall (simulated - would integrate with existing search)
        let item_start = std::time::Instant::now();
        // In real implementation, this would call the existing search engine
        let items: Vec<SearchResult> = vec![]; // Placeholder for actual item recall
        let item_time_ms = item_start.elapsed().as_millis() as u64;

        // Stage 5: Resource recall
        let resource_start = std::time::Instant::now();
        let item_ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
        let resource_result = self
            .resource_engine
            .get_resources_for_items(&item_ids)
            .await
            .unwrap_or(ResourceRecallResult {
                resources: vec![],
                success: false,
                error: Some("Resource recall failed".to_string()),
                recall_time_ms: 0,
            });
        let resource_time_ms = resource_start.elapsed().as_millis() as u64;

        // Stage 6: Final sufficiency check
        let final_context = SufficiencyContext::new(query.to_string())
            .with_categories(category_result.categories.len(), 0.7)
            .with_items(items.len(), 0.6)
            .with_resources(
                resource_result.resources.len(),
                resource_result
                    .resources
                    .iter()
                    .any(|r| r.summary.is_some()),
            );

        let final_sufficiency = self
            .sufficiency_checker
            .check(SufficiencyCheckType::Combined, &final_context)
            .await;

        let total_time_ms = total_start.elapsed().as_millis() as u64;

        // Determine if should continue
        let should_continue = final_sufficiency.suggested_action != SufficiencyAction::StopRetrieval
            && self.config.enable_early_exit;

        // Clone values to avoid borrow issues
        let categories = category_result.categories.clone();
        let category_count = categories.len();
        let items_count = items.len();
        let resources = resource_result.resources.clone();
        let resource_count = resources.len();

        info!(
            "Enhanced search V4 complete: {} categories, {} items, {} resources, continue={}, time={}ms",
            category_count,
            items_count,
            resource_count,
            should_continue,
            total_time_ms
        );

        EnhancedSearchV4Result {
            items,
            categories,
            resources,
            should_continue,
            sufficiency: Some(final_sufficiency),
            stats: EnhancedSearchV4Stats {
                total_time_ms,
                category_recall_time_ms: category_time_ms,
                item_recall_time_ms: item_time_ms,
                resource_recall_time_ms: resource_time_ms,
                sufficiency_check_time_ms: sufficiency_check_1_time_ms,
                category_count,
                item_count: items_count,
                resource_count,
            },
            error: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "inline_tests")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_route_intention() {
        let config = EnhancedSearchV4Config::default();
        let category_engine: Arc<dyn CategoryRecallEngine> = Arc::new(
            crate::search::category_recall::InMemoryCategoryRecall::new(
                crate::search::category_recall::CategoryRecallConfig::default(),
            )
            .with_sample_data()
            .await,
        );
        let resource_engine: Arc<dyn ResourceRecallEngine> = Arc::new(
            crate::search::resource_recall::InMemoryResourceRecall::new(
                crate::search::resource_recall::ResourceRecallConfig::default(),
            )
            .with_sample_data()
            .await,
        );

        let engine = EnhancedSearchV4::new(category_engine, resource_engine, config);

        // Test preference query routing
        let query_type = engine.route_intention("用户偏好什么编程语言?");
        assert_eq!(query_type, EnhancedQueryType::Preference);

        // Test skill query routing
        let query_type = engine.route_intention("用户擅长什么技能?");
        assert_eq!(query_type, EnhancedQueryType::Skill);
    }

    #[tokio::test]
    async fn test_enhanced_search() {
        let config = EnhancedSearchV4Config::default();
        let category_engine: Arc<dyn CategoryRecallEngine> = Arc::new(
            crate::search::category_recall::InMemoryCategoryRecall::new(
                crate::search::category_recall::CategoryRecallConfig::default(),
            )
            .with_sample_data()
            .await,
        );
        let resource_engine: Arc<dyn ResourceRecallEngine> = Arc::new(
            crate::search::resource_recall::InMemoryResourceRecall::new(
                crate::search::resource_recall::ResourceRecallConfig::default(),
            )
            .with_sample_data()
            .await,
        );

        let engine = EnhancedSearchV4::new(category_engine, resource_engine, config);

        let scope = CategoryScope::new("user-123".to_string());
        let result = engine.search("用户沟通偏好", &scope).await;

        assert!(result.stats.total_time_ms > 0);
        assert!(result.stats.category_count > 0);
    }
}
