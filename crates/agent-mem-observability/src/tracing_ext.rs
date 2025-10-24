//! Distributed tracing with OpenTelemetry
//!
//! This module provides OpenTelemetry integration for distributed tracing,
//! inspired by MIRIX's tracing.py but optimized for Rust.

use crate::error::{ObservabilityError, ObservabilityResult};
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler};
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name
    pub service_name: String,
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "agentmem".to_string(),
            otlp_endpoint: None,
        }
    }
}

/// Initialize OpenTelemetry tracing
pub async fn init_tracing(config: TracingConfig) -> ObservabilityResult<()> {
    // Create resource with service name
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    ]);

    // Create tracer provider
    let tracer = if let Some(otlp_endpoint) = config.otlp_endpoint {
        // OTLP exporter
        init_otlp_tracer(&otlp_endpoint, resource)?
    } else {
        return Err(ObservabilityError::ConfigError(
            "No tracing endpoint configured".to_string(),
        ));
    };

    // Create tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize subscriber
    tracing_subscriber::registry()
        .with(telemetry)
        .try_init()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    tracing::info!(
        service_name = %config.service_name,
        "OpenTelemetry tracing initialized"
    );

    Ok(())
}

/// Initialize OTLP tracer
fn init_otlp_tracer(
    endpoint: &str,
    resource: Resource,
) -> ObservabilityResult<opentelemetry_sdk::trace::Tracer> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::WithExportConfig;

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| ObservabilityError::TracingInitFailed(e.to_string()))?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(resource)
        .with_id_generator(RandomIdGenerator::default())
        .with_sampler(Sampler::AlwaysOn)
        .build();

    let tracer = provider.tracer("agentmem");

    Ok(tracer)
}

/// Shutdown tracing gracefully
pub async fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

/// Get current trace ID
pub fn get_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let span = tracing::Span::current();
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        Some(format!("{:032x}", span_context.trace_id()))
    } else {
        None
    }
}

/// Macro to create a traced function
///
/// # Example
///
/// ```rust
/// use agent_mem_observability::traced;
///
/// traced! {
///     async fn my_function(arg: i32) -> Result<String, Box<dyn std::error::Error>> {
///         // Function body
///         Ok(format!("Result: {}", arg))
///     }
/// }
/// ```
#[macro_export]
macro_rules! traced {
    ($func:item) => {
        #[tracing::instrument]
        $func
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.service_name, "agentmem");
        assert!(config.otlp_endpoint.is_none());
    }

    #[test]
    fn test_tracing_config_with_otlp() {
        let config = TracingConfig {
            service_name: "test".to_string(),
            otlp_endpoint: Some("http://localhost:4317".to_string()),
        };
        assert_eq!(config.service_name, "test");
        assert!(config.otlp_endpoint.is_some());
    }

    #[tokio::test]
    async fn test_init_tracing_no_endpoint() {
        let config = TracingConfig::default();
        let result = init_tracing(config).await;
        assert!(result.is_err());
    }
}
