# AgentMem - Enterprise-Grade Intelligent Memory Platform 🧠

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/agentmem/agentmem/actions)
[![WASM Plugins](https://img.shields.io/badge/plugins-WASM-blueviolet.svg)](https://webassembly.org/)
[![Architecture](https://img.shields.io/badge/architecture-modular-success.svg)](#architecture)

> **Next-Generation AI Memory Management System** - Enterprise-grade memory storage, retrieval, and reasoning capabilities for intelligent agents

**AgentMem** is a high-performance intelligent memory management platform built in Rust, designed for AI agents and LLM applications. Featuring modular architecture, **WASM plugin system**, **multimodal processing**, **graph memory networks**, and **Mem0-compatible API**.

---

## 🤔 Why Do AI Agents Need a Memory System?

### Core Challenges Facing Current AI Agents

```
Traditional LLM Limitations:
┌─────────────────────────────────────────────────────────┐
│  ❌ Stateless Interaction - Every conversation starts fresh
│  ❌ Context Window Limits - Cannot handle long-term history
│  ❌ Forgets User Info - Doesn't remember preferences/facts
│  ❌ Lacks Personalization - Cannot provide customized service
│  ❌ Hard to Update Knowledge - Static knowledge base
└─────────────────────────────────────────────────────────┘
```

**Problem Example**:

```
❌ AI Agent Without Memory:
User: My name is Zhang San, I like coffee
AI:   Okay, I've noted that

(5 minutes later)
User: What do I like to drink?
AI:   Sorry, I don't remember your preferences...

❌ Prompt-Dependent Solutions:
- Limited context window (4K-128K tokens)
- High cost (transmitting full history every time)
- Inefficient retrieval (linear scanning)
- Cannot maintain memory across sessions
```

### The AgentMem Solution

```
✅ AgentMem Memory System:
┌─────────────────────────────────────────────────────────┐
│  ✅ Persistent Memory   - Save user info across sessions
│  ✅ Intelligent Retrieval - Millisecond semantic search
│  ✅ Unlimited Capacity - Millions of memories, no limits
│  ✅ Automatic Learning - AI-driven fact extraction
│  ✅ Personalized Service - Customized responses based on history
└─────────────────────────────────────────────────────────┘
```

**Comparison**:

```
✅ With AgentMem:
User: My name is Zhang San, I like coffee
AI:   Got it! I've remembered: Your name is Zhang San, preferred drink is coffee

(One week later, new session)
User: What do I like to drink?
AI:   Based on your preference record, you like coffee. Would you like me to recommend some coffee shops?

✅ Key Advantages:
- Cross-session memory retention
- Intelligent semantic retrieval (< 100ms)
- Automatic fact extraction
- 90%+ cost reduction (no need to transmit full history)
```

---

## 🎯 Core Capabilities

### 1. 🧠 **Intelligent Memory Management**

**Problem**: How to make AI remember important information?

```rust
// Automatic extraction and storage of key facts
memory.add("user123", "I work at Google in San Francisco and love hiking").await?;

// AI automatically extracts:
// ✓ Location: San Francisco
// ✓ Company: Google
// ✓ Hobby: Hiking
```

**Key Features**:
- 🔹 **Automatic Fact Extraction**: AI-driven key information identification
- 🔹 **Intelligent Deduplication**: Auto-detect and merge duplicate memories
- 🔹 **Conflict Resolution**: Smart handling of contradictory information

---

### 2. ⚡ **High-Performance Semantic Search**

**Problem**: How to quickly find relevant memories from millions?

```rust
// Millisecond-level semantic search from millions of memories
let results = memory.search("What does the user like?").await?;

// Results (ranked by relevance):
// 1. "User likes coffee and hiking" (92% match)
// 2. "User works at Google" (78% match)
// 3. "User lives in San Francisco" (65% match)
```

**Performance**:
- 🔹 **Search Speed**: < 100ms (million-level memories)
- 🔹 **Semantic Understanding**: Vector embeddings + BM25 hybrid search
- 🔹 **Multilingual Support**: Chinese, English, Japanese, etc.

---

### 3. 🔌 **WASM Plugin System**

**Problem**: How to flexibly extend memory capabilities?

```rust
// Write plugins in any language (Rust, JavaScript, Go, Cangjie)
use agent_mem::plugin::Plugin;

// Plugin example: Automatic sentiment analysis
#[plugin]
fn sentiment_analysis(memory: &Memory) -> f32 {
    // Analyze emotional tone of memory content
    analyze_sentiment(&memory.content)
}
```

**Supported Plugin Types**:
- 🔹 **Memory Processors**: Content transformation, filtering
- 🔹 **Retrieval Enhancers**: Custom ranking, filtering
- 🔹 **Storage Backends**: PostgreSQL, LibSQL, Redis
- 🔹 **Embedding Models**: OpenAI, FastEmbed, ONNX

---

### 4. 🌐 **Multimodal Memory Support**

**Problem**: How to handle non-textual memories?

```rust
// Support multiple data formats
memory.add(Image {
    data: vec![...],
    format: ImageFormat::Jpeg,
    metadata: map!("caption" => "User's profile photo")
}).await?;

memory.add(Video {
    url: "https://example.com/video.mp4",
    duration: 120,
    transcript: "Video content summary..."
}).await?;
```

**Supported Formats**:
- 🔹 **Text**: Markdown, plain text, code
- 🔹 **Images**: JPEG, PNG (with description)
- 🔹 **Audio**: MP3, WAV (with transcription)
- 🔹 **Video**: MP4, WebM (with transcript)
- 🔹 **Files**: PDF, DOCX, etc.

---

### 5. 🕸️ **Graph Memory Network**

**Problem**: How to capture relationships between memories?

```rust
// Build knowledge graphs from memories
memory.graph()
    .add_entity("Zhang San", "Person")
    .add_entity("Google", "Company")
    .add_relation("Zhang San", "WORKS_AT", "Google")
    .add_relation("Zhang San", "LOCATED_IN", "San Francisco")
    .build().await?;

// Query relationships
let colleagues = memory.graph()
    .find_related("Zhang San", "WORKS_AT")
    .await?;
```

**Graph Capabilities**:
- 🔹 **Entity Extraction**: Auto-extract people, places, organizations
- 🔹 **Relationship Mining**: Discover connections between memories
- 🔹 **Graph Traversal**: Multi-hop relationship queries
- 🔹 **Knowledge Reasoning**: Inference based on relationships

---

## 🚀 Quick Start

### Installation

```bash
# Via cargo
cargo add agent-mem

# Or edit Cargo.toml
[dependencies]
agent-mem = "2.0"
```

### Basic Usage

```rust
use agent_mem::{Memory, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Quick start (zero configuration)
    let memory = Memory::quick().await?;

    // 2. Add memories
    memory.add("user123", "I work at Google in San Francisco").await?;

    // 3. Search memories
    let results = memory.search("Where does the user work?").await?;
    for result in results {
        println!("Found: {} (relevance: {:.0}%)",
                 result.content,
                 result.score * 100.0);
    }

    // 4. Delete memories
    memory.delete("memory_id").await?;

    Ok(())
}
```

### Advanced Configuration

```rust
use agent_mem::{Config, StorageBackend, EmbeddingModel};

let config = Config::builder()
    // Storage configuration
    .storage(StorageBackend::PostgreSQL {
        connection_string: "postgresql://localhost/agentmem"
    })
    // Embedding model configuration
    .embedding(EmbeddingModel::OpenAI {
        api_key: std::env::var("OPENAI_API_KEY")?,
        model: "text-embedding-3-small"
    })
    // Intelligent processing configuration
    .intelligent(true)
    .auto_extract(true)
    .deduplication(true)
    .build()?;

let memory = Memory::new(config).await?;
```

---

## 🏗️ Architecture

AgentMem adopts a **modular, plugin-based architecture**:

```
┌───────────────────────────────────────────────────────────┐
│                     Application Layer                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Chat Bot │  │  Agent   │  │ Analysis │  │  Custom  │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘ │
└───────┼────────────┼─────────────┼─────────────┼────────┘
        │            │             │             │
┌───────┴────────────┴─────────────┴─────────────┴────────┐
│                   API Layer                              │
│  ┌────────────────────────────────────────────────────┐ │
│  │  REST API • gRPC • WebSocket • MCP Protocol       │ │
│  └────────────────────────────────────────────────────┘ │
└────────────────────┬─────────────────────────────────────┘
                     │
┌────────────────────┴─────────────────────────────────────┐
│                  Core Memory Engine                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │Orchestrat│  │Intellig. │  │ Retrieva │  │ Lifecycle ││
│  │   or     │  │  Engine  │  │   Engine │  │ Manager   ││
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘│
└───────┼─────────────┼─────────────┼─────────────┼────────┘
        │             │             │             │
┌───────┴─────────────┴─────────────┴─────────────┴────────┐
│              Storage & Plugin Layer                       │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐ │
│  │ Vector DB  │  │ Graph DB   │  │  WASM Plugins      │ │
│  │ (LibSQL)   │  │ (Native)   │  │  • Processors      │ │
│  └────────────┘  └────────────┘  │  • Embedders       │ │
│                                  │  • Storage Backends │ │
│                                  └────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

### Core Components

- **Orchestrator**: Coordinates memory operations and workflows
- **Intelligence Engine**: AI-driven fact extraction and reasoning
- **Retrieval Engine**: Hybrid vector + keyword search
- **Lifecycle Manager**: Memory state transitions and archiving
- **Plugin System**: WASM-based extensibility

---

## 📚 Documentation

- **[Quick Start Guide](QUICKSTART.md)** - 5-minute getting started
- **[API Documentation](docs/api/README.md)** - Complete API reference
- **[Architecture Guide](README_ARCHITECTURE.md)** - System design and architecture
- **[Plugin Development](docs/guides/plugin-development.md)** - Writing custom plugins
- **[Deployment Guide](docs/guides/deployment.md)** - Production deployment
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions

---

## 🌟 Features & Capabilities

### Memory Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Semantic Memory** | General knowledge and facts | "User likes coffee" |
| **Episodic Memory** | Specific events and experiences | "User visited Paris in June" |
| **Procedural Memory** | Skills and procedures | "User knows how to use React" |
| **Working Memory** | Short-term context | Current conversation context |

### Storage Backends

| Backend | Description | Status |
|---------|-------------|--------|
| **SQLite** | Embedded database | ✅ Stable |
| **PostgreSQL** | Production database | ✅ Stable |
| **LibSQL** | Edge SQL database | ✅ Stable |
| **Redis** | In-memory cache | ✅ Stable |
| **Distributed** | Multi-node cluster | 🚧 Beta |

### Embedding Models

| Model | Provider | Dimension | Speed |
|-------|----------|-----------|-------|
| **text-embedding-3-small** | OpenAI | 1536 | ⭐⭐⭐ |
| **text-embedding-3-large** | OpenAI | 3072 | ⭐⭐⭐⭐ |
| **bge-m3** | FastEmbed | 1024 | ⭐⭐⭐⭐⭐ |
| **all-MiniLM-L6-v2** | ONNX | 384 | ⭐⭐⭐⭐⭐ |

---

## 🔧 Configuration

### Server Mode

```bash
# Start AgentMem server
agentmem-server start \
  --host 0.0.0.0 \
  --port 8080 \
  --storage postgresql://localhost/agentmem \
  --embedding openai \
  --api-key $OPENAI_API_KEY
```

### Embedded Mode

```rust
// In-process memory (no server needed)
let memory = Memory::builder()
    .storage(StorageBackend::SQLite {
        path: "./agentmem.db"
    })
    .embedding(EmbeddingModel::FastEmbed)
    .build()
    .await?;
```

---

## 📊 Performance Benchmarks

| Operation | Performance | Notes |
|-----------|-------------|-------|
| **Memory Add** | ~5ms | Including embedding generation |
| **Memory Search** | <100ms | From 1M memories |
| **Batch Import** | ~1000/s | Bulk import speed |
| **Vector Search** | <50ms | Semantic similarity search |
| **Graph Query** | <200ms | 3-hop relationship queries |

*Benchmarks performed on: MacBook Pro M1, 16GB RAM, LibSQL backend*

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/agentmem/agentmem.git
cd agentmem

# Run tests
cargo test --all

# Run linter
cargo clippy --all-targets

# Format code
cargo fmt --all
```

---

## 📜 License

Dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

---

## 🌐 Community

- **GitHub**: [https://github.com/agentmem/agentmem](https://github.com/agentmem/agentmem)
- **Documentation**: [https://docs.agentmem.dev](https://docs.agentmem.dev)
- **Discord**: [Join our Discord server](https://discord.gg/agentmem)
- **Twitter**: [@AgentMemHQ](https://twitter.com/AgentMemHQ)

---

## 🙏 Acknowledgments

- Inspired by [Mem0](https://github.com/mem0ai/mem0)
- Built with [Rust](https://www.rust-lang.org/)
- Vector search powered by [LibSQL](https://libsql.org)
- Embeddings from [OpenAI](https://openai.com)

---

## 🗺️ Roadmap

### v2.1 (Current)
- ✅ WASM plugin system
- ✅ Multimodal memory support
- ✅ Graph memory networks
- ✅ Mem0-compatible API

### v2.2 (Q1 2025)
- 🚧 Distributed storage backend
- 🚧 Real-time memory sync
- 🚧 Advanced graph algorithms
- 🚧 Performance optimizations

### v3.0 (Q2 2025)
- 📅 Plugin marketplace
- 📅 Enterprise features (RBAC, audit logs)
- 📅 Multi-tenant support
- 📅 Cloud hosting service

---

**Made with ❤️ by the AgentMem Team**

*Enterprise-Grade Intelligent Memory Platform for AI Agents*
