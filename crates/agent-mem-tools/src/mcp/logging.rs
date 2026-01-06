//! MCP Logging 支持
//!
//! 提供 MCP 协议的日志功能，包括：
//! - 日志级别管理
//! - 日志流式传输
//! - 日志过滤
//! - 与 tracing 框架集成

use super::error::{McpError, McpResult};
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogLevel {
    /// 追踪级别（最详细）
    Trace,
    /// 调试级别
    Debug,
    /// 信息级别
    #[default]
    Info,
    /// 警告级别
    Warn,
    /// 错误级别
    Error,
}

impl LogLevel {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 日志 ID
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 组件名称
    pub component: String,
    /// 操作名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// 用户 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// 会话 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 请求 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// 附加字段
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, message: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            message: message.into(),
            component: component.into(),
            operation: None,
            user_id: None,
            session_id: None,
            request_id: None,
            fields: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// 设置操作
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// 设置用户 ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// 设置会话 ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// 设置请求 ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// 添加字段
    pub fn with_field(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(key.into(), value);
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 日志过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    /// 过滤器名称
    pub name: String,
    /// 最小日志级别
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_level: Option<LogLevel>,
    /// 组件过滤（支持通配符）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// 用户 ID 过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// 标签过滤
    #[serde(default)]
    pub tags: Vec<String>,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl LogFilter {
    /// 创建新的日志过滤器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            min_level: None,
            component: None,
            user_id: None,
            tags: Vec::new(),
            enabled: true,
        }
    }

    /// 设置最小日志级别
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// 设置组件过滤
    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }

    /// 设置用户 ID 过滤
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// 添加标签过滤
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// 检查日志条目是否匹配过滤器
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查日志级别
        if let Some(min_level) = self.min_level {
            if entry.level < min_level {
                return false;
            }
        }

        // 检查组件
        if let Some(ref component) = self.component {
            if !entry.component.contains(component) {
                return false;
            }
        }

        // 检查用户 ID
        if let Some(ref user_id) = self.user_id {
            if entry.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        // 检查标签
        if !self.tags.is_empty() {
            let has_matching_tag = self.tags.iter().any(|tag| entry.tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        true
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 是否启用日志
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 默认日志级别
    #[serde(default)]
    pub default_level: LogLevel,
    /// 最大缓冲区大小
    #[serde(default = "default_buffer_size")]
    pub max_buffer_size: usize,
    /// 是否启用流式传输
    #[serde(default)]
    pub enable_streaming: bool,
    /// 是否集成 tracing
    #[serde(default = "default_true")]
    pub integrate_tracing: bool,
}

fn default_buffer_size() -> usize {
    1000
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_level: LogLevel::Info,
            max_buffer_size: 1000,
            enable_streaming: false,
            integrate_tracing: true,
        }
    }
}

/// 日志管理器
#[derive(Clone)]
pub struct LoggingManager {
    /// 配置
    config: LoggingConfig,
    /// 日志缓冲区
    buffer: Arc<RwLock<VecDeque<LogEntry>>>,
    /// 日志过滤器
    filters: Arc<RwLock<Vec<LogFilter>>>,
    /// 流式订阅者
    subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<LogEntry>>>>,
}

impl LoggingManager {
    /// 创建新的日志管理器
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(RwLock::new(VecDeque::new())),
            filters: Arc::new(RwLock::new(Vec::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 创建禁用的日志管理器
    pub fn disabled() -> Self {
        Self::new(LoggingConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// 记录日志
    pub async fn log(&self, entry: LogEntry) -> McpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // 检查日志级别
        if entry.level < self.config.default_level {
            return Ok(());
        }

        // 应用过滤器
        let filters = self.filters.read().await;
        let should_log = filters.is_empty() || filters.iter().any(|f| f.matches(&entry));
        drop(filters);

        if !should_log {
            return Ok(());
        }

        // 集成 tracing
        if self.config.integrate_tracing {
            self.log_to_tracing(&entry);
        }

        // 添加到缓冲区
        let mut buffer = self.buffer.write().await;
        buffer.push_back(entry.clone());

        // 限制缓冲区大小
        while buffer.len() > self.config.max_buffer_size {
            buffer.pop_front();
        }
        drop(buffer);

        // 发送给订阅者
        if self.config.enable_streaming {
            let mut subscribers = self.subscribers.write().await;
            subscribers.retain(|sender| sender.send(entry.clone()).is_ok());
        }

        Ok(())
    }

    /// 记录到 tracing
    fn log_to_tracing(&self, entry: &LogEntry) {
        let message = &entry.message;
        let component = &entry.component;

        match entry.level {
            LogLevel::Trace => trace!(component = %component, "{}", message),
            LogLevel::Debug => debug!(component = %component, "{}", message),
            LogLevel::Info => info!(component = %component, "{}", message),
            LogLevel::Warn => warn!(component = %component, "{}", message),
            LogLevel::Error => error!(component = %component, "{}", message),
        }
    }

    /// 获取日志
    pub async fn get_logs(&self, limit: Option<usize>, offset: Option<usize>) -> Vec<LogEntry> {
        let buffer = self.buffer.read().await;
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(buffer.len());

        buffer.iter().skip(offset).take(limit).cloned().collect()
    }

    /// 查询日志
    pub async fn query_logs(
        &self,
        min_level: Option<LogLevel>,
        component: Option<&str>,
        user_id: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<LogEntry> {
        let buffer = self.buffer.read().await;
        buffer
            .iter()
            .filter(|entry| {
                if let Some(level) = min_level {
                    if entry.level < level {
                        return false;
                    }
                }
                if let Some(comp) = component {
                    if !entry.component.contains(comp) {
                        return false;
                    }
                }
                if let Some(uid) = user_id {
                    if entry.user_id.as_ref() != Some(&uid.to_string()) {
                        return false;
                    }
                }
                if let Some(st) = start_time {
                    if entry.timestamp < st {
                        return false;
                    }
                }
                if let Some(et) = end_time {
                    if entry.timestamp > et {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// 清空日志
    pub async fn clear(&self) {
        self.buffer.write().await.clear();
    }

    /// 添加过滤器
    pub async fn add_filter(&self, filter: LogFilter) {
        self.filters.write().await.push(filter);
    }

    /// 移除过滤器
    pub async fn remove_filter(&self, name: &str) -> bool {
        let mut filters = self.filters.write().await;
        if let Some(pos) = filters.iter().position(|f| f.name == name) {
            filters.remove(pos);
            true
        } else {
            false
        }
    }

    /// 获取所有过滤器
    pub async fn get_filters(&self) -> Vec<LogFilter> {
        self.filters.read().await.clone()
    }

    /// 订阅日志流
    pub async fn subscribe(&self) -> McpResult<Pin<Box<dyn Stream<Item = LogEntry> + Send>>> {
        if !self.config.enable_streaming {
            return Err(McpError::Internal(
                "Log streaming is not enabled".to_string(),
            ));
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        self.subscribers.write().await.push(tx);

        let stream = async_stream::stream! {
            while let Some(entry) = rx.recv().await {
                yield entry;
            }
        };

        Ok(Box::pin(stream))
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> LogStats {
        let buffer = self.buffer.read().await;
        let mut stats = LogStats {
            total_logs: buffer.len(),
            trace_count: 0,
            debug_count: 0,
            info_count: 0,
            warn_count: 0,
            error_count: 0,
            components: HashMap::new(),
        };

        for entry in buffer.iter() {
            match entry.level {
                LogLevel::Trace => stats.trace_count += 1,
                LogLevel::Debug => stats.debug_count += 1,
                LogLevel::Info => stats.info_count += 1,
                LogLevel::Warn => stats.warn_count += 1,
                LogLevel::Error => stats.error_count += 1,
            }

            *stats.components.entry(entry.component.clone()).or_insert(0) += 1;
        }

        stats
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// 获取配置
    pub fn config(&self) -> &LoggingConfig {
        &self.config
    }
}

/// 日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    /// 总日志数
    pub total_logs: usize,
    /// Trace 级别日志数
    pub trace_count: usize,
    /// Debug 级别日志数
    pub debug_count: usize,
    /// Info 级别日志数
    pub info_count: usize,
    /// Warn 级别日志数
    pub warn_count: usize,
    /// Error 级别日志数
    pub error_count: usize,
    /// 各组件日志数
    pub components: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "Test message", "test-component")
            .with_operation("test-op")
            .with_user_id("user1")
            .with_tag("test");

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.component, "test-component");
        assert_eq!(entry.operation, Some("test-op".to_string()));
        assert_eq!(entry.user_id, Some("user1".to_string()));
        assert_eq!(entry.tags, vec!["test"]);
    }

    #[test]
    fn test_log_filter_matches() {
        let filter = LogFilter::new("test-filter")
            .with_min_level(LogLevel::Info)
            .with_component("mcp");

        let entry1 = LogEntry::new(LogLevel::Info, "Test", "mcp-server");
        assert!(filter.matches(&entry1));

        let entry2 = LogEntry::new(LogLevel::Debug, "Test", "mcp-server");
        assert!(!filter.matches(&entry2)); // 级别太低

        let entry3 = LogEntry::new(LogLevel::Info, "Test", "other");
        assert!(!filter.matches(&entry3)); // 组件不匹配
    }

    #[tokio::test]
    async fn test_logging_manager_creation() {
        let manager = LoggingManager::new(LoggingConfig::default());
        assert!(manager.is_enabled());

        let disabled = LoggingManager::disabled();
        assert!(!disabled.is_enabled());
    }

    #[tokio::test]
    async fn test_logging_manager_log() {
        let manager = LoggingManager::new(LoggingConfig::default());
        let entry = LogEntry::new(LogLevel::Info, "Test message", "test");

        manager.log(entry).await.unwrap();

        let logs = manager.get_logs(None, None).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "Test message");
    }

    #[tokio::test]
    async fn test_logging_manager_query() {
        let manager = LoggingManager::new(LoggingConfig::default());

        // 添加多个日志
        manager
            .log(LogEntry::new(LogLevel::Info, "Info message", "comp1"))
            .await
            .unwrap();
        manager
            .log(LogEntry::new(LogLevel::Warn, "Warn message", "comp1"))
            .await
            .unwrap();
        manager
            .log(LogEntry::new(LogLevel::Error, "Error message", "comp2"))
            .await
            .unwrap();

        // 查询所有日志
        let all_logs = manager.query_logs(None, None, None, None, None).await;
        assert_eq!(all_logs.len(), 3);

        // 查询 Warn 及以上级别
        let warn_logs = manager
            .query_logs(Some(LogLevel::Warn), None, None, None, None)
            .await;
        assert_eq!(warn_logs.len(), 2);

        // 查询特定组件
        let comp1_logs = manager
            .query_logs(None, Some("comp1"), None, None, None)
            .await;
        assert_eq!(comp1_logs.len(), 2);
    }

    #[tokio::test]
    async fn test_logging_manager_filters() {
        let manager = LoggingManager::new(LoggingConfig::default());

        // 添加过滤器
        let filter = LogFilter::new("test-filter").with_min_level(LogLevel::Warn);
        manager.add_filter(filter).await;

        // Info 级别的日志应该被过滤掉
        manager
            .log(LogEntry::new(LogLevel::Info, "Info message", "test"))
            .await
            .unwrap();

        // Warn 级别的日志应该通过
        manager
            .log(LogEntry::new(LogLevel::Warn, "Warn message", "test"))
            .await
            .unwrap();

        let logs = manager.get_logs(None, None).await;
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, LogLevel::Warn);

        // 移除过滤器
        assert!(manager.remove_filter("test-filter").await);
        assert!(!manager.remove_filter("non-existent").await);
    }

    #[tokio::test]
    async fn test_logging_manager_stats() {
        let manager = LoggingManager::new(LoggingConfig::default());

        manager
            .log(LogEntry::new(LogLevel::Info, "Info 1", "comp1"))
            .await
            .unwrap();
        manager
            .log(LogEntry::new(LogLevel::Info, "Info 2", "comp1"))
            .await
            .unwrap();
        manager
            .log(LogEntry::new(LogLevel::Warn, "Warn 1", "comp2"))
            .await
            .unwrap();
        manager
            .log(LogEntry::new(LogLevel::Error, "Error 1", "comp2"))
            .await
            .unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_logs, 4);
        assert_eq!(stats.info_count, 2);
        assert_eq!(stats.warn_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.components.get("comp1"), Some(&2));
        assert_eq!(stats.components.get("comp2"), Some(&2));
    }

    #[tokio::test]
    async fn test_logging_manager_clear() {
        let manager = LoggingManager::new(LoggingConfig::default());

        manager
            .log(LogEntry::new(LogLevel::Info, "Test", "test"))
            .await
            .unwrap();

        assert_eq!(manager.get_logs(None, None).await.len(), 1);

        manager.clear().await;
        assert_eq!(manager.get_logs(None, None).await.len(), 0);
    }
}
