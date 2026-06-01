//! Task executors for proactive tasks
//!
//! This module provides implementations of TaskExecutor for each proactive task type.

pub mod auto_categorize;
pub mod dedupe_merge;
pub mod generate_summaries;
pub mod health_check;
pub mod index_optimization;
pub mod resource_archival;

// Re-export all executors
pub use auto_categorize::AutoCategorizeExecutor;
pub use dedupe_merge::DedupeMergeExecutor;
pub use generate_summaries::GenerateSummariesExecutor;
pub use health_check::HealthCheckExecutor;
pub use index_optimization::IndexOptimizationExecutor;
pub use resource_archival::ResourceArchivalExecutor;

use std::sync::Arc;

use agent_mem_category::CategoryManager;
use agent_mem_traits::SemanticMemoryStore;
use tokio::sync::Mutex;

/// Shared semantic store handle for proactive maintenance tasks.
pub type SharedSemanticStore = Arc<dyn SemanticMemoryStore>;

/// Shared category manager handle for proactive maintenance tasks.
pub type SharedCategoryManager = Arc<Mutex<Box<dyn CategoryManager>>>;

/// Wrap a semantic store into a shared trait object.
pub fn shared_semantic_store<S>(store: S) -> SharedSemanticStore
where
    S: SemanticMemoryStore + 'static,
{
    Arc::new(store)
}

/// Wrap a category manager into a shared trait object.
pub fn shared_category_manager<M>(manager: M) -> SharedCategoryManager
where
    M: CategoryManager + 'static,
{
    Arc::new(Mutex::new(Box::new(manager)))
}

/// Smart truncation helper used by proactive summaries and merge previews.
pub(crate) fn smart_summarize(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        return content.to_string();
    }

    let head_len = (max_chars * 2) / 3;
    let tail_len = max_chars / 3;
    let omitted_chars = content.len().saturating_sub(head_len + tail_len);
    let marker = format!("...[omitted {omitted_chars} chars]...");

    let available = max_chars.saturating_sub(marker.len());
    let adjusted_head = (available * 2) / 3;
    let adjusted_tail = available / 3;

    let head = content
        .char_indices()
        .take_while(|(idx, _)| *idx < adjusted_head)
        .last()
        .map(|(idx, ch)| &content[..idx + ch.len_utf8()])
        .unwrap_or("");

    let tail_start = content.len().saturating_sub(adjusted_tail);
    let tail = content
        .char_indices()
        .skip_while(|(idx, _)| *idx < tail_start)
        .next()
        .map(|(idx, _)| &content[idx..])
        .unwrap_or("");

    format!("{head}{marker}{tail}")
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    use std::collections::HashMap;

    use agent_mem_traits::{
        Result as AgentMemResult, SemanticMemoryItem, SemanticMemoryStore, SemanticQuery,
    };
    use async_trait::async_trait;
    use chrono::Utc;
    use serde_json::json;

    #[derive(Clone, Default)]
    pub(crate) struct MockSemanticStore {
        items: Arc<Mutex<HashMap<String, SemanticMemoryItem>>>,
    }

    impl MockSemanticStore {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        fn key(user_id: &str, item_id: &str) -> String {
            format!("{user_id}:{item_id}")
        }

        pub(crate) async fn all_items_for_user(&self, user_id: &str) -> Vec<SemanticMemoryItem> {
            let items = self.items.lock().await;
            let mut values: Vec<_> = items
                .values()
                .filter(|item| item.user_id == user_id)
                .cloned()
                .collect();
            values.sort_by(|left, right| left.id.cmp(&right.id));
            values
        }
    }

    #[async_trait]
    impl SemanticMemoryStore for MockSemanticStore {
        async fn create_item(
            &self,
            item: SemanticMemoryItem,
        ) -> AgentMemResult<SemanticMemoryItem> {
            let key = Self::key(&item.user_id, &item.id);
            self.items.lock().await.insert(key, item.clone());
            Ok(item)
        }

        async fn get_item(
            &self,
            item_id: &str,
            user_id: &str,
        ) -> AgentMemResult<Option<SemanticMemoryItem>> {
            let key = Self::key(user_id, item_id);
            Ok(self.items.lock().await.get(&key).cloned())
        }

        async fn query_items(
            &self,
            user_id: &str,
            query: SemanticQuery,
        ) -> AgentMemResult<Vec<SemanticMemoryItem>> {
            let items = self.items.lock().await;
            let mut values: Vec<_> = items
                .values()
                .filter(|item| {
                    item.user_id == user_id
                        && query.name_query.as_ref().map_or(true, |query| {
                            let pattern = query.trim_matches('%');
                            item.name.contains(pattern)
                        })
                        && query.summary_query.as_ref().map_or(true, |query| {
                            let pattern = query.trim_matches('%');
                            item.summary.contains(pattern)
                        })
                        && query.tree_path_prefix.as_ref().map_or(true, |prefix| {
                            item.tree_path.len() >= prefix.len()
                                && item.tree_path[..prefix.len()] == prefix[..]
                        })
                })
                .cloned()
                .collect();

            values.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
            if let Some(limit) = query.limit {
                values.truncate(limit as usize);
            }

            Ok(values)
        }

        async fn update_item(&self, item: SemanticMemoryItem) -> AgentMemResult<bool> {
            let key = Self::key(&item.user_id, &item.id);
            let mut items = self.items.lock().await;
            if !items.contains_key(&key) {
                return Ok(false);
            }
            items.insert(key, item);
            Ok(true)
        }

        async fn delete_item(&self, item_id: &str, user_id: &str) -> AgentMemResult<bool> {
            let key = Self::key(user_id, item_id);
            Ok(self.items.lock().await.remove(&key).is_some())
        }

        async fn search_by_tree_path(
            &self,
            user_id: &str,
            tree_path: Vec<String>,
        ) -> AgentMemResult<Vec<SemanticMemoryItem>> {
            let items = self.items.lock().await;
            Ok(items
                .values()
                .filter(|item| item.user_id == user_id && item.tree_path == tree_path)
                .cloned()
                .collect())
        }

        async fn search_by_name(
            &self,
            user_id: &str,
            name_pattern: &str,
            limit: i64,
        ) -> AgentMemResult<Vec<SemanticMemoryItem>> {
            let items = self.items.lock().await;
            let mut values: Vec<_> = items
                .values()
                .filter(|item| item.user_id == user_id && item.name.contains(name_pattern))
                .cloned()
                .collect();
            values.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
            values.truncate(limit as usize);
            Ok(values)
        }
    }

    pub(crate) fn semantic_item(
        id: &str,
        user_id: &str,
        name: &str,
        summary: &str,
        details: Option<&str>,
        tree_path: Vec<&str>,
    ) -> SemanticMemoryItem {
        let now = Utc::now();
        SemanticMemoryItem {
            id: id.to_string(),
            organization_id: "org-test".to_string(),
            user_id: user_id.to_string(),
            agent_id: "agent-test".to_string(),
            name: name.to_string(),
            summary: summary.to_string(),
            details: details.map(str::to_string),
            source: Some("test-source".to_string()),
            tree_path: tree_path.into_iter().map(str::to_string).collect(),
            metadata: json!({
                "importance": 0.75
            }),
            created_at: now,
            updated_at: now,
        }
    }
}
