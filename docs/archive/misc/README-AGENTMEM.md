# AgentMem 2.0 - Next-Generation Intelligent Memory Management Platform

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/agentmem/agentmem)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://hub.docker.com/r/agentmem/server)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-blue.svg)](k8s/)

AgentMem 2.0 is a production-ready, enterprise-grade intelligent memory management platform built in Rust. It provides advanced memory processing, hierarchical organization, and seamless integration with multiple LLM providers and vector databases.

## 🚀 Key Features

### 🧠 **Advanced Memory Management**
- **Hierarchical Memory Architecture**: Multi-level memory organization (Global → Agent → User → Session)
- **Intelligent Processing**: Automatic conflict resolution, deduplication, and semantic merging
- **Adaptive Strategies**: Self-optimizing memory management based on usage patterns
- **Context-Aware Search**: Intelligent search with semantic understanding and contextual ranking

### ⚡ **High Performance**
- **Async Architecture**: Built on Tokio for maximum concurrency
- **Multi-level Caching**: L1/L2/L3 cache hierarchy for optimal performance
- **Batch Processing**: Efficient bulk operations with intelligent batching
- **Connection Pooling**: Optimized database and service connections

### 🌐 **Distributed & Scalable**
- **Cluster Support**: Native distributed computing with consensus algorithms
- **Auto-scaling**: Kubernetes-ready with HPA and resource management
- **Load Balancing**: Multiple strategies for optimal resource utilization
- **Fault Tolerance**: Automatic failover and recovery mechanisms

### 🔒 **Enterprise Security**
- **End-to-End Encryption**: Data encryption at rest and in transit
- **Access Control**: Role-based permissions and fine-grained access control
- **Threat Detection**: Real-time security monitoring and incident response
- **Audit Logging**: Comprehensive audit trails for compliance

### 📊 **Production Monitoring**
- **Real-time Metrics**: Comprehensive performance and health monitoring
- **Alerting System**: Configurable alerts with multiple notification channels
- **Distributed Tracing**: Full request tracing across services
- **Compliance Reporting**: Automated compliance and audit reports

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │    │   GraphQL API   │    │   gRPC API      │
│   (Axum)        │    │   (async-graphql)│    │   (tonic)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    AgentMem Core Engine                         │
├─────────────────┬─────────────────┬─────────────────┬───────────┤
│ Hierarchical    │ Intelligent     │ Context-Aware   │ Adaptive  │
│ Memory Service  │ Processing      │ Search Engine   │ Strategy  │
└─────────────────┴─────────────────┴─────────────────┴───────────┘
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Vector DBs    │    │   Graph DBs     │    │   Cache Layer   │
│ • Qdrant        │    │ • Neo4j         │    │ • Redis         │
│ • Pinecone      │    │ • Memgraph      │    │ • In-Memory     │
│ • Weaviate      │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 Quick Start

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/agentmem/agentmem.git
cd agentmem

# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f agentmem
```

The API will be available at `http://localhost:8080`

### Using Kubernetes

```bash
# Apply Kubernetes manifests
kubectl apply -f k8s/

# Check deployment status
kubectl get pods -n agentmem

# Port forward to access the API
kubectl port-forward -n agentmem svc/agentmem-service 8080:8080
```

### Building from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/agentmem/agentmem.git
cd agentmem
cargo build --release

# Run the server
./target/release/agent-mem-server
```

## 📖 API Documentation

### REST API

Once the server is running, visit `http://localhost:8080/swagger-ui/` for interactive API documentation.

### Basic Usage Examples

#### Add a Memory
```bash
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "my-agent",
    "content": "Remember that the user prefers morning meetings",
    "memory_type": "preference",
    "importance": "medium"
  }'
```

#### Search Memories
```bash
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "morning meetings",
    "agent_id": "my-agent",
    "limit": 10
  }'
```

#### Get Memory by ID
```bash
curl http://localhost:8080/api/v1/memories/{memory_id}
```

## 🔧 Configuration

### Environment Variables

```bash
# Server Configuration
AGENT_MEM_PORT=8080
AGENT_MEM_HOST=0.0.0.0
AGENT_MEM_ENABLE_CORS=true
AGENT_MEM_ENABLE_AUTH=false

# Database Configuration
AGENT_MEM_DATABASE_URL=postgresql://user:pass@localhost/agentmem
AGENT_MEM_REDIS_URL=redis://localhost:6379

# Vector Store Configuration
AGENT_MEM_VECTOR_STORE=qdrant
AGENT_MEM_QDRANT_URL=http://localhost:6333

# LLM Configuration
AGENT_MEM_LLM_PROVIDER=openai
AGENT_MEM_OPENAI_API_KEY=your-api-key

# Logging
RUST_LOG=info
AGENT_MEM_LOG_LEVEL=info
```

### Configuration File

Create `config/agentmem.toml`:

```toml
[server]
port = 8080
host = "0.0.0.0"
enable_cors = true
enable_auth = false

[database]
url = "postgresql://user:pass@localhost/agentmem"
max_connections = 10

[vector_store]
provider = "qdrant"
url = "http://localhost:6333"

[llm]
provider = "openai"
api_key = "your-api-key"
model = "gpt-4"

[monitoring]
enable_metrics = true
enable_tracing = true
metrics_port = 9090
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test -p agent-mem-core

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html
```

## 📊 Monitoring & Observability

### Metrics Endpoint
- **Prometheus metrics**: `http://localhost:8080/metrics`
- **Health check**: `http://localhost:8080/health`
- **System info**: `http://localhost:8080/info`

### Grafana Dashboards
Access Grafana at `http://localhost:3000` (admin/admin) to view:
- System performance metrics
- Memory usage statistics
- API request metrics
- Error rates and latencies

### Log Aggregation
Logs are collected by Filebeat and sent to Elasticsearch. View them in Kibana at `http://localhost:5601`.

## 🔒 Security

### Authentication
AgentMem supports multiple authentication methods:
- JWT tokens
- API keys
- OAuth 2.0 (coming soon)

### Encryption
- **At Rest**: AES-256 encryption for stored data
- **In Transit**: TLS 1.3 for all communications
- **Keys**: Secure key management with rotation

### Access Control
Role-based access control (RBAC) with fine-grained permissions:
- `admin`: Full system access
- `user`: Basic memory operations
- `readonly`: Read-only access

## 🚀 Deployment

### Production Checklist

- [ ] Configure TLS certificates
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategies
- [ ] Set resource limits and quotas
- [ ] Enable security scanning
- [ ] Configure log retention policies
- [ ] Set up disaster recovery

### Scaling Guidelines

- **Horizontal Scaling**: Use Kubernetes HPA for automatic scaling
- **Vertical Scaling**: Adjust resource limits based on workload
- **Database Scaling**: Use read replicas and connection pooling
- **Cache Scaling**: Implement Redis clustering for high availability

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Run in development mode with auto-reload
cargo watch -x run

# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Inspired by [mem0](https://github.com/mem0ai/mem0) and ContextEngine
- Vector databases: [Qdrant](https://qdrant.tech/), [Pinecone](https://www.pinecone.io/), [Weaviate](https://weaviate.io/)
- Graph databases: [Neo4j](https://neo4j.com/), [Memgraph](https://memgraph.com/)

## 📞 Support

- **Documentation**: [docs.agentmem.io](https://docs.agentmem.io)
- **Issues**: [GitHub Issues](https://github.com/agentmem/agentmem/issues)
- **Discussions**: [GitHub Discussions](https://github.com/agentmem/agentmem/discussions)
- **Discord**: [Join our community](https://discord.gg/agentmem)

---

**AgentMem 2.0** - Powering the next generation of intelligent applications with advanced memory management.
