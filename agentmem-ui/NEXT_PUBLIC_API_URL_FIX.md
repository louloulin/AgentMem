# Next.js 打包后后端 URL 配置未生效问题修复

## 问题分析

### 根本原因

**Next.js 环境变量的关键特性**：
- `NEXT_PUBLIC_*` 环境变量在**构建时**（build time）被嵌入到客户端代码中
- 如果构建时没有设置环境变量，打包后的代码会使用默认值 `http://localhost:8080`
- **运行时设置环境变量不会生效**，因为代码已经在构建时编译进去了

### 当前问题

1. **环境变量名称不一致**：
   - `api-client.ts` 使用 `NEXT_PUBLIC_API_URL`
   - 其他文件使用 `NEXT_PUBLIC_API_BASE_URL`

2. **构建脚本未设置环境变量**：
   - `build-release.sh` 在构建时没有设置 `NEXT_PUBLIC_API_URL`
   - 导致打包后的代码使用默认值

3. **启动脚本设置的环境变量无效**：
   - `start.sh` 在运行时设置 `NEXT_PUBLIC_API_URL`
   - 但这对已打包的代码无效

## 解决方案

### 方案 1: 在构建时设置环境变量（推荐）

修改 `build-release.sh`，在构建时设置环境变量：

```bash
# 构建前端
build_ui() {
    cd "$UI_DIR"
    
    # ✅ 在构建时设置环境变量
    export NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:8080}
    
    # 构建 Next.js 应用
    NODE_ENV=production NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL npm run build
}
```

### 方案 2: 使用 .env.production 文件

在 `agentmem-ui/` 目录创建 `.env.production` 文件：

```bash
NEXT_PUBLIC_API_URL=http://your-backend-url:8080
```

然后在构建时，Next.js 会自动读取这个文件。

### 方案 3: 统一环境变量名称并修复代码

1. **统一使用 `NEXT_PUBLIC_API_URL`**

2. **修复所有文件中的环境变量引用**

## 实施步骤

### 步骤 1: 统一环境变量名称

将所有 `NEXT_PUBLIC_API_BASE_URL` 改为 `NEXT_PUBLIC_API_URL`

### 步骤 2: 修改构建脚本

在构建时设置正确的环境变量

### 步骤 3: 创建环境变量配置文件

为不同环境创建配置文件

## 验证方法

### 验证构建时的环境变量

```bash
cd agentmem-ui
NEXT_PUBLIC_API_URL=http://test-backend:8080 npm run build

# 检查构建产物中的环境变量
grep -r "test-backend" .next/static/chunks/ || echo "未找到，说明环境变量未正确嵌入"
```

### 验证运行时的配置

```bash
# 启动服务
cd dist/ui
NEXT_PUBLIC_API_URL=http://runtime-backend:8080 ./start.sh

# 检查浏览器控制台
# 打开开发者工具，查看 Network 请求的 URL
```

## 最佳实践

1. **构建时配置**：在构建时设置所有 `NEXT_PUBLIC_*` 环境变量
2. **环境分离**：为不同环境创建不同的配置文件
3. **统一命名**：统一使用 `NEXT_PUBLIC_API_URL`
4. **文档记录**：在 README 中说明环境变量配置方法

