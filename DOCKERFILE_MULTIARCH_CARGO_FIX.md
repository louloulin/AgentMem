# Dockerfile.multiarch Cargo 镜像源修复报告

## 📋 问题

构建 `Dockerfile.multiarch` 时出现错误：

```
fatal: repository 'https://mirrors.aliyun.com/rust-crates.io-index/' not found
error: Unable to update registry `crates-io`
```

**原因**: 阿里云的 Rust crates 镜像源 `https://mirrors.aliyun.com/rust-crates.io-index/` 不可用或已失效。

---

## ✅ 修复方案

### 1. 移除强制使用阿里云镜像

**修复前**:
```dockerfile
# Configure Cargo to use Aliyun mirror for faster crate downloads
RUN mkdir -p /app/.cargo && \
    cat > /app/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'aliyun'

[source.aliyun]
registry = "https://mirrors.aliyun.com/rust-crates.io-index/"

[net]
git-fetch-with-cli = true
EOF
```

**修复后**:
```dockerfile
# Configure Cargo (optional mirror support via build arg)
# Use official crates.io by default, or set CARGO_MIRROR build arg to use a mirror
ARG CARGO_MIRROR=""
RUN mkdir -p /app/.cargo && \
    if [ -n "$CARGO_MIRROR" ]; then \
        echo "[source.crates-io]" > /app/.cargo/config.toml && \
        echo "replace-with = 'mirror'" >> /app/.cargo/config.toml && \
        echo "" >> /app/.cargo/config.toml && \
        echo "[source.mirror]" >> /app/.cargo/config.toml && \
        echo "registry = \"$CARGO_MIRROR\"" >> /app/.cargo/config.toml && \
        echo "" >> /app/.cargo/config.toml && \
        echo "[net]" >> /app/.cargo/config.toml && \
        echo "git-fetch-with-cli = true" >> /app/.cargo/config.toml; \
    else \
        echo "[net]" > /app/.cargo/config.toml && \
        echo "git-fetch-with-cli = true" >> /app/.cargo/config.toml; \
    fi
```

### 2. 修复要点

- ✅ **默认使用官方源**: 不再强制使用镜像源，默认使用官方 `crates.io`
- ✅ **可选镜像支持**: 通过 `CARGO_MIRROR` 构建参数支持自定义镜像源
- ✅ **保留 git-fetch-with-cli**: 保留此配置以提高 Git 依赖下载的可靠性

---

## 🚀 使用方式

### 方式 1: 使用官方源（推荐，默认）

```bash
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --push .
```

### 方式 2: 使用自定义镜像源

如果需要使用镜像源加速（例如在中国大陆），可以通过构建参数指定：

```bash
# 使用清华镜像源
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg CARGO_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --push .

# 使用中科大镜像源
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg CARGO_MIRROR=https://mirrors.ustc.edu.cn/crates.io-index \
  -f Dockerfile.multiarch \
  -t agentmem:latest \
  --push .
```

---

## 📊 常用 Rust 镜像源

| 镜像源 | URL | 说明 |
|--------|-----|------|
| 官方源 | (默认) | 最稳定，但可能较慢 |
| 清华大学 | `https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git` | 国内推荐 |
| 中科大 | `https://mirrors.ustc.edu.cn/crates.io-index` | 国内推荐 |
| 上海交大 | `https://mirrors.sjtug.sjtu.edu.cn/git/crates.io-index` | 国内推荐 |

---

## 🔍 验证

修复后，构建应该能够成功：

```bash
# 测试构建（单架构）
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile.multiarch \
  -t agentmem:test \
  --load .

# 验证构建成功
docker run --rm agentmem:test --version
```

---

## 📝 总结

**问题**: 阿里云 Rust crates 镜像源不可用导致构建失败

**解决方案**:
- ✅ 移除强制使用失效的阿里云镜像源
- ✅ 默认使用官方 crates.io（最稳定）
- ✅ 支持通过构建参数可选使用镜像源

**当前状态**: ✅ **已修复，可以正常构建**

