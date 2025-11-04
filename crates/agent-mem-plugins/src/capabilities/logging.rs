//! Logging capability

use anyhow::Result;

/// Logging capability - provides logging functionality to plugins
pub struct LoggingCapability {}

impl LoggingCapability {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Log a message
    pub fn log(&self, level: &str, message: &str) -> Result<()> {
        // Use standard logging
        match level {
            "error" => eprintln!("[PLUGIN ERROR] {}", message),
            "warn" => eprintln!("[PLUGIN WARN] {}", message),
            "info" => println!("[PLUGIN INFO] {}", message),
            "debug" => println!("[PLUGIN DEBUG] {}", message),
            _ => println!("[PLUGIN] {}", message),
        }
        Ok(())
    }
}

impl Default for LoggingCapability {
    fn default() -> Self {
        Self::new()
    }
}

