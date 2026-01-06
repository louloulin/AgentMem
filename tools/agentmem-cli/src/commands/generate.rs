//! Generate command implementation

use crate::config::MergedConfig;
use crate::commands::{CommandExecutor, GenerateSubcommand};
use anyhow::Result;

pub struct GenerateCommand {
    subcommand: GenerateSubcommand,
}

impl GenerateCommand {
    pub fn new(subcommand: GenerateSubcommand) -> Self {
        Self { subcommand }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for GenerateCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Generating...");
        Ok(())
    }
}
