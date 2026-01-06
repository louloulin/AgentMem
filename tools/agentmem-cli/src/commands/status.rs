//! Status command implementation

use crate::config::MergedConfig;
use crate::commands::CommandExecutor;
use anyhow::Result;

pub struct StatusCommand;

impl StatusCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl CommandExecutor for StatusCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Status: OK");
        Ok(())
    }
}
