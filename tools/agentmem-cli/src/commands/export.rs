//! Export command implementation

use crate::config::MergedConfig;
use crate::commands::CommandExecutor;
use anyhow::Result;

pub struct ExportCommand {
    file: String,
}

impl ExportCommand {
    pub fn new(file: String) -> Self {
        Self { file }
    }
}

#[async_trait::async_trait]
impl CommandExecutor for ExportCommand {
    async fn execute(&self, _config: &MergedConfig) -> Result<()> {
        println!("Exporting to: {}", self.file);
        Ok(())
    }
}
