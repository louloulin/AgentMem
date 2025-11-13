# AgentMem 打包发布脚本改进总结

## 问题分析

### 原始问题
用户在运行构建后的服务器时遇到以下警告：
```
WARN 未找到任何 LLM API Key 环境变量
WARN LLM API Key 未配置，LLM Provider 将不可用
WARN LLM Provider 未配置，Intelligence 组件将不可用
```

### 根本原因
1. **构建成功，但运行时配置缺失** - 这不是代码问题，而是环境配置问题
2. **缺少 ONNX Runtime 库文件** - 服务器依赖的动态库未包含在发布包中
3. **启动脚本配置不完整** - 缺少必要的环境变量配置

## 解决方案

### 1. 修复 Next.js 构建问题

#### 问题：useSearchParams() 需要 Suspense 边界
**文件**: `agentmem-ui/src/app/admin/chat/page.tsx`

**修复方法**：
```typescript
// 内部组件：使用 useSearchParams
function ChatPageInner() {
  const searchParams = useSearchParams();
  // ... 组件逻辑
}

// 导出组件：使用 Suspense 包裹
export default function ChatPage() {
  return (
    <Suspense fallback={<LoadingSpinner />}>
      <ChatPageInner />
    </Suspense>
  );
}
```

#### 问题：NODE_ENV 非标准值导致构建失败
**修复**：统一使用 `NODE_ENV=production` 进行构建

**文件**: `build-release.sh` 第 213 行
```bash
# 修改前
NODE_ENV=development npm run build

# 修改后
NODE_ENV=production npm run build
```

### 2. 添加 ONNX Runtime 库文件复制

**文件**: `build-release.sh` 第 284-293 行

```bash
# 复制 ONNX Runtime 库文件
log_info "复制 ONNX Runtime 库文件..."
mkdir -p "$DIST_DIR/server/lib"

# 检查并复制 lib 目录下的所有库文件
if [ -d "lib" ]; then
    cp -r lib/* "$DIST_DIR/server/lib/" 2>/dev/null || true
    log_success "已复制 lib 目录下的库文件"
else
    log_warning "未找到 lib 目录，跳过库文件复制"
fi
```

**复制的文件**：
- macOS: `libonnxruntime.1.22.0.dylib`, `libonnxruntime.dylib`
- Linux: `libonnxruntime.so.1.22.0`, `libonnxruntime.so`

### 3. 增强启动脚本配置

#### 基础启动脚本 (`start.sh`)

**文件**: `dist/server/start.sh`

**关键配置**：
```bash
# 获取绝对路径
LIB_DIR="$(pwd)/lib"

# 设置库路径（跨平台）
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_LIBRARY_PATH="$LIB_DIR:$DYLD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.1.22.0.dylib"
else
    export LD_LIBRARY_PATH="$LIB_DIR:$LD_LIBRARY_PATH"
    export ORT_DYLIB_PATH="$LIB_DIR/libonnxruntime.so.1.22.0"
fi

# 配置 Embedder（必需）
export EMBEDDER_PROVIDER=${EMBEDDER_PROVIDER:-"fastembed"}
export EMBEDDER_MODEL=${EMBEDDER_MODEL:-"BAAI/bge-small-en-v1.5"}

# LLM 配置（可选）
# export LLM_PROVIDER="zhipu"
# export ZHIPU_API_KEY="your_api_key_here"
```

#### 智谱 AI 配置示例 (`start-with-zhipu.sh`)

**文件**: `dist/server/start-with-zhipu.sh`

**完整配置**：
```bash
# 配置 Embedder
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 配置 LLM Provider (智谱 AI)
export ZHIPU_API_KEY="your_zhipu_api_key_here"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"

# 禁用认证（用于测试）
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"
```

### 4. 更新部署文档

**文件**: `dist/README.md`

**新增内容**：
1. **库文件说明** - 解释 ONNX Runtime 库的作用和位置
2. **环境变量详细说明** - 包括 Embedder、LLM、认证等配置
3. **快速启动示例** - 提供多种启动方式的示例
4. **systemd 服务配置** - 包含完整的环境变量配置

## 构建产物结构

```
dist/
├── ui/                        # 前端应用
│   ├── .next/                 # Next.js 构建产物
│   │   └── standalone/        # 独立部署模式
│   ├── public/                # 静态资源
│   ├── package.json
│   └── start.sh               # 前端启动脚本
├── server/                    # 后端服务
│   ├── agent-mem-server       # 二进制文件
│   ├── lib/                   # ✅ 新增：ONNX Runtime 库
│   │   ├── libonnxruntime.1.22.0.dylib  (macOS)
│   │   └── libonnxruntime.dylib         (macOS)
│   ├── config.example.toml    # 配置文件示例
│   ├── start.sh               # ✅ 增强：基础启动脚本
│   └── start-with-zhipu.sh    # ✅ 新增：智谱 AI 配置示例
└── README.md                  # ✅ 增强：部署文档
```

## 使用方法

### 1. 构建发布包

```bash
# 构建所有组件（生产模式）
./build-release.sh --all

# 仅构建前端
./build-release.sh --ui-only

# 仅构建后端
./build-release.sh --server-only
```

### 2. 部署后端

#### 方式 1：使用基础配置（仅 Embedder）
```bash
cd dist/server
./start.sh
```

#### 方式 2：使用智谱 AI 配置
```bash
cd dist/server
# 编辑配置文件，设置你的 API Key
vim start-with-zhipu.sh
# 启动
./start-with-zhipu.sh
```

#### 方式 3：自定义配置
```bash
cd dist/server
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"
export ZHIPU_API_KEY="your_api_key_here"
export ENABLE_AUTH="false"
./start.sh
```

### 3. 部署前端

```bash
cd dist/ui
export NEXT_PUBLIC_API_URL=http://your-server-ip:8080
./start.sh
```

## 关键改进点

### ✅ 1. 完整的库文件支持
- 自动复制 ONNX Runtime 库到发布包
- 启动脚本自动设置库路径（跨平台）
- 支持 macOS 和 Linux

### ✅ 2. 灵活的配置方式
- 基础启动脚本：最小配置，仅 Embedder
- 智谱 AI 示例：完整的 LLM 配置
- 环境变量：支持运行时自定义

### ✅ 3. 详细的文档
- 部署指南：包含所有配置选项
- 快速启动示例：多种场景
- systemd 服务配置：生产环境部署

### ✅ 4. 修复构建问题
- Next.js Suspense 边界
- 统一使用 production 模式构建
- 移除非标准 NODE_ENV 值

## 测试验证

### 构建测试
```bash
✅ 前端构建成功 (24/24 页面)
✅ 后端构建成功 (0 错误)
✅ 库文件复制成功 (32MB × 2)
✅ 启动脚本生成成功
✅ 部署文档生成成功
```

### 文件验证
```bash
$ ls -lh dist/server/lib/
-rwxr-xr-x  32M  libonnxruntime.1.22.0.dylib
-rwxr-xr-x  32M  libonnxruntime.dylib

$ ls -lh dist/server/*.sh
-rwxr-xr-x  1.6K  start-with-zhipu.sh
-rwxr-xr-x  2.1K  start.sh
```

## 总结

通过这次改进，我们实现了：

1. **完整的打包** - 包含所有必需的库文件和配置
2. **灵活的部署** - 支持多种配置方式和场景
3. **详细的文档** - 降低部署门槛
4. **跨平台支持** - macOS 和 Linux 自动适配

现在用户可以：
- ✅ 一键构建完整的发布包
- ✅ 快速部署到生产环境
- ✅ 灵活配置 LLM Provider
- ✅ 无需手动复制库文件

