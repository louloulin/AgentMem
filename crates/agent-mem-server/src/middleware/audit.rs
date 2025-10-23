//! Audit logging middleware
//!
//! This module provides middleware for audit logging:
//! - Request logging
//! - User action tracking
//! - Security event logging
//! - File-based persistence for audit logs

use crate::middleware::AuthUser;
use axum::{extract::Request, middleware::Next, response::Response};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub timestamp: i64,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub error: Option<String>,
}

/// Audit log persistence manager
pub struct AuditLogManager {
    log_dir: PathBuf,
    buffer: Arc<RwLock<Vec<AuditLog>>>,
    security_buffer: Arc<RwLock<Vec<SecurityEvent>>>,
}

impl AuditLogManager {
    /// Create a new audit log manager
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            log_dir,
            buffer: Arc::new(RwLock::new(Vec::new())),
            security_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Store audit log (async)
    pub async fn store_audit_log(&self, log: AuditLog) -> Result<(), std::io::Error> {
        // Add to buffer
        {
            let mut buffer = self.buffer.write().await;
            buffer.push(log.clone());
        }

        // Write to file asynchronously
        let log_file = self.log_dir.join(format!(
            "audit-{}.jsonl",
            chrono::Utc::now().format("%Y-%m-%d")
        ));

        // Ensure directory exists
        if let Some(parent) = log_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;

        let json_line = serde_json::to_string(&log).unwrap_or_default();
        file.write_all(format!("{}\n", json_line).as_bytes())
            .await?;
        file.flush().await?;

        debug!("Stored audit log to {:?}", log_file);
        Ok(())
    }

    /// Store security event (async)
    pub async fn store_security_event(&self, event: SecurityEvent) -> Result<(), std::io::Error> {
        // Add to buffer
        {
            let mut buffer = self.security_buffer.write().await;
            buffer.push(event.clone());
        }

        // Write to file asynchronously
        let log_file = self.log_dir.join(format!(
            "security-{}.jsonl",
            chrono::Utc::now().format("%Y-%m-%d")
        ));

        // Ensure directory exists
        if let Some(parent) = log_file.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await?;

        let json_line = serde_json::to_string(&event).unwrap_or_default();
        file.write_all(format!("{}\n", json_line).as_bytes())
            .await?;
        file.flush().await?;

        debug!("Stored security event to {:?}", log_file);
        Ok(())
    }

    /// Get recent audit logs from buffer
    pub async fn get_recent_logs(&self, limit: usize) -> Vec<AuditLog> {
        let buffer = self.buffer.read().await;
        let start = if buffer.len() > limit {
            buffer.len() - limit
        } else {
            0
        };
        buffer[start..].to_vec()
    }

    /// Get recent security events from buffer
    pub async fn get_recent_security_events(&self, limit: usize) -> Vec<SecurityEvent> {
        let buffer = self.security_buffer.read().await;
        let start = if buffer.len() > limit {
            buffer.len() - limit
        } else {
            0
        };
        buffer[start..].to_vec()
    }
}

/// Extract IP address from request headers (generic over body type)
fn extract_ip_address<B>(request: &axum::http::Request<B>) -> Option<String> {
    // Try X-Forwarded-For header (for proxied requests)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }

    // Try to get from connection info (if available)
    // Note: This would require access to the connection remote address
    // which is not directly available in Axum middleware without custom extension
    
    None
}

/// Audit logging middleware
pub async fn audit_logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let timestamp = Utc::now().timestamp();

    // Extract request information
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Extract IP address
    let ip_address = extract_ip_address(&request);

    // Extract user information if authenticated
    let auth_user = request.extensions().get::<AuthUser>().cloned();
    let user_id = auth_user.as_ref().map(|u| u.user_id.clone());
    let organization_id = auth_user.as_ref().map(|u| u.org_id.clone());

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration_ms = start.elapsed().as_millis() as u64;
    let status_code = response.status().as_u16();

    // Determine action and resource type from path
    let (action, resource_type, resource_id) = parse_path(&path, &method);

    // Create audit log entry
    let audit_log = AuditLog {
        timestamp,
        user_id,
        organization_id,
        action,
        resource_type,
        resource_id,
        method,
        path: path.clone(),
        status_code,
        duration_ms,
        ip_address, // ✅ Now extracted from request
        user_agent,
        error: if status_code >= 400 {
            Some(format!("HTTP {status_code}"))
        } else {
            None
        },
    };

    // Log audit entry
    log_audit_entry(&audit_log);

    response
}

/// Parse path to extract action, resource type, and resource ID
fn parse_path(path: &str, method: &str) -> (String, String, Option<String>) {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() < 3 {
        return (method.to_lowercase(), "unknown".to_string(), None);
    }

    // Expected format: /api/v1/{resource_type}/{resource_id}
    let resource_type = if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        "unknown".to_string()
    };

    let resource_id = if parts.len() >= 4 && !parts[3].starts_with('?') {
        Some(parts[3].to_string())
    } else {
        None
    };

    let action = match method {
        "GET" => {
            if resource_id.is_some() {
                "read"
            } else {
                "list"
            }
        }
        "POST" => "create",
        "PUT" | "PATCH" => "update",
        "DELETE" => "delete",
        _ => "unknown",
    };

    (action.to_string(), resource_type, resource_id)
}

/// Global audit log manager instance (lazy static)
static AUDIT_MANAGER: once_cell::sync::Lazy<AuditLogManager> = once_cell::sync::Lazy::new(|| {
    let log_dir = std::env::var("AUDIT_LOG_DIR")
        .unwrap_or_else(|_| "./logs/audit".to_string())
        .into();
    AuditLogManager::new(log_dir)
});

/// Log audit entry with persistence
fn log_audit_entry(audit_log: &AuditLog) {
    let user_info = if let Some(user_id) = &audit_log.user_id {
        format!("user={user_id}")
    } else {
        "user=anonymous".to_string()
    };

    let org_info = if let Some(org_id) = &audit_log.organization_id {
        format!("org={org_id}")
    } else {
        String::new()
    };

    let resource_info = if let Some(resource_id) = &audit_log.resource_id {
        format!("{}:{}", audit_log.resource_type, resource_id)
    } else {
        audit_log.resource_type.clone()
    };

    let ip_info = if let Some(ip) = &audit_log.ip_address {
        format!("ip={}", ip)
    } else {
        String::new()
    };

    if audit_log.status_code >= 400 {
        warn!(
            "AUDIT: {} {} {} {} {} {} status={} duration={}ms error={:?}",
            user_info,
            org_info,
            ip_info,
            audit_log.action,
            resource_info,
            audit_log.method,
            audit_log.status_code,
            audit_log.duration_ms,
            audit_log.error
        );
    } else {
        info!(
            "AUDIT: {} {} {} {} {} {} status={} duration={}ms",
            user_info,
            org_info,
            ip_info,
            audit_log.action,
            resource_info,
            audit_log.method,
            audit_log.status_code,
            audit_log.duration_ms
        );
    }

    // ✅ Store audit log asynchronously (fire-and-forget)
    let log_clone = audit_log.clone();
    tokio::spawn(async move {
        if let Err(e) = AUDIT_MANAGER.store_audit_log(log_clone).await {
            warn!("Failed to persist audit log: {}", e);
        }
    });
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    LoginSuccess {
        user_id: String,
        ip_address: Option<String>,
    },
    LoginFailure {
        email: String,
        ip_address: Option<String>,
        reason: String,
    },
    PasswordChanged {
        user_id: String,
    },
    ApiKeyCreated {
        user_id: String,
        key_id: String,
    },
    ApiKeyRevoked {
        user_id: String,
        key_id: String,
    },
    UnauthorizedAccess {
        path: String,
        ip_address: Option<String>,
    },
    PermissionDenied {
        user_id: String,
        resource: String,
        action: String,
    },
}

/// Log security event with persistence
pub fn log_security_event(event: SecurityEvent) {
    match &event {
        SecurityEvent::LoginSuccess {
            user_id,
            ip_address,
        } => {
            info!(
                "SECURITY: Login successful - user={} ip={:?}",
                user_id, ip_address
            );
        }
        SecurityEvent::LoginFailure {
            email,
            ip_address,
            reason,
        } => {
            warn!(
                "SECURITY: Login failed - email={} ip={:?} reason={}",
                email, ip_address, reason
            );
        }
        SecurityEvent::PasswordChanged { user_id } => {
            info!("SECURITY: Password changed - user={}", user_id);
        }
        SecurityEvent::ApiKeyCreated { user_id, key_id } => {
            info!(
                "SECURITY: API key created - user={} key_id={}",
                user_id, key_id
            );
        }
        SecurityEvent::ApiKeyRevoked { user_id, key_id } => {
            info!(
                "SECURITY: API key revoked - user={} key_id={}",
                user_id, key_id
            );
        }
        SecurityEvent::UnauthorizedAccess { path, ip_address } => {
            warn!(
                "SECURITY: Unauthorized access attempt - path={} ip={:?}",
                path, ip_address
            );
        }
        SecurityEvent::PermissionDenied {
            user_id,
            resource,
            action,
        } => {
            warn!(
                "SECURITY: Permission denied - user={} resource={} action={}",
                user_id, resource, action
            );
        }
    }

    // ✅ Store security event asynchronously (fire-and-forget)
    let event_clone = event.clone();
    tokio::spawn(async move {
        if let Err(e) = AUDIT_MANAGER.store_security_event(event_clone).await {
            warn!("Failed to persist security event: {}", e);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_path() {
        let (action, resource_type, resource_id) = parse_path("/api/v1/memories/123", "GET");
        assert_eq!(action, "read");
        assert_eq!(resource_type, "memories");
        assert_eq!(resource_id, Some("123".to_string()));

        let (action, resource_type, resource_id) = parse_path("/api/v1/users", "GET");
        assert_eq!(action, "list");
        assert_eq!(resource_type, "users");
        assert_eq!(resource_id, None);

        let (action, resource_type, resource_id) = parse_path("/api/v1/agents", "POST");
        assert_eq!(action, "create");
        assert_eq!(resource_type, "agents");
        assert_eq!(resource_id, None);
    }

    #[tokio::test]
    async fn test_audit_log_manager_store() {
        // Create temporary directory for test logs
        let temp_dir = TempDir::new().unwrap();
        let manager = AuditLogManager::new(temp_dir.path().to_path_buf());

        // Create test audit log
        let log = AuditLog {
            timestamp: Utc::now().timestamp(),
            user_id: Some("test_user".to_string()),
            organization_id: Some("test_org".to_string()),
            action: "create".to_string(),
            resource_type: "memories".to_string(),
            resource_id: Some("mem123".to_string()),
            method: "POST".to_string(),
            path: "/api/v1/memories".to_string(),
            status_code: 201,
            duration_ms: 150,
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Test Agent".to_string()),
            error: None,
        };

        // Store the log
        manager.store_audit_log(log.clone()).await.unwrap();

        // Verify it's in the buffer
        let recent_logs = manager.get_recent_logs(10).await;
        assert_eq!(recent_logs.len(), 1);
        assert_eq!(recent_logs[0].user_id, Some("test_user".to_string()));
        assert_eq!(recent_logs[0].action, "create");

        // Verify file was created
        let log_file = temp_dir.path().join(format!(
            "audit-{}.jsonl",
            chrono::Utc::now().format("%Y-%m-%d")
        ));
        assert!(log_file.exists(), "Audit log file should be created");

        // Read the file and verify content
        let content = tokio::fs::read_to_string(&log_file).await.unwrap();
        assert!(content.contains("test_user"), "Log file should contain user_id");
        assert!(content.contains("mem123"), "Log file should contain resource_id");
    }

    #[tokio::test]
    async fn test_security_event_manager_store() {
        // Create temporary directory for test logs
        let temp_dir = TempDir::new().unwrap();
        let manager = AuditLogManager::new(temp_dir.path().to_path_buf());

        // Create test security event
        let event = SecurityEvent::LoginSuccess {
            user_id: "test_user".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
        };

        // Store the event
        manager.store_security_event(event.clone()).await.unwrap();

        // Verify it's in the buffer
        let recent_events = manager.get_recent_security_events(10).await;
        assert_eq!(recent_events.len(), 1);

        // Verify file was created
        let log_file = temp_dir.path().join(format!(
            "security-{}.jsonl",
            chrono::Utc::now().format("%Y-%m-%d")
        ));
        assert!(log_file.exists(), "Security log file should be created");

        // Read the file and verify content
        let content = tokio::fs::read_to_string(&log_file).await.unwrap();
        assert!(content.contains("test_user"), "Log file should contain user_id");
        assert!(content.contains("192.168.1.100"), "Log file should contain IP address");
    }

    #[test]
    fn test_extract_ip_address() {
        use axum::http::{HeaderValue};
        
        // Test X-Forwarded-For header
        let req = axum::http::Request::builder()
            .header("x-forwarded-for", "203.0.113.1, 198.51.100.1")
            .body(())
            .unwrap();
        let ip = extract_ip_address(&req);
        assert_eq!(ip, Some("203.0.113.1".to_string()));

        // Test X-Real-IP header
        let req = axum::http::Request::builder()
            .header("x-real-ip", "203.0.113.2")
            .body(())
            .unwrap();
        let ip = extract_ip_address(&req);
        assert_eq!(ip, Some("203.0.113.2".to_string()));

        // Test no headers
        let req = axum::http::Request::builder()
            .body(())
            .unwrap();
        let ip = extract_ip_address(&req);
        assert_eq!(ip, None);
    }

    #[tokio::test]
    async fn test_audit_log_manager_multiple_logs() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AuditLogManager::new(temp_dir.path().to_path_buf());

        // Store multiple logs
        for i in 0..5 {
            let log = AuditLog {
                timestamp: Utc::now().timestamp(),
                user_id: Some(format!("user_{}", i)),
                organization_id: Some("test_org".to_string()),
                action: "read".to_string(),
                resource_type: "memories".to_string(),
                resource_id: None,
                method: "GET".to_string(),
                path: "/api/v1/memories".to_string(),
                status_code: 200,
                duration_ms: 50,
                ip_address: Some(format!("192.168.1.{}", i)),
                user_agent: Some("Test Agent".to_string()),
                error: None,
            };
            manager.store_audit_log(log).await.unwrap();
        }

        // Verify all logs are in buffer
        let recent_logs = manager.get_recent_logs(10).await;
        assert_eq!(recent_logs.len(), 5);

        // Verify limited retrieval
        let limited_logs = manager.get_recent_logs(3).await;
        assert_eq!(limited_logs.len(), 3);
    }
}
