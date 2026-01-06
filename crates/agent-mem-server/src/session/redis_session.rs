//! Redis Session Store
//!
//! Implements session storage using Redis for stateless service design.
//! This enables horizontal scaling by moving session state to shared storage.

use crate::error::{ServerError, ServerResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

#[cfg(feature = "redis-cache")]
use redis::{AsyncCommands, Client};

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session ID
    pub session_id: String,
    /// User ID
    pub user_id: String,
    /// Organization ID
    pub organization_id: String,
    /// Created timestamp
    pub created_at: i64,
    /// Last accessed timestamp
    pub last_accessed: i64,
    /// Expires at timestamp
    pub expires_at: i64,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl SessionData {
    /// Create a new session
    pub fn new(user_id: String, organization_id: String, ttl_secs: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            organization_id,
            created_at: now,
            last_accessed: now,
            expires_at: now + ttl_secs as i64,
            metadata: HashMap::new(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        now > self.expires_at
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
    }
}

/// Redis session store
pub struct RedisSessionStore {
    /// Redis client (if Redis is enabled)
    #[cfg(feature = "redis-cache")]
    redis_client: Option<Arc<Client>>,
    /// Session key prefix
    key_prefix: String,
    /// Default TTL in seconds
    default_ttl_secs: u64,
    /// In-memory fallback (for single instance)
    #[cfg(not(feature = "redis-cache"))]
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl RedisSessionStore {
    /// Create a new Redis session store
    #[cfg(feature = "redis-cache")]
    pub fn new(
        redis_client: Option<Arc<Client>>,
        key_prefix: impl Into<String>,
        default_ttl_secs: u64,
    ) -> Self {
        Self {
            redis_client,
            key_prefix: key_prefix.into(),
            default_ttl_secs,
        }
    }

    /// Create a new session store without Redis
    #[cfg(not(feature = "redis-cache"))]
    pub fn new(key_prefix: impl Into<String>, default_ttl_secs: u64) -> Self {
        Self {
            key_prefix: key_prefix.into(),
            default_ttl_secs,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate session key
    fn session_key(&self, session_id: &str) -> String {
        format!("{}session:{}", self.key_prefix, session_id)
    }

    /// Create a new session
    #[cfg(feature = "redis-cache")]
    pub async fn create_session(
        &self,
        user_id: String,
        organization_id: String,
        ttl_secs: Option<u64>,
    ) -> ServerResult<SessionData> {
        let ttl = ttl_secs.unwrap_or(self.default_ttl_secs);
        let mut session = SessionData::new(user_id, organization_id, ttl);

        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to get Redis connection: {e}"))
                })?;

            let key = self.session_key(&session.session_id);
            let value = serde_json::to_string(&session).map_err(|e| {
                ServerError::internal_error(format!("Failed to serialize session: {e}"))
            })?;

            conn.set_ex(&key, &value, ttl as usize)
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to store session in Redis: {e}"))
                })?;
        }

        Ok(session)
    }

    /// Create a new session (without Redis)
    #[cfg(not(feature = "redis-cache"))]
    pub async fn create_session(
        &self,
        user_id: String,
        organization_id: String,
        ttl_secs: Option<u64>,
    ) -> ServerResult<SessionData> {
        let ttl = ttl_secs.unwrap_or(self.default_ttl_secs);
        let session = SessionData::new(user_id, organization_id, ttl);

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());

        Ok(session)
    }

    /// Get session by ID
    #[cfg(feature = "redis-cache")]
    pub async fn get_session(&self, session_id: &str) -> ServerResult<Option<SessionData>> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to get Redis connection: {e}"))
                })?;

            let key = self.session_key(session_id);
            let value: Option<String> = conn.get(&key).await.map_err(|e| {
                ServerError::internal_error(format!("Failed to get session from Redis: {e}"))
            })?;

            if let Some(value) = value {
                let session: SessionData = serde_json::from_str(&value).map_err(|e| {
                    ServerError::internal_error(format!("Failed to deserialize session: {e}"))
                })?;

                if session.is_expired() {
                    return Ok(None);
                }

                Ok(Some(session))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get session by ID (without Redis)
    #[cfg(not(feature = "redis-cache"))]
    pub async fn get_session(&self, session_id: &str) -> ServerResult<Option<SessionData>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// Update session
    #[cfg(feature = "redis-cache")]
    pub async fn update_session(&self, session: &SessionData) -> ServerResult<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to get Redis connection: {e}"))
                })?;

            let key = self.session_key(&session.session_id);
            let value = serde_json::to_string(session).map_err(|e| {
                ServerError::internal_error(format!("Failed to serialize session: {e}"))
            })?;

            let ttl = (session.expires_at - session.last_accessed).max(0) as usize;
            conn.set_ex(&key, &value, ttl)
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to update session in Redis: {e}"))
                })?;
        }

        Ok(())
    }

    /// Update session (without Redis)
    #[cfg(not(feature = "redis-cache"))]
    pub async fn update_session(&self, session: &SessionData) -> ServerResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());
        Ok(())
    }

    /// Delete session
    #[cfg(feature = "redis-cache")]
    pub async fn delete_session(&self, session_id: &str) -> ServerResult<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to get Redis connection: {e}"))
                })?;

            let key = self.session_key(session_id);
            conn.del::<&str, u32>(&key)
                .await
                .map_err(|e| {
                    ServerError::internal_error(format!("Failed to delete session from Redis: {e}"))
                })?;
        }

        Ok(())
    }

    /// Delete session (without Redis)
    #[cfg(not(feature = "redis-cache"))]
    pub async fn delete_session(&self, session_id: &str) -> ServerResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        #[cfg(feature = "redis-cache")]
        let store = RedisSessionStore::new(None, "test:", 3600);
        #[cfg(not(feature = "redis-cache"))]
        let store = RedisSessionStore::new("test:", 3600);

        let session = store
            .create_session("user1".to_string(), "org1".to_string(), None)
            .await
            .unwrap();

        assert_eq!(session.user_id, "user1");
        assert_eq!(session.organization_id, "org1");
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_session_retrieval() {
        #[cfg(feature = "redis-cache")]
        let store = RedisSessionStore::new(None, "test:", 3600);
        #[cfg(not(feature = "redis-cache"))]
        let store = RedisSessionStore::new("test:", 3600);

        let session = store
            .create_session("user1".to_string(), "org1".to_string(), None)
            .await
            .unwrap();

        let retrieved = store.get_session(&session.session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, "user1");
    }
}

