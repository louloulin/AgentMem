# Dockerfile.multiarch 构建快速指南

## 🚀 快速开始

### 1. 检查 Docker Buildx

```bash
# 检查 buildx 是否可用
docker buildx version

# 创建多架构 builder（如果还没有）
docker buildx create --name multiarch --use

# 查看可用的 builders
docker buildx ls
```

### 2. 单架构构建（最快）

```bash
# 构建 amd64（在 amd64 机器上）
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t agentmem:amd64 \
  --load .

# 构建 arm64（在 arm64 机器上，如 Apple Silicon）
docker buildx build \
  --platform linux/arm64 \
  -f Dockerfile.multiarch \
  -t agentmem:arm64 \
  --load .
```

### 3. 交叉编译构建

```bash
# 在 Apple Silicon Mac 上构建 amd64
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t agentmem:amd64 \
  --load .

# 在 amd64 服务器上构建 arm64
docker buildx build \
  --platform linux/arm64 \
  -f Dockerfile.multiarch \
  -t agentmem:arm64 \
  --load .
```

### 4. 多架构构建（推荐用于发布）

```bash
# 构建并推送多架构镜像
docker buildx build \
  --platform linux/amd64,linux/arm64,linux/arm/v7 \
  -f Dockerfile.multiarch \
  -t godlinchong/agentmem:latest \
  -t godlinchong/agentmem:v2.0.0 \
  --push .
```

### 5. 使用构建脚本

```bash
# 使用提供的构建脚本
./build-docker-linux-amd64.sh \
  --dockerfile Dockerfile.multiarch \
  --platform linux/amd64 \
  --tag agentmem:latest \
  --load
```

---

## 🌏 国内网络优化

### 使用国内镜像源加速构建

```bash
# 使用清华镜像源加速 Cargo 下载
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg CARGO_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --push .
```

### 常用镜像源

| 镜像源 | URL |
|--------|-----|
| 清华大学 | `https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git` |
| 中科大 | `https://mirrors.ustc.edu.cn/crates.io-index` |
| 上海交大 | `https://mirrors.sjtug.sjtu.edu.cn/git/crates.io-index` |

---

## 🔧 常见问题排查

### 问题 1: buildx 未安装

**症状**: `docker: 'buildx' is not a docker command`

**解决**:
```bash
# Docker Desktop 通常已包含 buildx
# 如果未安装，可以手动安装
mkdir -p ~/.docker/cli-plugins
curl -L https://github.com/docker/buildx/releases/latest/download/buildx-v0.12.0.linux-amd64 -o ~/.docker/cli-plugins/docker-buildx
chmod +x ~/.docker/cli-plugins/docker-buildx
```

### 问题 2: 交叉编译工具链缺失

**症状**: `failed to find tool "x86_64-linux-gnu-gcc"`

**解决**: 
- 确认 Dockerfile.multiarch 已包含交叉编译工具链安装
- 检查 TARGETARCH 是否正确传递

### 问题 3: GLIBC 版本不兼容

**症状**: `version 'GLIBC_2.39' not found`

**解决**:
- 确认运行时镜像使用 `debian:sid-slim` 或 `ubuntu:24.04`
- 检查构建的二进制是否与运行时镜像兼容

### 问题 4: 构建时间过长

**原因**: 
- 依赖下载慢（网络问题）
- 每次全量编译

**解决**:
1. 使用国内镜像源（CARGO_MIRROR）
2. 考虑实施构建缓存优化（见分析文档）

---

## 📊 构建时间参考

| 场景 | 预计时间 | 说明 |
|------|---------|------|
| 单架构（amd64，首次） | 15-30 分钟 | 需要下载依赖和编译 |
| 单架构（amd64，缓存） | 5-10 分钟 | 依赖已缓存 |
| 交叉编译（arm64→amd64） | 20-40 分钟 | 需要安装交叉编译工具链 |
| 多架构构建 | 30-60 分钟 | 并行构建多个架构 |

**注意**: 实际时间取决于：
- 网络速度（依赖下载）
- CPU 性能
- 是否使用缓存
- 是否使用镜像源

---

## ✅ 验证构建结果

### 1. 检查镜像

```bash
# 查看镜像信息
docker image inspect agentmem:latest

# 查看镜像架构
docker image inspect agentmem:latest --format '{{.Architecture}}'
```

### 2. 测试运行

```bash
# 运行容器
docker run -d -p 8080:8080 --name agentmem-test agentmem:latest

# 检查健康状态
docker ps
docker logs agentmem-test

# 测试健康检查端点
curl http://localhost:8080/health

# 清理
docker stop agentmem-test
docker rm agentmem-test
```

### 3. 验证多架构

```bash
# 查看镜像的架构清单
docker buildx imagetools inspect godlinchong/agentmem:latest
```

---

## 🎯 最佳实践

### 1. 开发环境
- 使用单架构构建（更快）
- 使用 `--load` 加载到本地
- 使用缓存加速后续构建

### 2. CI/CD 环境
- 使用多架构构建
- 使用 `--push` 推送到镜像仓库
- 使用 `--cache-from` 和 `--cache-to` 优化缓存

### 3. 生产环境
- 使用多架构构建确保兼容性
- 使用版本标签（如 v2.0.0）
- 同时推送 latest 和版本标签

---

## 📝 构建命令模板

### 开发构建

```bash
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t agentmem:dev \
  --load .
```

### 生产构建（单架构）

```bash
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t godlinchong/agentmem:v2.0.0-amd64 \
  -t godlinchong/agentmem:latest-amd64 \
  --push .
```

### 生产构建（多架构）

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f Dockerfile.multiarch \
  -t godlinchong/agentmem:v2.0.0 \
  -t godlinchong/agentmem:latest \
  --push .
```

### 带镜像源加速的构建

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg CARGO_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --push .
```

---

## 🔗 相关文档

- `DOCKERFILE_MULTIARCH_COMPREHENSIVE_ANALYSIS.md` - 全面分析文档
- `DOCKERFILE_MULTIARCH_ANALYSIS.md` - 初始分析
- `DOCKERFILE_MULTIARCH_BASE_IMAGE_FIX.md` - 基础镜像修复
- `DOCKERFILE_MULTIARCH_CROSS_COMPILER_FIX.md` - 交叉编译修复
- `build-docker-linux-amd64.sh` - 构建脚本

---

**最后更新**: 2025-01-02

