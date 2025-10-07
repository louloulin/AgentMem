//! Built-in tools
//!
//! This module provides a collection of built-in tools for common operations.
//!
//! 参考 MIRIX 实现:
//! - mirix/functions/function_sets/base.py
//! - mirix/functions/function_sets/extras.py

pub mod calculator;
pub mod echo;
pub mod file_ops;
pub mod http;
pub mod json_parser;
pub mod search;
pub mod string_ops;
pub mod time_ops;

pub use calculator::CalculatorTool;
pub use echo::EchoTool;
pub use file_ops::{FileReadResult, FileReadTool, FileWriteResult, FileWriteTool};
pub use http::{HttpRequestTool, HttpResponse};
pub use json_parser::JsonParserTool;
pub use search::{SearchResult, SearchResults, SearchTool};
pub use string_ops::StringOpsTool;
pub use time_ops::TimeOpsTool;

use crate::{ToolExecutor, ToolResult};
use std::sync::Arc;

/// Register all built-in tools
///
/// 注册所有内置工具，包括：
/// - 基础工具（echo, calculator, json_parser, string_ops, time_ops）
/// - 搜索工具（search）
/// - 文件操作工具（file_read, file_write）
/// - HTTP 请求工具（http_request）
pub async fn register_all_builtin_tools(executor: &ToolExecutor) -> ToolResult<()> {
    // 基础工具
    executor.register_tool(Arc::new(CalculatorTool)).await?;
    executor.register_tool(Arc::new(EchoTool)).await?;
    executor.register_tool(Arc::new(JsonParserTool)).await?;
    executor.register_tool(Arc::new(StringOpsTool)).await?;
    executor.register_tool(Arc::new(TimeOpsTool)).await?;

    // 新增工具（参考 MIRIX）
    executor.register_tool(Arc::new(SearchTool)).await?;
    executor.register_tool(Arc::new(FileReadTool)).await?;
    executor.register_tool(Arc::new(FileWriteTool)).await?;
    executor.register_tool(Arc::new(HttpRequestTool)).await?;

    Ok(())
}
