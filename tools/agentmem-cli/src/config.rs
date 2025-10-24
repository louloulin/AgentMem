/*!
Configuration management for AgentMem CLI
*/

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::Cli;

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// API configuration
    pub api: ApiConfig,

    /// Output configuration
    pub output: OutputConfig,

    /// Project configuration
    pub project: Option<ProjectConfig>,

    /// Deployment configuration
    pub deploy: Option<DeployConfig>,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API key for authentication
    pub api_key: Option<String>,

    /// Base URL for AgentMem API
    pub base_url: String,

    /// API version
    pub api_version: String,

    /// Request timeout in seconds
    pub timeout: u64,

    /// Maximum retry attempts
    pub max_retries: u32,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output format (json, yaml, table)
    pub format: String,

    /// Enable verbose output
    pub verbose: bool,

    /// Enable colored output
    pub color: bool,

    /// Page size for paginated results
    pub page_size: usize,
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,

    /// Project description
    pub description: Option<String>,

    /// Default agent ID
    pub default_agent_id: Option<String>,

    /// Project templates directory
    pub templates_dir: Option<PathBuf>,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Target platform (docker, kubernetes, aws, gcp, azure)
    pub platform: String,

    /// Environment (dev, staging, prod)
    pub environment: String,

    /// Resource configuration
    pub resources: ResourceConfig,

    /// Environment variables
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Resource configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// CPU limit
    pub cpu_limit: String,

    /// Memory limit
    pub memory_limit: String,

    /// Storage size
    pub storage_size: String,

    /// Number of replicas
    pub replicas: u32,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                api_key: None,
                base_url: "https://api.agentmem.dev".to_string(),
                api_version: "v1".to_string(),
                timeout: 30,
                max_retries: 3,
            },
            output: OutputConfig {
                format: "table".to_string(),
                verbose: false,
                color: true,
                page_size: 20,
            },
            project: None,
            deploy: None,
        }
    }
}

impl CliConfig {
    /// Load configuration from file or create default
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        if config_file.exists() {
            let content = fs::read_to_string(&config_file).with_context(|| {
                format!("Failed to read config file: {}", config_file.display())
            })?;

            let config: CliConfig =
                if config_file.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    serde_yaml::from_str(&content).with_context(|| "Failed to parse YAML config")?
                } else {
                    toml::from_str(&content).with_context(|| "Failed to parse TOML config")?
                };

            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    #[allow(dead_code)]
    pub fn save(&self, config_path: Option<&str>) -> Result<()> {
        let config_file = match config_path {
            Some(path) => PathBuf::from(path),
            None => Self::default_config_path()?,
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = if config_file.extension().and_then(|s| s.to_str()) == Some("yaml") {
            serde_yaml::to_string(self).with_context(|| "Failed to serialize config to YAML")?
        } else {
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config to TOML")?
        };

        fs::write(&config_file, content)
            .with_context(|| format!("Failed to write config file: {}", config_file.display()))?;

        Ok(())
    }

    /// Get default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("agentmem");

        Ok(config_dir.join("config.toml"))
    }

    /// Merge configuration with CLI arguments
    pub fn merge_with_cli(&self, cli: &Cli) -> Result<MergedConfig> {
        let mut config = self.clone();

        // Override with CLI arguments
        if let Some(api_key) = &cli.api_key {
            config.api.api_key = Some(api_key.clone());
        }

        config.api.base_url = cli.base_url.clone();
        config.api.api_version = cli.api_version.clone();
        config.output.verbose = cli.verbose;
        config.output.format = cli.format.clone();

        Ok(MergedConfig {
            api_key: config.api.api_key,
            base_url: config.api.base_url,
            api_version: config.api.api_version,
            timeout: config.api.timeout,
            max_retries: config.api.max_retries,
            format: config.output.format,
            verbose: config.output.verbose,
            color: config.output.color,
            page_size: config.output.page_size,
            project: config.project,
            deploy: config.deploy,
        })
    }

    /// Initialize a new project configuration
    #[allow(dead_code)]
    pub fn init_project(&mut self, name: String, description: Option<String>) {
        self.project = Some(ProjectConfig {
            name,
            description,
            default_agent_id: None,
            templates_dir: None,
        });
    }

    /// Set deployment configuration
    #[allow(dead_code)]
    pub fn set_deploy_config(&mut self, deploy_config: DeployConfig) {
        self.deploy = Some(deploy_config);
    }
}

/// Merged configuration from file and CLI arguments
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergedConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub api_version: String,
    pub timeout: u64,
    pub max_retries: u32,
    pub format: String,
    pub verbose: bool,
    pub color: bool,
    pub page_size: usize,
    pub project: Option<ProjectConfig>,
    pub deploy: Option<DeployConfig>,
}

impl MergedConfig {
    /// Get API base URL
    pub fn api_base_url(&self) -> String {
        format!("{}/api/{}", self.base_url, self.api_version)
    }

    /// Check if project is initialized
    #[allow(dead_code)]
    pub fn is_project_initialized(&self) -> bool {
        self.project.is_some()
    }

    /// Get project name
    #[allow(dead_code)]
    pub fn project_name(&self) -> Option<&str> {
        self.project.as_ref().map(|p| p.name.as_str())
    }

    /// Get default agent ID
    #[allow(dead_code)]
    pub fn default_agent_id(&self) -> Option<&str> {
        self.project
            .as_ref()
            .and_then(|p| p.default_agent_id.as_deref())
    }
}
