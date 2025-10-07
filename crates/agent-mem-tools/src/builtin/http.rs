//! HTTP Request Tool - HTTP 请求工具
//!
//! 提供 HTTP 请求功能，参考 MIRIX 的 http_request 实现
//! 学习自 MIRIX: mirix/functions/function_sets/extras.py

use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{ParameterSchema, PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// HTTP 请求工具
///
/// 参考 MIRIX: http_request()
pub struct HttpRequestTool;

#[async_trait]
impl Tool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Make an HTTP request and return the response. Supports GET, POST, PUT, DELETE methods."
    }

    fn schema(&self) -> ToolSchema {
        let mut properties = HashMap::new();

        properties.insert(
            "method".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "HTTP method (GET, POST, PUT, DELETE)".to_string(),
                enum_values: Some(vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                ]),
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "url".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "The URL to send the request to".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "headers".to_string(),
            PropertySchema {
                prop_type: "object".to_string(),
                description: "Optional HTTP headers as key-value pairs".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "body".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "Optional request body (for POST, PUT)".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: ParameterSchema {
                param_type: "object".to_string(),
                properties,
                required: vec!["method".to_string(), "url".to_string()],
            },
        }
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("method is required".to_string()))?;

        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("url is required".to_string()))?;

        let headers = args
            .get("headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        let body = args.get("body").and_then(|v| v.as_str());

        let response = self.make_request(method, url, headers, body).await?;

        Ok(serde_json::to_value(response)?)
    }
}

impl HttpRequestTool {
    /// 发送 HTTP 请求
    ///
    /// 参考 MIRIX: http_request()
    async fn make_request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<&str>,
    ) -> ToolResult<HttpResponse> {
        tracing::info!("Making {} request to {}", method, url);

        // 在实际实现中，这里应该使用 reqwest 或其他 HTTP 客户端
        // 这里提供一个模拟实现
        let status_code = 200;
        let response_body = format!(
            "{{\"message\": \"Mock response for {} {}\", \"success\": true}}",
            method, url
        );

        let mut response_headers = HashMap::new();
        response_headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(HttpResponse {
            status_code,
            headers: response_headers,
            body: response_body,
        })
    }
}

/// HTTP 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP 状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: String,
}

