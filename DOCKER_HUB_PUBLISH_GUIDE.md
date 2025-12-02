# Docker Hub 发布指南 - godlinchong

## 📋 概述

本指南说明如何将 AgentMem Docker 镜像发布到 Docker Hub 的 `godlinchong` 用户下。

**Docker Hub 地址**: https://hub.docker.com/u/godlinchong

---

## 🚀 快速开始

### 步骤 1: 登录 Docker Hub

```bash
docker login
# 输入用户名: godlinchong
# 输入密码: (你的 Docker Hub 密码或 Access Token)
```

**或使用 Access Token** (推荐，更安全):
```bash
echo "YOUR_ACCESS_TOKEN" | docker login --username godlinchong --password-stdin
```

### 步骤 2: 构建并推送镜像

```bash
./build-docker-linux-amd64.sh \
  --tag godlinchong/agentmem:latest \
  --push
```

### 步骤 3: 验证推送

访问 https://hub.docker.com/r/godlinchong/agentmem 查看推送的镜像。

---

## 📝 详细配置

### 1. Docker Hub 镜像命名规范

Docker Hub 镜像标签格式：
```
<username>/<repository>:<tag>
```

**示例**:
- `godlinchong/agentmem:latest` - 最新版本
- `godlinchong/agentmem:v1.0.0` - 版本标签
- `godlinchong/agentmem:1.0.0` - 简化版本标签
- `godlinchong/agentmem:amd64` - 平台标签

### 2. 使用构建脚本推送

#### 推送最新版本
```bash
./build-docker-linux-amd64.sh \
  --tag godlinchong/agentmem:latest \
  --push
```

#### 推送版本标签
```bash
./build-docker-linux-amd64.sh \
  --tag godlinchong/agentmem:v1.0.0 \
  --push
```

#### 推送多个标签
```bash
# 构建一次，推送多个标签
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile \
  -t godlinchong/agentmem:latest \
  -t godlinchong/agentmem:v1.0.0 \
  --push .
```

### 3. 直接使用 docker buildx

```bash
# 登录
docker login

# 构建并推送
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile \
  -t godlinchong/agentmem:latest \
  --push .
```

---

## 🔐 认证配置

### 方式 1: 使用密码登录

```bash
docker login
# Username: godlinchong
# Password: <your-password>
```

### 方式 2: 使用 Access Token (推荐)

1. **创建 Access Token**:
   - 访问 https://hub.docker.com/settings/security
   - 点击 "New Access Token"
   - 设置权限和过期时间
   - 复制生成的 Token

2. **使用 Token 登录**:
   ```bash
   echo "YOUR_ACCESS_TOKEN" | docker login --username godlinchong --password-stdin
   ```

### 方式 3: 使用环境变量

```bash
export DOCKER_USERNAME=godlinchong
export DOCKER_PASSWORD=your_password_or_token

echo "$DOCKER_PASSWORD" | docker login --username "$DOCKER_USERNAME" --password-stdin
```

---

## 📦 版本管理策略

### 推荐的标签策略

```bash
# 1. 推送最新版本
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:latest --push

# 2. 推送版本标签
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:v1.0.0 --push

# 3. 推送主版本标签
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:v1 --push

# 4. 同时推送多个标签
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile \
  -t godlinchong/agentmem:latest \
  -t godlinchong/agentmem:v1.0.0 \
  -t godlinchong/agentmem:v1 \
  --push .
```

### 标签说明

| 标签 | 说明 | 示例 |
|------|------|------|
| `latest` | 最新版本 | `godlinchong/agentmem:latest` |
| `v1.0.0` | 语义化版本 | `godlinchong/agentmem:v1.0.0` |
| `v1` | 主版本 | `godlinchong/agentmem:v1` |
| `amd64` | 平台标签 | `godlinchong/agentmem:amd64` |
| `2025-12-02` | 日期标签 | `godlinchong/agentmem:2025-12-02` |

---

## 🔧 自动化脚本

### 创建发布脚本

创建 `publish-to-dockerhub.sh`:

```bash
#!/bin/bash
# 发布 AgentMem 到 Docker Hub

set -e

DOCKER_USERNAME="godlinchong"
IMAGE_NAME="agentmem"
VERSION="${1:-latest}"

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}发布 AgentMem 到 Docker Hub${NC}"
echo -e "${BLUE}========================================${NC}"
echo "用户: $DOCKER_USERNAME"
echo "镜像: $IMAGE_NAME"
echo "版本: $VERSION"
echo ""

# 检查是否已登录
if ! docker info 2>/dev/null | grep -q "Username"; then
    echo "⚠️  未检测到 Docker 登录状态"
    echo "正在登录 Docker Hub..."
    docker login
fi

# 构建并推送
echo -e "${GREEN}开始构建并推送镜像...${NC}"
./build-docker-linux-amd64.sh \
  --tag "$DOCKER_USERNAME/$IMAGE_NAME:$VERSION" \
  --tag "$DOCKER_USERNAME/$IMAGE_NAME:latest" \
  --push

echo ""
echo -e "${GREEN}✅ 发布成功！${NC}"
echo "镜像地址: https://hub.docker.com/r/$DOCKER_USERNAME/$IMAGE_NAME"
echo ""
echo "拉取镜像:"
echo "  docker pull $DOCKER_USERNAME/$IMAGE_NAME:$VERSION"
```

**使用方法**:
```bash
chmod +x publish-to-dockerhub.sh

# 发布最新版本
./publish-to-dockerhub.sh

# 发布指定版本
./publish-to-dockerhub.sh v1.0.0
```

---

## 🧪 验证和测试

### 1. 检查推送是否成功

```bash
# 检查远程镜像
docker manifest inspect godlinchong/agentmem:latest

# 或访问网页
# https://hub.docker.com/r/godlinchong/agentmem
```

### 2. 从其他机器拉取测试

```bash
# 拉取镜像
docker pull godlinchong/agentmem:latest

# 运行测试
docker run --rm -p 8080:8080 \
  -e ZHIPU_API_KEY="your_key" \
  godlinchong/agentmem:latest
```

### 3. 查看镜像信息

```bash
docker image inspect godlinchong/agentmem:latest
```

---

## 📊 完整发布流程

### 流程 1: 手动发布

```bash
# 1. 登录
docker login

# 2. 构建并推送
./build-docker-linux-amd64.sh \
  --tag godlinchong/agentmem:latest \
  --push

# 3. 验证
docker manifest inspect godlinchong/agentmem:latest
```

### 流程 2: 使用发布脚本

```bash
# 1. 创建发布脚本（见上方）
chmod +x publish-to-dockerhub.sh

# 2. 执行发布
./publish-to-dockerhub.sh v1.0.0
```

### 流程 3: CI/CD 集成

在 GitHub Actions 或其他 CI/CD 平台中：

```yaml
name: Publish to Docker Hub

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: godlinchong
          password: ${{ secrets.DOCKERHUB_TOKEN }}
      
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          platforms: linux/amd64
          push: true
          tags: |
            godlinchong/agentmem:latest
            godlinchong/agentmem:${{ github.ref_name }}
```

---

## ⚠️ 注意事项

### 1. 镜像大小

- Docker Hub 免费账户有存储限制
- 建议使用多阶段构建（已实现）
- 定期清理旧版本镜像

### 2. 安全

- ✅ 使用 Access Token 而不是密码
- ✅ 不要在代码中硬编码凭证
- ✅ 使用环境变量或密钥管理

### 3. 版本管理

- ✅ 使用语义化版本（semver）
- ✅ 保持 `latest` 标签指向最新稳定版
- ✅ 定期更新版本标签

### 4. 仓库设置

在 Docker Hub 上设置：
- **Visibility**: Public 或 Private
- **Description**: 添加镜像描述
- **README**: 添加使用说明
- **Build Settings**: 可选，配置自动构建

---

## 🔍 故障排查

### 问题 1: 认证失败

**错误**: `unauthorized: authentication required`

**解决**:
```bash
# 重新登录
docker logout
docker login
```

### 问题 2: 权限不足

**错误**: `denied: requested access to the resource is denied`

**解决**:
- 确认用户名正确: `godlinchong`
- 确认有推送权限
- 检查仓库是否为私有（需要权限）

### 问题 3: 标签格式错误

**错误**: `invalid reference format`

**解决**:
- 确保标签格式: `godlinchong/agentmem:tag`
- 不要使用 `godlinchong/agentmem`（缺少标签）

---

## 📝 示例命令汇总

```bash
# 1. 登录
docker login

# 2. 构建并推送最新版本
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:latest --push

# 3. 构建并推送版本标签
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:v1.0.0 --push

# 4. 推送多个标签
docker buildx build \
  --platform linux/amd64 \
  -f Dockerfile \
  -t godlinchong/agentmem:latest \
  -t godlinchong/agentmem:v1.0.0 \
  --push .

# 5. 验证推送
docker manifest inspect godlinchong/agentmem:latest

# 6. 拉取测试
docker pull godlinchong/agentmem:latest
```

---

## ✅ 总结

**Docker Hub 用户名**: `godlinchong`  
**镜像仓库**: `godlinchong/agentmem`  
**发布地址**: https://hub.docker.com/r/godlinchong/agentmem

**快速发布命令**:
```bash
docker login
./build-docker-linux-amd64.sh --tag godlinchong/agentmem:latest --push
```

---

**最后更新**: 2025-12-02  
**状态**: ✅ 配置完成

