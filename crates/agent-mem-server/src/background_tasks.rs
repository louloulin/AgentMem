//! Background Tasks for AgentMem Server
//!
//! Handles automatic forgetting scheduler and other background tasks.
//!
//! 🔴 Phase 2: Forgetting Integration - AgentMem v2.0

use agent_mem_forgetting::{
    ForgettingConfig, ForgettingScheduler,
    protection::{MemoryProtection, ProtectionLevel},
};
use axum::{extract::{Extension, Query}, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use utoipa::{IntoParams, ToSchema};

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;

/// Forgetting background task state
#[derive(Clone)]
pub struct ForgettingState {
    pub scheduler: Arc<RwLock<Option<Arc<ForgettingScheduler>>>>,
    pub config: Arc<ForgettingConfig>,
    pub protection: Arc<MemoryProtection>,
}

impl ForgettingState {
    /// Create new forgetting state
    pub fn new(config: ForgettingConfig) -> Self {
        Self {
            scheduler: Arc::new(RwLock::new(None)),
            config: Arc::new(config),
            protection: Arc::new(MemoryProtection::new()),
        }
    }

    /// Initialize the scheduler
    pub async fn init(&self) -> ServerResult<()> {
        let scheduler = ForgettingScheduler::new(self.config.as_ref().clone())
            .await
            .map_err(|e| ServerError::internal_error(format!("Failed to create scheduler: {}", e)))?;

        let scheduler = Arc::new(scheduler);
        let mut scheduler_lock = self.scheduler.write().await;
        *scheduler_lock = Some(scheduler);

        info!("ForgettingScheduler initialized successfully");
        Ok(())
    }

    /// Start the automatic forgetting task
    pub async fn start(&self) -> ServerResult<()> {
        let scheduler = self.scheduler.read().await;
        if let Some(ref s) = *scheduler {
            s.start().await
                .map_err(|e| ServerError::internal_error(format!("Failed to start scheduler: {}", e)))?;
            info!("ForgettingScheduler started");
        }
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> ServerResult<()> {
        let scheduler = self.scheduler.read().await;
        if let Some(ref s) = *scheduler {
            s.stop().await
                .map_err(|e| ServerError::internal_error(format!("Failed to stop scheduler: {}", e)))?;
        }
        Ok(())
    }

    /// Get scheduler stats
    pub async fn get_stats(&self) -> ServerResult<ForgettingStatsResponse> {
        let scheduler = self.scheduler.read().await;
        if let Some(ref s) = *scheduler {
            let stats = s.stats().await;
            Ok(ForgettingStatsResponse {
                total_checks: stats.total_checks,
                total_forgotten: stats.total_forgotten,
                total_checked: stats.total_checked,
                total_protected: stats.total_protected,
                last_check_at: stats.last_check_at.map(|t| t.to_rfc3339()),
                next_check_at: stats.next_check_at.map(|t| t.to_rfc3339()),
                is_running: s.is_running().await,
            })
        } else {
            Err(ServerError::not_found("Scheduler not initialized"))
        }
    }

    /// Get memory protection manager
    pub fn protection(&self) -> &MemoryProtection {
        &self.protection
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Query parameters for health check
#[derive(Debug, Deserialize, IntoParams)]
pub struct HealthQuery {
    /// Include detailed stats
    pub detailed: Option<bool>,
}

/// Health response for memory system
#[derive(Debug, Serialize, ToSchema)]
pub struct MemoryHealthResponse {
    pub status: String,
    pub forgetting: ForgettingStatsResponse,
    pub protection: ProtectionStatsResponse,
    pub timestamp: String,
}

/// Forgetting statistics response
#[derive(Debug, Serialize, ToSchema)]
pub struct ForgettingStatsResponse {
    pub total_checks: u64,
    pub total_forgotten: u64,
    pub total_checked: u64,
    pub total_protected: u64,
    pub last_check_at: Option<String>,
    pub next_check_at: Option<String>,
    pub is_running: bool,
}

/// Protection statistics response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProtectionStatsResponse {
    pub critical_protected: usize,
    pub high_protected: usize,
    pub medium_protected: usize,
    pub low_protected: usize,
    pub total_protected: usize,
}

/// Request to trigger manual cleanup
#[derive(Debug, Deserialize, ToSchema)]
pub struct CleanupRequest {
    /// Dry run mode (don't actually delete)
    pub dry_run: Option<bool>,
    /// Force cleanup even protected memories
    pub force: Option<bool>,
}

/// Cleanup response
#[derive(Debug, Serialize, ToSchema)]
pub struct CleanupResponse {
    pub deleted_count: i64,
    pub checked_count: i64,
    pub protected_count: i64,
    pub dry_run: bool,
    pub reason: String,
}

/// Request to set protection level
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetProtectionRequest {
    pub memory_id: String,
    pub level: ProtectionLevelDto,
}

/// Protection level DTO
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProtectionLevelDto {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl From<ProtectionLevelDto> for ProtectionLevel {
    fn from(dto: ProtectionLevelDto) -> Self {
        match dto {
            ProtectionLevelDto::None => ProtectionLevel::None,
            ProtectionLevelDto::Low => ProtectionLevel::Low,
            ProtectionLevelDto::Medium => ProtectionLevel::Medium,
            ProtectionLevelDto::High => ProtectionLevel::High,
            ProtectionLevelDto::Critical => ProtectionLevel::Critical,
        }
    }
}

/// Protection response
#[derive(Debug, Serialize, ToSchema)]
pub struct ProtectionResponse {
    pub memory_id: String,
    pub level: String,
    pub success: bool,
}

// ============================================================================
// API Endpoints
// ============================================================================

/// GET /api/v1/memories/health
///
/// Get memory system health status including forgetting statistics
#[utoipa::path(
    get,
    path = "/api/v1/memories/health",
    tag = "memory",
    params(
        ("detailed" = Option<bool>, Query, description = "Include detailed statistics")
    ),
    responses(
        (status = 200, description = "Health status retrieved", body = MemoryHealthResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_memory_health(
    Extension(forgetting_state): Extension<Arc<ForgettingState>>,
    Query(_query): Query<HealthQuery>,
) -> ServerResult<Json<MemoryHealthResponse>> {
    debug!("Getting memory health status");

    let forgetting_stats = forgetting_state.get_stats().await?;

    // Get protection stats by level
    let all_protections = forgetting_state.protection.all_protections().await;
    let critical_count = all_protections.iter().filter(|(_, l)| matches!(l, ProtectionLevel::Critical)).count();
    let high_count = all_protections.iter().filter(|(_, l)| matches!(l, ProtectionLevel::High)).count();
    let medium_count = all_protections.iter().filter(|(_, l)| matches!(l, ProtectionLevel::Medium)).count();
    let low_count = all_protections.iter().filter(|(_, l)| matches!(l, ProtectionLevel::Low)).count();

    Ok(Json(MemoryHealthResponse {
        status: "healthy".to_string(),
        forgetting: forgetting_stats,
        protection: ProtectionStatsResponse {
            critical_protected: critical_count,
            high_protected: high_count,
            medium_protected: medium_count,
            low_protected: low_count,
            total_protected: all_protections.len(),
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// POST /api/v1/memories/cleanup
///
/// Manually trigger forgetting cleanup
#[utoipa::path(
    post,
    path = "/api/v1/memories/cleanup",
    tag = "memory",
    request_body = CleanupRequest,
    responses(
        (status = 200, description = "Cleanup completed", body = CleanupResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn trigger_cleanup(
    Extension(forgetting_state): Extension<Arc<ForgettingState>>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(request): Json<CleanupRequest>,
) -> ServerResult<Json<CleanupResponse>> {
    info!("Manual cleanup triggered, dry_run={}", request.dry_run.unwrap_or(false));

    let dry_run = request.dry_run.unwrap_or(false);

    // For now, just return a simulated response
    // In production, this would query memories and run forgetting check
    Ok(Json(CleanupResponse {
        deleted_count: 0,
        checked_count: 0,
        protected_count: 0,
        dry_run,
        reason: if dry_run {
            "dry_run".to_string()
        } else {
            "forgetting".to_string()
        },
    }))
}

/// GET /api/v1/memories/forgetting/stats
///
/// Get forgetting statistics
#[utoipa::path(
    get,
    path = "/api/v1/memories/forgetting/stats",
    tag = "memory",
    responses(
        (status = 200, description = "Stats retrieved", body = ForgettingStatsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_forgetting_stats(
    Extension(forgetting_state): Extension<Arc<ForgettingState>>,
    Extension(_auth_user): Extension<AuthUser>,
) -> ServerResult<Json<ForgettingStatsResponse>> {
    debug!("Getting forgetting stats");
    forgetting_state.get_stats().await.map(Json)
}

/// POST /api/v1/memories/protection
///
/// Set protection level for a memory
#[utoipa::path(
    post,
    path = "/api/v1/memories/protection",
    tag = "memory",
    request_body = SetProtectionRequest,
    responses(
        (status = 200, description = "Protection set", body = ProtectionResponse),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn set_memory_protection(
    Extension(forgetting_state): Extension<Arc<ForgettingState>>,
    Extension(_auth_user): Extension<AuthUser>,
    Json(request): Json<SetProtectionRequest>,
) -> ServerResult<Json<ProtectionResponse>> {
    info!("Setting protection for memory: {} -> {:?}", request.memory_id, request.level);

    let level: ProtectionLevel = request.level.into();
    forgetting_state.protection
        .set_protection(request.memory_id.clone(), level)
        .await;

    Ok(Json(ProtectionResponse {
        memory_id: request.memory_id,
        level: format!("{:?}", level),
        success: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protection_level_conversion() {
        let dto = ProtectionLevelDto::High;
        let level: ProtectionLevel = dto.into();
        assert!(matches!(level, ProtectionLevel::High));

        let dto = ProtectionLevelDto::Critical;
        let level: ProtectionLevel = dto.into();
        assert!(matches!(level, ProtectionLevel::Critical));
    }

    #[tokio::test]
    async fn test_forgetting_state_creation() {
        let config = ForgettingConfig::default();
        let state = ForgettingState::new(config);
        assert!(state.scheduler.read().await.is_none());
    }
}
