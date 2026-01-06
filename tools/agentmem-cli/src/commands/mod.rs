/*!
Command implementations for AgentMem CLI
*/

use anyhow::Result;
use clap::Subcommand;

use crate::config::MergedConfig;

pub mod init;
pub mod memory;
pub mod search;
pub mod deploy;
pub mod migrate;
pub mod benchmark;
pub mod config;
pub mod status;
pub mod metrics;
pub mod generate;
pub mod import;
pub mod export;
/// Phase 3.3: Debug commands
pub mod debug;

pub use init::InitCommand;
pub use memory::MemoryCommand;
pub use search::SearchCommand;
pub use deploy::DeployCommand;
pub use migrate::MigrateCommand;
pub use benchmark::BenchmarkCommand;
pub use config::ConfigCommand;
pub use status::StatusCommand;
pub use metrics::MetricsCommand;
pub use generate::GenerateCommand;
pub use import::ImportCommand;
pub use export::ExportCommand;
pub use debug::{DebugCommand, DebugSubcommand};

/// Trait for command execution
pub trait CommandExecutor {
    async fn execute(&self, config: &MergedConfig) -> Result<()>;
}

/// Memory management commands
#[derive(Subcommand)]
pub enum MemorySubcommand {
    /// Add a new memory
    Add {
        /// Memory content
        content: String,
        
        /// Agent ID
        #[arg(long)]
        agent_id: String,
        
        /// Memory type (episodic, semantic, procedural, untyped)
        #[arg(long, default_value = "untyped")]
        memory_type: String,
        
        /// User ID
        #[arg(long)]
        user_id: Option<String>,
        
        /// Session ID
        #[arg(long)]
        session_id: Option<String>,
        
        /// Importance score (0.0 to 1.0)
        #[arg(long, default_value = "0.5")]
        importance: f64,
        
        /// Metadata as JSON string
        #[arg(long)]
        metadata: Option<String>,
    },
    
    /// Get a memory by ID
    Get {
        /// Memory ID
        id: String,
    },
    
    /// Update an existing memory
    Update {
        /// Memory ID
        id: String,
        
        /// New content
        #[arg(long)]
        content: Option<String>,
        
        /// New importance score
        #[arg(long)]
        importance: Option<f64>,
        
        /// New metadata as JSON string
        #[arg(long)]
        metadata: Option<String>,
    },
    
    /// Delete a memory
    Delete {
        /// Memory ID
        id: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    
    /// List memories for an agent
    List {
        /// Agent ID
        #[arg(long)]
        agent_id: String,
        
        /// Memory type filter
        #[arg(long)]
        memory_type: Option<String>,
        
        /// User ID filter
        #[arg(long)]
        user_id: Option<String>,
        
        /// Minimum importance
        #[arg(long)]
        min_importance: Option<f64>,
        
        /// Maximum age in seconds
        #[arg(long)]
        max_age: Option<u64>,
        
        /// Number of results to return
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    
    /// Get memory statistics
    Stats {
        /// Agent ID
        #[arg(long)]
        agent_id: String,
    },
}

/// Search commands
#[derive(Subcommand)]
pub enum SearchSubcommand {
    /// Search memories by text
    Text {
        /// Search query
        query: String,
        
        /// Agent ID
        #[arg(long)]
        agent_id: String,
        
        /// Memory type filter
        #[arg(long)]
        memory_type: Option<String>,
        
        /// User ID filter
        #[arg(long)]
        user_id: Option<String>,
        
        /// Minimum importance
        #[arg(long)]
        min_importance: Option<f64>,
        
        /// Maximum age in seconds
        #[arg(long)]
        max_age: Option<u64>,
        
        /// Number of results to return
        #[arg(long, default_value = "10")]
        limit: usize,
        
        /// Metadata filters as JSON string
        #[arg(long)]
        metadata_filters: Option<String>,
    },
    
    /// Search memories by vector
    Vector {
        /// Vector query as JSON array
        vector: String,
        
        /// Agent ID
        #[arg(long)]
        agent_id: String,
        
        /// Number of results to return
        #[arg(long, default_value = "10")]
        limit: usize,
    },
}

/// Deployment commands
#[derive(Subcommand)]
pub enum DeploySubcommand {
    /// Deploy to Docker
    Docker {
        /// Docker image tag
        #[arg(long, default_value = "latest")]
        tag: String,
        
        /// Docker registry
        #[arg(long)]
        registry: Option<String>,
        
        /// Environment variables file
        #[arg(long)]
        env_file: Option<String>,
    },
    
    /// Deploy to Kubernetes
    Kubernetes {
        /// Kubernetes namespace
        #[arg(long, default_value = "default")]
        namespace: String,
        
        /// Kubernetes context
        #[arg(long)]
        context: Option<String>,
        
        /// Deployment configuration file
        #[arg(long)]
        config_file: Option<String>,
    },
    
    /// Deploy to AWS
    Aws {
        /// AWS region
        #[arg(long, default_value = "us-east-1")]
        region: String,
        
        /// ECS cluster name
        #[arg(long)]
        cluster: Option<String>,
        
        /// CloudFormation template
        #[arg(long)]
        template: Option<String>,
    },
    
    /// Deploy to Google Cloud Platform
    Gcp {
        /// GCP project ID
        #[arg(long)]
        project_id: String,
        
        /// GCP region
        #[arg(long, default_value = "us-central1")]
        region: String,
        
        /// Cloud Run service name
        #[arg(long)]
        service_name: Option<String>,
    },
    
    /// Deploy to Microsoft Azure
    Azure {
        /// Azure resource group
        #[arg(long)]
        resource_group: String,
        
        /// Azure region
        #[arg(long, default_value = "eastus")]
        region: String,
        
        /// Container instance name
        #[arg(long)]
        instance_name: Option<String>,
    },
}

/// Migration commands
#[derive(Subcommand)]
pub enum MigrateSubcommand {
    /// Run database migrations
    Up {
        /// Number of migrations to run (default: all)
        #[arg(long)]
        steps: Option<u32>,
    },
    
    /// Rollback database migrations
    Down {
        /// Number of migrations to rollback
        #[arg(long, default_value = "1")]
        steps: u32,
    },
    
    /// Show migration status
    Status,
    
    /// Create a new migration
    Create {
        /// Migration name
        name: String,
    },
    
    /// Reset database (drop all tables and re-run migrations)
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

/// Configuration commands
#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Show current configuration
    Show,
    
    /// Set configuration value
    Set {
        /// Configuration key (e.g., api.base_url)
        key: String,
        
        /// Configuration value
        value: String,
    },
    
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Initialize configuration file
    Init {
        /// Configuration file path
        #[arg(long)]
        path: Option<String>,
        
        /// Overwrite existing configuration
        #[arg(long)]
        force: bool,
    },
    
    /// Validate configuration
    Validate,
}

/// Generate commands
#[derive(Subcommand)]
pub enum GenerateSubcommand {
    /// Generate project template
    Project {
        /// Project name
        name: String,
        
        /// Project template (basic, advanced, enterprise)
        #[arg(long, default_value = "basic")]
        template: String,
        
        /// Output directory
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Generate SDK code
    Sdk {
        /// Target language (python, javascript, go, rust)
        language: String,
        
        /// Output directory
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Generate deployment configuration
    Deploy {
        /// Target platform (docker, kubernetes, aws, gcp, azure)
        platform: String,
        
        /// Output directory
        #[arg(long)]
        output: Option<String>,
    },
    
    /// Generate API documentation
    Docs {
        /// Documentation format (markdown, html, openapi)
        #[arg(long, default_value = "markdown")]
        format: String,
        
        /// Output directory
        #[arg(long)]
        output: Option<String>,
    },
}
