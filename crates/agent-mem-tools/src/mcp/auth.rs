//! MCP 安全认证模块
//!
//! 提供多种认证方式：
//! - API 密钥认证
//! - OAuth 2.0 认证
//! - JWT 令牌认证
//! - 细粒度权限控制
//! - 审计日志

use super::error::{McpError, McpResult};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 认证方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// API 密钥认证
    ApiKey,
    /// OAuth 2.0 认证
    OAuth2,
    /// JWT 令牌认证
    Jwt,
    /// 无认证
    None,
}

/// 权限类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// 列出工具
    ListTools,
    /// 调用工具
    CallTool(String), // 工具名称
    /// 列出资源
    ListResources,
    /// 读取资源
    ReadResource(String), // 资源 URI
    /// 订阅资源
    SubscribeResource(String),
    /// 列出提示词
    ListPrompts,
    /// 获取提示词
    GetPrompt(String), // 提示词名称
    /// 管理员权限
    Admin,
}

/// 用户角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// 管理员
    Admin,
    /// 开发者
    Developer,
    /// 用户
    User,
    /// 只读用户
    ReadOnly,
    /// 自定义角色
    Custom(String),
}

impl Role {
    /// 获取角色的默认权限
    pub fn default_permissions(&self) -> HashSet<Permission> {
        match self {
            Role::Admin => {
                // 管理员拥有所有权限
                vec![
                    Permission::Admin,
                    Permission::ListTools,
                    Permission::ListResources,
                    Permission::ListPrompts,
                ]
                .into_iter()
                .collect()
            }
            Role::Developer => {
                // 开发者可以调用工具和访问资源
                vec![
                    Permission::ListTools,
                    Permission::ListResources,
                    Permission::ListPrompts,
                ]
                .into_iter()
                .collect()
            }
            Role::User => {
                // 普通用户可以列出和使用基本功能
                vec![Permission::ListTools, Permission::ListPrompts]
                    .into_iter()
                    .collect()
            }
            Role::ReadOnly => {
                // 只读用户只能列出
                vec![
                    Permission::ListTools,
                    Permission::ListResources,
                    Permission::ListPrompts,
                ]
                .into_iter()
                .collect()
            }
            Role::Custom(_) => {
                // 自定义角色默认无权限
                HashSet::new()
            }
        }
    }
}

/// JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Issued at (timestamp)
    pub iat: i64,

    /// Expiration time (timestamp)
    pub exp: i64,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// User role
    pub role: Option<String>,

    /// Custom claims
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl JwtClaims {
    /// Create new claims
    pub fn new(user_id: String, issuer: String, audience: String, expiry_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id,
            iat: now,
            exp: now + expiry_seconds,
            iss: issuer,
            aud: audience,
            role: None,
            custom: HashMap::new(),
        }
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// 认证凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// 认证方式
    pub method: AuthMethod,
    /// API 密钥（用于 ApiKey 认证）
    pub api_key: Option<String>,
    /// OAuth 2.0 访问令牌（用于 OAuth2 认证）
    pub access_token: Option<String>,
    /// JWT 令牌（用于 Jwt 认证）
    pub jwt_token: Option<String>,
}

/// 认证上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// 用户 ID
    pub user_id: String,
    /// 用户角色
    pub role: Role,
    /// 用户权限
    pub permissions: HashSet<Permission>,
    /// 认证时间
    pub authenticated_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl AuthContext {
    /// 创建新的认证上下文
    pub fn new(user_id: String, role: Role) -> Self {
        let permissions = role.default_permissions();
        Self {
            user_id,
            role,
            permissions,
            authenticated_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// 检查是否有权限
    pub fn has_permission(&self, permission: &Permission) -> bool {
        // 管理员拥有所有权限
        if self.permissions.contains(&Permission::Admin) {
            return true;
        }

        // 检查具体权限
        match permission {
            Permission::CallTool(_) => {
                // 检查是否有调用工具的通用权限或特定工具权限
                self.permissions.contains(permission)
                    || self
                        .permissions
                        .iter()
                        .any(|p| matches!(p, Permission::CallTool(_)))
            }
            Permission::ReadResource(_) => {
                // 检查是否有读取资源的通用权限或特定资源权限
                self.permissions.contains(permission)
                    || self
                        .permissions
                        .iter()
                        .any(|p| matches!(p, Permission::ReadResource(_)))
            }
            Permission::SubscribeResource(_) => {
                // 检查是否有订阅资源的通用权限或特定资源权限
                self.permissions.contains(permission)
                    || self
                        .permissions
                        .iter()
                        .any(|p| matches!(p, Permission::SubscribeResource(_)))
            }
            Permission::GetPrompt(_) => {
                // 检查是否有获取提示词的通用权限或特定提示词权限
                self.permissions.contains(permission)
                    || self
                        .permissions
                        .iter()
                        .any(|p| matches!(p, Permission::GetPrompt(_)))
            }
            _ => self.permissions.contains(permission),
        }
    }

    /// 添加权限
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// 移除权限
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// 设置过期时间
    pub fn set_expiry(&mut self, duration: Duration) {
        self.expires_at = Some(Utc::now() + duration);
    }
}

/// JWT 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT 密钥
    pub secret: String,
    /// 令牌有效期（秒）
    pub expiry_seconds: i64,
    /// 发行者
    pub issuer: String,
    /// 受众
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "default-secret-change-me".to_string(),
            expiry_seconds: 3600, // 1 小时
            issuer: "agentmem-mcp".to_string(),
            audience: "agentmem-client".to_string(),
        }
    }
}

/// OAuth 2.0 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// 客户端 ID
    pub client_id: String,
    /// 客户端密钥
    pub client_secret: String,
    /// 授权端点
    pub auth_url: String,
    /// 令牌端点
    pub token_url: String,
    /// 重定向 URI
    pub redirect_uri: String,
    /// 作用域
    pub scopes: Vec<String>,
}

impl Default for OAuth2Config {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            auth_url: String::new(),
            token_url: String::new(),
            redirect_uri: String::new(),
            scopes: vec!["read".to_string(), "write".to_string()],
        }
    }
}

/// 认证管理器
pub struct AuthManager {
    /// API 密钥映射（密钥 -> 用户 ID）
    api_keys: Arc<RwLock<HashMap<String, String>>>,

    /// 用户上下文（用户 ID -> 认证上下文）
    contexts: Arc<RwLock<HashMap<String, AuthContext>>>,

    /// JWT 配置
    jwt_config: JwtConfig,

    /// OAuth 2.0 配置
    oauth2_config: OAuth2Config,

    /// 是否启用认证
    enabled: bool,
}

impl AuthManager {
    /// 创建新的认证管理器
    pub fn new(jwt_config: JwtConfig, oauth2_config: OAuth2Config, enabled: bool) -> Self {
        Self {
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            jwt_config,
            oauth2_config,
            enabled,
        }
    }

    /// 创建默认认证管理器（禁用认证）
    pub fn disabled() -> Self {
        Self::new(JwtConfig::default(), OAuth2Config::default(), false)
    }

    /// 注册 API 密钥
    pub async fn register_api_key(&self, api_key: String, user_id: String) -> McpResult<()> {
        info!("Registering API key for user: {}", user_id);
        self.api_keys.write().await.insert(api_key, user_id);
        Ok(())
    }

    /// 撤销 API 密钥
    pub async fn revoke_api_key(&self, api_key: &str) -> McpResult<()> {
        info!("Revoking API key");
        self.api_keys.write().await.remove(api_key);
        Ok(())
    }

    /// 验证 API 密钥
    pub async fn verify_api_key(&self, api_key: &str) -> McpResult<AuthContext> {
        if !self.enabled {
            // 认证未启用，返回默认管理员上下文
            return Ok(AuthContext::new("anonymous".to_string(), Role::Admin));
        }

        let api_keys = self.api_keys.read().await;
        let user_id = api_keys
            .get(api_key)
            .ok_or_else(|| McpError::AuthenticationFailed("Invalid API key".to_string()))?;

        // 获取或创建用户上下文
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(user_id) {
            if context.is_expired() {
                return Err(McpError::AuthenticationFailed(
                    "Authentication expired".to_string(),
                ));
            }
            Ok(context.clone())
        } else {
            drop(contexts);
            // 创建新的上下文
            let context = AuthContext::new(user_id.clone(), Role::Developer);
            self.contexts
                .write()
                .await
                .insert(user_id.clone(), context.clone());
            Ok(context)
        }
    }

    /// 验证 JWT 令牌
    pub async fn verify_jwt(&self, token: &str) -> McpResult<AuthContext> {
        if !self.enabled {
            return Ok(AuthContext::new("anonymous".to_string(), Role::Admin));
        }

        debug!("Verifying JWT token");

        // ✅ 1. 创建验证配置
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.jwt_config.issuer]);
        validation.set_audience(&[&self.jwt_config.audience]);
        validation.validate_exp = true;

        // ✅ 2. 解码和验证 JWT
        let decoding_key = DecodingKey::from_secret(self.jwt_config.secret.as_bytes());
        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation).map_err(|e| {
            warn!("JWT verification failed: {}", e);
            McpError::AuthenticationFailed(format!("Invalid JWT: {e}"))
        })?;

        let claims = token_data.claims;

        // ✅ 3. 检查过期时间（双重检查）
        if claims.is_expired() {
            return Err(McpError::AuthenticationFailed("JWT expired".to_string()));
        }

        // ✅ 4. 提取用户信息
        let user_id = claims.sub.clone();

        // ✅ 5. 解析角色
        let role = if let Some(role_str) = claims.role {
            match role_str.as_str() {
                "admin" => Role::Admin,
                "developer" => Role::Developer,
                "user" => Role::User,
                "readonly" => Role::ReadOnly,
                custom => Role::Custom(custom.to_string()),
            }
        } else {
            Role::Developer // 默认角色
        };

        // ✅ 6. 获取或创建用户上下文
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(&user_id) {
            if context.is_expired() {
                return Err(McpError::AuthenticationFailed(
                    "Authentication expired".to_string(),
                ));
            }
            Ok(context.clone())
        } else {
            drop(contexts);

            // 创建新的上下文
            let mut context = AuthContext::new(user_id.clone(), role);

            // 设置过期时间（使用 JWT 的过期时间）
            let expiry_duration = Duration::seconds(claims.exp - Utc::now().timestamp());
            context.set_expiry(expiry_duration);

            // 添加自定义元数据
            for (key, value) in claims.custom {
                if let Some(s) = value.as_str() {
                    context.metadata.insert(key, s.to_string());
                }
            }

            self.contexts
                .write()
                .await
                .insert(user_id.clone(), context.clone());

            info!("Created new auth context for user: {}", user_id);
            Ok(context)
        }
    }

    /// 生成 JWT 令牌
    pub fn generate_jwt(&self, user_id: String, role: Role) -> McpResult<String> {
        // 创建 claims
        let mut claims = JwtClaims::new(
            user_id,
            self.jwt_config.issuer.clone(),
            self.jwt_config.audience.clone(),
            self.jwt_config.expiry_seconds,
        );

        // 设置角色
        claims.role = Some(match role {
            Role::Admin => "admin".to_string(),
            Role::Developer => "developer".to_string(),
            Role::User => "user".to_string(),
            Role::ReadOnly => "readonly".to_string(),
            Role::Custom(s) => s,
        });

        // 编码 JWT
        let encoding_key = EncodingKey::from_secret(self.jwt_config.secret.as_bytes());
        let token = encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| McpError::AuthenticationFailed(format!("Failed to generate JWT: {e}")))?;

        Ok(token)
    }

    /// 验证 OAuth 2.0 访问令牌
    pub async fn verify_oauth2(&self, access_token: &str) -> McpResult<AuthContext> {
        if !self.enabled {
            return Ok(AuthContext::new("anonymous".to_string(), Role::Admin));
        }

        debug!("Verifying OAuth 2.0 access token");

        // ✅ 1. 调用 OAuth 2.0 提供商的验证端点
        // 这里使用通用的 token introspection endpoint (RFC 7662)
        let client = reqwest::Client::new();

        let introspection_url = format!("{}/introspect", self.oauth2_config.token_url);

        let response = client
            .post(&introspection_url)
            .basic_auth(
                &self.oauth2_config.client_id,
                Some(&self.oauth2_config.client_secret),
            )
            .form(&[("token", access_token)])
            .send()
            .await
            .map_err(|e| {
                warn!("OAuth2 introspection request failed: {}", e);
                McpError::AuthenticationFailed(format!("OAuth2 verification failed: {e}"))
            })?;

        // ✅ 2. 解析响应
        #[derive(Deserialize)]
        struct IntrospectionResponse {
            active: bool,
            sub: Option<String>,
            exp: Option<i64>,
            scope: Option<String>,
            #[serde(flatten)]
            extra: HashMap<String, serde_json::Value>,
        }

        let introspection: IntrospectionResponse = response.json().await.map_err(|e| {
            warn!("Failed to parse OAuth2 introspection response: {}", e);
            McpError::AuthenticationFailed("Invalid OAuth2 response".to_string())
        })?;

        // ✅ 3. 验证令牌有效性
        if !introspection.active {
            return Err(McpError::AuthenticationFailed(
                "OAuth2 token is not active".to_string(),
            ));
        }

        // ✅ 4. 提取用户信息
        let user_id = introspection.sub.ok_or_else(|| {
            McpError::AuthenticationFailed("OAuth2 token missing subject".to_string())
        })?;

        // ✅ 5. 检查过期时间
        if let Some(exp) = introspection.exp {
            if Utc::now().timestamp() > exp {
                return Err(McpError::AuthenticationFailed(
                    "OAuth2 token expired".to_string(),
                ));
            }
        }

        // ✅ 6. 获取或创建用户上下文
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(&user_id) {
            if context.is_expired() {
                return Err(McpError::AuthenticationFailed(
                    "OAuth2 token expired".to_string(),
                ));
            }
            Ok(context.clone())
        } else {
            drop(contexts);

            // 创建新的上下文
            let mut context = AuthContext::new(user_id.clone(), Role::User);

            // 设置过期时间
            if let Some(exp) = introspection.exp {
                let expiry_duration = Duration::seconds(exp - Utc::now().timestamp());
                context.set_expiry(expiry_duration);
            } else {
                context.set_expiry(Duration::seconds(3600)); // 默认 1 小时
            }

            // 添加 scope 到元数据
            if let Some(scope) = introspection.scope {
                context.metadata.insert("scope".to_string(), scope);
            }

            // 添加其他元数据
            for (key, value) in introspection.extra {
                if let Some(s) = value.as_str() {
                    context.metadata.insert(key, s.to_string());
                }
            }

            self.contexts
                .write()
                .await
                .insert(user_id.clone(), context.clone());

            info!("Created new OAuth2 auth context for user: {}", user_id);
            Ok(context)
        }
    }

    /// 验证凭证
    pub async fn authenticate(&self, credentials: &Credentials) -> McpResult<AuthContext> {
        match credentials.method {
            AuthMethod::ApiKey => {
                let api_key = credentials
                    .api_key
                    .as_ref()
                    .ok_or_else(|| McpError::AuthenticationFailed("Missing API key".to_string()))?;
                self.verify_api_key(api_key).await
            }
            AuthMethod::Jwt => {
                let jwt_token = credentials.jwt_token.as_ref().ok_or_else(|| {
                    McpError::AuthenticationFailed("Missing JWT token".to_string())
                })?;
                self.verify_jwt(jwt_token).await
            }
            AuthMethod::OAuth2 => {
                let access_token = credentials.access_token.as_ref().ok_or_else(|| {
                    McpError::AuthenticationFailed("Missing OAuth 2.0 access token".to_string())
                })?;
                self.verify_oauth2(access_token).await
            }
            AuthMethod::None => {
                if self.enabled {
                    Err(McpError::AuthenticationFailed(
                        "Authentication required".to_string(),
                    ))
                } else {
                    Ok(AuthContext::new("anonymous".to_string(), Role::Admin))
                }
            }
        }
    }

    /// 检查权限
    pub async fn check_permission(
        &self,
        user_id: &str,
        permission: &Permission,
    ) -> McpResult<bool> {
        if !self.enabled {
            return Ok(true);
        }

        let contexts = self.contexts.read().await;
        let context = contexts
            .get(user_id)
            .ok_or_else(|| McpError::AuthenticationFailed("User not authenticated".to_string()))?;

        if context.is_expired() {
            return Err(McpError::AuthenticationFailed(
                "Authentication expired".to_string(),
            ));
        }

        Ok(context.has_permission(permission))
    }

    /// 授予权限
    pub async fn grant_permission(&self, user_id: &str, permission: Permission) -> McpResult<()> {
        info!("Granting permission {:?} to user: {}", permission, user_id);
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(user_id)
            .ok_or_else(|| McpError::AuthenticationFailed("User not found".to_string()))?;

        context.add_permission(permission);
        Ok(())
    }

    /// 撤销权限
    pub async fn revoke_permission(&self, user_id: &str, permission: &Permission) -> McpResult<()> {
        info!(
            "Revoking permission {:?} from user: {}",
            permission, user_id
        );
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(user_id)
            .ok_or_else(|| McpError::AuthenticationFailed("User not found".to_string()))?;

        context.remove_permission(permission);
        Ok(())
    }

    /// 更新用户角色
    pub async fn update_role(&self, user_id: &str, role: Role) -> McpResult<()> {
        info!("Updating role for user {}: {:?}", user_id, role);
        let mut contexts = self.contexts.write().await;

        // 如果用户不存在，创建新的上下文
        if !contexts.contains_key(user_id) {
            let context = AuthContext::new(user_id.to_string(), role.clone());
            contexts.insert(user_id.to_string(), context);
        } else {
            let context = contexts.get_mut(user_id).unwrap();
            context.role = role.clone();
            context.permissions = role.default_permissions();
        }

        Ok(())
    }

    /// 清理过期的上下文
    pub async fn cleanup_expired(&self) {
        let mut contexts = self.contexts.write().await;
        contexts.retain(|user_id, context| {
            if context.is_expired() {
                warn!("Removing expired context for user: {}", user_id);
                false
            } else {
                true
            }
        });
    }
}

/// 审计事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// 认证成功
    AuthenticationSuccess,
    /// 认证失败
    AuthenticationFailure,
    /// 权限检查
    PermissionCheck,
    /// 权限授予
    PermissionGranted,
    /// 权限撤销
    PermissionRevoked,
    /// 工具调用
    ToolCall,
    /// 资源访问
    ResourceAccess,
    /// 配置更改
    ConfigChange,
}

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件 ID
    pub id: String,
    /// 事件类型
    pub event_type: AuditEventType,
    /// 用户 ID
    pub user_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 操作
    pub action: String,
    /// 资源
    pub resource: Option<String>,
    /// 结果（成功/失败）
    pub success: bool,
    /// 详细信息
    pub details: HashMap<String, String>,
    /// IP 地址
    pub ip_address: Option<String>,
}

impl AuditEvent {
    /// 创建新的审计事件
    pub fn new(event_type: AuditEventType, user_id: String, action: String, success: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            user_id,
            timestamp: Utc::now(),
            action,
            resource: None,
            success,
            details: HashMap::new(),
            ip_address: None,
        }
    }

    /// 设置资源
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    /// 设置 IP 地址
    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// 添加详细信息
    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// 审计日志管理器
pub struct AuditLogger {
    /// 审计事件存储
    events: Arc<RwLock<Vec<AuditEvent>>>,
    /// 最大事件数量
    max_events: usize,
    /// 是否启用
    enabled: bool,
}

impl AuditLogger {
    /// 创建新的审计日志管理器
    pub fn new(max_events: usize, enabled: bool) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
            enabled,
        }
    }

    /// 创建禁用的审计日志管理器
    pub fn disabled() -> Self {
        Self::new(0, false)
    }

    /// 记录审计事件
    pub async fn log(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        info!(
            "Audit: {} - {} by {} ({})",
            event.event_type_str(),
            event.action,
            event.user_id,
            if event.success { "SUCCESS" } else { "FAILURE" }
        );

        let mut events = self.events.write().await;
        events.push(event);

        // 限制事件数量
        if events.len() > self.max_events {
            events.remove(0);
        }
    }

    /// 查询审计事件
    pub async fn query(
        &self,
        user_id: Option<&str>,
        event_type: Option<&AuditEventType>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| {
                if let Some(uid) = user_id {
                    if e.user_id != uid {
                        return false;
                    }
                }
                if let Some(et) = event_type {
                    if &e.event_type != et {
                        return false;
                    }
                }
                if let Some(st) = start_time {
                    if e.timestamp < st {
                        return false;
                    }
                }
                if let Some(et) = end_time {
                    if e.timestamp > et {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// 获取所有审计事件
    pub async fn get_all(&self) -> Vec<AuditEvent> {
        self.events.read().await.clone()
    }

    /// 清空审计日志
    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

impl AuditEventType {
    /// 获取事件类型字符串
    fn as_str(&self) -> &str {
        match self {
            AuditEventType::AuthenticationSuccess => "AUTH_SUCCESS",
            AuditEventType::AuthenticationFailure => "AUTH_FAILURE",
            AuditEventType::PermissionCheck => "PERMISSION_CHECK",
            AuditEventType::PermissionGranted => "PERMISSION_GRANTED",
            AuditEventType::PermissionRevoked => "PERMISSION_REVOKED",
            AuditEventType::ToolCall => "TOOL_CALL",
            AuditEventType::ResourceAccess => "RESOURCE_ACCESS",
            AuditEventType::ConfigChange => "CONFIG_CHANGE",
        }
    }
}

impl AuditEvent {
    /// 获取事件类型字符串
    fn event_type_str(&self) -> &str {
        self.event_type.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_manager_creation() {
        let manager = AuthManager::disabled();
        assert!(!manager.enabled);
    }

    #[tokio::test]
    async fn test_api_key_registration() {
        let manager = AuthManager::new(JwtConfig::default(), OAuth2Config::default(), true);
        manager
            .register_api_key("test-key".to_string(), "user1".to_string())
            .await
            .unwrap();

        let context = manager.verify_api_key("test-key").await.unwrap();
        assert_eq!(context.user_id, "user1");
    }

    #[tokio::test]
    async fn test_permission_check() {
        let manager = AuthManager::new(JwtConfig::default(), OAuth2Config::default(), true);
        manager
            .register_api_key("test-key".to_string(), "user1".to_string())
            .await
            .unwrap();

        let context = manager.verify_api_key("test-key").await.unwrap();
        assert!(context.has_permission(&Permission::ListTools));
    }

    #[tokio::test]
    async fn test_role_permissions() {
        let admin_role = Role::Admin;
        let admin_perms = admin_role.default_permissions();
        assert!(admin_perms.contains(&Permission::Admin));

        let user_role = Role::User;
        let user_perms = user_role.default_permissions();
        assert!(user_perms.contains(&Permission::ListTools));
        assert!(!user_perms.contains(&Permission::Admin));
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new(100, true);
        let event = AuditEvent::new(
            AuditEventType::AuthenticationSuccess,
            "user1".to_string(),
            "login".to_string(),
            true,
        );

        logger.log(event).await;

        let events = logger.get_all().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].user_id, "user1");
    }

    #[tokio::test]
    async fn test_audit_query() {
        let logger = AuditLogger::new(100, true);

        // 记录多个事件
        logger
            .log(AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                "user1".to_string(),
                "login".to_string(),
                true,
            ))
            .await;

        logger
            .log(AuditEvent::new(
                AuditEventType::ToolCall,
                "user1".to_string(),
                "call_tool".to_string(),
                true,
            ))
            .await;

        logger
            .log(AuditEvent::new(
                AuditEventType::AuthenticationSuccess,
                "user2".to_string(),
                "login".to_string(),
                true,
            ))
            .await;

        // 查询 user1 的事件
        let user1_events = logger.query(Some("user1"), None, None, None).await;
        assert_eq!(user1_events.len(), 2);

        // 查询认证事件
        let auth_events = logger
            .query(
                None,
                Some(&AuditEventType::AuthenticationSuccess),
                None,
                None,
            )
            .await;
        assert_eq!(auth_events.len(), 2);
    }

    #[tokio::test]
    async fn test_context_expiry() {
        let mut context = AuthContext::new("user1".to_string(), Role::User);
        assert!(!context.is_expired());

        context.set_expiry(Duration::seconds(-1)); // 设置为已过期
        assert!(context.is_expired());
    }
}
