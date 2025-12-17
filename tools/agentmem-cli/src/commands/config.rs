//! Config command implementation

use crate::config::MergedConfig;
use crate::commands::{CommandExecutor, ConfigSubcommand};
use anyhow::Result;

pub struct ConfigCommand {
    subcommand: ConfigSubcommand,
}

impl ConfigCommand {
    pub fn new(subcommand: ConfigSubcommand) -> Self {
        Self { subcommand }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for ConfigCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        match &self.subcommand {
            ConfigSubcommand::Show => {
                println!("Showing config...");
            }
            ConfigSubcommand::Set { key, value } => {
                println!("Setting {} = {}", key, value);
            }
            ConfigSubcommand::Get { key } => {
                println!("Getting {}", key);
            }
            ConfigSubcommand::Init { .. } => {
                println!("Initializing config...");
            }
            ConfigSubcommand::Validate => {
                println!("Validating config...");
            }
        }
        Ok(())
    }
}
