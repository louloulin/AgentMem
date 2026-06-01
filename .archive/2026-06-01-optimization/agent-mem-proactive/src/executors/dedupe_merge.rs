//! Dedupe-merge executor
//!
//! Detects and merges duplicate or similar semantic memory items.

use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use tracing::{info, warn};

use agent_mem_traits::{SemanticMemoryItem, SemanticQuery};

use crate::error::{ProactiveError, Result};
use crate::executors::{smart_summarize, SharedSemanticStore};
use crate::models::{ProactiveTask, TaskExecutionContext, TaskResult};
use crate::scheduler::TaskExecutor;

/// Dedupe-merge executor
///
/// Detects and merges duplicate memory items:
/// - Uses semantic-content similarity to group duplicates
/// - Applies a configurable similarity threshold
/// - Deletes redundant items from the semantic store
/// - Optionally writes merged content back to the surviving item
pub struct DedupeMergeExecutor {
    /// Similarity threshold (0.0-1.0)
    similarity_threshold: f32,
    /// Maximum items to process per run
    batch_size: u32,
    /// Merge strategy
    strategy: MergeStrategy,
    /// Semantic store used for read/update/delete operations
    semantic_store: Option<SharedSemanticStore>,
}

#[derive(Debug, Clone)]
enum MergeStrategy {
    /// Keep the newest item
    KeepNewest,
    /// Keep the oldest item
    KeepOldest,
    /// Merge content into the surviving item
    MergeContent,
}

impl DedupeMergeExecutor {
    /// Create a new dedupe-merge executor with default settings
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.9,
            batch_size: 1000,
            strategy: MergeStrategy::KeepNewest,
            semantic_store: None,
        }
    }

    /// Create with custom similarity threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            similarity_threshold: threshold,
            batch_size: 1000,
            strategy: MergeStrategy::KeepNewest,
            semantic_store: None,
        }
    }

    /// Attach a semantic store.
    pub fn with_semantic_store(mut self, semantic_store: SharedSemanticStore) -> Self {
        self.semantic_store = Some(semantic_store);
        self
    }

    /// Set merge strategy.
    pub fn with_strategy(mut self, strategy: &str) -> Self {
        self.strategy = match strategy {
            "keep_newest" => MergeStrategy::KeepNewest,
            "keep_oldest" => MergeStrategy::KeepOldest,
            "merge_content" => MergeStrategy::MergeContent,
            _ => MergeStrategy::KeepNewest,
        };
        self
    }

    /// Execute deduplication and merging.
    async fn perform_dedupe(
        &self,
        context: &TaskExecutionContext,
        threshold: f32,
    ) -> Result<TaskResult> {
        let started_at = Utc::now();
        let task_id = format!("dedupe-merge-{}", started_at.timestamp());

        info!(
            "Starting dedupe-merge (threshold: {}, batch_size: {})",
            threshold, self.batch_size
        );

        let Some(semantic_store) = &self.semantic_store else {
            warn!("Dedupe-merge executor has no semantic store configured; skipping");
            let mut result = TaskResult::new(task_id, ProactiveTask::DedupeMerge, started_at);
            result.completed(0, 0);
            return Ok(result);
        };

        let items = semantic_store
            .query_items(
                &context.user_id,
                SemanticQuery {
                    limit: Some(self.batch_size as i64),
                    ..Default::default()
                },
            )
            .await
            .map_err(|err| ProactiveError::StorageError(err.to_string()))?;

        let items_scanned = items.len() as u64;
        if items.len() < 2 {
            let mut result = TaskResult::new(task_id, ProactiveTask::DedupeMerge, started_at);
            result.completed(items_scanned, 0);
            return Ok(result);
        }

        let duplicate_groups = duplicate_groups(&items, threshold);
        let mut removed_ids = Vec::new();

        for group in duplicate_groups {
            let survivor = select_survivor(&group, &self.strategy);

            if matches!(self.strategy, MergeStrategy::MergeContent) {
                let updated_item = merged_semantic_item(&survivor, &group);
                let _ = semantic_store
                    .update_item(updated_item)
                    .await
                    .map_err(|err| ProactiveError::StorageError(err.to_string()))?;
            }

            for candidate in group {
                if candidate.id == survivor.id {
                    continue;
                }

                let _ = semantic_store
                    .delete_item(&candidate.id, &context.user_id)
                    .await
                    .map_err(|err| ProactiveError::StorageError(err.to_string()))?;
                removed_ids.push(candidate.id);
            }
        }

        let mut result = TaskResult::new(task_id, ProactiveTask::DedupeMerge, started_at);
        result.completed(items_scanned, removed_ids.len() as u64);

        info!(
            "Dedupe-merge completed: {} items scanned, {} duplicates removed",
            items_scanned,
            removed_ids.len()
        );

        Ok(result)
    }
}

impl Default for DedupeMergeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskExecutor for DedupeMergeExecutor {
    fn task_type(&self) -> ProactiveTask {
        ProactiveTask::DedupeMerge
    }

    async fn execute(&self, context: &TaskExecutionContext) -> Result<TaskResult> {
        let threshold = context
            .config
            .similarity_threshold
            .unwrap_or(self.similarity_threshold);

        if context.dry_run {
            let started_at = Utc::now();
            let task_id = format!("dedupe-merge-dry-{}", started_at.timestamp());
            let mut result = TaskResult::new(task_id, ProactiveTask::DedupeMerge, started_at);
            result.completed(0, 0);
            return Ok(result);
        }

        self.perform_dedupe(context, threshold).await
    }
}

fn duplicate_groups(items: &[SemanticMemoryItem], threshold: f32) -> Vec<Vec<SemanticMemoryItem>> {
    let mut groups = Vec::new();
    let mut visited = HashSet::new();

    for (index, item) in items.iter().enumerate() {
        if visited.contains(&item.id) {
            continue;
        }

        let mut group = vec![item.clone()];
        visited.insert(item.id.clone());

        for candidate in items.iter().skip(index + 1) {
            if visited.contains(&candidate.id) {
                continue;
            }

            if similarity(&semantic_content(item), &semantic_content(candidate)) >= threshold {
                visited.insert(candidate.id.clone());
                group.push(candidate.clone());
            }
        }

        if group.len() > 1 {
            groups.push(group);
        }
    }

    groups
}

fn select_survivor(group: &[SemanticMemoryItem], strategy: &MergeStrategy) -> SemanticMemoryItem {
    match strategy {
        MergeStrategy::KeepNewest | MergeStrategy::MergeContent => group
            .iter()
            .max_by_key(|item| item.created_at)
            .cloned()
            .expect("duplicate group must not be empty"),
        MergeStrategy::KeepOldest => group
            .iter()
            .min_by_key(|item| item.created_at)
            .cloned()
            .expect("duplicate group must not be empty"),
    }
}

fn merged_semantic_item(
    survivor: &SemanticMemoryItem,
    group: &[SemanticMemoryItem],
) -> SemanticMemoryItem {
    let mut updated = survivor.clone();
    let mut merged_content = Vec::new();

    for item in group {
        let content = semantic_content(item);
        if !merged_content.iter().any(|existing| existing == &content) {
            merged_content.push(content);
        }
    }

    let merged_content = merged_content.join("\n---\n");
    updated.summary = smart_summarize(&merged_content, 180);
    updated.details = Some(merged_content);
    updated.updated_at = Utc::now();
    updated
}

fn semantic_content(item: &SemanticMemoryItem) -> String {
    format!(
        "{}\n{}\n{}",
        item.name,
        item.summary,
        item.details.clone().unwrap_or_default()
    )
}

fn similarity(left: &str, right: &str) -> f32 {
    if left == right {
        return 1.0;
    }

    let len_diff = (left.len() as i32 - right.len() as i32).abs();
    if len_diff > 100 {
        return 0.0;
    }

    let min_len = left.len().min(right.len());
    if min_len == 0 {
        return 0.0;
    }

    let check_len = min_len.min(100);
    let left_chars: Vec<_> = left.chars().take(check_len).collect();
    let right_chars: Vec<_> = right.chars().take(check_len).collect();
    let matches = left_chars
        .iter()
        .zip(right_chars.iter())
        .filter(|(left, right)| left == right)
        .count();

    matches as f32 / check_len as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;

    use agent_mem_traits::SemanticMemoryStore;

    use crate::executors::{
        shared_semantic_store,
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
    async fn test_dedupe_merge_executor_without_store_is_noop() {
        let executor = DedupeMergeExecutor::new();
        let result = executor.execute(&context("system")).await.unwrap();

        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
        assert_eq!(result.items_affected, 0);
    }

    #[tokio::test]
    async fn test_dedupe_merge_removes_duplicate_semantic_items() {
        let store = MockSemanticStore::new();

        let mut older = semantic_item(
            "item-1",
            "user-123",
            "Rust",
            "A systems programming language",
            Some("Ownership and performance"),
            vec!["knowledge", "rust"],
        );
        older.created_at = Utc::now() - Duration::minutes(5);
        older.updated_at = older.created_at;

        let mut newer = semantic_item(
            "item-2",
            "user-123",
            "Rust",
            "A systems programming language",
            Some("Ownership and performance"),
            vec!["knowledge", "rust"],
        );
        newer.created_at = Utc::now();
        newer.updated_at = newer.created_at;

        store.create_item(older).await.unwrap();
        store.create_item(newer).await.unwrap();

        let executor =
            DedupeMergeExecutor::new().with_semantic_store(shared_semantic_store(store.clone()));

        let result = executor.execute(&context("user-123")).await.unwrap();
        assert_eq!(result.items_processed, 2);
        assert_eq!(result.items_affected, 1);

        let items = store.all_items_for_user("user-123").await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "item-2");
    }

    #[tokio::test]
    async fn test_dedupe_merge_dry_run() {
        let executor = DedupeMergeExecutor::new();
        let mut context = context("system");
        context.dry_run = true;

        let result = executor.execute(&context).await.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.items_processed, 0);
    }

    #[test]
    fn test_dedupe_merge_task_type() {
        let executor = DedupeMergeExecutor::new();
        assert_eq!(executor.task_type(), ProactiveTask::DedupeMerge);
    }
}
