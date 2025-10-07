//! Built-in Tools 测试
//!
//! 测试新增的内置工具功能

use agent_mem_tools::builtin::{
    FileReadTool, FileWriteTool, HttpRequestTool, SearchTool,
};
use agent_mem_tools::executor::{ExecutionContext, Tool};
use serde_json::json;
use std::time::Duration;

// ============================================================================
// SearchTool 测试
// ============================================================================

#[tokio::test]
async fn test_search_tool_basic() {
    let tool = SearchTool;

    assert_eq!(tool.name(), "search");
    assert!(tool.description().contains("Search"));

    let schema = tool.schema();
    assert_eq!(schema.name, "search");
    assert!(schema.parameters.properties.contains_key("query"));
    assert!(schema.parameters.required.contains(&"query".to_string()));
}

#[tokio::test]
async fn test_search_tool_execute_success() {
    let tool = SearchTool;
    let args = json!({
        "query": "rust programming",
        "num_results": 3
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_ok());

    let value = result.unwrap();
    assert!(value.get("query").is_some());
    assert!(value.get("results").is_some());
}

#[tokio::test]
async fn test_search_tool_missing_query() {
    let tool = SearchTool;
    let args = json!({});

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_tool_default_num_results() {
    let tool = SearchTool;
    let args = json!({
        "query": "test query"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_ok());
}

// ============================================================================
// FileReadTool 测试
// ============================================================================

#[tokio::test]
async fn test_file_read_tool_basic() {
    let tool = FileReadTool;

    assert_eq!(tool.name(), "file_read");
    assert!(tool.description().contains("Read"));

    let schema = tool.schema();
    assert_eq!(schema.name, "file_read");
    assert!(schema.parameters.properties.contains_key("filename"));
    assert!(schema.parameters.properties.contains_key("line_start"));
}



#[tokio::test]
async fn test_file_read_tool_missing_filename() {
    let tool = FileReadTool;
    let args = json!({
        "line_start": 1
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_read_tool_invalid_line_start() {
    let tool = FileReadTool;
    let args = json!({
        "filename": "test.txt",
        "line_start": 0  // Invalid
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

// ============================================================================
// FileWriteTool 测试
// ============================================================================

#[tokio::test]
async fn test_file_write_tool_basic() {
    let tool = FileWriteTool;

    assert_eq!(tool.name(), "file_write");
    assert!(tool.description().contains("Write"));

    let schema = tool.schema();
    assert_eq!(schema.name, "file_write");
    assert!(schema.parameters.properties.contains_key("filename"));
    assert!(schema.parameters.properties.contains_key("content"));
}



#[tokio::test]
async fn test_file_write_tool_missing_content() {
    let tool = FileWriteTool;
    let args = json!({
        "filename": "test.txt"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

// ============================================================================
// HttpRequestTool 测试
// ============================================================================

#[tokio::test]
async fn test_http_request_tool_basic() {
    let tool = HttpRequestTool;

    assert_eq!(tool.name(), "http_request");
    assert!(tool.description().contains("HTTP"));

    let schema = tool.schema();
    assert_eq!(schema.name, "http_request");
    assert!(schema.parameters.properties.contains_key("method"));
    assert!(schema.parameters.properties.contains_key("url"));
}

#[tokio::test]
async fn test_http_request_tool_get() {
    let tool = HttpRequestTool;
    let args = json!({
        "method": "GET",
        "url": "https://api.example.com/data"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_ok());

    let value = result.unwrap();
    assert!(value.get("status_code").is_some());
    assert!(value.get("body").is_some());
}

#[tokio::test]
async fn test_http_request_tool_post() {
    let tool = HttpRequestTool;
    let args = json!({
        "method": "POST",
        "url": "https://api.example.com/create",
        "body": "{\"name\": \"test\"}"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_http_request_tool_missing_method() {
    let tool = HttpRequestTool;
    let args = json!({
        "url": "https://api.example.com/data"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_request_tool_missing_url() {
    let tool = HttpRequestTool;
    let args = json!({
        "method": "GET"
    });

    let context = ExecutionContext {
        user: "test_user".to_string(),
        timeout: Duration::from_secs(30),
    };

    let result = tool.execute(args, &context).await;
    assert!(result.is_err());
}

