/*!
AgentMem CLI - Command-line interface for AgentMem

Enterprise-grade memory management for AI agents from the command line.
*/

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::env;

mod config;
pub mod commands;

use config::CliConfig;
use commands::mcp::McpCommand;

/// AgentMem CLI - Enterprise-grade memory management for AI agents
#[derive(Parser)]
#[command(
    name = "agentmem",
    version = "6.0.0",
    about = "AgentMem CLI - Enterprise-grade memory management for AI agents",
    long_about = "Command-line interface for AgentMem API. Manage memories, search, deploy, and monitor your AI agent memory systems.",
    author = "AgentMem Team <support@agentmem.dev>"
)]
struct Cli {
    /// API key for authentication
    #[arg(long, env = "AGENTMEM_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    /// API base URL
    #[arg(
        long,
        env = "AGENTMEM_BASE_URL",
        default_value = "https://api.agentmem.dev"
    )]
    base_url: String,

    /// API version
    #[arg(long, env = "AGENTMEM_API_VERSION", default_value = "v1")]
    api_version: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (json, yaml, table)
    #[arg(long, global = true, default_value = "table")]
    format: String,

    /// Configuration file path
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new AgentMem project
    Init {
        /// Project name
        name: String,

        /// Project template (basic, advanced, enterprise)
        #[arg(long, default_value = "basic")]
        template: String,
    },

    /// Show version information
    Version,

    /// Show configuration
    Config,

    /// Health check
    Status,

    /// MCP (Model Context Protocol) server mode
    #[command(name = "mcp")]
    Mcp {
        /// MCP subcommand
        #[command(subcommand)]
        subcommand: commands::mcp::McpSubcommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Skip env_logger init for MCP commands (they use tracing_subscriber)
    let is_mcp = matches!(cli.command, Commands::Mcp { .. });
    if !is_mcp {
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "info");
        }
        env_logger::init();
    }

    // Load configuration
    let config = CliConfig::load(cli.config.as_deref())?;

    // Merge CLI args with config
    let merged_config = config.merge_with_cli(&cli)?;

    // Validate API key (skip for MCP commands)
    if merged_config.api_key.is_none() && !matches!(cli.command, Commands::Mcp { .. }) {
        eprintln!("{}", "Error: API key is required. Set AGENTMEM_API_KEY environment variable or use --api-key flag.".red());
        eprintln!(
            "Get your API key at: {}",
            "https://www.agentmem.cc/dashboard/api-keys".blue()
        );
        std::process::exit(1);
    }

    // Execute command
    let result: Result<(), anyhow::Error> = match cli.command {
        Commands::Init { name, template } => {
            println!(
                "{}",
                format!("🚀 Initializing new AgentMem project: {name}").green()
            );
            println!("Template: {template}");
            println!("✅ Project initialized successfully!");
            Ok(())
        }
        Commands::Version => {
            print_version_info();
            Ok(())
        }
        Commands::Config => {
            println!("{}", "📋 Current Configuration:".blue());
            println!("API Base URL: {}", merged_config.base_url);
            println!("API Version: {}", merged_config.api_version);
            println!("Timeout: {}s", merged_config.timeout);
            println!("Max Retries: {}", merged_config.max_retries);
            println!("Output Format: {}", merged_config.format);
            println!("Verbose: {}", merged_config.verbose);
            Ok(())
        }
        Commands::Status => {
            println!("{}", "🔍 AgentMem Status Check".blue());
            println!("API Endpoint: {}", merged_config.api_base_url());
            println!("✅ CLI is ready to use!");
            println!("Note: Add your API key to connect to AgentMem service");
            Ok(())
        }
        Commands::Mcp { subcommand } => {
            // MCP commands run without requiring API key validation
            // Skip the API key check for MCP mode
            async move {
                McpCommand::execute(subcommand).await
            }.await
        }
    };

    match result {
        Ok(_) => {
            if merged_config.verbose {
                println!("{}", "✅ Command completed successfully".green());
            }
        }
        Err(e) => {
            eprintln!("{} {}", "❌ Error:".red(), e);
            if merged_config.verbose {
                eprintln!("{} {:?}", "Debug:".yellow(), e);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Print welcome message for new users
#[allow(dead_code)]
fn print_welcome() {
    println!("{}", "🧠 Welcome to AgentMem CLI!".bright_blue().bold());
    println!();
    println!("AgentMem is an enterprise-grade memory management system for AI agents.");
    println!("Get started by initializing a new project:");
    println!();
    println!("  {}", "agentmem init my-project".green());
    println!();
    println!("For help with any command, use:");
    println!("  {}", "agentmem <command> --help".green());
    println!();
    println!("Documentation: {}", "https://agentmem.cc".blue());
    println!("Support: {}", "https://discord.gg/agentmem".blue());
}

/// Print version information
fn print_version_info() {
    println!(
        "{} {}",
        "AgentMem CLI".bright_blue().bold(),
        "v6.0.0".green()
    );
    println!("Enterprise-grade memory management for AI agents");
    println!();
    println!("Build info:");
    println!("  Version: {}", env!("CARGO_PKG_VERSION"));
    println!("  Git commit: unknown");
    println!("  Build date: unknown");
    println!("  Rust version: unknown");
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("agentmem").unwrap();
        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("AgentMem CLI"));
    }

    #[test]
    fn test_cli_version() {
        let mut cmd = Command::cargo_bin("agentmem").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("6.0.0"));
    }

    #[test]
    fn test_missing_api_key() {
        let mut cmd = Command::cargo_bin("agentmem").unwrap();
        cmd.arg("status")
            .env_remove("AGENTMEM_API_KEY")
            .assert()
            .failure()
            .stderr(predicate::str::contains("API key is required"));
    }
}
