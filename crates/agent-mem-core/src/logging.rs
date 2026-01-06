//! Production Logging and Audit System
//!
//! Comprehensive logging, audit trails, and security monitoring for production
//! environments with structured logging and compliance support.

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Logging system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable structured logging
    pub enable_structured_logging: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Enable security logging
    pub enable_security_logging: bool,
    /// Log level filter
    pub log_level: LogLevel,
    /// Maximum log buffer size
    pub max_log_buffer_size: usize,
    /// Log retention period in days
    pub log_retention_days: u32,
    /// Enable log compression
    pub enable_compression: bool,
    /// Enable log encryption
    pub enable_encryption: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_structured_logging: true,
            enable_audit_logging: true,
            enable_security_logging: true,
            log_level: LogLevel::Info,
            max_log_buffer_size: 50000,
            log_retention_days: 90,
            enable_compression: true,
            enable_encryption: false,
        }
    }
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub operation: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    MemoryCreated,
    MemoryUpdated,
    MemoryDeleted,
    MemoryAccessed,
    UserLogin,
    UserLogout,
    ConfigurationChanged,
    SecurityViolation,
    DataExport,
    DataImport,
    SystemStartup,
    SystemShutdown,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: String,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub risk_score: u8, // 0-100
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Partial,
    Denied,
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationFailure,
    AuthorizationFailure,
    SuspiciousActivity,
    DataBreach,
    UnauthorizedAccess,
    MaliciousRequest,
    RateLimitExceeded,
    AnomalousPattern,
}

/// Security log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub description: String,
    pub threat_indicators: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Security severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive logging system
pub struct LoggingSystem {
    config: LoggingConfig,
    log_entries: Arc<RwLock<VecDeque<LogEntry>>>,
    audit_entries: Arc<RwLock<VecDeque<AuditEntry>>>,
    security_entries: Arc<RwLock<VecDeque<SecurityEntry>>>,
    log_filters: Arc<RwLock<Vec<LogFilter>>>,
}

/// Log filter for advanced filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub name: String,
    pub component: Option<String>,
    pub level: Option<LogLevel>,
    pub user_id: Option<String>,
    pub tags: Vec<String>,
    pub enabled: bool,
}

impl LoggingSystem {
    /// Create a new logging system
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            log_entries: Arc::new(RwLock::new(VecDeque::new())),
            audit_entries: Arc::new(RwLock::new(VecDeque::new())),
            security_entries: Arc::new(RwLock::new(VecDeque::new())),
            log_filters: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Log a structured message
    pub async fn log(
        &self,
        level: LogLevel,
        message: String,
        component: String,
        operation: Option<String>,
        user_id: Option<String>,
        session_id: Option<String>,
        request_id: Option<String>,
        fields: HashMap<String, serde_json::Value>,
        tags: Vec<String>,
    ) -> Result<()> {
        if !self.config.enable_structured_logging || level < self.config.log_level {
            return Ok(());
        }

        let entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            message,
            component,
            operation,
            user_id,
            session_id,
            request_id,
            fields,
            tags,
        };

        let mut log_entries = self.log_entries.write().await;
        log_entries.push_back(entry);

        // Limit buffer size
        while log_entries.len() > self.config.max_log_buffer_size {
            log_entries.pop_front();
        }

        Ok(())
    }

    /// Log an info message
    pub async fn info(&self, message: &str, component: &str) -> Result<()> {
        self.log(
            LogLevel::Info,
            message.to_string(),
            component.to_string(),
            None,
            None,
            None,
            None,
            HashMap::new(),
            Vec::new(),
        )
        .await
    }

    /// Log a warning message
    pub async fn warn(&self, message: &str, component: &str) -> Result<()> {
        self.log(
            LogLevel::Warn,
            message.to_string(),
            component.to_string(),
            None,
            None,
            None,
            None,
            HashMap::new(),
            Vec::new(),
        )
        .await
    }

    /// Log an error message
    pub async fn error(
        &self,
        message: &str,
        component: &str,
        error: Option<&dyn std::error::Error>,
    ) -> Result<()> {
        let mut fields = HashMap::new();
        if let Some(err) = error {
            fields.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
        }

        self.log(
            LogLevel::Error,
            message.to_string(),
            component.to_string(),
            None,
            None,
            None,
            None,
            fields,
            vec!["error".to_string()],
        )
        .await
    }

    /// Log an audit event
    pub async fn audit(
        &self,
        event_type: AuditEventType,
        user_id: Option<String>,
        session_id: Option<String>,
        resource_id: Option<String>,
        resource_type: Option<String>,
        action: String,
        result: AuditResult,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.config.enable_audit_logging {
            return Ok(());
        }

        // Calculate risk score based on event type and result
        let risk_score = self.calculate_risk_score(&event_type, &result);

        let entry = AuditEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            user_id,
            session_id,
            resource_id,
            resource_type,
            action,
            result,
            ip_address,
            user_agent,
            details,
            risk_score,
        };

        let mut audit_entries = self.audit_entries.write().await;
        audit_entries.push_back(entry);

        // Limit buffer size
        while audit_entries.len() > self.config.max_log_buffer_size {
            audit_entries.pop_front();
        }

        Ok(())
    }

    /// Log a security event
    pub async fn security(
        &self,
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        source_ip: Option<String>,
        user_id: Option<String>,
        session_id: Option<String>,
        description: String,
        threat_indicators: Vec<String>,
        mitigation_actions: Vec<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if !self.config.enable_security_logging {
            return Ok(());
        }

        let entry = SecurityEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            severity,
            source_ip,
            user_id,
            session_id,
            description,
            threat_indicators,
            mitigation_actions,
            metadata,
        };

        let mut security_entries = self.security_entries.write().await;
        security_entries.push_back(entry);

        // Limit buffer size
        while security_entries.len() > self.config.max_log_buffer_size {
            security_entries.pop_front();
        }

        Ok(())
    }

    /// Calculate risk score for audit events
    fn calculate_risk_score(&self, event_type: &AuditEventType, result: &AuditResult) -> u8 {
        let base_score = match event_type {
            AuditEventType::MemoryAccessed => 10,
            AuditEventType::MemoryCreated => 20,
            AuditEventType::MemoryUpdated => 25,
            AuditEventType::MemoryDeleted => 40,
            AuditEventType::UserLogin => 15,
            AuditEventType::UserLogout => 5,
            AuditEventType::ConfigurationChanged => 60,
            AuditEventType::SecurityViolation => 90,
            AuditEventType::DataExport => 70,
            AuditEventType::DataImport => 50,
            AuditEventType::SystemStartup => 30,
            AuditEventType::SystemShutdown => 35,
        };

        let result_multiplier = match result {
            AuditResult::Success => 1.0,
            AuditResult::Partial => 1.2,
            AuditResult::Failure => 1.5,
            AuditResult::Denied => 2.0,
        };

        ((base_score as f64 * result_multiplier) as u8).min(100)
    }

    /// Get log entries with optional filtering
    pub async fn get_logs(&self, filter: Option<&LogFilter>) -> Vec<LogEntry> {
        let log_entries = self.log_entries.read().await;

        if let Some(filter) = filter {
            log_entries
                .iter()
                .filter(|entry| self.matches_log_filter(entry, filter))
                .cloned()
                .collect()
        } else {
            log_entries.iter().cloned().collect()
        }
    }

    /// Get audit entries
    pub async fn get_audit_logs(&self) -> Vec<AuditEntry> {
        let audit_entries = self.audit_entries.read().await;
        audit_entries.iter().cloned().collect()
    }

    /// Get security entries
    pub async fn get_security_logs(&self) -> Vec<SecurityEntry> {
        let security_entries = self.security_entries.read().await;
        security_entries.iter().cloned().collect()
    }

    /// Check if log entry matches filter
    fn matches_log_filter(&self, entry: &LogEntry, filter: &LogFilter) -> bool {
        if !filter.enabled {
            return true;
        }

        if let Some(ref component) = filter.component {
            if &entry.component != component {
                return false;
            }
        }

        if let Some(ref level) = filter.level {
            if &entry.level != level {
                return false;
            }
        }

        if let Some(ref user_id) = filter.user_id {
            if entry.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if !filter.tags.is_empty() {
            let has_matching_tag = filter.tags.iter().any(|tag| entry.tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        true
    }

    /// Add log filter
    pub async fn add_log_filter(&self, filter: LogFilter) -> Result<()> {
        let mut log_filters = self.log_filters.write().await;
        log_filters.push(filter);
        Ok(())
    }

    /// Get high-risk audit events
    pub async fn get_high_risk_audit_events(&self, threshold: u8) -> Vec<AuditEntry> {
        let audit_entries = self.audit_entries.read().await;
        audit_entries
            .iter()
            .filter(|entry| entry.risk_score >= threshold)
            .cloned()
            .collect()
    }

    /// Get critical security events
    pub async fn get_critical_security_events(&self) -> Vec<SecurityEntry> {
        let security_entries = self.security_entries.read().await;
        security_entries
            .iter()
            .filter(|entry| entry.severity >= SecuritySeverity::High)
            .cloned()
            .collect()
    }

    /// Clean up old logs
    pub async fn cleanup_old_logs(&self) -> Result<()> {
        let cutoff_time =
            Utc::now() - chrono::Duration::days(self.config.log_retention_days as i64);

        // Clean up old log entries
        {
            let mut log_entries = self.log_entries.write().await;
            log_entries.retain(|entry| entry.timestamp > cutoff_time);
        }

        // Clean up old audit entries
        {
            let mut audit_entries = self.audit_entries.write().await;
            audit_entries.retain(|entry| entry.timestamp > cutoff_time);
        }

        // Clean up old security entries
        {
            let mut security_entries = self.security_entries.write().await;
            security_entries.retain(|entry| entry.timestamp > cutoff_time);
        }

        Ok(())
    }

    /// Export logs for compliance
    pub async fn export_logs_for_compliance(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ComplianceExport> {
        let log_entries = self.log_entries.read().await;
        let audit_entries = self.audit_entries.read().await;
        let security_entries = self.security_entries.read().await;

        let filtered_logs: Vec<_> = log_entries
            .iter()
            .filter(|entry| entry.timestamp >= start_time && entry.timestamp <= end_time)
            .cloned()
            .collect();

        let filtered_audit: Vec<_> = audit_entries
            .iter()
            .filter(|entry| entry.timestamp >= start_time && entry.timestamp <= end_time)
            .cloned()
            .collect();

        let filtered_security: Vec<_> = security_entries
            .iter()
            .filter(|entry| entry.timestamp >= start_time && entry.timestamp <= end_time)
            .cloned()
            .collect();

        Ok(ComplianceExport {
            export_id: Uuid::new_v4().to_string(),
            export_timestamp: Utc::now(),
            start_time,
            end_time,
            log_entries: filtered_logs,
            audit_entries: filtered_audit,
            security_entries: filtered_security,
        })
    }
}

/// Compliance export structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceExport {
    pub export_id: String,
    pub export_timestamp: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub log_entries: Vec<LogEntry>,
    pub audit_entries: Vec<AuditEntry>,
    pub security_entries: Vec<SecurityEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logging_system_creation() {
        let config = LoggingConfig::default();
        let logging = LoggingSystem::new(config);

        logging
            .info("Test message", "test_component")
            .await
            .unwrap();

        let logs = logging.get_logs(None).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let config = LoggingConfig::default();
        let logging = LoggingSystem::new(config);

        let mut details = HashMap::new();
        details.insert(
            "memory_id".to_string(),
            serde_json::Value::String("test-123".to_string()),
        );

        logging
            .audit(
                AuditEventType::MemoryCreated,
                Some("user123".to_string()),
                Some("session456".to_string()),
                Some("memory789".to_string()),
                Some("memory".to_string()),
                "create_memory".to_string(),
                AuditResult::Success,
                Some("192.168.1.1".to_string()),
                Some("Mozilla/5.0".to_string()),
                details,
            )
            .await
            .unwrap();

        let audit_logs = logging.get_audit_logs().await;
        assert_eq!(audit_logs.len(), 1);
        assert_eq!(audit_logs[0].action, "create_memory");
        assert!(audit_logs[0].risk_score > 0);
    }

    #[tokio::test]
    async fn test_security_logging() {
        let config = LoggingConfig::default();
        let logging = LoggingSystem::new(config);

        logging
            .security(
                SecurityEventType::AuthenticationFailure,
                SecuritySeverity::High,
                Some("192.168.1.100".to_string()),
                Some("attacker".to_string()),
                None,
                "Multiple failed login attempts".to_string(),
                vec!["brute_force".to_string(), "suspicious_ip".to_string()],
                vec!["ip_blocked".to_string(), "user_notified".to_string()],
                HashMap::new(),
            )
            .await
            .unwrap();

        let security_logs = logging.get_security_logs().await;
        assert_eq!(security_logs.len(), 1);
        assert_eq!(security_logs[0].severity, SecuritySeverity::High);

        let critical_events = logging.get_critical_security_events().await;
        assert_eq!(critical_events.len(), 1);
    }

    #[tokio::test]
    async fn test_risk_score_calculation() {
        let config = LoggingConfig::default();
        let logging = LoggingSystem::new(config);

        let score1 =
            logging.calculate_risk_score(&AuditEventType::MemoryAccessed, &AuditResult::Success);
        let score2 =
            logging.calculate_risk_score(&AuditEventType::SecurityViolation, &AuditResult::Denied);

        assert!(score1 < score2);
        assert!(score2 > 90);
    }
}
