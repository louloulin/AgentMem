//! Log aggregation and analysis routes
//!
//! 🆕 Phase 4.2: 日志聚合功能
//! 提供日志统计、查询和聚合分析功能

use crate::error::{ServerError, ServerResult};
use crate::models;
use axum::{extract::Query, response::Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// 日志统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatsResponse {
    /// 总日志行数
    pub total_lines: u64,
    /// 按级别统计
    pub by_level: HashMap<String, u64>,
    /// 错误数量
    pub error_count: u64,
    /// 警告数量
    pub warning_count: u64,
    /// 信息数量
    pub info_count: u64,
    /// 调试数量
    pub debug_count: u64,
    /// 日志文件大小（字节）
    pub file_size_bytes: u64,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    /// 日期（格式：YYYY-MM-DD），默认为今天
    pub date: Option<String>,
    /// 日志级别过滤（ERROR, WARN, INFO, DEBUG）
    pub level: Option<String>,
    /// 最大返回行数
    pub limit: Option<usize>,
}

/// 获取日志统计信息
/// 
/// 🆕 Phase 4.2: 日志聚合 - 提供日志统计和分析
#[utoipa::path(
    get,
    path = "/api/v1/logs/stats",
    tag = "logs",
    params(
        ("date" = Option<String>, Query, description = "Date in YYYY-MM-DD format (default: today)")
    ),
    responses(
        (status = 200, description = "Log statistics retrieved successfully", body = LogStatsResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_log_stats(
    Query(params): Query<HashMap<String, String>>,
) -> ServerResult<Json<models::ApiResponse<LogStatsResponse>>> {
    info!("📊 获取日志统计信息");

    // 确定日志文件路径
    let date = params.get("date").cloned().unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });
    let log_file = format!("logs/agentmem-server.log.{}", date);

    // 检查文件是否存在
    if !Path::new(&log_file).exists() {
        return Ok(Json(models::ApiResponse::success(LogStatsResponse {
            total_lines: 0,
            by_level: HashMap::new(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            debug_count: 0,
            file_size_bytes: 0,
            last_updated: Utc::now(),
        })));
    }

    // 读取日志文件
    let content = fs::read_to_string(&log_file).await.map_err(|e| {
        warn!("Failed to read log file {}: {}", log_file, e);
        ServerError::Internal(format!("Failed to read log file: {}", e))
    })?;

    // 统计日志信息
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len() as u64;
    let file_size_bytes = content.len() as u64;

    let mut by_level: HashMap<String, u64> = HashMap::new();
    let mut error_count = 0u64;
    let mut warning_count = 0u64;
    let mut info_count = 0u64;
    let mut debug_count = 0u64;

    for line in &lines {
        // 简单的日志级别检测（基于日志格式）
        if line.contains(" ERROR ") || line.contains("error") {
            error_count += 1;
            *by_level.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if line.contains(" WARN ") || line.contains("warn") {
            warning_count += 1;
            *by_level.entry("WARN".to_string()).or_insert(0) += 1;
        } else if line.contains(" INFO ") || line.contains("info") {
            info_count += 1;
            *by_level.entry("INFO".to_string()).or_insert(0) += 1;
        } else if line.contains(" DEBUG ") || line.contains("debug") {
            debug_count += 1;
            *by_level.entry("DEBUG".to_string()).or_insert(0) += 1;
        } else {
            // 默认归类为INFO
            info_count += 1;
            *by_level.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }

    let response = LogStatsResponse {
        total_lines,
        by_level,
        error_count,
        warning_count,
        info_count,
        debug_count,
        file_size_bytes,
        last_updated: Utc::now(),
    };

    info!("📊 日志统计: 总行数={}, 错误={}, 警告={}, 信息={}, 调试={}",
        response.total_lines,
        response.error_count,
        response.warning_count,
        response.info_count,
        response.debug_count);

    Ok(Json(models::ApiResponse::success(response)))
}

/// 查询日志内容
/// 
/// 🆕 Phase 4.2: 日志聚合 - 提供日志查询功能
#[utoipa::path(
    get,
    path = "/api/v1/logs/query",
    tag = "logs",
    params(
        ("date" = Option<String>, Query, description = "Date in YYYY-MM-DD format (default: today)"),
        ("level" = Option<String>, Query, description = "Log level filter (ERROR, WARN, INFO, DEBUG)"),
        ("limit" = Option<usize>, Query, description = "Maximum number of lines to return (default: 100)")
    ),
    responses(
        (status = 200, description = "Log query completed successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn query_logs(
    Query(params): Query<LogQueryParams>,
) -> ServerResult<Json<models::ApiResponse<serde_json::Value>>> {
    info!("🔍 查询日志内容");

    // 确定日志文件路径
    let date = params.date.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    });
    let log_file = format!("logs/agentmem-server.log.{}", date);

    // 检查文件是否存在
    if !Path::new(&log_file).exists() {
        return Ok(Json(models::ApiResponse::success(serde_json::json!({
            "lines": [],
            "total": 0,
            "date": date,
            "message": "Log file not found"
        }))));
    }

    // 读取日志文件
    let content = fs::read_to_string(&log_file).await.map_err(|e| {
        warn!("Failed to read log file {}: {}", log_file, e);
        ServerError::Internal(format!("Failed to read log file: {}", e))
    })?;

    // 过滤和限制日志行
    let lines: Vec<&str> = content.lines().collect();
    let mut filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| {
            // 按级别过滤
            if let Some(ref level) = params.level {
                let level_upper = level.to_uppercase();
                match level_upper.as_str() {
                    "ERROR" => line.contains(" ERROR ") || line.contains("error"),
                    "WARN" => line.contains(" WARN ") || line.contains("warn"),
                    "INFO" => line.contains(" INFO ") || line.contains("info"),
                    "DEBUG" => line.contains(" DEBUG ") || line.contains("debug"),
                    _ => true,
                }
            } else {
                true
            }
        })
        .map(|s| s.to_string())
        .collect();

    // 限制返回行数（默认100行，返回最新的）
    let limit = params.limit.unwrap_or(100);
    if filtered_lines.len() > limit {
        filtered_lines = filtered_lines
            .into_iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
    }

    let response = serde_json::json!({
        "lines": filtered_lines,
        "total": filtered_lines.len(),
        "date": date,
        "level": params.level,
        "limit": limit
    });

    info!("🔍 日志查询完成: 返回 {} 行", filtered_lines.len());

    Ok(Json(models::ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 🆕 Phase 4.2: 测试日志统计功能
    #[test]
    fn test_log_stats_structure() {
        let stats = LogStatsResponse {
            total_lines: 100,
            by_level: {
                let mut map = HashMap::new();
                map.insert("ERROR".to_string(), 5);
                map.insert("WARN".to_string(), 10);
                map.insert("INFO".to_string(), 80);
                map.insert("DEBUG".to_string(), 5);
                map
            },
            error_count: 5,
            warning_count: 10,
            info_count: 80,
            debug_count: 5,
            file_size_bytes: 10240,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.total_lines, 100);
        assert_eq!(stats.error_count, 5);
        assert_eq!(stats.warning_count, 10);
        assert_eq!(stats.info_count, 80);
        assert_eq!(stats.debug_count, 5);
        assert!(stats.file_size_bytes > 0);
    }

    /// 🆕 Phase 4.2: 测试日志查询参数
    #[test]
    fn test_log_query_params() {
        let params = LogQueryParams {
            date: Some("2024-01-01".to_string()),
            level: Some("ERROR".to_string()),
            limit: Some(50),
        };

        assert_eq!(params.date, Some("2024-01-01".to_string()));
        assert_eq!(params.level, Some("ERROR".to_string()));
        assert_eq!(params.limit, Some(50));
    }

    /// 🆕 Phase 4.2: 测试日志级别过滤逻辑
    #[test]
    fn test_log_level_filtering() {
        let test_lines = vec![
            "2024-01-01 ERROR: Test error message",
            "2024-01-01 WARN: Test warning message",
            "2024-01-01 INFO: Test info message",
            "2024-01-01 DEBUG: Test debug message",
        ];

        // 测试ERROR级别过滤
        let error_lines: Vec<&str> = test_lines
            .iter()
            .filter(|line| line.contains(" ERROR ") || line.contains("error"))
            .copied()
            .collect();
        assert_eq!(error_lines.len(), 1);
        assert!(error_lines[0].contains("ERROR"));

        // 测试WARN级别过滤
        let warn_lines: Vec<&str> = test_lines
            .iter()
            .filter(|line| line.contains(" WARN ") || line.contains("warn"))
            .copied()
            .collect();
        assert_eq!(warn_lines.len(), 1);
        assert!(warn_lines[0].contains("WARN"));
    }
}

