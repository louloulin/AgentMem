# AgentMem 5 分钟快速开始

欢迎使用 AgentMem！本指南将帮助您在 5 分钟内快速上手 AgentMem，体验智能记忆管理的强大功能。

## 📋 前置要求

- **Docker** 20.10+ 和 **Docker Compose** 2.0+
- **Python** 3.8+ (可选，用于 SDK 示例)
- **8GB RAM** 和 **10GB 磁盘空间**

## 🚀 步骤 1: 快速安装（1 分钟）

### 方式 A: Docker Compose（推荐）

```bash
# 克隆仓库
git clone https://github.com/louloulin/agentmem.git
cd agentmem

# 启动所有服务
docker-compose up -d

# 等待服务启动（约 30-60 秒）
docker-compose ps
```

**预期输出**:
```
NAME                    STATUS              PORTS
agentmem-server         Up 30 seconds       0.0.0.0:8080->8080/tcp
agentmem-postgres       Up 30 seconds       0.0.0.0:5432->5432/tcp
agentmem-redis          Up 30 seconds       0.0.0.0:6379->6379/tcp
agentmem-qdrant         Up 30 seconds       0.0.0.0:6333->6333/tcp
agentmem-neo4j          Up 30 seconds       0.0.0.0:7474->7474/tcp
agentmem-grafana        Up 30 seconds       0.0.0.0:3000->3000/tcp
```

### 方式 B: 二进制安装（高级用户）

```bash
# 下载最新版本
curl -L https://github.com/louloulin/agentmem/releases/latest/download/agentmem-linux-amd64 -o agentmem
chmod +x agentmem

# 启动服务器（需要手动配置数据库）
./agentmem server --port 8080
```

## ✅ 步骤 2: 验证安装（30 秒）

```bash
# 检查健康状态
curl http://localhost:8080/health

# 预期输出:
# {"status":"healthy","version":"2.1.0","uptime_seconds":10}
```

**如果失败**，请查看日志：
```bash
docker-compose logs agentmem
```

## 💡 步骤 3: 第一个记忆（2 分钟）

### 使用 Python SDK

```bash
# 安装 SDK
pip install agentmem-sdk
```

```python
# 创建客户端
from agentmem import AgentMem

client = AgentMem(base_url="http://localhost:8080")

# 添加记忆
client.add("John likes pizza")
client.add("John works at Google")
client.add("John lives in San Francisco")

# 搜索记忆
results = client.search("Where does John work?")
print(results)
# 输出: [{"content": "John works at Google", "score": 0.95, ...}]

# 获取所有记忆
memories = client.get_all()
print(f"Total memories: {len(memories)}")
```

### 使用 REST API

```bash
# 添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "John likes pizza",
    "memory_type": "semantic",
    "agent_id": "default"
  }'

# 搜索记忆
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What does John like?",
    "limit": 5
  }'

# 获取所有记忆
curl http://localhost:8080/api/v1/memories
```

## 🌐 步骤 4: 查看 Web UI（1 分钟）

打开浏览器访问: **http://localhost:3000**

- **Dashboard**: 查看系统统计和活动
- **Memories**: 浏览和搜索所有记忆
- **Agents**: 管理 AI 智能体
- **Settings**: 配置系统参数

**默认登录凭据**（如果启用了认证）:
- 用户名: `admin`
- 密码: `admin`

## 🎯 步骤 5: 探索更多功能（1 分钟）

### 不同的记忆类型

AgentMem 支持 9 种记忆类型：

```python
# 情景记忆（事件）
client.add("John met Alice at the conference", memory_type="episodic")

# 程序记忆（步骤）
client.add("To make coffee: 1. Boil water 2. Add coffee 3. Stir", 
           memory_type="procedural")

# 工作记忆（临时）
client.add("Current task: Write documentation", memory_type="working")

# 语义记忆（知识）
client.add("Python is a programming language", memory_type="semantic")

# 核心记忆（重要）
client.add("User prefers dark mode", memory_type="core")
```

### Agent 管理

```python
# 创建新的 Agent
agent = client.create_agent(
    name="Assistant",
    description="Personal AI assistant",
    llm_provider="openai",
    model="gpt-4"
)

# 使用特定 Agent
client.add("Important note", agent_id=agent.id)

# 列出所有 Agents
agents = client.list_agents()
print(f"Total agents: {len(agents)}")
```

### 高级搜索

```python
# 混合搜索（向量 + 关键词）
results = client.search(
    query="John's job",
    search_type="hybrid",
    limit=10,
    filters={"memory_type": "semantic"}
)

# 按时间范围搜索
from datetime import datetime, timedelta

results = client.search(
    query="recent events",
    start_time=datetime.now() - timedelta(days=7),
    end_time=datetime.now()
)
```

## 📚 下一步

恭喜！您已经完成了 AgentMem 的快速入门。接下来可以：

- 📖 [完整文档](./README.md) - 深入了解所有功能
- 🔧 [API 参考](./api-reference.md) - 完整的 API 文档
- 💡 [示例代码](../examples/) - 更多实用示例
- 🚀 [生产部署](./production-deployment-guide.md) - 生产环境部署指南
- 💬 [社区支持](https://github.com/louloulin/agentmem/discussions) - 加入社区讨论

## ❓ 常见问题

### Q: 如何更改 LLM 提供商？

A: 在环境变量中设置:
```bash
export AGENTMEM_LLM_PROVIDER=anthropic
export ANTHROPIC_API_KEY=your-api-key
```

支持的提供商: OpenAI, Anthropic, Google Gemini, Azure OpenAI, Ollama, DeepSeek, Groq, Mistral, Cohere, AWS Bedrock 等 14 个提供商。

### Q: 如何持久化数据？

A: 默认使用 Docker volumes 持久化。如果需要外部数据库:
```bash
export AGENTMEM_DATABASE_URL=postgresql://user:pass@localhost/agentmem
export AGENTMEM_REDIS_URL=redis://localhost:6379
```

### Q: 如何扩展到多个实例？

A: 参考 [Kubernetes 部署指南](./k8s-deployment.md) 或使用 Docker Swarm:
```bash
docker stack deploy -c docker-compose.yml agentmem
```

### Q: 如何备份数据？

A: 使用内置备份脚本:
```bash
./scripts/backup.sh
```

或手动备份数据库:
```bash
docker exec agentmem-postgres pg_dump -U agentmem agentmem > backup.sql
```

### Q: 性能如何优化？

A: 主要优化点:
1. 增加 Redis 缓存大小
2. 调整 PostgreSQL 连接池
3. 使用 SSD 存储
4. 启用 Qdrant 的 HNSW 索引

详见 [性能优化指南](./performance-optimization.md)

### Q: 如何监控系统？

A: 访问 Grafana 仪表板:
- URL: http://localhost:3000
- 用户名: admin
- 密码: admin

或使用 Prometheus 指标:
- URL: http://localhost:9090

## 🔧 故障排除

### 服务无法启动

```bash
# 检查端口占用
lsof -i :8080
lsof -i :5432

# 清理并重启
docker-compose down -v
docker-compose up -d
```

### 内存不足

```bash
# 减少服务数量（最小配置）
docker-compose up -d agentmem postgres redis qdrant
```

### 连接超时

```bash
# 增加健康检查超时
# 编辑 docker-compose.yml，增加 start_period
healthcheck:
  start_period: 120s  # 从 40s 增加到 120s
```

## 📞 获取帮助

- 🐛 [报告 Bug](https://github.com/louloulin/agentmem/issues)
- 💬 [讨论区](https://github.com/louloulin/agentmem/discussions)
- 📧 Email: support@agentmem.io
- 🌐 Website: https://agentmem.io

---

**祝您使用愉快！** 🎉

如果觉得 AgentMem 有用，请给我们一个 ⭐ Star！

