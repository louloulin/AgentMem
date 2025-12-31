# 插件开发教程

学习如何为 AgentMem 开发自定义插件，扩展其功能。

## 目录

- [插件系统概述](#插件系统概述)
- [创建插件](#创建插件)
- [插件 API](#插件-api)
- [常用钩子](#常用钩子)
- [插件示例](#插件示例)
- [插件调试](#插件调试)
- [发布插件](#发布插件)

## 插件系统概述

AgentMem 插件系统允许你扩展和自定义 AgentMem 的功能，而无需修改核心代码。

### 插件能力

- **记忆钩子**: 在添加、更新、删除记忆时执行自定义逻辑
- **搜索增强**: 自定义搜索算法和结果处理
- **内容处理**: 自定义内容提取、转换、验证
- **存储扩展**: 支持自定义存储后端
- **API 扩展**: 添加新的 API 端点
- **事件处理**: 响应系统事件

### 插件类型

| 类型 | 说明 | 示例 |
|-----|------|-----|
| Hook Plugin | 钩子插件 | 记忆验证、内容过滤 |
| Processor Plugin | 处理器插件 | 文本摘要、情感分析 |
| Storage Plugin | 存储插件 | 自定义数据库适配器 |
| Search Plugin | 搜索插件 | 自定义搜索算法 |
| API Plugin | API 插件 | REST 端点扩展 |

## 创建插件

### 1. 设置开发环境

```bash
# 创建新的插件项目
cargo new --lib my-agentmem-plugin
cd my-agentmem-plugin

# 添加 AgentMem 插件 SDK
cargo add agent-mem-plugin-sdk
```

### 2. 基础插件结构

```rust
// src/lib.rs
use agent_mem_plugin_sdk::{
    Plugin, PluginContext, PluginMetadata, PluginResult,
    hooks::{AddHook, UpdateHook, DeleteHook},
};

pub struct MyPlugin {
    config: serde_json::Value,
}

impl MyPlugin {
    pub fn new(config: serde_json::Value) -> Self {
        Self { config }
    }
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My custom AgentMem plugin".to_string(),
            author: "Your Name".to_string(),
        }
    }

    fn initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        tracing::info!("Initializing my-plugin");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("Shutting down my-plugin");
        Ok(())
    }
}
```

### 3. 注册钩子

```rust
use agent_mem_plugin_sdk::hooks::*;

impl AddHook for MyPlugin {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory> {
        // 在添加记忆前执行
        let mut modified = memory.clone();

        // 添加自定义元数据
        modified.metadata.insert(
            "processed_by".to_string(),
            "my-plugin".into()
        );

        Ok(modified)
    }
}

impl UpdateHook for MyPlugin {
    fn on_update(&self, memory: &Memory) -> PluginResult<Memory> {
        // 在更新记忆前执行
        Ok(memory.clone())
    }
}

impl DeleteHook for MyPlugin {
    fn on_delete(&self, memory_id: &str) -> PluginResult<()> {
        // 在删除记忆前执行
        tracing::info!("Deleting memory: {}", memory_id);
        Ok(())
    }
}
```

### 4. 构建插件

```toml
# Cargo.toml
[package]
name = "my-agentmem-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
agent-mem-plugin-sdk = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
```

```bash
# 构建动态库
cargo build --release

# 输出: target/release/libmy_agentmem_plugin.so
```

## 插件 API

### 钩子 API

```rust
pub trait AddHook {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory>;
}

pub trait UpdateHook {
    fn on_update(&self, memory: &Memory) -> PluginResult<Memory>;
}

pub trait DeleteHook {
    fn on_delete(&self, memory_id: &str) -> PluginResult<()>;
}

pub trait SearchHook {
    fn on_search(&self, query: &SearchQuery) -> PluginResult<SearchQuery>;
    fn on_search_results(&self, results: Vec<SearchResult>)
        -> PluginResult<Vec<SearchResult>>;
}

pub trait ContentHook {
    fn on_content_extract(&self, content: &Content)
        -> PluginResult<Content>;
    fn on_content_transform(&self, content: &Content)
        -> PluginResult<Content>;
}
```

### 上下文 API

```rust
pub struct PluginContext {
    pub config: Arc<Config>,
    pub storage: Arc<dyn Storage>,
    pub embedder: Arc<dyn Embedder>,
    pub llm: Arc<dyn LLM>,
}

impl PluginContext {
    // 访问配置
    pub fn get_config(&self, key: &str) -> Option<String>;

    // 访问存储
    pub async fn get_memory(&self, id: &str) -> PluginResult<Memory>;
    pub async fn add_memory(&self, memory: &Memory) -> PluginResult<String>;

    // 访问嵌入
    pub async fn embed(&self, text: &str) -> PluginResult<Vec<f32>>;

    // 访问 LLM
    pub async fn generate(&self, prompt: &str) -> PluginResult<String>;
}
```

## 常用钩子

### 1. 内容验证插件

```rust
pub struct ContentValidatorPlugin {
    max_length: usize,
    blocked_words: Vec<String>,
}

impl AddHook for ContentValidatorPlugin {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory> {
        // 检查长度
        if memory.content.len() > self.max_length {
            return Err(PluginError::ValidationError(
                "Content too long".to_string()
            ));
        }

        // 检查敏感词
        for word in &self.blocked_words {
            if memory.content.contains(word) {
                return Err(PluginError::ValidationError(
                    format!("Blocked word found: {}", word)
                ));
            }
        }

        Ok(memory.clone())
    }
}
```

### 2. 自动摘要插件

```rust
pub struct AutoSummarizerPlugin {
    summary_length: usize,
}

impl AddHook for AutoSummarizerPlugin {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory> {
        if memory.content.len() > 500 {
            let summary = summarize_text(&memory.content, self.summary_length);

            let mut modified = memory.clone();
            modified.metadata.insert(
                "summary".to_string(),
                summary.into()
            );

            Ok(modified)
        } else {
            Ok(memory.clone())
        }
    }
}

fn summarize_text(text: &str, max_length: usize) -> String {
    // 简单的摘要实现
    text.chars().take(max_length).collect::<String>() + "..."
}
```

### 3. 情感分析插件

```rust
pub struct SentimentAnalysisPlugin;

impl AddHook for SentimentAnalysisPlugin {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory> {
        let sentiment = analyze_sentiment(&memory.content);

        let mut modified = memory.clone();
        modified.metadata.insert(
            "sentiment".to_string(),
            sentiment.to_string().into()
        );
        modified.metadata.insert(
            "sentiment_score".to_string(),
            sentiment.score.into()
        );

        Ok(modified)
    }
}

fn analyze_sentiment(text: &str) -> Sentiment {
    // 简化的情感分析
    let positive_words = vec!["好", "棒", "优秀", "喜欢"];
    let negative_words = vec!["坏", "差", "讨厌", "不好"];

    let mut score = 0.0;

    for word in positive_words {
        if text.contains(word) {
            score += 0.2;
        }
    }

    for word in negative_words {
        if text.contains(word) {
            score -= 0.2;
        }
    }

    Sentiment {
        label: if score > 0.0 { "positive" }
                else if score < 0.0 { "negative" }
                else { "neutral" }.to_string(),
        score: score.abs(),
    }
}

struct Sentiment {
    label: String,
    score: f32,
}
```

### 4. 搜索增强插件

```rust
pub struct SearchBoostPlugin {
    boost_keywords: Vec<String>,
    boost_factor: f32,
}

impl SearchHook for SearchBoostPlugin {
    fn on_search_results(
        &self,
        results: Vec<SearchResult>
    ) -> PluginResult<Vec<SearchResult>> {
        let mut boosted = results;

        for result in &mut boosted {
            for keyword in &self.boost_keywords {
                if result.memory.content.contains(keyword) {
                    result.score *= self.boost_factor;
                }
            }
        }

        // 重新排序
        boosted.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap()
        });

        Ok(boosted)
    }
}
```

## 插件示例

### 完整示例：PII 过滤插件

```rust
use agent_mem_plugin_sdk::*;
use regex::Regex;

pub struct PIIFilterPlugin {
    email_pattern: Regex,
    phone_pattern: Regex,
    ssn_pattern: Regex,
}

impl PIIFilterPlugin {
    pub fn new() -> Self {
        Self {
            email_pattern: Regex::new(
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
            ).unwrap(),
            phone_pattern: Regex::new(
                r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b"
            ).unwrap(),
            ssn_pattern: Regex::new(
                r"\b\d{3}-\d{2}-\d{4}\b"
            ).unwrap(),
        }
    }

    fn filter_pii(&self, text: &str) -> String {
        let mut filtered = text.to_string();

        // 过滤邮箱
        filtered = self.email_pattern.replace_all(&filtered, "[EMAIL]").to_string();

        // 过滤电话
        filtered = self.phone_pattern.replace_all(&filtered, "[PHONE]").to_string();

        // 过滤 SSN
        filtered = self.ssn_pattern.replace_all(&filtered, "[SSN]").to_string();

        filtered
    }
}

impl Plugin for PIIFilterPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "pii-filter".to_string(),
            version: "1.0.0".to_string(),
            description: "Filters personally identifiable information".to_string(),
            author: "Security Team".to_string(),
        }
    }

    fn initialize(&mut self, _context: &PluginContext) -> PluginResult<()> {
        tracing::info!("PII Filter plugin initialized");
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        Ok(())
    }
}

impl AddHook for PIIFilterPlugin {
    fn on_add(&self, memory: &Memory) -> PluginResult<Memory> {
        let filtered = self.filter_pii(&memory.content);

        let mut modified = memory.clone();
        modified.content = filtered;
        modified.metadata.insert(
            "pii_filtered".to_string(),
            true.into()
        );

        Ok(modified)
    }
}

impl UpdateHook for PIIFilterPlugin {
    fn on_update(&self, memory: &Memory) -> PluginResult<Memory> {
        self.on_add(memory)  // 复用逻辑
    }
}
```

## 插件调试

### 1. 启用插件日志

```rust
let memory = Memory::builder()
    .plugin(MyPlugin::new(config))
    .plugin_log_level(tracing::Level::DEBUG)
    .build()
    .await?;
```

### 2. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_filter() {
        let plugin = PIIFilterPlugin::new();

        let input = "Contact me at john@example.com or 555-123-4567";
        let output = plugin.filter_pii(input);

        assert_eq!(output, "Contact me at [EMAIL] or [PHONE]");
    }

    #[tokio::test]
    async fn test_add_hook() {
        let plugin = PIIFilterPlugin::new();

        let memory = Memory::builder()
            .content("Email: test@example.com")
            .build();

        let result = plugin.on_add(&memory).unwrap();

        assert_eq!(result.content, "Email: [EMAIL]");
        assert!(result.metadata.get("pii_filtered").is_some());
    }
}
```

### 3. 集成测试

```bash
# 使用测试环境运行插件
cargo run --example plugin-test
```

## 发布插件

### 1. 准备发布

```bash
# 运行所有测试
cargo test --all

# 检查代码格式
cargo fmt --check

# 运行 clippy
cargo clippy -- -D warnings

# 构建发布版本
cargo build --release
```

### 2. 发布到 crates.io

```bash
# 登录 crates.io
cargo login

# 发布
cargo publish
```

### 3. 创建插件清单

```toml
# plugin.toml
[plugin]
name = "my-plugin"
version = "0.1.0"
author = "Your Name <you@example.com>"
description = "My awesome AgentMem plugin"
repository = "https://github.com/you/my-plugin"
license = "MIT"

[dependencies]
agentmem = ">=2.0.0"

[agentmem]
min_version = "2.0.0"
max_version = "3.0.0"
```

### 4. 文档

创建 README.md:

```markdown
# My Plugin

Description of your plugin.

## Installation

\`\`\`bash
cargo add my-plugin
\`\`\`

## Usage

\`\`\`rust
use agentmem::Memory;
use my_plugin::MyPlugin;

let memory = Memory::builder()
    .plugin(MyPlugin::new())
    .build()
    .await?;
\`\`\`

## Configuration

Configuration options...
```

## 最佳实践

1. **错误处理**: 始终返回 `PluginResult`
2. **性能**: 避免阻塞操作，使用异步
3. **日志**: 使用 `tracing` 记录重要事件
4. **测试**: 编写全面的单元测试和集成测试
5. **文档**: 提供清晰的 API 文档和示例
6. **版本管理**: 遵循语义化版本控制

## 插件市场

AgentMem 插件市场即将推出，你将能够：

- 发现和安装社区插件
- 发布自己的插件
- 获取插件使用统计
- 参与插件评分和评论

## 下一步

- [生产部署](production.md) - 部署到生产环境
- [API 参考](../api_reference/) - 完整的 API 文档
