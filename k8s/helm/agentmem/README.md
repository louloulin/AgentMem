# AgentMem Helm Chart

企业级 Kubernetes 部署配置，用于在 Kubernetes 集群中部署 AgentMem 记忆管理系统。

## 功能特性

- ✅ 高可用部署（多副本 + 反亲和性）
- ✅ 自动扩缩容（HPA）
- ✅ Pod 中断预算（PDB）
- ✅ 健康检查（Liveness/Readiness/Startup Probes）
- ✅ Ingress 配置（HTTPS + TLS）
- ✅ ConfigMap 和 Secret 管理
- ✅ Prometheus 监控集成
- ✅ 资源限制和请求
- ✅ 安全上下文配置
- ✅ 多环境配置（开发/预发布/生产）

## 前置要求

- Kubernetes 1.23+
- Helm 3.8+
- PV provisioner（用于持久化存储）
- Ingress Controller（如 nginx-ingress）
- cert-manager（可选，用于自动 TLS 证书）
- Prometheus Operator（可选，用于监控）

## 快速开始

### 1. 添加 Helm 仓库

```bash
# 添加依赖的 Helm 仓库
helm repo add bitnami https://charts.bitnami.com/bitnami
helm repo add elastic https://helm.elastic.co
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```

### 2. 安装 Chart

#### 开发环境（使用默认配置）

```bash
helm install agentmem ./agentmem \
  --namespace agentmem \
  --create-namespace
```

#### 预发布环境

```bash
helm install agentmem ./agentmem \
  --namespace agentmem-staging \
  --create-namespace \
  --values ./agentmem/values-staging.yaml
```

#### 生产环境

```bash
helm install agentmem ./agentmem \
  --namespace agentmem-prod \
  --create-namespace \
  --values ./agentmem/values-production.yaml \
  --set externalDatabase.password=<your-db-password> \
  --set externalRedis.password=<your-redis-password>
```

### 3. 验证部署

```bash
# 查看 Pod 状态
kubectl get pods -n agentmem

# 查看服务状态
kubectl get svc -n agentmem

# 查看 Ingress 状态
kubectl get ingress -n agentmem

# 查看 HPA 状态
kubectl get hpa -n agentmem

# 查看 PDB 状态
kubectl get pdb -n agentmem
```

### 4. 访问应用

```bash
# 获取 Ingress 地址
kubectl get ingress -n agentmem

# 或者使用端口转发（开发环境）
kubectl port-forward -n agentmem svc/agentmem 8080:8080

# 访问 API
curl http://localhost:8080/health
```

## 配置说明

### 核心配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `replicaCount` | Pod 副本数 | `3` |
| `image.registry` | 镜像仓库 | `docker.io` |
| `image.repository` | 镜像名称 | `agentmem/server` |
| `image.tag` | 镜像标签 | `6.0.0` |
| `image.pullPolicy` | 镜像拉取策略 | `IfNotPresent` |

### 服务配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `service.type` | 服务类型 | `ClusterIP` |
| `service.port` | 服务端口 | `8080` |
| `service.targetPort` | 目标端口 | `8080` |

### Ingress 配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `ingress.enabled` | 启用 Ingress | `true` |
| `ingress.className` | Ingress 类名 | `nginx` |
| `ingress.hosts[0].host` | 主机名 | `agentmem.example.com` |
| `ingress.tls[0].secretName` | TLS Secret 名称 | `agentmem-tls` |

### 资源配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `resources.limits.cpu` | CPU 限制 | `2000m` |
| `resources.limits.memory` | 内存限制 | `4Gi` |
| `resources.requests.cpu` | CPU 请求 | `1000m` |
| `resources.requests.memory` | 内存请求 | `2Gi` |

### 自动扩缩容配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `autoscaling.enabled` | 启用 HPA | `true` |
| `autoscaling.minReplicas` | 最小副本数 | `3` |
| `autoscaling.maxReplicas` | 最大副本数 | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | CPU 目标使用率 | `70` |
| `autoscaling.targetMemoryUtilizationPercentage` | 内存目标使用率 | `80` |

### 数据库配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `postgresql.enabled` | 启用内部 PostgreSQL | `true` |
| `externalDatabase.host` | 外部数据库主机 | `""` |
| `externalDatabase.port` | 外部数据库端口 | `5432` |
| `externalDatabase.database` | 数据库名称 | `agentmem` |
| `externalDatabase.username` | 数据库用户名 | `agentmem` |
| `externalDatabase.password` | 数据库密码 | `""` |

### Redis 配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `redis.enabled` | 启用内部 Redis | `true` |
| `externalRedis.host` | 外部 Redis 主机 | `""` |
| `externalRedis.port` | 外部 Redis 端口 | `6379` |
| `externalRedis.password` | Redis 密码 | `""` |

### 监控配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `metrics.enabled` | 启用 Prometheus 指标 | `true` |
| `metrics.port` | 指标端口 | `9090` |
| `metrics.path` | 指标路径 | `/metrics` |
| `metrics.serviceMonitor.enabled` | 启用 ServiceMonitor | `false` |

## 高级配置

### 使用外部数据库

```yaml
postgresql:
  enabled: false

externalDatabase:
  host: "my-postgres.example.com"
  port: 5432
  database: "agentmem"
  username: "agentmem"
  password: "my-secure-password"
```

### 使用外部 Redis

```yaml
redis:
  enabled: false

externalRedis:
  host: "my-redis.example.com"
  port: 6379
  password: "my-redis-password"
```

### 配置 TLS

```yaml
tls:
  enabled: true
  cert: |
    -----BEGIN CERTIFICATE-----
    ...
    -----END CERTIFICATE-----
  key: |
    -----BEGIN PRIVATE KEY-----
    ...
    -----END PRIVATE KEY-----
```

### 配置节点亲和性

```yaml
nodeSelector:
  workload: production
  node.kubernetes.io/instance-type: c5.2xlarge

affinity:
  nodeAffinity:
    requiredDuringSchedulingIgnoredDuringExecution:
      nodeSelectorTerms:
      - matchExpressions:
        - key: node.kubernetes.io/instance-type
          operator: In
          values:
          - c5.2xlarge
          - c5.4xlarge
```

## 升级

```bash
# 升级到新版本
helm upgrade agentmem ./agentmem \
  --namespace agentmem \
  --values ./agentmem/values-production.yaml

# 查看升级历史
helm history agentmem -n agentmem

# 回滚到上一个版本
helm rollback agentmem -n agentmem
```

## 卸载

```bash
helm uninstall agentmem -n agentmem
```

## 故障排查

### 查看 Pod 日志

```bash
kubectl logs -n agentmem -l app.kubernetes.io/name=agentmem --tail=100
```

### 查看 Pod 事件

```bash
kubectl describe pod -n agentmem <pod-name>
```

### 查看 HPA 状态

```bash
kubectl describe hpa -n agentmem agentmem
```

### 查看 PDB 状态

```bash
kubectl describe pdb -n agentmem agentmem
```

## 许可证

MIT License

## 支持

- 文档: https://agentmem.dev/docs
- GitHub: https://github.com/agentmem/agentmem
- Email: support@agentmem.dev

