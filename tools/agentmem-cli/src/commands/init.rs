//! Init command implementation

use crate::config::MergedConfig;
use crate::commands::CommandExecutor;
use anyhow::Result;

pub struct InitCommand {
    name: String,
    template: String,
}

impl InitCommand {
    pub fn new(name: String, template: String) -> Self {
        Self { name, template }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for InitCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Initializing project: {} with template: {}", self.name, self.template);
        Ok(())
    }
}
