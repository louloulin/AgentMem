//! Enterprise Security Management System
//!
//! Comprehensive enterprise-grade security features including:
//! - RBAC (Role-Based Access Control)
//! - AES-256 End-to-End Encryption
//! - JWT + OAuth2 Authentication
//! - Complete Audit Logging
//! - Data Masking and PII Protection

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use agent_mem_traits::{AgentMemError, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Enterprise security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseSecurityConfig {
    /// Enable RBAC
    pub enable_rbac: bool,
    /// Enable end-to-end encryption
    pub enable_e2e_encryption: bool,
    /// Enable JWT authentication
    pub enable_jwt_auth: bool,
    /// Enable OAuth2 authentication
    pub enable_oauth2_auth: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Enable data masking
    pub enable_data_masking: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT token expiry hours
    pub jwt_expiry_hours: i64,
    /// OAuth2 client ID
    pub oauth2_client_id: String,
    /// OAuth2 client secret
    pub oauth2_client_secret: String,
    /// Encryption key (base64 encoded)
    pub encryption_key: String,
    /// Maximum failed login attempts
    pub max_failed_attempts: u32,
    /// Account lockout duration minutes
    pub lockout_duration_minutes: u32,
    /// Session timeout minutes
    pub session_timeout_minutes: u32,
    /// Enable IP whitelisting
    pub enable_ip_whitelisting: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
}

impl Default for EnterpriseSecurityConfig {
    fn default() -> Self {
        Self {
            enable_rbac: true,
            enable_e2e_encryption: true,
            enable_jwt_auth: true,
            enable_oauth2_auth: false,
            enable_audit_logging: true,
            enable_data_masking: true,
            jwt_secret: "default-secret-key-change-in-production".to_string(),
            jwt_expiry_hours: 24,
            oauth2_client_id: String::new(),
            oauth2_client_secret: String::new(),
            encryption_key: base64::encode([0u8; 32]), // Default key, should be changed
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            session_timeout_minutes: 60,
            enable_ip_whitelisting: false,
            allowed_ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
        }
    }
}

/// User role definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
    /// Role ID
    pub id: String,
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Permissions
    pub permissions: HashSet<Permission>,
    /// Role hierarchy level (higher = more privileged)
    pub level: u32,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Permission enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // Memory operations
    ReadMemory,
    WriteMemory,
    DeleteMemory,
    SearchMemory,
    ExportMemory,
    ImportMemory,

    // User management
    CreateUser,
    ReadUser,
    UpdateUser,
    DeleteUser,
    ManageRoles,

    // System administration
    SystemAdmin,
    ViewAuditLogs,
    ManageConfig,
    ViewMetrics,
    ManageBackups,

    // Security operations
    ManageSecurity,
    ViewSecurityLogs,
    ManageEncryption,

    // API access
    ApiAccess,
    AdminApiAccess,
}

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Password hash
    pub password_hash: String,
    /// Assigned roles
    pub roles: Vec<String>,
    /// Account status
    pub active: bool,
    /// Failed login attempts
    pub failed_attempts: u32,
    /// Account locked until
    pub locked_until: Option<DateTime<Utc>>,
    /// Last login
    pub last_login: Option<DateTime<Utc>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// Issued at
    pub iat: i64,
    /// Expiration time
    pub exp: i64,
    /// Session ID
    pub session_id: String,
}

/// OAuth2 token response
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    // Authentication events
    LoginSuccess,
    LoginFailure,
    Logout,
    TokenGenerated,
    TokenValidated,
    TokenExpired,

    // Authorization events
    AccessGranted,
    AccessDenied,
    RoleAssigned,
    RoleRevoked,

    // Memory operations
    MemoryCreated,
    MemoryRead,
    MemoryUpdated,
    MemoryDeleted,
    MemorySearched,
    MemoryExported,
    MemoryImported,

    // System events
    ConfigChanged,
    BackupCreated,
    BackupRestored,
    SystemStarted,
    SystemStopped,

    // Security events
    SecurityViolation,
    EncryptionKeyRotated,
    SuspiciousActivity,
    DataMasked,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Entry ID
    pub id: String,
    /// Event type
    pub event_type: AuditEventType,
    /// User ID
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Resource type
    pub resource_type: Option<String>,
    /// Action performed
    pub action: String,
    /// Result (success/failure)
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
    /// Risk score (0-100)
    pub risk_score: u8,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// JWT token
    pub token: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// IP address
    pub ip_address: String,
    /// User agent
    pub user_agent: String,
    /// Session active
    pub active: bool,
}

/// Enterprise Security Manager
pub struct EnterpriseSecurityManager {
    /// Configuration
    config: EnterpriseSecurityConfig,
    /// Roles storage
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// Users storage
    users: Arc<RwLock<HashMap<String, UserAccount>>>,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    /// Audit logs
    audit_logs: Arc<RwLock<Vec<AuditLogEntry>>>,
    /// JWT encoding key
    jwt_encoding_key: EncodingKey,
    /// JWT decoding key
    jwt_decoding_key: DecodingKey,
    /// Encryption cipher
    cipher: Aes256Gcm,
}

impl EnterpriseSecurityManager {
    /// Create new enterprise security manager
    pub fn new(config: EnterpriseSecurityConfig) -> Result<Self> {
        info!(
            "Initializing EnterpriseSecurityManager with config: {:?}",
            config
        );

        // Initialize JWT keys
        let jwt_encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let jwt_decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());

        // Initialize encryption cipher
        let encryption_key = base64::decode(&config.encryption_key)
            .map_err(|e| AgentMemError::config_error(format!("Invalid encryption key: {e}")))?;

        if encryption_key.len() != 32 {
            return Err(AgentMemError::config_error(
                "Encryption key must be 32 bytes",
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
        let cipher = Aes256Gcm::new(key);

        let manager = Self {
            config,
            roles: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            audit_logs: Arc::new(RwLock::new(Vec::new())),
            jwt_encoding_key,
            jwt_decoding_key,
            cipher,
        };

        info!("EnterpriseSecurityManager initialized successfully");
        Ok(manager)
    }

    /// Initialize with default roles and admin user
    pub async fn initialize_defaults(&self) -> Result<()> {
        info!("Initializing default roles and admin user");

        // Create default roles
        self.create_default_roles().await?;

        // Create default admin user
        self.create_default_admin().await?;

        info!("Default roles and admin user created successfully");
        Ok(())
    }

    /// Create default system roles
    async fn create_default_roles(&self) -> Result<()> {
        let roles_to_create = vec![
            Role {
                id: "admin".to_string(),
                name: "Administrator".to_string(),
                description: "Full system administrator with all permissions".to_string(),
                permissions: [
                    Permission::ReadMemory,
                    Permission::WriteMemory,
                    Permission::DeleteMemory,
                    Permission::SearchMemory,
                    Permission::ExportMemory,
                    Permission::ImportMemory,
                    Permission::CreateUser,
                    Permission::ReadUser,
                    Permission::UpdateUser,
                    Permission::DeleteUser,
                    Permission::ManageRoles,
                    Permission::SystemAdmin,
                    Permission::ViewAuditLogs,
                    Permission::ManageConfig,
                    Permission::ViewMetrics,
                    Permission::ManageBackups,
                    Permission::ManageSecurity,
                    Permission::ViewSecurityLogs,
                    Permission::ManageEncryption,
                    Permission::ApiAccess,
                    Permission::AdminApiAccess,
                ]
                .iter()
                .cloned()
                .collect(),
                level: 100,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: "user".to_string(),
                name: "Regular User".to_string(),
                description: "Regular user with basic memory operations".to_string(),
                permissions: [
                    Permission::ReadMemory,
                    Permission::WriteMemory,
                    Permission::SearchMemory,
                    Permission::ApiAccess,
                ]
                .iter()
                .cloned()
                .collect(),
                level: 10,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Role {
                id: "readonly".to_string(),
                name: "Read Only".to_string(),
                description: "Read-only access to memories".to_string(),
                permissions: [
                    Permission::ReadMemory,
                    Permission::SearchMemory,
                    Permission::ApiAccess,
                ]
                .iter()
                .cloned()
                .collect(),
                level: 5,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        let mut roles = self.roles.write().await;
        for role in roles_to_create {
            roles.insert(role.id.clone(), role);
        }

        Ok(())
    }

    /// Create default admin user
    async fn create_default_admin(&self) -> Result<()> {
        let admin_user = UserAccount {
            id: "admin".to_string(),
            username: "admin".to_string(),
            email: "admin@agentmem.local".to_string(),
            password_hash: self.hash_password("admin123").await?,
            roles: vec!["admin".to_string()],
            active: true,
            failed_attempts: 0,
            locked_until: None,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let mut users = self.users.write().await;
        users.insert(admin_user.id.clone(), admin_user);

        Ok(())
    }

    /// Hash password using bcrypt
    async fn hash_password(&self, password: &str) -> Result<String> {
        use bcrypt::{hash, DEFAULT_COST};
        hash(password, DEFAULT_COST)
            .map_err(|e| AgentMemError::auth_error(format!("Password hashing failed: {e}")))
    }

    /// Verify password against hash
    async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        use bcrypt::verify;
        verify(password, hash)
            .map_err(|e| AgentMemError::auth_error(format!("Password verification failed: {e}")))
    }

    /// Authenticate user and create session
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<UserSession> {
        info!("Authentication attempt for user: {}", username);

        // Check IP whitelist if enabled
        if self.config.enable_ip_whitelisting && !self.is_ip_allowed(ip_address) {
            warn!("Authentication failed: IP {} not in whitelist", ip_address);
            self.log_audit_event(
                AuditEventType::LoginFailure,
                None,
                None,
                Some(ip_address.to_string()),
                Some(user_agent.to_string()),
                None,
                None,
                "authenticate".to_string(),
                false,
                Some("IP not in whitelist".to_string()),
                HashMap::new(),
                80,
            )
            .await?;
            return Err(AgentMemError::auth_error("IP address not allowed"));
        }

        let mut users = self.users.write().await;
        let user = users
            .values_mut()
            .find(|u| u.username == username && u.active)
            .ok_or_else(|| {
                warn!(
                    "Authentication failed: User {} not found or inactive",
                    username
                );
                AgentMemError::auth_error("Invalid credentials")
            })?;

        // Check if account is locked
        if let Some(locked_until) = user.locked_until {
            if Utc::now() < locked_until {
                warn!(
                    "Authentication failed: Account {} is locked until {}",
                    username, locked_until
                );
                self.log_audit_event(
                    AuditEventType::LoginFailure,
                    Some(user.id.clone()),
                    None,
                    Some(ip_address.to_string()),
                    Some(user_agent.to_string()),
                    None,
                    None,
                    "authenticate".to_string(),
                    false,
                    Some("Account locked".to_string()),
                    HashMap::new(),
                    90,
                )
                .await?;
                return Err(AgentMemError::auth_error("Account is locked"));
            } else {
                // Unlock account if lock period has expired
                user.locked_until = None;
                user.failed_attempts = 0;
            }
        }

        // Verify password
        let password_valid = self.verify_password(password, &user.password_hash).await?;

        if !password_valid {
            user.failed_attempts += 1;

            // Lock account if too many failed attempts
            if user.failed_attempts >= self.config.max_failed_attempts {
                user.locked_until = Some(
                    Utc::now() + Duration::minutes(self.config.lockout_duration_minutes as i64),
                );
                warn!(
                    "Account {} locked due to {} failed attempts",
                    username, user.failed_attempts
                );
            }

            warn!(
                "Authentication failed: Invalid password for user {}",
                username
            );
            self.log_audit_event(
                AuditEventType::LoginFailure,
                Some(user.id.clone()),
                None,
                Some(ip_address.to_string()),
                Some(user_agent.to_string()),
                None,
                None,
                "authenticate".to_string(),
                false,
                Some("Invalid password".to_string()),
                HashMap::new(),
                70,
            )
            .await?;
            return Err(AgentMemError::auth_error("Invalid credentials"));
        }

        // Reset failed attempts on successful authentication
        user.failed_attempts = 0;
        user.last_login = Some(Utc::now());

        // Generate JWT token
        let session_id = Uuid::new_v4().to_string();
        let token = self
            .generate_jwt_token(&user.id, &user.username, &user.roles, &session_id)
            .await?;

        // Create session
        let session = UserSession {
            id: session_id.clone(),
            user_id: user.id.clone(),
            token,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at: Utc::now() + Duration::hours(self.config.jwt_expiry_hours),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            active: true,
        };

        // Store session
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        info!("Authentication successful for user: {}", username);
        self.log_audit_event(
            AuditEventType::LoginSuccess,
            Some(user.id.clone()),
            Some(session_id),
            Some(ip_address.to_string()),
            Some(user_agent.to_string()),
            None,
            None,
            "authenticate".to_string(),
            true,
            None,
            HashMap::new(),
            10,
        )
        .await?;

        Ok(session)
    }

    /// Generate JWT token
    async fn generate_jwt_token(
        &self,
        user_id: &str,
        username: &str,
        roles: &[String],
        session_id: &str,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.jwt_expiry_hours);

        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles: roles.to_vec(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            session_id: session_id.to_string(),
        };

        encode(&Header::default(), &claims, &self.jwt_encoding_key)
            .map_err(|e| AgentMemError::auth_error(format!("JWT generation failed: {e}")))
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(token, &self.jwt_decoding_key, &Validation::default())
            .map_err(|e| AgentMemError::auth_error(format!("JWT validation failed: {e}")))?;

        let claims = token_data.claims;

        // Check if session is still active
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&claims.session_id)
            .ok_or_else(|| AgentMemError::auth_error("Session not found"))?;

        if !session.active || Utc::now() > session.expires_at {
            return Err(AgentMemError::auth_error("Session expired or inactive"));
        }

        Ok(claims)
    }

    /// Check if IP address is allowed
    fn is_ip_allowed(&self, ip_address: &str) -> bool {
        self.config.allowed_ips.contains(&ip_address.to_string())
    }

    /// Check if user has permission
    pub async fn check_permission(&self, user_id: &str, permission: &Permission) -> Result<bool> {
        if !self.config.enable_rbac {
            return Ok(true);
        }

        let users = self.users.read().await;
        let user = users
            .get(user_id)
            .ok_or_else(|| AgentMemError::auth_error("User not found"))?;

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

    /// Encrypt data using AES-256-GCM
    pub async fn encrypt_data(&self, data: &str) -> Result<String> {
        if !self.config.enable_e2e_encryption {
            return Ok(data.to_string());
        }

        let mut rng = thread_rng();
        let nonce_bytes: [u8; 12] = rng.gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| AgentMemError::internal_error(format!("Encryption failed: {e}")))?;

        // Combine nonce and ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(base64::encode(&result))
    }

    /// Decrypt data using AES-256-GCM
    pub async fn decrypt_data(&self, encrypted_data: &str) -> Result<String> {
        if !self.config.enable_e2e_encryption {
            return Ok(encrypted_data.to_string());
        }

        let data = base64::decode(encrypted_data)
            .map_err(|e| AgentMemError::internal_error(format!("Base64 decode failed: {e}")))?;

        if data.len() < 12 {
            return Err(AgentMemError::validation_error("Invalid encrypted data"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AgentMemError::internal_error(format!("Decryption failed: {e}")))?;

        String::from_utf8(plaintext)
            .map_err(|e| AgentMemError::internal_error(format!("UTF-8 decode failed: {e}")))
    }

    /// Mask sensitive data (PII protection)
    pub async fn mask_sensitive_data(&self, data: &str) -> Result<String> {
        if !self.config.enable_data_masking {
            return Ok(data.to_string());
        }

        let mut masked_data = data.to_string();

        // Mask email addresses
        let email_regex = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
            .map_err(|e| AgentMemError::internal_error(format!("Regex error: {e}")))?;
        masked_data = email_regex
            .replace_all(&masked_data, "***@***.***")
            .to_string();

        // Mask phone numbers
        let phone_regex = regex::Regex::new(r"\b\d{3}-\d{3}-\d{4}\b")
            .map_err(|e| AgentMemError::internal_error(format!("Regex error: {e}")))?;
        masked_data = phone_regex
            .replace_all(&masked_data, "***-***-****")
            .to_string();

        // Mask credit card numbers
        let cc_regex = regex::Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b")
            .map_err(|e| AgentMemError::internal_error(format!("Regex error: {e}")))?;
        masked_data = cc_regex
            .replace_all(&masked_data, "**** **** **** ****")
            .to_string();

        // Mask SSN
        let ssn_regex = regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
            .map_err(|e| AgentMemError::internal_error(format!("Regex error: {e}")))?;
        masked_data = ssn_regex
            .replace_all(&masked_data, "***-**-****")
            .to_string();

        Ok(masked_data)
    }

    /// Log audit event
    async fn log_audit_event(
        &self,
        event_type: AuditEventType,
        user_id: Option<String>,
        session_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        resource_id: Option<String>,
        resource_type: Option<String>,
        action: String,
        success: bool,
        error_message: Option<String>,
        details: HashMap<String, serde_json::Value>,
        risk_score: u8,
    ) -> Result<()> {
        if !self.config.enable_audit_logging {
            return Ok(());
        }

        let entry = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            event_type,
            user_id,
            session_id,
            ip_address,
            user_agent,
            resource_id,
            resource_type,
            action,
            success,
            error_message,
            details,
            risk_score,
            timestamp: Utc::now(),
        };

        let mut audit_logs = self.audit_logs.write().await;
        audit_logs.push(entry);

        // Keep only last 10000 entries to prevent memory issues
        if audit_logs.len() > 10000 {
            audit_logs.drain(0..1000);
        }

        Ok(())
    }

    /// Get audit logs
    pub async fn get_audit_logs(&self, limit: Option<usize>) -> Result<Vec<AuditLogEntry>> {
        let audit_logs = self.audit_logs.read().await;
        let limit = limit.unwrap_or(100);

        Ok(audit_logs.iter().rev().take(limit).cloned().collect())
    }

    /// Create new user
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        roles: Vec<String>,
    ) -> Result<String> {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = self.hash_password(password).await?;

        let user = UserAccount {
            id: user_id.clone(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            roles,
            active: true,
            failed_attempts: 0,
            locked_until: None,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };

        let mut users = self.users.write().await;
        users.insert(user_id.clone(), user);

        info!("User created: {}", username);
        Ok(user_id)
    }

    /// Logout user
    pub async fn logout(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.active = false;

            self.log_audit_event(
                AuditEventType::Logout,
                Some(session.user_id.clone()),
                Some(session_id.to_string()),
                Some(session.ip_address.clone()),
                Some(session.user_agent.clone()),
                None,
                None,
                "logout".to_string(),
                true,
                None,
                HashMap::new(),
                10,
            )
            .await?;
        }

        Ok(())
    }
}
