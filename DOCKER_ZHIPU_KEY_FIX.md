# Docker 中 ZHIPU_API_KEY 未生效问题修复

## 问题分析

### 症状
在 Docker Compose 中设置了 `ZHIPU_API_KEY` 环境变量，但启动后 key 没有生效。

### 根本原因

1. **配置文件优先级问题**
   - 容器内可能存在 `config.toml` 文件
   - 如果 `config.toml` 中 LLM 配置为空或错误，会覆盖环境变量
   - `ServerConfig::override_from_env()` 方法**只覆盖服务器基本配置**，不覆盖 LLM 配置

2. **LLM 配置加载逻辑**
   - LLM 配置在 `orchestrator_factory.rs` 中从 Agent 配置读取
   - 如果 Agent 配置中没有 API key，才会尝试从环境变量读取
   - 格式：`{PROVIDER}_API_KEY`，对于 zhipu 就是 `ZHIPU_API_KEY`

3. **Docker 环境变量传递**
   - Docker Compose 的环境变量需要正确传递到容器内
   - 容器内的进程需要能访问到这些环境变量

## 解决方案

### 方案 1: 确保环境变量正确传递（推荐）

修改 docker-compose.yml，确保环境变量正确传递：

```yaml
version: '3.8'

services:
  agentmem:
    image: agentmem:amd64-prod
    container_name: agentmem
    restart: unless-stopped
    ports:
      - "8090:8080"
    environment:
      - RUST_LOG=info
      - AGENTMEM_HOST=0.0.0.0
      - AGENTMEM_PORT=8080
      - DATABASE_BACKEND=libsql
      - AGENTMEM_DATA_DIR=/app/data
      - AGENTMEM_LOG_DIR=/app/logs
      - AGENTMEM_CONFIG_DIR=/app/config
      - AGENTMEM_VECTOR_STORE=disabled
      # ✅ 确保 ZHIPU_API_KEY 正确设置
      - ZHIPU_API_KEY=99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k
      # ✅ 可选：显式设置 LLM Provider
      - LLM_PROVIDER=zhipu
      - LLM_MODEL=glm-4.6
    volumes:
      - agentmem_data:/app/data
      - agentmem_logs:/app/logs
      # ⚠️ 重要：不要挂载包含错误配置的 config.toml
      # - ./config.toml:/app/config/config.toml  # 如果挂载，确保配置正确
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  agentmem_data:
    driver: local
  agentmem_logs:
    driver: local
```

### 方案 2: 使用 .env 文件（更安全）

创建 `.env` 文件：

```bash
ZHIPU_API_KEY=99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k
LLM_PROVIDER=zhipu
LLM_MODEL=glm-4.6
```

docker-compose.yml:

```yaml
version: '3.8'

services:
  agentmem:
    image: agentmem:amd64-prod
    container_name: agentmem
    restart: unless-stopped
    ports:
      - "8090:8080"
    env_file:
      - .env  # ✅ 从 .env 文件加载环境变量
    environment:
      - RUST_LOG=info
      - AGENTMEM_HOST=0.0.0.0
      - AGENTMEM_PORT=8080
      # ... 其他配置
    volumes:
      - agentmem_data:/app/data
      - agentmem_logs:/app/logs
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

volumes:
  agentmem_data:
    driver: local
  agentmem_logs:
    driver: local
```

### 方案 3: 检查容器内配置

进入容器检查环境变量和配置：

```bash
# 进入容器
docker exec -it agentmem bash

# 检查环境变量
echo $ZHIPU_API_KEY

# 检查配置文件
cat /app/config/config.toml 2>/dev/null || echo "No config.toml found"

# 检查进程环境变量
ps aux | grep agent-mem-server
```

### 方案 4: 修改 Dockerfile 确保环境变量传递

如果问题仍然存在，检查 Dockerfile 是否正确处理环境变量：

```dockerfile
# 在 Dockerfile 中确保环境变量可以传递
ENV ZHIPU_API_KEY=""
ENV LLM_PROVIDER=""
ENV LLM_MODEL=""

# 运行时从环境变量读取
CMD ["./agent-mem-server"]
```

## 验证步骤

### 1. 检查环境变量是否传递

```bash
# 启动容器
docker-compose up -d

# 检查环境变量
docker exec agentmem env | grep ZHIPU
```

应该看到：
```
ZHIPU_API_KEY=99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k
```

### 2. 检查日志

```bash
# 查看启动日志
docker-compose logs agentmem | grep -i "zhipu\|llm\|api.*key"
```

应该看到类似：
```
INFO ✅ 检测到 ZHIPU_API_KEY，自动切换到 zhipu provider
```

### 3. 测试 API

```bash
# 测试健康检查
curl http://localhost:8090/health

# 测试聊天功能（如果已实现）
curl -X POST http://localhost:8090/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "你好"}'
```

## 常见问题

### Q1: 环境变量设置了但还是不生效？

**A**: 检查以下几点：
1. 容器是否重启（`docker-compose restart agentmem`）
2. 是否有 config.toml 文件覆盖了配置
3. 环境变量名称是否正确（`ZHIPU_API_KEY`，不是 `ZHIPUAI_API_KEY`）

### Q2: 如何确认环境变量在容器内？

**A**: 
```bash
docker exec agentmem printenv | grep ZHIPU
```

### Q3: 配置文件和环境变量哪个优先级更高？

**A**: 根据代码逻辑：
1. 首先加载配置文件（如果存在）
2. 然后环境变量覆盖配置文件
3. 但是 LLM 配置的特殊处理：如果 Agent 配置中没有 API key，会从环境变量读取

### Q4: 为什么需要设置 LLM_PROVIDER？

**A**: 虽然系统可以自动检测 `ZHIPU_API_KEY` 并切换到 zhipu provider，但显式设置可以确保：
- 避免自动检测失败
- 明确指定使用的 provider
- 更好的可维护性

## 最佳实践

1. **使用 .env 文件**：将敏感信息（API keys）放在 `.env` 文件中，不要提交到版本控制
2. **不要挂载错误的 config.toml**：如果挂载配置文件，确保配置正确
3. **验证环境变量**：启动后检查环境变量是否正确传递
4. **查看日志**：启动时查看日志，确认配置加载情况
5. **使用健康检查**：确保服务正常启动

## 相关文件

- `crates/agent-mem-server/src/config.rs` - 配置加载逻辑
- `crates/agent-mem-server/src/orchestrator_factory.rs` - LLM 配置解析
- `crates/agent-mem/src/orchestrator/initialization.rs` - LLM Provider 初始化

