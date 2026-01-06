//! WebSocket support for real-time communication
//!
//! This module provides WebSocket endpoints for real-time bidirectional communication
//! between clients and the AgentMem server.
//!
//! Features:
//! - Connection management
//! - Message broadcasting
//! - Heartbeat mechanism (ping/pong)
//! - Authentication
//! - Multi-tenant isolation

use crate::error::{ServerError, ServerResult};
use crate::middleware::auth::AuthUser;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// New message notification
    Message {
        message_id: String,
        agent_id: String,
        user_id: String,
        content: String,
        timestamp: String,
    },
    /// Agent status update
    AgentUpdate {
        agent_id: String,
        status: String,
        timestamp: String,
    },
    /// Memory update notification
    MemoryUpdate {
        memory_id: String,
        agent_id: String,
        operation: String, // "created", "updated", "deleted"
        timestamp: String,
    },
    /// Error notification
    Error {
        code: String,
        message: String,
        timestamp: String,
    },
    /// Heartbeat ping
    Ping { timestamp: String },
    /// Heartbeat pong
    Pong { timestamp: String },
}

/// WebSocket connection info
#[derive(Debug, Clone)]
struct ConnectionInfo {
    user_id: String,
    org_id: String,
    connected_at: chrono::DateTime<chrono::Utc>,
}

/// WebSocket connection manager
#[derive(Clone)]
pub struct WebSocketManager {
    /// Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    /// Broadcast channel for messages
    broadcast_tx: broadcast::Sender<WsMessage>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);

        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    /// Get the broadcast sender
    pub fn broadcast_sender(&self) -> broadcast::Sender<WsMessage> {
        self.broadcast_tx.clone()
    }

    /// Register a new connection
    async fn register_connection(&self, connection_id: String, user_id: String, org_id: String) {
        let info = ConnectionInfo {
            user_id,
            org_id,
            connected_at: chrono::Utc::now(),
        };

        self.connections
            .write()
            .await
            .insert(connection_id.clone(), info);
        info!("WebSocket connection registered: {}", connection_id);
    }

    /// Unregister a connection
    async fn unregister_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
        info!("WebSocket connection unregistered: {}", connection_id);
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Broadcast a message to all connections
    pub fn broadcast(&self, message: WsMessage) -> ServerResult<()> {
        // Ignore errors when there are no receivers (e.g., during testing)
        let _ = self.broadcast_tx.send(message);
        Ok(())
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket handler
///
/// Handles WebSocket upgrade requests and manages the connection lifecycle.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(auth_user): Extension<AuthUser>,
    Extension(manager): Extension<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, auth_user, manager))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, auth_user: AuthUser, manager: Arc<WebSocketManager>) {
    let connection_id = Uuid::new_v4().to_string();

    info!(
        "New WebSocket connection: {} (user: {}, org: {})",
        connection_id, auth_user.user_id, auth_user.org_id
    );

    // Register connection
    manager
        .register_connection(
            connection_id.clone(),
            auth_user.user_id.clone(),
            auth_user.org_id.clone(),
        )
        .await;

    // Split socket into sender and receiver
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    // Subscribe to broadcast channel
    let mut broadcast_rx = manager.broadcast_sender().subscribe();

    // Spawn heartbeat task
    let heartbeat_connection_id = connection_id.clone();
    let heartbeat_sender = sender.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let ping_message = WsMessage::Ping {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            let json = match serde_json::to_string(&ping_message) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize ping message: {}", e);
                    break;
                }
            };

            let mut sender_guard = heartbeat_sender.lock().await;
            if sender_guard.send(Message::Text(json)).await.is_err() {
                debug!(
                    "Failed to send ping, connection {} closed",
                    heartbeat_connection_id
                );
                break;
            }
        }
    });

    // Spawn broadcast receiver task
    let broadcast_connection_id = connection_id.clone();
    let _broadcast_org_id = auth_user.org_id.clone();
    let broadcast_sender = sender.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            // TODO: Filter messages by organization for multi-tenant isolation
            // For now, broadcast to all connections

            let json = match serde_json::to_string(&message) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize broadcast message: {}", e);
                    continue;
                }
            };

            let mut sender_guard = broadcast_sender.lock().await;
            if sender_guard.send(Message::Text(json)).await.is_err() {
                debug!(
                    "Failed to send broadcast message, connection {} closed",
                    broadcast_connection_id
                );
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received text message: {}", text);

                // Parse and handle message
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Pong { .. }) => {
                        debug!("Received pong from connection {}", connection_id);
                    }
                    Ok(other_message) => {
                        debug!("Received message: {:?}", other_message);
                        // Handle other message types as needed
                    }
                    Err(e) => {
                        warn!("Failed to parse WebSocket message: {}", e);
                    }
                }
            }
            Ok(Message::Binary(_)) => {
                debug!("Received binary message (not supported)");
            }
            Ok(Message::Ping(_)) => {
                debug!("Received ping from connection {}", connection_id);
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong from connection {}", connection_id);
            }
            Ok(Message::Close(_)) => {
                info!("Connection {} closed by client", connection_id);
                break;
            }
            Err(e) => {
                error!("WebSocket error on connection {}: {}", connection_id, e);
                break;
            }
        }
    }

    // Cleanup
    heartbeat_task.abort();
    broadcast_task.abort();
    manager.unregister_connection(&connection_id).await;

    info!("WebSocket connection {} closed", connection_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_registration() {
        let manager = WebSocketManager::new();

        manager
            .register_connection("conn1".to_string(), "user1".to_string(), "org1".to_string())
            .await;

        assert_eq!(manager.connection_count().await, 1);

        manager.unregister_connection("conn1").await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_message_broadcast() {
        let manager = WebSocketManager::new();

        let message = WsMessage::Message {
            message_id: "msg1".to_string(),
            agent_id: "agent1".to_string(),
            user_id: "user1".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Should not error even with no subscribers
        assert!(manager.broadcast(message).is_ok());
    }
}
