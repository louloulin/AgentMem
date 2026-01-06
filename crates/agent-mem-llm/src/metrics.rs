//! LLM 性能监控模块
//!
//! 提供 LLM 调用的性能指标收集和分析，包括：
//! - 请求延迟追踪
//! - Token 使用统计
//! - 错误率统计
//! - 成本追踪

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// LLM 调用指标
#[derive(Debug, Clone)]
pub struct LLMMetrics {
    /// 提供商名称
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// 请求延迟（毫秒）
    pub latency_ms: u64,
    /// 输入 token 数量
    pub input_tokens: u32,
    /// 输出 token 数量
    pub output_tokens: u32,
    /// 总 token 数量
    pub total_tokens: u32,
    /// 是否成功
    pub success: bool,
    /// 错误类型（如果失败）
    pub error_type: Option<String>,
    /// 时间戳
    pub timestamp: Instant,
}

impl LLMMetrics {
    /// 创建新的指标
    pub fn new(provider: String, model: String) -> Self {
        Self {
            provider,
            model,
            latency_ms: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            success: false,
            error_type: None,
            timestamp: Instant::now(),
        }
    }

    /// 设置延迟
    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency_ms = latency.as_millis() as u64;
        self
    }

    /// 设置 token 数量
    pub fn with_tokens(mut self, input: u32, output: u32) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self.total_tokens = input + output;
        self
    }

    /// 设置成功状态
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = success;
        self
    }

    /// 设置错误类型
    pub fn with_error(mut self, error_type: String) -> Self {
        self.error_type = Some(error_type);
        self.success = false;
        self
    }

    /// 估算成本（美元）
    pub fn estimated_cost(&self) -> f64 {
        // 简化的成本估算（实际成本因提供商和模型而异）
        let input_cost_per_1k = match self.provider.as_str() {
            "openai" => match self.model.as_str() {
                m if m.contains("gpt-4") => 0.03,
                m if m.contains("gpt-3.5") => 0.0015,
                _ => 0.002,
            },
            "anthropic" => 0.008,
            "azure" => 0.002,
            _ => 0.001,
        };

        let output_cost_per_1k = input_cost_per_1k * 2.0; // 输出通常是输入的2倍

        let input_cost = (self.input_tokens as f64 / 1000.0) * input_cost_per_1k;
        let output_cost = (self.output_tokens as f64 / 1000.0) * output_cost_per_1k;

        input_cost + output_cost
    }
}

/// LLM 性能统计
#[derive(Debug, Clone)]
pub struct LLMStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 最小延迟（毫秒）
    pub min_latency_ms: u64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: u64,
    /// P50 延迟（毫秒）
    pub p50_latency_ms: u64,
    /// P95 延迟（毫秒）
    pub p95_latency_ms: u64,
    /// P99 延迟（毫秒）
    pub p99_latency_ms: u64,
    /// 总输入 token 数
    pub total_input_tokens: u64,
    /// 总输出 token 数
    pub total_output_tokens: u64,
    /// 总 token 数
    pub total_tokens: u64,
    /// 总成本（美元）
    pub total_cost: f64,
    /// 错误率
    pub error_rate: f64,
    /// 按错误类型分组的错误数
    pub errors_by_type: HashMap<String, u64>,
}

impl Default for LLMStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: u64::MAX,
            max_latency_ms: 0,
            p50_latency_ms: 0,
            p95_latency_ms: 0,
            p99_latency_ms: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            total_cost: 0.0,
            error_rate: 0.0,
            errors_by_type: HashMap::new(),
        }
    }
}

/// LLM 性能监控器
pub struct LLMMonitor {
    /// 指标历史记录
    metrics_history: Arc<Mutex<Vec<LLMMetrics>>>,
    /// 是否启用
    enabled: bool,
    /// 最大历史记录数
    max_history_size: usize,
}

impl LLMMonitor {
    /// 创建新的监控器
    pub fn new(enabled: bool, max_history_size: usize) -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            enabled,
            max_history_size,
        }
    }

    /// 使用默认配置创建监控器
    pub fn default() -> Self {
        Self::new(true, 1000)
    }

    /// 禁用监控
    pub fn disabled() -> Self {
        Self::new(false, 0)
    }

    /// 记录指标
    pub fn record(&self, metrics: LLMMetrics) {
        if !self.enabled {
            return;
        }

        debug!(
            "📊 LLM Metrics: provider={}, model={}, latency={}ms, tokens={}, success={}",
            metrics.provider,
            metrics.model,
            metrics.latency_ms,
            metrics.total_tokens,
            metrics.success
        );

        let mut history = self.metrics_history.lock().unwrap();
        history.push(metrics);

        // 限制历史记录大小
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> LLMStats {
        if !self.enabled {
            return LLMStats::default();
        }

        let history = self.metrics_history.lock().unwrap();
        if history.is_empty() {
            return LLMStats::default();
        }

        let total_requests = history.len() as u64;
        let successful_requests = history.iter().filter(|m| m.success).count() as u64;
        let failed_requests = total_requests - successful_requests;

        // 计算延迟统计
        let mut latencies: Vec<u64> = history.iter().map(|m| m.latency_ms).collect();
        latencies.sort_unstable();

        let sum_latency: u64 = latencies.iter().sum();
        let avg_latency_ms = sum_latency as f64 / total_requests as f64;
        let min_latency_ms = *latencies.first().unwrap_or(&0);
        let max_latency_ms = *latencies.last().unwrap_or(&0);

        let p50_index = (total_requests as f64 * 0.50) as usize;
        let p95_index = (total_requests as f64 * 0.95) as usize;
        let p99_index = (total_requests as f64 * 0.99) as usize;

        let p50_latency_ms = latencies.get(p50_index).copied().unwrap_or(0);
        let p95_latency_ms = latencies.get(p95_index).copied().unwrap_or(0);
        let p99_latency_ms = latencies.get(p99_index).copied().unwrap_or(0);

        // 计算 token 统计
        let total_input_tokens: u64 = history.iter().map(|m| m.input_tokens as u64).sum();
        let total_output_tokens: u64 = history.iter().map(|m| m.output_tokens as u64).sum();
        let total_tokens = total_input_tokens + total_output_tokens;

        // 计算成本
        let total_cost: f64 = history.iter().map(|m| m.estimated_cost()).sum();

        // 计算错误率
        let error_rate = failed_requests as f64 / total_requests as f64;

        // 按错误类型分组
        let mut errors_by_type: HashMap<String, u64> = HashMap::new();
        for metric in history.iter() {
            if let Some(error_type) = &metric.error_type {
                *errors_by_type.entry(error_type.clone()).or_insert(0) += 1;
            }
        }

        LLMStats {
            total_requests,
            successful_requests,
            failed_requests,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            total_input_tokens,
            total_output_tokens,
            total_tokens,
            total_cost,
            error_rate,
            errors_by_type,
        }
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        let stats = self.get_stats();
        info!("📊 LLM Performance Statistics:");
        info!("  Total Requests: {}", stats.total_requests);
        info!(
            "  Successful: {} ({:.2}%)",
            stats.successful_requests,
            (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
        );
        info!(
            "  Failed: {} ({:.2}%)",
            stats.failed_requests,
            stats.error_rate * 100.0
        );
        info!(
            "  Latency: avg={:.2}ms, min={}ms, max={}ms",
            stats.avg_latency_ms, stats.min_latency_ms, stats.max_latency_ms
        );
        info!(
            "  Latency Percentiles: P50={}ms, P95={}ms, P99={}ms",
            stats.p50_latency_ms, stats.p95_latency_ms, stats.p99_latency_ms
        );
        info!(
            "  Tokens: input={}, output={}, total={}",
            stats.total_input_tokens, stats.total_output_tokens, stats.total_tokens
        );
        info!("  Estimated Cost: ${:.4}", stats.total_cost);

        if !stats.errors_by_type.is_empty() {
            info!("  Errors by Type:");
            for (error_type, count) in &stats.errors_by_type {
                info!("    {}: {}", error_type, count);
            }
        }
    }

    /// 重置统计信息
    pub fn reset(&self) {
        if !self.enabled {
            return;
        }

        let mut history = self.metrics_history.lock().unwrap();
        history.clear();
        info!("🔄 LLM metrics reset");
    }

    /// 获取历史记录数量
    pub fn history_size(&self) -> usize {
        if !self.enabled {
            return 0;
        }

        let history = self.metrics_history.lock().unwrap();
        history.len()
    }
}

impl Clone for LLMMonitor {
    fn clone(&self) -> Self {
        Self {
            metrics_history: Arc::clone(&self.metrics_history),
            enabled: self.enabled,
            max_history_size: self.max_history_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = LLMMetrics::new("openai".to_string(), "gpt-4".to_string())
            .with_latency(Duration::from_millis(500))
            .with_tokens(100, 200)
            .with_success(true);

        assert_eq!(metrics.provider, "openai");
        assert_eq!(metrics.model, "gpt-4");
        assert_eq!(metrics.latency_ms, 500);
        assert_eq!(metrics.input_tokens, 100);
        assert_eq!(metrics.output_tokens, 200);
        assert_eq!(metrics.total_tokens, 300);
        assert!(metrics.success);
    }

    #[test]
    fn test_cost_estimation() {
        let metrics =
            LLMMetrics::new("openai".to_string(), "gpt-4".to_string()).with_tokens(1000, 2000);

        let cost = metrics.estimated_cost();
        assert!(cost > 0.0);
    }

    #[test]
    fn test_monitor() {
        let monitor = LLMMonitor::default();

        let metrics1 = LLMMetrics::new("openai".to_string(), "gpt-4".to_string())
            .with_latency(Duration::from_millis(500))
            .with_tokens(100, 200)
            .with_success(true);

        let metrics2 = LLMMetrics::new("openai".to_string(), "gpt-4".to_string())
            .with_latency(Duration::from_millis(600))
            .with_tokens(150, 250)
            .with_success(false)
            .with_error("timeout".to_string());

        monitor.record(metrics1);
        monitor.record(metrics2);

        let stats = monitor.get_stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.error_rate, 0.5);
    }
}
