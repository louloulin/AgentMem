# AgentMem Docker 部署指南

本目录包含 AgentMem 项目的 Docker 部署配置，支持完整的向量数据库生态系统。

## 📋 支持的向量数据库

### 核心向量数据库
- **Milvus**: 高性能向量数据库，支持大规模向量搜索
- **Chroma**: 轻量级向量数据库，适合开发和小规模部署
- **Weaviate**: 支持 GraphQL 的向量数据库
- **Qdrant**: Rust 编写的高性能向量数据库

### 传统数据库向量扩展
- **Elasticsearch**: 支持向量搜索的搜索引擎
- **PostgreSQL + pgvector**: 关系数据库向量扩展
- **Redis**: 内存数据库向量搜索

### 云服务集成
- **Pinecone**: 托管向量数据库服务
- **Supabase**: 基于 PostgreSQL 的云数据库

## 🚀 快速启动

### 启动所有服务
```bash
# 启动完整的向量数据库栈
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 启动特定服务
```bash
# 只启动 Milvus
docker-compose up -d milvus etcd minio

# 只启动 Chroma
docker-compose up -d chroma

# 只启动 Qdrant
docker-compose up -d qdrant
```

## 🔧 配置说明

### 环境变量
复制 `.env.example` 到 `.env` 并根据需要修改配置：

```bash
cp .env.example .env
```

### 数据持久化
所有数据库数据都持久化到 `./data/` 目录：
- `./data/milvus/` - Milvus 数据
- `./data/chroma/` - Chroma 数据
- `./data/qdrant/` - Qdrant 数据
- `./data/elasticsearch/` - Elasticsearch 数据
- `./data/postgres/` - PostgreSQL 数据
- `./data/redis/` - Redis 数据

## 📊 服务端口

| 服务 | 端口 | 协议 | 用途 |
|------|------|------|------|
| Milvus | 19530 | gRPC | 向量操作 |
| Milvus | 9091 | HTTP | 管理界面 |
| Chroma | 8000 | HTTP | REST API |
| Weaviate | 8080 | HTTP | GraphQL/REST |
| Qdrant | 6333 | HTTP | REST API |
| Qdrant | 6334 | gRPC | gRPC API |
| Elasticsearch | 9200 | HTTP | REST API |
| PostgreSQL | 5432 | TCP | SQL |
| Redis | 6379 | TCP | Redis 协议 |

## 🔍 健康检查

### 检查所有服务状态
```bash
# 使用 AgentMem 内置健康检查
cargo run --example health-check

# 或者手动检查
curl http://localhost:8000/api/v1/heartbeat  # Chroma
curl http://localhost:8080/v1/.well-known/ready  # Weaviate
curl http://localhost:6333/health  # Qdrant
curl http://localhost:9200/_cluster/health  # Elasticsearch
```

## 🧪 测试和验证

### 运行集成测试
```bash
# 启动测试环境
docker-compose -f docker-compose.test.yml up -d

# 运行集成测试
cargo test --features integration-tests

# 清理测试环境
docker-compose -f docker-compose.test.yml down -v
```

## 📈 性能调优

### Milvus 性能优化
- 调整 `MILVUS_CONFIG_PATH` 中的配置
- 根据数据量调整 `segment.maxSize` 和 `segment.sealProportion`
- 使用 GPU 加速：设置 `gpu.enable=true`

### Elasticsearch 性能优化
- 调整 JVM 堆大小：`ES_JAVA_OPTS=-Xms2g -Xmx2g`
- 优化索引设置：`number_of_shards` 和 `number_of_replicas`

### PostgreSQL 性能优化
- 调整 `shared_buffers` 和 `work_mem`
- 优化 pgvector 索引：`ivfflat` vs `hnsw`

## 🔒 安全配置

### 生产环境安全
1. **修改默认密码**：更新 `.env` 中的所有密码
2. **网络隔离**：使用自定义网络，限制外部访问
3. **TLS 加密**：启用 HTTPS/TLS 连接
4. **访问控制**：配置防火墙规则

### 示例安全配置
```yaml
# 在 docker-compose.yml 中添加
networks:
  agentmem-internal:
    driver: bridge
    internal: true
```

## 🚨 故障排除

### 常见问题

#### Milvus 启动失败
```bash
# 检查依赖服务
docker-compose logs etcd minio

# 重置 Milvus 数据
docker-compose down
sudo rm -rf ./data/milvus
docker-compose up -d
```

#### 内存不足
```bash
# 检查系统资源
docker stats

# 调整服务资源限制
# 编辑 docker-compose.yml 中的 mem_limit
```

#### 端口冲突
```bash
# 检查端口占用
netstat -tulpn | grep :19530

# 修改 docker-compose.yml 中的端口映射
```

## 📚 相关文档

- [Milvus 官方文档](https://milvus.io/docs)
- [Chroma 文档](https://docs.trychroma.com/)
- [Weaviate 文档](https://weaviate.io/developers/weaviate)
- [Qdrant 文档](https://qdrant.tech/documentation/)
- [AgentMem 集成指南](../docs/integration.md)
