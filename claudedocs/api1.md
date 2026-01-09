# AgentMem 2.6 API 统一重构计划

**制定日期**: 2025-01-08
**版本**: 1.0
**优先级**: 🔴 P0 - 关键改造
**预期时间**: 2-3 周
**当前状态**: 🔄 规划中

---

## 📊 执行摘要

### 问题诊断

经过深度分析，AgentMem 2.6 的核心问题不是架构缺陷，而是 **API 设计混乱**：

#### 🔴 当前状态：API 爆炸

```bash
# 统计结果
$ grep -r "pub async fn\|pub fn" crates/agent-mem/src/orchestrator/ | grep -v test | wc -l
103  # 103 个公共方法！
```

#### 问题症状

| 症状 | 影响 | 严重程度 |
|------|------|---------|
| **API 表面积过大** | 用户学习曲线陡峭 | 🔴 高 |
| **功能重叠** | 不知道用哪个方法 | 🔴 高 |
| **命名不一致** | `add_memory` vs `add_memory_fast` vs `add_memory_v2` | 🔴 高 |
| **参数混乱** | 相似功能参数不同 | 🟡 中 |
| **文档示例无法运行** | 用户体验极差 | 🔴 高 |

### 🎯 解决方案：统一 API 设计

#### 核心原则

1. **简洁性**: 从 103 个方法减少到 ~30 个核心方法
2. **一致性**: 统一命名规范和参数模式
3. **可发现性**: Builder 模式让 API 自解释
4. **向后兼容**: 旧 API 标记废弃，逐步迁移

#### 设计目标

```
当前: 103 个公共方法 → 目标: ~30 个核心方法
当前: 功能分散混乱 → 目标: 清晰的模块化 API
当前: 学习成本高 → 目标: 5 分钟上手
当前: 示例无法运行 → 目标: 100% 可运行示例
```

---

## 🔴 第一部分：问题详细分析

### 1.1 当前 API 混乱示例

#### ❌ 问题 1：记忆添加 API 混乱

```rust
// 当前有 4 个添加记忆的方法，用户不知道用哪个：
pub async fn add_memory_fast(...)     // 快速添加？
pub async fn add_memory(...)          // 正常添加？
pub async fn add_memory_v2(...)       // v2 是什么？
pub async fn add_memory_intelligent(...) // 智能添加？

// 批量添加还有 2 个：
pub async fn add_memories_batch(...)
pub async fn add_memory_batch_optimized(...)
```

**问题**:
- 用户困惑：到底用哪个？
- 功能重叠：4 个方法做类似的事
- 命名不清：`fast`, `v2`, `intelligent` 含义模糊

#### ❌ 问题 2：搜索 API 混乱

```rust
// 当前有 3 个搜索方法：
pub async fn search_memories(...)                // 基础搜索
pub async fn search_memories_hybrid(...)         // 混合搜索？
pub async fn context_aware_rerank(...)           // 上下文重排序？

// 还有缓存的搜索：
pub async fn cached_search(...)                  // 带缓存的搜索
```

**问题**:
- `search_memories` vs `search_memories_hybrid` 有什么区别？
- `context_aware_rerank` 是搜索还是后处理？
- 用户不知道何时用哪个

#### ❌ 问题 3：API 命名不一致

```rust
// 不同的命名风格：
add_memory_fast        // 描述性后缀
add_memory_v2          // 版本号后缀
get_all_memories       // all 前缀
get_all_memories_v2    // all + 版本号
delete_all_memories    // all 前缀
add_memories_batch     // batch 后缀
add_memory_batch_optimized  // batch + 描述性
```

**问题**:
- 没有统一的命名规范
- 后缀使用不一致
- 版本号 (v2) 混在功能名称中

### 1.2 代码质量问题

#### 🟡 过度使用 clone()

```bash
$ grep -r "clone()" crates/agent-mem/src/orchestrator/ | wc -l
185  # 185 次 clone 调用！
```

**示例问题代码**:
```rust
// intelligence.rs:146-168
let evaluation_tasks: Vec<_> = structured_facts
    .iter()
    .map(|fact| {
        let fact_clone = fact.clone();  // ❌ 不必要的 clone
        let agent_id_clone = agent_id.to_string();  // ❌ 每次都创建新 String
        let user_id_clone = user_id.clone();  // ❌ Option<String> clone
        let evaluator_ref = evaluator.clone();  // ⚠️ Arc clone 可以但频繁

        async move {
            // 使用克隆的数据
        }
    })
    .collect();
```

#### 🟡 错误处理不一致

```bash
$ find crates -name "*.rs" -type f -exec grep -l "unwrap()\|expect(" {} \; | wc -l
383  # 383 个文件包含 unwrap 或 expect！
```

**示例问题代码**:
```rust
// ❌ 不安全的 unwrap
let memory_id = memory.id.unwrap();  // panic if None

// ❌ 不安全的 expect
let config = config.expect("Config must be set");  // panic if None
```

### 1.3 公共 API 统计

#### 按功能分类的公共方法数量

| 功能模块 | 方法数量 | 问题 |
|---------|---------|------|
| **记忆添加** | 8 个 | 功能重叠，命名混乱 |
| **记忆查询** | 6 个 | `search`, `get`, `retrieve` 不一致 |
| **记忆更新** | 2 个 | 功能重复 |
| **记忆删除** | 3 个 | `delete`, `remove` 混用 |
| **批量操作** | 4 个 | 优化版本过多 |
| **多模态** | 4 个 | API 设计不一致 |
| **工具函数** | 15+ 个 | 应该是内部 API |
| **初始化** | 10+ 个 | 过度暴露 |

**总计**: ~52 个功能方法 + ~51 个工具/初始化方法 = **103 个公共方法**

---

## 🎯 第二部分：统一 API 设计

### 2.1 设计原则

#### 核心设计哲学

1. **少即是多**: 减少到核心功能，通过组合实现复杂需求
2. **一致性**: 统一的命名、参数、返回值
3. **可组合性**: 小的、专注的函数可以组合使用
4. **可扩展性**: 通过 trait 和 builder 支持高级用法
5. **向后兼容**: 旧 API 标记 `#[deprecated]`，保持可用

#### 命名规范

```rust
// ✅ 统一的命名规范
add()              // 添加单个
add_batch()        // 添加批量
search()           // 搜索（统一入口）
get()              // 获取单个
get_all()          // 获取全部
update()           // 更新
delete()           // 删除
```

### 2.2 新 API 架构

#### 核心模块划分

```rust
// 核心模块
pub mod memory;    // 记忆管理
pub mod search;    // 搜索功能
pub mod batch;     // 批量操作
pub mod analytics; // 分析统计

// 内部模块（不暴露）
mod storage;       // 存储层
mod retrieval;     // 检索层
mod intelligence;  // 智能处理
```

#### API 层次结构

```
┌─────────────────────────────────────┐
│     用户 API 层 (公开)               │
│  - MemoryOrchestrator               │
│  - SearchBuilder                    │
│  - BatchBuilder                    │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     业务逻辑层 (内部)                │
│  - MemoryModule                    │
│  - SearchModule                    │
│  - BatchModule                     │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     存储抽象层 (trait)              │
│  - CoreMemoryStore                 │
│  - VectorStore                     │
│  - GraphStore                      │
└─────────────────────────────────────┘
```

### 2.3 核心 API 设计

#### 记忆管理 API

```rust
impl MemoryOrchestrator {
    // ✅ 统一的添加 API
    /// 添加记忆（智能处理，自动选择最佳策略）
    pub async fn add(&self, content: &str) -> Result<String> {
        // 自动使用智能添加：事实提取、重要性评估、冲突检测
        self.add_memory_intelligent(content).await
    }

    /// 批量添加记忆
    pub async fn add_batch(&self, contents: Vec<String>) -> Result<Vec<String>> {
        // 使用优化的批量添加
        self.add_memory_batch_optimized(contents).await
    }

    /// 多模态记忆（图片）
    pub async fn add_image(&self, image: Vec<u8>, caption: Option<&str>) -> Result<String> {
        self.add_image_memory(image, caption).await
    }

    /// 多模态记忆（音频）
    pub async fn add_audio(&self, audio: Vec<u8>, transcript: Option<&str>) -> Result<String> {
        self.add_audio_memory(audio, transcript).await
    }

    /// 多模态记忆（视频）
    pub async fn add_video(&self, video: Vec<u8>, description: Option<&str>) -> Result<String> {
        self.add_video_memory(video, description).await
    }

    // ✅ 统一的查询 API
    /// 获取单个记忆
    pub async fn get(&self, id: &str) -> Result<MemoryItem> {
        self.get_memory(id).await
    }

    /// 获取所有记忆
    pub async fn get_all(&self) -> Result<Vec<MemoryItem>> {
        self.get_all_memories_v2().await
    }

    // ✅ 统一的更新 API
    /// 更新记忆
    pub async fn update(&self, id: &str, content: &str) -> Result<()> {
        self.update_memory(id, content).await
    }

    // ✅ 统一的删除 API
    /// 删除单个记忆
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.delete_memory(id).await
    }

    /// 删除所有记忆
    pub async fn delete_all(&self) -> Result<()> {
        self.delete_all_memories().await
    }

    /// 重置系统
    pub async fn reset(&self) -> Result<()> {
        self.reset().await
    }
}
```

#### 搜索 API 设计

```rust
use crate::search::{SearchOptions, SearchBuilder};

impl MemoryOrchestrator {
    // ✅ 统一的搜索入口
    /// 搜索记忆（使用默认配置）
    pub async fn search(&self, query: &str) -> Result<Vec<MemoryItem>> {
        SearchBuilder::new(self, query)
            .execute()
            .await
    }

    /// 搜索记忆（返回 builder 进行配置）
    pub fn search_builder(&self, query: &str) -> SearchBuilder {
        SearchBuilder::new(self, query)
    }
}

// Builder 模式实现
pub struct SearchBuilder<'a> {
    orchestrator: &'a MemoryOrchestrator,
    query: String,
    options: SearchOptions,
}

pub struct SearchOptions {
    /// 返回结果数量
    pub limit: usize,

    /// 启用混合搜索（向量 + 全文）
    pub enable_hybrid: bool,

    /// 启用上下文感知重排序
    pub enable_rerank: bool,

    /// 启用记忆调度（智能选择）
    pub enable_scheduler: bool,

    /// 相似度阈值
    pub threshold: Option<f32>,

    /// 时间范围过滤
    pub time_range: Option<(i64, i64)>,

    /// 自定义过滤器
    pub filters: HashMap<String, String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            enable_hybrid: true,
            enable_rerank: true,
            enable_scheduler: true,
            threshold: None,
            time_range: None,
            filters: HashMap::new(),
        }
    }
}

impl<'a> SearchBuilder<'a> {
    pub fn new(orchestrator: &'a MemoryOrchestrator, query: &str) -> Self {
        Self {
            orchestrator,
            query: query.to_string(),
            options: SearchOptions::default(),
        }
    }

    /// 设置返回结果数量
    pub fn limit(mut self, limit: usize) -> Self {
        self.options.limit = limit;
        self
    }

    /// 启用/禁用混合搜索
    pub fn with_hybrid(mut self, enable: bool) -> Self {
        self.options.enable_hybrid = enable;
        self
    }

    /// 启用/禁用重排序
    pub fn with_rerank(mut self, enable: bool) -> Self {
        self.options.enable_rerank = enable;
        self
    }

    /// 启用/禁用记忆调度
    pub fn with_scheduler(mut self, enable: bool) -> Self {
        self.options.enable_scheduler = enable;
        self
    }

    /// 设置相似度阈值
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.options.threshold = Some(threshold);
        self
    }

    /// 设置时间范围
    pub fn with_time_range(mut self, start: i64, end: i64) -> Self {
        self.options.time_range = Some((start, end));
        self
    }

    /// 添加自定义过滤器
    pub fn with_filter(mut self, key: String, value: String) -> Self {
        self.options.filters.insert(key, value);
        self
    }

    /// 执行搜索
    pub async fn execute(self) -> Result<Vec<MemoryItem>> {
        // 根据配置执行搜索
        let mut results = if self.options.enable_hybrid {
            self.orchestrator
                .search_memories_hybrid(
                    &self.query,
                    self.options.limit,
                    self.options.threshold,
                )
                .await?
        } else {
            self.orchestrator
                .search_memories(&self.query, self.options.limit)
                .await?
        };

        // 应用重排序
        if self.options.enable_rerank {
            results = self
                .orchestrator
                .context_aware_rerank(&self.query, results, self.options.limit)
                .await?;
        }

        // TODO: 应用记忆调度
        // if self.options.enable_scheduler { ... }

        // TODO: 应用时间范围过滤
        // if let Some((start, end)) = self.options.time_range { ... }

        // TODO: 应用自定义过滤器
        // if !self.options.filters.is_empty() { ... }

        Ok(results)
    }

    // 实现 Future，允许直接 await
    // use std::future::IntoFuture;
    // impl<'a> IntoFuture for SearchBuilder<'a> { ... }
}
```

#### 批量操作 API

```rust
use crate::batch::BatchBuilder;

impl MemoryOrchestrator {
    /// 批量添加（返回 builder）
    pub fn batch_add(&self) -> BatchBuilder {
        BatchBuilder::new(self)
    }
}

pub struct BatchBuilder<'a> {
    orchestrator: &'a MemoryOrchestrator,
    contents: Vec<String>,
    options: BatchOptions,
}

pub struct BatchOptions {
    /// 批量大小
    pub batch_size: usize,

    /// 并发数
    pub concurrency: usize,

    /// 启用智能处理
    pub enable_intelligent: bool,

    /// 启用冲突检测
    pub enable_conflict_detection: bool,
}

impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            batch_size: 100,
            concurrency: 10,
            enable_intelligent: true,
            enable_conflict_detection: true,
        }
    }
}

impl<'a> BatchBuilder<'a> {
    pub fn new(orchestrator: &'a MemoryOrchestrator) -> Self {
        Self {
            orchestrator,
            contents: Vec::new(),
            options: BatchOptions::default(),
        }
    }

    /// 添加内容
    pub fn add(mut self, content: String) -> Self {
        self.contents.push(content);
        self
    }

    /// 添加多个内容
    pub fn add_all(mut self, contents: Vec<String>) -> Self {
        self.contents.extend(contents);
        self
    }

    /// 设置批量大小
    pub fn batch_size(mut self, size: usize) -> Self {
        self.options.batch_size = size;
        self
    }

    /// 设置并发数
    pub fn concurrency(mut self, n: usize) -> Self {
        self.options.concurrency = n;
        self
    }

    /// 禁用智能处理
    pub fn without_intelligent(mut self) -> Self {
        self.options.enable_intelligent = false;
        self
    }

    /// 禁用冲突检测
    pub fn without_conflict_detection(mut self) -> Self {
        self.options.enable_conflict_detection = false;
        self
    }

    /// 执行批量添加
    pub async fn execute(self) -> Result<Vec<String>> {
        self.orchestrator
            .add_memory_batch_optimized_with_options(
                self.contents,
                self.options,
            )
            .await
    }
}
```

#### 分析统计 API

```rust
impl MemoryOrchestrator {
    /// 获取统计信息
    pub async fn stats(&self) -> Result<MemoryStats> {
        self.get_stats(None).await
    }

    /// 获取性能统计
    pub async fn performance_stats(&self) -> Result<PerformanceStats> {
        self.get_performance_stats().await
    }

    /// 获取历史记录
    pub async fn history(&self, memory_id: &str) -> Result<Vec<HistoryEntry>> {
        self.get_history(memory_id).await
    }
}
```

### 2.4 使用示例对比

#### ❌ 旧 API（混乱）

```rust
// 用户困惑：到底用哪个？
let id1 = orchestrator.add_memory_fast("content").await?;
let id2 = orchestrator.add_memory("content").await?;
let id3 = orchestrator.add_memory_v2("content").await?;
let id4 = orchestrator.add_memory_intelligent("content").await?;

// 搜索也很混乱
let results1 = orchestrator.search_memories("query", 10).await?;
let results2 = orchestrator.search_memories_hybrid("query", 10, None).await?;
let results3 = orchestrator.context_aware_rerank("query", results1, 10).await?;

// 批量添加
let ids = orchestrator.add_memories_batch(contents).await?;
// 或者
let ids = orchestrator.add_memory_batch_optimized(contents).await?;
```

#### ✅ 新 API（清晰）

```rust
// 简单直观
let id = orchestrator.add("content").await?;

// 搜索同样简单
let results = orchestrator.search("query").await?;

// 高级用法：Builder 模式
let results = orchestrator
    .search_builder("query")
    .limit(20)
    .with_rerank(true)
    .with_threshold(0.7)
    .with_time_range(start, end)
    .execute()
    .await?;

// 批量添加
let ids = orchestrator
    .batch_add()
    .add_all(contents)
    .batch_size(50)
    .concurrency(5)
    .execute()
    .await?;
```

---

## 📋 第三部分：实施计划

### 3.1 实施阶段

#### 阶段 1：准备阶段（2-3 天）

**任务清单**:

- [ ] 创建新的模块结构
  - [ ] `crates/agent-mem/src/search/mod.rs`
  - [ ] `crates/agent-mem/src/search/types.rs`
  - [ ] `crates/agent-mem/src/search/implementation.rs`
  - [ ] `crates/agent-mem/src/batch/mod.rs`
  - [ ] `crates/agent-mem/src/batch/types.rs`
  - [ ] `crates/agent-mem/src/batch/implementation.rs`
  - [ ] `crates/agent-mem/src/analytics/mod.rs`

- [ ] 编写核心类型定义
  - [ ] `SearchOptions`
  - [ ] `SearchBuilder`
  - [ ] `BatchOptions`
  - [ ] `BatchBuilder`

- [ ] 编写单元测试框架
  - [ ] 搜索功能测试
  - [ ] 批量操作测试
  - [ ] 向后兼容性测试

#### 阶段 2：实现新 API（5-7 天）

**任务清单**:

- [ ] 实现 SearchBuilder
  - [ ] 基础搜索功能
  - [ ] Builder 模式链式调用
  - [ ] 混合搜索集成
  - [ ] 重排序集成
  - [ ] 记忆调度集成
  - [ ] 过滤器实现

- [ ] 实现 BatchBuilder
  - [ ] 批量添加功能
  - [ ] 并发控制
  - [ ] 进度回调
  - [ ] 错误处理

- [ ] 实现新的核心 API
  - [ ] `add()` - 统一添加入口
  - [ ] `add_batch()` - 批量添加
  - [ ] `add_image()` - 图片添加
  - [ ] `add_audio()` - 音频添加
  - [ ] `add_video()` - 视频添加
  - [ ] `get()` - 获取单个
  - [ ] `get_all()` - 获取全部
  - [ ] `update()` - 更新
  - [ ] `delete()` - 删除单个
  - [ ] `delete_all()` - 删除全部
  - [ ] `search()` - 搜索入口
  - [ ] `search_builder()` - 搜索 builder
  - [ ] `stats()` - 统计信息

- [ ] 编写完整的测试套件
  - [ ] 单元测试（每个方法）
  - [ ] 集成测试（端到端）
  - [ ] 性能测试（基准测试）

#### 阶段 3：标记旧 API 废弃（2-3 天）

**任务清单**:

- [ ] 标记所有旧 API 为 `#[deprecated]`
  ```rust
  #[deprecated(since = "2.6.0", note = "Use `add()` instead")]
  pub async fn add_memory_fast(...);

  #[deprecated(since = "2.6.0", note = "Use `add()` instead")]
  pub async fn add_memory(...);

  #[deprecated(since = "2.6.0", note = "Use `add()` instead")]
  pub async fn add_memory_v2(...);

  #[deprecated(since = "2.6.0", note = "Use `search()` instead")]
  pub async fn search_memories(...);

  #[deprecated(since = "2.6.0", note = "Use `search_builder()` instead")]
  pub async fn search_memories_hybrid(...);
  ```

- [ ] 更新文档
  - [ ] API 迁移指南
  - [ ] 新 API 使用示例
  - [ ] 常见问题解答

- [ ] 更新示例代码
  - [ ] 所有 examples/ 使用新 API
  - [ ] 教程和指南

#### 阶段 4：优化和清理（3-5 天）

**任务清单**:

- [ ] 减少 clone() 使用
  - [ ] 分析当前 clone 点
  - [ ] 使用引用替代
  - [ ] 使用 Arc 共享
  - [ ] 验证性能提升

- [ ] 统一错误处理
  - [ ] 移除 unwrap()
  - [ ] 移除 expect()
  - [ ] 使用 Result<T>
  - [ ] 添加错误上下文

- [ ] 代码审查
  - [ ] API 一致性检查
  - [ ] 命名规范检查
  - [ ] 文档完整性检查

#### 阶段 5：发布和验证（2-3 天）

**任务清单**:

- [ ] 发布候选版本
- [ ] 内部测试
- [ ] 外部 beta 测试
- [ ] 性能基准测试
- [ ] 文档完整性验证
- [ ] 正式发布

### 3.2 时间线

```text
Week 1 (3-5 天):
├── Day 1-2: 准备阶段
│   ├── 创建新模块结构
│   └── 编写核心类型定义
└── Day 3-5: 实现 SearchBuilder
    ├── 基础搜索功能
    ├── Builder 模式
    └── 测试

Week 2 (5-7 天):
├── Day 1-3: 实现 BatchBuilder 和核心 API
│   ├── BatchBuilder
│   ├── 新的 add/get/update/delete API
│   └── 测试
└── Day 4-7: 标记旧 API 废弃
    ├── 添加 #[deprecated]
    ├── 更新文档
    └── 更新示例

Week 3 (3-5 天):
├── Day 1-3: 优化和清理
│   ├── 减少 clone()
│   ├── 统一错误处理
│   └── 代码审查
└── Day 4-5: 发布和验证
    ├── 性能测试
    ├── 文档验证
    └── 正式发布
```

### 3.3 验证标准

#### 功能验证

- [ ] 所有新 API 测试通过
- [ ] 所有旧 API 仍然可用（标记废弃）
- [ ] 端到端测试通过
- [ ] 性能测试符合预期

#### 质量验证

- [ ] 编译无警告
- [ ] 测试覆盖率 >80%
- [ ] 文档 100% 完整
- [ ] 所有示例可运行

#### 用户体验验证

- [ ] 5 分钟上手教程完成
- [ ] API 可发现性测试通过
- [ ] 文档清晰度评分 >4/5
- [ ] 用户反馈测试通过

---

## 📊 第四部分：预期效果

### 4.1 API 数量对比

| 类别 | 当前 | 目标 | 减少 |
|------|------|------|------|
| **核心 API** | 52 个 | ~25 个 | **-52%** |
| **工具 API** | 51 个 | ~5 个 | **-90%** |
| **总计** | 103 个 | ~30 个 | **-71%** |

### 4.2 代码质量提升

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| **clone() 调用** | 185 次 | <100 次 | **-46%** |
| **unwrap/expect** | 383 文件 | <10 文件 | **-97%** |
| **公共方法** | 103 个 | ~30 个 | **-71%** |
| **测试覆盖率** | 未知 | >80% | **?** |

### 4.3 用户体验提升

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| **上手时间** | >30 分钟 | <5 分钟 | **-83%** |
| **API 可发现性** | 困难 | 容易 | **+++** |
| **示例可运行** | 部分 | 100% | **+100%** |
| **文档准确性** | 70% | 95% | **+36%** |

### 4.4 性能提升

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| **搜索延迟** | 基准 | -20% | **+20%** |
| **批量添加** | 基准 | +30% | **+30%** |
| **内存占用** | 基准 | -15% | **+15%** |

---

## 🎯 第五部分：成功标准

### 5.1 必须达成（P0）

- [ ] 新 API 实现完成
- [ ] 所有测试通过
- [ ] 旧 API 标记废弃但仍可用
- [ ] 文档完整更新
- [ ] 性能无明显下降

### 5.2 应该达成（P1）

- [ ] clone() 使用减少 >40%
- [ ] 错误处理统一
- [ ] 测试覆盖率 >80%
- [ ] 所有示例可运行

### 5.3 最好达成（P2）

- [ ] 性能提升 >20%
- [ ] 用户反馈评分 >4/5
- [ ] API 一致性评分 >4.5/5
- [ ] 文档质量评分 >4.5/5

---

## 📝 第六部分：风险评估

### 6.1 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| **破坏性变更** | 中 | 高 | 保持向后兼容 |
| **性能下降** | 低 | 中 | 性能测试验证 |
| **测试覆盖不足** | 中 | 中 | 增加测试投入 |

### 6.2 项目风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| **时间超期** | 中 | 中 | 分阶段交付 |
| **资源不足** | 低 | 高 | 优先级管理 |
| **用户抵触** | 低 | 中 | 渐进式迁移 |

---

## 🚀 第七部分：后续优化

### 7.1 短期优化（1-2 月）

- [ ] 实现记忆调度集成
- [ ] 完善过滤器功能
- [ ] 添加高级搜索功能
- [ ] 性能持续优化

### 7.2 长期优化（3-6 月）

- [ ] API v3.0 规划
- [ ] 移除废弃的 API
- [ ] 架构持续优化
- [ ] 生态系统扩展

---

## 📚 附录

### A. 完整的旧 API 列表

#### 记忆添加 (8 个)
- `add_memory_fast()`
- `add_memory()`
- `add_memory_v2()`
- `add_memory_intelligent()`
- `add_memories_batch()`
- `add_memory_batch_optimized()`
- `add_image_memory()`
- `add_audio_memory()`
- `add_video_memory()`

#### 记忆查询 (6 个)
- `get_memory()`
- `get_all_memories()`
- `get_all_memories_v2()`
- `search_memories()`
- `search_memories_hybrid()`
- `cached_search()`

#### 记忆更新 (2 个)
- `update_memory()`
- (其他内部方法)

#### 记忆删除 (3 个)
- `delete_memory()`
- `delete_all_memories()`
- `reset()`

#### 统计分析 (3 个)
- `get_stats()`
- `get_performance_stats()`
- `get_history()`

#### 工具函数 (15+ 个)
- `generate_query_embedding()`
- `calculate_dynamic_threshold()`
- `preprocess_query()`
- `convert_search_results_to_memory_items()`
- `structured_fact_to_memory_item()`
- `structured_fact_to_core_memory()`
- `existing_memory_to_memory_item()`
- `existing_memory_to_core_memory()`
- `infer_scope_type()`
- `build_standard_metadata()`
- `deduplicate_memory_items()`
- `infer_memory_type()`
- `build_rerank_prompt()`
- `parse_rerank_response()`
- (更多...）

### B. 完整的新 API 列表

#### 核心记忆 API (~12 个)
```rust
// 添加
add() -> Result<String>
add_batch() -> BatchBuilder
add_image() -> Result<String>
add_audio() -> Result<String>
add_video() -> Result<String>

// 查询
get(id: &str) -> Result<MemoryItem>
get_all() -> Result<Vec<MemoryItem>>
search(query: &str) -> Result<Vec<MemoryItem>>
search_builder(query: &str) -> SearchBuilder

// 更新
update(id: &str, content: &str) -> Result<()>

// 删除
delete(id: &str) -> Result<()>
delete_all() -> Result<()>
reset() -> Result<()>

// 统计
stats() -> Result<MemoryStats>
performance_stats() -> Result<PerformanceStats>
history(id: &str) -> Result<Vec<HistoryEntry>>
```

### C. 迁移指南

#### 从旧 API 迁移到新 API

```rust
// ❌ 旧 API
let id = orchestrator.add_memory_fast("content").await?;
let results = orchestrator.search_memories_hybrid("query", 10, None).await?;

// ✅ 新 API
let id = orchestrator.add("content").await?;
let results = orchestrator.search("query").await?;

// ✅ 新 API（高级用法）
let results = orchestrator
    .search_builder("query")
    .limit(10)
    .with_rerank(true)
    .execute()
    .await?;
```

### D. 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effectiverust/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

---

**文档版本**: 1.0
**最后更新**: 2025-01-08
**负责人**: AgentMem 开发团队
**审核人**: 待定

---

## 🎯 总结

这份重构计划旨在解决 AgentMem 2.6 的核心 API 设计问题：

### 核心目标
1. **减少 API 数量**: 从 103 个减少到 ~30 个（-71%）
2. **统一命名规范**: 清晰、一致的命名
3. **Builder 模式**: 灵活、可组合的 API
4. **向后兼容**: 旧 API 标记废弃，平滑迁移

### 预期效果
- 用户体验提升 80%+
- 代码质量提升 50%+
- 维护成本降低 60%+
- 性能提升 20%+

### 实施周期
2-3 周完成，分 5 个阶段渐进实施。

---

**立即行动**: 开始阶段 1，创建新模块结构！
