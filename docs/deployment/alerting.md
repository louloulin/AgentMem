# AgentMem 告警配置指南

**版本**: v1.0  
**更新日期**: 2025-11-03  
**适用版本**: AgentMem 2.0+

---

## 📋 目录

1. [告警架构](#告警架构)
2. [告警规则](#告警规则)
3. [通知渠道](#通知渠道)
4. [告警升级策略](#告警升级策略)
5. [测试和验证](#测试和验证)
6. [最佳实践](#最佳实践)

---

## 🏗️ 告警架构

### 组件架构

```
┌─────────────┐       ┌──────────────┐       ┌────────────────┐
│  Prometheus │──────▶│ Alertmanager │──────▶│  通知渠道      │
│  (告警规则)  │       │  (路由分发)   │       │  (邮件/Slack)  │
└─────────────┘       └──────────────┘       └────────────────┘
       │                      │
       │                      │
       ▼                      ▼
┌─────────────┐       ┌──────────────┐
│  AgentMem   │       │  抑制规则    │
│  Metrics    │       │  (去重)      │
└─────────────┘       └──────────────┘
```

### 告警流程

1. **监控采集**: Prometheus采集AgentMem指标
2. **规则评估**: Prometheus根据告警规则评估指标
3. **告警触发**: 条件满足时触发告警
4. **路由分发**: Alertmanager根据规则路由告警
5. **通知发送**: 通过配置的渠道发送通知
6. **升级处理**: 根据严重性和时间升级

---

## 🚨 告警规则

### 严重性级别

| 级别 | 说明 | 响应时间 | 通知方式 |
|------|------|---------|---------|
| **Critical** | 服务中断 | 立即 (5分钟内) | 邮件+Slack+电话 |
| **Warning** | 性能问题 | 30分钟内 | 邮件+Slack |
| **Info** | 信息提示 | 2小时内 | Slack |

### 核心告警规则

#### 1. 服务可用性

**AgentMemServiceDown** (Critical)
```yaml
expr: up{job="agentmem-server"} == 0
for: 1m
```
- **含义**: AgentMem服务停止响应
- **影响**: 所有功能不可用
- **处理**: 立即重启服务，检查日志

#### 2. 高错误率

**AgentMemHighErrorRate** (Warning)
```yaml
expr: rate(agentmem_http_requests_total{status=~"5.."}[5m]) / 
      rate(agentmem_http_requests_total[5m]) > 0.05
for: 5m
```
- **含义**: 错误率超过5%
- **影响**: 用户体验下降
- **处理**: 检查错误日志，定位问题

#### 3. 高延迟

**AgentMemHighLatency** (Warning)
```yaml
expr: histogram_quantile(0.95, 
      rate(agentmem_http_request_duration_seconds_bucket[5m])) > 2
for: 5m
```
- **含义**: P95延迟超过2秒
- **影响**: 响应慢，用户等待时间长
- **处理**: 性能分析，优化慢查询

#### 4. 内存使用率高

**AgentMemHighMemoryUsage** (Warning)
```yaml
expr: (agentmem_memory_usage_bytes / agentmem_memory_limit_bytes) > 0.85
for: 5m
```
- **含义**: 内存使用超过85%
- **影响**: 可能导致OOM
- **处理**: 检查内存泄漏，增加内存限制

#### 5. 数据库连接池

**AgentMemDatabaseConnectionPoolHigh** (Warning)
```yaml
expr: agentmem_database_connections_active / 
      agentmem_database_connections_max > 0.8
for: 2m
```
- **含义**: 数据库连接池使用率超过80%
- **影响**: 新请求可能被阻塞
- **处理**: 增加连接池大小，优化查询

---

## 📧 通知渠道

### 1. 邮件通知

**配置示例**:
```yaml
email_configs:
  - to: 'team@agentmem.io'
    headers:
      Subject: '[AgentMem] Alert: {{ .GroupLabels.alertname }}'
    html: |
      <h2>Alert: {{ .GroupLabels.alertname }}</h2>
      <p><strong>Severity:</strong> {{ .CommonLabels.severity }}</p>
      <p><strong>Summary:</strong> {{ .CommonAnnotations.summary }}</p>
```

**优势**:
- ✅ 可靠性高
- ✅ 详细信息完整
- ✅ 有历史记录

**最佳实践**:
- 设置合理的邮件主题
- 包含关键信息和处理建议
- 添加链接到Grafana/Prometheus
- 区分不同严重性级别

### 2. Slack通知

**配置示例**:
```yaml
slack_configs:
  - channel: '#agentmem-alerts'
    username: 'AlertManager'
    icon_emoji: ':warning:'
    title: '[{{ .Status | toUpper }}] {{ .GroupLabels.alertname }}'
    text: |
      *Severity:* {{ .CommonLabels.severity }}
      *Summary:* {{ .CommonAnnotations.summary }}
    send_resolved: true
```

**优势**:
- ✅ 实时性强
- ✅ 团队可见
- ✅ 便于协作

**最佳实践**:
- 使用不同channel区分严重性
- Critical告警@channel提醒
- 发送resolved消息
- 添加快速响应按钮

### 3. PagerDuty

**配置示例**:
```yaml
pagerduty_configs:
  - service_key: '${PAGERDUTY_SERVICE_KEY}'
    description: '{{ .GroupLabels.alertname }}'
```

**优势**:
- ✅ OnCall管理
- ✅ 告警升级
- ✅ 电话/SMS通知

**适用场景**:
- Critical级别告警
- 工作时间外
- 需要立即响应

### 4. Webhook

**配置示例**:
```yaml
webhook_configs:
  - url: 'https://your-webhook-endpoint.com/alerts'
    send_resolved: true
```

**用途**:
- 集成自定义系统
- 触发自动化流程
- 数据持久化

---

## 📈 告警升级策略

### 升级级别

```
Level 1: 团队Slack     (立即)
         │
         ▼ (15分钟未处理)
Level 2: 团队邮件     (15分钟后)
         │
         ▼ (30分钟未处理)
Level 3: OnCall工程师  (30分钟后)
         │
         ▼ (1小时未处理)
Level 4: 技术主管     (1小时后)
```

### 配置示例

```yaml
route:
  receiver: 'team-slack'
  routes:
    - match:
        severity: critical
      receiver: 'critical-alerts'
      group_wait: 10s
      continue: true  # 继续匹配后续规则
    
    - match:
        severity: critical
      receiver: 'oncall-phone'
      group_wait: 15m  # 15分钟后升级到电话
```

### 升级触发条件

1. **时间触发**
   - Critical: 15分钟未处理
   - Warning: 1小时未处理

2. **严重性触发**
   - Critical立即多渠道通知
   - 持续告警自动升级

3. **业务影响触发**
   - 影响用户数 > 100
   - 错误率 > 10%
   - 服务完全中断

---

## 🧪 测试和验证

### 运行告警测试

```bash
# 运行完整测试
./scripts/test_alerts.sh

# 测试特定告警规则
./scripts/test_alerts.sh --alert AgentMemHighErrorRate

# 测试通知渠道
./scripts/test_alerts.sh --test-notifications
```

### 测试清单

- [ ] Prometheus连接正常
- [ ] Alertmanager连接正常
- [ ] 告警规则加载成功
- [ ] 测试告警发送成功
- [ ] 邮件通知收到
- [ ] Slack通知收到
- [ ] 告警路由正确
- [ ] 抑制规则生效
- [ ] 升级策略生效

### 手动发送测试告警

```bash
# 发送测试告警到Alertmanager
curl -X POST http://localhost:9093/api/v2/alerts \
  -H "Content-Type: application/json" \
  -d '[
    {
      "labels": {
        "alertname": "TestAlert",
        "severity": "warning",
        "service": "agentmem"
      },
      "annotations": {
        "summary": "This is a test alert"
      }
    }
  ]'
```

### 验证告警规则

```bash
# 检查Prometheus告警规则
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].rules[] | select(.type=="alerting")'

# 查看当前活跃告警
curl http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | select(.state=="firing")'
```

---

## ✅ 最佳实践

### 1. 告警设计原则

**DO**:
- ✅ 每个告警必须可操作
- ✅ 清晰的告警描述和建议
- ✅ 合理的阈值和持续时间
- ✅ 区分不同严重性级别

**DON'T**:
- ❌ 过于敏感，频繁误报
- ❌ 没有处理建议的告警
- ✅ 所有告警都是Critical
- ❌ 告警风暴

### 2. 阈值设置

| 指标 | Warning | Critical |
|------|---------|----------|
| 错误率 | >5% | >10% |
| P95延迟 | >2s | >5s |
| 内存使用 | >85% | >95% |
| CPU使用 | >80% | >95% |
| 磁盘使用 | >80% | >90% |

### 3. 告警分组

**按服务分组**:
```yaml
group_by: ['alertname', 'service']
group_wait: 30s
group_interval: 5m
```

**按严重性分组**:
```yaml
routes:
  - match:
      severity: critical
    group_wait: 10s
  - match:
      severity: warning
    group_wait: 1m
```

### 4. 抑制规则

避免告警风暴:
```yaml
inhibit_rules:
  # 服务down时抑制其他告警
  - source_match:
      alertname: 'AgentMemServiceDown'
    target_match_re:
      alertname: '(AgentMemHigh.*|AgentMemLow.*)'
    equal: ['service']
```

### 5. 静默规则

计划维护时使用:
```bash
# 创建静默
amtool silence add \
  alertname="AgentMemHighLatency" \
  --duration=1h \
  --comment="Scheduled maintenance"

# 查看静默
amtool silence query

# 删除静默
amtool silence expire <silence-id>
```

### 6. OnCall轮换

配置OnCall轮换:
```yaml
# 使用PagerDuty的轮换功能
pagerduty_configs:
  - service_key: '${PAGERDUTY_SERVICE_KEY}'
    # PagerDuty会根据设置的轮换表自动通知
```

---

## 🔧 故障排查

### 告警未触发

**检查步骤**:
1. 验证Prometheus是否采集到指标
2. 检查告警规则表达式
3. 确认持续时间(for)设置
4. 查看Prometheus日志

```bash
# 查看指标
curl http://localhost:9090/api/v1/query?query=up{job="agentmem-server"}

# 评估告警规则
curl http://localhost:9090/api/v1/rules
```

### 未收到通知

**检查步骤**:
1. 确认Alertmanager配置正确
2. 检查路由规则
3. 验证接收者配置
4. 查看Alertmanager日志

```bash
# 查看Alertmanager状态
curl http://localhost:9093/api/v2/status

# 查看活跃告警
curl http://localhost:9093/api/v2/alerts
```

### 告警过多

**解决方案**:
1. 调整阈值
2. 增加持续时间
3. 添加抑制规则
4. 优化分组策略

---

## 📚 相关资源

### 内部文档
- [性能监控指南](performance-monitoring-guide.md)
- [故障排查指南](troubleshooting-guide.md)
- [部署指南](deployment/production-guide.md)

### 外部资源
- [Prometheus Alerting](https://prometheus.io/docs/alerting/latest/overview/)
- [Alertmanager配置](https://prometheus.io/docs/alerting/latest/configuration/)
- [PromQL查询语言](https://prometheus.io/docs/prometheus/latest/querying/basics/)

---

**文档版本**: v1.0  
**最后更新**: 2025-11-03  
**维护团队**: AgentMem SRE Team

---

## ✅ 快速参考

### 运行告警测试
```bash
./scripts/test_alerts.sh
```

### 查看活跃告警
```bash
curl http://localhost:9090/api/v1/alerts
```

### 发送测试告警
```bash
curl -X POST http://localhost:9093/api/v2/alerts \
  -d '[{"labels":{"alertname":"Test","severity":"warning"}}]'
```

### 创建静默
```bash
amtool silence add alertname="Test" --duration=1h
```

---

🚨 **及时响应告警，保持系统健康！**

📊 **定期审查告警规则，避免误报和漏报！**

🔄 **持续优化告警策略，提升运维效率！**

