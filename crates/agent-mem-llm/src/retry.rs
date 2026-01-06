//! LLM 重试机制模块
//!
//! 提供统一的重试策略，包括：
//! - 指数退避
//! - 速率限制处理
//! - 超时处理
//! - 错误分类

use agent_mem_traits::{AgentMemError, Result};
use std::time::Duration;
use tracing::{debug, warn};

/// 重试策略配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 基础延迟时间（毫秒）
    pub base_delay_ms: u64,
    /// 最大延迟时间（毫秒）
    pub max_delay_ms: u64,
    /// 退避因子（指数退避）
    pub backoff_factor: f64,
    /// 是否启用抖动（jitter）
    pub enable_jitter: bool,
    /// 超时时间（秒）
    pub timeout_seconds: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000, // 1 秒
            max_delay_ms: 60000, // 60 秒
            backoff_factor: 2.0, // 指数退避因子
            enable_jitter: true, // 启用抖动
            timeout_seconds: 60, // 60 秒超时
        }
    }
}

impl RetryConfig {
    /// 创建保守的重试配置（更多重试，更长延迟）
    pub fn conservative() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 2000,
            max_delay_ms: 120000,
            backoff_factor: 2.5,
            enable_jitter: true,
            timeout_seconds: 120,
        }
    }

    /// 创建激进的重试配置（更少重试，更短延迟）
    pub fn aggressive() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_factor: 1.5,
            enable_jitter: false,
            timeout_seconds: 30,
        }
    }

    /// 创建无重试配置
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            base_delay_ms: 0,
            max_delay_ms: 0,
            backoff_factor: 1.0,
            enable_jitter: false,
            timeout_seconds: 30,
        }
    }
}

/// 错误类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// 网络错误（可重试）
    Network,
    /// 速率限制错误（可重试，需要更长延迟）
    RateLimit,
    /// 超时错误（可重试）
    Timeout,
    /// 服务器错误（可重试）
    ServerError,
    /// 客户端错误（不可重试）
    ClientError,
    /// 认证错误（不可重试）
    Authentication,
    /// 配置错误（不可重试）
    Configuration,
    /// 未知错误（可重试）
    Unknown,
}

impl ErrorType {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorType::Network
                | ErrorType::RateLimit
                | ErrorType::Timeout
                | ErrorType::ServerError
                | ErrorType::Unknown
        )
    }

    /// 从 AgentMemError 分类错误类型
    pub fn from_error(error: &AgentMemError) -> Self {
        match error {
            AgentMemError::NetworkError(_) => ErrorType::Network,
            AgentMemError::RateLimitError(_) => ErrorType::RateLimit,
            AgentMemError::TimeoutError(_) => ErrorType::Timeout,
            AgentMemError::AuthError(_) => ErrorType::Authentication,
            AgentMemError::ConfigError(_) | AgentMemError::InvalidConfig(_) => {
                ErrorType::Configuration
            }
            AgentMemError::LLMError(msg) => {
                // 尝试从错误消息中推断错误类型
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("rate limit") || msg_lower.contains("429") {
                    ErrorType::RateLimit
                } else if msg_lower.contains("timeout") {
                    ErrorType::Timeout
                } else if msg_lower.contains("500")
                    || msg_lower.contains("502")
                    || msg_lower.contains("503")
                    || msg_lower.contains("504")
                {
                    ErrorType::ServerError
                } else if msg_lower.contains("400")
                    || msg_lower.contains("401")
                    || msg_lower.contains("403")
                    || msg_lower.contains("404")
                {
                    ErrorType::ClientError
                } else {
                    ErrorType::Unknown
                }
            }
            _ => ErrorType::Unknown,
        }
    }

    /// 获取建议的延迟倍数
    pub fn delay_multiplier(&self) -> f64 {
        match self {
            ErrorType::RateLimit => 3.0, // 速率限制需要更长延迟
            ErrorType::ServerError => 2.0,
            ErrorType::Network => 1.5,
            ErrorType::Timeout => 1.5,
            _ => 1.0,
        }
    }
}

/// 重试策略执行器
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// 创建新的重试执行器
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建重试执行器
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// 计算延迟时间（指数退避 + 抖动）
    fn calculate_delay(&self, attempt: u32, error_type: ErrorType) -> Duration {
        // 基础延迟 * (退避因子 ^ 尝试次数) * 错误类型倍数
        let base_delay = self.config.base_delay_ms as f64;
        let backoff = self.config.backoff_factor.powi(attempt as i32);
        let multiplier = error_type.delay_multiplier();
        let mut delay_ms = base_delay * backoff * multiplier;

        // 限制最大延迟
        delay_ms = delay_ms.min(self.config.max_delay_ms as f64);

        // 添加抖动（随机 ±25%）
        if self.config.enable_jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(-0.25..=0.25);
            delay_ms *= 1.0 + jitter;
        }

        Duration::from_millis(delay_ms as u64)
    }

    /// 执行带重试的异步操作
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            debug!(
                "Executing operation (attempt {}/{})",
                attempt + 1,
                self.config.max_retries + 1
            );

            // 执行操作（带超时）
            let result = tokio::time::timeout(
                Duration::from_secs(self.config.timeout_seconds),
                operation(),
            )
            .await;

            match result {
                Ok(Ok(value)) => {
                    if attempt > 0 {
                        debug!("✅ Operation succeeded after {} retries", attempt);
                    }
                    return Ok(value);
                }
                Ok(Err(error)) => {
                    let error_type = ErrorType::from_error(&error);
                    warn!(
                        "❌ Operation failed (attempt {}): {:?} - {}",
                        attempt + 1,
                        error_type,
                        error
                    );

                    // 检查是否可重试
                    if !error_type.is_retryable() {
                        debug!("Error type {:?} is not retryable, aborting", error_type);
                        return Err(error);
                    }

                    last_error = Some(error);

                    // 如果还有重试次数，等待后重试
                    if attempt < self.config.max_retries {
                        let delay = self.calculate_delay(attempt, error_type);
                        debug!("⏳ Waiting {:?} before retry...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
                Err(_) => {
                    // 超时错误
                    let timeout_error = AgentMemError::TimeoutError(format!(
                        "Operation timed out after {} seconds",
                        self.config.timeout_seconds
                    ));
                    warn!("⏱️ Operation timed out (attempt {})", attempt + 1);

                    last_error = Some(timeout_error);

                    // 如果还有重试次数，等待后重试
                    if attempt < self.config.max_retries {
                        let delay = self.calculate_delay(attempt, ErrorType::Timeout);
                        debug!("⏳ Waiting {:?} before retry...", delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // 所有重试都失败了
        Err(last_error.unwrap_or_else(|| AgentMemError::llm_error("All retry attempts failed")))
    }

    /// 执行带重试的异步操作（带上下文）
    pub async fn execute_with_context<F, Fut, T>(&self, operation: F, context: &str) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        debug!("Starting operation: {}", context);
        let result = self.execute(operation).await;
        match &result {
            Ok(_) => debug!("✅ Operation succeeded: {}", context),
            Err(e) => warn!("❌ Operation failed: {} - {}", context, e),
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_classification() {
        let network_error = AgentMemError::NetworkError("Connection failed".to_string());
        assert_eq!(ErrorType::from_error(&network_error), ErrorType::Network);
        assert!(ErrorType::Network.is_retryable());

        let auth_error = AgentMemError::AuthError("Invalid token".to_string());
        assert_eq!(
            ErrorType::from_error(&auth_error),
            ErrorType::Authentication
        );
        assert!(!ErrorType::Authentication.is_retryable());

        let rate_limit_error = AgentMemError::RateLimitError("Too many requests".to_string());
        assert_eq!(
            ErrorType::from_error(&rate_limit_error),
            ErrorType::RateLimit
        );
        assert!(ErrorType::RateLimit.is_retryable());
    }

    #[test]
    fn test_retry_config() {
        let default_config = RetryConfig::default();
        assert_eq!(default_config.max_retries, 3);
        assert_eq!(default_config.base_delay_ms, 1000);

        let conservative_config = RetryConfig::conservative();
        assert_eq!(conservative_config.max_retries, 5);
        assert!(conservative_config.base_delay_ms > default_config.base_delay_ms);

        let aggressive_config = RetryConfig::aggressive();
        assert_eq!(aggressive_config.max_retries, 2);
        assert!(aggressive_config.base_delay_ms < default_config.base_delay_ms);
    }
}
