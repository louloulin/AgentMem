//! 错误处理和恢复系统
//!
//! 提供生产级的错误处理、重试策略、熔断器和降级机制

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    /// 网络错误
    Network,
    /// 超时错误
    Timeout,
    /// 认证错误
    Authentication,
    /// 限流错误
    RateLimit,
    /// 服务不可用
    ServiceUnavailable,
    /// 内部错误
    Internal,
    /// 配置错误
    Configuration,
    /// 数据错误
    Data,
}

impl ErrorType {
    /// 从错误中推断错误类型
    pub fn from_error(error: &AgentMemError) -> Self {
        match error {
            AgentMemError::NetworkError(_) => ErrorType::Network,
            AgentMemError::TimeoutError(_) => ErrorType::Timeout,
            AgentMemError::AuthError(_) => ErrorType::Authentication,
            AgentMemError::RateLimitError(_) => ErrorType::RateLimit,
            AgentMemError::StorageError(_) => ErrorType::ServiceUnavailable,
            AgentMemError::ConfigError(_) => ErrorType::Configuration,
            AgentMemError::ValidationError(_) => ErrorType::Data,
            _ => ErrorType::Internal,
        }
    }
}

/// 重试策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 基础延迟时间
    pub base_delay: Duration,
    /// 最大延迟时间
    pub max_delay: Duration,
    /// 退避策略
    pub backoff_strategy: BackoffStrategy,
    /// 可重试的错误类型
    pub retryable_errors: Vec<ErrorType>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_strategy: BackoffStrategy::Exponential,
            retryable_errors: vec![
                ErrorType::Network,
                ErrorType::Timeout,
                ErrorType::ServiceUnavailable,
                ErrorType::RateLimit,
            ],
        }
    }
}

/// 退避策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// 固定延迟
    Fixed,
    /// 线性退避
    Linear,
    /// 指数退避
    Exponential,
    /// 指数退避加抖动
    ExponentialWithJitter,
}

impl BackoffStrategy {
    /// 计算延迟时间
    pub fn calculate_delay(
        &self,
        attempt: u32,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Duration {
        let delay = match self {
            BackoffStrategy::Fixed => base_delay,
            BackoffStrategy::Linear => base_delay * attempt,
            BackoffStrategy::Exponential => base_delay * (2_u32.pow(attempt.saturating_sub(1))),
            BackoffStrategy::ExponentialWithJitter => {
                let base = base_delay * (2_u32.pow(attempt.saturating_sub(1)));
                let jitter = Duration::from_millis(fastrand::u64(0..=base.as_millis() as u64 / 4));
                base + jitter
            }
        };

        delay.min(max_delay)
    }
}

/// 熔断器状态
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// 关闭状态 - 正常工作
    Closed,
    /// 开启状态 - 拒绝请求
    Open,
    /// 半开状态 - 尝试恢复
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败阈值
    pub failure_threshold: u32,
    /// 成功阈值（半开状态）
    pub success_threshold: u32,
    /// 超时时间
    pub timeout: Duration,
    /// 重置时间
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            reset_timeout: Duration::from_secs(60),
        }
    }
}

/// 熔断器实现
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// 检查是否允许请求
    pub async fn allow_request(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否可以转换到半开状态
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        drop(state);
                        self.transition_to_half_open().await;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// 记录成功
    pub async fn record_success(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                // 重置失败计数
                *self.failure_count.write().await = 0;
            }
            CircuitBreakerState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;

                if *success_count >= self.config.success_threshold {
                    drop(state);
                    drop(success_count);
                    self.transition_to_closed().await;
                }
            }
            CircuitBreakerState::Open => {
                // 在开启状态下不应该有成功请求
            }
        }
    }

    /// 记录失败
    pub async fn record_failure(&self) {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;
                *self.last_failure_time.write().await = Some(Instant::now());

                if *failure_count >= self.config.failure_threshold {
                    drop(state);
                    drop(failure_count);
                    self.transition_to_open().await;
                }
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                self.transition_to_open().await;
            }
            CircuitBreakerState::Open => {
                *self.last_failure_time.write().await = Some(Instant::now());
            }
        }
    }

    /// 转换到关闭状态
    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitBreakerState::Closed;
        *self.failure_count.write().await = 0;
        *self.success_count.write().await = 0;
        info!("Circuit breaker transitioned to CLOSED state");
    }

    /// 转换到开启状态
    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitBreakerState::Open;
        *self.success_count.write().await = 0;
        warn!("Circuit breaker transitioned to OPEN state");
    }

    /// 转换到半开状态
    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitBreakerState::HalfOpen;
        *self.success_count.write().await = 0;
        info!("Circuit breaker transitioned to HALF_OPEN state");
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }
}

/// 降级策略
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    /// 返回默认值
    DefaultValue(String),
    /// 返回缓存值
    CachedValue,
    /// 返回空结果
    EmptyResult,
    /// 调用备用服务
    AlternativeService(String),
}

/// 错误恢复系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryConfig {
    /// 重试策略映射
    pub retry_policies: HashMap<ErrorType, RetryPolicy>,
    /// 熔断器配置映射
    pub circuit_breaker_configs: HashMap<String, CircuitBreakerConfig>,
    /// 是否启用降级
    pub enable_fallback: bool,
    /// 全局超时时间
    pub global_timeout: Duration,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        let mut retry_policies = HashMap::new();
        retry_policies.insert(ErrorType::Network, RetryPolicy::default());
        retry_policies.insert(ErrorType::Timeout, RetryPolicy::default());
        retry_policies.insert(ErrorType::ServiceUnavailable, RetryPolicy::default());

        let mut circuit_breaker_configs = HashMap::new();
        circuit_breaker_configs.insert("default".to_string(), CircuitBreakerConfig::default());

        Self {
            retry_policies,
            circuit_breaker_configs,
            enable_fallback: true,
            global_timeout: Duration::from_secs(30),
        }
    }
}

/// 生产级错误恢复系统
pub struct ProductionErrorHandler {
    config: ErrorRecoveryConfig,
    retry_policies: HashMap<ErrorType, RetryPolicy>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    fallback_strategies: HashMap<String, FallbackStrategy>,
}

impl ProductionErrorHandler {
    /// 创建新的错误处理器
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        let retry_policies = config.retry_policies.clone();
        let mut fallback_strategies = HashMap::new();

        // 设置默认降级策略
        fallback_strategies.insert("memory_add".to_string(), FallbackStrategy::EmptyResult);
        fallback_strategies.insert("memory_search".to_string(), FallbackStrategy::EmptyResult);
        fallback_strategies.insert("memory_get".to_string(), FallbackStrategy::CachedValue);

        Self {
            config,
            retry_policies,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            fallback_strategies,
        }
    }

    /// 获取或创建熔断器
    async fn get_or_create_circuit_breaker(&self, service_name: &str) -> Arc<CircuitBreaker> {
        let mut breakers = self.circuit_breakers.write().await;

        if let Some(breaker) = breakers.get(service_name) {
            breaker.clone()
        } else {
            let config = self
                .config
                .circuit_breaker_configs
                .get(service_name)
                .or_else(|| self.config.circuit_breaker_configs.get("default"))
                .cloned()
                .unwrap_or_default();

            let breaker = Arc::new(CircuitBreaker::new(config));
            breakers.insert(service_name.to_string(), breaker.clone());
            breaker
        }
    }

    /// 执行带有错误恢复的操作
    pub async fn execute_with_recovery<F, T>(&self, operation_name: &str, operation: F) -> Result<T>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>> + Send + Sync,
        T: Send + 'static,
    {
        let circuit_breaker = self.get_or_create_circuit_breaker(operation_name).await;

        // 检查熔断器状态
        if !circuit_breaker.allow_request().await {
            warn!("Circuit breaker is OPEN for service: {}", operation_name);
            return self.execute_fallback(operation_name).await;
        }

        // 执行重试逻辑
        let result = self.execute_with_retry(operation).await;

        match &result {
            Ok(_) => {
                circuit_breaker.record_success().await;
                debug!("Operation {} succeeded", operation_name);
            }
            Err(error) => {
                circuit_breaker.record_failure().await;
                warn!("Operation {} failed: {}", operation_name, error);

                // 如果启用降级，尝试降级策略
                if self.config.enable_fallback {
                    return self.execute_fallback(operation_name).await;
                }
            }
        }

        result
    }

    /// 执行重试逻辑
    async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T>> + Send>> + Send + Sync,
        T: Send + 'static,
    {
        let mut last_error = None;

        for attempt in 1..=3 {
            // 默认最多重试3次
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let error_type = ErrorType::from_error(&error);
                    let default_policy = RetryPolicy::default();
                    let retry_policy = self
                        .retry_policies
                        .get(&error_type)
                        .unwrap_or(&default_policy);

                    if attempt >= retry_policy.max_attempts {
                        last_error = Some(error);
                        break;
                    }

                    if !retry_policy.retryable_errors.contains(&error_type) {
                        return Err(error);
                    }

                    let delay = retry_policy.backoff_strategy.calculate_delay(
                        attempt,
                        retry_policy.base_delay,
                        retry_policy.max_delay,
                    );

                    debug!("Retrying operation in {:?} (attempt {})", delay, attempt);
                    tokio::time::sleep(delay).await;
                    last_error = Some(error);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AgentMemError::internal_error("Unknown error during retry".to_string())
        }))
    }

    /// 执行降级策略
    async fn execute_fallback<T>(&self, operation_name: &str) -> Result<T>
    where
        T: Send + 'static,
    {
        if let Some(strategy) = self.fallback_strategies.get(operation_name) {
            match strategy {
                FallbackStrategy::EmptyResult => {
                    info!("Using empty result fallback for {}", operation_name);
                    // 这里需要根据具体类型返回适当的空结果
                    // 由于泛型限制，这里返回错误，实际使用时需要具体实现
                    Err(AgentMemError::internal_error(
                        "Fallback not implemented for this type".to_string(),
                    ))
                }
                FallbackStrategy::DefaultValue(value) => {
                    info!(
                        "Using default value fallback for {}: {}",
                        operation_name, value
                    );
                    Err(AgentMemError::internal_error(
                        "Default value fallback not implemented".to_string(),
                    ))
                }
                FallbackStrategy::CachedValue => {
                    info!("Using cached value fallback for {}", operation_name);
                    Err(AgentMemError::internal_error(
                        "Cached value fallback not implemented".to_string(),
                    ))
                }
                FallbackStrategy::AlternativeService(service) => {
                    info!(
                        "Using alternative service fallback for {}: {}",
                        operation_name, service
                    );
                    Err(AgentMemError::internal_error(
                        "Alternative service fallback not implemented".to_string(),
                    ))
                }
            }
        } else {
            Err(AgentMemError::internal_error(format!(
                "No fallback strategy for {operation_name}"
            )))
        }
    }

    /// 获取错误恢复统计信息
    pub async fn get_recovery_stats(&self) -> HashMap<String, RecoveryStats> {
        let mut stats = HashMap::new();
        let breakers = self.circuit_breakers.read().await;

        for (service_name, breaker) in breakers.iter() {
            let state = breaker.get_state().await;
            stats.insert(
                service_name.clone(),
                RecoveryStats {
                    circuit_breaker_state: state,
                    total_requests: 0, // 这里需要实际的统计数据
                    failed_requests: 0,
                    success_rate: 0.0,
                },
            );
        }

        stats
    }
}

/// 错误恢复统计信息
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    pub circuit_breaker_state: CircuitBreakerState,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            reset_timeout: Duration::from_millis(200),
        };

        let breaker = CircuitBreaker::new(config);

        // 初始状态应该是关闭
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
        assert!(breaker.allow_request().await);

        // 记录失败，应该仍然关闭
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);

        // 再次失败，应该开启
        breaker.record_failure().await;
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Open);
        assert!(!breaker.allow_request().await);

        // 等待重置时间
        tokio::time::sleep(Duration::from_millis(250)).await;

        // 应该转换到半开状态
        assert!(breaker.allow_request().await);
        assert_eq!(breaker.get_state().await, CircuitBreakerState::HalfOpen);

        // 记录成功
        breaker.record_success().await;
        breaker.record_success().await;

        // 应该转换回关闭状态
        assert_eq!(breaker.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        let policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Exponential,
            retryable_errors: vec![ErrorType::Network],
        };

        let delay1 =
            policy
                .backoff_strategy
                .calculate_delay(1, policy.base_delay, policy.max_delay);
        let delay2 =
            policy
                .backoff_strategy
                .calculate_delay(2, policy.base_delay, policy.max_delay);
        let delay3 =
            policy
                .backoff_strategy
                .calculate_delay(3, policy.base_delay, policy.max_delay);

        assert_eq!(delay1, Duration::from_millis(10));
        assert_eq!(delay2, Duration::from_millis(20));
        assert_eq!(delay3, Duration::from_millis(40));
    }

    #[tokio::test]
    async fn test_error_recovery_system() {
        let config = ErrorRecoveryConfig::default();
        let handler = ProductionErrorHandler::new(config);

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let operation = move || -> Pin<Box<dyn Future<Output = Result<String>> + Send>> {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                if count < 2 {
                    Err(AgentMemError::network_error(
                        "Simulated network error".to_string(),
                    ))
                } else {
                    Ok("Success".to_string())
                }
            })
        };

        let result = handler
            .execute_with_recovery("test_operation", operation)
            .await;

        // 应该最终成功（经过重试）
        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }
}
