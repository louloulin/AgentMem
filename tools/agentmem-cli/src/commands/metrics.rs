//! Metrics command implementation

use crate::config::MergedConfig;
use crate::commands::CommandExecutor;
use anyhow::Result;

pub struct MetricsCommand;

impl MetricsCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl CommandExecutor for MetricsCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Metrics: OK");
        Ok(())
    }
}
