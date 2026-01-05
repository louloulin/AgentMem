# Developing AgentMem

This guide is for developers who want to contribute to or work on the AgentMem codebase.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Setting Up Development Environment](#setting-up-development-environment)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Code Organization](#code-organization)
- [Development Workflow](#development-workflow)
- [Writing Tests](#writing-tests)
- [Debugging](#debugging)
- [Performance Profiling](#performance-profiling)
- [Release Process](#release-process)

---

## Prerequisites

### Required

- **Rust**: 1.75 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **Git**: For version control
- **Cargo**: Comes with Rust installation

### Recommended

- **PostgreSQL**: 15+ for integration tests
- **LibSQL**: For edge database testing
- **Docker**: For containerized testing

### Optional

- **OpenAI API Key**: For embedding model tests
- **Python 3.8+**: For Python SDK development
- **Node.js 18+**: For JavaScript SDK development

---

## Setting Up Development Environment

### 1. Clone the Repository

```bash
git clone https://github.com/agentmem/agentmem.git
cd agentmem
```

### 2. Install Dependencies

```bash
# Install Rust dependencies
cargo fetch

# Install development tools
cargo install cargo-watch
cargo install cargo-tarpaulin
cargo install cargo-audit
```

### 3. Set Up Environment Variables

```bash
# Copy example environment file
cp .env.example .env

# Edit .env with your configuration
# - DATABASE_URL for PostgreSQL
# - OPENAI_API_KEY for embeddings
# - Other service credentials
```

### 4. Set Up Pre-commit Hooks

```bash
# Install pre-commit hooks
./scripts/setup-hooks.sh

# Or manually install
cargo install cargo-husky
cargo husky install
```

---

## Building the Project

### Development Build

```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p agent-mem-core

# Build with optimizations (faster runtime)
cargo build --release
```

### Check Compilation

```bash
# Quick check (faster than full build)
cargo check

# Check all targets
cargo check --all-targets

# Check with all features
cargo check --all-features
```

### Compilation Time Optimization

For faster development builds, the workspace is configured with:

```toml
[profile.dev]
opt-level = 0
codegen-units = 256
incremental = true
```

---

## Running Tests

### Run All Tests

```bash
# Run all tests in workspace
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests in parallel
cargo test --workspace -- --test-threads=8
```

### Run Specific Tests

```bash
# Run tests for specific crate
cargo test -p agent-mem-core

# Run specific test
cargo test test_memory_add

# Run tests matching pattern
cargo test memory*

# Run integration tests only
cargo test --test '*_test'
```

### Test Coverage

```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html

# Generate coverage for specific crate
cargo tarpaulin -p agent-mem-core --out Html

# Check coverage threshold (60%)
cargo tarpaulin --workspace --fail-under 60
```

### Run Specific Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*_integration_test'

# E2E tests only
cargo test --test '*_e2e_test'

# Doc tests
cargo test --doc
```

---

## Code Organization

### Workspace Structure

```
agentmen/
├── Cargo.toml                 # Workspace root
├── crates/                    # Core crates
│   ├── agent-mem-traits/      # Core traits and types
│   ├── agent-mem-core/        # Core memory engine
│   ├── agent-mem-llm/         # LLM integration
│   ├── agent-mem-storage/     # Storage backends
│   ├── agent-mem-embeddings/  # Embedding models
│   ├── agent-mem-intelligence/# AI-driven features
│   └── ...                    # Other crates
├── sdks/                      # Language bindings
│   ├── agent-mem-python/      # Python SDK
│   └── ...
├── tools/                     # CLI tools and utilities
│   ├── agentmem-cli/
│   └── ...
└── examples/                  # Usage examples
```

### Core Crates

#### `agent-mem-traits`
Core traits and type definitions:
- `Memory` - Main memory trait
- `StorageBackend` - Storage abstraction
- `EmbeddingModel` - Embedding model trait
- `Plugin` - Plugin system traits

#### `agent-mem-core`
Core memory management engine:
- `Orchestrator` - Coordinate operations
- `IntelligenceEngine` - AI-driven features
- `RetrievalEngine` - Search and retrieval
- `LifecycleManager` - Memory state management

#### `agent-mem-storage`
Storage backend implementations:
- SQLite (embedded)
- PostgreSQL (production)
- LibSQL (edge)
- Redis (cache)

#### `agent-mem-embeddings`
Embedding model integrations:
- OpenAI
- FastEmbed
- ONNX

---

## Development Workflow

### 1. Create a Feature Branch

```bash
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

```bash
# Watch for file changes and recompile
cargo watch -x check

# Or watch and run tests
cargo watch -x test
```

### 3. Run Checks Before Commit

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Run audit
cargo audit
```

### 4. Commit Changes

```bash
# Stage changes
git add .

# Commit with conventional commit message
git commit -m "feat: add new embedding model support"

# Or use commitizen (if installed)
git cz
```

### 5. Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
# Ensure all CI checks pass
```

---

## Writing Tests

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_add() {
        let memory = Memory::quick().await.unwrap();
        memory.add("user123", "Test content").await.unwrap();

        let results = memory.search("Test").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_metadata_validation() {
        let metadata = MemoryMetadata::new();
        assert!(metadata.is_valid());
    }
}
```

### Integration Tests

```rust
// crates/agent-mem-core/tests/integration_test.rs
use agent_mem_core::Memory;

#[tokio::test]
async fn test_full_workflow() {
    // Setup
    let memory = Memory::quick().await.unwrap();

    // Test add
    memory.add("user1", "Test memory").await.unwrap();

    // Test search
    let results = memory.search("Test").await.unwrap();
    assert!(!results.is_empty());

    // Test delete
    memory.delete(&results[0].id).await.unwrap();
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_memory_roundtrip(content in "\\PC*") {
        let memory = Memory::quick().await.unwrap();
        memory.add("user", &content).await.unwrap();

        let results = memory.search(&content).await.unwrap();
        assert!(!results.is_empty());
    }
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_search(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let memory = rt.block_on(Memory::quick()).unwrap();

    c.bench_function("search", |b| {
        b.iter(|| {
            rt.block_on(memory.search(black_box("test")));
        })
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
```

---

## Debugging

### Enable Debug Logging

```bash
# Set RUST_LOG environment variable
RUST_LOG=debug cargo run

# For specific module
RUST_LOG=agent_mem_core::orchestrator=debug cargo run

# For trace-level logging
RUST_LOG=trace cargo run
```

### Use LLDB/GDB

```bash
# Build with debug symbols
cargo build

# Debug with LLDB (macOS)
lldb target/debug/agentmem

# Debug with GDB (Linux)
gdb target/debug/agentmem
```

### VS Code Debug Configuration

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug unit tests",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--bin=agentmem"
        ]
      },
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

---

## Performance Profiling

### Flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin agentmem

# Open flamegraph
open flamegraph.svg
```

### Heap Profiling

```bash
# Use dhat for heap profiling
cargo install cargo-dhat

# Run with dhat
cargo dhat -- --test-threads=1

# Open dhat report
open dhat-timer.json
```

### Criterion Benchmarks

```bash
# Run benchmarks
cargo bench

# Generate comparison
cargo bench -- --baseline main

# View results
open target/criterion/report/index.html
```

---

## Release Process

### Version Bump

```bash
# Update version in Cargo.toml
cargo set-version --workspace 2.1.0

# Update CHANGELOG.md
# Add release notes

# Commit changes
git commit -m "chore: bump version to 2.1.0"
```

### Create Release

```bash
# Create git tag
git tag -a v2.1.0 -m "Release v2.1.0"

# Push tag
git push upstream v2.1.0

# GitHub Actions will:
# - Build release binaries
# - Publish to crates.io
# - Create GitHub release
```

### Publish to Crates.io

```bash
# Login to crates.io
cargo login

# Publish all crates
cargo publish -p agent-mem-traits
cargo publish -p agent-mem-core
# ... publish other crates in dependency order
```

---

## Additional Resources

- [Rust Guidelines](https://rust-lang.github.io/api-guidelines/)
- [API Documentation](https://docs.rs/agent-mem)
- [Architecture Guide](README_ARCHITECTURE.md)
- [Contributing Guide](CONTRIBUTING.md)

---

## Getting Help

- **GitHub Issues**: [Bug reports and feature requests](https://github.com/agentmem/agentmem/issues)
- **Discord**: [Join our community](https://discord.gg/agentmem)
- **Email**: support@agentmem.dev

---

*Happy hacking! 🚀*
