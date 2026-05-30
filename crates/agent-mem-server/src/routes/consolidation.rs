//! Memory Consolidation API
//!
//! Provides endpoints for memory consolidation (summarization and cleanup).
//!
//! 🔴 Phase 4: Consolidation API - AgentMem v2.0

use agent_mem_core::prompt::MemorySummarizer;
use agent_mem_core::storage::factory::Repositories;
use agent_mem_traits::{Content, MemoryV4 as Memory};
use axum::{
    extract::{Extension, Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use utoipa::{IntoParams, ToSchema};

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;

/// Consolidation state for managing summarization operations
#[derive(Clone)]
pub struct ConsolidationState {
    /// Default max chars per memory
    default_max_chars: usize,
}

impl ConsolidationState {
    /// Create new consolidation state
    pub fn new(max_chars: usize) -> Self {
        Self {
            default_max_chars: max_chars,
        }
    }

    /// Get default max chars
    pub fn default_max_chars(&self) -> usize {
        self.default_max_chars
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for consolidation
#[derive(Debug, Deserialize, ToSchema)]
pub struct ConsolidateRequest {
    /// Filter by agent_id
    pub agent_id: Option<String>,
    /// Filter by user_id
    pub user_id: Option<String>,
    /// Dry run mode (don't actually consolidate)
    pub dry_run: Option<bool>,
    /// Max chars per memory after summarization (optional, uses default)
    pub max_chars: Option<usize>,
}

/// Consolidation result
#[derive(Debug, Serialize, ToSchema)]
pub struct ConsolidationResult {
    /// Number of memories that were summarized
    pub summarized_count: i64,
    /// Number of memories deleted
    pub deleted_count: i64,
    /// Number of memories retained (no action needed)
    pub retained_count: i64,
    /// Total tokens saved (approximate)
    pub total_chars_saved: i64,
    /// Whether this was a dry run
    pub dry_run: bool,
    /// Per-memory details
    pub memories: Vec<MemoryConsolidationItem>,
}

/// Individual memory consolidation item
#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryConsolidationItem {
    /// Memory ID
    pub memory_id: String,
    /// Action taken: summarized, retained, protected, would_summarize, failed
    pub action: String,
    /// Original content size in characters
    pub original_size: usize,
    /// New content size after summarization (if applicable)
    pub new_size: Option<usize>,
    /// Reason for the action
    pub reason: String,
}

/// Query for summarizable memories
#[derive(Debug, Deserialize, IntoParams)]
pub struct SummarizableQuery {
    /// Filter by agent_id
    pub agent_id: Option<String>,
    /// Filter by user_id
    pub user_id: Option<String>,
    /// Minimum content size (in chars) to consider for summarization
    pub min_size: Option<usize>,
    /// Limit results
    pub limit: Option<i64>,
}

/// Response for summarizable memories
#[derive(Debug, Serialize, ToSchema)]
pub struct SummarizableResponse {
    /// Total count of summarizable memories
    pub count: i64,
    /// List of memories eligible for summarization
    pub memories: Vec<MemorySummary>,
}

/// Basic memory summary for listing
#[derive(Debug, Serialize, ToSchema)]
pub struct MemorySummary {
    /// Memory ID
    pub id: String,
    /// Content preview (first 200 chars)
    pub content_preview: String,
    /// Full content size in characters
    pub size: usize,
    /// Memory type (episodic, semantic, etc.)
    pub memory_type: Option<String>,
    /// Importance score
    pub importance: Option<f64>,
    /// Creation timestamp
    pub created_at: String,
}

// ============================================================================
// Helpers
// ============================================================================

/// Extract text content length from Memory
fn content_len(memory: &Memory) -> usize {
    match &memory.content {
        Content::Text(s) => s.len(),
        Content::Structured(v) => v.to_string().len(),
        Content::Vector(v) => v.len() * 4, // approximate bytes
        Content::Multimodal(parts) => parts.iter().map(|c| content_len_for_content(c)).sum(),
        Content::Binary(b) => b.len(),
    }
}

fn content_len_for_content(content: &Content) -> usize {
    match content {
        Content::Text(s) => s.len(),
        Content::Structured(v) => v.to_string().len(),
        Content::Vector(v) => v.len() * 4,
        Content::Multimodal(parts) => parts.iter().map(|c| content_len_for_content(c)).sum(),
        Content::Binary(b) => b.len(),
    }
}

/// Get text representation of memory content
fn content_as_text(memory: &Memory) -> String {
    match &memory.content {
        Content::Text(s) => s.clone(),
        Content::Structured(v) => v.to_string(),
        _ => memory.content.to_string(),
    }
}

/// Build a list of memories by filter
async fn list_filtered_memories(
    repositories: &Repositories,
    agent_id: Option<&str>,
    user_id: Option<&str>,
    limit: i64,
) -> ServerResult<Vec<Memory>> {
    if let Some(aid) = agent_id {
        repositories
            .memories
            .find_by_agent_id(aid, limit)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to list memories: {e}")))
    } else if let Some(uid) = user_id {
        repositories
            .memories
            .find_by_user_id(uid, limit)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to list memories: {e}")))
    } else {
        repositories
            .memories
            .list(limit, 0)
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to list memories: {e}")))
    }
}

// ============================================================================
// API Endpoints
// ============================================================================

/// GET /api/v1/memories/consolidate/summarizable
///
/// Get memories that can be summarized (content size > threshold)
#[utoipa::path(
    get,
    path = "/api/v1/memories/consolidate/summarizable",
    tag = "consolidation",
    params(
        SummarizableQuery
    ),
    responses(
        (status = 200, description = "Summarizable memories retrieved", body = SummarizableResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_summarizable_memories(
    Extension(state): Extension<Arc<ConsolidationState>>,
    Extension(repositories): Extension<Arc<Repositories>>,
    Query(query): Query<SummarizableQuery>,
) -> ServerResult<Json<SummarizableResponse>> {
    debug!(
        "Getting summarizable memories, agent_id={:?}, user_id={:?}",
        query.agent_id, query.user_id
    );

    let min_size = query.min_size.unwrap_or(state.default_max_chars() / 2);
    let limit = query.limit.unwrap_or(100);

    let memories = list_filtered_memories(
        &repositories,
        query.agent_id.as_deref(),
        query.user_id.as_deref(),
        limit,
    )
    .await?;

    // Filter memories that are candidates for summarization
    let summarizable: Vec<MemorySummary> = memories
        .into_iter()
        .filter(|m| content_len(m) >= min_size)
        .map(|m| {
            let text = content_as_text(&m);
            let preview_len = text.chars().count().min(200);
            let preview: String = text.chars().take(preview_len).collect();
            MemorySummary {
                id: m.id.to_string(),
                content_preview: if text.len() > 200 {
                    format!("{}...", preview)
                } else {
                    preview
                },
                size: text.len(),
                memory_type: m.memory_type(),
                importance: m.importance(),
                created_at: m.created_at().to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(SummarizableResponse {
        count: summarizable.len() as i64,
        memories: summarizable,
    }))
}

/// POST /api/v1/memories/consolidate
///
/// Trigger memory consolidation (summarization of long memories)
#[utoipa::path(
    post,
    path = "/api/v1/memories/consolidate",
    tag = "consolidation",
    request_body = ConsolidateRequest,
    responses(
        (status = 200, description = "Consolidation completed", body = ConsolidationResult),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn consolidate_memories(
    Extension(state): Extension<Arc<ConsolidationState>>,
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(request): Json<ConsolidateRequest>,
) -> ServerResult<Json<ConsolidationResult>> {
    let dry_run = request.dry_run.unwrap_or(false);
    let max_chars = request.max_chars.unwrap_or(state.default_max_chars());
    info!(
        "Consolidation triggered, dry_run={}, max_chars={}",
        dry_run, max_chars
    );

    let summarizer = MemorySummarizer::new(max_chars);

    let memories = list_filtered_memories(
        &repositories,
        request.agent_id.as_deref(),
        request.user_id.as_deref(),
        1000,
    )
    .await?;

    let mut items: Vec<MemoryConsolidationItem> = Vec::new();
    let mut summarized_count = 0i64;
    let mut retained_count = 0i64;
    let mut total_chars_saved = 0i64;

    for memory in memories {
        let original_size = content_len(&memory);
        let memory_id = memory.id.to_string();

        // Skip if memory is too short to summarize
        if original_size < max_chars {
            retained_count += 1;
            items.push(MemoryConsolidationItem {
                memory_id,
                action: "retained".to_string(),
                original_size,
                new_size: None,
                reason: "size_within_limit".to_string(),
            });
            continue;
        }

        // Skip if memory is protected (high importance > 0.8)
        if memory.importance().map(|i| i > 0.8).unwrap_or(false) {
            retained_count += 1;
            items.push(MemoryConsolidationItem {
                memory_id,
                action: "protected".to_string(),
                original_size,
                new_size: None,
                reason: "high_importance".to_string(),
            });
            continue;
        }

        // Summarize the text content
        let text = content_as_text(&memory);
        let new_text = summarizer.summarize(&text);
        let new_size = new_text.len();

        if dry_run {
            summarized_count += 1;
            total_chars_saved += (original_size.saturating_sub(new_size)) as i64;
            items.push(MemoryConsolidationItem {
                memory_id,
                action: "would_summarize".to_string(),
                original_size,
                new_size: Some(new_size),
                reason: format!(
                    "{:.1}% reduction",
                    (1.0 - new_size as f64 / original_size as f64) * 100.0
                ),
            });
        } else {
            // Update the memory with summarized content
            let mut updated = memory.clone();
            updated.content = Content::text(&new_text);
            updated.metadata.updated_at = chrono::Utc::now();

            match repositories.memories.update(&updated).await {
                Ok(updated_mem) => {
                    let final_size = content_len(&updated_mem);
                    summarized_count += 1;
                    total_chars_saved += (original_size.saturating_sub(final_size)) as i64;
                    items.push(MemoryConsolidationItem {
                        memory_id,
                        action: "summarized".to_string(),
                        original_size,
                        new_size: Some(final_size),
                        reason: format!(
                            "{:.1}% reduction",
                            (1.0 - final_size as f64 / original_size as f64) * 100.0
                        ),
                    });
                }
                Err(e) => {
                    warn!("Failed to update memory {}: {}", memory_id, e);
                    retained_count += 1;
                    items.push(MemoryConsolidationItem {
                        memory_id,
                        action: "failed".to_string(),
                        original_size,
                        new_size: None,
                        reason: format!("update_error: {}", e),
                    });
                }
            }
        }
    }

    info!(
        "Consolidation complete: summarized={}, retained={}",
        summarized_count, retained_count
    );

    Ok(Json(ConsolidationResult {
        summarized_count,
        deleted_count: 0,
        retained_count,
        total_chars_saved,
        dry_run,
        memories: items,
    }))
}

/// POST /api/v1/memories/consolidate/{memory_id}
///
/// Consolidate a single memory by its ID
#[utoipa::path(
    post,
    path = "/api/v1/memories/consolidate/{memory_id}",
    tag = "consolidation",
    params(
        ("memory_id" = String, Path, description = "Memory ID to consolidate")
    ),
    request_body = Option<ConsolidateRequest>,
    responses(
        (status = 200, description = "Memory consolidated", body = MemoryConsolidationItem),
        (status = 404, description = "Memory not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn consolidate_single_memory(
    Extension(state): Extension<Arc<ConsolidationState>>,
    Extension(repositories): Extension<Arc<Repositories>>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(memory_id): Path<String>,
    Json(request): Json<Option<ConsolidateRequest>>,
) -> ServerResult<Json<MemoryConsolidationItem>> {
    debug!("Consolidating single memory: {}", memory_id);

    let request = request.unwrap_or(ConsolidateRequest {
        agent_id: None,
        user_id: None,
        dry_run: Some(false),
        max_chars: None,
    });

    let max_chars = request.max_chars.unwrap_or(state.default_max_chars());
    let summarizer = MemorySummarizer::new(max_chars);

    // Get the memory
    let memory = repositories
        .memories
        .find_by_id(&memory_id)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to find memory: {e}")))?
        .ok_or_else(|| ServerError::not_found("Memory not found"))?;

    let original_size = content_len(&memory);

    // Check if summarization is needed
    if original_size < max_chars {
        return Ok(Json(MemoryConsolidationItem {
            memory_id,
            action: "retained".to_string(),
            original_size,
            new_size: None,
            reason: "size_within_limit".to_string(),
        }));
    }

    // Summarize
    let text = content_as_text(&memory);
    let new_text = summarizer.summarize(&text);
    let new_size = new_text.len();

    if request.dry_run.unwrap_or(false) {
        return Ok(Json(MemoryConsolidationItem {
            memory_id,
            action: "would_summarize".to_string(),
            original_size,
            new_size: Some(new_size),
            reason: format!(
                "{:.1}% reduction",
                (1.0 - new_size as f64 / original_size as f64) * 100.0
            ),
        }));
    }

    // Update the memory
    let mut updated = memory.clone();
    updated.content = Content::text(&new_text);
    updated.metadata.updated_at = chrono::Utc::now();

    let updated_mem = repositories
        .memories
        .update(&updated)
        .await
        .map_err(|e| ServerError::internal_error(format!("Failed to update memory: {e}")))?;

    let final_size = content_len(&updated_mem);

    Ok(Json(MemoryConsolidationItem {
        memory_id,
        action: "summarized".to_string(),
        original_size,
        new_size: Some(final_size),
        reason: format!(
            "{:.1}% reduction",
            (1.0 - final_size as f64 / original_size as f64) * 100.0
        ),
    }))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidation_state_creation() {
        let state = ConsolidationState::new(500);
        assert_eq!(state.default_max_chars(), 500);
    }

    #[test]
    fn test_summarizer_basic() {
        let summarizer = MemorySummarizer::new(100);
        let long_text = "a".repeat(200);
        let summary = summarizer.summarize(&long_text);
        assert!(summary.len() <= 120); // Includes ellipsis
    }

    #[test]
    fn test_consolidate_request_deserialization() {
        let json = r#"{"agent_id":"agent1","dry_run":true,"max_chars":200}"#;
        let request: ConsolidateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.agent_id, Some("agent1".to_string()));
        assert_eq!(request.dry_run, Some(true));
        assert_eq!(request.max_chars, Some(200));
    }

    #[test]
    fn test_consolidate_request_defaults() {
        let json = r#"{}"#;
        let request: ConsolidateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.agent_id, None);
        assert_eq!(request.dry_run, None);
        assert_eq!(request.max_chars, None);
    }

    #[test]
    fn test_consolidation_result_serialization() {
        let result = ConsolidationResult {
            summarized_count: 5,
            deleted_count: 0,
            retained_count: 10,
            total_chars_saved: 5000,
            dry_run: true,
            memories: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("summarized_count"));
        assert!(json.contains("dry_run"));
    }

    #[test]
    fn test_content_len_text() {
        use agent_mem_traits::MemoryId;
        let memory = Memory {
            id: MemoryId::new(),
            content: Content::text("hello world"),
            attributes: Default::default(),
            relations: Default::default(),
            metadata: Default::default(),
        };
        assert_eq!(content_len(&memory), 11);
    }
}