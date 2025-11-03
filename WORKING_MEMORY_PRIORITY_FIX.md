# Working Memory 优先级问题修复报告

## 📅 时间：2024年11月3日

## ❌ 问题描述

用户发现Working Memory存在问题：**Agent在回复时混入了历史长期记忆，而不是优先使用当前会话的上下文**。

### 具体表现

**对话场景：**
1. 用户说："我是lin" （09:48:10）
2. Agent回复：认识lin （09:48:22）
3. 用户问："你是谁" （09:48:30）
4. Agent回复：提到了"张三"和"北京"等历史信息 ❌

**问题：** Agent应该基于当前会话记住"用户是lin"，但却引用了其他会话的历史记忆（"张三"、"北京"）。

---

## 🔍 根本原因分析

### 1. Working Memory实现正确
- ✅ `get_session_items` 正确按 `session_id` 过滤
- ✅ `memory_type='working'` 正确区分短期和长期记忆
- ✅ Working Memory本身的存储和检索逻辑没有问题

### 2. 长期记忆检索无过滤
```rust
async fn retrieve_memories(&self, request: &ChatRequest) -> Result<Vec<Memory>> {
    let memories = self.memory_integrator
        .retrieve_relevant_memories(&request.message, &request.agent_id, max_count)
        .await?;
    // ⚠️ 这里检索了所有相关的长期记忆，不区分session
    // 会包含其他会话的历史信息（如"张三"、"北京"）
}
```

### 3. Prompt优先级不够明确

**修复前的Prompt：**
```rust
// 1. 添加会话上下文（优先级最高）
if !working_context.is_empty() {
    system_message_parts.push(format!(
        "## Current Session Context\n\nThe following is the recent conversation history in this session:\n\n{}",
        working_context
    ));
}

// 2. 添加长期记忆
if !memories.is_empty() {
    let memory_context = self.memory_integrator.inject_memories_to_prompt(memories);
    system_message_parts.push(memory_context);
}
```

**问题：**
- 注释说"优先级最高"，但LLM看不到注释
- 没有明确告诉LLM如何处理冲突信息
- 长期记忆和当前会话记忆没有明确标记区分

---

## ✅ 修复方案

### 修改文件
`agentmen/crates/agent-mem-core/src/orchestrator/mod.rs`

### 修复内容

#### 1. 增强系统Prompt - 明确优先级

```rust
async fn build_messages_with_context(
    &self,
    request: &ChatRequest,
    working_context: &str,
    memories: &[Memory],
) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    let mut system_message_parts = Vec::new();

    // 1. 添加会话上下文（最高优先级，加强标记）
    if !working_context.is_empty() {
        system_message_parts.push(format!(
            "## ⚠️ CURRENT SESSION CONTEXT (HIGHEST PRIORITY)\n\n\
            **IMPORTANT**: The following is the CURRENT conversation in THIS session. \
            This information has the HIGHEST priority and should OVERRIDE any conflicting information from past memories.\n\n\
            **Current Session History:**\n{}",
            working_context
        ));
    }

    // 2. 添加长期记忆（明确标记为参考）
    if !memories.is_empty() {
        let memory_context = self.memory_integrator.inject_memories_to_prompt(memories);
        system_message_parts.push(format!(
            "## 📚 PAST MEMORIES (For Reference Only)\n\n\
            **Note**: The following are memories from PAST conversations. \
            If there is any conflict between these past memories and the current session context above, \
            ALWAYS prioritize the current session information.\n\n\
            {}",
            memory_context
        ));
    }

    // 构建系统消息
    if !system_message_parts.is_empty() {
        let system_content = system_message_parts.join("\n\n");
        messages.push(Message::system(&system_content));
    }

    // 添加用户消息
    messages.push(Message::user(&request.message));

    Ok(messages)
}
```

#### 2. 关键改进点

| 改进项 | 修复前 | 修复后 |
|-------|-------|-------|
| **标题强调** | "Current Session Context" | "⚠️ CURRENT SESSION CONTEXT (HIGHEST PRIORITY)" |
| **优先级说明** | 无 | "This information has the HIGHEST priority and should OVERRIDE..." |
| **冲突处理** | 无 | "If there is any conflict... ALWAYS prioritize current session" |
| **记忆分类** | 无明确区分 | 明确区分"CURRENT SESSION"和"PAST MEMORIES (For Reference Only)" |
| **视觉标记** | 无 | 使用emoji (⚠️ 📚) 增强区分度 |

---

## 📊 预期效果

### 修复后的对话流程

1. **用户：** "我是lin"
   - ✅ Working Memory记录："[09:48:10] User: 我是lin"
   
2. **用户：** "你是谁"
   - ✅ 系统Prompt包含：
     ```
     ⚠️ CURRENT SESSION CONTEXT (HIGHEST PRIORITY)
     Current Session History:
     [09:48:10] User: 我是lin
     [09:48:22] Assistant: 很高兴认识你...
     
     📚 PAST MEMORIES (For Reference Only)
     [可能包含其他用户的记忆，但明确标记为仅供参考]
     ```
   - ✅ Agent回复：基于当前会话，记得用户是"lin"，不会提到"张三"

### 测试验证点

- [ ] Agent能正确记住当前会话的用户名（lin）
- [ ] Agent不会混入其他会话的用户名（张三）
- [ ] Agent不会混入其他会话的地点信息（北京）
- [ ] Working Memory正确记录当前会话的所有消息
- [ ] 长期记忆仅作为背景知识使用，不干扰当前会话

---

## 🔧 编译和部署

```bash
# 1. 编译
cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
cargo build --release --bin agent-mem-server

# 2. 重启服务
pkill -f agent-mem-server
./start_server_with_correct_onnx.sh

# 3. 验证（在浏览器中测试）
open http://localhost:3001/admin/chat
```

### 编译结果
✅ 编译成功 (2024-11-03 09:55)

---

## 🧪 测试方法

### 方法1：自动化测试脚本

```bash
./test_working_memory_priority.sh
```

测试场景：
1. 创建新session
2. 用户说："我是lin"
3. 用户问："你是谁"
4. 验证：回复提到"lin"，不提到"张三"

### 方法2：UI手动测试

1. 打开 http://localhost:3001/admin/chat
2. 选择agent
3. 测试对话：
   ```
   用户: 我是lin
   [等待回复]
   用户: 你是谁
   [检查回复是否正确使用当前会话上下文]
   ```

---

## 📝 技术要点

### 1. 为什么不在检索时过滤长期记忆？

**当前方案（Prompt优先级）的优点：**
- ✅ 保留长期记忆的价值（可能有用的背景知识）
- ✅ 让LLM智能判断何时使用长期记忆
- ✅ 实现简单，不破坏现有检索逻辑
- ✅ 灵活性高，可以根据上下文调整

**如果在检索时强制过滤的缺点：**
- ❌ 可能丢失有用的背景知识
- ❌ 需要修改MemoryEngine的核心逻辑
- ❌ 可能影响其他功能
- ❌ session过滤粒度可能过粗

### 2. Prompt Engineering最佳实践

1. **清晰的层次结构**
   - 使用markdown标题（##）区分不同部分
   - 使用emoji视觉标记
   
2. **明确的指令**
   - 使用"IMPORTANT"、"ALWAYS"等强调词
   - 明确说明冲突处理规则
   
3. **优先级标记**
   - "HIGHEST PRIORITY"
   - "For Reference Only"
   
4. **结构化信息**
   - 当前会话 vs 历史记忆
   - 事实 vs 参考

### 3. Working Memory vs Long-term Memory

| 维度 | Working Memory | Long-term Memory |
|-----|---------------|------------------|
| **存储位置** | memories表 (memory_type='working') | memories表 (memory_type='long_term') |
| **隔离方式** | session_id | 无，全局共享 |
| **生命周期** | 24小时（expires_at） | 永久（直到删除） |
| **检索方式** | 按session直接查询 | 语义向量检索 |
| **用途** | 当前对话上下文 | 历史知识库 |
| **优先级** | 最高（当前真实情况） | 参考（可能过时） |

---

## ✨ 下一步优化建议

### 1. 短期（已完成）
- ✅ 增强Prompt明确优先级
- ✅ 区分当前会话和历史记忆
- ✅ 添加冲突处理规则

### 2. 中期（可选）
- [ ] 在检索长期记忆时，降低其他session的权重
- [ ] 添加时间衰减：越旧的记忆权重越低
- [ ] 用户级别的记忆隔离（只检索当前用户的记忆）

### 3. 长期（架构优化）
- [ ] 实现混合检索策略
- [ ] 智能记忆合并和冲突解决
- [ ] 上下文窗口管理优化
- [ ] 记忆重要性动态评分

---

## 📖 相关文档

- `STREAMING_IMPLEMENTATION_REPORT.md` - SSE流式响应实现
- `CHAT_WORKING_MEMORY_VERIFICATION_REPORT.md` - Working Memory初始验证
- `agentmen/crates/agent-mem-core/src/orchestrator/mod.rs` - 核心实现
- `agentmen/crates/agent-mem-storage/src/backends/libsql_working.rs` - 存储实现

---

## ✅ 修复总结

| 项目 | 状态 |
|-----|------|
| **问题识别** | ✅ 完成 |
| **根因分析** | ✅ 完成 |
| **代码修复** | ✅ 完成 |
| **编译通过** | ✅ 完成 |
| **服务重启** | ✅ 完成 |
| **UI测试** | 🔄 进行中 |

**当前状态：** 修复已部署，等待UI验证最终效果。

浏览器已打开 → http://localhost:3001/admin/chat

**测试指引：**
1. 选择"Working Memory Test Agent"
2. 发送："我是lin"
3. 等待回复
4. 发送："你是谁"
5. 检查回复是否正确提到"lin"，而不是"张三"或其他历史信息

---

## 🎯 成功标准

修复成功的判断标准：
1. ✅ Agent能记住当前会话的用户名（lin）
2. ✅ Agent不会混入其他会话的信息（张三、北京等）
3. ✅ Working Memory正确记录当前会话
4. ✅ 长期记忆可以访问，但不干扰当前会话
5. ✅ 多轮对话保持一致的上下文

---

**报告完成时间：** 2024-11-03 09:58
**修复状态：** ✅ 代码修复完成，等待最终验证

