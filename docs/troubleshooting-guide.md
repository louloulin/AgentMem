# AgentMem 故障排查指南

**最后更新**: 2025-11-03  
**版本**: v1.0  
**适用版本**: AgentMem 2.0+

---

## 📋 目录

1. [常见问题](#常见问题)
2. [启动失败](#启动失败)
3. [性能问题](#性能问题)
4. [连接问题](#连接问题)
5. [内存问题](#内存问题)
6. [数据库问题](#数据库问题)
7. [监控和日志](#监控和日志)
8. [获取帮助](#获取帮助)

---

## 🔍 常见问题

### Q1: Docker容器无法启动

**症状**:
```bash
docker-compose up -d
# 容器启动后立即退出
```

**诊断步骤**:
```bash
# 1. 检查日志
docker-compose logs agentmem

# 2. 检查端口占用
lsof -i :8080
lsof -i :5432

# 3. 检查磁盘空间
df -h
```

**常见原因**:
- 端口已被占用
- 环境变量配置错误
- 数据库连接失败
- 内存不足

**解决方案**:
```bash
# 清理并重启
docker-compose down -v
docker-compose up -d

# 如果端口占用，修改docker-compose.yml
# 或停止占用端口的进程
```

---

### Q2: 健康检查失败

**症状**:
```bash
curl http://localhost:8080/health
# 返回 503 或连接超时
```

**诊断**:
```bash
# 1. 检查服务状态
docker-compose ps

# 2. 检查依赖服务
curl http://localhost:5432  # PostgreSQL
curl http://localhost:6379  # Redis
curl http://localhost:6333  # Qdrant

# 3. 检查健康检查日志
docker-compose logs agentmem | grep health
```

**解决方案**:
```bash
# 1. 重启依赖服务
docker-compose restart postgres redis qdrant

# 2. 等待服务完全启动（通常需要30-60秒）
sleep 60

# 3. 再次检查健康状态
curl http://localhost:8080/health/ready
```

---

### Q3: API请求返回500错误

**症状**:
```bash
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"content": "test"}'
# 返回500 Internal Server Error
```

**诊断**:
```bash
# 1. 检查错误日志
docker-compose logs agentmem --tail=100

# 2. 检查数据库连接
docker exec agentmem-postgres psql -U agentmem -c "SELECT 1"

# 3. 检查trace_id
# 错误响应会包含trace_id，用于日志搜索
```

**常见原因**:
- 数据库连接断开
- 请求格式错误
- 权限不足
- LLM API限流

**解决方案**:
```bash
# 1. 验证数据库连接
docker-compose restart postgres
sleep 10

# 2. 检查请求格式
# 必需字段: content, memory_type

# 3. 检查认证token
# 添加 Authorization: Bearer <token>
```

---

## 🚀 启动失败

### 数据库初始化失败

**症状**:
```
Error: Failed to initialize database
```

**解决方案**:
```bash
# 1. 清理数据库
docker-compose down -v

# 2. 删除持久化卷
docker volume rm agentmem_postgres-data

# 3. 重新启动
docker-compose up -d

# 4. 等待初始化完成
docker-compose logs -f agentmem | grep "Database initialized"
```

### 向量存储连接失败

**症状**:
```
Error: Failed to connect to vector store
```

**解决方案**:
```bash
# 1. 检查Qdrant状态
docker-compose ps qdrant

# 2. 重启Qdrant
docker-compose restart qdrant

# 3. 验证连接
curl http://localhost:6333/health
```

---

## ⚡ 性能问题

### 响应时间过长

**症状**: API响应时间 > 2秒

**诊断**:
```bash
# 1. 检查系统负载
top
htop

# 2. 检查数据库性能
docker exec agentmem-postgres psql -U agentmem -c "
SELECT 
  pid, 
  now() - query_start as duration, 
  query 
FROM pg_stat_activity 
WHERE state = 'active' 
ORDER BY duration DESC;
"

# 3. 检查慢查询日志
docker-compose logs agentmem | grep "slow query"
```

**解决方案**:
```bash
# 1. 增加数据库连接池
export AGENTMEM_DB_POOL_SIZE=20

# 2. 启用Redis缓存
export AGENTMEM_REDIS_URL=redis://localhost:6379

# 3. 调整向量搜索参数
export AGENTMEM_VECTOR_SEARCH_LIMIT=50

# 4. 重启服务
docker-compose restart agentmem
```

### 内存使用过高

**症状**: 内存使用 > 4GB

**诊断**:
```bash
# 1. 检查内存使用
docker stats

# 2. 检查具体服务
docker stats agentmem-server
docker stats agentmem-postgres
docker stats agentmem-redis
```

**解决方案**:
```bash
# 1. 限制容器内存
# 编辑 docker-compose.yml
services:
  agentmem:
    deploy:
      resources:
        limits:
          memory: 2G
        reservations:
          memory: 1G

# 2. 减少并发连接
export AGENTMEM_MAX_CONNECTIONS=100

# 3. 启用内存优化模式
export AGENTMEM_MEMORY_OPTIMIZATION=true
```

---

## 🔌 连接问题

### WebSocket连接断开

**症状**: WebSocket频繁断开重连

**诊断**:
```bash
# 1. 检查网络稳定性
ping localhost

# 2. 检查WebSocket日志
docker-compose logs agentmem | grep websocket

# 3. 测试WebSocket连接
wscat -c ws://localhost:8080/api/v1/ws
```

**解决方案**:
```bash
# 1. 增加WebSocket超时
export AGENTMEM_WS_TIMEOUT=600

# 2. 启用WebSocket心跳
export AGENTMEM_WS_PING_INTERVAL=30

# 3. 使用nginx反向代理（生产环境）
# 在nginx配置中添加WebSocket支持
```

### LLM API连接失败

**症状**:
```
Error: Failed to call LLM API
```

**诊断**:
```bash
# 1. 检查API Key
echo $OPENAI_API_KEY
echo $DEEPSEEK_API_KEY

# 2. 测试API连接
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# 3. 检查网络
curl -I https://api.openai.com
```

**解决方案**:
```bash
# 1. 设置正确的API Key
export OPENAI_API_KEY=sk-xxx

# 2. 配置代理（如果需要）
export HTTPS_PROXY=http://proxy.example.com:8080

# 3. 切换到备用provider
export AGENTMEM_LLM_PROVIDER=deepseek
```

---

## 💾 数据库问题

### PostgreSQL连接池耗尽

**症状**:
```
FATAL: sorry, too many clients already
```

**解决方案**:
```bash
# 1. 增加PostgreSQL最大连接数
docker exec agentmem-postgres psql -U agentmem -c "
ALTER SYSTEM SET max_connections = 200;
"

# 2. 重启PostgreSQL
docker-compose restart postgres

# 3. 减少应用连接池大小
export AGENTMEM_DB_POOL_SIZE=10
```

### 数据库磁盘空间满

**症状**:
```
ERROR: could not extend file: No space left on device
```

**解决方案**:
```bash
# 1. 检查磁盘使用
docker exec agentmem-postgres du -sh /var/lib/postgresql/data

# 2. 清理旧数据
docker exec agentmem-postgres psql -U agentmem -c "
VACUUM FULL;
"

# 3. 归档历史数据
# 导出30天前的数据
docker exec agentmem-postgres pg_dump \
  -U agentmem \
  -t memories \
  --where="created_at < NOW() - INTERVAL '30 days'" \
  > old_memories.sql

# 删除已归档的数据
docker exec agentmem-postgres psql -U agentmem -c "
DELETE FROM memories WHERE created_at < NOW() - INTERVAL '30 days';
"
```

---

## 📊 监控和日志

### 查看实时日志

```bash
# 所有服务日志
docker-compose logs -f

# 特定服务日志
docker-compose logs -f agentmem

# 最近100条日志
docker-compose logs --tail=100 agentmem

# 包含时间戳
docker-compose logs -t agentmem
```

### 访问Grafana监控

```bash
# 1. 打开Grafana
open http://localhost:3000

# 2. 默认登录
# 用户名: admin
# 密码: admin

# 3. 查看Dashboard
# 导航到 Dashboards → AgentMem Overview
```

### 导出日志

```bash
# 导出所有日志
docker-compose logs > agentmem-logs.txt

# 导出特定时间段
docker-compose logs --since="2025-11-03T00:00:00" > logs.txt

# 压缩日志
docker-compose logs | gzip > logs.gz
```

### Prometheus查询

```bash
# 1. 访问Prometheus
open http://localhost:9090

# 2. 常用查询
# 请求速率
rate(agentmem_http_requests_total[5m])

# P95延迟
histogram_quantile(0.95, agentmem_http_request_duration_seconds)

# 错误率
rate(agentmem_errors_total[5m])

# 内存使用
process_resident_memory_bytes / 1024 / 1024 / 1024
```

---

## 🆘 获取帮助

### 诊断信息收集

运行诊断脚本：
```bash
#!/bin/bash
# diagnose.sh - 收集诊断信息

echo "=== AgentMem诊断报告 ==="
echo "生成时间: $(date)"
echo ""

echo "=== 版本信息 ==="
docker-compose version
docker version
echo ""

echo "=== 容器状态 ==="
docker-compose ps
echo ""

echo "=== 资源使用 ==="
docker stats --no-stream
echo ""

echo "=== 磁盘空间 ==="
df -h
echo ""

echo "=== 最近100条日志 ==="
docker-compose logs --tail=100
echo ""

echo "=== 健康检查 ==="
curl -s http://localhost:8080/health | jq .
echo ""

echo "=== 指标快照 ==="
curl -s http://localhost:8080/metrics | head -50
```

### 提交Issue

当您需要报告问题时，请包含：

1. **环境信息**:
   - 操作系统和版本
   - Docker版本
   - AgentMem版本

2. **问题描述**:
   - 预期行为
   - 实际行为
   - 复现步骤

3. **诊断信息**:
   - 运行上面的诊断脚本
   - 相关日志
   - 错误截图

4. **配置信息** (脱敏后):
   - docker-compose.yml
   - 环境变量

**提交位置**:
- GitHub Issues: https://github.com/louloulin/agentmem/issues
- Email: support@agentmem.io

###社区支持

- 💬 **Discord**: https://discord.gg/agentmem
- 📖 **GitHub Discussions**: https://github.com/louloulin/agentmem/discussions
- 📧 **Email**: support@agentmem.io
- 🌐 **文档**: https://docs.agentmem.io

---

## 📚 相关文档

- [快速开始指南](user-guide/quickstart.md)
- [生产部署指南](deployment/production-guide.md)
- [性能调优指南](performance-tuning-guide.md)
- [API参考](api/API_REFERENCE.md)

---

**文档版本**: v1.0  
**最后更新**: 2025-11-03  
**维护团队**: AgentMem Support Team

---

## ✅ 快速参考

### 重启服务
```bash
docker-compose restart agentmem
```

### 查看日志
```bash
docker-compose logs -f agentmem --tail=100
```

### 健康检查
```bash
curl http://localhost:8080/health
```

### 清理重启
```bash
docker-compose down -v && docker-compose up -d
```

### 备份数据
```bash
./scripts/backup.sh
```

---

🎯 **大多数问题可以通过重启服务解决！**

💡 **记得查看日志获取详细错误信息！**

🆘 **如果问题持续，请联系我们！**
