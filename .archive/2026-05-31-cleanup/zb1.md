# AgentMem 关键性能指标

## 1. 并发与连接

| 参数 | Dev | Full | Prod |
|------|-----|------|------|
| workers | 4 | 8 | 8 |
| max_connections | 10 | 20 | 50 |
| connection_timeout | 30s | 30s | 30s |
| idle_timeout | 600s | 600s | 300s |

## 2. 搜索参数

| 参数 | 默认值 |
|------|--------|
| vector_weight | 0.7 |
| fulltext_weight | 0.3 |
| rrf_k | 60.0 |
| default_threshold | 0.3 |
| max_results | 10 |
| timeout_seconds | 30 |
| dimension | 768 |

## 3. 批处理

| 参数 | Dev | Full |
|------|-----|------|
| batch_size | 32 | 100 |

## 4. 缓存

| 参数 | 默认值 |
|------|--------|
| cache_size | 512MB |
| cache_ttl_seconds | 3600 |

## 5. 数据库

| 参数 | Dev | Full | Prod |
|------|-----|------|------|
| cache_size_kb | 10240 | 20480 | 51200 |
| page_size | 4096 | 4096 | 4096 |

## 6. 重要性权重

| 维度 | 权重 |
|------|------|
| recency | 0.25 |
| frequency | 0.20 |
| relevance | 0.25 |
| emotional | 0.15 |
| context | 0.10 |
| interaction | 0.05 |

## 7. 监控指标

- average_latency, p95_latency, p99_latency
- throughput, QPS
- cache_hit_rate, error_rate

---

# AI Agent 记忆指标

## 1. 三层记忆权重

| 记忆层 | 权重 | 说明 |
|--------|------|------|
| episodic | 1.2 | 长期记忆优先 |
| working | 1.0 | 上下文补充 |
| semantic | 0.9 | 广泛范围备份 |

## 2. 智能配置

| 参数 | 默认值 |
|------|--------|
| conflict_sensitivity | 0.8 |
| auto_resolution_threshold | 0.9 |
| max_memories | 10 |
| relevance_threshold | 0.1 |

## 3. 压缩配置

| 参数 | 默认值 |
|------|--------|
| enable_compression | true |
| min_importance_threshold | 0.3 |
| target_compression_ratio | 0.7 |
| semantic_similarity_threshold | 0.85 |
| temporal_decay_factor | 0.95 |

## 4. 自适应阈值

| 查询类型 | 阈值 |
|----------|------|
| ExactId | 0.0 |
| ShortKeyword | 0.1 |
| NaturalLanguage | 0.3 |
| Semantic | 0.5 |
| Temporal | 0.0 |

## 5. LLM 参数

| 参数 | 默认值 |
|------|--------|
| max_tokens | 512 |
| temperature | 0.7 |
| model | glm-4.6 |

## 6. 嵌入配置

| 参数 | 默认值 |
|------|--------|
| embedder_model | text-embedding-3-small |
| dimension | 768/1536 |
| enable_parallel | true |

## 7. 速率限制

| 参数 | 默认值 |
|------|--------|
| rate_limit_per_minute | 1000 |
| burst_size | 200 |

---

# 效果指标

## 1. 检索质量

| 指标 | 目标值 | 说明 |
|------|--------|------|
| estimated_recall | ≥0.95 | 预估召回率 |
| recall_rate | ≥0.95 | 召回率 |
| precision_rate | ≥0.92 | 精确率 |
| accuracy | ≥0.85 | 检索准确率 |
| F1_score | ≥0.65 | 平衡指标 |

## 2. 缓存效果

| 指标 | 目标值 | 说明 |
|------|--------|------|
| cache_hit_rate | >70-80% | 缓存命中率 |
| embedding_cache_hit_rate | >80% | 嵌入缓存命中率 |
| search_cache_hit_rate | >70% | 搜索缓存命中率 |
| miss_rate | <20% | 未命中率 |

## 3. 延迟指标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| avg_response_time | <100ms | 平均响应时间 |
| p95_latency | <200ms | P95延迟 |
| p99_latency | <500ms | P99延迟 |
| category_recall_time | <50ms | 类别召回时间 |
| item_recall_time | <100ms | 项目召回时间 |

## 4. HNSW 索引参数

| 指标 | 目标值 | 说明 |
|------|--------|------|
| ef_search | 200 | 搜索精度参数 |
| m | 16 | 边连接数 |
| ef_construction | 200 | 构建精度 |

## 5. 量化精度

| 精度 | 说明 |
|------|------|
| FP32 | 全精度 |
| FP16 | 半精度 |
| BF16 | BFLOAT16 |
| INT8 | 8位整数 |

## 6. 警报阈值

| 指标 | 警告 | 严重 |
|------|------|------|
| cache_hit_rate | <70% | <50% |
| avg_latency | >200ms | >500ms |
| error_rate | >1% | >5% |

---

# AI Agent 记忆效果核心指标 ⭐

## 1. 记忆检索质量

| 指标 | 目标值 | 说明 |
|------|--------|------|
| **relevance_score** | ≥0.8 | 记忆相关性得分 |
| **context_relevance** | ≥0.85 | 上下文相关性 |
| **memory_recall_rate** | ≥95% | 记忆召回率 |
| **information_retention** | ≥80% | 信息保留率 |

## 2. 记忆遗忘曲线 (Ebbinghaus)

| 指标 | 目标值 | 说明 |
|------|--------|------|
| **retention_rate** | >36.8% | 1天后保留率 |
| **forgetting_threshold** | 10% | 遗忘阈值 |
| **retention_period** | 30天 | 默认保留期 |
| **memory_strength** | 可调 | 记忆强度 |

## 3. 记忆保护级别

| 级别 | 延迟倍数 | 说明 |
|------|----------|------|
| Low | 2x | 轻度保护 |
| Medium | 5x | 中度保护 |
| High | 10x | 高度保护 |
| Permanent | ∞ | 永久保护 |

## 4. 记忆重要性评分

| 维度 | 权重 | 说明 |
|------|------|------|
| base_importance | 0.6 | 基础重要性 |
| semantic_score | 0.4 | 语义得分 |
| recency_weight | 0.25 | 时效性权重 |
| frequency_weight | 0.20 | 访问频率权重 |
| relevance_weight | 0.25 | 相关性权重 |
| emotional_weight | 0.15 | 情感权重 |
| context_weight | 0.10 | 上下文权重 |

## 5. 上下文增强

| 指标 | 默认值 | 说明 |
|------|--------|------|
| context_relevance_weight | 0.2 | 上下文权重 |
| enable_context_relevance | false | 启用上下文相关性 |

## 6. 记忆质量统计

| 指标 | 说明 |
|------|------|
| memory_quality_score | 记忆质量综合分 |
| synthesis_confidence | 合成置信度 |
| avg_relevance_score | 平均相关性得分 |

## 7. 记忆持久化

| 指标 | 默认值 | 说明 |
|------|--------|------|
| data_retention_days | 365 | 数据保留天数 |
| backup_retention_days | 30 | 备份保留天数 |
| metrics_retention_hours | 24 | 指标保留小时 |

## 8. 重写保留

| 指标 | 默认值 | 说明 |
|------|--------|------|
| rewrite_retention_ratio | 0.7 | 重写时内容保留比例 |
| retention_ratio | 0.8 | 默认保留比例 |

## 9. 记忆检索评分

```rust
final_score = relevance_score * time_decay * user_match_boost * (0.5 + 0.5 * importance)
```

## 10. 遗忘曲线公式

```
R(t) = e^(-t/S)
- R: 保留率
- t: 时间单位
- S: 记忆强度(保留率降至36.8%所需时间)
```
