# AgentMem 前端部署指南

**版本**: 1.0  
**日期**: 2025-10-01  
**适用于**: AgentMem v2.1

---

## 📋 目录

1. [环境准备](#1-环境准备)
2. [本地开发部署](#2-本地开发部署)
3. [生产环境部署](#3-生产环境部署)
4. [Docker 部署](#4-docker-部署)
5. [环境变量配置](#5-环境变量配置)
6. [故障排除](#6-故障排除)

---

## 1. 环境准备

### 1.1 系统要求

**操作系统**:
- macOS 10.15+
- Ubuntu 20.04+
- Windows 10+ (WSL2 推荐)

**硬件要求**:
- CPU: 2 核心+
- RAM: 4 GB+
- 磁盘: 2 GB+ 可用空间

---

### 1.2 安装 Node.js

#### macOS (使用 Homebrew)

```bash
# 安装 Homebrew (如果未安装)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装 Node.js
brew install node

# 验证安装
node --version  # 应显示 v18.x.x 或更高
npm --version   # 应显示 9.x.x 或更高
```

#### Ubuntu/Debian

```bash
# 使用 NodeSource 仓库
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 验证安装
node --version
npm --version
```

#### Windows

1. 访问 https://nodejs.org/
2. 下载 LTS 版本安装包
3. 运行安装程序
4. 在命令提示符中验证: `node --version`

#### 使用 nvm (推荐)

```bash
# 安装 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# 重新加载 shell 配置
source ~/.bashrc  # 或 ~/.zshrc

# 安装 Node.js
nvm install 18
nvm use 18

# 验证
node --version
```

---

### 1.3 安装包管理器（可选）

#### pnpm (推荐，更快更节省空间)

```bash
npm install -g pnpm

# 验证
pnpm --version
```

#### yarn

```bash
npm install -g yarn

# 验证
yarn --version
```

---

## 2. 本地开发部署

### 2.1 克隆项目

```bash
# 克隆主仓库
git clone <repository-url>
cd contextengine

# 初始化子模块
git submodule update --init --recursive

# 进入前端目录
cd agentmen/agentmem-website
```

---

### 2.2 安装依赖

```bash
# 使用 npm
npm install

# 或使用 pnpm (推荐)
pnpm install

# 或使用 yarn
yarn install
```

**预期输出**:
```
added 1234 packages in 45s
```

**常见问题**:
- 如果遇到权限错误，不要使用 `sudo`
- 如果遇到网络问题，配置 npm 镜像:
  ```bash
  npm config set registry https://registry.npmmirror.com
  ```

---

### 2.3 配置环境变量

创建 `.env.local` 文件:

```bash
# 复制示例配置
cp .env.example .env.local

# 编辑配置
nano .env.local
```

`.env.local` 内容:

```env
# API 配置
NEXT_PUBLIC_API_URL=http://localhost:8080

# 可选：API 认证
NEXT_PUBLIC_API_KEY=your-api-key-here

# 可选：分析工具
NEXT_PUBLIC_GA_ID=G-XXXXXXXXXX
```

---

### 2.4 启动开发服务器

```bash
npm run dev
```

**预期输出**:
```
  ▲ Next.js 15.5.2
  - Local:        http://localhost:3000
  - Network:      http://192.168.1.100:3000

 ✓ Ready in 2.5s
```

**访问应用**:
- 打开浏览器访问: http://localhost:3000
- 管理界面: http://localhost:3000/admin

**开发服务器特性**:
- ✅ 热重载 (Hot Reload)
- ✅ 快速刷新 (Fast Refresh)
- ✅ 错误覆盖 (Error Overlay)
- ✅ TypeScript 类型检查

---

### 2.5 验证开发环境

```bash
# 在新终端中运行
curl http://localhost:3000

# 应返回 HTML 内容
```

---

## 3. 生产环境部署

### 3.1 构建生产版本

```bash
# 构建
npm run build

# 预期输出
Route (app)                              Size     First Load JS
┌ ○ /                                    5.2 kB         95 kB
├ ○ /admin                               2.1 kB         92 kB
├ ○ /admin/agents                        3.5 kB         93 kB
├ ○ /admin/chat                          4.2 kB         94 kB
├ ○ /admin/memories                      3.8 kB         94 kB
├ ○ /admin/users                         2.3 kB         92 kB
├ ○ /admin/settings                      2.5 kB         92 kB
└ ○ /about                               1.8 kB         91 kB

○  (Static)  prerendered as static content

✓ Compiled successfully
```

---

### 3.2 启动生产服务器

```bash
npm run start
```

**预期输出**:
```
  ▲ Next.js 15.5.2
  - Local:        http://localhost:3000

 ✓ Ready in 500ms
```

---

### 3.3 使用 PM2 管理进程（推荐）

#### 安装 PM2

```bash
npm install -g pm2
```

#### 创建 PM2 配置文件

创建 `ecosystem.config.js`:

```javascript
module.exports = {
  apps: [{
    name: 'agentmem-frontend',
    script: 'npm',
    args: 'start',
    cwd: '/path/to/agentmen/agentmem-website',
    instances: 2,
    exec_mode: 'cluster',
    env: {
      NODE_ENV: 'production',
      PORT: 3000
    }
  }]
};
```

#### 启动应用

```bash
# 启动
pm2 start ecosystem.config.js

# 查看状态
pm2 status

# 查看日志
pm2 logs agentmem-frontend

# 重启
pm2 restart agentmem-frontend

# 停止
pm2 stop agentmem-frontend

# 设置开机自启
pm2 startup
pm2 save
```

---

### 3.4 使用 Nginx 反向代理

#### 安装 Nginx

```bash
# Ubuntu/Debian
sudo apt-get install nginx

# macOS
brew install nginx
```

#### 配置 Nginx

创建 `/etc/nginx/sites-available/agentmem`:

```nginx
server {
    listen 80;
    server_name agentmem.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # 静态资源缓存
    location /_next/static {
        proxy_pass http://localhost:3000;
        proxy_cache_valid 200 60m;
        add_header Cache-Control "public, immutable";
    }
}
```

#### 启用配置

```bash
# 创建符号链接
sudo ln -s /etc/nginx/sites-available/agentmem /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```

---

### 3.5 配置 HTTPS (使用 Let's Encrypt)

```bash
# 安装 Certbot
sudo apt-get install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d agentmem.example.com

# 自动续期
sudo certbot renew --dry-run
```

---

## 4. Docker 部署

### 4.1 创建 Dockerfile

创建 `agentmen/agentmem-website/Dockerfile`:

```dockerfile
# 构建阶段
FROM node:18-alpine AS builder

WORKDIR /app

# 复制依赖文件
COPY package*.json ./
COPY pnpm-lock.yaml ./

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

ENV NODE_ENV production

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

ENV PORT 3000

CMD ["node", "server.js"]
```

---

### 4.2 创建 docker-compose.yml

```yaml
version: '3.8'

services:
  frontend:
    build:
      context: ./agentmen/agentmem-website
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8080
    depends_on:
      - backend
    restart: unless-stopped

  backend:
    image: agentmem-backend:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/agentmem
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
      - POSTGRES_DB=agentmem
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data:
```

---

### 4.3 构建和运行

```bash
# 构建镜像
docker-compose build

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f frontend

# 停止服务
docker-compose down
```

---

## 5. 环境变量配置

### 5.1 必需的环境变量

| 变量名 | 说明 | 示例值 |
|--------|------|--------|
| `NEXT_PUBLIC_API_URL` | 后端 API 地址 | `http://localhost:8080` |

### 5.2 可选的环境变量

| 变量名 | 说明 | 示例值 |
|--------|------|--------|
| `NEXT_PUBLIC_API_KEY` | API 认证密钥 | `your-api-key` |
| `NEXT_PUBLIC_GA_ID` | Google Analytics ID | `G-XXXXXXXXXX` |
| `PORT` | 服务器端口 | `3000` |
| `NODE_ENV` | 运行环境 | `production` |

---

## 6. 故障排除

### 6.1 端口已被占用

**错误**: `Error: listen EADDRINUSE: address already in use :::3000`

**解决方案**:
```bash
# 查找占用端口的进程
lsof -i :3000

# 杀死进程
kill -9 <PID>

# 或使用不同端口
PORT=3001 npm run dev
```

---

### 6.2 依赖安装失败

**错误**: `npm ERR! code EACCES`

**解决方案**:
```bash
# 不要使用 sudo，而是修复 npm 权限
mkdir ~/.npm-global
npm config set prefix '~/.npm-global'
echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

---

### 6.3 构建失败

**错误**: `Error: Cannot find module '@/components/ui/...'`

**解决方案**:
检查 `tsconfig.json`:
```json
{
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

---

### 6.4 API 连接失败

**错误**: `Failed to fetch`

**解决方案**:
1. 检查后端服务器是否运行
2. 检查 `.env.local` 中的 `NEXT_PUBLIC_API_URL`
3. 检查 CORS 配置
4. 检查防火墙设置

---

## 7. 性能优化建议

### 7.1 启用 Gzip 压缩

在 `next.config.ts` 中:
```typescript
const nextConfig = {
  compress: true,
};
```

### 7.2 配置 CDN

使用 Vercel、Netlify 或 Cloudflare CDN 加速静态资源。

### 7.3 启用图片优化

Next.js 自动优化图片，确保使用 `next/image` 组件。

---

## 8. 监控和日志

### 8.1 应用监控

推荐使用:
- Vercel Analytics
- Google Analytics
- Sentry (错误追踪)

### 8.2 日志管理

```bash
# PM2 日志
pm2 logs agentmem-frontend

# Docker 日志
docker-compose logs -f frontend

# Nginx 日志
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log
```

---

## 9. 备份和恢复

### 9.1 备份

```bash
# 备份配置文件
tar -czf agentmem-frontend-backup.tar.gz \
  .env.local \
  ecosystem.config.js \
  /etc/nginx/sites-available/agentmem
```

### 9.2 恢复

```bash
# 解压备份
tar -xzf agentmem-frontend-backup.tar.gz

# 重新安装依赖
npm install

# 重新构建
npm run build

# 重启服务
pm2 restart agentmem-frontend
```

---

## 10. 安全建议

1. ✅ 使用 HTTPS
2. ✅ 配置 CSP (Content Security Policy)
3. ✅ 定期更新依赖: `npm audit fix`
4. ✅ 使用环境变量存储敏感信息
5. ✅ 限制 API 访问（CORS、Rate Limiting）

---

## 附录：快速命令参考

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

# 依赖更新
npm update

# 安全审计
npm audit
```

---

**文档维护**: 本文档应随着项目更新而更新。如有问题，请提交 Issue。

