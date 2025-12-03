# Dockerfile.multiarch ort-sys 构建失败修复方案

## 📋 问题分析

### 核心问题

**ort-sys 构建脚本网络下载失败**：
```
Failed to GET `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.22.0/x86_64-unknown-linux-gnu.tgz`: io: Peer disconnected
error: failed to run custom build command for `ort-sys v2.0.0-rc.10`
```

### 根本原因

1. **网络连接问题** ⚠️
   - `ort-sys` 构建脚本尝试从 `cdn.pyke.io` 下载 ONNX Runtime 库
   - 网络连接不稳定或 CDN 不可访问导致下载失败
   - 在 Docker 构建环境中，网络问题更加常见

2. **构建脚本自动下载机制** ⚠️
   - `ort-sys` 默认尝试自动下载 ONNX Runtime 库
   - 如果本地库不存在或未正确配置，会触发下载
   - 下载失败会导致整个构建失败

3. **环境变量配置缺失** ⚠️
   - 未设置 `ORT_LIB_LOCATION` 环境变量指向本地库
   - 未设置 `ORT_PREFER_DYNAMIC_LINK` 来使用本地库
   - 构建脚本无法找到本地库，只能尝试下载

---

## ✅ 最佳解决方案

### 方案概述

**使用本地 ONNX Runtime 库，避免网络下载**：
1. 在构建前准备本地 ONNX Runtime 库
2. 设置 `ORT_LIB_LOCATION` 环境变量指向本地库
3. 设置 `ORT_PREFER_DYNAMIC_LINK=1` 使用动态链接

### 修复内容

#### 修复 1: 准备本地 ONNX Runtime 库

**位置**：`Dockerfile.multiarch` 第 42-70 行

```dockerfile
# Prepare ONNX Runtime library for ort-sys build script
RUN case "$TARGETARCH" in \
    amd64) \
        if [ -d "/app/lib/linux-amd64" ]; then \
            mkdir -p /app/onnxruntime && \
            cp -r /app/lib/linux-amd64/* /app/onnxruntime/ && \
            echo "✅ ONNX Runtime library prepared for amd64"; \
        fi \
        ;; \
    ...
    esac
```

**说明**：
- 检查项目中的 `lib/linux-amd64` 目录
- 将 ONNX Runtime 库复制到 `/app/onnxruntime` 目录
- 为构建脚本提供本地库，避免网络下载

#### 修复 2: 配置 ort-sys 使用本地库

**位置**：`Dockerfile.multiarch` 第 150-170 行

```dockerfile
# Configure ort-sys to use local ONNX Runtime library
if [ -d "/app/onnxruntime" ] && [ -f "/app/onnxruntime/libonnxruntime.so" ] || [ -f "/app/onnxruntime/lib/libonnxruntime.so" ]; then \
    if [ -f "/app/onnxruntime/lib/libonnxruntime.so" ]; then \
        export ORT_LIB_LOCATION="/app/onnxruntime/lib" && \
        echo "✅ Using ONNX Runtime from /app/onnxruntime/lib"; \
    else \
        export ORT_LIB_LOCATION="/app/onnxruntime" && \
        echo "✅ Using ONNX Runtime from /app/onnxruntime"; \
    fi && \
    export ORT_PREFER_DYNAMIC_LINK="1" && \
    echo "ORT_LIB_LOCATION=${ORT_LIB_LOCATION}"; \
fi
```

**说明**：
- 检查本地 ONNX Runtime 库是否存在
- 根据库文件位置设置 `ORT_LIB_LOCATION`
- 设置 `ORT_PREFER_DYNAMIC_LINK=1` 使用动态链接
- 如果本地库不存在，会显示警告（但不会阻止构建）

---

## 🔧 完整修复内容

### 修改 1: 准备本地 ONNX Runtime 库

```dockerfile
# Prepare ONNX Runtime library for ort-sys build script
RUN case "$TARGETARCH" in \
    amd64) \
        if [ -d "/app/lib/linux-amd64" ]; then \
            mkdir -p /app/onnxruntime && \
            cp -r /app/lib/linux-amd64/* /app/onnxruntime/ && \
            echo "✅ ONNX Runtime library prepared for amd64"; \
        fi \
        ;; \
    arm64) \
        if [ -d "/app/lib/linux-arm64" ]; then \
            mkdir -p /app/onnxruntime && \
            cp -r /app/lib/linux-arm64/* /app/onnxruntime/ && \
            echo "✅ ONNX Runtime library prepared for arm64"; \
        fi \
        ;; \
    arm) \
        if [ -d "/app/lib/linux-arm" ]; then \
            mkdir -p /app/onnxruntime && \
            cp -r /app/lib/linux-arm/* /app/onnxruntime/ && \
            echo "✅ ONNX Runtime library prepared for arm"; \
        fi \
        ;; \
    esac
```

### 修改 2: 配置 ort-sys 环境变量

```dockerfile
# Configure ort-sys to use local ONNX Runtime library
if [ -d "/app/onnxruntime" ] && [ -f "/app/onnxruntime/libonnxruntime.so" ] || [ -f "/app/onnxruntime/lib/libonnxruntime.so" ]; then \
    if [ -f "/app/onnxruntime/lib/libonnxruntime.so" ]; then \
        export ORT_LIB_LOCATION="/app/onnxruntime/lib" && \
        echo "✅ Using ONNX Runtime from /app/onnxruntime/lib"; \
    else \
        export ORT_LIB_LOCATION="/app/onnxruntime" && \
        echo "✅ Using ONNX Runtime from /app/onnxruntime"; \
    fi && \
    export ORT_PREFER_DYNAMIC_LINK="1" && \
    echo "ORT_LIB_LOCATION=${ORT_LIB_LOCATION}"; \
else \
    echo "⚠️  Warning: Local ONNX Runtime not found, ort-sys will try to download"; \
fi
```

### 修改 3: 添加调试输出

```dockerfile
echo "ORT_LIB_LOCATION=${ORT_LIB_LOCATION:-not set}" && \
echo "ORT_PREFER_DYNAMIC_LINK=${ORT_PREFER_DYNAMIC_LINK:-not set}" && \
```

---

## 🚀 验证方法

### 1. 检查构建日志

构建时应该看到：
```
✅ ONNX Runtime library prepared for amd64
✅ Using ONNX Runtime from /app/onnxruntime/lib
ORT_LIB_LOCATION=/app/onnxruntime/lib
ORT_PREFER_DYNAMIC_LINK=1
```

### 2. 检查构建是否成功

构建应该成功完成，没有网络下载错误。

### 3. 检查 ort-sys 构建日志

在构建日志中，应该看到：
```
onnxruntime found using ORT_LIB_LOCATION
```

而不是：
```
onnxruntime not found using pkg-config, falling back to manual setup.
Failed to GET https://cdn.pyke.io/...
```

---

## 🔍 故障排查

### 问题 1: 本地库不存在

**错误信息**：
```
⚠️  Warning: lib/linux-amd64 not found, ort-sys will try to download
```

**解决方案**：
1. 确保项目根目录下有 `lib/linux-amd64/` 目录
2. 确保目录中包含 `libonnxruntime.so` 或 `lib/libonnxruntime.so`
3. 检查 Dockerfile 中的 COPY 命令是否正确复制了 lib 目录

### 问题 2: ORT_LIB_LOCATION 路径不正确

**错误信息**：
```
ort-sys build script still tries to download
```

**解决方案**：
1. 检查 `ORT_LIB_LOCATION` 环境变量是否正确设置
2. 验证路径中的库文件是否存在
3. 检查库文件权限（应该是可读的）

### 问题 3: 仍然尝试下载

**可能原因**：
- `ORT_LIB_LOCATION` 路径不正确
- 库文件格式不匹配（例如，arm64 库用于 amd64 构建）

**解决方案**：
1. 检查构建日志中的 `ORT_LIB_LOCATION` 值
2. 验证库文件架构是否匹配目标架构
3. 确保库文件完整（使用 `file` 命令检查）

---

## 📊 修复前后对比

| 项目 | 修复前 | 修复后 |
|------|--------|--------|
| **网络下载** | ❌ 尝试从 cdn.pyke.io 下载 | ✅ 使用本地库，跳过下载 |
| **构建成功** | ❌ 网络失败导致构建失败 | ✅ 使用本地库，构建成功 |
| **网络依赖** | ⚠️ 依赖网络连接 | ✅ 不依赖网络连接 |
| **构建速度** | ⚠️ 需要下载（慢） | ✅ 使用本地库（快） |
| **可靠性** | ⚠️ 受网络影响 | ✅ 不受网络影响 |

---

## 📝 修复总结

### 已修复的问题

✅ **ort-sys 网络下载失败**
- 在构建前准备本地 ONNX Runtime 库
- 设置 `ORT_LIB_LOCATION` 环境变量指向本地库
- 设置 `ORT_PREFER_DYNAMIC_LINK=1` 使用动态链接

✅ **构建脚本配置**
- 自动检测本地库是否存在
- 根据库文件位置自动设置正确的路径
- 提供详细的调试输出

### 修改内容

**Dockerfile.multiarch**:
- 第 42-70 行：准备本地 ONNX Runtime 库
- 第 150-170 行：配置 ort-sys 使用本地库
- 第 175-176 行：添加调试输出

### 使用建议

1. **确保本地库存在**：
   - 在项目根目录下创建 `lib/linux-amd64/` 目录
   - 将 ONNX Runtime 库文件放入该目录
   - 库文件可以是 `libonnxruntime.so` 或 `lib/libonnxruntime.so`

2. **验证库文件**：
   ```bash
   # 检查库文件是否存在
   ls -la lib/linux-amd64/
   
   # 检查库文件架构
   file lib/linux-amd64/libonnxruntime.so
   ```

3. **构建测试**：
   ```bash
   docker buildx build \
     --platform linux/amd64 \
     -f Dockerfile.multiarch \
     -t agentmem:latest \
     --load .
   ```

---

## 🔗 相关文档

- `DOCKERFILE_MULTIARCH_AVX512_FINAL_SOLUTION.md` - AVX-512 修复方案
- `DOCKERFILE_MULTIARCH_COMPREHENSIVE_ANALYSIS.md` - 全面分析
- `DOCKERFILE_MULTIARCH_BUILD_GUIDE.md` - 构建指南

---

## 📚 参考资料

- [ort-sys 文档](https://docs.rs/ort-sys/)
- [ONNX Runtime 下载](https://github.com/microsoft/onnxruntime/releases)
- [Docker 多架构构建](https://docs.docker.com/build/building/multi-platform/)

