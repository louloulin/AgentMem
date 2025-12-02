# ZHIPU_BASE_URL 配置说明

## 📋 概述

已为 AgentMem 添加 `ZHIPU_BASE_URL` 环境变量支持，允许自定义智谱 AI API 的基础 URL。

**默认值**: `https://open.bigmodel.cn/api/coding/paas/v4`

---

## ✅ 配置位置

### 1. Dockerfile

在 Dockerfile 的运行时阶段添加了环境变量：

```dockerfile
# LLM Provider configuration (Zhipu AI)
# ZHIPU_BASE_URL can be overridden at runtime if needed
ENV ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
```

**位置**: `Dockerfile` 第 87 行

### 2. 启动脚本

在 `start-with-zhipu.sh` 中添加了环境变量：

```bash
# 支持自定义 Zhipu API Base URL (可选，默认: https://open.bigmodel.cn/api/paas/v4)
export ZHIPU_BASE_URL=${ZHIPU_BASE_URL:-"https://open.bigmodel.cn/api/coding/paas/v4"}
```

**位置**: `dist/server/start-with-zhipu.sh` 第 33 行

### 3. 代码支持

在 `crates/agent-mem/src/orchestrator/initialization.rs` 中添加了环境变量读取：

```rust
// 从环境变量读取 base_url（如果提供）
let base_url = match final_provider.to_lowercase().as_str() {
    "zhipu" => std::env::var("ZHIPU_BASE_URL").ok(),
    "openai" => std::env::var("OPENAI_BASE_URL").ok(),
    "anthropic" => std::env::var("ANTHROPIC_BASE_URL").ok(),
    "deepseek" => std::env::var("DEEPSEEK_BASE_URL").ok(),
    "huawei_maas" => std::env::var("HUAWEI_MAAS_BASE_URL").ok(),
    _ => None,
};
```

**位置**: 
- `create_llm_provider()` 函数（第 283-291 行）
- `create_llm_provider_with_config()` 函数（第 329-337 行）

---

## 🚀 使用方式

### 方式 1: Docker 容器

#### 使用默认值
```bash
docker run -p 8080:8080 agentmem:latest
```

#### 运行时覆盖
```bash
docker run -p 8080:8080 \
  -e ZHIPU_API_KEY="your_api_key" \
  -e ZHIPU_BASE_URL="https://custom.url/api/v4" \
  agentmem:latest
```

#### 使用 docker-compose
```yaml
services:
  agentmem:
    image: agentmem:latest
    environment:
      - ZHIPU_API_KEY=your_api_key
      - ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
    ports:
      - "8080:8080"
```

### 方式 2: 启动脚本

#### 使用默认值
```bash
./dist/server/start-with-zhipu.sh
```

#### 执行前覆盖
```bash
export ZHIPU_BASE_URL="https://custom.url/api/v4"
./dist/server/start-with-zhipu.sh
```

#### 单次执行覆盖
```bash
ZHIPU_BASE_URL="https://custom.url/api/v4" ./dist/server/start-with-zhipu.sh
```

### 方式 3: 环境变量文件

创建 `.env` 文件：
```bash
ZHIPU_API_KEY=your_api_key
ZHIPU_BASE_URL=https://open.bigmodel.cn/api/coding/paas/v4
LLM_PROVIDER=zhipu
LLM_MODEL=glm-4.6
```

使用 docker-compose 加载：
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
   - Docker: `docker run -e ZHIPU_BASE_URL=...`
   - 脚本: `ZHIPU_BASE_URL=... ./start-with-zhipu.sh`

2. **Dockerfile ENV** (中等优先级)
   - 在 Dockerfile 中设置的默认值

3. **启动脚本默认值** (最低优先级)
   - 在启动脚本中使用 `${VAR:-default}` 语法

---

## 🔍 验证配置

### 检查环境变量

在容器内检查：
```bash
docker exec <container_id> env | grep ZHIPU_BASE_URL
```

在脚本执行时检查：
```bash
# 在 start-with-zhipu.sh 中已包含显示
./dist/server/start-with-zhipu.sh
# 输出会显示: Zhipu Base URL: https://open.bigmodel.cn/api/coding/paas/v4
```

### 检查代码读取

查看日志输出，确认使用的 Base URL：
```
🔵 Zhipu API 请求开始
   模型: glm-4.6
   URL: https://open.bigmodel.cn/api/coding/paas/v4/chat/completions
```

---

## 📝 相关文件

- `Dockerfile` - Docker 镜像配置
- `dist/server/start-with-zhipu.sh` - 启动脚本
- `crates/agent-mem/src/orchestrator/initialization.rs` - LLM Provider 初始化代码
- `crates/agent-mem-llm/src/providers/zhipu.rs` - Zhipu Provider 实现

---

## ✅ 配置完成

**状态**: ✅ **已完成**

**配置内容**:
- ✅ Dockerfile 中添加 `ZHIPU_BASE_URL` 环境变量
- ✅ 启动脚本中添加 `ZHIPU_BASE_URL` 环境变量
- ✅ 代码支持从环境变量读取 `ZHIPU_BASE_URL`
- ✅ 所有配置使用相同的默认值

**默认值**: `https://open.bigmodel.cn/api/coding/paas/v4`

---

**最后更新**: 2025-12-02  
**状态**: ✅ 配置完成

