# AgentMem UI 打包与后端配置完整分析

**版本**: 1.0  
**日期**: 2025-01-20  
**项目**: AgentMem UI (Next.js 15.5.2)

---

## 📋 目录

1. [项目架构概览](#1-项目架构概览)
2. [打包配置分析](#2-打包配置分析)
3. [后端配置详解](#3-后端配置详解)
4. [环境变量配置](#4-环境变量配置)
5. [构建与部署流程](#5-构建与部署流程)
6. [常见问题与解决方案](#6-常见问题与解决方案)

---

## 1. 项目架构概览

### 1.1 技术栈

- **框架**: Next.js 15.5.2 (React 19.1.0)
- **语言**: TypeScript 5.x
- **样式**: Tailwind CSS 3.4.17
- **UI组件**: Radix UI + shadcn/ui
- **包管理器**: npm/pnpm/yarn (支持多种)
- **构建输出**: Standalone 模式

### 1.2 项目结构

```
agentmem-ui/
├── src/
│   ├── app/              # Next.js App Router 页面
│   ├── components/       # React 组件
│   ├── lib/              # 工具库和 API 客户端
│   │   ├── api-client.ts    # 后端 API 客户端
│   │   └── constants.ts     # 应用常量
│   ├── hooks/            # React Hooks
│   └── contexts/         # React Contexts
├── public/               # 静态资源
├── next.config.ts        # Next.js 配置
├── package.json          # 项目依赖
└── .env.local            # 环境变量（需创建）
```

---

## 2. 打包配置分析

### 2.1 Next.js 配置 (`next.config.ts`)

```typescript
const nextConfig: NextConfig = {
  // ✅ Standalone 输出模式
  // 生成独立的部署包，包含所有依赖
  output: 'standalone',
  
  // ✅ Turbopack 配置（Next.js 15.5.2 推荐）
  turbopack: {
    root: process.cwd(),
    rules: {
      '*.svg': {
        loaders: ['@svgr/webpack'],
        as: '*.js',
      },
    },
  },
  
  // ✅ 生产环境移除 console
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
  },
  
  // ✅ 图片优化
  images: {
    formats: ['image/webp', 'image/avif'],
  },
  
  // ✅ 构建时忽略 ESLint 警告
  eslint: {
    ignoreDuringBuilds: true,
  },
};
```

**关键配置说明**:

1. **`output: 'standalone'`**
   - 生成独立的部署包
   - 包含所有必要的 Node.js 依赖
   - 适合 Docker 容器化部署
   - 输出目录: `.next/standalone/`

2. **Turbopack**
   - Next.js 15 的新构建引擎
   - 比 Webpack 快 10x
   - 支持增量编译

3. **图片优化**
   - 自动转换为 WebP/AVIF 格式
   - 减少带宽使用

### 2.2 构建脚本 (`package.json`)

```json
{
  "scripts": {
    "dev": "next dev --port 3001",      // 开发模式，端口 3001
    "build": "next build",               // 生产构建
    "start": "next start",               // 生产服务器
    "lint": "eslint"                     // 代码检查
  }
}
```

**构建流程**:

```bash
# 1. 安装依赖
npm install
# 或
pnpm install

# 2. 构建生产版本
npm run build

# 3. 启动生产服务器
npm run start
```

**构建输出结构**:

```
.next/
├── standalone/          # Standalone 部署包
│   ├── server.js        # 服务器入口
│   ├── node_modules/    # 运行时依赖
│   └── public/          # 静态资源
├── static/              # 静态资源
└── server/              # 服务器端代码
```

---

## 3. 后端配置详解

### 3.1 API 客户端配置 (`src/lib/api-client.ts`)

#### 3.1.1 后端地址配置

```typescript
// 默认后端地址
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
```

**配置优先级**:
1. 环境变量 `NEXT_PUBLIC_API_URL` (最高优先级)
2. 默认值 `http://localhost:8080`

#### 3.1.2 认证配置

```typescript
// 默认用户和组织 ID（开发环境）
export const DEFAULT_USER_ID = 'default';
export const DEFAULT_ORG_ID = 'default-org';

// API 请求头
headers: {
  'Content-Type': 'application/json',
  'X-User-ID': DEFAULT_USER_ID,           // 必需
  'X-Organization-ID': DEFAULT_ORG_ID,    // 必需
  'Authorization': `Bearer ${token}`,      // 可选（生产环境）
}
```

**重要说明**:
- 开发环境使用默认 ID (`default`, `default-org`)
- 生产环境应使用实际认证 token
- 后端必须支持这些请求头

#### 3.1.3 API 端点列表

| 功能 | 端点 | 方法 |
|------|------|------|
| 获取所有代理 | `/api/v1/agents` | GET |
| 创建代理 | `/api/v1/agents` | POST |
| 获取代理详情 | `/api/v1/agents/{id}` | GET |
| 更新代理 | `/api/v1/agents/{id}` | PUT |
| 删除代理 | `/api/v1/agents/{id}` | DELETE |
| 发送聊天消息 | `/api/v1/agents/{id}/chat` | POST |
| LumosAI 聊天 | `/api/v1/agents/{id}/chat/lumosai` | POST |
| 获取记忆 | `/api/v1/agents/{id}/memories` | GET |
| 获取所有记忆 | `/api/v1/memories` | GET |
| 搜索记忆 | `/api/v1/memories/search` | POST |
| 创建记忆 | `/api/v1/memories` | POST |
| 删除记忆 | `/api/v1/memories/{id}` | DELETE |
| 工作记忆 | `/api/v1/working-memory` | GET/POST/DELETE |
| 获取用户 | `/api/v1/users` | GET |
| 仪表板统计 | `/api/v1/stats/dashboard` | GET |
| 记忆增长统计 | `/api/v1/stats/memories/growth` | GET |
| 代理活动统计 | `/api/v1/stats/agents/activity` | GET |
| 记忆质量统计 | `/api/v1/stats/memory/quality` | GET |
| 系统健康检查 | `/health` | GET |
| 系统指标 | `/metrics` | GET |

### 3.2 常量配置 (`src/lib/constants.ts`)

#### 3.2.1 默认认证信息

```typescript
// 必须与后端匹配
export const DEFAULT_USER_ID = 'default';
export const DEFAULT_ORG_ID = 'default-org';
export const DEFAULT_ROLES = ['admin', 'user'];
```

**后端对应配置** (Rust):
```rust
// crates/agent-mem-server/src/middleware/auth.rs
pub struct AuthUser {
    pub user_id: String,      // 默认: "default"
    pub org_id: String,       // 默认: "default-org"
    pub roles: Vec<String>,   // 默认: ["admin", "user"]
}
```

#### 3.2.2 API 配置

```typescript
export const API_CONFIG = {
  BASE_URL: process.env.NEXT_PUBLIC_API_BASE_URL || 'http://localhost:8080',
  TIMEOUT: 30000,        // 30 秒超时
  RETRY_ATTEMPTS: 3,    // 重试 3 次
  RETRY_DELAY: 1000,    // 重试延迟 1 秒
} as const;
```

#### 3.2.3 缓存配置

```typescript
export const CACHE_CONFIG = {
  DEFAULT_TTL: 30000,    // 30 秒
  AGENTS_TTL: 60000,     // 1 分钟
  MEMORIES_TTL: 30000,   // 30 秒
  STATS_TTL: 10000,      // 10 秒
} as const;
```

### 3.3 后端连接验证

#### 3.3.1 健康检查

```bash
# 检查后端是否运行
curl http://localhost:8080/health

# 预期响应
{
  "status": "ok",
  "timestamp": "2025-01-20T10:00:00Z"
}
```

#### 3.3.2 CORS 配置

后端必须允许前端域名访问：

```rust
// 后端 CORS 配置示例
Cors::default()
    .allow_origin("http://localhost:3001")  // 开发环境
    .allow_origin("https://yourdomain.com")  // 生产环境
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([
        header::CONTENT_TYPE,
        header::AUTHORIZATION,
        HeaderName::from_static("x-user-id"),
        HeaderName::from_static("x-organization-id"),
    ])
```

---

## 4. 环境变量配置

### 4.1 必需的环境变量

创建 `.env.local` 文件（开发环境）或 `.env.production`（生产环境）：

```bash
# 后端 API 地址（必需）
NEXT_PUBLIC_API_URL=http://localhost:8080

# 生产环境示例
# NEXT_PUBLIC_API_URL=https://api.yourdomain.com
```

### 4.2 可选的环境变量

```bash
# API 认证密钥（生产环境）
NEXT_PUBLIC_API_KEY=your-api-key-here

# Google Analytics（可选）
NEXT_PUBLIC_GA_ID=G-XXXXXXXXXX

# 服务器端口（默认 3000）
PORT=3001

# 运行环境
NODE_ENV=production
```

### 4.3 环境变量使用规则

1. **`NEXT_PUBLIC_*` 前缀**
   - 这些变量会暴露到浏览器
   - 只能包含非敏感信息
   - 在构建时注入到客户端代码

2. **服务器端变量**
   - 不使用 `NEXT_PUBLIC_` 前缀
   - 仅在服务器端可用
   - 可以包含敏感信息

3. **优先级**
   ```
   .env.production.local > .env.local > .env.production > .env
   ```

### 4.4 不同环境的配置

#### 开发环境 (`.env.local`)

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080
NODE_ENV=development
```

#### 生产环境 (`.env.production`)

```bash
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
NEXT_PUBLIC_API_KEY=prod-api-key-here
NODE_ENV=production
```

#### Docker 环境

```yaml
# docker-compose.yml
environment:
  - NEXT_PUBLIC_API_URL=http://backend:8080
  - NODE_ENV=production
```

---

## 5. 构建与部署流程

### 5.1 本地开发流程

```bash
# 1. 安装依赖
npm install

# 2. 配置环境变量
cp .env.example .env.local
# 编辑 .env.local，设置 NEXT_PUBLIC_API_URL

# 3. 启动开发服务器
npm run dev

# 4. 访问应用
# http://localhost:3001
```

### 5.2 生产构建流程

```bash
# 1. 设置生产环境变量
export NODE_ENV=production
export NEXT_PUBLIC_API_URL=https://api.yourdomain.com

# 2. 安装依赖（生产依赖）
npm ci --production=false

# 3. 构建应用
npm run build

# 4. 验证构建
ls -la .next/standalone/

# 5. 启动生产服务器
npm run start
```

### 5.3 Docker 部署流程

#### 5.3.1 创建 Dockerfile

```dockerfile
# 构建阶段
FROM node:18-alpine AS builder

WORKDIR /app

# 复制依赖文件
COPY package*.json ./
COPY pnpm-lock.yaml* ./

# 安装依赖
RUN npm install -g pnpm
RUN pnpm install --frozen-lockfile

# 复制源代码
COPY . .

# 构建应用
RUN pnpm run build

# 生产阶段
FROM node:18-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production

# 创建非 root 用户
RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# 复制构建产物
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

# 设置权限
RUN chown -R nextjs:nodejs /app

USER nextjs

EXPOSE 3000

ENV PORT=3000

CMD ["node", "server.js"]
```

#### 5.3.2 构建和运行

```bash
# 构建镜像
docker build -t agentmem-ui:latest .

# 运行容器
docker run -d \
  -p 3000:3000 \
  -e NEXT_PUBLIC_API_URL=http://backend:8080 \
  --name agentmem-ui \
  agentmem-ui:latest

# 或使用 docker-compose
docker-compose up -d
```

### 5.4 部署检查清单

- [ ] 环境变量已正确配置
- [ ] 后端服务已启动并可访问
- [ ] CORS 配置正确
- [ ] 健康检查通过 (`/health`)
- [ ] API 端点可访问
- [ ] 静态资源可加载
- [ ] 错误日志正常记录

---

## 6. 常见问题与解决方案

### 6.1 后端连接失败

**错误**: `Failed to fetch` 或 `Network Error`

**解决方案**:

1. **检查后端服务状态**
   ```bash
   curl http://localhost:8080/health
   ```

2. **检查环境变量**
   ```bash
   # 确认 .env.local 中的配置
   cat .env.local | grep NEXT_PUBLIC_API_URL
   ```

3. **检查 CORS 配置**
   - 确认后端允许前端域名
   - 检查请求头是否正确

4. **检查网络连接**
   ```bash
   # 测试连接
   curl -v http://localhost:8080/api/v1/agents \
     -H "X-User-ID: default" \
     -H "X-Organization-ID: default-org"
   ```

### 6.2 认证失败

**错误**: `401 Unauthorized` 或 `403 Forbidden`

**解决方案**:

1. **检查请求头**
   ```typescript
   // 确认包含必需的请求头
   headers: {
     'X-User-ID': 'default',
     'X-Organization-ID': 'default-org',
   }
   ```

2. **检查后端认证中间件**
   - 确认后端支持默认认证
   - 检查认证中间件配置

3. **生产环境使用 Token**
   ```typescript
   apiClient.setToken('your-production-token');
   ```

### 6.3 构建失败

**错误**: `Error: Cannot find module` 或类型错误

**解决方案**:

1. **清理并重新安装**
   ```bash
   rm -rf node_modules .next
   npm install
   npm run build
   ```

2. **检查 TypeScript 配置**
   ```bash
   npx tsc --noEmit
   ```

3. **检查路径别名**
   ```json
   // tsconfig.json
   {
     "compilerOptions": {
       "paths": {
         "@/*": ["./src/*"]
       }
     }
   }
   ```

### 6.4 端口冲突

**错误**: `Error: listen EADDRINUSE: address already in use :::3001`

**解决方案**:

```bash
# 查找占用端口的进程
lsof -i :3001

# 杀死进程
kill -9 <PID>

# 或使用不同端口
PORT=3002 npm run dev
```

### 6.5 缓存问题

**问题**: 数据不更新或显示旧数据

**解决方案**:

1. **清除浏览器缓存**
   - 硬刷新: `Cmd+Shift+R` (Mac) 或 `Ctrl+Shift+R` (Windows)

2. **清除 API 客户端缓存**
   ```typescript
   // 在浏览器控制台
   apiClient.invalidateCache();
   ```

3. **检查缓存 TTL**
   - 确认缓存配置合理
   - 必要时调整 `CACHE_CONFIG`

---

## 7. 性能优化建议

### 7.1 构建优化

1. **启用压缩**
   ```typescript
   // next.config.ts
   const nextConfig = {
     compress: true,
   };
   ```

2. **代码分割**
   - Next.js 自动进行代码分割
   - 使用动态导入减少初始包大小

3. **图片优化**
   - 使用 `next/image` 组件
   - 启用 WebP/AVIF 格式

### 7.2 运行时优化

1. **API 缓存**
   - 合理设置缓存 TTL
   - 使用请求去重

2. **错误重试**
   - 指数退避重试
   - 避免频繁请求

3. **请求合并**
   - 合并多个 API 请求
   - 使用批量接口

---

## 8. 监控和调试

### 8.1 开发工具

1. **浏览器 DevTools**
   - Network 标签查看 API 请求
   - Console 查看日志和错误

2. **Next.js 调试**
   ```bash
   # 启用详细日志
   DEBUG=* npm run dev
   ```

### 8.2 生产监控

1. **错误追踪**
   - 集成 Sentry
   - 记录 API 错误

2. **性能监控**
   - 使用 Vercel Analytics
   - 监控 API 响应时间

3. **日志管理**
   ```bash
   # PM2 日志
   pm2 logs agentmem-ui
   
   # Docker 日志
   docker logs -f agentmem-ui
   ```

---

## 9. 安全建议

1. **环境变量安全**
   - 不要提交 `.env.local` 到 Git
   - 使用密钥管理服务（生产环境）

2. **API 安全**
   - 使用 HTTPS（生产环境）
   - 实现 API 限流
   - 验证输入数据

3. **依赖安全**
   ```bash
   # 定期检查依赖漏洞
   npm audit
   npm audit fix
   ```

---

## 10. 快速参考

### 10.1 常用命令

```bash
# 开发
npm run dev

# 构建
npm run build

# 生产运行
npm run start

# 类型检查
npx tsc --noEmit

# 代码检查
npm run lint
```

### 10.2 配置文件位置

- 环境变量: `.env.local`
- Next.js 配置: `next.config.ts`
- API 客户端: `src/lib/api-client.ts`
- 常量配置: `src/lib/constants.ts`
- TypeScript 配置: `tsconfig.json`

### 10.3 关键端点

- 前端: `http://localhost:3001`
- 后端 API: `http://localhost:8080`
- 健康检查: `http://localhost:8080/health`
- 系统指标: `http://localhost:8080/metrics`

---

## 附录：完整配置示例

### A. 开发环境配置 (`.env.local`)

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080
NODE_ENV=development
PORT=3001
```

### B. 生产环境配置 (`.env.production`)

```bash
NEXT_PUBLIC_API_URL=https://api.yourdomain.com
NEXT_PUBLIC_API_KEY=your-production-api-key
NODE_ENV=production
PORT=3000
```

### C. Docker Compose 配置

```yaml
version: '3.8'

services:
  frontend:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8080
      - NODE_ENV=production
    depends_on:
      - backend
    restart: unless-stopped

  backend:
    image: agentmem-backend:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
```

---

**文档维护**: 本文档应随着项目更新而更新。如有问题，请提交 Issue。

**最后更新**: 2025-01-20

