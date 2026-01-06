# Getting Started with AgentMem

This guide will help you get up and running with AgentMem in 5 minutes.

## Prerequisites

- **Rust**: 1.75 or higher ([Install Rust](https://www.rust-lang.org/tools/install))
- **LLM API Key**: OpenAI, Anthropic, or DeepSeek (optional but recommended)
- **Operating System**: Linux, macOS, or Windows

## Installation

### Option 1: Cargo Install (Recommended)

```bash
# Add to your Rust project
cargo add agent-mem

# Or install the CLI tool
cargo install agentmem-cli
```

### Option 2: Build from Source

```bash
git clone https://github.com/louloulin/agentmem.git
cd agentmem
cargo build --release
```

### Option 3: Docker

```bash
docker pull agentmem/agentmem:latest
docker run -p 8080:8080 agentmem/agentmem:latest
```

## Your First AgentMem Application

### Step 1: Create a New Project

```bash
cargo new my_agent_app
cd my_agent_app
```

### Step 2: Add Dependency

Edit `Cargo.toml`:

```toml
[dependencies]
agent-mem = "2.0"
tokio = { version = "1", features = ["full"] }
```

### Step 3: Write Your Code

Edit `src/main.rs`:

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your API key (or use DEEPSEEK_API_KEY, ANTHROPIC_API_KEY)
    std::env::set_var("OPENAI_API_KEY", "sk-...");

    // Initialize AgentMem - zero configuration required!
    let memory = Memory::new().await?;

    // Add some memories
    memory.add("My name is Alice").await?;
    memory.add("I love Rust programming").await?;
    memory.add("I work as a software engineer").await?;

    // Search memories
    let results = memory.search("What do you know about me?").await?;

    println!("Found {} memories:", results.len());
    for result in results {
        println!("  - {}", result.memory);
    }

    Ok(())
}
```

### Step 4: Run Your App

```bash
cargo run
```

Expected output:

```
Found 3 memories:
  - My name is Alice
  - I love Rust programming
  - I work as a software engineer
```

## Next Steps

### Configure Your API Keys

AgentMem supports multiple LLM providers:

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# DeepSeek (recommended for cost-effectiveness)
export DEEPSEEK_API_KEY="..."

# Anthropic Claude
export ANTHROPIC_API_KEY="sk-..."

# Or create a .env file
echo "OPENAI_API_KEY=sk-..." > .env
```

### Advanced Configuration

```rust
use agent_mem::{Memory, MemoryBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Custom configuration
    let memory = Memory::builder()
        .storage_backend("postgres")  // Use PostgreSQL instead of SQLite
        .vector_store("lancedb")      // Use LanceDB for vectors
        .embedder_model("BAAI/bge-small-en-v1.5")  // Custom embedding model
        .build()
        .await?;

    Ok(())
}
```

## Running the Server

For a full HTTP API and web UI:

```bash
# Start the server
cargo run --bin agent-mem-server

# Or use the just command
just start-full
```

Access points:
- **API**: http://localhost:8080
- **Web UI**: http://localhost:3001
- **API Docs**: http://localhost:8080/swagger-ui/
- **Health Check**: http://localhost:8080/health

## Common Use Cases

### Chatbot Memory

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // User says something
    let user_message = "Hi, I'm Bob and I prefer tea over coffee";

    // Store it
    memory.add(user_message).await?;

    // Later, retrieve context
    let context = memory.search("User preferences", "user123").await?;

    Ok(())
}
```

### Knowledge Base

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // Add documents
    memory.add("Company policy: Remote work allowed on Fridays").await?;
    memory.add("Office hours: 9 AM to 5 PM EST").await?;

    // Query the knowledge base
    let results = memory.search("remote work policy").await?;

    Ok(())
}
```

### Multi-Agent System

```rust
use agent_mem::{Memory, MemoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // Different scopes for different agents
    let agent_scope = MemoryScope::Agent {
        user_id: "alice",
        agent_id: "coding-assistant"
    };

    memory.add_with_scope("User prefers Rust over Go", agent_scope).await?;

    Ok(())
}
```

## Troubleshooting

### Issue: "OPENAI_API_KEY not set"

**Solution**: Set your API key in environment variables:
```bash
export OPENAI_API_KEY="sk-..."
```

### Issue: "Database connection error"

**Solution**: AgentMem uses LibSQL by default (no setup required). If you see this error, check:
1. File permissions in the current directory
2. Disk space availability

### Issue: Slow embedding generation

**Solution**:
1. Use a faster embedding model (e.g., `BAAI/bge-small-en-v1.5`)
2. Enable caching (enabled by default)
3. Use DeepSeek API (faster and cheaper than OpenAI)

## What's Next?

- 📖 [Core Concepts](./core-concepts.md) - Understanding memory scopes and types
- 🔍 [Advanced Search](./advanced-search.md) - Using the 5 search engines
- 🧠 [Intelligent Features](./intelligent-memory.md) - Fact extraction and reasoning
- 🔌 [Plugin Development](../developer-guide/plugin-development.md) - Building WASM plugins

## Getting Help

- 📚 [Documentation](https://docs.agentmem.dev)
- 💬 [Discord Community](https://discord.gg/agentmem)
- 🐛 [Report Issues](https://github.com/louloulin/agentmem/issues)
- ✉️ [Email Support](mailto:support@agentmem.dev)
