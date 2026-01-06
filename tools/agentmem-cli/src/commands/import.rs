//! Import command implementation

use crate::config::MergedConfig;
use crate::commands::CommandExecutor;
use anyhow::Result;

pub struct ImportCommand {
    file: String,
}

impl ImportCommand {
    pub fn new(file: String) -> Self {
        Self { file }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for ImportCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Importing from: {}", self.file);
        Ok(())
    }
}
