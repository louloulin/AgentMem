# AgentMem 智能推理引擎

## 概述

AgentMem 智能推理引擎是一个基于大语言模型（LLM）的智能记忆处理系统，能够从对话中自动提取事实信息，并智能决策记忆操作（添加、更新、删除、合并）。

## 核心组件

### 1. DeepSeek LLM 提供商 (`agent-mem-llm/providers/deepseek.rs`)

- **功能**: 集成 DeepSeek API，提供高质量的中英文语言模型服务
- **特性**:
  - 支持聊天完成 API
  - 支持结构化 JSON 响应生成
  - 支持系统和用户消息组合
  - 自动错误处理和重试机制
  - 可配置的超时和参数设置

```rust
// 使用示例
let provider = DeepSeekProvider::with_api_key(api_key)?;
let response = provider.generate_json::<MyStruct>(&prompt).await?;
```

### 2. 事实提取器 (`agent-mem-intelligence/fact_extraction.rs`)

- **功能**: 使用 LLM 从对话消息中提取结构化事实信息
- **核心类型**:
  - `ExtractedFact`: 提取的事实信息，包含内容、置信度、类别、实体等
  - `FactCategory`: 事实类别（个人信息、偏好、关系、事件、知识、程序性知识）
  - `Message`: 消息结构，支持角色、内容、时间戳等

- **主要功能**:
  - 智能事实提取
  - 事实验证和过滤
  - 相似事实合并
  - 置信度评估

```rust
// 使用示例
let extractor = FactExtractor::new(api_key)?;
let facts = extractor.extract_facts(&messages).await?;
let validated_facts = extractor.validate_facts(facts);
```

### 3. 记忆决策引擎 (`agent-mem-intelligence/decision_engine.rs`)

- **功能**: 基于提取的事实和现有记忆，智能决策记忆操作
- **决策类型**:
  - `Add`: 添加新记忆
  - `Update`: 更新现有记忆
  - `Delete`: 删除过时记忆
  - `Merge`: 合并相似记忆
  - `NoAction`: 无需操作

- **智能功能**:
  - 相似记忆检测
  - 记忆冲突识别
  - 重要性评估
  - 合并策略选择

```rust
// 使用示例
let engine = MemoryDecisionEngine::new(api_key)?;
let decisions = engine.make_decisions(&facts, &existing_memories).await?;
```

### 4. 智能记忆处理器 (`agent-mem-intelligence/intelligent_processor.rs`)

- **功能**: 整合事实提取和决策引擎，提供完整的智能记忆处理能力
- **核心特性**:
  - 端到端的消息处理
  - 记忆健康分析
  - 处理统计和推荐
  - 可配置的处理参数

```rust
// 使用示例
let processor = IntelligentMemoryProcessor::new(api_key)?;
let result = processor.process_messages(&messages, &existing_memories).await?;

// 分析记忆健康状况
let health_report = processor.analyze_memory_health(&existing_memories).await?;
```

## 架构设计

### 处理流程

1. **消息输入**: 接收用户和助手的对话消息
2. **事实提取**: 使用 LLM 从消息中提取结构化事实
3. **事实验证**: 过滤低质量和重复的事实
4. **决策生成**: 基于事实和现有记忆生成操作决策
5. **结果输出**: 返回处理结果、统计信息和推荐

### 数据流

```
消息 → 事实提取器 → 验证/合并 → 决策引擎 → 记忆操作
  ↓                                        ↓
现有记忆 ←←←←←←←←←←←←←←←←←←←←←←←←←←←←←←← 更新后记忆
```

## 配置选项

### ProcessingConfig

```rust
pub struct ProcessingConfig {
    pub similarity_threshold: f32,      // 相似度阈值 (默认: 0.7)
    pub confidence_threshold: f32,      // 置信度阈值 (默认: 0.5)
    pub max_facts_per_message: usize,   // 每条消息最大事实数 (默认: 10)
    pub enable_fact_validation: bool,   // 启用事实验证 (默认: true)
    pub enable_fact_merging: bool,      // 启用事实合并 (默认: true)
}
```

## 演示程序

### 运行演示

```bash
cd examples/intelligent-reasoning-demo
cargo run
```

### 演示功能

1. **智能事实提取**: 从对话中提取个人信息、偏好、技能等事实
2. **记忆决策**: 智能决定添加、更新或合并记忆
3. **健康分析**: 分析现有记忆的质量和重复情况
4. **统计报告**: 提供详细的处理统计和推荐

## 技术特点

### 1. 多语言支持
- 支持中英文混合处理
- 使用 DeepSeek 模型的强大多语言能力

### 2. 智能决策
- 基于语义相似性的记忆匹配
- 智能冲突检测和解决
- 动态重要性评估

### 3. 可扩展架构
- 模块化设计，易于扩展
- 支持多种 LLM 提供商
- 灵活的配置选项

### 4. 性能优化
- 批量处理支持
- 智能缓存机制
- 异步处理架构

## 使用场景

1. **个人助手**: 记住用户偏好和个人信息
2. **客服系统**: 维护客户历史和偏好
3. **教育平台**: 跟踪学习进度和知识点
4. **企业知识库**: 自动整理和更新知识内容

## 未来扩展

1. **多模态支持**: 支持图像、音频等多媒体内容
2. **实时处理**: 支持流式处理和实时更新
3. **联邦学习**: 支持分布式记忆处理
4. **个性化模型**: 基于用户行为的个性化调优

## 总结

AgentMem 智能推理引擎提供了一个完整的、可扩展的智能记忆处理解决方案。通过结合先进的 LLM 技术和智能决策算法，它能够自动化地管理和优化记忆系统，为各种应用场景提供强大的记忆能力支持。
