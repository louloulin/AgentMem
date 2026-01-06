# Docker 镜像导出指南

## 📋 概述

本指南说明如何将 Docker 镜像导出为 tar 包，用于离线部署、备份或在不同环境间传输。

---

## 🚀 快速使用

### 导出 godlinchong/agentmem:latest

```bash
# 基本导出
./export-docker-image.sh

# 导出并压缩（推荐，文件更小）
./export-docker-image.sh --compress

# 导出指定镜像
./export-docker-image.sh --image godlinchong/agentmem:v1.0.0

# 导出到指定路径
./export-docker-image.sh --output /path/to/agentmem.tar
```

---

## 📝 详细说明

### 脚本功能

`export-docker-image.sh` 脚本提供以下功能：

1. **自动检查镜像**: 检查本地是否存在镜像，不存在则自动拉取
2. **导出镜像**: 将镜像导出为 tar 包
3. **压缩选项**: 支持压缩为 tar.gz（文件更小）
4. **验证导出**: 自动验证导出文件完整性

### 命令行选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--image IMAGE` | 镜像名称 | `godlinchong/agentmem:latest` |
| `--output FILE` | 输出文件路径 | `agentmem-latest.tar` |
| `--compress` | 压缩 tar 包 | `false` |
| `--pull` | 强制拉取最新镜像 | `false` |
| `--help` | 显示帮助信息 | - |

---

## 🔧 使用示例

### 示例 1: 基本导出

```bash
./export-docker-image.sh
```

**输出**:
- 文件: `agentmem-latest.tar`
- 位置: 项目根目录

### 示例 2: 导出并压缩

```bash
./export-docker-image.sh --compress
```

**输出**:
- 文件: `agentmem-latest.tar.gz`
- 位置: 项目根目录
- 优势: 文件更小，适合传输

### 示例 3: 导出指定版本

```bash
./export-docker-image.sh \
  --image godlinchong/agentmem:v1.0.0 \
  --output dist/docker/agentmem-v1.0.0.tar \
  --compress
```

**输出**:
- 文件: `dist/docker/agentmem-v1.0.0.tar.gz`
- 位置: `dist/docker/` 目录

### 示例 4: 强制拉取最新版本

```bash
./export-docker-image.sh --pull --compress
```

**说明**:
- 即使本地已有镜像，也会从 Docker Hub 拉取最新版本
- 然后导出并压缩

---

## 📦 导入镜像

### 方式 1: 导入 tar 包

```bash
# 未压缩的 tar 包
docker load -i agentmem-latest.tar

# 压缩的 tar.gz 包
gunzip -c agentmem-latest.tar.gz | docker load
```

### 方式 2: 使用脚本导入

创建 `import-docker-image.sh`:

```bash
#!/bin/bash
IMAGE_FILE="${1:-agentmem-latest.tar}"

if [[ "$IMAGE_FILE" =~ \.tar\.gz$ ]]; then
    echo "导入压缩镜像: $IMAGE_FILE"
    gunzip -c "$IMAGE_FILE" | docker load
else
    echo "导入镜像: $IMAGE_FILE"
    docker load -i "$IMAGE_FILE"
fi

echo "✅ 导入完成"
docker images | grep agentmem
```

**使用**:
```bash
chmod +x import-docker-image.sh
./import-docker-image.sh agentmem-latest.tar
./import-docker-image.sh agentmem-latest.tar.gz
```

---

## 🔍 验证导出文件

### 检查文件大小

```bash
ls -lh agentmem-latest.tar*
```

### 验证 tar 文件

```bash
# 未压缩
tar -tf agentmem-latest.tar | head -10

# 压缩
gunzip -c agentmem-latest.tar.gz | tar -t | head -10
```

### 查看镜像信息

```bash
# 导入后查看
docker images | grep agentmem

# 查看详细信息
docker image inspect godlinchong/agentmem:latest
```

---

## 📊 文件大小对比

### 典型大小

| 格式 | 大小 | 说明 |
|------|------|------|
| `.tar` | ~500MB - 1GB | 未压缩 |
| `.tar.gz` | ~200MB - 400MB | 压缩后（推荐） |

**建议**: 使用 `--compress` 选项，文件更小，传输更快。

---

## 🚀 完整工作流程

### 流程 1: 导出 → 传输 → 导入

```bash
# 1. 在源机器导出
./export-docker-image.sh --compress

# 2. 传输文件（使用 scp, rsync, 或其他方式）
scp agentmem-latest.tar.gz user@target-server:/path/to/

# 3. 在目标机器导入
gunzip -c agentmem-latest.tar.gz | docker load
```

### 流程 2: 批量导出多个版本

```bash
#!/bin/bash
# 导出多个版本

versions=("latest" "v1.0.0" "v1.0.1")

for version in "${versions[@]}"; do
    echo "导出版本: $version"
    ./export-docker-image.sh \
      --image "godlinchong/agentmem:$version" \
      --output "dist/docker/agentmem-$version.tar.gz" \
      --compress
done
```

---

## ⚠️ 注意事项

### 1. 磁盘空间

- 确保有足够的磁盘空间
- tar 包大小通常接近镜像大小
- 压缩后约为原大小的 40-60%

### 2. 网络连接

- 如果镜像不存在，需要网络连接拉取
- 使用 `--pull` 强制拉取最新版本

### 3. 文件权限

- 确保有写入输出目录的权限
- 导出文件会继承当前用户的权限

### 4. 导入环境

- 目标机器需要安装 Docker
- 确保有足够的磁盘空间导入镜像

---

## 🔧 故障排查

### 问题 1: 镜像不存在

**错误**: `Error response from daemon: manifest for godlinchong/agentmem:latest not found`

**解决**:
```bash
# 先拉取镜像
docker pull godlinchong/agentmem:latest

# 或使用脚本自动拉取
./export-docker-image.sh --pull
```

### 问题 2: 磁盘空间不足

**错误**: `No space left on device`

**解决**:
- 清理 Docker 未使用的资源: `docker system prune -a`
- 使用压缩选项: `--compress`
- 导出到其他有空间的目录

### 问题 3: 权限不足

**错误**: `Permission denied`

**解决**:
```bash
# 使用 sudo（不推荐）
sudo ./export-docker-image.sh

# 或修复权限
sudo chmod +x export-docker-image.sh
```

---

## 📝 相关命令

### 直接使用 docker 命令

```bash
# 导出镜像
docker save godlinchong/agentmem:latest -o agentmem-latest.tar

# 导出并压缩
docker save godlinchong/agentmem:latest | gzip > agentmem-latest.tar.gz

# 导入镜像
docker load -i agentmem-latest.tar

# 导入压缩镜像
gunzip -c agentmem-latest.tar.gz | docker load
```

### 查看镜像列表

```bash
# 查看本地镜像
docker images | grep agentmem

# 查看镜像详细信息
docker image inspect godlinchong/agentmem:latest
```

---

## ✅ 总结

**导出脚本**: `export-docker-image.sh`  
**默认镜像**: `godlinchong/agentmem:latest`  
**默认输出**: `agentmem-latest.tar`

**推荐使用**:
```bash
./export-docker-image.sh --compress
```

**导入镜像**:
```bash
gunzip -c agentmem-latest.tar.gz | docker load
```

---

**最后更新**: 2025-12-02  
**状态**: ✅ 配置完成

