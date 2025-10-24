//! File Operations Tool - 文件操作工具
//!
//! 提供文件读写功能，参考 MIRIX 的文件操作实现
//! 学习自 MIRIX: mirix/functions/function_sets/extras.py

use crate::error::{ToolError, ToolResult};
use crate::executor::{ExecutionContext, Tool};
use crate::schema::{ParameterSchema, PropertySchema, ToolSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// 文件读取工具
///
/// 参考 MIRIX: read_from_text_file()
pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read lines from a text file. Returns the content of the specified lines."
    }

    fn schema(&self) -> ToolSchema {
        let mut properties = HashMap::new();

        properties.insert(
            "filename".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "The path of the file to read".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "line_start".to_string(),
            PropertySchema {
                prop_type: "integer".to_string(),
                description: "Line to start reading from (1-indexed)".to_string(),
                enum_values: None,
                default: None,
                minimum: Some(1.0),
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "num_lines".to_string(),
            PropertySchema {
                prop_type: "integer".to_string(),
                description: "How many lines to read (defaults to 1)".to_string(),
                enum_values: None,
                default: Some(serde_json::json!(1)),
                minimum: Some(1.0),
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
                required: vec!["filename".to_string(), "line_start".to_string()],
            },
        }
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("filename is required".to_string()))?;

        let line_start = args
            .get("line_start")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ToolError::InvalidArgument("line_start is required".to_string()))?
            as usize;

        let num_lines = args
            .get("num_lines")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as usize;

        if line_start < 1 || num_lines < 1 {
            return Err(ToolError::InvalidArgument(
                "line_start and num_lines must be positive integers".to_string(),
            ));
        }

        let content = self.read_file(filename, line_start, num_lines).await?;

        Ok(serde_json::to_value(FileReadResult {
            filename: filename.to_string(),
            line_start,
            num_lines,
            content,
        })?)
    }
}

impl FileReadTool {
    /// 读取文件内容
    ///
    /// 参考 MIRIX: read_from_text_file()
    async fn read_file(
        &self,
        filename: &str,
        line_start: usize,
        num_lines: usize,
    ) -> ToolResult<String> {
        let path = PathBuf::from(filename);

        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File not found: {filename}"
            )));
        }

        let file = fs::File::open(&path).await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to open file: {e}"))
        })?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut result_lines = Vec::new();
        let mut current_line = 1;

        while let Some(line) = lines.next_line().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to read line: {e}"))
        })? {
            if current_line >= line_start && current_line < line_start + num_lines {
                result_lines.push(line);
            }
            if current_line >= line_start + num_lines {
                break;
            }
            current_line += 1;
        }

        Ok(result_lines.join("\n"))
    }
}

/// 文件写入工具
///
/// 参考 MIRIX: append_to_text_file()
pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write or append content to a text file."
    }

    fn schema(&self) -> ToolSchema {
        let mut properties = HashMap::new();

        properties.insert(
            "filename".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "The path of the file to write to".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "content".to_string(),
            PropertySchema {
                prop_type: "string".to_string(),
                description: "Content to write to the file".to_string(),
                enum_values: None,
                default: None,
                minimum: None,
                maximum: None,
                items: None,
            },
        );

        properties.insert(
            "append".to_string(),
            PropertySchema {
                prop_type: "boolean".to_string(),
                description: "Whether to append to the file (defaults to false)".to_string(),
                enum_values: None,
                default: Some(serde_json::json!(false)),
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
                required: vec!["filename".to_string(), "content".to_string()],
            },
        }
    }

    async fn execute(&self, args: Value, _context: &ExecutionContext) -> ToolResult<Value> {
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("filename is required".to_string()))?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgument("content is required".to_string()))?;

        let append = args
            .get("append")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        self.write_file(filename, content, append).await?;

        Ok(serde_json::to_value(FileWriteResult {
            filename: filename.to_string(),
            bytes_written: content.len(),
            append,
        })?)
    }
}

impl FileWriteTool {
    /// 写入文件
    ///
    /// 参考 MIRIX: append_to_text_file()
    async fn write_file(&self, filename: &str, content: &str, append: bool) -> ToolResult<()> {
        let path = PathBuf::from(filename);

        let mut file = if append {
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
        } else {
            fs::File::create(&path).await
        }
        .map_err(|e| ToolError::ExecutionFailed(format!("Failed to open file: {e}")))?;

        file.write_all(content.as_bytes())
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write to file: {e}")))?;

        if append {
            file.write_all(b"\n").await.map_err(|e| {
                ToolError::ExecutionFailed(format!("Failed to write newline: {e}"))
            })?;
        }

        Ok(())
    }
}

/// 文件读取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResult {
    pub filename: String,
    pub line_start: usize,
    pub num_lines: usize,
    pub content: String,
}

/// 文件写入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteResult {
    pub filename: String,
    pub bytes_written: usize,
    pub append: bool,
}

