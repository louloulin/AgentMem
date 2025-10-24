//! SSE (Server-Sent Events) 传输层实现
//!
//! 提供基于 SSE 的 MCP 流式传输

use super::{Transport, McpError, McpResult};
use async_trait::async_trait;
use eventsource_client::{self as es, Client as _};
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn};

/// SSE 传输
pub struct SseTransport {
    /// 服务器 URL
    url: String,

    /// 连接状态
    connected: Arc<RwLock<bool>>,

    /// 响应通道
    response_tx: Option<mpsc::UnboundedSender<Value>>,
    response_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<Value>>>>,
}

impl SseTransport {
    /// 创建新的 SSE 传输
    pub fn new(url: String) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            url,
            connected: Arc::new(RwLock::new(false)),
            response_tx: Some(tx),
            response_rx: Arc::new(RwLock::new(Some(rx))),
        }
    }
}

#[async_trait]
impl Transport for SseTransport {
    async fn connect(&mut self) -> McpResult<()> {
        info!("Connecting to SSE server: {}", self.url);

        // 创建 SSE 客户端
        let client = es::ClientBuilder::for_url(&self.url)
            .map_err(|e| McpError::TransportError(format!("Failed to create SSE client: {e}")))?
            .header("Accept", "text/event-stream")
            .map_err(|e| McpError::TransportError(format!("Failed to set header: {e}")))?
            .build();

        // 获取事件流并启动监听
        let mut stream = client.stream();
        let connected = self.connected.clone();
        let response_tx = self.response_tx.clone();

        // 在后台任务中处理事件流
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(es::SSE::Event(ev)) => {
                        // 解析事件数据
                        if let Ok(value) = serde_json::from_str::<Value>(&ev.data) {
                            if let Some(tx) = &response_tx {
                                let _ = tx.send(value);
                            }
                        }
                    }
                    Ok(es::SSE::Comment(_)) => {
                        // 忽略注释
                    }
                    Err(_) => {
                        *connected.write().await = false;
                        break;
                    }
                }
            }
        });

        *self.connected.write().await = true;
        info!("Successfully connected to SSE server");

        Ok(())
    }
    
    async fn send_request(&self, method: &str, params: Value) -> McpResult<Value> {
        if !self.is_connected() {
            return Err(McpError::NotConnected);
        }
        
        // SSE 是单向通信（服务器到客户端）
        // 如果需要发送请求，需要使用额外的 HTTP 端点
        warn!("SSE transport does not support sending requests directly");
        warn!("Method: {}, Params: {}", method, params);
        
        // 返回一个模拟响应
        Ok(serde_json::json!({
            "jsonrpc": "2.0",
            "id": uuid::Uuid::new_v4().to_string(),
            "result": {
                "message": "SSE transport does not support bidirectional communication"
            }
        }))
    }
    
    async fn disconnect(&mut self) -> McpResult<()> {
        info!("Disconnecting from SSE server");

        *self.connected.write().await = false;

        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected.try_read()
            .map(|guard| *guard)
            .unwrap_or(false)
    }
}

impl SseTransport {
    /// 接收下一个事件
    pub async fn receive_event(&self) -> Option<Value> {
        let mut rx_guard = self.response_rx.write().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.recv().await
        } else {
            None
        }
    }

    /// 尝试接收事件（非阻塞）
    pub async fn try_receive_event(&self) -> Option<Value> {
        let mut rx_guard = self.response_rx.write().await;
        if let Some(rx) = rx_guard.as_mut() {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    /// 模拟发送事件（仅用于测试）
    #[cfg(test)]
    pub fn send_test_event(&self, event: Value) -> Result<(), String> {
        if let Some(tx) = &self.response_tx {
            tx.send(event).map_err(|e| e.to_string())
        } else {
            Err("Response channel not available".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_sse_transport_creation() {
        let transport = SseTransport::new("http://localhost:8080/events".to_string());
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_sse_transport_disconnect() {
        let mut transport = SseTransport::new("http://localhost:8080/events".to_string());
        
        // 模拟连接状态
        *transport.connected.write().await = true;
        assert!(transport.is_connected());
        
        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn test_send_request_not_connected() {
        let transport = SseTransport::new("http://localhost:8080/events".to_string());
        
        let result = transport.send_request("test_method", json!({})).await;
        assert!(result.is_err());
        
        if let Err(McpError::NotConnected) = result {
            // Expected error
        } else {
            panic!("Expected NotConnected error");
        }
    }

    #[tokio::test]
    async fn test_sse_transport_receive_event() {
        let transport = SseTransport::new("http://localhost:8080/events".to_string());

        // 发送一个测试事件
        if let Some(tx) = &transport.response_tx {
            tx.send(json!({"test": "data"})).unwrap();
        }

        // 接收事件
        let event = transport.receive_event().await;
        assert!(event.is_some());
        assert_eq!(event.unwrap(), json!({"test": "data"}));
    }

    #[tokio::test]
    async fn test_sse_transport_try_receive_event() {
        let transport = SseTransport::new("http://localhost:8080/events".to_string());

        // 没有事件时应该返回 None
        let event = transport.try_receive_event().await;
        assert!(event.is_none());

        // 发送一个测试事件
        if let Some(tx) = &transport.response_tx {
            tx.send(json!({"test": "data"})).unwrap();
        }

        // 现在应该能接收到事件
        let event = transport.try_receive_event().await;
        assert!(event.is_some());
        assert_eq!(event.unwrap(), json!({"test": "data"}));
    }
}

