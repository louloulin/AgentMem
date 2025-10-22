//! AgentMem Server Binary
//!
//! Standalone server for AgentMem memory management platform.

use agent_mem_server::{MemoryServer, ServerConfig};
use clap::Parser;
use std::process;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "agent-mem-server")]
#[command(about = "AgentMem REST API Server")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Server host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Enable CORS
    #[arg(long, default_value = "true")]
    cors: bool,

    /// Enable authentication
    #[arg(long, default_value = "false")]
    auth: bool,

    /// JWT secret (required if auth is enabled)
    #[arg(long)]
    jwt_secret: Option<String>,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Create server configuration
    // Phase 10 MVP P0-1: 配置文件加载系统 ✅
    let mut config = match ServerConfig::load(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Override with CLI arguments
    config.port = cli.port;
    config.host = cli.host;
    config.enable_cors = cli.cors;
    config.enable_auth = cli.auth;
    config.log_level = cli.log_level;

    if cli.auth {
        if let Some(secret) = cli.jwt_secret {
            config.jwt_secret = secret;
        } else {
            eprintln!("Error: JWT secret is required when authentication is enabled");
            eprintln!("Use --jwt-secret <SECRET> or set AGENT_MEM_JWT_SECRET environment variable");
            process::exit(1);
        }
    }

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration error: {}", e);
        process::exit(1);
    }

    // Create and start server
    match MemoryServer::new(config).await {
        Ok(server) => {
            info!("Starting AgentMem server...");

            // Setup graceful shutdown
            let shutdown_signal = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to install CTRL+C signal handler");
                info!("Shutdown signal received");
            };

            // Start server with graceful shutdown
            tokio::select! {
                result = server.start() => {
                    if let Err(e) = result {
                        error!("Server error: {}", e);
                        process::exit(1);
                    }
                }
                _ = shutdown_signal => {
                    info!("Shutting down server gracefully...");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            process::exit(1);
        }
    }
}
