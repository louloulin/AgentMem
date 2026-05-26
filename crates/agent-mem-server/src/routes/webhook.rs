//! Webhook management routes
//!
//! Provides endpoints for webhook CRUD operations and event delivery.
//!
//! # Webhook API
//!
//! - POST /api/v1/webhooks - Create webhook subscription
//! - GET /api/v1/webhooks - List webhooks
//! - GET /api/v1/webhooks/:id - Get webhook
//! - PUT /api/v1/webhooks/:id - Update webhook
//! - DELETE /api/v1/webhooks/:id - Delete webhook
//! - GET /api/v1/webhooks/stats - Get webhook statistics
//! - POST /api/v1/webhooks/:id/test - Test webhook delivery

use crate::error::{ServerError, ServerResult};
use crate::middleware::AuthUser;
use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Webhook event types that can trigger webhooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Memory created event
    MemoryCreated,
    /// Memory updated event
    MemoryUpdated,
    /// Memory deleted event
    MemoryDeleted,
    /// Memory searched event
    MemorySearched,
    /// Agent message received
    AgentMessage,
    /// Agent state changed
    AgentStateChanged,
    /// System health changed
    HealthChanged,
    /// Custom event
    Custom(String),
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEventType::MemoryCreated => write!(f, "memory_created"),
            WebhookEventType::MemoryUpdated => write!(f, "memory_updated"),
            WebhookEventType::MemoryDeleted => write!(f, "memory_deleted"),
            WebhookEventType::MemorySearched => write!(f, "memory_searched"),
            WebhookEventType::AgentMessage => write!(f, "agent_message"),
            WebhookEventType::AgentStateChanged => write!(f, "agent_state_changed"),
            WebhookEventType::HealthChanged => write!(f, "health_changed"),
            WebhookEventType::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

/// Webhook delivery status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebhookDeliveryStatus {
    /// Pending delivery
    Pending,
    /// Successfully delivered
    Success,
    /// Failed delivery
    Failed,
    /// Retry pending
    Retrying,
}

/// Webhook subscription response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookSubscriptionResponse {
    /// Webhook ID
    pub id: String,
    /// User ID who owns this webhook
    pub user_id: String,
    /// Webhook name
    pub name: String,
    /// Target URL to receive events
    pub url: String,
    /// Secret for signature verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Event types to subscribe to
    pub event_types: Vec<String>,
    /// Whether webhook is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub updated_at: i64,
}

/// Create webhook request
#[derive(Debug, Clone, Deserialize, ToSchema, IntoParams)]
pub struct CreateWebhookRequest {
    /// Webhook name
    pub name: String,
    /// Target URL to receive events
    pub url: String,
    /// Event types to subscribe to
    pub event_types: Vec<String>,
    /// Optional: set webhook as active (default: true)
    #[serde(default = "default_active")]
    pub is_active: bool,
}

fn default_active() -> bool {
    true
}

/// Update webhook request
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateWebhookRequest {
    /// Optional: webhook name
    pub name: Option<String>,
    /// Optional: target URL
    pub url: Option<String>,
    /// Optional: event types
    pub event_types: Option<Vec<String>>,
    /// Optional: active status
    pub is_active: Option<bool>,
}

/// Webhook event payload
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookEvent {
    /// Event ID
    pub id: String,
    /// Webhook ID
    pub webhook_id: String,
    /// Event type
    pub event_type: String,
    /// Event data (JSON)
    pub data: serde_json::Value,
    /// Delivery status
    pub status: WebhookDeliveryStatus,
    /// Attempt count
    pub attempt_count: i32,
    /// Last attempt timestamp
    pub last_attempt_at: Option<i64>,
    /// Next retry timestamp
    pub next_retry_at: Option<i64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Creation timestamp
    pub created_at: i64,
}

/// Webhook event delivery request (internal)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookDeliveryRequest {
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: i64,
    /// Signature
    pub signature: String,
}

/// List webhooks response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ListWebhooksResponse {
    /// List of webhooks
    pub webhooks: Vec<WebhookSubscriptionResponse>,
    /// Total count
    pub total: usize,
}

/// Webhook statistics
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookStats {
    /// Total webhooks
    pub total: usize,
    /// Active webhooks
    pub active: usize,
    /// Total deliveries
    pub total_deliveries: usize,
    /// Successful deliveries
    pub successful_deliveries: usize,
    /// Failed deliveries
    pub failed_deliveries: usize,
    /// Success rate
    pub success_rate: f64,
}

/// Create a new webhook subscription
///
/// POST /api/v1/webhooks
#[utoipa::path(
    post,
    path = "/api/v1/webhooks",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token")
    ),
    request_body = CreateWebhookRequest,
    responses(
        (status = 201, description = "Webhook created successfully", body = WebhookSubscriptionResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_webhook(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateWebhookRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate URL
    if !req.url.starts_with("http://") && !req.url.starts_with("https://") {
        return Err(ServerError::bad_request("URL must start with http:// or https://"));
    }

    // Validate event types
    if req.event_types.is_empty() {
        return Err(ServerError::bad_request("At least one event type is required"));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Generate webhook (hide secret in response)
    let webhook = WebhookSubscriptionResponse {
        id: Uuid::new_v4().to_string(),
        user_id: auth_user.user_id.clone(),
        name: req.name,
        url: req.url,
        secret: Some(generate_secret()),
        event_types: req.event_types,
        is_active: req.is_active,
        created_at: now,
        updated_at: now,
    };

    // Store webhook
    webhook_state.add_webhook(webhook.clone()).await?;

    // Return without secret
    let mut response = webhook.clone();
    response.secret = None;

    Ok((StatusCode::CREATED, Json(response)))
}

/// List webhooks for current user
///
/// GET /api/v1/webhooks
#[utoipa::path(
    get,
    path = "/api/v1/webhooks",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token")
    ),
    responses(
        (status = 200, description = "List of webhooks", body = ListWebhooksResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_webhooks(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<impl IntoResponse> {
    let webhooks = webhook_state.list_webhooks(&auth_user.user_id).await?;

    // Hide secrets in response
    let webhooks: Vec<_> = webhooks
        .into_iter()
        .map(|mut w| {
            w.secret = None;
            w
        })
        .collect();

    Ok(Json(ListWebhooksResponse {
        total: webhooks.len(),
        webhooks,
    }))
}

/// Get webhook by ID
///
/// GET /api/v1/webhooks/:id
#[utoipa::path(
    get,
    path = "/api/v1/webhooks/{id}",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("id" = String, Path, description = "Webhook ID")
    ),
    responses(
        (status = 200, description = "Webhook details", body = WebhookSubscriptionResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Webhook not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_webhook(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    let webhook = webhook_state
        .get_webhook(&id, &auth_user.user_id)
        .await?
        .ok_or_else(|| ServerError::not_found("Webhook not found"))?;

    let mut response = webhook;
    response.secret = None;

    Ok(Json(response))
}

/// Update webhook
///
/// PUT /api/v1/webhooks/:id
#[utoipa::path(
    put,
    path = "/api/v1/webhooks/{id}",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("id" = String, Path, description = "Webhook ID")
    ),
    request_body = UpdateWebhookRequest,
    responses(
        (status = 200, description = "Webhook updated", body = WebhookSubscriptionResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Webhook not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn update_webhook(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWebhookRequest>,
) -> ServerResult<impl IntoResponse> {
    // Validate URL if provided
    if let Some(ref url) = req.url {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ServerError::bad_request("URL must start with http:// or https://"));
        }
    }

    let webhook = webhook_state
        .update_webhook(&id, &auth_user.user_id, req)
        .await?
        .ok_or_else(|| ServerError::not_found("Webhook not found"))?;

    let mut response = webhook;
    response.secret = None;

    Ok(Json(response))
}

/// Delete webhook
///
/// DELETE /api/v1/webhooks/:id
#[utoipa::path(
    delete,
    path = "/api/v1/webhooks/{id}",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("id" = String, Path, description = "Webhook ID")
    ),
    responses(
        (status = 204, description = "Webhook deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Webhook not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_webhook(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    webhook_state
        .delete_webhook(&id, &auth_user.user_id)
        .await?
        .ok_or_else(|| ServerError::not_found("Webhook not found"))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get webhook statistics
///
/// GET /api/v1/webhooks/stats
#[utoipa::path(
    get,
    path = "/api/v1/webhooks/stats",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token")
    ),
    responses(
        (status = 200, description = "Webhook statistics", body = WebhookStats),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_webhook_stats(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> ServerResult<impl IntoResponse> {
    let stats = webhook_state.get_stats(&auth_user.user_id).await?;
    Ok(Json(stats))
}

/// Test webhook delivery
///
/// POST /api/v1/webhooks/:id/test
#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{id}/test",
    tag = "webhooks",
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
        ("id" = String, Path, description = "Webhook ID")
    ),
    responses(
        (status = 200, description = "Test delivered successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Webhook not found"),
        (status = 500, description = "Delivery failed")
    )
)]
pub async fn test_webhook(
    Extension(webhook_state): Extension<Arc<WebhookState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> ServerResult<impl IntoResponse> {
    let webhook = webhook_state
        .get_webhook(&id, &auth_user.user_id)
        .await?
        .ok_or_else(|| ServerError::not_found("Webhook not found"))?;

    // Send test event
    let test_event = WebhookDeliveryRequest {
        event_type: "test".to_string(),
        data: serde_json::json!({
            "message": "This is a test webhook event from AgentMem"
        }),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        signature: String::new(),
    };

    // Deliver webhook (fire and forget)
    let url = webhook.url.clone();
    let secret = webhook.secret.unwrap_or_default();
    tokio::spawn(async move {
        deliver_webhook(&url, &secret, &test_event).await;
    });

    Ok(Json(serde_json::json!({
        "message": "Test event sent",
        "webhook_id": id
    })))
}

/// Webhook state manager
#[derive(Clone)]
pub struct WebhookState {
    /// Webhook storage (in-memory for MVP)
    webhooks: Arc<dashmap::DashMap<String, WebhookSubscriptionResponse>>,
    /// Event sender for broadcasting
    event_tx: tokio::sync::broadcast::Sender<WebhookEvent>,
}

impl WebhookState {
    /// Create new webhook state
    pub fn new() -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        Self {
            webhooks: Arc::new(dashmap::DashMap::new()),
            event_tx,
        }
    }

    /// Add a new webhook
    pub async fn add_webhook(&self, webhook: WebhookSubscriptionResponse) -> ServerResult<()> {
        self.webhooks.insert(webhook.id.clone(), webhook);
        Ok(())
    }

    /// List webhooks for a user
    pub async fn list_webhooks(&self, user_id: &str) -> ServerResult<Vec<WebhookSubscriptionResponse>> {
        Ok(self
            .webhooks
            .iter()
            .filter(|w| w.user_id == user_id)
            .map(|w| w.clone())
            .collect())
    }

    /// Get webhook by ID
    pub async fn get_webhook(
        &self,
        id: &str,
        user_id: &str,
    ) -> ServerResult<Option<WebhookSubscriptionResponse>> {
        Ok(self.webhooks.get(id).and_then(|w| {
            if w.user_id == user_id {
                Some(w.clone())
            } else {
                None
            }
        }))
    }

    /// Update webhook
    pub async fn update_webhook(
        &self,
        id: &str,
        user_id: &str,
        req: UpdateWebhookRequest,
    ) -> ServerResult<Option<WebhookSubscriptionResponse>> {
        let webhook = self.webhooks.get(id).map(|w| w.clone());

        if let Some(mut wh) = webhook {
            if wh.user_id != user_id {
                return Ok(None);
            }

            if let Some(name) = req.name {
                wh.name = name;
            }
            if let Some(url) = req.url {
                wh.url = url;
            }
            if let Some(event_types) = req.event_types {
                wh.event_types = event_types;
            }
            if let Some(is_active) = req.is_active {
                wh.is_active = is_active;
            }
            wh.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            self.webhooks.insert(id.to_string(), wh.clone());
            Ok(Some(wh))
        } else {
            Ok(None)
        }
    }

    /// Delete webhook
    pub async fn delete_webhook(&self, id: &str, user_id: &str) -> ServerResult<Option<()>> {
        if let Some(w) = self.webhooks.get(id) {
            if w.user_id == user_id {
                self.webhooks.remove(id);
                return Ok(Some(()));
            }
        }
        Ok(None)
    }

    /// Get webhook statistics
    pub async fn get_stats(&self, user_id: &str) -> ServerResult<WebhookStats> {
        let webhooks: Vec<_> = self
            .webhooks
            .iter()
            .filter(|w| w.user_id == user_id)
            .map(|w| w.clone())
            .collect();
        let total = webhooks.len();
        let active = webhooks.iter().filter(|w| w.is_active).count();

        Ok(WebhookStats {
            total,
            active,
            total_deliveries: 0,
            successful_deliveries: 0,
            failed_deliveries: 0,
            success_rate: 0.0,
        })
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<WebhookEvent> {
        self.event_tx.subscribe()
    }

    /// Publish an event
    pub async fn publish(&self, event: WebhookEvent) -> ServerResult<()> {
        let _ = self.event_tx.send(event);
        Ok(())
    }
}

impl Default for WebhookState {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random secret for webhook
fn generate_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Deliver webhook to URL
async fn deliver_webhook(url: &str, secret: &str, event: &WebhookDeliveryRequest) {
    if url.is_empty() {
        return;
    }

    let client = reqwest::Client::new();
    let timestamp = event.timestamp.to_string();
    let payload = serde_json::to_string(event).unwrap_or_default();

    // Generate signature using HMAC-SHA256
    use ring::hmac::{self, HMAC_SHA256};
    let key = hmac::Key::new(HMAC_SHA256, secret.as_bytes());
    let signature = hmac::sign(&key, payload.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let _ = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-AgentMem-Signature", format!("sha256={}", signature_hex))
        .header("X-AgentMem-Timestamp", timestamp)
        .body(payload)
        .send()
        .await;
}