# Multi-stage Docker build for AgentMem production deployment
# Optimized for security, performance, and minimal image size
# Supports Linux amd64 architecture for production servers
# Reference: feature-claudecode branch - simplified build approach

# Build stage - using latest Rust for Cargo.lock v4 support
FROM rust:latest AS builder

# Install build dependencies including protobuf-compiler
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 agentmem

# Set working directory
WORKDIR /app

# Copy all source code (simplified approach from feature-claudecode)
COPY . .

# Build the application with RUSTFLAGS to handle SQLite linking conflicts
# Use --allow-multiple-definition to resolve libsql_ffi and libsqlite3_sys conflicts
RUN RUSTFLAGS="-C link-arg=-Wl,--allow-multiple-definition" \
    cargo build --release --workspace \
    --bin agent-mem-server \
    --exclude agent-mem-python \
    --exclude demo-multimodal \
    --exclude demo-codebase-memory

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create app user and group
RUN groupadd -r agentmem && useradd -r -g agentmem -u 1001 agentmem

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/config \
    && chown -R agentmem:agentmem /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/agent-mem-server /app/agent-mem-server

# Copy configuration files
COPY --chown=agentmem:agentmem docker/config/ /app/config/

# Set permissions
RUN chmod +x /app/agent-mem-server

# Switch to non-root user
USER agentmem

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV AGENT_MEM_PORT=8080
ENV AGENT_MEM_HOST=0.0.0.0
ENV AGENT_MEM_DATA_DIR=/app/data
ENV AGENT_MEM_LOG_DIR=/app/logs
ENV AGENT_MEM_CONFIG_DIR=/app/config

# Run the application
CMD ["./agent-mem-server"]
