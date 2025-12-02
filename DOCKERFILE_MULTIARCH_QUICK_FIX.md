# Dockerfile.multiarch 快速修复指南

## ✅ 已修复的问题

1. ✅ **内存不足** - 添加了并行任务限制（默认 2 个任务）
2. ✅ **安全警告** - 移除了 `ENV ZHIPU_API_KEY=""`
3. ✅ **构建优化** - 添加了内存优化选项

---

## 🚀 立即使用

### 1. 增加 Docker Desktop 内存（必须）

**macOS/Windows**:
1. 打开 Docker Desktop
2. Settings → Resources → Advanced
3. 增加 Memory 到至少 **8GB**（推荐 16GB）
4. Apply & Restart

### 2. 构建命令

#### 基础构建（推荐，内存受限环境）

```bash
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --load .
```

#### 高性能构建（内存充足，16GB+）

```bash
docker buildx build \
  --platform linux/amd64 \
  --build-arg CARGO_BUILD_JOBS=4 \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --load .
```

#### 多架构构建

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg CARGO_BUILD_JOBS=2 \
  -f Dockerfile.multiarch \
  -t godlinchong/agentmem:latest \
  --push .
```

---

## ⚠️ 重要提示

1. **必须添加 `--load` 或 `--push`**，否则构建结果只存在于缓存中
2. **增加 Docker Desktop 内存**是最重要的步骤
3. **如果仍然内存不足**，使用 `--build-arg CARGO_BUILD_JOBS=1`

---

## 🔧 运行时配置 API Key

```bash
# 方式 1: 环境变量
docker run -d -p 8080:8080 \
  -e ZHIPU_API_KEY=your_key_here \
  agentmem:latest

# 方式 2: 配置文件
docker run -d -p 8080:8080 \
  -v $(pwd)/config:/app/config:ro \
  agentmem:latest
```

---

## 📚 详细文档

- `DOCKERFILE_MULTIARCH_MEMORY_FIX.md` - 详细修复说明
- `DOCKERFILE_MULTIARCH_BUILD_GUIDE.md` - 完整构建指南
- `DOCKERFILE_MULTIARCH_COMPREHENSIVE_ANALYSIS.md` - 全面分析

---

**修复完成时间**: 2025-01-02

