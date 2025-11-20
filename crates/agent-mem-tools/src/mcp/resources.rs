//! MCP Resources 支持
//!
//! 实现 MCP 协议的 Resources 功能，允许客户端访问和订阅资源

use super::error::{McpError, McpResult};
use super::types::McpContent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP 资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// 资源 URI
    pub uri: String,

    /// 资源名称
    pub name: String,

    /// 资源描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME 类型
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,

    /// 资源元数据
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl McpResource {
    /// 创建新的资源
    pub fn new(uri: String, name: String) -> Self {
        Self {
            uri,
            name,
            description: None,
            mime_type: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置 MIME 类型
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 资源内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// 资源 URI
    pub uri: String,

    /// 内容
    pub content: McpContent,

    /// 最后修改时间
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastModified")]
    pub last_modified: Option<DateTime<Utc>>,
}

/// 资源模板参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// 参数名称
    pub name: String,

    /// 参数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 是否必需
    pub required: bool,
}

/// 资源模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    /// 模板 URI 模式
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,

    /// 模板名称
    pub name: String,

    /// 模板描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 模板参数
    pub parameters: Vec<TemplateParameter>,
}

/// 资源列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpListResourcesResponse {
    /// 资源列表
    pub resources: Vec<McpResource>,
}

/// 资源读取请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpReadResourceRequest {
    /// 资源 URI
    pub uri: String,
}

/// 资源读取响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpReadResourceResponse {
    /// 资源内容列表
    pub contents: Vec<ResourceContent>,
}

/// 资源订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSubscribeResourceRequest {
    /// 资源 URI
    pub uri: String,
}

/// 资源订阅响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSubscribeResourceResponse {
    /// 订阅 ID
    pub subscription_id: String,
}

/// 资源变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChangeEvent {
    /// 资源 URI
    pub uri: String,

    /// 变更类型
    pub change_type: ResourceChangeType,

    /// 变更时间
    pub timestamp: DateTime<Utc>,
}

/// 资源变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceChangeType {
    /// 创建
    Created,
    /// 更新
    Updated,
    /// 删除
    Deleted,
}

/// 缓存的资源
#[derive(Debug, Clone)]
struct CachedResource {
    /// 资源内容
    content: ResourceContent,

    /// 缓存时间
    cached_at: DateTime<Utc>,

    /// TTL（秒）
    ttl: u64,
}

impl CachedResource {
    /// 创建新的缓存资源
    fn new(content: ResourceContent, ttl: u64) -> Self {
        Self {
            content,
            cached_at: Utc::now(),
            ttl,
        }
    }

    /// 检查是否过期
    fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.cached_at)
            .num_seconds() as u64;
        elapsed >= self.ttl
    }
}

/// 资源订阅
#[derive(Debug, Clone)]
pub struct ResourceSubscription {
    /// 订阅 ID
    pub id: String,

    /// 资源 URI
    pub uri: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 资源管理器
pub struct ResourceManager {
    /// 资源缓存
    cache: Arc<RwLock<HashMap<String, CachedResource>>>,

    /// 资源订阅
    subscriptions: Arc<RwLock<HashMap<String, ResourceSubscription>>>,

    /// 缓存 TTL（秒）
    cache_ttl: u64,
}

impl ResourceManager {
    /// 创建新的资源管理器
    pub fn new(cache_ttl: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// 列出所有资源
    pub async fn list_resources(&self) -> McpResult<Vec<McpResource>> {
        info!("Listing all resources");

        // 这里返回预定义的资源列表
        // 在实际应用中，应该从 ResourceAgent 或数据库获取
        let resources = vec![
            McpResource::new(
                "agentmem://memory/core".to_string(),
                "Core Memory".to_string(),
            )
            .with_description("Core memory blocks and templates".to_string())
            .with_mime_type("application/json".to_string()),
            McpResource::new(
                "agentmem://memory/episodic".to_string(),
                "Episodic Memory".to_string(),
            )
            .with_description("Time-based episodic memories".to_string())
            .with_mime_type("application/json".to_string()),
            McpResource::new(
                "agentmem://memory/semantic".to_string(),
                "Semantic Memory".to_string(),
            )
            .with_description("Semantic knowledge and facts".to_string())
            .with_mime_type("application/json".to_string()),
        ];

        debug!("Found {} resources", resources.len());
        Ok(resources)
    }

    /// 读取资源内容
    pub async fn read_resource(&self, uri: &str) -> McpResult<ResourceContent> {
        info!("Reading resource: {}", uri);

        // 先检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(uri) {
                if !cached.is_expired() {
                    debug!("Resource found in cache: {}", uri);
                    return Ok(cached.content.clone());
                } else {
                    debug!("Cached resource expired: {}", uri);
                }
            }
        }

        // 从存储获取资源
        let content = self.fetch_resource_content(uri).await?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                uri.to_string(),
                CachedResource::new(content.clone(), self.cache_ttl),
            );
        }

        Ok(content)
    }

    /// 从存储获取资源内容
    async fn fetch_resource_content(&self, uri: &str) -> McpResult<ResourceContent> {
        // 解析 URI 并获取资源
        // 这里提供模拟实现，实际应该从 ResourceAgent 获取

        let content = match uri {
            "agentmem://memory/core" => McpContent::Text {
                text: serde_json::json!({
                    "type": "core_memory",
                    "blocks": [],
                    "templates": []
                })
                .to_string(),
            },
            "agentmem://memory/episodic" => McpContent::Text {
                text: serde_json::json!({
                    "type": "episodic_memory",
                    "events": []
                })
                .to_string(),
            },
            "agentmem://memory/semantic" => McpContent::Text {
                text: serde_json::json!({
                    "type": "semantic_memory",
                    "facts": []
                })
                .to_string(),
            },
            _ => {
                return Err(McpError::ResourceNotFound(uri.to_string()));
            }
        };

        Ok(ResourceContent {
            uri: uri.to_string(),
            content,
            last_modified: Some(Utc::now()),
        })
    }

    /// 订阅资源变更
    pub async fn subscribe_resource(&self, uri: &str) -> McpResult<ResourceSubscription> {
        info!("Subscribing to resource: {}", uri);

        // 验证资源是否存在
        self.read_resource(uri).await?;

        // 创建订阅
        let subscription = ResourceSubscription {
            id: uuid::Uuid::new_v4().to_string(),
            uri: uri.to_string(),
            created_at: Utc::now(),
        };

        // 保存订阅
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription.id.clone(), subscription.clone());
        }

        debug!("Created subscription: {}", subscription.id);
        Ok(subscription)
    }

    /// 取消订阅
    pub async fn unsubscribe_resource(&self, subscription_id: &str) -> McpResult<()> {
        info!("Unsubscribing: {}", subscription_id);

        let mut subscriptions = self.subscriptions.write().await;
        if subscriptions.remove(subscription_id).is_some() {
            debug!("Removed subscription: {}", subscription_id);
            Ok(())
        } else {
            Err(McpError::SubscriptionNotFound(subscription_id.to_string()))
        }
    }

    /// 获取所有订阅
    pub async fn list_subscriptions(&self) -> Vec<ResourceSubscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.values().cloned().collect()
    }

    /// 通知资源变更
    pub async fn notify_resource_change(
        &self,
        uri: &str,
        change_type: ResourceChangeType,
    ) -> McpResult<()> {
        info!("Notifying resource change: {} ({:?})", uri, change_type);

        // 清除缓存
        {
            let mut cache = self.cache.write().await;
            cache.remove(uri);
        }

        // 查找相关订阅
        let subscriptions = self.subscriptions.read().await;
        let affected_subscriptions: Vec<_> = subscriptions
            .values()
            .filter(|s| s.uri == uri)
            .cloned()
            .collect();

        debug!(
            "Found {} subscriptions for resource: {}",
            affected_subscriptions.len(),
            uri
        );

        // 在实际应用中，这里应该通过 WebSocket 或 SSE 发送通知
        // 目前只记录日志
        for subscription in affected_subscriptions {
            debug!(
                "Would notify subscription {} about {:?} change",
                subscription.id, change_type
            );
        }

        Ok(())
    }

    /// 清除过期缓存
    pub async fn cleanup_expired_cache(&self) -> usize {
        let mut cache = self.cache.write().await;
        let initial_size = cache.len();

        cache.retain(|_, cached| !cached.is_expired());

        let removed = initial_size - cache.len();
        if removed > 0 {
            info!("Cleaned up {} expired cache entries", removed);
        }

        removed
    }

    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let subscriptions = self.subscriptions.read().await;

        CacheStats {
            total_cached: cache.len(),
            total_subscriptions: subscriptions.len(),
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// 缓存的资源数量
    pub total_cached: usize,

    /// 订阅数量
    pub total_subscriptions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_resources() {
        let manager = ResourceManager::new(300);
        let resources = manager.list_resources().await.unwrap();

        assert!(!resources.is_empty());
        assert!(resources.iter().any(|r| r.uri == "agentmem://memory/core"));
    }

    #[tokio::test]
    async fn test_read_resource() {
        let manager = ResourceManager::new(300);
        let content = manager
            .read_resource("agentmem://memory/core")
            .await
            .unwrap();

        assert_eq!(content.uri, "agentmem://memory/core");
        assert!(content.last_modified.is_some());
    }

    #[tokio::test]
    async fn test_read_nonexistent_resource() {
        let manager = ResourceManager::new(300);
        let result = manager.read_resource("agentmem://nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_subscribe_resource() {
        let manager = ResourceManager::new(300);
        let subscription = manager
            .subscribe_resource("agentmem://memory/core")
            .await
            .unwrap();

        assert!(!subscription.id.is_empty());
        assert_eq!(subscription.uri, "agentmem://memory/core");
    }

    #[tokio::test]
    async fn test_unsubscribe_resource() {
        let manager = ResourceManager::new(300);
        let subscription = manager
            .subscribe_resource("agentmem://memory/core")
            .await
            .unwrap();

        let result = manager.unsubscribe_resource(&subscription.id).await;
        assert!(result.is_ok());

        // 再次取消订阅应该失败
        let result = manager.unsubscribe_resource(&subscription.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resource_cache() {
        let manager = ResourceManager::new(300);

        // 第一次读取
        let content1 = manager
            .read_resource("agentmem://memory/core")
            .await
            .unwrap();

        // 第二次读取应该从缓存获取
        let content2 = manager
            .read_resource("agentmem://memory/core")
            .await
            .unwrap();

        assert_eq!(content1.uri, content2.uri);
    }

    #[tokio::test]
    async fn test_notify_resource_change() {
        let manager = ResourceManager::new(300);

        // 订阅资源
        let _subscription = manager
            .subscribe_resource("agentmem://memory/core")
            .await
            .unwrap();

        // 通知变更
        let result = manager
            .notify_resource_change("agentmem://memory/core", ResourceChangeType::Updated)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let manager = ResourceManager::new(300);

        // 读取一些资源
        let _ = manager.read_resource("agentmem://memory/core").await;
        let _ = manager.subscribe_resource("agentmem://memory/core").await;

        let stats = manager.get_cache_stats().await;
        assert!(stats.total_cached > 0);
        assert!(stats.total_subscriptions > 0);
    }
}
