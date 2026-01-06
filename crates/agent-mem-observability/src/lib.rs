//! AgentMem Observability - Enterprise-grade monitoring and observability
//!
//! This crate provides comprehensive monitoring and observability for AgentMem,
//! inspired by MIRIX's tracing system but optimized for Rust's performance.
//!
//! # Features
//!
//! - **Distributed Tracing**: OpenTelemetry integration with Jaeger/Zipkin
//! - **Metrics Collection**: Prometheus metrics with custom exporters
//! - **Structured Logging**: High-performance structured logging with tracing
//! - **Health Checks**: Liveness and readiness probes
//! - **Performance Analysis**: Request tracking and bottleneck identification
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Observability Framework                         │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │  Tracing     │  │   Metrics    │  │   Logging    │      │
//! │  │ (OpenTelemetry)│ │ (Prometheus) │  │  (tracing)   │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! │         │                 │                  │               │
//! │         └─────────────────┴──────────────────┘               │
//! │                          │                                   │
//! │                  ┌───────▼────────┐                          │
//! │                  │  Health Check  │                          │
//! │                  └────────────────┘                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use agent_mem_observability::{init_observability, ObservabilityConfig};
//! use tracing::info;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize observability
//!     let config = ObservabilityConfig::default();
//!     init_observability(config).await?;
//!
//!     // Use structured logging
//!     info!("Application started");
//!
//!     // Metrics are automatically collected
//!     // Traces are automatically propagated
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Distributed tracing with OpenTelemetry
pub mod tracing_ext;

/// Prometheus metrics collection
pub mod metrics;

/// Health check endpoints
pub mod health;

/// Structured logging
pub mod logging;

/// Performance analysis
pub mod performance;

/// Error types
pub mod error;

// Re-export main types
pub use error::{ObservabilityError, ObservabilityResult};
pub use health::{HealthCheck, HealthStatus};
pub use logging::init_logging;
pub use metrics::{MetricsCollector, MetricsRegistry};
pub use performance::PerformanceAnalyzer;
pub use tracing_ext::{init_tracing, TracingConfig};

/// Observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Service name for tracing
    pub service_name: String,
    /// OpenTelemetry endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: Option<String>,
    /// Enable Prometheus metrics
    pub enable_metrics: bool,
    /// Metrics port
    pub metrics_port: u16,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Enable JSON logging
    pub json_logging: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            service_name: "agentmem".to_string(),
            otlp_endpoint: None,
            enable_metrics: true,
            metrics_port: 9090,
            log_level: "info".to_string(),
            json_logging: false,
        }
    }
}

/// Initialize observability (tracing, metrics, logging)
pub async fn init_observability(config: ObservabilityConfig) -> ObservabilityResult<()> {
    // Initialize logging first
    init_logging(&config.log_level, config.json_logging)?;

    // Initialize tracing if endpoint is configured
    if config.otlp_endpoint.is_some() {
        let tracing_config = TracingConfig {
            service_name: config.service_name.clone(),
            otlp_endpoint: config.otlp_endpoint.clone(),
        };
        init_tracing(tracing_config).await?;
    }

    // Initialize metrics if enabled
    if config.enable_metrics {
        let _registry = MetricsRegistry::new();
        // Metrics server will be started separately
    }

    tracing::info!(
        service_name = %config.service_name,
        "Observability initialized"
    );

    Ok(())
}

/// Observability statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservabilityStats {
    /// Service name
    pub service_name: String,
    /// Tracing enabled
    pub tracing_enabled: bool,
    /// Metrics enabled
    pub metrics_enabled: bool,
    /// Total spans created
    pub total_spans: u64,
    /// Total metrics collected
    pub total_metrics: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}
