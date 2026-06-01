//! Generate summaries executor
//!
//! Generates summaries for categories from the semantic memories stored beneath them.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use tracing::{info, warn};

use agent_mem_category::{Category, CategoryScope};
use agent_mem_traits::{SemanticMemoryItem, SemanticQuery};

use crate::error::{ProactiveError, Result};
use crate::executors::{smart_summarize, SharedCategoryManager, SharedSemanticStore};
use crate::models::{ProactiveTask, TaskExecutionContext, TaskResult};
use crate::scheduler::TaskExecutor;

/// Generate summaries executor
///
/// Generates category summaries:
/// - Finds categories that are stale or explicitly requested
/// - Pulls memories under the category tree path
/// - Builds a concise summary from recent items
/// - Writes the summary back to the category manager
pub struct GenerateSummariesExecutor {
    /// Maximum categories to process per run
    batch_size: u32,
    /// Whether to only update stale categories
    stale_only: bool,
    /// Stale threshold in days
    stale_threshold_days: u32,
    /// Maximum items to include in summary context
    max_context_items: u32,
    /// Semantic store for fetching category memories
    semantic_store: Option<SharedSemanticStore>,
    /// Category manager for listing and updating categories
    category_manager: Option<SharedCategoryManager>,
}

impl GenerateSummariesExecutor {
    /// Create a new generate summaries executor
    pub fn new() -> Self {
        Self {
            batch_size: 10,
            stale_only: true,
            stale_threshold_days: 7,
            max_context_items: 50,
            semantic_store: None,
            category_manager: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(batch_size: u32, stale_only: bool, stale_threshold_days: u32) -> Self {
        Self {
            batch_size,
            stale_only,
            stale_threshold_days,
            max_context_items: 50,
            semantic_store: None,
            category_manager: None,
        }
    }

    /// Attach a semantic store.
    pub fn with_semantic_store(mut self, semantic_store: SharedSemanticStore) -> Self {
        self.semantic_store = Some(semantic_store);
        self
    }

    /// Attach a category manager.
    pub fn with_category_manager(mut self, category_manager: SharedCategoryManager) -> Self {
        self.category_manager = Some(category_manager);
        self
    }

    /// Execute summary generation.
    async fn perform_summary_generation(
        &self,
        context: &TaskExecutionContext,
        stale_only: bool,
    ) -> Result<TaskResult> {
        let started_at = Utc::now();
        let task_id = format!("generate-summaries-{}", started_at.timestamp());

        info!(
            "Starting summary generation (batch_size: {}, stale_only: {})",
            self.batch_size, stale_only
        );

        let Some(semantic_store) = &self.semantic_store else {
            warn!("Generate-summaries executor has no semantic store configured; skipping");
            let mut result = TaskResult::new(task_id, ProactiveTask::GenerateSummaries, started_at);
            result.completed(0, 0);
            return Ok(result);
        };

        let Some(category_manager) = &self.category_manager else {
            warn!("Generate-summaries executor has no category manager configured; skipping");
            let mut result = TaskResult::new(task_id, ProactiveTask::GenerateSummaries, started_at);
            result.completed(0, 0);
            return Ok(result);
        };

        let scope = category_scope(context);
        let categories = {
            let manager = category_manager.lock().await;
            manager
                .list_categories(&scope)
                .await
                .map_err(|err| ProactiveError::CategoryError(err.to_string()))?
        };

        let selected_categories = context.config.categories.clone();
        let stale_before = Utc::now() - Duration::days(self.stale_threshold_days as i64);

        let candidates: Vec<_> = categories
            .into_iter()
            .filter(|category| {
                should_process_category(
                    category,
                    selected_categories.as_deref(),
                    stale_only,
                    stale_before,
                )
            })
            .take(self.batch_size as usize)
            .collect();

        let mut categories_processed = 0u64;
        let mut summaries_generated = 0u64;

        for category in candidates {
            categories_processed += 1;
            let segments = parse_category_path(&category.path);
            let items = semantic_store
                .query_items(
                    &context.user_id,
                    SemanticQuery {
                        tree_path_prefix: Some(segments),
                        limit: Some(self.max_context_items as i64),
                        ..Default::default()
                    },
                )
                .await
                .map_err(|err| ProactiveError::StorageError(err.to_string()))?;

            if items.is_empty() {
                continue;
            }

            let summary = build_summary(&category.path, &items, self.max_context_items);
            if category.summary.as_deref() == Some(summary.as_str()) {
                continue;
            }

            let mut manager = category_manager.lock().await;
            manager
                .update_summary(&category.id, summary)
                .await
                .map_err(|err| ProactiveError::CategoryError(err.to_string()))?;
            summaries_generated += 1;
        }

        let mut result = TaskResult::new(task_id, ProactiveTask::GenerateSummaries, started_at);
        result.completed(categories_processed, summaries_generated);

        info!(
            "Summary generation completed: {} categories processed, {} summaries updated",
            categories_processed, summaries_generated
        );

        Ok(result)
    }
}

impl Default for GenerateSummariesExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskExecutor for GenerateSummariesExecutor {
    fn task_type(&self) -> ProactiveTask {
        ProactiveTask::GenerateSummaries
    }

    async fn execute(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        let stale_only = context
            .config
            .stale_categories_only
            .unwrap_or(self.stale_only);

        if context.dry_run {
            let started_at = Utc::now();
            let task_id = format!("generate-summaries-dry-{}", started_at.timestamp());
            let mut result = TaskResult::new(task_id, ProactiveTask::GenerateSummaries, started_at);
            result.completed(0, 0);
            return Ok(result);
        }

        self.perform_summary_generation(context, stale_only).await
    }
}

fn category_scope(context: &TaskExecutionContext) -> CategoryScope {
    match &context.agent_id {
        Some(agent_id) => CategoryScope::with_agent(context.user_id.clone(), agent_id.clone()),
        None => CategoryScope::new(context.user_id.clone()),
    }
}

fn should_process_category(
    category: &Category,
    selected_categories: Option<&[String]>,
    stale_only: bool,
    stale_before: chrono::DateTime<Utc>,
) -> bool {
    if let Some(selected_categories) = selected_categories {
        return selected_categories
            .iter()
            .any(|path| path == &category.path);
    }

    if !stale_only {
        return true;
    }

    category.summary.is_none() || category.updated_at <= stale_before
}

fn parse_category_path(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect()
}

fn build_summary(
    category_path: &str,
    items: &[SemanticMemoryItem],
    max_context_items: u32,
) -> String {
    let mut recent_items = items.to_vec();
    recent_items.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    let combined = recent_items
        .into_iter()
        .take(max_context_items as usize)
        .map(|item| {
            let details = item.details.unwrap_or_default();
            smart_summarize(&format!("{}: {} {}", item.name, item.summary, details), 120)
        })
        .collect::<Vec<_>>()
        .join(" | ");

    let combined_summary = smart_summarize(&combined, 240);
    format!(
        "{} memories summarized for {}. {}",
        items.len(),
        category_path,
        combined_summary
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use agent_mem_category::{CategoryManager, InMemoryCategoryManager};
    use agent_mem_traits::SemanticMemoryStore;

    use crate::executors::{
        shared_category_manager, shared_semantic_store,
        test_support::{semantic_item, MockSemanticStore},
    };
    use crate::models::TaskConfig;
    use crate::models::TaskStatus;

    fn context(user_id: &str) -> TaskExecutionContext {
        TaskExecutionContext {
            user_id: user_id.to_string(),
            agent_id: None,
            config: Default::default(),
            max_cpu_percent: 5,
            max_memory_mb: 512,
            dry_run: false,
        }
    }

    #[tokio::test]
    async fn test_generate_summaries_executor_without_integrations_is_noop() {
        let executor = GenerateSummariesExecutor::new();
        let result = executor.execute(&context("system")).await.unwrap();

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
        assert_eq!(result.items_affected, 0);
    }

    #[tokio::test]
    async fn test_generate_summaries_updates_category_summary() {
        let store = MockSemanticStore::new();
        store
            .create_item(semantic_item(
                "item-1",
                "user-123",
                "Rust ownership",
                "Ownership rules keep memory safe",
                Some("Borrow checker and move semantics"),
                vec!["knowledge", "rust"],
            ))
            .await
            .unwrap();
        store
            .create_item(semantic_item(
                "item-2",
                "user-123",
                "Rust traits",
                "Traits enable polymorphism",
                Some("Blanket impls and trait bounds"),
                vec!["knowledge", "rust"],
            ))
            .await
            .unwrap();

        let mut manager = InMemoryCategoryManager::new();
        let scope = CategoryScope::new("user-123".to_string());
        manager
            .create_category("/knowledge/rust", scope.clone())
            .await
            .unwrap();

        let shared_manager = shared_category_manager(manager);
        let executor = GenerateSummariesExecutor::new()
            .with_semantic_store(shared_semantic_store(store))
            .with_category_manager(shared_manager.clone());

        let mut execution_context = context("user-123");
        execution_context.config = TaskConfig {
            categories: Some(vec!["/knowledge/rust".to_string()]),
            ..Default::default()
        };

        let result = executor.execute(&execution_context).await.unwrap();
        assert_eq!(result.items_processed, 1);
        assert_eq!(result.items_affected, 1);

        let manager = shared_manager.lock().await;
        let category = manager
            .get_category_by_path("/knowledge/rust", &scope)
            .await
            .unwrap();
        let summary = category.summary.unwrap();
        assert!(summary.contains("Rust"));
        assert!(summary.contains("/knowledge/rust"));
    }

    #[tokio::test]
    async fn test_generate_summaries_dry_run() {
        let executor = GenerateSummariesExecutor::new();
        let mut context = context("system");
        context.dry_run = true;

        let result = executor.execute(&context).await.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
    }

    #[test]
    fn test_generate_summaries_task_type() {
        let executor = GenerateSummariesExecutor::new();
        assert_eq!(executor.task_type(), ProactiveTask::GenerateSummaries);
    }
}
