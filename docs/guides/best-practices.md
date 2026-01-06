# AgentMem 最佳实践指南

**版本**: v3.0.0  
**更新日期**: 2025-12-10  
**状态**: ✅ Phase 3 已完成

---

## 📋 目录

1. [快速开始最佳实践](#快速开始最佳实践)
2. [性能优化最佳实践](#性能优化最佳实践)
3. [准确性提升最佳实践](#准确性提升最佳实践)
4. [错误处理最佳实践](#错误处理最佳实践)
5. [生产环境部署](#生产环境部署)
6. [常见问题](#常见问题)

---

## 🚀 快速开始最佳实践

### 1. 使用零配置启动

**推荐**:
```rust
// ✅ 推荐：零配置启动
let mem = Memory::new_smart().await?;
```

**不推荐**:
```rust
// ❌ 不推荐：手动配置所有选项
let mem = Memory::builder()
    .with_storage("libsql://agentmem.db")
    .with_llm_provider("openai")
    // ... 10+行配置
    .build()
    .await?;
```

### 2. 使用链式调用简化代码

**推荐**:
```rust
// ✅ 推荐：链式调用
let results = fluent
    .add("记忆1")
    .await?
    .add("记忆2")
    .await?
    .search("查询")
    .await?;
```

**不推荐**:
```rust
// ❌ 不推荐：多次调用
fluent.add("记忆1").await?;
fluent.add("记忆2").await?;
let results = fluent.search("查询").await?;
```

### 3. 利用智能默认值

**推荐**:
```rust
// ✅ 推荐：让系统自动检测
let defaults = SmartDefaults::detect().await?;
let mem = Memory::with_smart_defaults(defaults).await?;
```

---

## ⚡ 性能优化最佳实践

### 1. 批量操作

**推荐**:
```rust
// ✅ 推荐：批量添加
let memories = vec![
    "记忆1".to_string(),
    "记忆2".to_string(),
    "记忆3".to_string(),
];
mem.batch_add(memories).await?;
```

**不推荐**:
```rust
// ❌ 不推荐：循环单个添加
for content in memories {
    mem.add(content).await?; // 每次都是单独请求
}
```

### 2. 使用缓存

**推荐**:
```rust
// ✅ 推荐：启用Redis缓存
let mem = Memory::builder()
    .with_redis_cache("redis://localhost:6379")
    .build()
    .await?;
```

### 3. 合理设置批量大小

**推荐**:
```rust
// ✅ 推荐：根据数据量调整批量大小
let mem = Memory::builder()
    .with_batch_size(100) // 大量数据时使用较大批量
    .build()
    .await?;
```

---

## 🎯 准确性提升最佳实践

### 1. 使用多维度评分

**推荐**:
```rust
// ✅ 推荐：启用多维度评分
let mem = Memory::builder()
    .enable_multi_dimensional_scoring()
    .build()
    .await?;
```

### 2. 启用重排序

**推荐**:
```rust
// ✅ 推荐：启用LLM重排序
let mem = Memory::builder()
    .enable_reranking()
    .with_reranker_config(RerankerConfig {
        use_llm: true,
        cache_enabled: true,
    })
    .build()
    .await?;
```

### 3. 利用上下文增强

**推荐**:
```rust
// ✅ 推荐：启用上下文理解
let mem = Memory::builder()
    .enable_context_enhancement()
    .with_context_config(ContextConfig {
        window_expansion: true,
        multi_turn: true,
        compression: true,
    })
    .build()
    .await?;
```

### 4. 使用Persona提取

**推荐**:
```rust
// ✅ 推荐：启用Persona提取
let mem = Memory::builder()
    .enable_persona_extraction()
    .build()
    .await?;
```

---

## ⚠️ 错误处理最佳实践

### 1. 使用EnhancedError

**推荐**:
```rust
// ✅ 推荐：使用增强错误处理
match mem.add("test").await {
    Ok(_) => println!("成功"),
    Err(e) => {
        let enhanced = ErrorEnhancer::enhance(e);
        eprintln!("错误: {}", enhanced.message());
        if let Some(suggestion) = enhanced.suggestion() {
            eprintln!("建议: {}", suggestion);
        }
    }
}
```

### 2. 错误恢复

**推荐**:
```rust
// ✅ 推荐：实现重试机制
async fn add_with_retry(mem: &Memory, content: &str) -> Result<()> {
    let mut retries = 3;
    loop {
        match mem.add(content).await {
            Ok(_) => return Ok(()),
            Err(e) if retries > 0 => {
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 🏭 生产环境部署

### 1. 配置管理

**推荐**:
```rust
// ✅ 推荐：使用环境变量
// .env文件
DATABASE_URL=postgresql://localhost/agentmem
REDIS_URL=redis://localhost:6379
OPENAI_API_KEY=sk-...

// 代码中
let mem = Memory::new_smart().await?; // 自动读取环境变量
```

### 2. 监控和日志

**推荐**:
```rust
// ✅ 推荐：启用监控
let mem = Memory::builder()
    .enable_observability()
    .with_log_level(LogLevel::Info)
    .build()
    .await?;
```

### 3. 健康检查

**推荐**:
```rust
// ✅ 推荐：定期健康检查
async fn health_check(mem: &Memory) -> bool {
    match mem.health_check().await {
        Ok(status) => status.is_healthy(),
        Err(_) => false,
    }
}
```

---

## ❓ 常见问题

### Q1: 如何选择存储后端？

**A**: 
- **开发环境**: LibSQL嵌入式（零配置）
- **生产环境**: PostgreSQL（性能+可靠性）
- **大规模**: 分布式存储（PostgreSQL + 向量存储）

### Q2: 如何提升搜索准确率？

**A**:
1. 启用多维度评分
2. 启用LLM重排序
3. 使用上下文增强
4. 利用Persona提取

### Q3: 如何处理大量数据？

**A**:
1. 使用批量操作
2. 启用Redis缓存
3. 合理设置批量大小
4. 使用异步处理

### Q4: 如何优化性能？

**A**:
1. 启用多层缓存（L1 + L2）
2. 使用批量操作
3. 合理配置批量大小
4. 使用KV-cache优化

---

## 📚 相关文档

- [API参考文档](../api/api-reference-v3.md)
- [架构文档](../architecture/architecture-overview.md)
- [性能优化指南](../performance/performance-optimization.md)
- [部署指南](../deployment/deployment-guide.md)

---

**文档维护**: AgentMem Team  
**反馈**: https://github.com/louloulin/agentmem/issues

