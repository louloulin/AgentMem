//! Resilience patterns for AgentMem Cognitive
//! 
//! Provides circuit breaker and rate limiting

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, Instant};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

// AtomicU8 representation for thread-safe state
const STATE_CLOSED: u8 = 0;
const STATE_OPEN: u8 = 1;
const STATE_HALFOPEN: u8 = 2;

impl CircuitState {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Circuit breaker - thread-safe implementation
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: std::sync::Mutex<Option<Instant>>,
    threshold: u64,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u64, timeout: Duration) -> Self {
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: std::sync::Mutex::new(None),
            threshold,
            timeout,
        }
    }
    
    pub fn state(&self) -> CircuitState {
        let current = self.state.load(Ordering::Relaxed);
        
        if current == STATE_OPEN {
            // Check if timeout has passed - update to half-open if so
            let last_failure = self.last_failure_time.lock().unwrap();
            if let Some(time) = *last_failure {
                if time.elapsed() >= self.timeout {
                    self.state.store(STATE_HALFOPEN, Ordering::Relaxed);
                    return CircuitState::HalfOpen;
                }
            }
        }
        
        CircuitState::from_u8(current)
    }
    
    pub fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        
        // Only transition from half-open to closed on success
        self.state.compare_exchange(
            STATE_HALFOPEN, STATE_CLOSED, Ordering::Relaxed, Ordering::Relaxed
        ).ok();
    }
    
    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());
        
        if self.failure_count.load(Ordering::Relaxed) >= self.threshold {
            self.state.store(STATE_OPEN, Ordering::Relaxed);
        }
    }
    
    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }
    
    pub fn reset(&self) {
        self.state.store(STATE_CLOSED, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        *self.last_failure_time.lock().unwrap() = None;
    }
}

/// Rate limiter token bucket - thread-safe implementation
pub struct RateLimiter {
    tokens: AtomicU64,
    max_tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: std::sync::Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        Self {
            tokens: AtomicU64::new(max_tokens),
            max_tokens,
            refill_rate,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }
    
    fn refill(&self) {
        let mut last = self.last_refill.lock().unwrap();
        let elapsed = last.elapsed().as_secs_f64();
        
        // Refill based on actual elapsed time in seconds
        if elapsed > 0.0 {
            let new_tokens = (elapsed * self.refill_rate as f64) as u64;
            let current = self.tokens.load(Ordering::Relaxed);
            let new_value = (current + new_tokens).min(self.max_tokens);
            self.tokens.store(new_value, Ordering::Relaxed);
            *last = Instant::now();
        }
    }
    
    pub fn try_acquire(&self, tokens: u64) -> bool {
        self.refill();
        
        loop {
            let current = self.tokens.load(Ordering::Relaxed);
            if current < tokens {
                return false;
            }
            
            let new_value = current - tokens;
            if self.tokens.compare_exchange(
                current, new_value, Ordering::Relaxed, Ordering::Relaxed
            ).is_ok() {
                return true;
            }
        }
    }
    
    pub fn available_tokens(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::Relaxed)
    }
    
    pub fn reset(&self) {
        self.tokens.store(self.max_tokens, Ordering::Relaxed);
        *self.last_refill.lock().unwrap() = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(5, Duration::from_secs(60));
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }
    
    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        
        cb.record_failure(); // Now at threshold
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }
    
    #[test]
    fn test_circuit_breaker_half_open_recovery() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(10));
        
        // Trigger circuit break
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        
        // Wait for timeout - state() will auto-transition to half-open
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        
        // Success should close the circuit
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
    
    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10, 1);
        
        assert!(limiter.try_acquire(5));
        assert!(limiter.try_acquire(5));
        assert!(!limiter.try_acquire(1)); // Should fail, only 0 left
    }
    
    #[test]
    fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(10, 100); // 100 tokens per second
        
        limiter.try_acquire(10);
        assert_eq!(limiter.available_tokens(), 0);
        
        // Wait 20ms = ~2 tokens at 100/s rate
        std::thread::sleep(Duration::from_millis(20));
        let tokens = limiter.available_tokens();
        assert!(tokens >= 1, "Expected at least 1 token, got {}", tokens);
    }
}
