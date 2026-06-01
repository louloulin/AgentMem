//! Extraction pipeline orchestrator

use crate::error::{ExtractionError, Result};
use crate::models::{
    ExecutionMode, ExtractionContext, ExtractionInput, ExtractionOutput, PipelineConfig,
};
use crate::stage::ExtractionStage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Extraction pipeline orchestrator
///
/// Manages a sequence of extraction stages and executes them in order.
/// Supports sequential, parallel, and conditional execution modes.
pub struct ExtractionPipeline {
    /// Pipeline stages (sorted by priority)
    stages: Vec<Box<dyn ExtractionStage>>,

    /// Pipeline configuration
    config: PipelineConfig,

    /// Stage metrics
    metrics: Arc<RwLock<PipelineMetrics>>,
}

/// Internal pipeline metrics
#[derive(Debug, Default)]
struct PipelineMetrics {
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    stage_metrics: HashMap<String, StageMetrics>,
}

/// Per-stage metrics
#[derive(Debug, Default)]
struct StageMetrics {
    executions: u64,
    failures: u64,
    avg_duration_ms: f64,
}

impl ExtractionPipeline {
    /// Create new extraction pipeline
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            stages: Vec::new(),
            config,
            metrics: Arc::new(RwLock::new(PipelineMetrics::default())),
        }
    }

    /// Create pipeline with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PipelineConfig::default())
    }

    /// Add a stage to the pipeline
    pub async fn add_stage(&mut self, stage: Box<dyn ExtractionStage>) -> Result<()> {
        // Validate stage
        stage.validate()?;

        // Insert stage in priority order
        let priority = stage.priority();
        let insert_pos = self
            .stages
            .iter()
            .position(|s| s.priority() < priority)
            .unwrap_or(self.stages.len());

        self.stages.insert(insert_pos, stage);
        debug!("Added stage at position {}", insert_pos);

        Ok(())
    }

    /// Remove a stage by name
    pub async fn remove_stage(&mut self, name: &str) -> Result<()> {
        let initial_len = self.stages.len();
        self.stages.retain(|s| s.name() != name);

        if self.stages.len() == initial_len {
            return Err(ExtractionError::ConfigurationError(format!(
                "Stage '{}' not found",
                name
            )));
        }

        Ok(())
    }

    /// Get list of stage names
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name()).collect()
    }

    /// Execute the pipeline
    pub async fn execute(&self, input: ExtractionInput) -> Result<ExtractionOutput> {
        let start_time = Instant::now();
        let id = input.id.clone();

        info!("Starting extraction pipeline for {}", input.uri);

        // Create context
        let mut context =
            ExtractionContext::new(id.clone(), input.scope.clone(), self.config.clone());

        // Initialize output
        let mut output = ExtractionOutput::new(id.clone());

        // Execute stages based on mode
        let result = match self.config.execution_mode {
            ExecutionMode::Sequential => {
                self.execute_sequential(input, &mut output, &mut context)
                    .await
            }
            ExecutionMode::Parallel => {
                self.execute_parallel(input, &mut output, &mut context)
                    .await
            }
            ExecutionMode::Conditional => {
                self.execute_conditional(input, &mut output, &mut context)
                    .await
            }
        };

        // Update metrics
        output.metrics.total_duration_ms = start_time.elapsed().as_millis() as u64;

        // Update pipeline metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_executions += 1;
        if result.is_ok() {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }

        result?;

        info!(
            "Extraction pipeline completed in {}ms, extracted {} items",
            output.metrics.total_duration_ms,
            output.items.len()
        );

        Ok(output)
    }

    /// Execute stages sequentially
    async fn execute_sequential(
        &self,
        input: ExtractionInput,
        output: &mut ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        let current_input = input;

        for stage in &self.stages {
            let stage_name = stage.name();
            let stage_start = Instant::now();

            // Check if stage should be skipped
            if stage.should_skip(&current_input, context) {
                debug!("Skipping stage: {}", stage_name);
                continue;
            }

            debug!("Executing stage: {}", stage_name);

            // Execute stage with retry logic
            let result = self
                .execute_stage_with_retry(stage, current_input.clone(), output.clone(), context)
                .await?;

            // Update output
            *output = result;

            // Record timing
            let duration_ms = stage_start.elapsed().as_millis() as u64;
            output
                .metrics
                .stage_timings
                .insert(stage_name.to_string(), duration_ms);

            debug!("Stage {} completed in {}ms", stage_name, duration_ms);
        }

        Ok(())
    }

    /// Execute stages in parallel (where possible)
    async fn execute_parallel(
        &self,
        input: ExtractionInput,
        output: &mut ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        // For now, parallel execution is not implemented
        // Future: identify independent stages and execute them concurrently
        warn!("Parallel execution not yet implemented, falling back to sequential");
        self.execute_sequential(input, output, context).await
    }

    /// Execute stages with conditional branching
    async fn execute_conditional(
        &self,
        input: ExtractionInput,
        output: &mut ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<()> {
        // For now, conditional execution checks should_skip for each stage
        self.execute_sequential(input, output, context).await
    }

    /// Execute a stage with retry logic
    #[allow(clippy::borrowed_box)]
    async fn execute_stage_with_retry(
        &self,
        stage: &Box<dyn ExtractionStage>,
        input: ExtractionInput,
        output: ExtractionOutput,
        context: &mut ExtractionContext,
    ) -> Result<ExtractionOutput> {
        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            match stage.process(input.clone(), output.clone(), context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(ExtractionError::StageFailed {
                            name: stage.name().to_string(),
                            message: format!("Failed after {} retries: {}", retries, e),
                            source: Box::new(e),
                        });
                    }

                    warn!(
                        "Stage {} failed (attempt {}/{}): {}",
                        stage.name(),
                        retries,
                        max_retries,
                        e
                    );

                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(retries as u32)))
                        .await;
                }
            }
        }
    }

    /// Get pipeline statistics
    pub async fn get_stats(&self) -> PipelineStats {
        let metrics = self.metrics.read().await;

        PipelineStats {
            total_executions: metrics.total_executions,
            successful_executions: metrics.successful_executions,
            failed_executions: metrics.failed_executions,
            success_rate: if metrics.total_executions > 0 {
                metrics.successful_executions as f64 / metrics.total_executions as f64
            } else {
                0.0
            },
        }
    }
}

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64,
}

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = ExtractionPipeline::new(config);
        assert_eq!(pipeline.stage_names().len(), 0);
    }

    #[tokio::test]
    async fn test_pipeline_stats() {
        let pipeline = ExtractionPipeline::with_default_config();
        let stats = pipeline.get_stats().await;
        assert_eq!(stats.total_executions, 0);
    }
}
