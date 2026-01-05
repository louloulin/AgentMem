# AgentMem

<div align="center">

**Enterprise-Grade Memory Management for AI Agents**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/agentmem/agentmem/actions)
[![docs.rs](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/agent-mem)

[Quick Start](#quick-start) • [Documentation](#documentation) • [Examples](#examples) • [Contributing](CONTRIBUTING.md)

</div>

---

## 🧠 What is AgentMem?

AgentMem is a high-performance memory management system designed for AI agents and LLM-powered applications. It enables intelligent, persistent memory with semantic search, automatic fact extraction, and multi-backend support.

### Why AgentMem?

**Without AgentMem:**
- AI agents have no memory between conversations
- Users must repeat context every time
- No personalization or learning
- Expensive token costs for long conversations

**With AgentMem:**
- ✅ Persistent memory across sessions
- ✅ Semantic search in milliseconds
- ✅ Automatic fact extraction from conversations
- ✅ 90%+ reduction in token costs

### Key Features

- **🚀 High Performance** - 216K ops/sec with sub-millisecond latency
- **🧠 Intelligent Memory** - AI-powered fact extraction and reasoning
- **🔍 Semantic Search** - Vector, BM25, full-text, fuzzy, and hybrid search
- **🔌 Extensible** - WASM plugin system with hot-reload
- **💾 Multi-Backend** - LibSQL, PostgreSQL, Redis, LanceDB support
- **🤖 LLM Integration** - 20+ providers including OpenAI, Anthropic, DeepSeek
- **🛡️ Production Ready** - RBAC, observability, horizontal scaling

## 🚀 Quick Start

### Installation

```bash
# Install with cargo
cargo add agent-mem

# Or clone the repository
git clone https://github.com/agentmem/agentmem.git
cd agentmem
```

### Basic Usage

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set your LLM API key
    std::env::set_var("OPENAI_API_KEY", "sk-...");

    // Initialize with zero configuration
    let memory = Memory::new().await?;

    // Add memories with automatic fact extraction
    memory.add("I love pizza and live in San Francisco").await?;
    memory.add("My favorite food is pizza").await?;  // Auto-deduplicated

    // Semantic search
    let results = memory.search("What do you know about me?").await?;
    for result in results {
        println!("- {}", result.memory);
    }

    Ok(())
}
```

**That's it!** AgentMem automatically handles:
- Embedding generation and vector search
- Fact extraction and deduplication
- Memory storage and retrieval
- Conflict resolution

For more examples, see [examples.md](docs/user-guide/examples.md).

## 📚 Documentation

### For Users
- **[Installation Guide](INSTALL.md)** - Detailed setup instructions
- **[Quick Start Guide](QUICKSTART.md)** - Get started in 5 minutes
- **[Core Concepts](docs/user-guide/core-concepts.md)** - Understanding the memory system
- **[API Reference](docs/user-guide/api-reference.md)** - Complete API documentation
- **[Configuration](docs/user-guide/configuration.md)** - Configuration options
- **[Troubleshooting](docs/user-guide/troubleshooting.md)** - Common issues and solutions

### For Developers
- **[Architecture](docs/developer-guide/architecture.md)** - System architecture overview
- **[Development Setup](docs/developer-guide/development-setup.md)** - Setting up dev environment
- **[Plugin Development](docs/developer-guide/plugin-development.md)** - Creating WASM plugins
- **[Testing Guide](docs/developer-guide/testing-guide.md)** - Running and writing tests
- **[Contributing](CONTRIBUTING.md)** - Contribution guidelines

### For Operations
- **[Production Setup](docs/deployment/production-setup.md)** - Production deployment
- **[Scaling](docs/deployment/scaling.md)** - Scaling strategies
- **[Monitoring](docs/deployment/monitoring.md)** - Observability and monitoring
- **[Migration Guide](docs/deployment/migration-guide.md)** - Migrating from Mem0

## 💡 Examples

### 1. Basic Memory Operations

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // Add memories
    memory.add("User prefers dark mode").await?;
    memory.add("User is based in New York").await?;

    // Search memories
    let results = memory.search("user preferences").await?;

    Ok(())
}
```

### 2. Multi-User Memory

```rust
use agent_mem::{Memory, MemoryScope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // Isolate memories by user
    let scope = MemoryScope::User {
        user_id: "user123".to_string(),
    };

    memory.add_with_scope("I love hiking", scope).await?;

    // Search only this user's memories
    let results = memory.search_with_scope("hobbies", scope).await?;

    Ok(())
}
```

### 3. Intelligent Memory Processing

```rust
use agent_mem_intelligence::IntelligentMemoryProcessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = IntelligentMemoryProcessor::new(api_key)?;

    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "I'm John, I work at Google, and I love hiking".to_string(),
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            message_id: Some("msg1".to_string()),
        }
    ];

    let result = processor.process_messages(&messages, &[]).await?;

    println!("Extracted facts: {:?}", result.extracted_facts);
    println!("Memory decisions: {:?}", result.memory_decisions);

    Ok(())
}
```

### 4. Custom Plugin

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn process_memory(input: String) -> FnResult<String> {
    // Your custom processing logic
    let output = format!("Processed: {}", input);
    Ok(output)
}
```

See [examples.md](docs/user-guide/examples.md) for more examples.

## 🏗️ Architecture

AgentMem uses a modular architecture with 18 specialized crates:

```
┌─────────────────────────────────────────┐
│         Application Layer                │
│  ┌──────┐  ┌──────┐  ┌──────┐          │
│  │ CLI  │  │ HTTP │  │Python│          │
│  └──────┘  └──────┘  └──────┘          │
├─────────────────────────────────────────┤
│         Core Services                   │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │ Memory  │ │ Plugin  │ │ Search   │  │
│  │ Manager │ │ Manager │ │ Engine   │  │
│  └─────────┘ └─────────┘ └──────────┘  │
├─────────────────────────────────────────┤
│         Storage Layer                   │
│  ┌──────┐  ┌──────┐  ┌──────┐         │
│  │ SQL  │  │Vector│  │Cache │         │
│  └──────┘  └──────┘  └──────┘         │
└─────────────────────────────────────────┘
```

Key components:
- **agent-mem-core** - Memory management engine
- **agent-mem-intelligence** - AI-powered reasoning
- **agent-mem-plugins** - WASM plugin system
- **agent-mem-storage** - Multi-backend storage abstraction
- **agent-mem-server** - HTTP API server

For detailed architecture, see [architecture.md](docs/developer-guide/architecture.md).

## 📊 Performance

| Operation | Throughput | Latency (P50) | Latency (P99) |
|-----------|-----------|---------------|---------------|
| Add Memory | 5,000 ops/s | 20ms | 50ms |
| Vector Search | 10,000 ops/s | 10ms | 30ms |
| Plugin Call | 216,000 ops/s | 1ms | 5ms |
| Batch Import | 50,000 ops/s | 100ms | 300ms |

*Benchmarks performed on M1 Max, 16GB RAM, LibSQL backend*

## 🔌 Plugin System

AgentMem supports WebAssembly plugins for safe, extensible functionality:

```bash
# Register a plugin
curl -X POST http://localhost:8080/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-plugin",
    "path": "/path/to/plugin.wasm"
  }'
```

Plugins run in a sandbox with:
- 🛡️ Memory isolation
- ⚡ LRU caching (93,000x speedup)
- 🔌 Capability-based permissions
- 🔄 Hot reload support

Learn more: [Plugin Development Guide](docs/developer-guide/plugin-development.md)

## 🚦 Deployment

### Docker

```bash
docker run -p 8080:8080 agentmem/server:latest
```

### Kubernetes

```bash
helm install agentmem ./k8s/helm-chart
```

### From Source

```bash
cargo build --release --features plugins
cargo run --bin agent-mem-server
```

See [Production Setup](docs/deployment/production-setup.md) for detailed deployment guides.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

Areas where we'd love help:
- Additional examples and documentation
- Performance optimizations
- Bug fixes and improvements
- New storage backends
- Plugin development

## 📄 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

## 🔗 Links

- **Documentation**: [docs.agentmem.dev](https://docs.agentmem.dev)
- **GitHub**: [github.com/agentmem/agentmem](https://github.com/agentmem/agentmem)
- **Crates.io**: [crates.io/crates/agent-mem](https://crates.io/crates/agent-mem)
- **Discord**: [discord.gg/agentmem](https://discord.gg/agentmem)

## 🌟 Acknowledgments

Inspired by and designed to be compatible with [Mem0](https://github.com/mem0ai/mem0).

---

<div align="center">

**Built with ❤️ by the AgentMem team**

</div>
