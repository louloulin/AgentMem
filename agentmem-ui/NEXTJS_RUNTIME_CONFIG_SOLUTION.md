# Next.js 打包后后端 URL 配置未生效 - 完整解决方案

## 🔍 问题根本原因

### Next.js 环境变量的工作机制

根据 Next.js 官方文档和实际测试，**`NEXT_PUBLIC_*` 环境变量在构建时（build time）被嵌入到客户端代码中**，而不是运行时（runtime）。

#### 构建时 vs 运行时

```
构建时 (Build Time)         运行时 (Runtime)
├─ npm run build          ├─ npm start / node server.js
├─ 环境变量被嵌入代码      ├─ 环境变量已无法改变代码
└─ 生成静态文件            └─ 执行已编译的代码
```

**关键点**：
- ✅ 构建时设置 `NEXT_PUBLIC_API_URL` → **会生效**
- ❌ 运行时设置 `NEXT_PUBLIC_API_URL` → **不会生效**（代码已包含构建时的值）

### 当前问题分析

#### 问题 1: 启动脚本设置无效

```bash
# dist/ui/start.sh (第 7 行)
export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}
```

**为什么无效**：
- 这个环境变量在**运行时**设置
- 但代码在**构建时**已经包含了环境变量的值
- 运行时设置无法改变已编译的代码

#### 问题 2: 构建脚本可能未设置

```bash
# build-release.sh
NODE_ENV=production npm run build
```

如果构建时没有设置 `NEXT_PUBLIC_API_URL`，代码会使用默认值 `http://localhost:8080`。

---

## ✅ 解决方案

### 方案 1: 构建时设置环境变量（推荐，已修复）

#### 1.1 修改构建脚本

已在 `build-release.sh` 中修复：

```bash
# 构建前端
build_ui() {
    cd "$UI_DIR"
    
    # ✅ 在构建时设置环境变量
    export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}
    log_info "构建时 API URL: $NEXT_PUBLIC_API_URL"
    
    # 构建时传递环境变量
    NODE_ENV=production NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL npm run build
}
```

#### 1.2 使用方法

```bash
# 方式 1: 通过环境变量
export NEXT_PUBLIC_API_URL=http://your-backend:8080
./build-release.sh

# 方式 2: 直接在命令中设置
NEXT_PUBLIC_API_URL=http://your-backend:8080 ./build-release.sh
```

### 方案 2: 使用 .env.production 文件

#### 2.1 创建配置文件

在 `agentmem-ui/` 目录创建 `.env.production`：

```bash
# .env.production
NEXT_PUBLIC_API_URL=http://your-backend:8080
```

#### 2.2 Next.js 自动读取

Next.js 在构建时会自动读取：
- `.env.production.local` (最高优先级，gitignored)
- `.env.production`
- `.env.local` (gitignored)
- `.env`

### 方案 3: 运行时动态配置（高级方案）

如果需要在运行时动态配置，需要使用 Next.js 的服务器端配置。

#### 3.1 创建运行时配置 API

创建 `src/app/api/config/route.ts`：

```typescript
import { NextResponse } from 'next/server';

export async function GET() {
  // 从环境变量或配置文件读取
  const apiUrl = process.env.API_URL || 
                 process.env.NEXT_PUBLIC_API_URL || 
                 'http://localhost:8080';
  
  return NextResponse.json({
    apiUrl,
    // 其他运行时配置
  });
}
```

#### 3.2 客户端获取配置

创建 `src/lib/runtime-config.ts`：

```typescript
let cachedConfig: { apiUrl: string } | null = null;

export async function getRuntimeConfig() {
  if (cachedConfig) return cachedConfig;
  
  try {
    const response = await fetch('/api/config');
    cachedConfig = await response.json();
    return cachedConfig;
  } catch (error) {
    console.error('Failed to load runtime config:', error);
    // 回退到构建时配置
    return {
      apiUrl: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080'
    };
  }
}

export function getApiBaseUrl(): string {
  // 优先使用 localStorage（如果用户设置了）
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('agentmem_api_url');
    if (saved) return saved;
  }
  
  // 使用构建时配置
  return process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
}
```

#### 3.3 使用运行时配置

```typescript
// src/lib/api-client.ts
import { getApiBaseUrl } from './runtime-config';

// 动态获取 API URL
const API_BASE_URL = getApiBaseUrl();

class ApiClient {
  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }
}
```

### 方案 4: 使用 Next.js 中间件（推荐用于运行时配置）

#### 4.1 创建中间件

创建 `src/middleware.ts`：

```typescript
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  // 从请求头或 cookie 读取配置
  const apiUrl = request.headers.get('x-api-url') || 
                 request.cookies.get('api_url')?.value ||
                 process.env.NEXT_PUBLIC_API_URL ||
                 'http://localhost:8080';
  
  // 将配置注入到响应头
  const response = NextResponse.next();
  response.headers.set('x-api-url', apiUrl);
  
  return response;
}

export const config = {
  matcher: '/api/:path*',
};
```

#### 4.2 客户端读取配置

```typescript
// 从响应头读取
const apiUrl = response.headers.get('x-api-url') || 
               process.env.NEXT_PUBLIC_API_URL || 
               'http://localhost:8080';
```

---

## 🚀 实际部署方案

### 场景 1: 单环境部署（推荐）

**使用构建时配置**：

```bash
# 1. 设置环境变量
export NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080

# 2. 构建
cd agentmem-ui
npm run build

# 3. 部署
# 构建产物已包含正确的 API URL
```

### 场景 2: 多环境部署

**使用 .env 文件**：

```bash
# 开发环境
# .env.development
NEXT_PUBLIC_API_URL=http://localhost:8080

# 生产环境
# .env.production
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080

# 构建时自动选择
NODE_ENV=production npm run build  # 使用 .env.production
```

### 场景 3: Docker 部署

#### Dockerfile

```dockerfile
# 构建阶段
FROM node:18-alpine AS builder

WORKDIR /app

# 复制文件
COPY package*.json ./
COPY . .

# 构建时传入环境变量
ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL

# 构建
RUN npm run build

# 运行阶段
FROM node:18-alpine AS runner
WORKDIR /app

# 复制构建产物
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

EXPOSE 3000
CMD ["node", "server.js"]
```

#### 构建和运行

```bash
# 构建时传入环境变量
docker build --build-arg NEXT_PUBLIC_API_URL=http://api:8080 -t agentmem-ui .

# 运行（注意：运行时设置的环境变量不会影响已构建的代码）
docker run -p 3000:3000 agentmem-ui
```

### 场景 4: 需要运行时动态配置

如果确实需要在运行时动态配置，使用**方案 3 或方案 4**。

---

## 🔧 修复当前问题

### 步骤 1: 确保构建时设置环境变量

```bash
# 检查构建脚本
grep -A 5 "构建 Next.js" build-release.sh

# 应该看到：
# export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}
# NODE_ENV=production NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL npm run build
```

### 步骤 2: 重新构建

```bash
# 设置正确的 API URL
export NEXT_PUBLIC_API_URL=http://your-backend:8080

# 重新构建
./build-release.sh
```

### 步骤 3: 验证构建产物

```bash
# 检查构建产物中的环境变量
cd dist/ui/.next/standalone
grep -r "localhost:8080" . | head -5

# 或者检查 JavaScript 文件
grep -r "NEXT_PUBLIC_API_URL" .next/static/chunks/ | head -3
```

### 步骤 4: 测试

```bash
# 启动服务
cd dist/ui
./start.sh

# 在浏览器中检查
# 打开开发者工具 -> Network -> 查看 API 请求的 URL
```

---

## 📊 配置优先级总结

### 当前实现（构建时配置）

1. **构建时环境变量** `NEXT_PUBLIC_API_URL` (最高优先级)
2. **代码默认值** `http://localhost:8080`

### 建议实现（支持运行时）

1. **localStorage** `agentmem_api_url` (运行时，最高优先级)
2. **构建时环境变量** `NEXT_PUBLIC_API_URL`
3. **代码默认值** `http://localhost:8080`

---

## ⚠️ 重要注意事项

### 1. 环境变量命名

- ✅ **客户端可用**: `NEXT_PUBLIC_*` 前缀
- ❌ **仅服务器端**: 不带 `NEXT_PUBLIC_` 前缀

### 2. 构建时 vs 运行时

- ✅ **构建时设置**: 会嵌入到代码中，生效
- ❌ **运行时设置**: 不会改变已编译的代码，无效

### 3. Standalone 模式

- Standalone 模式仍然遵循构建时配置的规则
- 运行时环境变量不会影响客户端代码

### 4. 安全考虑

- `NEXT_PUBLIC_*` 变量会暴露到浏览器
- 不要在其中存储敏感信息（API keys、密码等）

---

## 🎯 最佳实践

### 1. 开发环境

```bash
# .env.local (gitignored)
NEXT_PUBLIC_API_URL=http://localhost:8080
```

### 2. 生产环境

```bash
# .env.production
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080

# 或通过构建脚本
NEXT_PUBLIC_API_URL=http://api.yourdomain.com:8080 npm run build
```

### 3. 多环境

```bash
# 使用不同的 .env 文件
.env.development
.env.staging
.env.production
```

### 4. CI/CD

```yaml
# GitHub Actions 示例
- name: Build
  env:
    NEXT_PUBLIC_API_URL: ${{ secrets.API_URL }}
  run: npm run build
```

---

## 📝 总结

### 问题根源
- Next.js 的 `NEXT_PUBLIC_*` 环境变量在**构建时**嵌入代码
- 运行时设置的环境变量**不会生效**

### 解决方案
1. ✅ **构建时设置**（已修复）- 推荐
2. ✅ **使用 .env 文件** - 简单
3. ⚠️ **运行时配置** - 需要额外实现

### 当前状态
- ✅ 构建脚本已修复，支持构建时设置环境变量
- ✅ 所有文件已统一使用 `NEXT_PUBLIC_API_URL`
- ⚠️ 运行时配置功能未实现（可选）

### 下一步
1. 确保构建时设置正确的 `NEXT_PUBLIC_API_URL`
2. 重新构建应用
3. 验证构建产物中的 API URL
4. （可选）实现运行时配置功能

---

**最后更新**: 2025-12-01  
**参考**: Next.js 官方文档 - Environment Variables

