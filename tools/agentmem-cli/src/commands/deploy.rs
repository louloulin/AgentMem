//! Deploy command implementation

use crate::config::MergedConfig;
use crate::commands::{CommandExecutor, DeploySubcommand};
use anyhow::Result;

pub struct DeployCommand {
    subcommand: DeploySubcommand,
}

impl DeployCommand {
    pub fn new(subcommand: DeploySubcommand) -> Self {
        Self { subcommand }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for DeployCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Deploying...");
        Ok(())
    }
}
