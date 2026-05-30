//!
//! Search Analytics routes
//!
//! Provides endpoints for:
//! - POST /api/v1/analytics/search/record - Record a search event
//! - GET /api/v1/analytics/search/report - Get analytics report
//! - GET /api/v1/analytics/search/patterns - Get query patterns
//! - GET /api/v1/analytics/search/distribution - Get result distribution
//!

use crate::error::ServerResult;
use agent_mem_core::search::search_analytics::{
    SearchAnalytics, SearchAnalyticsConfig, SearchEvent, SearchEventType,
};
use axum::{
    extract::{Extension, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use utoipa::ToSchema;

/// Application state for search analytics
#[derive(Clone)]
pub struct SearchAnalyticsState {
    pub analytics: Arc<SearchAnalytics>,
}

/// Record search event request
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordSearchRequest {
    /// Query text
    pub query: String,
    /// Number of results returned
    pub result_count: usize,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Average score of results
    pub avg_score: Option<f32>,
    /// Maximum score of results
    pub max_score: Option<f32>,
    /// Whether cache was hit
    pub cache_hit: Option<bool>,
    /// Event type
    pub event_type: Option<String>,
    /// First relevant result position (for MRR calculation)
    pub first_relevant_position: Option<u32>,
    /// Relevance scores for each result (for NDCG calculation)
    pub relevance_scores: Option<Vec<f32>>,
}

/// Record search event response
#[derive(Debug, Serialize, ToSchema)]
pub struct RecordSearchResponse {
    /// Event ID
    pub id: String,
    /// Whether recording was successful
    pub success: bool,
}

/// Analytics report response
#[derive(Debug, Serialize, ToSchema)]
pub struct AnalyticsReportResponse {
    /// Total searches
    pub total_searches: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Peak hour
    pub peak_hour: u32,
    /// Searches today
    pub searches_today: u64,
    /// Searches this week
    pub searches_this_week: u64,
    /// Searches this month
    pub searches_this_month: u64,
}

/// Query patterns response
#[derive(Debug, Serialize, ToSchema)]
pub struct QueryPatternsResponse {
    /// Top query patterns
    pub patterns: Vec<QueryPattern>,
}

/// Query pattern item
#[derive(Debug, Serialize, ToSchema)]
pub struct QueryPattern {
    /// Pattern text
    pub pattern: String,
    /// Occurrence count
    pub count: u64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
}

/// Result distribution response
#[derive(Debug, Serialize, ToSchema)]
pub struct ResultDistributionResponse {
    /// High score results (>0.8)
    pub high_score_results: u64,
    /// Medium score results (0.5-0.8)
    pub medium_score_results: u64,
    /// Low score results (0.3-0.5)
    pub low_score_results: u64,
    /// Zero result searches
    pub zero_result_searches: u64,
}

/// Record a search event
#[utoipa::path(
    post,
    path = "/api/v1/analytics/search/record",
    tag = "search-analytics",
    request_body = RecordSearchRequest,
    responses(
        (status = 200, description = "Event recorded", body = RecordSearchResponse),
        (status = 500, description = "Failed to record event")
    )
)]
pub async fn record_search(
    State(state): State<SearchAnalyticsState>,
    Extension(user_id): Extension<String>,
    params: Json<RecordSearchRequest>,
) -> ServerResult<Json<RecordSearchResponse>> {
    info!("📊 [SearchAnalytics] Recording search event from user: {}", user_id);

    let event_type = match params.event_type.as_deref() {
        Some("reranked") => SearchEventType::RerankedSearch,
        Some("batch") => SearchEventType::BatchSearch,
        Some("adaptive") => SearchEventType::AdaptiveSearch,
        _ => SearchEventType::Search,
    };

    let event = SearchEvent {
        id: format!("evt_{}", uuid::Uuid::new_v4()),
        event_type,
        query: params.query.clone(),
        query_length: params.query.len(),
        result_count: params.result_count,
        response_time_ms: params.response_time_ms,
        avg_score: params.avg_score.unwrap_or(0.0),
        max_score: params.max_score.unwrap_or(0.0),
        cache_hit: params.cache_hit.unwrap_or(false),
        timestamp: chrono::Utc::now(),
        first_relevant_position: params.first_relevant_position,
        relevance_scores: params.relevance_scores.clone(),
    };

    let event_id = event.id.clone();
    state.analytics.record_search(event).await;

    Ok(Json(RecordSearchResponse {
        id: event_id,
        success: true,
    }))
}

/// Get analytics report
#[utoipa::path(
    get,
    path = "/api/v1/analytics/search/report",
    tag = "search-analytics",
    responses(
        (status = 200, description = "Report retrieved", body = AnalyticsReportResponse),
        (status = 500, description = "Failed to get report")
    )
)]
pub async fn get_report(
    State(state): State<SearchAnalyticsState>,
) -> ServerResult<Json<AnalyticsReportResponse>> {
    info!("📊 [SearchAnalytics] Getting analytics report");

    let report = state.analytics.get_report().await;

    Ok(Json(AnalyticsReportResponse {
        total_searches: report.performance.total_searches,
        cache_hit_rate: report.cache_hit_rate,
        avg_response_time_ms: report.avg_response_time_ms,
        peak_hour: report.peak_search_hour.unwrap_or(0),
        searches_today: 0, // Would need hourly tracking
        searches_this_week: 0,
        searches_this_month: report.performance.total_searches,
    }))
}

/// Get query patterns
#[utoipa::path(
    get,
    path = "/api/v1/analytics/search/patterns",
    tag = "search-analytics",
    responses(
        (status = 200, description = "Patterns retrieved", body = QueryPatternsResponse),
        (status = 500, description = "Failed to get patterns")
    )
)]
pub async fn get_patterns(
    State(state): State<SearchAnalyticsState>,
) -> ServerResult<Json<QueryPatternsResponse>> {
    info!("📊 [SearchAnalytics] Getting query patterns");

    let patterns = state.analytics.get_query_patterns().await;

    // Convert QueryPatternStats to QueryPatternsResponse
    let pattern_list = vec![
        QueryPattern {
            pattern: "short (1-3 words)".to_string(),
            count: patterns.short_queries,
            avg_latency_ms: patterns.avg_query_length * 10.0,
        },
        QueryPattern {
            pattern: "medium (4-10 words)".to_string(),
            count: patterns.medium_queries,
            avg_latency_ms: patterns.avg_query_length * 10.0,
        },
        QueryPattern {
            pattern: "long (10+ words)".to_string(),
            count: patterns.long_queries,
            avg_latency_ms: patterns.avg_query_length * 10.0,
        },
    ];

    Ok(Json(QueryPatternsResponse {
        patterns: pattern_list,
    }))
}

/// Get result distribution
#[utoipa::path(
    get,
    path = "/api/v1/analytics/search/distribution",
    tag = "search-analytics",
    responses(
        (status = 200, description = "Distribution retrieved", body = ResultDistributionResponse),
        (status = 500, description = "Failed to get distribution")
    )
)]
pub async fn get_distribution(
    State(state): State<SearchAnalyticsState>,
) -> ServerResult<Json<ResultDistributionResponse>> {
    info!("📊 [SearchAnalytics] Getting result distribution");

    let distribution = state.analytics.get_result_distribution().await;

    Ok(Json(ResultDistributionResponse {
        high_score_results: distribution.high_score_results,
        medium_score_results: distribution.medium_score_results,
        low_score_results: distribution.low_score_results,
        zero_result_searches: distribution.zero_result_searches,
    }))
}

/// Create the search analytics router
pub fn create_search_analytics_router(state: SearchAnalyticsState) -> Router {
    Router::new()
        .route("/record", post(record_search))
        .route("/report", get(get_report))
        .route("/patterns", get(get_patterns))
        .route("/distribution", get(get_distribution))
        .with_state(state)
}
