# AgentMem UI 后端 API 配置全面分析

## 📋 概述

本文档全面分析 `agentmem-ui` 项目中后端 API 的配置方式，包括配置位置、优先级、使用场景和最佳实践。

---

## 🔍 配置方式总览

### 1. 环境变量配置（主要方式）

#### 1.1 环境变量名称
- **统一使用**: `NEXT_PUBLIC_API_URL`
- **默认值**: `http://localhost:8080`
- **作用域**: 客户端和服务器端都可用（因为 `NEXT_PUBLIC_` 前缀）

#### 1.2 关键特性
⚠️ **重要**: Next.js 的 `NEXT_PUBLIC_*` 环境变量在**构建时**（build time）被嵌入到代码中，而不是运行时。

这意味着：
- ✅ 构建时设置的环境变量会生效
- ❌ 运行时设置的环境变量**不会生效**（已打包的代码已包含构建时的值）

---

## 📁 配置文件位置

### 2.1 核心配置文件

#### `src/lib/api-client.ts` (主要 API 客户端)
```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

class ApiClient {
  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }
}
```

**用途**: 
- 所有 API 请求的基础 URL
- 通过 `ApiClient` 类统一管理所有 API 调用

#### `src/lib/constants.ts` (应用常量)
```typescript
export const API_CONFIG = {
  BASE_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
  TIMEOUT: 30000,
  RETRY_ATTEMPTS: 3,
  RETRY_DELAY: 1000,
} as const;
```

**用途**: 
- 提供统一的 API 配置对象
- 包含超时、重试等配置

### 2.2 页面级配置

#### `src/app/admin/page.tsx` (仪表板)
```typescript
const API_BASE_URL = typeof window !== 'undefined' 
  ? (process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080')
  : 'http://localhost:8080';
  
const WS_URL = API_BASE_URL.replace(/^http/, 'ws') + '/api/v1/ws';
```

**用途**: 
- WebSocket 连接配置
- 实时更新功能

#### `src/app/admin/agents/page.tsx` (代理管理)
```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const WS_URL = API_BASE_URL.replace('http', 'ws') + '/api/v1/ws';
```

#### `src/app/admin/chat/page.tsx` (聊天页面)
```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

// SSE 连接
const { isConnected: sseConnected } = useSSE(`${API_BASE_URL}/api/v1/sse`, {...});

// 流式聊天
const streamUrl = `${API_BASE_URL}/api/v1/agents/${selectedAgentId}/chat/stream`;
```

#### `src/app/admin/memories/page-enhanced.tsx` (记忆管理)
```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const WS_URL = API_BASE_URL.replace('http', 'ws') + '/api/v1/ws';
```

#### `src/hooks/use-memory-search.ts` (记忆搜索 Hook)
```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

const url = `${API_BASE_URL}/api/v1/memories/search`;
```

### 2.3 用户设置页面

#### `src/app/admin/settings/page.tsx` (设置页面)
```typescript
const [apiUrl, setApiUrl] = useState('http://localhost:8080');

const handleSave = () => {
  // 保存到 localStorage（但实际代码中未使用）
  localStorage.setItem('agentmem_api_url', apiUrl);
  localStorage.setItem('agentmem_api_key', apiKey);
};
```

⚠️ **注意**: Settings 页面保存的 `agentmem_api_url` 到 localStorage，但**实际代码中并未读取使用**。这是一个未完成的功能。

---

## 🔄 配置优先级

### 3.1 当前优先级（实际）

1. **构建时环境变量** `NEXT_PUBLIC_API_URL` (最高优先级)
   - 在 `npm run build` 时设置
   - 嵌入到打包后的代码中
   - 无法在运行时修改

2. **代码默认值** `http://localhost:8080`
   - 如果构建时未设置环境变量，使用此默认值

### 3.2 理想优先级（建议实现）

1. **localStorage** (运行时动态配置) - 当前未实现
2. **构建时环境变量** `NEXT_PUBLIC_API_URL`
3. **代码默认值** `http://localhost:8080`

---

## 🛠️ 配置方法

### 4.1 方法 1: 构建时环境变量（推荐）

#### 开发环境
```bash
cd agentmem-ui
NEXT_PUBLIC_API_URL=http://localhost:8080 npm run dev
```

#### 生产构建
```bash
cd agentmem-ui
NEXT_PUBLIC_API_URL=http://your-backend:8080 npm run build
```

#### 使用构建脚本
```bash
# 在 build-release.sh 中
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}
NODE_ENV=production NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL npm run build
```

### 4.2 方法 2: .env 文件

#### 创建 `.env.local` (开发环境)
```bash
NEXT_PUBLIC_API_URL=http://localhost:8080
```

#### 创建 `.env.production` (生产环境)
```bash
NEXT_PUBLIC_API_URL=http://your-backend:8080
```

Next.js 会自动读取这些文件：
- `.env.local` - 本地开发（gitignored）
- `.env.production` - 生产构建
- `.env` - 默认配置

### 4.3 方法 3: Docker 环境变量

```yaml
# docker-compose.yml
services:
  frontend:
    build:
      context: ./agentmem-ui
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8080
```

⚠️ **注意**: 在 Docker 中，环境变量必须在**构建时**设置，而不是运行时。

### 4.4 方法 4: 运行时配置（未实现）

Settings 页面有保存功能，但代码中未实现读取：

```typescript
// 当前实现（未使用）
localStorage.setItem('agentmem_api_url', apiUrl);

// 建议实现
const getApiUrl = () => {
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('agentmem_api_url');
    if (saved) return saved;
  }
  return process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
};
```

---

## 📊 配置使用统计

### 5.1 使用 `NEXT_PUBLIC_API_URL` 的文件

| 文件 | 用途 | 类型 |
|------|------|------|
| `src/lib/api-client.ts` | 主要 API 客户端 | 核心 |
| `src/lib/constants.ts` | 应用常量配置 | 核心 |
| `src/app/admin/page.tsx` | 仪表板 WebSocket | 页面 |
| `src/app/admin/agents/page.tsx` | 代理管理 | 页面 |
| `src/app/admin/chat/page.tsx` | 聊天功能 | 页面 |
| `src/app/admin/memories/page-enhanced.tsx` | 记忆管理 | 页面 |
| `src/hooks/use-memory-search.ts` | 记忆搜索 | Hook |

### 5.2 配置方式统计

- ✅ **环境变量**: 7 个文件使用
- ⚠️ **localStorage**: 1 个文件保存（但未读取）
- ❌ **运行时配置**: 未实现

---

## ⚠️ 当前问题

### 6.1 问题 1: 构建时 vs 运行时

**问题**: 
- 环境变量在构建时嵌入，运行时无法修改
- 打包后的代码无法动态更改后端 URL

**影响**: 
- 不同环境需要重新构建
- 无法在运行时切换后端服务器

### 6.2 问题 2: Settings 页面未生效

**问题**: 
- Settings 页面保存了 `agentmem_api_url` 到 localStorage
- 但代码中未读取使用

**影响**: 
- 用户无法通过 UI 动态配置后端 URL

### 6.3 问题 3: 配置分散

**问题**: 
- 多个文件中重复定义 `API_BASE_URL`
- 没有统一的配置管理

**影响**: 
- 维护困难
- 容易出错

---

## ✅ 最佳实践建议

### 7.1 统一配置管理

创建 `src/lib/config.ts`:

```typescript
/**
 * 统一配置管理
 * 支持构建时和运行时配置
 */

function getApiUrl(): string {
  // 1. 优先使用 localStorage（运行时配置）
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('agentmem_api_url');
    if (saved) return saved;
  }
  
  // 2. 使用构建时环境变量
  if (process.env.NEXT_PUBLIC_API_URL) {
    return process.env.NEXT_PUBLIC_API_URL;
  }
  
  // 3. 默认值
  return 'http://localhost:8080';
}

export const API_CONFIG = {
  BASE_URL: getApiUrl(),
  TIMEOUT: 30000,
  RETRY_ATTEMPTS: 3,
  RETRY_DELAY: 1000,
} as const;

// 导出函数以便动态获取
export function getApiBaseUrl(): string {
  return getApiUrl();
}
```

### 7.2 更新所有文件使用统一配置

```typescript
// 替换所有文件中的
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

// 为
import { getApiBaseUrl } from '@/lib/config';
const API_BASE_URL = getApiBaseUrl();
```

### 7.3 实现 Settings 页面功能

```typescript
// src/app/admin/settings/page.tsx
useEffect(() => {
  // 加载保存的配置
  const saved = localStorage.getItem('agentmem_api_url');
  if (saved) {
    setApiUrl(saved);
  } else {
    // 使用构建时的配置
    setApiUrl(process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080');
  }
}, []);

const handleSave = () => {
  localStorage.setItem('agentmem_api_url', apiUrl);
  // 提示用户刷新页面
  alert('配置已保存，请刷新页面生效');
};
```

---

## 🚀 部署配置指南

### 8.1 开发环境

```bash
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:8080
```

### 8.2 生产环境

#### 方式 1: 构建时设置
```bash
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080 npm run build
```

#### 方式 2: .env.production
```bash
# .env.production
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080
```

#### 方式 3: Docker
```dockerfile
# Dockerfile
ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL

# 构建时
docker build --build-arg NEXT_PUBLIC_API_URL=http://api:8080 .
```

### 8.3 多环境配置

```bash
# .env.development
NEXT_PUBLIC_API_URL=http://localhost:8080

# .env.staging
NEXT_PUBLIC_API_URL=http://staging-api:8080

# .env.production
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080
```

---

## 📝 总结

### 当前状态
- ✅ 统一使用 `NEXT_PUBLIC_API_URL` 环境变量
- ✅ 所有文件已更新为统一变量名
- ⚠️ 配置在构建时嵌入，运行时无法修改
- ⚠️ Settings 页面保存但未使用

### 建议改进
1. **实现运行时配置**: 支持 localStorage 动态配置
2. **统一配置管理**: 创建 `src/lib/config.ts`
3. **完善 Settings 页面**: 实现配置读取和应用
4. **添加配置验证**: 验证 API URL 格式和可访问性

### 配置优先级（建议）
1. localStorage (运行时) - 需要实现
2. 构建时环境变量 - 已实现
3. 代码默认值 - 已实现

---

**最后更新**: 2025-12-01  
**维护者**: AgentMem Team

