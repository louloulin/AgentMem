//! HTTP 传输层实现
//!
//! 提供基于 HTTP/HTTPS 的 MCP 传输

use super::{Transport, McpError, McpResult};
use async_trait::async_trait;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, error};

/// HTTP 传输
pub struct HttpTransport {
    /// 服务器 URL
    url: String,
    
    /// HTTP 客户端
    client: Client,
    
    /// 自定义请求头
    headers: HeaderMap,
    
    /// 连接状态
    connected: Arc<RwLock<bool>>,
    
    /// 超时时间（秒）
    timeout: Duration,
}

impl HttpTransport {
    /// 创建新的 HTTP 传输
    pub fn new(url: String, headers: Option<HashMap<String, String>>) -> Self {
        let mut header_map = HeaderMap::new();
        
        // 添加默认请求头
        header_map.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        header_map.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        
        // 添加自定义请求头
        if let Some(headers) = headers {
            for (key, value) in headers {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_bytes(key.as_bytes()),
                    HeaderValue::from_str(&value),
                ) {
                    header_map.insert(name, val);
                }
            }
        }
        
        let timeout = Duration::from_secs(30);
        
        let client = Client::builder()
            .default_headers(header_map.clone())
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            url,
            client,
            headers: header_map,
            connected: Arc::new(RwLock::new(false)),
            timeout,
        }
    }
    
    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        
        // 重新创建客户端
        self.client = Client::builder()
            .default_headers(self.headers.clone())
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        
        self
    }
    
    /// 添加请求头
    pub fn add_header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            self.headers.insert(name, val);
        }
        self
    }
    
    /// 发送 POST 请求
    async fn post(&self, endpoint: &str, body: Value) -> McpResult<Value> {
        let url = format!("{}/{}", self.url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        debug!("Sending POST request to: {}", url);
        debug!("Request body: {}", body);
        
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| McpError::TransportError(format!("HTTP request failed: {e}")))?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(McpError::TransportError(
                format!("HTTP request failed with status {status}: {error_text}")
            ));
        }
        
        let result = response.json::<Value>().await
            .map_err(|e| McpError::TransportError(format!("Failed to parse response: {e}")))?;
        
        debug!("Response: {}", result);
        
        Ok(result)
    }
    
    /// 发送 GET 请求
    async fn get(&self, endpoint: &str) -> McpResult<Value> {
        let url = format!("{}/{}", self.url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        debug!("Sending GET request to: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| McpError::TransportError(format!("HTTP request failed: {e}")))?;
        
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(McpError::TransportError(
                format!("HTTP request failed with status {status}: {error_text}")
            ));
        }
        
        let result = response.json::<Value>().await
            .map_err(|e| McpError::TransportError(format!("Failed to parse response: {e}")))?;
        
        debug!("Response: {}", result);
        
        Ok(result)
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&mut self) -> McpResult<()> {
        info!("Connecting to HTTP server: {}", self.url);
        
        // 测试连接（发送健康检查请求）
        match self.get("health").await {
            Ok(_) => {
                *self.connected.write().await = true;
                info!("Successfully connected to HTTP server");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to HTTP server: {}", e);
                // 即使健康检查失败，也标记为已连接（服务器可能没有健康检查端点）
                *self.connected.write().await = true;
                Ok(())
            }
        }
    }
    
    async fn send_request(&self, method: &str, params: Value) -> McpResult<Value> {
        if !self.is_connected() {
            return Err(McpError::NotConnected);
        }
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "method": method,
            "params": params,
        });
        
        self.post("rpc", request_body).await
    }
    
    async fn disconnect(&mut self) -> McpResult<()> {
        info!("Disconnecting from HTTP server");
        *self.connected.write().await = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        // 使用 try_read 避免阻塞
        self.connected.try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_http_transport_creation() {
        let transport = HttpTransport::new("http://localhost:8080".to_string(), None);
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "test-key".to_string());
        
        let transport = HttpTransport::new("http://localhost:8080".to_string(), Some(headers));
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_with_timeout() {
        let transport = HttpTransport::new("http://localhost:8080".to_string(), None)
            .with_timeout(Duration::from_secs(60));
        
        assert_eq!(transport.timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_http_transport_add_header() {
        let transport = HttpTransport::new("http://localhost:8080".to_string(), None)
            .add_header("X-Custom-Header", "custom-value");
        
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_connect() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string(), None);
        
        // 连接会尝试健康检查，但即使失败也会标记为已连接
        let result = transport.connect().await;
        assert!(result.is_ok());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_http_transport_disconnect() {
        let mut transport = HttpTransport::new("http://localhost:8080".to_string(), None);
        
        transport.connect().await.unwrap();
        assert!(transport.is_connected());
        
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_send_request_not_connected() {
        let transport = HttpTransport::new("http://localhost:8080".to_string(), None);
        
        let result = transport.send_request("test_method", json!({})).await;
        assert!(result.is_err());
        
        if let Err(McpError::NotConnected) = result {
            // Expected error
        } else {
            panic!("Expected NotConnected error");
        }
    }
}

