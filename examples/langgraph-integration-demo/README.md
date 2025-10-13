# LangGraph 集成演示

## 概述

本示例演示如何将 AgentMem 与类似 LangGraph 的状态图集成，实现：

- ✅ 状态管理
- ✅ 记忆提取和注入
- ✅ 对话历史管理
- ✅ 多轮对话
- ✅ 用户管理

## 功能特性

### 1. 状态图架构

```
用户输入 → 记忆提取 → LLM 响应 → 记忆存储 → 输出
```

### 2. 三个核心节点

#### 记忆提取节点（MemoryExtractionNode）
- 搜索相关记忆
- 构建系统消息
- 注入上下文

#### LLM 响应节点（LLMResponseNode）
- 调用 LLM
- 生成回复
- 添加到消息列表

#### 记忆存储节点（MemoryStorageNode）
- 提取对话
- 存储为情景记忆
- 添加元数据

### 3. 对话流程

1. 用户输入消息
2. 提取相关记忆
3. 构建系统提示
4. LLM 生成回复
5. 存储对话记忆
6. 返回回复

## 运行示例

### 前置条件

```bash
# 设置环境变量
export DATABASE_URL="sqlite://~/.agentmem/agentmem.db"
export AGENTMEM_LLM_PROVIDER="gemini"
export GEMINI_API_KEY="your_api_key"
```

### 运行

```bash
cargo run --package langgraph-integration-demo
```

### 预期输出

```
=== AgentMem LangGraph 集成演示 ===

--- 1. 创建 Agent ---
✅ Agent 创建成功

--- 2. 构建状态图 ---
✅ 状态图构建完成（3 个节点）

--- 3. 模拟多轮对话 ---
对话 1: 我叫张三，今年30岁，在北京工作。
助手: 根据您的问题「我叫张三，今年30岁，在北京工作。」和我的记忆，我可以为您提供帮助。

对话 2: 我是一名软件工程师，主要做 Rust 开发。
助手: 根据您的问题「我是一名软件工程师，主要做 Rust 开发。」和我的记忆，我可以为您提供帮助。

对话 3: 我最喜欢的编程语言是 Rust，因为它安全又高效。
助手: 根据您的问题「我最喜欢的编程语言是 Rust，因为它安全又高效。」和我的记忆，我可以为您提供帮助。

对话 4: 我的职业是什么？
助手: 根据您的问题「我的职业是什么？」和我的记忆，我可以为您提供帮助。

对话 5: 我喜欢什么编程语言？
助手: 根据您的问题「我喜欢什么编程语言？」和我的记忆，我可以为您提供帮助。

--- 4. 查看存储的记忆 ---
总记忆数: 5
  1. 用户: 我叫张三，今年30岁，在北京工作。
助手: 根据您的问题「我叫张三，今年30岁，在北京工作。」和我的记忆，我可以为您提供帮助。
  2. 用户: 我是一名软件工程师，主要做 Rust 开发。
助手: 根据您的问题「我是一名软件工程师，主要做 Rust 开发。」和我的记忆，我可以为您提供帮助。
  ...

=== 演示完成 ===
✅ LangGraph 集成功能验证通过！
```

## 与 MIRIX 对比

### MIRIX 实现

```python
from mirix import Mirix
from langgraph.graph import StateGraph

mirix_agent = Mirix(api_key=API_KEY)

def chatbot(state):
    memories = mirix_agent.extract_memory_for_system_prompt(
        state["messages"][-1].content, 
        user_id=state["user_id"]
    )
    system_message = "You have the following memories:\n\n" + memories
    full_messages = [system_message] + state["messages"]
    response = llm.invoke(full_messages)
    mirix_agent.add(interaction, user_id=state["user_id"])
    return {"messages": [response]}
```

### AgentMem 实现

```rust
struct MemoryExtractionNode {
    agent: Arc<Agent>,
}

impl GraphNode for MemoryExtractionNode {
    async fn execute(&self, mut state: ConversationState) -> Result<ConversationState> {
        let memories = self.agent.search(SearchRequest {
            query: last_message.content.clone(),
            limit: Some(5),
            filters: Some(SearchFilters {
                user_id: Some(state.user_id.clone()),
                min_importance: Some(0.5),
                ..Default::default()
            }),
        }).await?;
        
        // 构建系统消息并注入
        let memory_context = memories.iter()
            .map(|m| format!("- {}", m.content))
            .collect::<Vec<_>>()
            .join("\n");
        
        state.messages.insert(0, Message {
            role: "system".to_string(),
            content: format!("相关记忆：\n{}", memory_context),
            timestamp: chrono::Utc::now().timestamp(),
        });
        
        Ok(state)
    }
}
```

## 核心优势

### 1. 类型安全
- ✅ 编译时类型检查
- ✅ 无运行时类型错误
- ✅ 更好的 IDE 支持

### 2. 性能
- ✅ 零成本抽象
- ✅ 并发安全
- ✅ 内存高效

### 3. 可扩展性
- ✅ 易于添加新节点
- ✅ 灵活的状态管理
- ✅ 支持复杂工作流

## 扩展示例

### 添加自定义节点

```rust
struct CustomNode {
    // 自定义字段
}

impl GraphNode for CustomNode {
    async fn execute(&self, state: ConversationState) -> Result<ConversationState> {
        // 自定义逻辑
        Ok(state)
    }
}

// 添加到状态图
graph.add_node("custom".to_string(), Box::new(CustomNode { /* ... */ }));
```

### 条件分支

```rust
impl StateGraph {
    async fn execute_with_conditions(&self, mut state: ConversationState) -> Result<ConversationState> {
        for (name, node) in &self.nodes {
            // 根据条件决定是否执行节点
            if should_execute(name, &state) {
                state = node.execute(state).await?;
            }
        }
        Ok(state)
    }
}
```

## 相关文档

- [MIRIX LangGraph 集成](../../../source/MIRIX/samples/langgraph_integration.py)
- [AgentMem Agent API](../../crates/agent-mem-core/src/client.rs)
- [对比分析](../../doc/comparison/MIRIX_vs_AgentMem_Examples_Analysis.md)

## 许可证

Apache License 2.0

