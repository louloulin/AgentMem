# Frequently Asked Questions (FAQ)

常见问题解答 for AgentMem

---

## Table of Contents

- [General Questions](#general-questions)
- [Installation & Setup](#installation--setup)
- [Usage & Features](#usage--features)
- [Performance](#performance)
- [Integration](#integration)
- [Troubleshooting](#troubleshooting)

---

## General Questions

### What is AgentMem?

AgentMem is an enterprise-grade intelligent memory management platform designed for AI agents and LLM applications. It provides persistent memory storage, semantic search, and AI-driven knowledge extraction.

**Key Features**:
- 🧠 Intelligent memory management
- 🔍 Semantic search (< 100ms)
- 📊 Multi-modal support (text, image, audio, video)
- 🔌 WASM plugin system
- 🌐 Multi-language SDKs (Rust, Python, JavaScript, Go)

### How does AgentMem differ from other memory systems?

| Feature | AgentMem | Mem0 | LangChain Memory |
|---------|----------|------|------------------|
| Semantic Search | ✅ | ✅ | ⚠️ Basic |
| Multi-modal | ✅ | ⚠️ Limited | ❌ |
| WASM Plugins | ✅ | ❌ | ❌ |
| Graph Memory | ✅ | ❌ | ❌ |
| Performance | < 100ms | ~200ms | Varies |
| SDKs | 4 languages | Python only | Python/JS |

### Is AgentMem production-ready?

Yes! AgentMem is production-ready with:
- ✅ Comprehensive test coverage (80%+)
- ✅ Enterprise-grade security
- ✅ Scalable architecture (millions of memories)
- ✅ Multiple deployment options (Server, Embed, Edge)
- ✅ Active maintenance and support

### What are the licensing terms?

AgentMem is dual-licensed:
- **Open Source**: MIT License (free for all use)
- **Enterprise**: Commercial licenses available for enterprise features

See [LICENSE](LICENSE) for details.

---

## Installation & Setup

### How do I install AgentMem?

**Rust**:
```bash
cargo add agent-mem
```

**Python**:
```bash
pip install agentmem
```

**JavaScript/TypeScript**:
```bash
npm install @agentmem/core
```

**Docker**:
```bash
docker pull agentmem/server:latest
```

### What are the system requirements?

**Minimum Requirements**:
- CPU: 2 cores
- RAM: 4 GB
- Disk: 1 GB
- OS: Linux, macOS, Windows

**Recommended for Production**:
- CPU: 4+ cores
- RAM: 8+ GB
- Disk: 10+ GB SSD
- OS: Linux (Ubuntu 20.04+, RHEL 8+)

### How do I configure AgentMem?

Create a `agentmem.toml` configuration file:

```toml
[general]
database_url = "sqlite:///path/to/database.db"
embedding_model = "all-MiniLM-L6-v2"

[server]
host = "0.0.0.0"
port = 8080

[performance]
max_connections = 100
query_timeout = 30
```

See [Configuration Guide](docs/guides/config-migration.md) for details.

### Can I use AgentMem without external dependencies?

Yes! AgentMem supports:
- **SQLite** (built-in, no external dependencies)
- **FastEmbed** (built-in embeddings, no API keys needed)
- **Local LLMs** (Ollama, LocalAI integration)

---

## Usage & Features

### How do I add a memory?

**Rust**:
```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<()> {
    let memory = Memory::quick();
    memory.add("I love coffee in the morning").await?;
    Ok(())
}
```

**Python**:
```python
from agentmem import Memory

memory = Memory.quick()
memory.add("I love coffee in the morning")
```

### How does semantic search work?

AgentMem uses vector embeddings to find semantically similar memories:

```rust
let results = memory.search("beverage preference").await?;
// Returns: "I love coffee in the morning" (semantic match)
```

**Features**:
- ✅ Semantic similarity (meaning-based)
- ✅ Hybrid search (semantic + keyword)
- ✅ Dynamic thresholds (adaptive relevance)
- ✅ Multi-lingual support

### Can I store multi-modal data?

Yes! AgentMem supports:

**Images**:
```rust
memory.add_image(
    "A photo of a sunset",
    image_bytes,
    "image/jpeg"
).await?;
```

**Audio**:
```rust
memory.add_audio(
    "Meeting recording",
    audio_bytes,
    "audio/wav"
).await?;
```

**Video**:
```rust
memory.add_video(
    "Product demo",
    video_bytes,
    "video/mp4"
).await?;
```

### How does the intelligent extraction work?

AgentMem automatically extracts:
- **Facts**: Names, dates, locations, quantities
- **Entities**: People, organizations, places
- **Relationships**: X works at Y, Z is located in W
- **Importance**: Automatic relevance scoring

```rust
// Input: "John works at Google in Mountain View"
// Extracted:
// - Person: John
// - Organization: Google
// - Location: Mountain View
// - Relationship: works_at
```

### What is graph memory?

Graph memory links related memories as a knowledge graph:

```rust
memory.add("John works at Google").await?;
memory.add("Google is in Mountain View").await?;

// Automatically creates relationships:
// John --[works_at]--> Google --[located_in]--> Mountain View
```

**Benefits**:
- Contextual retrieval
- Relationship discovery
- Knowledge inference

---

## Performance

### What are the performance benchmarks?

| Operation | Performance |
|-----------|-------------|
| Add Memory | < 10ms |
| Search (1000 memories) | < 100ms |
| Search (1M memories) | < 500ms |
| Batch Add (100 memories) | < 2s |
| Embedding Generation | < 50ms |

*Benchmarks on M1 Mac, 8GB RAM, SQLite backend*

### How do I optimize performance?

**Configuration**:
```toml
[performance]
# Increase cache size
cache_size = 1000

# Connection pooling
max_connections = 100

# Embedding optimization
batch_size = 32
```

**Best Practices**:
- Use batch operations for bulk imports
- Enable query caching
- Use appropriate embedding models
- Index frequently searched fields

### Can AgentMem handle millions of memories?

Yes! AgentMem is designed for scale:
- ✅ Tested with 10M+ memories
- ✅ Sub-second search at scale
- ✅ Efficient memory compression
- ✅ Database sharding support

### How do I monitor performance?

```rust
// Enable metrics
let config = Config::builder()
    .with_metrics(true)
    .with_profiling(true)
    .build();

let memory = Memory::new(config).await?;

// Access metrics
let metrics = memory.get_metrics().await?;
println!("Query latency: {}ms", metrics.avg_query_latency);
```

---

## Integration

### How do I integrate with LangChain?

```python
from langchain.memory import AgentMemory
from agentmem import Memory as AgentMem

# Initialize AgentMem
agentmem = AgentMem.quick()

# Use in LangChain
memory = AgentMemory(
    memory_backend=agentmem,
    human_prefix="User",
    ai_prefix="AI"
)
```

### How do I integrate with LlamaIndex?

```python
from llama_index import Memory
from agentmem import Memory as AgentMem

# Initialize
agentmem = AgentMem.quick()

# Create LlamaIndex memory
memory = Memory(
    memory_backend=agentmem
)
```

### Does AgentMem work with LLM providers?

Yes! Supported providers:
- ✅ OpenAI (GPT-3.5, GPT-4)
- ✅ Anthropic (Claude)
- ✅ Google (Gemini)
- ✅ Open Source (Llama, Mistral)
- ✅ Local (Ollama, LocalAI)
- ✅ Chinese (Zhipu AI,百度文心)

### Can I use AgentMem in a web application?

Yes! AgentMem provides:
- REST API server
- WebSocket support (real-time)
- Server-Sent Events (SSE)
- Multi-tenant support

**Example**:
```bash
# Start server
agentmem-server --port 8080

# Use REST API
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello, AgentMem!"}'
```

---

## Troubleshooting

### Installation fails with "linking error"

**Problem**: Native compilation fails on some systems.

**Solutions**:
1. Use pre-built binaries: `cargo install agentmem-cli`
2. Use Docker: `docker run agentmem/server`
3. Install system dependencies:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install build-essential libssl-dev pkg-config

   # macOS
   brew install openssl pkg-config
   ```

### "Database locked" error

**Problem**: SQLite database is locked.

**Solutions**:
1. Enable WAL mode:
   ```toml
   [database]
   wal_mode = true
   ```

2. Increase timeout:
   ```toml
   [database]
   timeout = 30
   ```

3. Use PostgreSQL instead for concurrent access.

### Search returns no results

**Problem**: Semantic search returns empty results.

**Solutions**:
1. Check embedding model is loaded
2. Verify memories were added successfully
3. Adjust similarity threshold:
   ```rust
   let results = memory.search("query")
       .with_threshold(0.5)  # Lower threshold
       .await?;
   ```

4. Try keyword search as fallback:
   ```rust
   let results = memory.search("query")
       .with_mode(SearchMode::Hybrid)
       .await?;
   ```

### Out of memory error

**Problem**: AgentMem consumes too much RAM.

**Solutions**:
1. Reduce cache size:
   ```toml
   [performance]
   cache_size = 100
   ```

2. Use streaming for large datasets:
   ```rust
   memory.batch_add_stream(memories_stream).await?;
   ```

3. Enable memory compression:
   ```toml
   [storage]
   compression = true
   ```

### How do I get help?

**Resources**:
- 📖 [Documentation](docs/)
- 🐛 [Issue Tracker](https://github.com/agentmem/agentmem/issues)
- 💬 [Discussions](https://github.com/agentmem/agentmem/discussions)
- 📧 Email: support@agentmem.dev

**Professional Support**:
- Enterprise support: enterprise@agentmem.dev
- Consulting: consulting@agentmem.dev
- Training: training@agentmem.dev

---

## Additional Questions?

For more information:
- 📚 [Full Documentation](docs/)
- 🚀 [Quick Start Guide](QUICKSTART.md)
- 🏗️ [Architecture Overview](README_ARCHITECTURE.md)
- 🤝 [Contributing Guide](CONTRIBUTING.md)
- 🔐 [Security Policy](SECURITY.md)

---

*Last updated: 2025-01-05*
