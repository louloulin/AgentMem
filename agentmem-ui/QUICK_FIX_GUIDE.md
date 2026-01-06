# Next.js 后端 URL 配置快速修复指南

## 🚨 问题

打包后的 Next.js 应用，后端 URL 配置没有生效，始终使用默认值 `http://localhost:8080`。

## 💡 根本原因

**Next.js 的 `NEXT_PUBLIC_*` 环境变量在构建时（build time）嵌入到代码中，运行时设置无效！**

```
❌ 错误理解: 运行时设置环境变量 → 代码读取
✅ 实际情况: 构建时环境变量 → 嵌入代码 → 运行时无法改变
```

## ✅ 解决方案（3 种方法）

### 方法 1: 构建时设置环境变量（推荐，已修复）

```bash
# 1. 设置环境变量
export NEXT_PUBLIC_API_URL=http://your-backend:8080

# 2. 构建
./build-release.sh

# 或直接
NEXT_PUBLIC_API_URL=http://your-backend:8080 ./build-release.sh
```

**已修复**: `build-release.sh` 现在会在构建时设置环境变量。

### 方法 2: 使用 .env.production 文件

```bash
# 1. 创建 .env.production
cd agentmem-ui
echo "NEXT_PUBLIC_API_URL=http://your-backend:8080" > .env.production

# 2. 构建（Next.js 会自动读取）
npm run build
```

### 方法 3: 在 next.config.ts 中设置（不推荐）

```typescript
// next.config.ts
const nextConfig = {
  env: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080',
  },
};
```

## 🔍 验证方法

### 1. 检查构建脚本

```bash
grep -A 3 "构建时 API URL" build-release.sh
# 应该看到: export NEXT_PUBLIC_API_URL=...
```

### 2. 检查构建产物

```bash
cd dist/ui/.next/standalone
grep -r "your-backend" . | head -3
# 如果找到，说明配置已嵌入
```

### 3. 浏览器检查

1. 打开应用
2. 按 F12 打开开发者工具
3. Network 标签 → 查看 API 请求的 URL
4. 应该看到正确的后端 URL，而不是 `localhost:8080`

## ⚠️ 常见错误

### ❌ 错误 1: 在启动脚本中设置

```bash
# dist/ui/start.sh
export NEXT_PUBLIC_API_URL=http://your-backend:8080  # ❌ 无效！
```

**为什么无效**: 代码在构建时已经包含了环境变量的值，运行时设置无法改变。

### ❌ 错误 2: 构建后修改 .env 文件

```bash
npm run build
# 然后修改 .env.production
# ❌ 无效！需要重新构建
```

### ❌ 错误 3: Docker 运行时设置

```yaml
# docker-compose.yml
services:
  frontend:
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8080  # ❌ 运行时设置无效
```

**正确做法**: 在 Dockerfile 构建时设置：

```dockerfile
ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
RUN npm run build
```

## 📋 检查清单

- [ ] 构建时设置了 `NEXT_PUBLIC_API_URL`
- [ ] 构建脚本已更新（已修复）
- [ ] 重新构建了应用
- [ ] 验证了构建产物中的 URL
- [ ] 浏览器中检查了 API 请求 URL

## 🚀 快速修复步骤

```bash
# 1. 设置正确的后端 URL
export NEXT_PUBLIC_API_URL=http://your-backend:8080

# 2. 重新构建
cd /path/to/agentmen
./build-release.sh

# 3. 验证
cd dist/ui
grep -r "your-backend" .next/standalone | head -1

# 4. 启动测试
./start.sh
```

## 📚 详细文档

- 完整分析: `API_CONFIGURATION_ANALYSIS.md`
- 解决方案: `NEXTJS_RUNTIME_CONFIG_SOLUTION.md`
- Next.js 官方文档: https://nextjs.org/docs/basic-features/environment-variables

---

**记住**: Next.js 环境变量 = 构建时嵌入，不是运行时读取！

