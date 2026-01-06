# Dockerfile LLM 配置说明

## 📋 概述

已在 Dockerfile 中添加完整的 LLM Provider 配置（智谱 AI），与启动脚本 `start-with-zhipu.sh` 保持一致。

---

## ✅ Dockerfile 配置

### 环境变量配置

在 Dockerfile 的运行时阶段添加了以下环境变量：

```dockerfile
# LLM Provider configuration (Zhipu AI)
# These can be overridden at runtime if needed
ENV ZHIPU_API_KEY=""
ENV LLM_PROVIDER="zhipu"
ENV LLM_MODEL="glm-4.6"
ENV ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
```

**位置**: `Dockerfile` 第 85-90 行

### 配置说明

| 环境变量 | 默认值 | 说明 | 是否必需 |
|---------|--------|------|---------|
| `ZHIPU_API_KEY` | `""` | 智谱 AI API Key | ✅ 是（运行时必须设置） |
| `LLM_PROVIDER` | `"zhipu"` | LLM 提供商 | ✅ 是 |
| `LLM_MODEL` | `"glm-4.6"` | 使用的模型 | ✅ 是 |
| `ZHIPU_BASE_URL` | `https://open.bigmodel.cn/api/coding/paas/v4` | API 基础 URL | ⚠️ 可选 |

---

## 🔄 与启动脚本的对比

### Dockerfile 配置
```dockerfile
ENV ZHIPU_API_KEY=""
ENV LLM_PROVIDER="zhipu"
ENV LLM_MODEL="glm-4.6"
ENV ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
```

### 启动脚本配置 (`start-with-zhipu.sh`)
```bash
export ZHIPU_API_KEY="your_zhipu_api_key_here"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"
export ZHIPU_BASE_URL=${ZHIPU_BASE_URL:-"https://open.bigmodel.cn/api/coding/paas/v4"}
```

### 一致性

✅ **完全一致**:
- `LLM_PROVIDER`: `"zhipu"` ✅
- `LLM_MODEL`: `"glm-4.6"` ✅
- `ZHIPU_BASE_URL`: `https://open.bigmodel.cn/api/coding/paas/v4` ✅

⚠️ **差异说明**:
- `ZHIPU_API_KEY`: Dockerfile 中设置为空字符串（安全考虑），启动脚本中为占位符
  - 两者都需要在运行时提供实际的 API Key

---

## 🚀 使用方式

### 方式 1: Docker 运行时设置

#### 基本使用
```bash
docker run -p 8080:8080 \
  -e ZHIPU_API_KEY="your_actual_api_key_here" \
  agentmem:latest
```

#### 完整配置
```bash
docker run -p 8080:8080 \
  -e ZHIPU_API_KEY="your_actual_api_key_here" \
  -e LLM_PROVIDER="zhipu" \
  -e LLM_MODEL="glm-4.6" \
  -e ZHIPU_BASE_URL="https://open.bigmodel.cn/api/coding/paas/v4" \
  agentmem:latest
```

#### 自定义 Base URL
```bash
docker run -p 8080:8080 \
  -e ZHIPU_API_KEY="your_actual_api_key_here" \
  -e ZHIPU_BASE_URL="https://custom.url/api/v4" \
  agentmem:latest
```

### 方式 2: docker-compose

```yaml
version: '3.8'

services:
  agentmem:
    image: agentmem:latest
    ports:
      - "8080:8080"
    environment:
      - ZHIPU_API_KEY=your_actual_api_key_here
      - LLM_PROVIDER=zhipu
      - LLM_MODEL=glm-4.6
      - ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
```

### 方式 3: 使用环境变量文件

创建 `.env` 文件：
```bash
ZHIPU_API_KEY=your_actual_api_key_here
LLM_PROVIDER=zhipu
LLM_MODEL=glm-4.6
ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
```

在 docker-compose 中使用：
```yaml
services:
  agentmem:
    image: agentmem:latest
    env_file:
      - .env
```

---

## 📊 配置优先级

1. **运行时环境变量** (最高优先级)
   ```bash
   docker run -e ZHIPU_API_KEY="..." ...
   ```

2. **Dockerfile ENV** (中等优先级)
   ```dockerfile
   ENV ZHIPU_API_KEY=""
   ENV LLM_PROVIDER="zhipu"
   ENV LLM_MODEL="glm-4.6"
   ENV ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
   ```

3. **代码默认值** (最低优先级)
   - 如果环境变量未设置，代码会使用内置默认值

---

## 🔍 验证配置

### 检查环境变量

在容器内检查：
```bash
docker exec <container_id> env | grep -E "ZHIPU|LLM"
```

输出示例：
```
ZHIPU_API_KEY=your_actual_api_key_here
LLM_PROVIDER=zhipu
LLM_MODEL=glm-4.6
ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
```

### 检查应用日志

查看应用启动日志，确认 LLM Provider 配置：
```bash
docker logs <container_id> | grep -i "llm\|zhipu"
```

---

## ⚠️ 安全注意事项

### API Key 安全

1. **不要在 Dockerfile 中硬编码 API Key**
   - ✅ 当前配置：`ENV ZHIPU_API_KEY=""`（空字符串）
   - ❌ 错误做法：`ENV ZHIPU_API_KEY="actual_key"`

2. **使用运行时环境变量**
   ```bash
   docker run -e ZHIPU_API_KEY="actual_key" ...
   ```

3. **使用 Docker Secrets（生产环境推荐）**
   ```yaml
   services:
     agentmem:
       secrets:
         - zhipu_api_key
   secrets:
     zhipu_api_key:
       external: true
   ```

4. **使用环境变量文件（开发环境）**
   - 确保 `.env` 文件不被提交到版本控制
   - 添加到 `.gitignore`

---

## 📝 相关文件

- `Dockerfile` - Docker 镜像配置（已更新）
- `dist/server/start-with-zhipu.sh` - 启动脚本（参考配置）
- `crates/agent-mem/src/orchestrator/initialization.rs` - LLM Provider 初始化代码
- `crates/agent-mem-llm/src/providers/zhipu.rs` - Zhipu Provider 实现

---

## ✅ 配置完成

**状态**: ✅ **已完成**

**配置内容**:
- ✅ Dockerfile 中添加 `ZHIPU_API_KEY` 环境变量（空字符串，安全）
- ✅ Dockerfile 中添加 `LLM_PROVIDER` 环境变量（默认: "zhipu"）
- ✅ Dockerfile 中添加 `LLM_MODEL` 环境变量（默认: "glm-4.6"）
- ✅ Dockerfile 中添加 `ZHIPU_BASE_URL` 环境变量（默认: coding API URL）
- ✅ 与启动脚本配置保持一致

**安全考虑**:
- ✅ API Key 不在 Dockerfile 中硬编码
- ✅ 所有配置可在运行时覆盖
- ✅ 支持通过环境变量文件管理敏感信息

---

**最后更新**: 2025-12-02  
**状态**: ✅ 配置完成

