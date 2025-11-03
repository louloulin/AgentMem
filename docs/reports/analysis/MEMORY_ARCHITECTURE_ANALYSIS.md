# AgentMem 记忆冲突问题 - 深度架构分析

## 🚨 问题现象（截图分析）

### 对话记录
```
09:57:18 - 用户: 我是冲
09:57:55 - Agent: 你好，冲！...你可能是张三、lin或吕洁。不过，为了确保准确性...

09:57:55 - 用户: 我是谁
09:58:03 - Agent: 你好！根据我之前的对话记忆，你可能是张三、lin或吕洁...
```

### 核心问题
1. **Session隔离失败** - 用户说"我是冲"，但Agent回复提到"张三、lin、吕洁"
2. **Working Memory失效** - 当前会话的信息被历史会话污染
3. **记忆混淆严重** - Agent无法区分当前用户和历史用户
4. **优先级混乱** - 长期记忆覆盖了当前会话上下文

## 📊 完整架构分析

### 1. 记忆检索路径

```
用户输入 "我是冲"
    ↓
AgentOrchestrator.step()
    ↓
┌─────────────────────────────────────────┐
│ 1. get_working_context(session_id)      │ ✅ 按session_id过滤
│    → WorkingMemoryStore.get_session_items│
│    → 返回当前会话的消息                   │
└─────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────┐
│ 2. retrieve_memories(query, agent_id)   │ ❌ 没有session_id过滤！
│    → MemoryIntegrator.retrieve_relevant │
│    → MemoryEngine.search_memories       │
│    → LibSQL: find_by_agent_id()         │ ← 检索所有agent的记忆
│    → 返回：张三、lin、吕洁的所有记忆     │
└─────────────────────────────────────────┘
    ↓
build_messages_with_context()
    ↓ 构建Prompt:
    [Working Context] + [长期记忆（所有用户）]
    ↓
LLM看到混合的记忆 → 无法区分当前用户
```

### 2. 问题根源定位

#### 问题1: MemoryEngine.search_memories 不支持session过滤
```rust
// agentmen/crates/agent-mem-core/src/engine.rs:184
pub async fn search_memories(
    &self,
    query: &str,
    scope: Option<MemoryScope>,  // ❌ 只有Agent/User/Organization，没有Session
    limit: Option<usize>,
) -> crate::CoreResult<Vec<Memory>> {
    // ...
    let agent_id = match &scope {
        Some(MemoryScope::Agent(id)) => Some(id.as_str()),
        _ => None,
    };
    
    // ❌ 只按agent_id过滤，不考虑session_id
    memory_repo.find_by_agent_id(aid, fetch_limit).await
}
```

#### 问题2: MemoryScope缺少Session级别
```rust
// MemoryScope枚举（推测）
pub enum MemoryScope {
    Agent(String),      // ✅ 有
    User(String),       // ✅ 有
    Organization(String), // ✅ 有
    // ❌ 缺少: Session(String)
}
```

#### 问题3: 记忆没有时间衰减
```rust
// agentmen/crates/agent-mem-core/src/engine.rs:275
scored_memories.sort_by(|(mem_a, score_a), (mem_b, score_b)| {
    // 只考虑内容相关性 + 重要性
    let combined_a = score_a + (mem_a.importance as f64 * 0.3);
    let combined_b = score_b + (mem_b.importance as f64 * 0.3);
    // ❌ 没有考虑时间因素！旧记忆和新记忆权重相同
    combined_b.partial_cmp(&combined_a).unwrap_or(std::cmp::Ordering::Equal)
});
```

#### 问题4: Prompt优先级不够
即使我之前加了"HIGHEST PRIORITY"，但LLM依然会看到所有历史记忆。

## 🎯 论文启发的最佳方案

### 研究参考
1. **MemGPT** - 分层记忆管理，主记忆vs召回记忆
2. **A-MEM** - 动态记忆网络，主动过滤
3. **AgentCDM** - 协作决策，减少认知偏差
4. **Memory Engineering** - 时间衰减、情境优先

### 核心原则
1. **时间局部性** - 最近的记忆优先级最高
2. **会话隔离** - 不同session的记忆严格隔离
3. **渐进式衰减** - 记忆随时间衰减
4. **分层检索** - 先Working Memory，再Long-term Memory

## ✅ 综合解决方案设计

### 方案架构

```
记忆检索的三层过滤机制：

Layer 1: Session Working Memory (最高优先级)
  - 存储：memories表 (memory_type='working', session_id=xxx)
  - 检索：直接按session_id查询
  - 权重：1.0 (不衰减)
  - 用途：当前对话上下文

Layer 2: Session Long-term Memory (中优先级)  
  - 存储：memories表 (memory_type='long_term', session_id=xxx)
  - 检索：按session_id + 语义相关性
  - 权重：0.8 × time_decay
  - 用途：当前session的历史知识

Layer 3: Global Long-term Memory (低优先级)
  - 存储：memories表 (memory_type='long_term', session_id IS NULL 或其他session)
  - 检索：仅按agent_id + 语义相关性
  - 权重：0.3 × time_decay × relevance
  - 用途：跨session的背景知识（降权）
```

### 时间衰减公式（参考记忆工程）

```rust
fn calculate_memory_weight(memory: &Memory, current_time: DateTime<Utc>) -> f64 {
    let base_weight = match memory.memory_type {
        MemoryType::Working => 1.0,  // 不衰减
        MemoryType::Episodic | MemoryType::Semantic => 0.8,
        _ => 0.5,
    };
    
    // 时间衰减（指数衰减，半衰期24小时）
    let age_hours = (current_time - memory.created_at).num_hours() as f64;
    let time_decay = (- age_hours / 24.0).exp(); // e^(-t/24)
    
    // Session匹配加权
    let session_boost = if is_current_session {
        2.0  // 当前session的记忆加倍
    } else if is_same_user {
        1.2  // 同用户不同session的记忆略微提升
    } else {
        0.3  // 其他用户的记忆大幅降权
    };
    
    // 重要性权重
    let importance_weight = memory.importance as f64;
    
    // 综合权重
    base_weight * time_decay * session_boost * (0.5 + 0.5 * importance_weight)
}
```

## 🔧 具体实现步骤

### Step 1: 扩展MemoryScope支持Session
### Step 2: 实现session过滤的记忆检索
### Step 3: 实现时间衰减和权重计算
### Step 4: 优化MemoryIntegrator的整合策略
### Step 5: 增强Prompt构建逻辑
### Step 6: 添加记忆冲突检测

## 📈 预期效果

修复后的对话：
```
09:57:18 - 用户: 我是冲
09:57:55 - Agent: 你好，冲！很高兴认识你。（只使用当前会话信息）

09:57:55 - 用户: 我是谁
09:58:03 - Agent: 你是冲！（准确记忆当前用户）
```

记忆权重示例：
```
查询: "我是谁"
当前session_id: "sess-123"
当前用户: "冲"

检索结果权重：
[Working Memory - sess-123] "用户说：我是冲" → 权重: 1.0 ✅
[Long-term - sess-123] "冲喜欢..." → 权重: 0.8 × 0.95 = 0.76
[Long-term - sess-456] "张三说..." → 权重: 0.3 × 0.5 = 0.15 ⬇️
[Long-term - sess-789] "lin说..." → 权重: 0.3 × 0.5 = 0.15 ⬇️

最终只使用sess-123的记忆！
```

---

**下一步：开始实现**

