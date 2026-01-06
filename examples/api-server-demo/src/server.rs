//! AgentMem API Server Demo
//!
//! This demo shows how to start the AgentMem REST API server.

use agent_mem_server::{MemoryServer, ServerConfig};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting AgentMem API Server Demo");

    // Create server configuration
    let mut config = ServerConfig::default();
    config.port = 8080;
    config.enable_logging = false; // Disable server logging since we already initialized tracing
    config.enable_metrics = true;
    config.enable_cors = true;

    info!("📋 Server Configuration:");
    info!("  - Port: {}", config.port);
    info!("  - Logging: {}", config.enable_logging);
    info!("  - Metrics: {}", config.enable_metrics);
    info!("  - CORS: {}", config.enable_cors);

    // Create and start the server
    match MemoryServer::new(config).await {
        Ok(server) => {
            info!("✅ Server created successfully");
            info!("🌐 Starting server on http://localhost:8080");
            info!("📚 API Documentation: http://localhost:8080/swagger-ui/");
            info!("❤️  Health Check: http://localhost:8080/health");
            info!("📊 Metrics: http://localhost:8080/metrics");

            // Start the server (this will block)
            if let Err(e) = server.start().await {
                error!("❌ Failed to start server: {}", e);
                return Err(e.into());
            }
        }
        Err(e) => {
            error!("❌ Failed to create server: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
