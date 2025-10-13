//! Agent from Environment Demo
//!
//! This example demonstrates the new `from_env()` factory method for agents,
//! which automatically configures storage from environment variables.
//!
//! # Usage
//!
//! ```bash
//! # Use default LibSQL database (agentmem.db)
//! cargo run --example agent-from-env-demo
//!
//! # Use custom LibSQL path
//! AGENTMEM_DB_PATH="./data/memory.db" cargo run --example agent-from-env-demo
//!
//! # Use PostgreSQL
//! DATABASE_URL="postgresql://user:pass@localhost/agentmem" cargo run --example agent-from-env-demo
//! ```

use agent_mem_core::agents::{CoreAgent, EpisodicAgent, SemanticAgent, MemoryAgent};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Agent from Environment Demo");
    info!("================================");
    info!("");
    info!("This demo shows how to create agents with automatic storage configuration");
    info!("from environment variables. No manual store setup required!");
    info!("");

    // Example 1: Create CoreAgent from environment
    info!("📝 Example 1: CoreAgent from environment");
    info!("   Creating CoreAgent with auto-configured storage...");
    match CoreAgent::from_env("core-agent-1".to_string()).await {
        Ok(mut agent) => {
            info!("   ✅ CoreAgent created successfully!");
            info!("   Agent ID: {}", agent.agent_id());

            // Initialize the agent
            match agent.initialize().await {
                Ok(_) => info!("   ✅ Agent initialized successfully"),
                Err(e) => error!("   ❌ Failed to initialize agent: {}", e),
            }

            // Check health
            if agent.health_check().await {
                info!("   ✅ Agent health check passed");
            } else {
                error!("   ❌ Agent health check failed");
            }

            // Get stats
            let stats = agent.get_stats().await;
            info!("   📊 Agent stats: {} total tasks, {} successful", stats.total_tasks, stats.successful_tasks);
        }
        Err(e) => {
            error!("   ❌ Failed to create CoreAgent: {}", e);
            error!("   Make sure database is accessible");
        }
    }

    // Example 2: Create EpisodicAgent from environment
    info!("");
    info!("📝 Example 2: EpisodicAgent from environment");
    info!("   Creating EpisodicAgent with auto-configured storage...");
    match EpisodicAgent::from_env("episodic-agent-1".to_string()).await {
        Ok(mut agent) => {
            info!("   ✅ EpisodicAgent created successfully!");
            info!("   Agent ID: {}", agent.agent_id());

            // Initialize the agent
            match agent.initialize().await {
                Ok(_) => info!("   ✅ Agent initialized successfully"),
                Err(e) => error!("   ❌ Failed to initialize agent: {}", e),
            }

            // Check health
            if agent.health_check().await {
                info!("   ✅ Agent health check passed");
            }
        }
        Err(e) => {
            error!("   ❌ Failed to create EpisodicAgent: {}", e);
        }
    }

    // Example 3: Create SemanticAgent from environment
    info!("");
    info!("📝 Example 3: SemanticAgent from environment");
    info!("   Creating SemanticAgent with auto-configured storage...");
    match SemanticAgent::from_env("semantic-agent-1".to_string()).await {
        Ok(mut agent) => {
            info!("   ✅ SemanticAgent created successfully!");
            info!("   Agent ID: {}", agent.agent_id());

            // Initialize the agent
            match agent.initialize().await {
                Ok(_) => info!("   ✅ Agent initialized successfully"),
                Err(e) => error!("   ❌ Failed to initialize agent: {}", e),
            }

            // Check health
            if agent.health_check().await {
                info!("   ✅ Agent health check passed");
            }
        }
        Err(e) => {
            error!("   ❌ Failed to create SemanticAgent: {}", e);
        }
    }

    info!("");
    info!("✅ Demo completed!");
    info!("   All agents were created with automatic storage configuration");
    info!("   Database file: agentmem.db (default) or custom path from AGENTMEM_DB_PATH");
    info!("");
    info!("💡 Try running with custom configuration:");
    info!("   AGENTMEM_DB_PATH=\"./data/memory.db\" cargo run --example agent-from-env-demo");

    Ok(())
}

