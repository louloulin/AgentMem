# Architecture Guide

This guide provides a comprehensive overview of AgentMem's architecture and design decisions.

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐   │
│  │ HTTP API │  │ CLI Tool │  │Py SDK    │  │WASM Plugins  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                        Core Services Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │Memory Manager│  │Plugin Manager│  │Search Engines    │    │
│  └──────────────┘  └──────────────┘  └──────────────────┘    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │LLM Integration│  │Intelligence  │  │Multimodal Proc.  │    │
│  └──────────────┘  └──────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Storage Abstraction Layer                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │Memory Store  │  │Vector Store  │  │Graph Store       │    │
│  │(LibSQL/PG)   │  │(LanceDB)     │  │(Native)          │    │
│  └──────────────┘  └──────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                       Infrastructure Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │Observability │  │Security      │  │Performance       │    │
│  │Prometheus    │  │RBAC/Auth     │  │Async/Caching     │    │
│  └──────────────┘  └──────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Organization

AgentMem is organized into 18 specialized crates:

### Foundation Layer

#### `agent-mem-traits`
**Purpose**: Core abstractions and trait definitions
**Size**: ~2K lines
**Key Types**:
- `Memory` - Core memory trait
- `Store` - Storage backend trait
- `Embedder` - Embedding model trait
- `LLMProvider` - LLM provider trait

**Design Principle**: Trait-based design for flexibility and extensibility

#### `agent-mem-utils`
**Purpose**: Common utilities and helpers
**Size**: ~1K lines
**Key Utilities**:
- `retry()` - Exponential backoff retry
- `hash()` - Consistent hashing
- `validate()` - Input validation

#### `agent-mem-config`
**Purpose**: Configuration management
**Size**: ~1K lines
**Features**:
- Environment variable loading
- TOML configuration files
- Runtime configuration updates

### Core Engine Layer

#### `agent-mem-core`
**Purpose**: Memory management engine
**Size**: ~25K lines
**Key Components**:
- `MemoryManager` - CRUD operations
- `MemoryStore` - Storage backend coordination
- `SearchEngine` - Search orchestration
- `CacheManager` - Multi-level caching

**Design Patterns**:
- Repository Pattern
- Strategy Pattern (search engines)
- Observer Pattern (memory events)

#### `agent-mem`
**Purpose**: Unified high-level API
**Size**: ~3K lines
**Key Features**:
- `Memory::new()` - Zero-config initialization
- `Memory::builder()` - Advanced configuration
- Convenience methods for common operations

**Design Principle**: Progressive disclosure - simple default, power when needed

#### `agent-mem-intelligence`
**Purpose**: AI-powered reasoning engine
**Size**: ~8K lines
**Key Components**:
- `FactExtractor` - Extract facts from text
- `ConflictResolver` - Resolve contradictions
- `MemoryDecisionEngine` - Decide add/update/ignore

**LLM Integration**:
- DeepSeek (primary)
- OpenAI GPT-4
- Anthropic Claude

### Integration Layer

#### `agent-mem-llm`
**Purpose**: LLM provider integrations
**Size**: ~6K lines
**Supported Providers**:
- OpenAI (GPT-3.5, GPT-4)
- Anthropic (Claude)
- DeepSeek
- Google (Gemini)
- Groq
- Ollama (local)

**Design Pattern**: Adapter Pattern for provider abstraction

#### `agent-mem-embeddings`
**Purpose**: Embedding model integrations
**Size**: ~3K lines
**Supported Models**:
- OpenAI (text-embedding-ada-002)
- FastEmbed (BAAI/bge-*)
- Cohere
- HuggingFace (via sentence-transformers)

#### `agent-mem-storage`
**Purpose**: Storage backend abstraction
**Size**: ~10K lines
**Backends**:
- LibSQL (default, embedded)
- PostgreSQL (production)
- SQLite (legacy)
- MySQL (planned)

**Design Pattern**: Strategy Pattern for backend selection

#### `agent-mem-tools`
**Purpose**: MCP (Model Context Protocol) tool integration
**Size**: ~5K lines
**Tools**:
- Memory access tools
- Search tools
- Management tools

### Service Layer

#### `agent-mem-server`
**Purpose**: HTTP REST API server
**Size**: ~8K lines
**Endpoints**: 175+ REST endpoints
**Features**:
- OpenAPI/Swagger documentation
- JWT authentication
- Request validation
- Rate limiting

**Framework**: Axum + Tower

#### `agent-mem-client`
**Purpose**: HTTP client SDK
**Size**: ~2K lines
**Features**:
- Async HTTP client
- Connection pooling
- Retry logic
- Type-safe responses

#### `agent-mem-compat`
**Purpose**: Mem0 API compatibility layer
**Size**: ~3K lines
**Compatibility**: 100% API compatible with Mem0

### Extension Layer

#### `agent-mem-plugin-sdk`
**Purpose**: WASM plugin development SDK
**Size**: ~500 lines
**Features**:
- Plugin macros
- Host function declarations
- Capability definitions

#### `agent-mem-plugins`
**Purpose**: Plugin manager and runtime
**Size**: ~1.5K lines
**Features**:
- Hot-reload
- LRU caching (93,000x speedup)
- Capability system
- Sandboxed execution

**Technology**: Extism WASM framework

#### `agent-mem-python`
**Purpose**: Python bindings (PyO3)
**Size**: ~800 lines
**Features**:
- Native Python API
- NumPy integration
- Async support

### Operations Layer

#### `agent-mem-observability`
**Purpose**: Monitoring and metrics
**Size**: ~2K lines
**Integrations**:
- Prometheus (metrics)
- OpenTelemetry (tracing)
- Structured logging

#### `agent-mem-performance`
**Purpose**: Performance optimization
**Size**: ~3K lines
**Features**:
- Profiling tools
- Performance regression tests
- Benchmarking harness

#### `agent-mem-deployment`
**Purpose**: Deployment automation
**Size**: ~2K lines
**Features**:
- Kubernetes manifests
- Helm charts
- Docker images
- CI/CD pipelines

#### `agent-mem-distributed`
**Purpose**: Distributed systems support
**Size**: ~1.5K lines
**Features**:
- Sharding
- Replication
- Consensus (Raft)

## Data Flow

### Adding a Memory

```
User Request
    ↓
HTTP API / Rust API
    ↓
Memory::add()
    ↓
1. Preprocessing (validation, metadata)
    ↓
2. Fact Extraction (LLM)
    ↓
3. Deduplication Check
    ↓
4. Conflict Detection
    ↓
5. Embedding Generation
    ↓
6. Storage (Memory Store + Vector Store)
    ↓
7. Cache Update
    ↓
Response (memory_id)
```

### Searching Memories

```
User Query
    ↓
HTTP API / Rust API
    ↓
Memory::search()
    ↓
1. Query Analysis (intent detection)
    ↓
2. Engine Selection (Vector/BM25/Hybrid)
    ↓
3. Parallel Search (if hybrid)
    ↓
4. Ranking (RRF for hybrid)
    ↓
5. Filtering (scope, permissions)
    ↓
6. Result Pagination
    ↓
Response (search results)
```

### Plugin Execution

```
Plugin Request
    ↓
PluginManager::execute()
    ↓
1. Cache Lookup (93,000x speedup if hit)
    ↓
2. WASM Module Load (if not cached)
    ↓
3. Capability Check
    ↓
4. Sandboxed Execution
    ↓
5. Result Validation
    ↓
6. Cache Update
    ↓
Response (plugin result)
```

## Design Decisions

### Why Rust?

**Performance**:
- Zero-cost abstractions
- Memory safety without GC
- Predictable performance

**Concurrency**:
- Fearless concurrency with `async/await`
- No data races at compile time
- Efficient async I/O

**Reliability**:
- Memory safety guarantees
- Thread safety guarantees
- Rich type system

**Ecosystem**:
- Excellent async runtime (Tokio)
- Great web frameworks (Axum)
- Strong ORM (SQLx)

### Why WASM for Plugins?

**Security**:
- Sandboxed execution
- No access to host system
- Capability-based security

**Performance**:
- Near-native execution speed
- 31ms first load, 333ns cached
- 216K calls/sec throughput

**Portability**:
- Language-agnostic
- Run anywhere
- No recompilation needed

### Why Multiple Search Engines?

Different use cases require different approaches:

- **Vector Search**: Semantic understanding
- **BM25**: Exact keyword matching
- **Full-Text**: Phrase matching
- **Fuzzy**: Typos and misspellings
- **Hybrid**: Combined approach

### Why LibSQL by Default?

**Zero Configuration**:
- Embedded database
- No setup required
- SQLite compatible

**Performance**:
- In-memory replication
- Fast reads
- Low latency

**Portability**:
- Single file
- Easy backup
- Cross-platform

## Performance Characteristics

### Scalability

| Metric | Single Node | Cluster (3 nodes) |
|--------|-------------|-------------------|
| Memories | 10M+ | 30M+ |
| QPS | 50K | 150K |
| Storage | 100GB | 300GB |
| Memory | 2GB | 6GB |

### Latency

| Operation | P50 | P95 | P99 |
|-----------|-----|-----|-----|
| Add Memory | 20ms | 40ms | 50ms |
| Search | 10ms | 25ms | 30ms |
| Plugin Call | 1ms | 3ms | 5ms |
| Batch Add | 100ms | 250ms | 300ms |

### Throughput

| Operation | Peak QPS | Sustained QPS |
|-----------|----------|---------------|
| Add Memory | 10K | 5K |
| Vector Search | 20K | 10K |
| BM25 Search | 30K | 15K |
| Plugin Call | 250K | 200K |

## Security Architecture

### Authentication

- **JWT**: Token-based authentication
- **API Keys**: For service accounts
- **OAuth 2.0**: For third-party integration

### Authorization

- **RBAC**: Role-based access control
- **ABAC**: Attribute-based access control
- **Scope-based**: Memory scope isolation

### Data Security

- **Encryption at Rest**: AES-256
- **Encryption in Transit**: TLS 1.3
- **Audit Logging**: All operations logged

## Extension Points

### Custom Storage Backends

```rust
impl Store for CustomBackend {
    async fn add(&self, memory: Memory) -> Result<String> {
        // Your implementation
    }

    async fn get(&self, id: &str) -> Result<Memory> {
        // Your implementation
    }
}
```

### Custom Search Engines

```rust
impl SearchEngine for CustomEngine {
    async fn search(&self, query: &str) -> Result<Vec<Memory>> {
        // Your implementation
    }
}
```

### Custom LLM Providers

```rust
impl LLMProvider for CustomProvider {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Your implementation
    }
}
```

### Custom Plugins

```rust
#[plugin_fn]
pub fn custom_function(input: String) -> FnResult<String> {
    // Your WASM plugin
}
```

## Testing Strategy

### Unit Tests
- Per-crate test suites
- >95% coverage target
- Property-based testing (proptest)

### Integration Tests
- API endpoint tests
- Database integration
- LLM provider tests

### Performance Tests
- Benchmark suite (criterion)
- Load testing (k6)
- Performance regression tests

### E2E Tests
- Full workflow tests
- Multi-node cluster tests
- Disaster recovery tests

## Monitoring

### Metrics (Prometheus)

- `agentmem_add_memory_total`
- `agentmem_search_duration_seconds`
- `agentmem_plugin_calls_total`
- `agentmem_cache_hit_ratio`

### Tracing (OpenTelemetry)

- Request tracing
- Distributed tracing
- Performance profiling

### Logging

- Structured logging (tracing)
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Log aggregation (ELK, Loki)

## Future Architecture

### Planned Features

1. **Horizontal Sharding**: Distribute memories across nodes
2. **Read Replicas**: Scale read operations
3. **Streaming Updates**: Real-time memory synchronization
4. **Graph Neural Networks**: Advanced reasoning
5. **Federated Learning**: Privacy-preserving model updates

### Scalability Roadmap

| Phase | Memories | QPS | Nodes |
|-------|----------|-----|-------|
| Current | 10M | 50K | 1-3 |
| Phase 2 | 100M | 500K | 3-10 |
| Phase 3 | 1B | 5M | 10-100 |

## Contributing to Architecture

See [Development Setup](./development-setup.md) for setting up your development environment.

Key areas for contribution:
- New storage backends
- Additional LLM providers
- Performance optimizations
- Documentation improvements

## Further Reading

- [Plugin Development](./plugin-development.md)
- [API Reference](../api/rust-docs.md)
- [Deployment Guide](../deployment/production-setup.md)
