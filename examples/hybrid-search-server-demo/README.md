# 混合检索Server演示

通过REST API和MCP协议验证混合检索系统。

## 🚀 快速开始

### 1. 启动Server

```bash
cd agentmen/examples/hybrid-search-server-demo
cargo run --release
```

**输出**:
```
🚀 启动混合检索Server演示
🌐 Server启动在 http://127.0.0.1:3000
📋 可用端点:
  - GET  /health           - 健康检查
  - POST /api/search       - 搜索
  - GET  /api/classify     - 查询分类
```

### 2. 测试API

#### 健康检查
```bash
curl http://localhost:3000/health
```

**响应**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "features": [
    "query-classification",
    "adaptive-threshold",
    "hybrid-search",
    "vector-search",
    "bm25-search"
  ]
}
```

#### 搜索
```bash
curl -X POST http://localhost:3000/api/search \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Apple 手机",
    "limit": 10
  }'
```

**响应**:
```json
{
  "success": true,
  "query": "Apple 手机",
  "query_type": "ShortKeyword",
  "results": [
    {
      "id": "result1",
      "content": "匹配查询 'Apple 手机' 的结果1",
      "score": 0.95
    },
    {
      "id": "result2",
      "content": "匹配查询 'Apple 手机' 的结果2",
      "score": 0.87
    }
  ],
  "stats": {
    "total_time_ms": 45,
    "vector_time_ms": 20,
    "bm25_time_ms": 15,
    "results_count": 2
  }
}
```

#### 查询分类
```bash
curl 'http://localhost:3000/api/classify?query=iPhone'
```

**响应**:
```json
{
  "query": "iPhone",
  "query_type": "ShortKeyword",
  "strategy": {
    "use_vector": true,
    "use_bm25": true,
    "vector_weight": 0.5,
    "bm25_weight": 0.5,
    "threshold": 0.1
  }
}
```

## 🔧 API端点

### GET /health

健康检查，返回服务状态和可用功能。

### POST /api/search

执行混合检索搜索。

**请求体**:
```json
{
  "query": "搜索关键词",
  "limit": 10
}
```

**响应**:
```json
{
  "success": true,
  "query": "...",
  "query_type": "...",
  "results": [...],
  "stats": {...}
}
```

### GET /api/classify

对查询进行分类并返回推荐策略。

**查询参数**:
- `query`: 要分类的查询字符串

**响应**:
```json
{
  "query": "...",
  "query_type": "...",
  "strategy": {...}
}
```

## 🧪 测试场景

### 场景1: 品牌查询
```bash
curl -X POST http://localhost:3000/api/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "Apple"}'
```

### 场景2: 自然语言查询
```bash
curl -X POST http://localhost:3000/api/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "推荐一款拍照好的手机"}'
```

### 场景3: 精确ID查询
```bash
curl -X POST http://localhost:3000/api/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "P001"}'
```

## 🔌 MCP集成

（MCP集成在完整版中实现）

### MCP工具

- `hybrid_search`: 执行混合检索
- `classify_query`: 分类查询
- `get_strategy`: 获取搜索策略

### 使用示例

```json
{
  "method": "tools/call",
  "params": {
    "name": "hybrid_search",
    "arguments": {
      "query": "Apple 手机",
      "limit": 10
    }
  }
}
```

## 📊 性能监控

Server自动记录所有请求的性能指标：

- 查询分类时间
- 向量搜索时间
- BM25搜索时间
- 结果融合时间
- 总响应时间

## 🔒 安全性

（生产环境建议添加）

- API密钥认证
- 速率限制
- CORS配置
- 请求日志审计

## 📝 注意事项

1. 这是一个演示版本，未连接真实的数据库
2. 实际生产环境需要配置：
   - 向量数据库 (LanceDB)
   - LibSQL数据库
   - Embedder服务
   - LLM服务（可选）

## 🔗 相关文档

- [混合检索系统概述](../../../FINAL_README.md)
- [API集成指南](../../../doc/technical-design/HYBRID_RETRIEVAL_IMPLEMENTATION_REPORT.md)
- [MCP协议文档](../../crates/agent-mem-tools/docs/mcp/README.md)

