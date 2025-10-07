//! Search Tool - 搜索工具
//!
//! 提供网络搜索功能，参考 MIRIX 的 web_search 实现
//! 学习自 MIRIX: mirix/functions/function_sets/extras.py

use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{ParameterSchema, PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 搜索工具
///
/// 参考 MIRIX: web_search()
pub struct SearchTool;

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns search results with titles, URLs, and descriptions."
    }

    fn schema(&self) -> ToolSchema {
        let mut properties = HashMap::new();

        properties.insert(
            "query".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "The search query to look for on the web".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "num_results".to_string(),
            PropertySchema {
                prop_type: "integer".to_string(),
                description: "Maximum number of results to return (defaults to 5, max 10)".to_string(),
                enum_values: None,
                default: Some(serde_json::json!(5)),
                minimum: Some(1.0),
                maximum: Some(10.0),
                items: None,
            },
        );

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: ParameterSchema {
                param_type: "object".to_string(),
                properties,
                required: vec!["query".to_string()],
            },
        }
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("query is required".to_string()))?;

        let num_results = args
            .get("num_results")
            .and_then(|v| v.as_i64())
            .unwrap_or(5)
            .min(10)
            .max(1) as usize;

        // 执行搜索
        let results = self.perform_search(query, num_results).await?;

        Ok(serde_json::to_value(results)?)
    }
}

impl SearchTool {
    /// 执行搜索
    ///
    /// 参考 MIRIX: web_search()
    async fn perform_search(&self, query: &str, num_results: usize) -> ToolResult<SearchResults> {
        tracing::info!("Searching for: {}", query);

        // 模拟搜索结果
        // 在实际实现中，这里应该调用真实的搜索 API（如 DuckDuckGo、Google 等）
        let results = vec![
            SearchResult {
                title: format!("Search result for: {}", query),
                url: "https://example.com/result1".to_string(),
                description: format!("This is a search result for the query: {}", query),
            },
            SearchResult {
                title: format!("Another result for: {}", query),
                url: "https://example.com/result2".to_string(),
                description: "More information about the search query".to_string(),
            },
        ];

        let limited_results: Vec<SearchResult> = results.into_iter().take(num_results).collect();

        Ok(SearchResults {
            query: query.to_string(),
            num_results: limited_results.len(),
            results: limited_results,
        })
    }
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// 搜索查询
    pub query: String,
    /// 结果数量
    pub num_results: usize,
    /// 搜索结果列表
    pub results: Vec<SearchResult>,
}

/// 单个搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 标题
    pub title: String,
    /// URL
    pub url: String,
    /// 描述
    pub description: String,
}



