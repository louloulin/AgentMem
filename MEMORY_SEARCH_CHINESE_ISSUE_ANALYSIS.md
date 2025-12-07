# 记忆检索中文查询问题分析

## 问题描述

用户输入"仓颉"进行记忆检索，但系统返回的都是不相关的记忆（批量测试记忆、林是IT工程师等），无法检索到与"仓颉"相关的信息。

## 根本原因分析

### 1. **Embedding模型不支持中文** ⚠️ 核心问题

**位置**: `crates/agent-mem-server/src/routes/memory.rs:69-70`

```rust
// 使用默认FastEmbed配置
info!("  - Provider: fastembed (默认)");
info!("  - Model: BAAI/bge-small-en-v1.5");  // ❌ 这是英文模型！
builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");
```

**问题**：
- `BAAI/bge-small-en-v1.5` 是**英文专用模型**（注意`-en-`后缀）
- 对中文文本的embedding质量很差
- 导致中文查询"仓颉"无法生成准确的向量表示
- 向量相似度搜索时无法匹配到相关记忆

### 2. **相似度阈值过高**

**位置**: `crates/agent-mem-server/src/routes/memory.rs:1185`

```rust
let min_score_threshold = request.threshold.unwrap_or(0.7); // 默认最低阈值 0.7
```

**问题**：
- 默认阈值0.7对于使用英文模型处理中文查询来说过高
- 即使有相关记忆，也可能因为相似度分数低于0.7而被过滤

### 3. **可能没有相关记忆**

用户可能还没有存储过关于"仓颉"的记忆，所以即使搜索也找不到。

## 解决方案

### 方案1：使用支持中文的Embedding模型（推荐）✅

#### 1.1 使用多语言模型

**推荐模型**：
- `multilingual-e5-small` (384维，支持中英文)
- `multilingual-e5-base` (768维，支持中英文)
- `BAAI/bge-small-zh-v1.5` (384维，中文优化)

**修改位置**: `crates/agent-mem-server/src/routes/memory.rs:67-71`

```rust
// 修改前
builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");

// 修改后（支持中文）
builder = builder.with_embedder("fastembed", "multilingual-e5-small");
// 或者
builder = builder.with_embedder("fastembed", "BAAI/bge-small-zh-v1.5");
```

#### 1.2 通过环境变量配置

**设置环境变量**：
```bash
export EMBEDDING_PROVIDER=fastembed
export FASTEMBED_MODEL=multilingual-e5-small
```

**代码会自动读取**（已在`crates/agent-mem-embeddings/src/factory.rs:378-380`实现）：
```rust
let model = std::env::var("FASTEMBED_MODEL")
    .unwrap_or_else(|_| "bge-small-en-v1.5".to_string());
```

### 方案2：降低中文查询的阈值

**位置**: `crates/agent-mem-server/src/routes/memory.rs:1185`

```rust
// 检测中文查询，降低阈值
let min_score_threshold = if contains_chinese(&request.query) {
    request.threshold.unwrap_or(0.5)  // 中文查询使用0.5
} else {
    request.threshold.unwrap_or(0.7)  // 英文查询使用0.7
};
```

**辅助函数**：
```rust
fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)
}
```

### 方案3：增强自适应阈值逻辑

**位置**: `crates/agent-mem-server/src/routes/memory.rs:438`

```rust
fn get_adaptive_threshold(query: &str) -> f32 {
    // 检测中文
    let has_chinese = query.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF);
    
    if has_chinese {
        // 中文查询：降低阈值，提高召回率
        0.5
    } else {
        // 英文查询：使用默认阈值
        0.7
    }
}
```

### 方案4：添加混合搜索策略

结合向量搜索和关键词搜索，提高中文查询的准确性：

```rust
// 1. 向量语义搜索（用于语义相似）
let vector_results = vector_search(query, threshold: 0.5).await?;

// 2. 关键词搜索（用于精确匹配）
let keyword_results = keyword_search(query).await?;

// 3. 合并和去重
let combined_results = merge_and_deduplicate(vector_results, keyword_results);
```

## 实施建议

### 优先级1：立即修复（P0）

1. **修改默认Embedding模型为支持中文的模型**
   - 文件：`crates/agent-mem-server/src/routes/memory.rs:70`
   - 修改：`"BAAI/bge-small-en-v1.5"` → `"multilingual-e5-small"`

2. **添加中文检测和阈值调整**
   - 文件：`crates/agent-mem-server/src/routes/memory.rs:438`
   - 增强`get_adaptive_threshold`函数，支持中文检测

### 优先级2：配置优化（P1）

1. **支持通过配置文件设置Embedding模型**
   - 允许用户通过`config.toml`配置模型
   - 支持运行时切换模型

2. **添加模型健康检查**
   - 检测当前模型是否支持中文
   - 在日志中明确显示模型语言支持情况

### 优先级3：功能增强（P2）

1. **实现混合搜索**
   - 结合向量搜索和关键词搜索
   - 提高中文查询的准确性和召回率

2. **添加搜索质量监控**
   - 记录搜索查询和结果
   - 分析搜索效果，自动调整阈值

## 验证步骤

1. **测试中文查询**：
   ```bash
   curl -X POST http://localhost:8080/api/v1/memories/search \
     -H "Content-Type: application/json" \
     -d '{"query": "仓颉", "limit": 10}'
   ```

2. **检查Embedding质量**：
   - 查看日志中的embedding生成时间
   - 检查向量维度是否正确（multilingual-e5-small应该是384维）

3. **验证搜索结果**：
   - 确保相关记忆能被检索到
   - 检查相似度分数是否合理

## 相关代码位置

- Embedding配置：`crates/agent-mem-server/src/routes/memory.rs:60-71`
- 搜索阈值：`crates/agent-mem-server/src/routes/memory.rs:438, 1185`
- Embedding工厂：`crates/agent-mem-embeddings/src/factory.rs:256-278`
- 向量搜索：`crates/agent-mem-core/src/search/vector_search.rs:153-210`

## 总结

**核心问题**：默认使用英文Embedding模型处理中文查询，导致语义搜索失效。

**最佳解决方案**：使用`multilingual-e5-small`或`BAAI/bge-small-zh-v1.5`等支持中文的模型，并针对中文查询降低相似度阈值。

**预期效果**：
- ✅ 中文查询能够正确生成embedding
- ✅ 能够检索到相关的中文记忆
- ✅ 搜索准确率和召回率显著提升

