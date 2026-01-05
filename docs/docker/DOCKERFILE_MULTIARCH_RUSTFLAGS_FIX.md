# Dockerfile.multiarch RUSTFLAGS 传递问题修复

## 问题描述

在构建多架构 Docker 镜像时，遇到 AVX-512 链接错误：

```
undefined reference to `sum_4bit_dist_table_32bytes_batch_avx512'
```

错误信息显示构建时使用的 `RUSTFLAGS` 仍然是旧值（只有 `-C link-arg=-Wl,--allow-multiple-definition`），没有包含 `-C target-cpu=generic` 和 `-C target-feature=-avx512f,...` 等禁用 AVX-512 的选项。

## 根本原因

在 Dockerfile 的 `case` 语句中，每个分支的 `export RUSTFLAGS` 是独立的。虽然设置了 `RUSTFLAGS`，但在 `esac` 之后，由于 shell 的作用域问题，`RUSTFLAGS` 可能没有正确传递到 `cargo build` 命令。

## 解决方案

重构了 `RUN` 命令，确保 `RUSTFLAGS` 正确设置和传递：

1. **在 case 之前初始化基础 RUSTFLAGS**：
   ```dockerfile
   RUSTFLAGS_BASE="-C link-arg=-Wl,--allow-multiple-definition"
   ```

2. **在 case 语句中使用变量引用**：
   ```dockerfile
   export RUSTFLAGS="${RUSTFLAGS_BASE} -C target-cpu=generic -C target-feature=-avx512f,..."
   ```

3. **在 esac 之后验证并确保 RUSTFLAGS 被使用**：
   ```dockerfile
   echo "RUSTFLAGS=${RUSTFLAGS}" && \
   export CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS && \
   cargo build ...
   ```

## 修复内容

### 修改前的问题

```dockerfile
RUN TARGET_TRIPLE=$(cat /tmp/target_triple) && \
    case "$TARGETARCH" in \
        amd64) \
            export RUSTFLAGS="..." \
            ;; \
    esac && \
    export RUSTFLAGS="${RUSTFLAGS}" && \
    cargo build ...
```

问题：`RUSTFLAGS` 在 case 语句中设置后，可能由于作用域问题没有正确传递。

### 修改后的解决方案

```dockerfile
RUN TARGET_TRIPLE=$(cat /tmp/target_triple) && \
    # Initialize RUSTFLAGS with base flags
    RUSTFLAGS_BASE="-C link-arg=-Wl,--allow-multiple-definition" && \
    # Set architecture-specific environment variables and RUSTFLAGS
    case "$TARGETARCH" in \
        amd64) \
            export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc && \
            export CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ && \
            export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar && \
            export PKG_CONFIG_ALLOW_CROSS=1 && \
            export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig && \
            # Disable CPU-specific SIMD features for cross-compilation
            export RUSTFLAGS="${RUSTFLAGS_BASE} -C target-cpu=generic -C target-feature=-avx512f,-avx512cd,-avx512bw,-avx512dq,-avx512vl" \
            ;; \
        arm64) \
            export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc && \
            export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ && \
            export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar && \
            export PKG_CONFIG_ALLOW_CROSS=1 && \
            export RUSTFLAGS="${RUSTFLAGS_BASE} -C target-cpu=generic" \
            ;; \
        arm) \
            export CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc && \
            export CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++ && \
            export AR_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-ar && \
            export PKG_CONFIG_ALLOW_CROSS=1 && \
            export RUSTFLAGS="${RUSTFLAGS_BASE} -C target-cpu=generic" \
            ;; \
    esac && \
    # Verify RUSTFLAGS is set and export it
    echo "RUSTFLAGS=${RUSTFLAGS}" && \
    export CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS && \
    # Build with the configured RUSTFLAGS
    cargo build --release --workspace \
    --bin agent-mem-server \
    --target $TARGET_TRIPLE \
    --jobs $CARGO_BUILD_JOBS \
    --exclude agent-mem-python \
    --exclude demo-multimodal \
    --exclude demo-codebase-memory
```

## 关键改进

1. **使用变量引用**：通过 `${RUSTFLAGS_BASE}` 确保基础标志被正确包含
2. **添加调试输出**：`echo "RUSTFLAGS=${RUSTFLAGS}"` 用于验证 RUSTFLAGS 是否正确设置
3. **简化逻辑**：移除了冗余的 `export RUSTFLAGS="${RUSTFLAGS}"`，因为 RUSTFLAGS 已经在 case 语句中 export 了

## 验证方法

构建时应该看到类似输出：

```
RUSTFLAGS=-C link-arg=-Wl,--allow-multiple-definition -C target-cpu=generic -C target-feature=-avx512f,-avx512cd,-avx512bw,-avx512dq,-avx512vl
```

如果看到这个输出，说明 RUSTFLAGS 已正确设置。

## 相关文档

- [DOCKERFILE_MULTIARCH_AVX512_LINKING_FIX.md](./DOCKERFILE_MULTIARCH_AVX512_LINKING_FIX.md) - AVX-512 链接错误的初始修复
- [DOCKERFILE_MULTIARCH_ALL_FIXES_SUMMARY.md](./DOCKERFILE_MULTIARCH_ALL_FIXES_SUMMARY.md) - 所有修复的总结

## 注意事项

1. **清理 Docker 缓存**：如果之前构建失败，建议清理缓存后重新构建：
   ```bash
   docker buildx prune -f
   docker buildx build --no-cache ...
   ```

2. **验证构建输出**：构建时检查 `echo "RUSTFLAGS=${RUSTFLAGS}"` 的输出，确保包含所有必要的标志。

3. **多架构构建**：此修复适用于所有目标架构（amd64、arm64、arm/v7）。

