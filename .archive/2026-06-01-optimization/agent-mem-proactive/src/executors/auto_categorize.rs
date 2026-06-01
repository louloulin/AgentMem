//! Auto-categorize executor
//!
//! Automatically categorizes new semantic memories into hierarchical categories.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Map, Value};
use tracing::{info, warn};

use agent_mem_category::CategoryScope;
use agent_mem_traits::{SemanticMemoryItem, SemanticQuery};

use crate::error::{ProactiveError, Result};
use crate::executors::{SharedCategoryManager, SharedSemanticStore};
use crate::models::{ProactiveTask, TaskExecutionContext, TaskResult};
use crate::scheduler::TaskExecutor;

/// Auto-categorize executor
///
/// Automatically categorizes uncategorized semantic memory items:
/// - Scans for items without tree path assignments
/// - Chooses a category from config or simple heuristics
/// - Creates category hierarchy on demand
/// - Writes the inferred tree path back to the semantic store
pub struct AutoCategorizeExecutor {
    /// Maximum items to process per run
    batch_size: u32,
    /// Similarity threshold for configured category matching
    similarity_threshold: f32,
    /// Whether to create new categories if needed
    create_new_categories: bool,
    /// Semantic store used for reading and updating memories
    semantic_store: Option<SharedSemanticStore>,
    /// Category manager used to keep the category tree in sync
    category_manager: Option<SharedCategoryManager>,
}

impl AutoCategorizeExecutor {
    /// Create a new auto-categorize executor
    pub fn new() -> Self {
        Self {
            batch_size: 100,
            similarity_threshold: 0.8,
            create_new_categories: true,
            semantic_store: None,
            category_manager: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(batch_size: u32, similarity_threshold: f32) -> Self {
        Self {
            batch_size,
            similarity_threshold,
            create_new_categories: true,
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

    /// Control whether missing categories should be created automatically.
    pub fn with_category_creation(mut self, create_new_categories: bool) -> Self {
        self.create_new_categories = create_new_categories;
        self
    }

    /// Execute auto-categorization.
    async fn perform_categorization(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        let started_at = Utc::now();
        let task_id = format!("auto-categorize-{}", started_at.timestamp());

        info!(
            "Starting auto-categorization (batch_size: {}, threshold: {})",
            self.batch_size, self.similarity_threshold
        );

        let Some(semantic_store) = &self.semantic_store else {
            warn!("Auto-categorize executor has no semantic store configured; skipping");
            let mut result = TaskResult::new(task_id, ProactiveTask::AutoCategorize, started_at);
            result.completed(0, 0);
            return Ok(result);
        };

        let Some(category_manager) = &self.category_manager else {
            warn!("Auto-categorize executor has no category manager configured; skipping");
            let mut result = TaskResult::new(task_id, ProactiveTask::AutoCategorize, started_at);
            result.completed(0, 0);
            return Ok(result);
        };

        let query = SemanticQuery {
            limit: Some((self.batch_size.saturating_mul(5)) as i64),
            ..Default::default()
        };
        let items = semantic_store
            .query_items(&context.user_id, query)
            .await
            .map_err(|err| ProactiveError::StorageError(err.to_string()))?;

        let mut items_processed = 0u64;
        let mut items_categorized = 0u64;

        for item in items
            .into_iter()
            .filter(|item| item.tree_path.is_empty())
            .take(self.batch_size as usize)
        {
            items_processed += 1;

            let segments = self.infer_category_segments(&item, context);
            if segments.is_empty() {
                continue;
            }

            if !self
                .ensure_category_hierarchy(category_manager, context, &segments)
                .await?
            {
                continue;
            }

            let mut updated_item = item;
            updated_item.tree_path = segments.clone();
            updated_item.updated_at = Utc::now();
            updated_item.metadata = annotate_category_metadata(updated_item.metadata, &segments);

            let updated = semantic_store
                .update_item(updated_item)
                .await
                .map_err(|err| ProactiveError::StorageError(err.to_string()))?;

            if updated {
                items_categorized += 1;
            }
        }

        let mut result = TaskResult::new(task_id, ProactiveTask::AutoCategorize, started_at);
        result.completed(items_processed, items_categorized);

        info!(
            "Auto-categorization completed: {} items processed, {} categorized",
            items_processed, items_categorized
        );

        Ok(result)
    }

    fn infer_category_segments(
        &self,
        item: &SemanticMemoryItem,
        context: &TaskExecutionContext,
    ) -> Vec<String> {
        let text = semantic_text(item);

        if let Some(categories) = context.config.categories.as_deref() {
            if let Some(selected) = self.select_configured_category(&text, categories) {
                return selected;
            }
        }

        heuristic_category_segments(item)
    }

    fn select_configured_category(&self, text: &str, categories: &[String]) -> Option<Vec<String>> {
        let text_tokens = tokens(text);
        let mut best_match: Option<(f32, Vec<String>)> = None;

        for category in categories {
            let segments = parse_category_path(category);
            if segments.is_empty() {
                continue;
            }

            let category_tokens: Vec<String> = segments
                .iter()
                .flat_map(|segment| tokens(segment))
                .collect();
            if category_tokens.is_empty() {
                continue;
            }

            let overlap = category_tokens
                .iter()
                .filter(|token| text_tokens.iter().any(|candidate| candidate == *token))
                .count();
            let score = overlap as f32 / category_tokens.len() as f32;

            if score >= self.similarity_threshold {
                match &best_match {
                    Some((best_score, _)) if score <= *best_score => {}
                    _ => best_match = Some((score, segments)),
                }
            }
        }

        best_match.map(|(_, segments)| segments).or_else(|| {
            categories
                .first()
                .map(|category| parse_category_path(category))
        })
    }

    async fn ensure_category_hierarchy(
        &self,
        category_manager: &SharedCategoryManager,
        context: &TaskExecutionContext,
        segments: &[String],
    ) -> Result<bool> {
        let scope = category_scope(context);
        let mut manager = category_manager.lock().await;

        for depth in 1..=segments.len() {
            let path = category_path(&segments[..depth]);
            let category = match manager.get_category_by_path(&path, &scope).await {
                Ok(category) => category,
                Err(_) if !self.create_new_categories => return Ok(false),
                Err(_) => manager
                    .create_category(&path, scope.clone())
                    .await
                    .map_err(|err| ProactiveError::CategoryError(err.to_string()))?,
            };

            manager
                .increment_item_count(&category.id)
                .await
                .map_err(|err| ProactiveError::CategoryError(err.to_string()))?;
        }

        Ok(true)
    }
}

impl Default for AutoCategorizeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskExecutor for AutoCategorizeExecutor {
    fn task_type(&self) -> ProactiveTask {
        ProactiveTask::AutoCategorize
    }

    async fn execute(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        if context.dry_run {
            let started_at = Utc::now();
            let task_id = format!("auto-categorize-dry-{}", started_at.timestamp());
            let mut result = TaskResult::new(task_id, ProactiveTask::AutoCategorize, started_at);
            result.completed(0, 0);
            return Ok(result);
        }

        self.perform_categorization(context).await
    }
}

fn category_scope(context: &TaskExecutionContext) -> CategoryScope {
    match &context.agent_id {
        Some(agent_id) => CategoryScope::with_agent(context.user_id.clone(), agent_id.clone()),
        None => CategoryScope::new(context.user_id.clone()),
    }
}

fn category_path(segments: &[String]) -> String {
    format!("/{}", segments.join("/"))
}

fn parse_category_path(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(slugify_segment)
        .collect()
}

fn semantic_text(item: &SemanticMemoryItem) -> String {
    format!(
        "{} {} {} {}",
        item.name,
        item.summary,
        item.details.clone().unwrap_or_default(),
        item.source.clone().unwrap_or_default()
    )
}

fn heuristic_category_segments(item: &SemanticMemoryItem) -> Vec<String> {
    let text = semantic_text(item).to_lowercase();
    let leaf = preferred_leaf(item);

    if contains_any(
        &text,
        &["deploy", "workflow", "pipeline", "runbook", "automation"],
    ) {
        return vec!["procedures".to_string(), "automation".to_string()];
    }

    if contains_any(
        &text,
        &[
            "preference",
            "preferences",
            "favorite",
            "style",
            "setting",
            "likes",
        ],
    ) {
        return vec!["preferences".to_string(), leaf];
    }

    if contains_any(
        &text,
        &[
            "rust",
            "python",
            "javascript",
            "typescript",
            "java",
            "go",
            "database",
            "api",
            "architecture",
            "knowledge",
            "programming",
            "language",
        ],
    ) || contains_any(&text, &["wikipedia", "docs", "documentation", "reference"])
    {
        return vec!["knowledge".to_string(), leaf];
    }

    vec!["general".to_string(), leaf]
}

fn preferred_leaf(item: &SemanticMemoryItem) -> String {
    let lower = semantic_text(item).to_lowercase();
    for keyword in [
        "rust",
        "python",
        "javascript",
        "typescript",
        "java",
        "go",
        "database",
        "api",
        "automation",
        "preferences",
    ] {
        if lower.contains(keyword) {
            return keyword.to_string();
        }
    }

    tokens(&item.name)
        .into_iter()
        .find(|token| !matches!(token.as_str(), "the" | "and" | "for" | "with"))
        .unwrap_or_else(|| "misc".to_string())
}

fn tokens(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect()
}

fn slugify_segment(input: &str) -> String {
    let slug = input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        "misc".to_string()
    } else {
        slug
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| text.contains(keyword))
}

fn annotate_category_metadata(metadata: Value, segments: &[String]) -> Value {
    let mut object = match metadata {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    object.insert(
        "proactive_category_path".to_string(),
        Value::String(category_path(segments)),
    );
    object.insert(
        "auto_categorized_by".to_string(),
        Value::String("agent-mem-proactive".to_string()),
    );
    object.insert(
        "auto_categorized_at".to_string(),
        Value::String(Utc::now().to_rfc3339()),
    );

    Value::Object(object)
}

#[cfg(test)]
mod tests {
    use super::*;

    use agent_mem_category::InMemoryCategoryManager;
    use agent_mem_traits::SemanticMemoryStore;

    use crate::executors::{
        shared_category_manager, shared_semantic_store,
        test_support::{semantic_item, MockSemanticStore},
    };
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
    async fn test_auto_categorize_executor_without_integrations_is_noop() {
        let executor = AutoCategorizeExecutor::new();
        let result = executor.execute(&context("system")).await.unwrap();

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
        assert_eq!(result.items_affected, 0);
    }

    #[tokio::test]
    async fn test_auto_categorize_assigns_tree_path_and_updates_categories() {
        let store = MockSemanticStore::new();
        store
            .create_item(semantic_item(
                "item-1",
                "user-123",
                "Rust",
                "Systems programming language",
                Some("Ownership and fearless concurrency"),
                vec![],
            ))
            .await
            .unwrap();

        let shared_store = shared_semantic_store(store.clone());
        let shared_manager = shared_category_manager(InMemoryCategoryManager::new());
        let executor = AutoCategorizeExecutor::new()
            .with_semantic_store(shared_store)
            .with_category_manager(shared_manager.clone());

        let result = executor.execute(&context("user-123")).await.unwrap();
        assert_eq!(result.items_processed, 1);
        assert_eq!(result.items_affected, 1);

        let items = store.all_items_for_user("user-123").await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tree_path, vec!["knowledge", "rust"]);

        let scope = CategoryScope::new("user-123".to_string());
        let manager = shared_manager.lock().await;
        let category = manager
            .get_category_by_path("/knowledge/rust", &scope)
            .await
            .unwrap();
        assert_eq!(category.item_count, 1);
    }

    #[tokio::test]
    async fn test_auto_categorize_dry_run() {
        let executor = AutoCategorizeExecutor::new();
        let mut context = context("system");
        context.dry_run = true;

        let result = executor.execute(&context).await.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
    }

    #[test]
    fn test_auto_categorize_task_type() {
        let executor = AutoCategorizeExecutor::new();
        assert_eq!(executor.task_type(), ProactiveTask::AutoCategorize);
    }
}
