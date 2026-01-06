//! Session management trait definitions

use crate::{Filters, Metadata, Result, Session};
use async_trait::async_trait;

/// Core trait for session managers
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create a new session
    async fn create_session(
        &self,
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
    ) -> Result<Session>;

    /// Get an existing session
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>>;

    /// Update session metadata
    async fn update_session(&self, session_id: &str, metadata: &Metadata) -> Result<()>;

    /// Delete a session
    async fn delete_session(&self, session_id: &str) -> Result<()>;

    /// Build filters for a session
    fn build_filters(&self, session: &Session) -> Filters;

    /// Build metadata for a session
    fn build_metadata(&self, session: &Session) -> Metadata;

    /// Validate a session
    fn validate_session(&self, session: &Session) -> Result<()>;
}
