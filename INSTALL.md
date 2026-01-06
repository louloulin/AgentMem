# Installation Guide

This guide covers installing AgentMem in various environments.

## System Requirements

- **Rust**: 1.75 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: 4GB RAM minimum (8GB recommended)
- **Disk Space**: 1GB free space

## Installation Methods

### Method 1: Cargo (Recommended for Rust Users)

Add AgentMem to your `Cargo.toml`:

```toml
[dependencies]
agent-mem = "2.0"
tokio = { version = "1", features = ["full"] }
```

Then run:

```bash
cargo build
```

### Method 2: Git Clone

Clone the repository:

```bash
git clone https://github.com/agentmem/agentmem.git
cd agentmem
```

Build the workspace:

```bash
cargo build --release --features plugins
```

### Method 3: Docker

Pull the latest image:

```bash
docker pull agentmem/server:latest
```

Or build from source:

```bash
docker build -t agentmem/server .
```

## Configuration

### Environment Variables

AgentMem uses environment variables for configuration:

```bash
# LLM Provider (required for intelligent features)
export OPENAI_API_KEY="sk-..."
# or
export ANTHROPIC_API_KEY="sk-ant-..."
# or
export DEEPSEEK_API_KEY="..."

# Optional: Database backend
export DATABASE_BACKEND="libsql"  # or "postgres"
export DATABASE_URL="file:data/agentmem.db"

# Optional: Vector store
export VECTOR_STORE="lancedb"  # or "redis", "pinecone"
```

### Configuration File

For advanced configuration, use a TOML file:

```toml
# config.toml
[database]
backend = "libsql"
url = "file:data/agentmem.db"

[llm]
provider = "openai"
api_key = "sk-..."
model = "gpt-4"

[embeddings]
provider = "fastembed"
model = "BAAI/bge-small-en-v1.5"

[server]
host = "0.0.0.0"
port = 8080
```

Run with configuration file:

```bash
cargo run --bin agent-mem-server -- --config config.toml
```

## Verification

Test your installation:

```bash
# Run the quickstart example
cargo run --example quickstart-zero-config

# Or test with a simple command
echo "Testing AgentMem installation..."
cargo test --workspace
```

Expected output:
```
running 1 test
test tests::integration_test ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

## Troubleshooting

### Rust Version Too Old

**Error**: `error: package agent-mem v2.0.0 cannot be built because it requires rust 1.75`

**Solution**:
```bash
rustup update stable
rustup default stable
```

### Missing OpenSSL Headers (Linux)

**Error**: `error: failed to run custom build command for 'openssl-sys'`

**Solution** (Ubuntu/Debian):
```bash
sudo apt-get install pkg-config libssl-dev
```

**Solution** (Fedora):
```bash
sudo dnf install openssl-devel pkg-config
```

**Solution** (macOS):
```bash
brew install openssl
export OPENSSL_DIR=/usr/local/opt/openssl
```

### WASM Plugin Build Failures

**Error**: `error: linker 'clang' not found`

**Solution**:
```bash
# Install WASM target
rustup target add wasm32-wasip1

# macOS: Install Xcode command line tools
xcode-select --install

# Linux: Install clang
sudo apt-get install clang  # Ubuntu/Debian
sudo dnf install clang      # Fedora
```

### Database Initialization Errors

**Error**: `Database initialization failed`

**Solution**:
```bash
# Ensure data directory exists
mkdir -p data

# Check file permissions
chmod 755 data

# Try with a fresh database
rm -f data/agentmem.db
cargo run --bin agent-mem-server
```

### Out of Memory During Build

**Error**: `fatal error: Killed signal` or `error: could not compile`

**Solution**:
```bash
# Limit parallel jobs
export CARGO_BUILD_JOBS=2
cargo build --release

# Or use release mode with lower optimization
cargo build --release --profile dev
```

## Next Steps

After successful installation:

1. **Read the [Quick Start Guide](QUICKSTART.md)** - Get started in 5 minutes
2. **Explore [Examples](docs/user-guide/examples.md)** - See practical usage
3. **Understand [Core Concepts](docs/user-guide/core-concepts.md)** - Learn how it works
4. **Configure for Production** - See [Production Setup](docs/deployment/production-setup.md)

## Getting Help

If you encounter issues not covered here:

- Check [Troubleshooting](docs/user-guide/troubleshooting.md)
- Search [GitHub Issues](https://github.com/agentmem/agentmem/issues)
- Join our [Discord community](https://discord.gg/agentmem)
- Open a [new issue](https://github.com/agentmem/agentmem/issues/new)

## Uninstallation

To remove AgentMem:

```bash
# If installed via cargo
cargo uninstall agent-mem

# If installed from source
rm -rf /path/to/agentmem

# If installed via Docker
docker rmi agentmem/server

# Clean up data directory (WARNING: deletes all data)
rm -rf data/
```

## Advanced Installation

### Build Custom Features

```bash
# Build with all features
cargo build --release --features "plugins,postgres,python"

# Build specific features
cargo build --release --features plugins
```

Available features:
- `plugins` - WASM plugin system
- `postgres` - PostgreSQL backend
- `python` - Python bindings
- `observability` - Monitoring and metrics

### Install from Git Branch

```bash
cargo install --git https://github.com/agentmem/agentmem.git --branch main
```

### Development Installation

For contributors:

```bash
git clone https://github.com/agentmem/agentmem.git
cd agentmem

# Install development tools
cargo install cargo-watch cargo-nextest

# Run tests
cargo test --workspace

# Run with hot-reload during development
cargo watch -x check -x test
```

See [Development Setup](docs/developer-guide/development-setup.md) for complete contributor guide.
