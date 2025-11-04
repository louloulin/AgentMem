//! Plugin trait and lifecycle management

use crate::types::*;
use anyhow::Result;

/// Plugin lifecycle trait
pub trait Plugin {
    /// Initialize the plugin
    /// Called once when the plugin is loaded
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// Start the plugin
    /// Called after initialization when the plugin is ready
    fn start(&mut self) -> Result<()>;
    
    /// Stop the plugin
    /// Called before the plugin is unloaded
    fn stop(&mut self) -> Result<()>;
    
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
}

/// Memory processor trait
pub trait MemoryProcessor {
    /// Process a memory object
    fn process_memory(&self, memory: Memory) -> Result<Memory>;
    
    /// Check if this processor can handle the memory type
    fn can_process(&self, memory_type: &str) -> bool;
}

/// Code analyzer trait
pub trait CodeAnalyzer {
    /// Analyze code and extract information
    fn analyze_code(&self, input: &CodeInput) -> Result<CodeAnalysis>;
    
    /// Extract patterns from code
    fn extract_patterns(&self, code: &str) -> Result<Vec<CodePattern>>;
    
    /// Find dependencies in code
    fn find_dependencies(&self, code: &str) -> Result<Vec<String>>;
}

/// Search algorithm trait
pub trait SearchAlgorithm {
    /// Search memories based on query
    fn search(&self, query: &str, candidates: Vec<Memory>) -> Result<Vec<SearchResult>>;
    
    /// Compute similarity between query and memory
    fn compute_similarity(&self, query: &str, memory: &Memory) -> Result<f32>;
    
    /// Rerank search results
    fn rerank(&self, results: Vec<SearchResult>) -> Result<Vec<SearchResult>>;
}

