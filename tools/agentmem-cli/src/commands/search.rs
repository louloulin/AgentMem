//! Search command implementation

use crate::config::MergedConfig;
use crate::commands::{CommandExecutor, SearchSubcommand};
use anyhow::Result;

pub struct SearchCommand {
    subcommand: SearchSubcommand,
}

impl SearchCommand {
    pub fn new(subcommand: SearchSubcommand) -> Self {
        Self { subcommand }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for SearchCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        match &self.subcommand {
            SearchSubcommand::Text { query, .. } => {
                println!("Searching for: {}", query);
            }
            SearchSubcommand::Vector { vector, .. } => {
                println!("Vector search: {}", vector);
            }
        }
        Ok(())
    }
}
