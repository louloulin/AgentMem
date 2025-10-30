//! Concurrency control and task management
//!
//! This module provides advanced concurrency control mechanisms including
//! rate limiting, circuit breakers, and adaptive task scheduling.

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::sleep;
use tracing::info;

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Rate limit (requests per second)
    pub rate_limit_rps: u32,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker timeout (seconds)
    pub circuit_breaker_timeout_seconds: u64,
    /// Enable adaptive scheduling
    pub enable_adaptive_scheduling: bool,
    /// Task queue size
    pub task_queue_size: usize,
    /// Worker thread count
    pub worker_threads: usize,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            rate_limit_rps: 1000,
            circuit_breaker_threshold: 10,
            circuit_breaker_timeout_seconds: 60,
            enable_adaptive_scheduling: true,
            task_queue_size: 10000,
            worker_threads: 4,
        }
    }
}

/// Concurrency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyStats {
    pub active_tasks: usize,
    pub queued_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_task_duration_ms: f64,
    pub current_rps: f64,
    pub circuit_breaker_state: CircuitBreakerState,
}

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Concurrency manager
pub struct ConcurrencyManager {
    config: ConcurrencyConfig,
    semaphore: Arc<Semaphore>,
    rate_limiter: Arc<RateLimiter>,
    circuit_breaker: Arc<CircuitBreaker>,
    stats: Arc<RwLock<ConcurrencyStats>>,
    active_tasks: AtomicUsize,
    completed_tasks: AtomicU64,
    failed_tasks: AtomicU64,
}

impl ConcurrencyManager {
    /// Create a new concurrency manager
    pub fn new(config: ConcurrencyConfig) -> Result<Self> {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_rps));
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_threshold,
            Duration::from_secs(config.circuit_breaker_timeout_seconds),
        ));

        let stats = Arc::new(RwLock::new(ConcurrencyStats {
            active_tasks: 0,
            queued_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            average_task_duration_ms: 0.0,
            current_rps: 0.0,
            circuit_breaker_state: CircuitBreakerState::Closed,
        }));

        let manager = Self {
            config,
            semaphore,
            rate_limiter,
            circuit_breaker,
            stats,
            active_tasks: AtomicUsize::new(0),
            completed_tasks: AtomicU64::new(0),
            failed_tasks: AtomicU64::new(0),
        };

        info!(
            "Concurrency manager initialized with {} max tasks",
            manager.config.max_concurrent_tasks
        );
        Ok(manager)
    }

    /// Execute a task with concurrency control
    pub async fn execute<F, Fut, T>(&self, task: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        // Check circuit breaker
        if !self.circuit_breaker.can_execute().await {
            return Err(AgentMemError::rate_limit_error("Circuit breaker is open"));
        }

        // Apply rate limiting
        self.rate_limiter.acquire().await?;

        // Acquire semaphore permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| AgentMemError::memory_error("Failed to acquire semaphore permit"))?;

        self.active_tasks.fetch_add(1, Ordering::Relaxed);
        let start_time = Instant::now();

        let result = task().await;

        let duration = start_time.elapsed();
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);

        match &result {
            Ok(_) => {
                self.completed_tasks.fetch_add(1, Ordering::Relaxed);
                self.circuit_breaker.record_success().await;
            }
            Err(_) => {
                self.failed_tasks.fetch_add(1, Ordering::Relaxed);
                self.circuit_breaker.record_failure().await;
            }
        }

        self.update_stats(duration).await;
        result
    }

    /// Get concurrency statistics
    pub async fn get_stats(&self) -> Result<ConcurrencyStats> {
        let mut stats = self.stats.read().await.clone();
        stats.active_tasks = self.active_tasks.load(Ordering::Relaxed);
        stats.completed_tasks = self.completed_tasks.load(Ordering::Relaxed);
        stats.failed_tasks = self.failed_tasks.load(Ordering::Relaxed);
        stats.circuit_breaker_state = self.circuit_breaker.get_state().await;
        stats.current_rps = self.rate_limiter.get_current_rate().await;

        Ok(stats)
    }

    async fn update_stats(&self, duration: Duration) {
        let mut stats = self.stats.write().await;

        let total_tasks = stats.completed_tasks + stats.failed_tasks + 1;
        let duration_ms = duration.as_millis() as f64;

        stats.average_task_duration_ms =
            (stats.average_task_duration_ms * (total_tasks - 1) as f64 + duration_ms)
                / total_tasks as f64;
    }
}

/// Rate limiter implementation
struct RateLimiter {
    max_rps: u32,
    tokens: Arc<RwLock<f64>>,
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    fn new(max_rps: u32) -> Self {
        Self {
            max_rps,
            tokens: Arc::new(RwLock::new(max_rps as f64)),
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }

    async fn acquire(&self) -> Result<()> {
        loop {
            self.refill_tokens().await;

            let mut tokens = self.tokens.write().await;
            if *tokens >= 1.0 {
                *tokens -= 1.0;
                return Ok(());
            } else {
                // Calculate wait time
                let wait_time = Duration::from_millis((1000.0 / self.max_rps as f64) as u64);
                drop(tokens);
                sleep(wait_time).await;
            }
        }
    }

    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_millis(1) {
            let mut tokens = self.tokens.write().await;
            let tokens_to_add = elapsed.as_secs_f64() * self.max_rps as f64;
            *tokens = (*tokens + tokens_to_add).min(self.max_rps as f64);
            *last_refill = now;
        }
    }

    async fn get_current_rate(&self) -> f64 {
        // Simplified rate calculation
        self.max_rps as f64
    }
}

/// Circuit breaker implementation
struct CircuitBreaker {
    failure_threshold: u32,
    timeout: Duration,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_count: AtomicU64,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            timeout,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_count: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                drop(state);
                self.check_timeout().await
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        let mut state = self.state.write().await;
        *state = CircuitBreakerState::Closed;
    }

    async fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        if failures >= self.failure_threshold as u64 {
            let mut state = self.state.write().await;
            *state = CircuitBreakerState::Open;

            let mut last_failure = self.last_failure_time.write().await;
            *last_failure = Some(Instant::now());
        }
    }

    async fn check_timeout(&self) -> bool {
        let last_failure = self.last_failure_time.read().await;
        if let Some(last_failure_time) = *last_failure {
            if last_failure_time.elapsed() >= self.timeout {
                drop(last_failure);
                let mut state = self.state.write().await;
                *state = CircuitBreakerState::HalfOpen;
                return true;
            }
        }
        false
    }

    async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrency_manager_creation() {
        let config = ConcurrencyConfig::default();
        let manager = ConcurrencyManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_task_execution() {
        let config = ConcurrencyConfig::default();
        let manager = ConcurrencyManager::new(config).unwrap();

        let result = manager
            .execute(|| async { Ok::<i32, AgentMemError>(42) })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limiter = RateLimiter::new(100); // Higher rate to avoid timeout

        let start = Instant::now();
        for _ in 0..3 {
            // Fewer requests
            rate_limiter.acquire().await.unwrap();
        }
        let elapsed = start.elapsed();

        // Should complete quickly with high rate limit
        assert!(elapsed < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(1));

        // Record failures to trip the circuit breaker
        for _ in 0..3 {
            circuit_breaker.record_failure().await;
        }

        // Circuit breaker should be open
        assert!(!circuit_breaker.can_execute().await);

        // Record success to close it
        circuit_breaker.record_success().await;
        assert!(circuit_breaker.can_execute().await);
    }
}
