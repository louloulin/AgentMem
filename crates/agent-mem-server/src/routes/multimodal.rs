//!
//! Multimodal routes for image upload and search
//!
//! Provides endpoints for:
//! - POST /api/v1/multimodal/upload - Upload image for storage
//! - POST /api/v1/multimodal/search - Search for similar images
//! - GET /api/v1/multimodal/stats - Get multimodal storage statistics
//!

use crate::error::ServerResult;
use agent_mem_core::multimodal_storage::{
    ImageVectorizer, MockImageVectorizer, MultimodalStorage, MultimodalStorageConfig,
    MultimodalStorageBackend, MultimodalType,
};
use axum::{
    extract::{Extension, Query},
    response::Json,
    routing::{get, post},
    Router,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use utoipa::ToSchema;

/// Application state for multimodal routes
#[derive(Clone)]
pub struct MultimodalState {
    pub storage: Arc<MultimodalStorage>,
}

/// Upload request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UploadRequest {
    /// MIME type of the image (e.g., "image/jpeg")
    pub mime_type: Option<String>,
    /// Optional metadata tags
    pub tags: Option<Vec<String>>,
}

/// Upload response
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadResponse {
    /// The ID of the stored image
    pub id: String,
    /// URL to access the image
    pub url: String,
    /// MIME type
    pub mime_type: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// Timestamp
    pub created_at: String,
}

/// Search request
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchRequest {
    /// Image data (Base64 encoded)
    pub image_data: Option<String>,
    /// Existing image ID to search by
    pub image_id: Option<String>,
    /// Number of results to return
    pub limit: Option<usize>,
    /// Minimum similarity threshold
    pub threshold: Option<f32>,
}

/// Search response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    /// List of similar images
    pub results: Vec<SimilarImage>,
    /// Total number found
    pub total: usize,
}

/// Similar image result
#[derive(Debug, Serialize, ToSchema)]
pub struct SimilarImage {
    /// Image ID
    pub id: String,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f32,
    /// Preview URL
    pub preview_url: String,
    /// MIME type
    pub mime_type: String,
}

/// Multimodal statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct MultimodalStats {
    /// Total stored images
    pub total_images: usize,
    /// Total storage used (bytes)
    pub total_storage_bytes: usize,
    /// Average image size
    pub avg_size_bytes: f64,
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Status
    pub status: String,
    /// Storage stats
    pub storage: MultimodalStats,
}

/// Upload an image
#[utoipa::path(
    post,
    path = "/api/v1/multimodal/upload",
    tag = "multimodal",
    params(
        ("mime_type" = Option<String>, Query, description = "MIME type of the image"),
        ("tags" = Option<Vec<String>>, Query, description = "Optional tags")
    ),
    responses(
        (status = 200, description = "Image uploaded successfully", body = UploadResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Upload failed")
    )
)]
pub async fn upload_image(
    Extension(state): Extension<Arc<MultimodalState>>,
    Extension(user_id): Extension<String>,
    params: Query<UploadRequest>,
) -> ServerResult<Json<UploadResponse>> {
    info!("📤 [Multimodal] Upload request from user: {}", user_id);

    // Mock image data (in production, read from multipart form)
    let mock_data = vec![0u8; 1024]; // 1KB placeholder
    let mime_type = params.mime_type.clone().unwrap_or_else(|| "image/jpeg".to_string());
    
    // Create metadata
    let metadata = agent_mem_core::multimodal_storage::MultimodalMetadata {
        filename: None,
        size_bytes: Some(mock_data.len() as u64),
        width: None,
        height: None,
        duration_ms: None,
        tags: params.tags.clone().unwrap_or_default(),
        custom: std::collections::HashMap::new(),
    };
    
    // Store image
    let id = state.storage.store_image(mock_data.clone(), &mime_type, metadata).await
        .map_err(|e| {
            warn!("❌ [Multimodal] Store failed: {}", e);
            crate::error::ServerError::internal_error(format!("Failed to store image: {}", e))
        })?;
    
    info!("✅ [Multimodal] Image stored: {}", id);
    
    Ok(Json(UploadResponse {
        id: id.clone(),
        url: format!("/api/v1/multimodal/{}", id),
        mime_type,
        size_bytes: mock_data.len(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Search for similar images
#[utoipa::path(
    post,
    path = "/api/v1/multimodal/search",
    tag = "multimodal",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed", body = SearchResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Search failed")
    )
)]
pub async fn search_similar(
    Extension(state): Extension<Arc<MultimodalState>>,
    Extension(user_id): Extension<String>,
    params: Json<SearchRequest>,
) -> ServerResult<Json<SearchResponse>> {
    info!("🔍 [Multimodal] Search request from user: {}", user_id);
    
    let limit = params.limit.unwrap_or(10);
    
    let results = if let Some(ref image_id) = params.image_id {
        // Get the stored image and search by its embedding
        if let Ok(Some(memory)) = state.storage.get(image_id).await {
            if let Some(ref embedding) = memory.embedding {
                match state.storage.search_by_embedding(embedding, limit).await {
                    Ok(r) => r,
                    Err(_) => vec![],
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        warn!("⚠️ [Multimodal] No image_id provided");
        return Ok(Json(SearchResponse {
            results: vec![],
            total: 0,
        }));
    };
    
    // Convert to response format
    let similar_images: Vec<SimilarImage> = results.iter().map(|result| {
        SimilarImage {
            id: result.id.clone(),
            similarity: result.score,
            preview_url: format!("/api/v1/multimodal/{}", result.id),
            mime_type: "image/jpeg".to_string(),
        }
    }).collect();
    
    let total = similar_images.len();
    
    info!("✅ [Multimodal] Found {} similar images", total);
    
    Ok(Json(SearchResponse {
        results: similar_images,
        total,
    }))
}

/// Get multimodal statistics
#[utoipa::path(
    get,
    path = "/api/v1/multimodal/stats",
    tag = "multimodal",
    responses(
        (status = 200, description = "Stats retrieved", body = MultimodalStats),
        (status = 500, description = "Failed to get stats")
    )
)]
pub async fn get_stats(
    Extension(state): Extension<Arc<MultimodalState>>,
) -> ServerResult<Json<MultimodalStats>> {
    let stats = state.storage.get_stats().await;
    
    Ok(Json(MultimodalStats {
        total_images: stats.total_items as usize,
        total_storage_bytes: stats.total_size_bytes as usize,
        avg_size_bytes: if stats.total_items > 0 {
            stats.total_size_bytes as f64 / stats.total_items as f64
        } else {
            0.0
        },
    }))
}

/// Health check
#[utoipa::path(
    get,
    path = "/api/v1/multimodal/health",
    tag = "multimodal",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(
    Extension(state): Extension<Arc<MultimodalState>>,
) -> ServerResult<Json<HealthResponse>> {
    let stats = state.storage.get_stats().await;
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        storage: MultimodalStats {
            total_images: stats.total_items as usize,
            total_storage_bytes: stats.total_size_bytes as usize,
            avg_size_bytes: if stats.total_items > 0 {
                stats.total_size_bytes as f64 / stats.total_items as f64
            } else {
                0.0
            },
        },
    }))
}

/// Create the multimodal router
/// Note: MultimodalState is now passed via Extension layer, not with_state
/// This function is kept for backwards compatibility but is no longer used
pub fn create_multimodal_router(state: MultimodalState) -> Router {
    Router::new()
        .route("/upload", post(upload_image))
        .route("/search", post(search_similar))
        .route("/stats", get(get_stats))
        .route("/health", get(health_check))
        // State is passed via Extension layer instead
}
