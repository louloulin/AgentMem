# AgentMem 智能功能集成指南

**版本**: 2.0.0  
**日期**: 2025-10-08  
**状态**: Phase 1.1 完成

---

## 📖 概述

本指南介绍如何在 AgentMem 中集成和使用智能记忆处理功能，包括：
- 智能事实提取 (FactExtractor)
- 智能决策引擎 (DecisionEngine)
- 记忆去重 (MemoryDeduplicator)

---

## 🏗️ 架构设计

### 依赖关系

```
agent-mem-traits (定义 trait)
    ↑
    ├── agent-mem-core (使用 trait)
    └── agent-mem-intelligence (实现 trait)
```

### 核心 Trait

#### 1. FactExtractor Trait

```rust
use agent_mem_traits::{FactExtractor, ExtractedFact, Message, Result};
use async_trait::async_trait;

#[async_trait]
pub trait FactExtractor: Send + Sync {
    /// 从消息中提取结构化事实
    async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>>;
}
```

#### 2. DecisionEngine Trait

```rust
use agent_mem_traits::{DecisionEngine, MemoryDecision, ExtractedFact, MemoryItem, Result};
use async_trait::async_trait;

#[async_trait]
pub trait DecisionEngine: Send + Sync {
    /// 为事实做出记忆操作决策
    async fn decide(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[MemoryItem],
    ) -> Result<MemoryDecision>;
}
```

---

## 🚀 快速开始

### 1. 添加依赖

```toml
[dependencies]
agent-mem-core = "2.0"
agent-mem-intelligence = "2.0"
agent-mem-llm = "2.0"
agent-mem-traits = "2.0"
tokio = { version = "1.35", features = ["full"] }
```

### 2. 创建智能组件

```rust
use agent_mem_intelligence::{FactExtractor, MemoryDecisionEngine};
use agent_mem_llm::providers::OpenAIProvider;
use agent_mem_traits::{LLMConfig, LLMProvider};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建 LLM 提供商
    let llm_config = LLMConfig {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        api_key: std::env::var("OPENAI_API_KEY")?,
        ..Default::default()
    };
    
    let llm_provider: Arc<dyn LLMProvider> = Arc::new(
        OpenAIProvider::new(llm_config)?
    );
    
    // 2. 创建智能组件
    let fact_extractor: Arc<dyn agent_mem_traits::FactExtractor> = 
        Arc::new(FactExtractor::new(llm_provider.clone()));
    
    let decision_engine: Arc<dyn agent_mem_traits::DecisionEngine> = 
        Arc::new(MemoryDecisionEngine::new(llm_provider.clone()));
    
    // 3. 创建 MemoryManager
    let config = agent_mem_config::MemoryConfig::default();
    let memory_manager = agent_mem_core::MemoryManager::with_intelligent_components(
        config,
        Some(fact_extractor),
        Some(decision_engine),
        Some(llm_provider),
    );
    
    // 4. 使用智能功能
    let memory_id = memory_manager.add_memory(
        "agent1".to_string(),
        Some("user1".to_string()),
        "我喜欢 Rust 编程，特别是异步编程。".to_string(),
        None,
        None,
        None,
    ).await?;
    
    println!("创建记忆: {}", memory_id);
    
    Ok(())
}
```

---

## 📝 详细使用指南

### 配置智能功能

#### 1. 启用/禁用智能功能

```rust
use agent_mem_config::MemoryConfig;

let mut config = MemoryConfig::default();

// 启用智能事实提取
config.intelligence.enable_intelligent_extraction = true;

// 启用智能决策引擎
config.intelligence.enable_decision_engine = true;

// 启用记忆去重
config.intelligence.enable_deduplication = true;
```

#### 2. 配置事实提取

```rust
// 设置最小置信度阈值
config.intelligence.fact_extraction.min_confidence = 0.7;

// 启用实体提取
config.intelligence.fact_extraction.extract_entities = true;

// 启用关系提取
config.intelligence.fact_extraction.extract_relations = true;

// 设置每条消息最大事实数
config.intelligence.fact_extraction.max_facts_per_message = 10;
```

#### 3. 配置决策引擎

```rust
// 设置相似度阈值
config.intelligence.decision_engine.similarity_threshold = 0.85;

// 设置最小决策置信度
config.intelligence.decision_engine.min_decision_confidence = 0.6;

// 启用智能合并
config.intelligence.decision_engine.enable_intelligent_merge = true;

// 设置最大相似记忆数
config.intelligence.decision_engine.max_similar_memories = 5;
```

#### 4. 配置去重

```rust
// 设置去重相似度阈值
config.intelligence.deduplication.similarity_threshold = 0.9;

// 设置时间窗口（秒）
config.intelligence.deduplication.time_window_seconds = Some(3600);

// 设置合并策略
config.intelligence.deduplication.merge_strategy = "intelligent_merge".to_string();
```

---

### 智能处理流程

#### 1. 智能 add_memory 流程

```rust
// 用户调用
memory_manager.add_memory(
    agent_id,
    user_id,
    content,
    memory_type,
    importance,
    metadata,
).await?

    ↓
    
// 自动选择流程
if fact_extractor.is_some() && decision_engine.is_some() {
    add_memory_intelligent()  // 智能流程
} else {
    add_memory_simple()       // 简单流程（降级）
}

    ↓
    
// 智能流程步骤
1. extract_facts_from_content()      // 提取事实
2. find_similar_memories_for_fact()  // 查找相似记忆
3. make_decision_for_fact()          // 做出决策
4. execute_memory_action()           // 执行操作
```

#### 2. 决策类型

智能决策引擎可以做出 5 种决策：

```rust
pub enum MemoryActionType {
    // 1. 添加新记忆
    Add {
        content: String,
        importance: f32,
        metadata: HashMap<String, String>,
    },
    
    // 2. 更新现有记忆
    Update {
        memory_id: String,
        new_content: String,
        merge_strategy: String,  // "replace", "append", "merge"
    },
    
    // 3. 删除记忆
    Delete {
        memory_id: String,
        reason: String,
    },
    
    // 4. 合并多个记忆
    Merge {
        primary_memory_id: String,
        secondary_memory_ids: Vec<String>,
        merged_content: String,
    },
    
    // 5. 不执行任何操作
    NoAction {
        reason: String,
    },
}
```

---

## 🔧 高级用法

### 1. 自定义 FactExtractor 实现

```rust
use agent_mem_traits::{FactExtractor, ExtractedFact, Message, Result};
use async_trait::async_trait;

pub struct CustomFactExtractor {
    // 自定义字段
}

#[async_trait]
impl FactExtractor for CustomFactExtractor {
    async fn extract_facts(&self, messages: &[Message]) -> Result<Vec<ExtractedFact>> {
        // 自定义实现
        let mut facts = Vec::new();
        
        for message in messages {
            // 自定义提取逻辑
            let fact = ExtractedFact {
                content: message.content.clone(),
                confidence: 0.8,
                category: "custom".to_string(),
                metadata: HashMap::new(),
            };
            facts.push(fact);
        }
        
        Ok(facts)
    }
}
```

### 2. 自定义 DecisionEngine 实现

```rust
use agent_mem_traits::{DecisionEngine, MemoryDecision, MemoryActionType, ExtractedFact, MemoryItem, Result};
use async_trait::async_trait;

pub struct CustomDecisionEngine {
    // 自定义字段
}

#[async_trait]
impl DecisionEngine for CustomDecisionEngine {
    async fn decide(
        &self,
        fact: &ExtractedFact,
        existing_memories: &[MemoryItem],
    ) -> Result<MemoryDecision> {
        // 自定义决策逻辑
        let action = if existing_memories.is_empty() {
            MemoryActionType::Add {
                content: fact.content.clone(),
                importance: fact.confidence,
                metadata: fact.metadata.clone(),
            }
        } else {
            MemoryActionType::Update {
                memory_id: existing_memories[0].id.clone(),
                new_content: fact.content.clone(),
                merge_strategy: "append".to_string(),
            }
        };
        
        Ok(MemoryDecision {
            action,
            confidence: 0.9,
            reasoning: "Custom decision logic".to_string(),
        })
    }
}
```

### 3. 使用自定义实现

```rust
let custom_fact_extractor: Arc<dyn FactExtractor> = 
    Arc::new(CustomFactExtractor::new());

let custom_decision_engine: Arc<dyn DecisionEngine> = 
    Arc::new(CustomDecisionEngine::new());

let memory_manager = MemoryManager::with_intelligent_components(
    config,
    Some(custom_fact_extractor),
    Some(custom_decision_engine),
    None,
);
```

---

## 🧪 测试

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_intelligent_memory_addition() {
        // 创建测试组件
        let llm_provider = Arc::new(LocalTestProvider::new());
        let fact_extractor = Arc::new(FactExtractor::new(llm_provider.clone()));
        let decision_engine = Arc::new(MemoryDecisionEngine::new(llm_provider.clone()));
        
        // 创建 MemoryManager
        let config = MemoryConfig::default();
        let manager = MemoryManager::with_intelligent_components(
            config,
            Some(fact_extractor),
            Some(decision_engine),
            Some(llm_provider),
        );
        
        // 测试添加记忆
        let memory_id = manager.add_memory(
            "test_agent".to_string(),
            Some("test_user".to_string()),
            "测试内容".to_string(),
            None,
            None,
            None,
        ).await.unwrap();
        
        assert!(!memory_id.is_empty());
    }
}
```

---

## 📊 性能考虑

### 1. 降级处理

智能功能失败时会自动降级到简单流程：

```rust
// 智能流程失败
match fact_extractor.extract_facts(&messages).await {
    Ok(facts) => {
        // 使用提取的事实
    },
    Err(e) => {
        warn!("智能提取失败: {}, 降级到简单流程", e);
        // 自动降级
        return self.add_memory_simple(...).await;
    }
}
```

### 2. 缓存机制

建议为智能组件添加缓存：

```rust
// TODO: 在 Day 3-4 实现
// - LRU 缓存提取的事实
// - 缓存决策结果
// - 缓存相似度计算
```

### 3. 批处理

对于大量记忆，建议使用批处理：

```rust
// TODO: 在 Day 3-4 实现
// - 批量提取事实
// - 批量做决策
// - 批量执行操作
```

---

## 🔍 故障排查

### 常见问题

#### 1. 智能功能未启用

**症状**: 记忆直接添加，没有智能处理

**原因**: 智能组件未注入或配置未启用

**解决**:
```rust
// 确保注入智能组件
let manager = MemoryManager::with_intelligent_components(
    config,
    Some(fact_extractor),  // 必须提供
    Some(decision_engine), // 必须提供
    Some(llm_provider),
);

// 确保配置启用
config.intelligence.enable_intelligent_extraction = true;
config.intelligence.enable_decision_engine = true;
```

#### 2. LLM 调用失败

**症状**: 智能功能报错

**原因**: LLM API 密钥无效或网络问题

**解决**:
```rust
// 检查 API 密钥
std::env::var("OPENAI_API_KEY")?;

// 使用本地测试提供商进行测试
let llm_provider = Arc::new(LocalTestProvider::new());
```

#### 3. 决策置信度过低

**症状**: 所有决策都降级为 Add

**原因**: 决策置信度阈值设置过高

**解决**:
```rust
// 降低阈值
config.intelligence.decision_engine.min_decision_confidence = 0.5;
```

---

## 📚 参考资料

- [agent-mem-traits API 文档](./crates/agent-mem-traits/README.md)
- [agent-mem-intelligence API 文档](./crates/agent-mem-intelligence/README.md)
- [agent-mem-core API 文档](./crates/agent-mem-core/README.md)
- [示例代码](./examples/test-intelligent-integration/)

---

## 🎯 下一步

- [ ] Day 3-4: 性能优化和可观测性
- [ ] Day 5: 缓存机制和配置优化
- [ ] Day 6-7: 集成测试和文档完善

---

**版本历史**:
- v2.0.0 (2025-10-08): 初始版本，完成 Phase 1.1 智能功能集成

