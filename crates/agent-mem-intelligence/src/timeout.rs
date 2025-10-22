//! 超时控制模块
//!
//! 提供统一的超时控制功能，防止LLM调用hang住
//!
//! **优化项 P0-#2, #12, #22**: 为所有LLM调用添加超时控制

use agent_mem_traits::Result;
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;

/// 超时配置
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// 事实提取超时时间（秒）
    pub fact_extraction_timeout_secs: u64,
    /// 决策生成超时时间（秒）
    pub decision_timeout_secs: u64,
    /// 重排序超时时间（秒）
    pub rerank_timeout_secs: u64,
    /// 冲突检测超时时间（秒）
    pub conflict_detection_timeout_secs: u64,
    /// 并行搜索超时时间（秒）
    pub search_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            fact_extraction_timeout_secs: 30,
            decision_timeout_secs: 60,
            rerank_timeout_secs: 10,
            conflict_detection_timeout_secs: 30,
            search_timeout_secs: 5,
        }
    }
}

/// 带超时的异步函数执行
pub async fn with_timeout<F, T>(
    future: F,
    timeout_secs: u64,
    operation_name: &str,
) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match timeout(Duration::from_secs(timeout_secs), future).await {
        Ok(result) => result,
        Err(_) => {
            warn!(
                "Operation '{}' timed out after {} seconds",
                operation_name, timeout_secs
            );
            Err(agent_mem_traits::AgentMemError::internal_error(format!(
                "Operation '{}' timed out after {} seconds",
                operation_name, timeout_secs
            )))
        }
    }
}

/// 带超时和重试的异步函数执行
pub async fn with_timeout_and_retry<F, Fut, T>(
    mut operation: F,
    timeout_secs: u64,
    max_retries: u32,
    operation_name: &str,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempts = 0;
    let mut last_error = None;

    while attempts <= max_retries {
        if attempts > 0 {
            warn!(
                "Retrying operation '{}' (attempt {}/{})",
                operation_name,
                attempts + 1,
                max_retries + 1
            );
        }

        match timeout(Duration::from_secs(timeout_secs), operation()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) => {
                last_error = Some(e);
                attempts += 1;
            }
            Err(_) => {
                warn!(
                    "Operation '{}' timed out after {} seconds (attempt {}/{})",
                    operation_name,
                    timeout_secs,
                    attempts + 1,
                    max_retries + 1
                );
                last_error = Some(agent_mem_traits::AgentMemError::internal_error(format!(
                    "Operation timed out after {} seconds",
                    timeout_secs
                )));
                attempts += 1;
            }
        }

        // 等待一段时间再重试
        if attempts <= max_retries {
            tokio::time::sleep(Duration::from_millis(500 * attempts as u64)).await;
        }
    }

    Err(last_error.unwrap_or_else(|| {
        agent_mem_traits::AgentMemError::internal_error(format!(
            "Operation '{}' failed after {} retries",
            operation_name, max_retries
        ))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, agent_mem_traits::AgentMemError>("success")
            },
            1,
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_with_timeout_failure() {
        let result = with_timeout(
            async {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok::<_, agent_mem_traits::AgentMemError>("success")
            },
            1,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_timeout_and_retry_success() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = with_timeout_and_retry(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if count < 2 {
                        Err(agent_mem_traits::AgentMemError::internal_error(
                            "simulated error",
                        ))
                    } else {
                        Ok("success")
                    }
                }
            },
            1,
            3,
            "test_operation",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_with_timeout_and_retry_failure() {
        let result = with_timeout_and_retry(
            || async {
                Err::<&str, _>(agent_mem_traits::AgentMemError::internal_error(
                    "simulated error",
                ))
            },
            1,
            2,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
    }
}

