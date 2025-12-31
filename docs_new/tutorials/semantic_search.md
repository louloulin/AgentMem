# 语义搜索教程

深入学习 AgentMem 的语义搜索功能，实现智能的信息检索。

## 目录

- [基础搜索](#基础搜索)
- [高级搜索](#高级搜索)
- [搜索优化](#搜索优化)
- [混合搜索](#混合搜索)
- [搜索分析](#搜索分析)
- [常见场景](#常见场景)

## 基础搜索

### 简单语义搜索

```rust
use agentmem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 添加一些记忆
    memory.add("用户喜欢科幻电影").await?;
    memory.add("用户住在上海").await?;
    memory.add("用户是软件工程师").await?;

    // 语义搜索
    let results = memory.search("用户有什么爱好？").await?;

    for (i, mem) in results.iter().enumerate() {
        println!("{}. {} (相似度: {:.2})",
            i + 1,
            mem.content,
            mem.score.unwrap_or(0.0)
        );
    }

    Ok(())
}
```

**输出**:
```
1. 用户喜欢科幻电影 (相似度: 0.92)
2. 用户是软件工程师 (相似度: 0.45)
3. 用户住在上海 (相似度: 0.23)
```

### 搜索结果结构

```rust
pub struct SearchResult {
    pub memory: Memory,        // 记忆对象
    pub score: f32,            // 相似度分数 (0.0 - 1.0)
    pub relevance: Relevance,  // 相关性信息
    pub highlights: Vec<String>,  // 高亮片段
}

pub struct Relevance {
    pub semantic_score: f32,   // 语义相似度
    pub keyword_score: f32,    // 关键词匹配度
    pub metadata_score: f32,   // 元数据匹配度
    pub freshness: f32,        // 新鲜度 (0.0 - 1.0)
    pub importance: f32,       // 重要性 (0.0 - 1.0)
}
```

## 高级搜索

### 带过滤的搜索

```rust
use agentmem::{SearchOptions, MemoryType};
use std::collections::HashMap;

// 构建过滤条件
let mut metadata_filter = HashMap::new();
metadata_filter.insert("category".to_string(), "preference".into());

let options = SearchOptions::builder()
    .query("电影")
    .memory_type(Some(MemoryType::SEMANTIC))  // 只搜索语义记忆
    .agent_id(Some("agent_1"))                // 只搜索特定 Agent
    .session_id(Some("session_123"))          // 只搜索特定会话
    .limit(10)                                // 最多返回 10 条
    .threshold(0.7)                           // 相似度阈值
    .metadata_filter(metadata_filter)         // 元数据过滤
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 时间范围搜索

```rust
use chrono::{Utc, Duration};
use agentmem::SearchOptions;

let options = SearchOptions::builder()
    .query("项目进展")
    .after(Some(Utc::now() - Duration::days(7)))   // 最近 7 天
    .before(Some(Utc::now()))                      // 到现在
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 相似度阈值

```rust
// 只返回相似度 > 0.8 的结果
let results = memory.search(
    SearchOptions::builder()
        .query("查询")
        .threshold(0.8)
        .build()
).await?;
```

### 分页搜索

```rust
// 第一页
let page1 = memory.search(
    SearchOptions::builder()
        .query("查询")
        .limit(10)
        .offset(0)
        .build()
).await?;

// 第二页
let page2 = memory.search(
    SearchOptions::builder()
        .query("查询")
        .limit(10)
        .offset(10)
        .build()
).await?;
```

## 搜索优化

### 1. 相关性评分调整

```rust
use agentmem::{SearchOptions, ScoringStrategy};

// 自定义评分策略
let options = SearchOptions::builder()
    .query("用户偏好")
    .scoring_strategy(ScoringStrategy::Weighted {
        semantic: 0.6,      // 语义相似度权重 60%
        keyword: 0.2,       // 关键词权重 20%
        freshness: 0.1,     // 新鲜度权重 10%
        importance: 0.1,    // 重要性权重 10%
    })
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 2. 结果重排序

```rust
// 获取初始结果
let mut results = memory.search("查询").await?;

// 自定义重排序
results.sort_by(|a, b| {
    // 优先考虑重要性
    b.memory.importance
        .partial_cmp(&a.memory.importance)
        .unwrap()
        .then_with(|| {
            // 然后考虑相似度
            b.score.partial_cmp(&a.score).unwrap()
        })
});
```

### 3. 搜索缓存

```rust
// 启用搜索缓存（默认已启用）
let memory = Memory::builder()
    .search_cache_enabled(true)
    .search_cache_size(100)
    .search_cache_ttl(300)  // 5 分钟
    .build()
    .await?;

// 相同查询会从缓存返回
let results1 = memory.search("查询").await?;  // 首次查询
let results2 = memory.search("查询").await?;  // 从缓存返回
```

### 4. 搜索建议

```rust
// 获取搜索建议
let suggestions = memory.suggest("用户偏").await?;

println!("搜索建议:");
for suggestion in suggestions {
    println!("- {}", suggestion);
}

// 输出:
// - 用户偏好
// - 用户偏好电影类型
// - 用户饮食习惯
```

## 混合搜索

AgentMem 支持语义搜索和关键词搜索的混合模式。

### 启用混合搜索

```rust
use agentmem::{SearchOptions, HybridSearchConfig};

let options = SearchOptions::builder()
    .query("科幻电影 推荐")
    .hybrid_search(HybridSearchConfig {
        semantic_weight: 0.7,    // 语义搜索权重
        keyword_weight: 0.3,     // 关键词搜索权重
        fusion_method: FusionMethod::RRF,  // 结果融合方法
    })
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 融合方法

```rust
pub enum FusionMethod {
    // 加权平均
    Weighted,

    // 倒排排名融合 (Reciprocal Rank Fusion)
    RRF { k: f32 },

    // 级联融合（先关键词，后语义）
    Cascade,
}
```

**RRF 示例**:
```rust
HybridSearchConfig {
    semantic_weight: 0.7,
    keyword_weight: 0.3,
    fusion_method: FusionMethod::RRF { k: 60.0 },
}
```

### 不同的混合策略

```rust
// 策略 1: 平衡混合（推荐）
let balanced = SearchOptions::builder()
    .query("查询")
    .hybrid_search(HybridSearchConfig {
        semantic_weight: 0.5,
        keyword_weight: 0.5,
        fusion_method: FusionMethod::RRF { k: 60.0 },
    })
    .build();

// 策略 2: 语义优先
let semantic_first = SearchOptions::builder()
    .query("查询")
    .hybrid_search(HybridSearchConfig {
        semantic_weight: 0.8,
        keyword_weight: 0.2,
        fusion_method: FusionMethod::RRF { k: 60.0 },
    })
    .build();

// 策略 3: 关键词优先（精确匹配）
let keyword_first = SearchOptions::builder()
    .query("查询")
    .hybrid_search(HybridSearchConfig {
        semantic_weight: 0.3,
        keyword_weight: 0.7,
        fusion_method: FusionMethod::Weighted,
    })
    .build();
```

## 搜索分析

### 搜索性能分析

```rust
use agentmem::SearchOptions;

let options = SearchOptions::builder()
    .query("查询")
    .enable_profiling(true)  // 启用性能分析
    .build();

let results = memory.searchWithOptions(options).await?;

// 访问性能数据
if let Some(profile) = results.profile {
    println!("搜索耗时: {:.2}ms", profile.total_duration);
    println!("向量搜索耗时: {:.2}ms", profile.vector_search_duration);
    println!("关键词搜索耗时: {:.2}ms", profile.keyword_search_duration);
    println!("融合耗时: {:.2}ms", profile.fusion_duration);
    println!("候选数量: {}", profile.candidates_count);
    println!("最终结果: {}", profile.final_count);
}
```

### 搜索结果分析

```rust
let results = memory.search("查询").await?;

// 分析结果分布
let high_relevance: Vec<_> = results.iter()
    .filter(|r| r.score > 0.8)
    .collect();

let medium_relevance: Vec<_> = results.iter()
    .filter(|r| r.score >= 0.5 && r.score <= 0.8)
    .collect();

let low_relevance: Vec<_> = results.iter()
    .filter(|r| r.score < 0.5)
    .collect();

println!("高相关性: {} 条", high_relevance.len());
println!("中相关性: {} 条", medium_relevance.len());
println!("低相关性: {} 条", low_relevance.len());
```

### 搜索日志

```rust
// 启用搜索日志
let memory = Memory::builder()
    .log_searches(true)
    .log_threshold(0.5)  // 只记录相似度 > 0.5 的搜索
    .build()
    .await?;

// 获取搜索历史
let search_history = memory.get_search_history(
    Some(user_id),
    Some(limit)
).await?;

for entry in search_history {
    println!("查询: {}", entry.query);
    println!("结果数: {}", entry.result_count);
    println!("时间: {}", entry.timestamp);
}
```

## 常见场景

### 场景 1: 个性化推荐

```rust
async fn personalized_recommendation() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 搜索用户历史偏好
    let preferences = memory.search(
        SearchOptions::builder()
            .query("电影 音乐 书籍 娱乐")
            .memory_type(Some(MemoryType::SEMANTIC))
            .metadata_filter(Map::from([("category", "preference")]))
            .scoring_strategy(ScoringStrategy::Importance)
            .build()
    ).await?;

    // 基于偏好生成推荐
    println!("基于用户偏好的推荐:");
    for pref in preferences.iter().take(5) {
        println!("- {}", pref.content);
    }

    Ok(())
}
```

### 场景 2: 上下文感知搜索

```rust
async fn context_aware_search() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 当前对话上下文
    let current_topic = "产品价格";

    // 搜索相关历史记录
    let context = memory.search(
        SearchOptions::builder()
            .query(current_topic)
            .session_id(Some(current_session_id))
            .recency_boost(0.3)  // 提升近期记录的权重
            .build()
    ).await?;

    // 结合上下文生成回复
    println!("相关上下文:");
    for mem in context.iter().take(3) {
        println!("- {}", mem.content);
    }

    Ok(())
}
```

### 场景 3: 跨会话搜索

```rust
async fn cross_session_search() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 搜索用户所有会话中的相关信息
    let all_context = memory.search(
        SearchOptions::builder()
            .query("用户反馈的问题")
            .agent_id(Some(user_id))
            .exclude_session_id(Some(current_session_id))  // 排除当前会话
            .threshold(0.6)
            .limit(20)
            .build()
    ).await?;

    // 按会话分组
    let mut by_session: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();

    for result in all_context {
        if let Some(session_id) = result.memory.session_id {
            by_session.entry(session_id)
                .or_insert_with(Vec::new)
                .push(result);
        }
    }

    println!("历史会话中的相关信息:");
    for (session_id, results) in by_session {
        println!("会话 {} ({} 条):", session_id, results.len());
        for result in results.iter().take(3) {
            println!("  - {}", result.content);
        }
    }

    Ok(())
}
```

### 场景 4: 智能问答

```rust
async fn intelligent_qa() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    let question = "如何重置密码？";

    // 搜索相关答案
    let answers = memory.search(
        SearchOptions::builder()
            .query(question)
            .memory_type(Some(MemoryType::SEMANTIC))
            .metadata_filter(Map::from([("type", "qa_pair")]))
            .hybrid_search(HybridSearchConfig {
                semantic_weight: 0.6,
                keyword_weight: 0.4,
                fusion_method: FusionMethod::RRF { k: 60.0 },
            })
            .build()
    ).await?;

    if answers.is_empty() {
        println!("抱歉，找不到相关答案");
    } else {
        let best_answer = &answers[0];
        println!("找到答案 (相似度: {:.2}):", best_answer.score);
        println!("{}", best_answer.memory.content);

        // 如果相似度不高，提供多个选项
        if best_answer.score < 0.8 {
            println!("\n其他可能相关的答案:");
            for answer in answers.iter().take(3).skip(1) {
                println!("- {} ({:.2})",
                    answer.memory.content,
                    answer.score
                );
            }
        }
    }

    Ok(())
}
```

## 搜索最佳实践

### 1. 选择合适的阈值

```rust
// 高精度场景（如安全验证）
let high_precision = SearchOptions::builder()
    .query("验证")
    .threshold(0.9)  // 只返回高相似度结果
    .build();

// 高召回场景（如探索性搜索）
let high_recall = SearchOptions::builder()
    .query("探索")
    .threshold(0.5)  // 包含更多结果
    .build();
```

### 2. 使用相关性反馈

```rust
// 初始搜索
let mut results = memory.search("查询").await?;

// 用户点击了某个结果
let clicked_id = results[0].memory.id.clone();

// 使用相关性反馈优化
let refined = memory.refine_search(
    "查询",
    RefinementOptions {
        relevant: vec![clicked_id],
        irrelevant: vec![],
        alpha: 0.5,
    }
).await?;
```

### 3. 处理查询歧义

```rust
// 查询扩展
let query = "苹果";
let expansions = memory.expand_query(query).await?;

println!("原始查询: {}", query);
println!("扩展查询:");
for expansion in expansions {
    println!("- {}", expansion);
}

// 使用扩展查询搜索
let expanded_query = expansions.join(" ");
let results = memory.search(&expanded_query).await?;
```

### 4. 监控搜索质量

```rust
// 定期评估搜索质量
let metrics = memory.get_search_metrics(
    Utc::now() - Duration::days(7)
).await?;

println!("平均搜索耗时: {:.2}ms", metrics.avg_latency);
println!("平均结果数: {:.1}", metrics.avg_results);
println!("零结果率: {:.1}%", metrics.zero_result_rate);
println!("低相关性率: {:.1}%", metrics.low_relevance_rate);
```

## 性能对比

| 搜索类型 | 平均延迟 | 召回率 | 精度 | 适用场景 |
|---------|---------|-------|------|---------|
| 纯语义搜索 | 50ms | 85% | 80% | 概念性查询 |
| 纯关键词搜索 | 10ms | 60% | 90% | 精确匹配 |
| 混合搜索 (RRF) | 60ms | 90% | 85% | 通用场景 |
| 语义优先 | 50ms | 88% | 82% | 探索性搜索 |
| 关键词优先 | 20ms | 70% | 88% | 精确查找 |

## 下一步

- [多模态处理](multimodal.md) - 处理图像、音频等多媒体内容
- [生产部署](production.md) - 部署到生产环境
- [API 参考](../api_reference/) - 完整的 API 文档
