# AgentMem 安全加固指南

**版本**: v1.0  
**更新日期**: 2025-11-03  
**适用版本**: AgentMem 2.0+

---

## 📋 目录

1. [RBAC权限系统](#rbac权限系统)
2. [认证和授权](#认证和授权)
3. [数据加密](#数据加密)
4. [网络安全](#网络安全)
5. [依赖安全](#依赖安全)
6. [安全审计](#安全审计)
7. [最佳实践](#最佳实践)

---

## 🔐 RBAC权限系统

### 角色定义

AgentMem实现了三级角色系统：

| 角色 | 权限 | 适用场景 |
|------|------|---------|
| **Admin** | 完全访问权限 | 系统管理员 |
| **User** | 读写权限 | 普通用户 |
| **ReadOnly** | 仅读取权限 | 只读用户、审计员 |

### 权限矩阵

| 操作 | Admin | User | ReadOnly |
|------|-------|------|----------|
| **记忆** |
| - 读取 | ✅ | ✅ | ✅ |
| - 创建 | ✅ | ✅ | ❌ |
| - 更新 | ✅ | ✅ | ❌ |
| - 删除 | ✅ | ❌ | ❌ |
| **Agent** |
| - 读取 | ✅ | ✅ | ✅ |
| - 创建 | ✅ | ✅ | ❌ |
| - 更新 | ✅ | ✅ | ❌ |
| - 删除 | ✅ | ❌ | ❌ |
| **用户** |
| - 读取 | ✅ | ✅ | ✅ |
| - 创建 | ✅ | ❌ | ❌ |
| - 更新 | ✅ | ❌ | ❌ |
| - 删除 | ✅ | ❌ | ❌ |
| **系统** |
| - 查看指标 | ✅ | ❌ | ❌ |
| - 系统管理 | ✅ | ❌ | ❌ |

### 使用示例

```rust
use agent_mem_server::rbac::{Role, RbacChecker, Permission, Resource, Action};

// 检查权限
let roles = vec!["user".to_string()];
let result = RbacChecker::check_permission(&roles, Permission::ReadMemory);

// 检查资源操作
let result = RbacChecker::check_resource_action(
    &roles,
    Resource::Memory,
    Action::Write
);

// 检查是否管理员
if RbacChecker::is_admin(&roles) {
    // 管理员操作
}
```

### 权限审计

所有权限检查都会自动记录审计日志：

```rust
AuditLogEntry {
    timestamp: 2025-11-03T10:00:00Z,
    user_id: "user123",
    action: "DELETE",
    resource: "memory",
    resource_id: Some("mem456"),
    allowed: false,  // 权限拒绝
    roles: ["user"],
    ip_address: Some("192.168.1.1"),
}
```

---

## 🔑 认证和授权

### JWT Token

AgentMem使用JWT进行身份认证：

```bash
# 生成token
curl -X POST http://localhost:8080/api/v1/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "secret"
  }'

# 使用token
curl http://localhost:8080/api/v1/memories \
  -H "Authorization: Bearer <token>"
```

### Token配置

```toml
# config.toml
[auth]
jwt_secret = "your-secret-key-min-32-chars"
token_expiry = "24h"  # Token过期时间
```

**安全建议**:
- ✅ 使用至少32字符的强随机密钥
- ✅ 定期轮换JWT密钥
- ✅ 设置合理的过期时间 (建议24小时)
- ✅ 使用HTTPS传输token
- ✅ 不要在URL中传递token

### 密码安全

AgentMem使用Argon2进行密码哈希：

```rust
use argon2::{Argon2, PasswordHasher};

// 哈希密码
let argon2 = Argon2::default();
let salt = SaltString::generate(&mut OsRng);
let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
```

**安全建议**:
- ✅ 最小密码长度: 8字符
- ✅ 要求大小写字母、数字、特殊字符
- ✅ 实施密码过期策略
- ✅ 防止暴力破解 (账户锁定)

### API Key认证

除了JWT，还支持API Key认证：

```bash
curl http://localhost:8080/api/v1/memories \
  -H "X-API-Key: your-api-key"
```

---

## 🔒 数据加密

### 传输加密

**生产环境必须使用HTTPS/TLS**：

```bash
# 使用Let's Encrypt获取证书
certbot certonly --standalone -d api.agentmem.io

# Nginx配置
server {
    listen 443 ssl http2;
    server_name api.agentmem.io;
    
    ssl_certificate /etc/letsencrypt/live/api.agentmem.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.agentmem.io/privkey.pem;
    
    # TLS配置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    
    # HSTS
    add_header Strict-Transport-Security "max-age=31536000" always;
}
```

### 静态数据加密

敏感数据应该加密存储：

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};

// 加密敏感字段
let cipher = Aes256Gcm::new(key);
let nonce = Nonce::from_slice(b"unique nonce");
let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
```

**需要加密的数据**:
- ✅ 用户密码 (Argon2哈希)
- ✅ API Keys
- ✅ 敏感的记忆内容
- ✅ 个人身份信息 (PII)

### 数据库加密

```sql
-- PostgreSQL透明数据加密
ALTER SYSTEM SET encryption = on;

-- 启用SSL连接
hostssl all all 0.0.0.0/0 md5
```

---

## 🌐 网络安全

### 防火墙配置

```bash
# 仅允许必要的端口
sudo ufw allow 22/tcp   # SSH
sudo ufw allow 80/tcp   # HTTP (重定向到HTTPS)
sudo ufw allow 443/tcp  # HTTPS
sudo ufw enable

# 限制SSH访问
sudo ufw limit 22/tcp
```

### 速率限制

AgentMem内置速率限制：

```rust
// 在middleware/quota.rs中配置
QuotaLimits {
    requests_per_minute: 60,
    requests_per_hour: 1000,
    requests_per_day: 10000,
}
```

### CORS配置

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("https://app.agentmem.io".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE]);
```

### 安全头

```rust
// 添加安全响应头
response.headers_mut().insert(
    "X-Content-Type-Options",
    "nosniff".parse().unwrap(),
);
response.headers_mut().insert(
    "X-Frame-Options",
    "DENY".parse().unwrap(),
);
response.headers_mut().insert(
    "X-XSS-Protection",
    "1; mode=block".parse().unwrap(),
);
```

---

## 📦 依赖安全

### 定期审计

```bash
# 运行安全审计
./scripts/security_audit.sh

# 或单独运行
cargo audit
cargo geiger
cargo clippy -- -W clippy::security
```

### 依赖更新策略

```bash
# 检查过时依赖
cargo outdated

# 更新依赖
cargo update

# 测试更新后的代码
cargo test
```

### 可信依赖

只使用来自可信来源的依赖：
- ✅ crates.io官方包
- ✅ 知名组织维护的包
- ✅ 活跃维护的包 (最近6个月有更新)
- ✅ 高下载量的包
- ✅ 有安全审计的包

### Cargo.lock

```bash
# 提交Cargo.lock到版本控制
git add Cargo.lock
git commit -m "Lock dependencies"
```

---

## 🔍 安全审计

### 自动化审计

GitHub Actions自动运行安全检查：

```yaml
name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 0 * * *'  # 每天运行

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run security audit
        run: ./scripts/security_audit.sh
```

### 手动审计清单

- [ ] 运行 `cargo audit` 检查依赖漏洞
- [ ] 运行 `cargo geiger` 检查unsafe代码
- [ ] 运行 `cargo clippy` 安全检查
- [ ] 审查最近的代码变更
- [ ] 检查权限配置
- [ ] 验证认证流程
- [ ] 测试授权控制
- [ ] 审查日志记录
- [ ] 检查加密实现
- [ ] 验证网络配置

### 渗透测试

定期进行渗透测试：

```bash
# SQL注入测试
sqlmap -u "http://localhost:8080/api/v1/memories?id=1"

# XSS测试
# 尝试注入恶意脚本

# 认证绕过测试
# 尝试访问未授权资源

# CSRF测试
# 验证CSRF保护
```

---

## ✅ 最佳实践

### 开发环境

- [ ] 使用不同的密钥 (开发 vs 生产)
- [ ] 不要提交敏感信息到Git
- [ ] 使用环境变量存储密钥
- [ ] 启用所有安全lint

### 生产环境

- [ ] 使用HTTPS/TLS
- [ ] 启用防火墙
- [ ] 配置速率限制
- [ ] 启用审计日志
- [ ] 定期备份数据
- [ ] 监控安全事件
- [ ] 及时应用安全补丁

### 代码审查

- [ ] 所有PR必须经过审查
- [ ] 关注安全相关的变更
- [ ] 检查权限控制逻辑
- [ ] 验证输入验证
- [ ] 审查SQL查询 (防止注入)
- [ ] 检查敏感数据处理

### 应急响应

制定安全事件响应计划：

1. **发现** - 监控和检测
2. **遏制** - 隔离受影响系统
3. **根除** - 移除威胁
4. **恢复** - 恢复正常操作
5. **总结** - 事后分析

---

## 🛡️ 安全配置检查清单

### 服务器配置

- [ ] 禁用root SSH登录
- [ ] 使用SSH密钥认证
- [ ] 配置防火墙规则
- [ ] 启用自动安全更新
- [ ] 配置fail2ban
- [ ] 设置系统日志

### 应用配置

- [ ] JWT密钥强度 ≥ 32字符
- [ ] Token过期时间 ≤ 24小时
- [ ] 启用HTTPS
- [ ] 配置CORS白名单
- [ ] 启用速率限制
- [ ] 配置审计日志

### 数据库配置

- [ ] 使用强密码
- [ ] 限制网络访问
- [ ] 启用SSL连接
- [ ] 定期备份
- [ ] 最小权限原则
- [ ] 审计日志启用

---

## 📚 相关资源

### 内部文档
- [故障排查指南](troubleshooting-guide.md)
- [部署指南](deployment/production-guide.md)
- [API文档](api/API_REFERENCE.md)

### 外部资源
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust安全指南](https://anssi-fr.github.io/rust-guide/)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [CWE Top 25](https://cwe.mitre.org/top25/)

---

**文档版本**: v1.0  
**最后更新**: 2025-11-03  
**维护团队**: AgentMem Security Team

---

## ✅ 快速参考

### 运行安全审计
```bash
./scripts/security_audit.sh
```

### 检查依赖漏洞
```bash
cargo audit
```

### 检查unsafe代码
```bash
cargo geiger
```

### 安全Lint
```bash
cargo clippy -- -W clippy::security
```

---

🔐 **安全是持续的过程，而不是一次性的任务！**

🛡️ **始终保持警惕，定期审计，及时响应！**

✅ **遵循最小权限原则，防御纵深策略！**

