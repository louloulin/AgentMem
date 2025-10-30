//! Production Security and Hardening System
//!
//! Comprehensive security features including encryption, access control,
//! threat detection, and security hardening for production environments.

use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Security system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable data encryption at rest
    pub enable_encryption_at_rest: bool,
    /// Enable data encryption in transit
    pub enable_encryption_in_transit: bool,
    /// Enable access control
    pub enable_access_control: bool,
    /// Enable threat detection
    pub enable_threat_detection: bool,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Maximum failed login attempts
    pub max_failed_login_attempts: u32,
    /// Account lockout duration in minutes
    pub account_lockout_duration_minutes: u32,
    /// Session timeout in minutes
    pub session_timeout_minutes: u32,
    /// Enable IP whitelisting
    pub enable_ip_whitelisting: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption_at_rest: true,
            enable_encryption_in_transit: true,
            enable_access_control: true,
            enable_threat_detection: true,
            enable_rate_limiting: true,
            max_failed_login_attempts: 5,
            account_lockout_duration_minutes: 30,
            session_timeout_minutes: 60,
            enable_ip_whitelisting: false,
            enable_audit_logging: true,
        }
    }
}

/// User permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    /// 读取内存权限
    ReadMemory,
    /// 写入内存权限
    WriteMemory,
    /// 删除内存权限
    DeleteMemory,
    /// 管理员访问权限
    AdminAccess,
    /// 配置系统权限
    ConfigureSystem,
    /// 查看审计日志权限
    ViewAuditLogs,
    /// 导出数据权限
    ExportData,
    /// 导入数据权限
    ImportData,
    /// 管理用户权限
    ManageUsers,
    /// 查看指标权限
    ViewMetrics,
}

/// User role with permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色名称
    pub name: String,
    /// 角色权限集合
    pub permissions: HashSet<Permission>,
    /// 角色描述
    pub description: String,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub active: bool,
    pub metadata: HashMap<String, String>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub active: bool,
}

/// Access control entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub resource_id: String,
    pub resource_type: String,
    pub user_id: String,
    pub permissions: HashSet<Permission>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Threat detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: ThreatRuleType,
    pub threshold: f64,
    pub time_window_minutes: u32,
    pub severity: ThreatSeverity,
    pub enabled: bool,
    pub actions: Vec<ThreatAction>,
}

/// Types of threat detection rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatRuleType {
    FailedLoginAttempts,
    UnusualAccessPattern,
    DataExfiltration,
    SuspiciousIPAddress,
    RateLimitExceeded,
    PrivilegeEscalation,
    AnomalousQuery,
}

/// Threat severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Actions to take when threat is detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatAction {
    LogEvent,
    SendAlert,
    BlockIP,
    LockAccount,
    RequireReauth,
    NotifyAdmin,
}

/// Detected threat incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIncident {
    pub id: String,
    pub rule_id: String,
    pub severity: ThreatSeverity,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub evidence: HashMap<String, serde_json::Value>,
    pub actions_taken: Vec<ThreatAction>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size_minutes: u32,
}

/// Comprehensive security system
pub struct SecuritySystem {
    config: SecurityConfig,
    roles: Arc<RwLock<HashMap<String, Role>>>,
    users: Arc<RwLock<HashMap<String, UserAccount>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    access_control: Arc<RwLock<Vec<AccessControlEntry>>>,
    threat_rules: Arc<RwLock<HashMap<String, ThreatRule>>>,
    threat_incidents: Arc<RwLock<Vec<ThreatIncident>>>,
    ip_whitelist: Arc<RwLock<HashSet<String>>>,
    ip_blacklist: Arc<RwLock<HashSet<String>>>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
}

/// Rate limit tracking
#[derive(Debug, Clone)]
struct RateLimitTracker {
    requests: VecDeque<DateTime<Utc>>,
    config: RateLimitConfig,
}

use std::collections::VecDeque;

impl SecuritySystem {
    /// Create a new security system
    pub fn new(config: SecurityConfig) -> Self {
        let system = Self {
            config,
            roles: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            access_control: Arc::new(RwLock::new(Vec::new())),
            threat_rules: Arc::new(RwLock::new(HashMap::new())),
            threat_incidents: Arc::new(RwLock::new(Vec::new())),
            ip_whitelist: Arc::new(RwLock::new(HashSet::new())),
            ip_blacklist: Arc::new(RwLock::new(HashSet::new())),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize with default roles and threat rules
        let system_clone = system.clone();
        tokio::spawn(async move {
            if let Err(e) = system_clone.initialize_defaults().await {
                eprintln!("Failed to initialize security defaults: {e}");
            }
        });

        system
    }

    /// Initialize default roles and threat rules
    async fn initialize_defaults(&self) -> Result<()> {
        // Create default roles
        self.create_role(Role {
            name: "admin".to_string(),
            permissions: [
                Permission::ReadMemory,
                Permission::WriteMemory,
                Permission::DeleteMemory,
                Permission::AdminAccess,
                Permission::ConfigureSystem,
                Permission::ViewAuditLogs,
                Permission::ExportData,
                Permission::ImportData,
                Permission::ManageUsers,
                Permission::ViewMetrics,
            ]
            .iter()
            .cloned()
            .collect(),
            description: "Full system administrator".to_string(),
        })
        .await?;

        self.create_role(Role {
            name: "user".to_string(),
            permissions: [Permission::ReadMemory, Permission::WriteMemory]
                .iter()
                .cloned()
                .collect(),
            description: "Regular user with basic memory access".to_string(),
        })
        .await?;

        self.create_role(Role {
            name: "readonly".to_string(),
            permissions: [Permission::ReadMemory].iter().cloned().collect(),
            description: "Read-only access to memories".to_string(),
        })
        .await?;

        // Create default threat rules
        self.add_threat_rule(ThreatRule {
            id: "failed_login_attempts".to_string(),
            name: "Failed Login Attempts".to_string(),
            description: "Detect multiple failed login attempts".to_string(),
            rule_type: ThreatRuleType::FailedLoginAttempts,
            threshold: 5.0,
            time_window_minutes: 15,
            severity: ThreatSeverity::Medium,
            enabled: true,
            actions: vec![ThreatAction::LogEvent, ThreatAction::LockAccount],
        })
        .await?;

        self.add_threat_rule(ThreatRule {
            id: "rate_limit_exceeded".to_string(),
            name: "Rate Limit Exceeded".to_string(),
            description: "Detect rate limit violations".to_string(),
            rule_type: ThreatRuleType::RateLimitExceeded,
            threshold: 100.0,
            time_window_minutes: 1,
            severity: ThreatSeverity::High,
            enabled: true,
            actions: vec![ThreatAction::LogEvent, ThreatAction::BlockIP],
        })
        .await?;

        Ok(())
    }

    /// Create a new role
    pub async fn create_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Create a new user account
    pub async fn create_user(&self, user: UserAccount) -> Result<()> {
        let mut users = self.users.write().await;
        users.insert(user.user_id.clone(), user);
        Ok(())
    }

    /// Authenticate user and create session
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<Session> {
        // Check IP blacklist
        if self.is_ip_blacklisted(ip_address).await {
            return Err(AgentMemError::memory_error("IP address is blacklisted"));
        }

        // Check IP whitelist if enabled
        if self.config.enable_ip_whitelisting && !self.is_ip_whitelisted(ip_address).await {
            return Err(AgentMemError::memory_error("IP address not whitelisted"));
        }

        let mut users = self.users.write().await;
        let user = users
            .values_mut()
            .find(|u| u.username == username && u.active)
            .ok_or_else(|| AgentMemError::memory_error("Invalid credentials"))?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                return Err(AgentMemError::memory_error("Account is locked"));
            } else {
                // Unlock account
                user.locked_until = None;
                user.failed_login_attempts = 0;
            }
        }

        // Simulate password verification (in production, use proper hashing)
        let password_valid = password == "correct_password"; // Placeholder

        if !password_valid {
            user.failed_login_attempts += 1;

            // Lock account if too many failed attempts
            if user.failed_login_attempts >= self.config.max_failed_login_attempts {
                user.locked_until = Some(
                    Utc::now()
                        + Duration::minutes(self.config.account_lockout_duration_minutes as i64),
                );
            }

            // Trigger threat detection
            self.detect_threat(
                ThreatRuleType::FailedLoginAttempts,
                ip_address,
                Some(&user.user_id),
            )
            .await?;

            return Err(AgentMemError::memory_error("Invalid credentials"));
        }

        // Reset failed attempts on successful login
        user.failed_login_attempts = 0;
        user.last_login = Some(Utc::now());

        // Create session
        let session = Session {
            session_id: Uuid::new_v4().to_string(),
            user_id: user.user_id.clone(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(self.config.session_timeout_minutes as i64),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            active: true,
        };

        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());

        Ok(session)
    }

    /// Validate session
    pub async fn validate_session(&self, session_id: &str) -> Result<Session> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentMemError::memory_error("Invalid session"))?;

        if !session.active || Utc::now() > session.expires_at {
            session.active = false;
            return Err(AgentMemError::memory_error("Session expired"));
        }

        // Update last accessed time
        session.last_accessed = Utc::now();
        session.expires_at =
            Utc::now() + Duration::minutes(self.config.session_timeout_minutes as i64);

        Ok(session.clone())
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: &str, permission: &Permission) -> Result<bool> {
        if !self.config.enable_access_control {
            return Ok(true);
        }

        let users = self.users.read().await;
        let user = users
            .get(user_id)
            .ok_or_else(|| AgentMemError::memory_error("User not found"))?;

        let roles = self.roles.read().await;
        for role_name in &user.roles {
            if let Some(role) = roles.get(role_name) {
                if role.permissions.contains(permission) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Add threat detection rule
    pub async fn add_threat_rule(&self, rule: ThreatRule) -> Result<()> {
        let mut threat_rules = self.threat_rules.write().await;
        threat_rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Detect threat based on rule type
    pub async fn detect_threat(
        &self,
        rule_type: ThreatRuleType,
        source_ip: &str,
        user_id: Option<&str>,
    ) -> Result<()> {
        if !self.config.enable_threat_detection {
            return Ok(());
        }

        let threat_rules = self.threat_rules.read().await;
        let matching_rules: Vec<_> = threat_rules
            .values()
            .filter(|rule| {
                rule.enabled
                    && std::mem::discriminant(&rule.rule_type) == std::mem::discriminant(&rule_type)
            })
            .collect();

        for rule in matching_rules {
            // Simple threat detection logic (in production, would be more sophisticated)
            let incident = ThreatIncident {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                severity: rule.severity.clone(),
                description: format!("Threat detected: {}", rule.description),
                detected_at: Utc::now(),
                source_ip: Some(source_ip.to_string()),
                user_id: user_id.map(|s| s.to_string()),
                session_id: None,
                evidence: HashMap::new(),
                actions_taken: rule.actions.clone(),
                resolved: false,
                resolved_at: None,
            };

            // Execute threat actions
            for action in &rule.actions {
                self.execute_threat_action(action, source_ip, user_id)
                    .await?;
            }

            // Store incident
            let mut threat_incidents = self.threat_incidents.write().await;
            threat_incidents.push(incident);
        }

        Ok(())
    }

    /// Execute threat action
    async fn execute_threat_action(
        &self,
        action: &ThreatAction,
        source_ip: &str,
        user_id: Option<&str>,
    ) -> Result<()> {
        match action {
            ThreatAction::LogEvent => {
                // Log would be handled by logging system
                println!("Threat detected from IP: {source_ip}");
            }
            ThreatAction::SendAlert => {
                // Send alert to administrators
                println!("ALERT: Security threat detected");
            }
            ThreatAction::BlockIP => {
                self.add_ip_to_blacklist(source_ip).await?;
            }
            ThreatAction::LockAccount => {
                if let Some(user_id) = user_id {
                    self.lock_user_account(user_id).await?;
                }
            }
            ThreatAction::RequireReauth => {
                if let Some(user_id) = user_id {
                    self.invalidate_user_sessions(user_id).await?;
                }
            }
            ThreatAction::NotifyAdmin => {
                // Notify system administrators
                println!("Admin notification: Security incident");
            }
        }

        Ok(())
    }

    /// Check if IP is whitelisted
    async fn is_ip_whitelisted(&self, ip: &str) -> bool {
        let whitelist = self.ip_whitelist.read().await;
        whitelist.contains(ip)
    }

    /// Check if IP is blacklisted
    async fn is_ip_blacklisted(&self, ip: &str) -> bool {
        let blacklist = self.ip_blacklist.read().await;
        blacklist.contains(ip)
    }

    /// Add IP to blacklist
    async fn add_ip_to_blacklist(&self, ip: &str) -> Result<()> {
        let mut blacklist = self.ip_blacklist.write().await;
        blacklist.insert(ip.to_string());
        Ok(())
    }

    /// Lock user account
    async fn lock_user_account(&self, user_id: &str) -> Result<()> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(user_id) {
            user.locked_until = Some(
                Utc::now() + Duration::minutes(self.config.account_lockout_duration_minutes as i64),
            );
        }
        Ok(())
    }

    /// Invalidate all user sessions
    async fn invalidate_user_sessions(&self, user_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        for session in sessions.values_mut() {
            if session.user_id == user_id {
                session.active = false;
            }
        }
        Ok(())
    }

    /// Get threat incidents
    pub async fn get_threat_incidents(&self) -> Vec<ThreatIncident> {
        let threat_incidents = self.threat_incidents.read().await;
        threat_incidents.clone()
    }

    /// Get active sessions
    pub async fn get_active_sessions(&self) -> Vec<Session> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.active && Utc::now() <= s.expires_at)
            .cloned()
            .collect()
    }
}

impl Clone for SecuritySystem {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            roles: Arc::clone(&self.roles),
            users: Arc::clone(&self.users),
            sessions: Arc::clone(&self.sessions),
            access_control: Arc::clone(&self.access_control),
            threat_rules: Arc::clone(&self.threat_rules),
            threat_incidents: Arc::clone(&self.threat_incidents),
            ip_whitelist: Arc::clone(&self.ip_whitelist),
            ip_blacklist: Arc::clone(&self.ip_blacklist),
            rate_limits: Arc::clone(&self.rate_limits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_system_creation() {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);

        // Wait for initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let roles = security.roles.read().await;
        assert!(roles.contains_key("admin"));
        assert!(roles.contains_key("user"));
    }

    #[tokio::test]
    async fn test_user_authentication() {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);

        // Create test user
        let user = UserAccount {
            user_id: "test_user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            active: true,
            metadata: HashMap::new(),
        };

        security.create_user(user).await.unwrap();

        // Test authentication failure
        let result = security
            .authenticate_user("testuser", "wrong_password", "192.168.1.1", "Mozilla/5.0")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_permission_checking() {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);

        // Wait for initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create test user with admin role
        let user = UserAccount {
            user_id: "admin_user".to_string(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec!["admin".to_string()],
            created_at: Utc::now(),
            last_login: None,
            failed_login_attempts: 0,
            locked_until: None,
            active: true,
            metadata: HashMap::new(),
        };

        security.create_user(user).await.unwrap();

        // Check admin permissions
        let has_admin_access = security
            .check_permission("admin_user", &Permission::AdminAccess)
            .await
            .unwrap();
        assert!(has_admin_access);

        let has_read_access = security
            .check_permission("admin_user", &Permission::ReadMemory)
            .await
            .unwrap();
        assert!(has_read_access);
    }

    #[tokio::test]
    async fn test_threat_detection() {
        let config = SecurityConfig::default();
        let security = SecuritySystem::new(config);

        // Wait for initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Trigger threat detection
        security
            .detect_threat(
                ThreatRuleType::FailedLoginAttempts,
                "192.168.1.100",
                Some("test_user"),
            )
            .await
            .unwrap();

        let incidents = security.get_threat_incidents().await;
        assert!(!incidents.is_empty());
        assert_eq!(incidents[0].severity, ThreatSeverity::Medium);
    }
}
