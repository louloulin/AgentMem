//! Server-Sent Events (SSE) support for streaming responses
//!
//! This module provides SSE endpoints for server-to-client streaming communication.
//!
//! Features:
//! - Streaming message delivery
//! - Keep-alive support
//! - Authentication
//! - Multi-tenant isolation
//! - Error handling

use crate::error::ServerResult;
use crate::middleware::auth::AuthUser;
use axum::{
    extract::Extension,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error};

/// SSE message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseMessage {
    /// New message notification
    Message {
        message_id: String,
        agent_id: String,
        user_id: String,
        org_id: String,
        content: String,
        timestamp: String,
    },
    /// Agent status update
    AgentUpdate {
        agent_id: String,
        org_id: String,
        status: String,
        timestamp: String,
    },
    /// Memory update notification
    MemoryUpdate {
        memory_id: String,
        agent_id: String,
        org_id: String,
        operation: String, // "created", "updated", "deleted"
        timestamp: String,
    },
    /// Streaming chunk (for LLM responses)
    StreamChunk {
        request_id: String,
        org_id: String,
        chunk: String,
        is_final: bool,
        timestamp: String,
    },
    /// Error notification
    Error {
        code: String,
        message: String,
        timestamp: String,
    },
    /// Keep-alive heartbeat
    Heartbeat { timestamp: String },
}

/// SSE manager for broadcasting messages with multi-tenant isolation
#[derive(Clone)]
pub struct SseManager {
    /// Default broadcast channel for backwards compatibility
    broadcast_tx: broadcast::Sender<SseMessage>,
    /// Per-organization broadcast channels for tenant isolation
    org_channels: Arc<RwLock<HashMap<String, broadcast::Sender<SseMessage>>>>,
}

impl SseManager {
    /// Create a new SSE manager
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            broadcast_tx,
            org_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a specific organization
    async fn get_org_channel(&self, org_id: &str) -> broadcast::Sender<SseMessage> {
        let mut channels = self.org_channels.write().await;
        if let Some(tx) = channels.get(org_id) {
            return tx.clone();
        }
        let (tx, _) = broadcast::channel(1000);
        channels.insert(org_id.to_string(), tx.clone());
        tx
    }

    /// Broadcast to organization-specific channel
    pub async fn broadcast_to_org(&self, org_id: String, message: SseMessage) -> ServerResult<()> {
        let tx = self.get_org_channel(&org_id).await;
        let _ = tx.send(message);
        Ok(())
    }

    /// Broadcast a message to all SSE clients (global, backwards compatible)
    pub fn broadcast(&self, message: SseMessage) -> ServerResult<()> {
        let _ = self.broadcast_tx.send(message);
        Ok(())
    }

    /// Subscribe to organization-specific channel
    pub async fn subscribe_to_org(&self, org_id: &str) -> broadcast::Receiver<SseMessage> {
        self.get_org_channel(org_id).await.subscribe()
    }

    /// Get the global broadcast sender
    pub fn broadcast_sender(&self) -> broadcast::Sender<SseMessage> {
        self.broadcast_tx.clone()
    }
}

impl Default for SseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE handler with multi-tenant isolation
pub async fn sse_handler(
    Extension(auth_user): Extension<AuthUser>,
    Extension(manager): Extension<Arc<SseManager>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let org_id = auth_user.org_id.clone();
    
    debug!(
        "New SSE connection from user: {}, org: {}",
        auth_user.user_id, org_id
    );

    // Subscribe to organization-specific broadcast channel
    let rx = manager.subscribe_to_org(&org_id).await;

    // Create stream from broadcast receiver
    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        let org_id = org_id.clone();
        async move {
            match result {
                Ok(message) => {
                    // Only forward messages for this organization
                    match &message {
                        SseMessage::Message { org_id: msg_org, .. } |
                        SseMessage::AgentUpdate { org_id: msg_org, .. } |
                        SseMessage::MemoryUpdate { org_id: msg_org, .. } |
                        SseMessage::StreamChunk { org_id: msg_org, .. } => {
                            if msg_org != &org_id {
                                return None;
                            }
                        }
                        SseMessage::Heartbeat { .. } | SseMessage::Error { .. } => {
                            // These are global messages
                        }
                    }

                    match serde_json::to_string(&message) {
                        Ok(json) => Some(Ok(Event::default().data(json))),
                        Err(e) => {
                            error!("Failed to serialize SSE message: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    error!("Broadcast receive error: {}", e);
                    None
                }
            }
        }
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    )
}

/// SSE streaming endpoint for LLM responses with multi-tenant isolation
pub async fn sse_stream_llm_response(
    Extension(auth_user): Extension<AuthUser>,
    Extension(manager): Extension<Arc<SseManager>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let org_id = auth_user.org_id.clone();
    
    debug!(
        "New SSE LLM streaming connection from user: {}, org: {}",
        auth_user.user_id, org_id
    );

    // Subscribe to organization-specific channel
    let rx = manager.subscribe_to_org(&org_id).await;

    // Create stream that only forwards StreamChunk messages for this org
    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        let org_id = org_id.clone();
        async move {
            match result {
                Ok(SseMessage::StreamChunk {
                    request_id,
                    org_id: msg_org,
                    chunk,
                    is_final,
                    timestamp,
                }) => {
                    // Filter by organization
                    if msg_org != org_id {
                        return None;
                    }

                    let message = SseMessage::StreamChunk {
                        request_id,
                        org_id: msg_org,
                        chunk,
                        is_final,
                        timestamp,
                    };

                    match serde_json::to_string(&message) {
                        Ok(json) => Some(Ok(Event::default().data(json))),
                        Err(e) => {
                            error!("Failed to serialize stream chunk: {}", e);
                            None
                        }
                    }
                }
                Ok(_) => None, // Ignore other message types
                Err(e) => {
                    error!("Broadcast receive error: {}", e);
                    None
                }
            }
        }
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_manager_creation() {
        let manager = SseManager::new();
        assert!(manager.broadcast_sender().receiver_count() == 0);
    }

    #[test]
    fn test_sse_message_serialization() {
        let message = SseMessage::Message {
            message_id: "msg1".to_string(),
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
            org_id: "org1".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&message)
            .expect("SSE message serialization should succeed in test");
        assert!(json.contains("message_id"));
        assert!(json.contains("msg1"));
        assert!(json.contains("org_id"));
    }

    #[test]
    fn test_stream_chunk_serialization() {
        let message = SseMessage::StreamChunk {
            request_id: "req1".to_string(),
            org_id: "org1".to_string(),
            chunk: "Hello".to_string(),
            is_final: false,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&message)
            .expect("SSE message serialization should succeed in test");
        assert!(json.contains("stream_chunk"));
        assert!(json.contains("req1"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_sse_broadcast() {
        let manager = SseManager::new();

        let message = SseMessage::Heartbeat {
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Should not error even with no subscribers
        assert!(manager.broadcast(message).is_ok());
    }

    #[tokio::test]
    async fn test_org_broadcast() {
        let manager = SseManager::new();

        let message = SseMessage::Message {
            message_id: "msg1".to_string(),
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
            org_id: "org1".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert!(manager.broadcast_to_org("org1".to_string(), message).await.is_ok());
    }
}
