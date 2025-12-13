//! Quota management middleware
//!
//! This module provides middleware for quota management:
//! - Request rate limiting
//! - Resource quota checking
//! - Usage tracking

use crate::error::{ServerError, ServerResult};
use crate::middleware::AuthUser;
use axum::{extract::Request, middleware::Next, response::Response};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Quota limits for an organization
#[derive(Debug, Clone)]
pub struct QuotaLimits {
    pub max_requests_per_minute: u32,
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
    pub max_users: u32,
    pub max_agents: u32,
    pub max_memories: u32,
    pub max_api_keys: u32,
}

impl Default for QuotaLimits {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
            max_requests_per_day: 10000,
            max_users: 100,
            max_agents: 50,
            max_memories: 10000,
            max_api_keys: 10,
        }
    }
}

/// Usage statistics for an organization
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub requests_this_minute: u32,
    pub requests_this_hour: u32,
    pub requests_this_day: u32,
    pub last_minute_reset: DateTime<Utc>,
    pub last_hour_reset: DateTime<Utc>,
    pub last_day_reset: DateTime<Utc>,
    pub total_users: u32,
    pub total_agents: u32,
    pub total_memories: u32,
    pub total_api_keys: u32,
}

impl Default for UsageStats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            requests_this_minute: 0,
            requests_this_hour: 0,
            requests_this_day: 0,
            last_minute_reset: now,
            last_hour_reset: now,
            last_day_reset: now,
            total_users: 0,
            total_agents: 0,
            total_memories: 0,
            total_api_keys: 0,
        }
    }
}

/// Quota manager
pub struct QuotaManager {
    limits: Arc<RwLock<HashMap<String, QuotaLimits>>>,
    usage: Arc<RwLock<HashMap<String, UsageStats>>>,
}

impl QuotaManager {
    /// Create a new quota manager
    pub fn new() -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set quota limits for an organization
    pub async fn set_limits(&self, org_id: &str, limits: QuotaLimits) {
        let mut limits_map = self.limits.write().await;
        limits_map.insert(org_id.to_string(), limits);
    }

    /// Get quota limits for an organization
    pub async fn get_limits(&self, org_id: &str) -> QuotaLimits {
        let limits_map = self.limits.read().await;
        limits_map.get(org_id).cloned().unwrap_or_default()
    }

    /// Get usage stats for an organization
    pub async fn get_usage(&self, org_id: &str) -> UsageStats {
        let usage_map = self.usage.read().await;
        usage_map.get(org_id).cloned().unwrap_or_default()
    }

    /// Check if request is within quota
    pub async fn check_request_quota(&self, org_id: &str) -> ServerResult<()> {
        let limits = self.get_limits(org_id).await;
        let mut usage_map = self.usage.write().await;
        let usage = usage_map
            .entry(org_id.to_string())
            .or_insert_with(UsageStats::default);

        let now = Utc::now();

        // Reset counters if time windows have passed
        if now - usage.last_minute_reset > Duration::minutes(1) {
            usage.requests_this_minute = 0;
            usage.last_minute_reset = now;
        }
        if now - usage.last_hour_reset > Duration::hours(1) {
            usage.requests_this_hour = 0;
            usage.last_hour_reset = now;
        }
        if now - usage.last_day_reset > Duration::days(1) {
            usage.requests_this_day = 0;
            usage.last_day_reset = now;
        }

        // Check quotas
        if usage.requests_this_minute >= limits.max_requests_per_minute {
            return Err(ServerError::QuotaExceeded(
                "Rate limit exceeded: too many requests per minute".to_string(),
            ));
        }
        if usage.requests_this_hour >= limits.max_requests_per_hour {
            return Err(ServerError::QuotaExceeded(
                "Rate limit exceeded: too many requests per hour".to_string(),
            ));
        }
        if usage.requests_this_day >= limits.max_requests_per_day {
            return Err(ServerError::QuotaExceeded(
                "Rate limit exceeded: too many requests per day".to_string(),
            ));
        }

        // Increment counters
        usage.requests_this_minute += 1;
        usage.requests_this_hour += 1;
        usage.requests_this_day += 1;

        Ok(())
    }

    /// Check if resource creation is within quota
    pub async fn check_resource_quota(
        &self,
        org_id: &str,
        resource_type: &str,
    ) -> ServerResult<()> {
        let limits = self.get_limits(org_id).await;
        let usage = self.get_usage(org_id).await;

        match resource_type {
            "user" => {
                if usage.total_users >= limits.max_users {
                    return Err(ServerError::QuotaExceeded(format!(
                        "User quota exceeded: {} / {}",
                        usage.total_users, limits.max_users
                    )));
                }
            }
            "agent" => {
                if usage.total_agents >= limits.max_agents {
                    return Err(ServerError::QuotaExceeded(format!(
                        "Agent quota exceeded: {} / {}",
                        usage.total_agents, limits.max_agents
                    )));
                }
            }
            "memory" => {
                if usage.total_memories >= limits.max_memories {
                    return Err(ServerError::QuotaExceeded(format!(
                        "Memory quota exceeded: {} / {}",
                        usage.total_memories, limits.max_memories
                    )));
                }
            }
            "api_key" => {
                if usage.total_api_keys >= limits.max_api_keys {
                    return Err(ServerError::QuotaExceeded(format!(
                        "API key quota exceeded: {} / {}",
                        usage.total_api_keys, limits.max_api_keys
                    )));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Update resource count
    pub async fn update_resource_count(&self, org_id: &str, resource_type: &str, delta: i32) {
        let mut usage_map = self.usage.write().await;
        let usage = usage_map
            .entry(org_id.to_string())
            .or_insert_with(UsageStats::default);

        match resource_type {
            "user" => {
                usage.total_users = (usage.total_users as i32 + delta).max(0) as u32;
            }
            "agent" => {
                usage.total_agents = (usage.total_agents as i32 + delta).max(0) as u32;
            }
            "memory" => {
                usage.total_memories = (usage.total_memories as i32 + delta).max(0) as u32;
            }
            "api_key" => {
                usage.total_api_keys = (usage.total_api_keys as i32 + delta).max(0) as u32;
            }
            _ => {}
        }
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Quota checking middleware
/// 
/// This middleware enforces rate limiting and quota checks per organization.
/// It checks:
/// - Requests per minute
/// - Requests per hour  
/// - Requests per day
/// 
/// If quota is exceeded, returns a 429 Too Many Requests error.
pub async fn quota_middleware(request: Request, next: Next) -> Result<Response, ServerError> {
    // Extract authenticated user
    let auth_user = request.extensions().get::<AuthUser>().cloned();

    if let Some(user) = auth_user {
        // Get quota manager from extensions
        if let Some(quota_manager) = request.extensions().get::<Arc<QuotaManager>>() {
            // Check request quota
            quota_manager.check_request_quota(&user.org_id).await?;
        }
    }
    // If no authenticated user, allow request (will be handled by auth middleware)

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quota_manager() {
        let manager = QuotaManager::new();

        // Set limits
        let limits = QuotaLimits {
            max_requests_per_minute: 5,
            ..Default::default()
        };
        manager.set_limits("org123", limits).await;

        // Check quota (should pass)
        for _ in 0..5 {
            assert!(manager.check_request_quota("org123").await.is_ok());
        }

        // Check quota (should fail)
        assert!(manager.check_request_quota("org123").await.is_err());
    }

    #[tokio::test]
    async fn test_resource_quota() {
        let manager = QuotaManager::new();

        // Set limits
        let limits = QuotaLimits {
            max_users: 2,
            ..Default::default()
        };
        manager.set_limits("org123", limits).await;

        // Update resource count
        manager.update_resource_count("org123", "user", 2).await;

        // Check quota (should fail)
        assert!(manager
            .check_resource_quota("org123", "user")
            .await
            .is_err());
    }
}
