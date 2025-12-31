# 记忆管理教程

学习如何在 AgentMem 中添加、更新、删除和管理记忆。

## 目录

- [添加记忆](#添加记忆)
- [搜索记忆](#搜索记忆)
- [更新记忆](#更新记忆)
- [删除记忆](#删除记忆)
- [批量操作](#批量操作)
- [会话管理](#会话管理)
- [常见场景](#常见场景)

## 添加记忆

### 基础添加

最简单的方式是添加文本记忆：

```rust
use agentmem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 添加简单文本记忆
    let result = memory.add("用户喜欢咖啡").await?;

    println!("记忆 ID: {}", result.id);
    println!("重要性评分: {:.2}", result.importance);

    Ok(())
}
```

### 带选项的添加

使用 `AddMemoryOptions` 进行更精细的控制：

```rust
use agentmem::{Memory, AddMemoryOptions, MemoryType};
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("category".to_string(), "preference".into());
metadata.insert("source".to_string(), "direct".into());

let options = AddMemoryOptions::builder()
    .content("用户喜欢科幻电影")
    .memory_type(MemoryType::SEMANTIC)
    .agent_id("agent_1")
    .session_id(Some("session_123"))
    .importance(0.8)
    .metadata(metadata)
    .build();

let result = memory.addWithOptions(options).await?;
```

### 添加多模态内容

```rust
use agentmem::{Memory, MemoryType, Metadata};

// 添加图像记忆
let mut metadata = Metadata::new();
metadata.insert("type", "image");
metadata.insert("url", "https://example.com/photo.jpg");
metadata.insert("caption", "用户上传的照片");

memory.add(
    "用户上传了一张照片",
    None,
    MemoryType::RESOURCE
).await?;

// 添加结构化数据
let json_data = r#"{
    "name": "张三",
    "age": 30,
    "city": "上海"
}"#;

memory.add(
    json_data,
    None,
    MemoryType::SEMANTIC
).await?;
```

### 添加到不同记忆类型

```rust
use agentmem::MemoryType;

// 核心记忆（临时状态）
memory.add(
    "用户正在浏览产品页面",
    None,
    MemoryType::CORE
).await?;

// 工作记忆（当前任务）
memory.add(
    "用户正在比较产品 A 和 B",
    Some(task_id),
    MemoryType::WORKING
).await?;

// 情节记忆（对话历史）
memory.add(
    "用户: 你好",
    Some(session_id),
    MemoryType::EPISODIC
).await?;

// 语义记忆（长期知识）
memory.add(
    "用户喜欢科幻电影",
    None,
    MemoryType::SEMANTIC
).await?;

// 程序记忆（操作流程）
memory.add(
    "处理流程: 1. 接收 2. 处理 3. 回复",
    None,
    MemoryType::PROCEDURAL
).await?;

// 资源记忆（外部引用）
memory.add(
    "用户上传了文档: requirements.pdf",
    None,
    MemoryType::RESOURCE
).await?;
```

## 搜索记忆

### 语义搜索

```rust
// 简单语义搜索
let results = memory.search("用户喜欢什么样的电影？").await?;

for (i, mem) in results.iter().enumerate() {
    println!("{}. {} (相似度: {:.2})",
        i + 1,
        mem.content,
        mem.score.unwrap_or(0.0)
    );
}
```

### 带过滤的搜索

```rust
use agentmem::{SearchOptions, MemoryType};
use std::collections::HashMap;

let mut metadata_filter = HashMap::new();
metadata_filter.insert("category".to_string(), "preference".into());

let options = SearchOptions::builder()
    .query("电影")
    .memory_type(Some(MemoryType::SEMANTIC))
    .agent_id(Some("agent_1"))
    .session_id(Some("session_123"))
    .limit(10)
    .threshold(0.7)  // 相似度阈值
    .metadata_filter(metadata_filter)
    .build();

let results = memory.searchWithOptions(options).await?;
```

### 获取所有记忆

```rust
use agentmem::{GetAllOptions, MemoryType};

// 获取所有记忆
let all = memory.get_all(GetAllOptions::default()).await?;

// 获取特定类型的记忆
let semantic = memory.get_all(
    GetAllOptions::builder()
        .memory_type(Some(MemoryType::SEMANTIC))
        .build()
).await?;

// 获取特定会话的记忆
let session_memories = memory.get_all(
    GetAllOptions::builder()
        .session_id(Some("session_123"))
        .build()
).await?;

// 分页获取
let page1 = memory.get_all(
    GetAllOptions::builder()
        .limit(10)
        .offset(0)
        .build()
).await?;
```

### 按 ID 获取

```rust
// 获取单条记忆
let mem = memory.get("memory_id").await?;

// 批量获取
let ids = vec!["id1", "id2", "id3"];
let memories = memory.get_batch(&ids).await?;
```

## 更新记忆

### 更新内容

```rust
use agentmem::UpdateMemoryOptions;

let options = UpdateMemoryOptions::builder()
    .content("用户更喜欢茶而不是咖啡")
    .importance(0.9)
    .build();

memory.update("memory_id", options).await?;
```

### 更新元数据

```rust
use std::collections::HashMap;

let mut new_metadata = HashMap::new();
new_metadata.insert("category".to_string(), "updated_preference".into());
new_metadata.insert("verified".to_string(), true.into());

let options = UpdateMemoryOptions::builder()
    .metadata(new_metadata)
    .build();

memory.update("memory_id", options).await?;
```

### 合并更新

```rust
// 智能合并（如果内容冲突，保留更新的）
memory.add(
    "用户今年31岁，刚过生日",
    None,
    MemoryType::SEMANTIC
).await?;

// AgentMem 会自动识别这是对之前"用户今年30岁"的更新
```

## 删除记忆

### 删除单条记忆

```rust
memory.delete("memory_id").await?;
```

### 条件删除

```rust
use agentmem::{DeleteAllOptions, MemoryType};

// 删除特定类型的所有记忆
memory.delete_all(
    DeleteAllOptions::builder()
        .memory_type(MemoryType::CORE)
        .build()
).await?;

// 删除特定会话的所有记忆
memory.delete_all(
    DeleteAllOptions::builder()
        .session_id("session_123")
        .build()
).await?;

// 删除特定时间之前的记忆
use chrono::{Utc, Duration};

memory.delete_all(
    DeleteAllOptions::builder()
        .before(Utc::now() - Duration::days(30))
        .build()
).await?;
```

### 批量删除

```rust
// 批量删除
let ids = vec!["id1", "id2", "id3"];
memory.delete_batch(&ids).await?;
```

## 批量操作

### 批量添加

```rust
use agentmem::{Memory, MemoryType};

let memories = vec![
    "用户喜欢咖啡",
    "用户住在上海",
    "用户是软件工程师",
];

// 批量添加（性能优化）
let results = memory.add_batch(memories, None, MemoryType::SEMANTIC).await?;

println!("成功添加 {} 条记忆", results.len());
```

### 批量更新

```rust
use std::collections::HashMap;

let updates = vec![
    ("id1", UpdateMemoryOptions::builder().content("新内容1").build()),
    ("id2", UpdateMemoryOptions::builder().content("新内容2").build()),
];

memory.update_batch(updates).await?;
```

### 事务操作

```rust
use agentmem::Transaction;

// 开始事务
let tx = memory.begin_transaction().await?;

// 执行多个操作
tx.add("记忆1", None, MemoryType::SEMANTIC).await?;
tx.add("记忆2", None, MemoryType::SEMANTIC).await?;

// 提交事务
tx.commit().await?;

// 或回滚
// tx.rollback().await?;
```

## 会话管理

### 创建会话

```rust
use agentmem::{Memory, SessionOptions};

let options = SessionOptions::builder()
    .agent_id("agent_1")
    .user_id("user_123")
    .metadata(Map::from([("title", "产品咨询"]]))
    .build();

let session = memory.create_sessionWithOptions(options).await?;

println!("会话 ID: {}", session.id);
```

### 获取会话历史

```rust
// 获取会话的所有记忆
let history = memory.get_session_history("session_id").await?;

// 获取最近的 N 条记忆
let recent = memory.get_session_history(
    "session_id",
    Some(10)  // 最近 10 条
).await?;

// 按时间范围获取
use chrono::{DateTime, Utc};

let range = memory.get_session_history_by_time(
    "session_id",
    start_time,
    end_time
).await?;
```

### 会话总结

```rust
// 生成会话总结
let summary = memory.summarize_session("session_id").await?;

println!("会话总结: {}", summary.content);
println!("关键点数量: {}", summary.key_points.len());

for point in summary.key_points {
    println!("- {}", point);
}
```

### 会话搜索

```rust
// 在特定会话中搜索
let results = memory.search_in_session(
    "session_id",
    "产品价格"
).await?;
```

## 常见场景

### 场景 1: 聊天机器人

```rust
use agentmem::{Memory, MemoryType};

async fn chatbot_example() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 创建会话
    let session = memory.create_session("user_123").await?;

    // 用户消息
    let user_msg = "你好，我想了解产品价格";
    memory.add(
        &format!("用户: {}", user_msg),
        Some(session.id),
        MemoryType::EPISODIC
    ).await?;

    // 获取相关上下文
    let context = memory.search("产品价格").await?;

    // 生成回复（这里简化为固定回复）
    let bot_reply = "我们的产品价格从 $99 到 $299 不等";
    memory.add(
        &format!("助手: {}", bot_reply),
        Some(session.id),
        MemoryType::EPISODIC
    ).await?;

    // 提取关键信息
    memory.add(
        "用户询问了产品价格",
        None,
        MemoryType::SEMANTIC
    ).await?;

    Ok(())
}
```

### 场景 2: 用户偏好管理

```rust
async fn preference_management_example() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 添加偏好
    memory.add(
        "用户喜欢科幻电影，尤其是《星际穿越》和《银翼杀手》",
        None,
        MemoryType::SEMANTIC
    ).await?;

    // 更新偏好
    memory.add(
        "用户最近开始喜欢看悬疑电影，推荐《禁闭岛》",
        None,
        MemoryType::SEMANTIC
    ).await?;

    // 获取所有偏好
    let preferences = memory.get_all(
        GetAllOptions::builder()
            .metadata_filter(Map::from([("category", "preference")]))
            .build()
    ).await?;

    println!("用户偏好:");
    for pref in preferences {
        println!("- {}", pref.content);
    }

    Ok(())
}
```

### 场景 3: 知识库管理

```rust
async fn knowledge_base_example() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 添加知识
    let knowledge = vec![
        "公司成立于2020年",
        "总部位于上海",
        "员工数量约200人",
        "主要产品是 AI 解决方案",
    ];

    for item in knowledge {
        memory.add(
            item,
            None,
            MemoryType::SEMANTIC
        ).await?;
    }

    // 搜索知识
    let query = "公司的基本信息";
    let results = memory.search(query).await?;

    println!("找到相关知识:");
    for result in results {
        println!("- {}", result.content);
    }

    Ok(())
}
```

### 场景 4: 多 Agent 协作

```rust
async fn multi_agent_example() -> Result<(), Box<dyn std::error::Error>> {
    // Agent 1: 数据收集
    let collector = Memory::builder()
        .agent_id("data_collector")
        .build()
        .await?;

    collector.add("用户提供了新需求文档").await?;

    // Agent 2: 数据分析
    let analyzer = Memory::builder()
        .agent_id("data_analyzer")
        .build()
        .await?;

    // Analyzer 访问 Collector 的数据
    let data = analyzer.get_all(
        GetAllOptions::builder()
            .agent_id("data_collector")
            .build()
    ).await?;

    for item in data {
        analyzer.add(
            &format!("分析中: {}", item.content),
            None,
            MemoryType::WORKING
        ).await?;
    }

    // Agent 3: 结果报告
    let reporter = Memory::builder()
        .agent_id("reporter")
        .build()
        .await?;

    let analysis = reporter.get_all(
        GetAllOptions::builder()
            .agent_id("data_analyzer")
            .build()
    ).await?;

    Ok(())
}
```

## 性能优化

### 1. 使用批量操作

```rust
// ❌ 慢：逐条添加
for item in items {
    memory.add(item).await?;
}

// ✅ 快：批量添加
memory.add_batch(items, None, MemoryType::SEMANTIC).await?;
```

### 2. 合理设置限制

```rust
// 限制返回数量，减少传输开销
let results = memory.search(
    SearchOptions::builder()
        .query("查询")
        .limit(10)  // 只需要前 10 条
        .build()
).await?;
```

### 3. 使用缓存

```rust
// 启用缓存（默认已启用）
let memory = Memory::builder()
    .cache_enabled(true)
    .cache_size(1000)
    .build()
    .await?;
```

### 4. 异步处理

```rust
// 对于非关键操作，使用后台任务
tokio::spawn(async move {
    memory.add("后台记录").await.unwrap();
});
```

## 最佳实践

1. **选择合适的记忆类型**：根据使用场景选择 CORE, WORKING, EPISODIC, SEMANTIC 等
2. **使用会话组织对话**：将相关记忆放在同一会话中
3. **添加结构化元数据**：便于后续查询和管理
4. **定期清理过期记忆**：避免数据膨胀
5. **使用批量操作**：提高性能
6. **处理错误**：妥善处理网络错误、存储错误等

## 下一步

- [语义搜索](semantic_search.md) - 深入了解搜索功能
- [多模态处理](multimodal.md) - 处理图像、音频等多媒体内容
- [生产部署](production.md) - 部署到生产环境
