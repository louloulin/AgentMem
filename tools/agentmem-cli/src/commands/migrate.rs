//! Migrate command implementation

use crate::config::MergedConfig;
use crate::commands::{CommandExecutor, MigrateSubcommand};
use anyhow::Result;

pub struct MigrateCommand {
    subcommand: MigrateSubcommand,
}

impl MigrateCommand {
    pub fn new(subcommand: MigrateSubcommand) -> Self {
        Self { subcommand }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for MigrateCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Running migration...");
        Ok(())
    }
}
