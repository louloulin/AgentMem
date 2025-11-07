# AgentMem Phase 1 + 1.5 前端UI测试指南

**日期**: 2025-11-07
**版本**: v3.2
**目的**: 验证 Episodic-first 检索策略和跨 Session 记忆连续性

---

## 🚀 快速开始

### 1. 检查服务状态

```bash
# 检查后端服务（应在端口 8080）
curl http://localhost:8080/health | jq '.'

# 检查前端服务（应在端口 3001）
curl http://localhost:3001 | head -20

# 检查进程
ps aux | grep -E "(agent-mem-server|node.*3001)"
```

**预期结果**:
- ✅ 后端返回 `{"status":"healthy",...}`
- ✅ 前端返回 HTML
- ✅ 两个进程都在运行

---

## 📝 测试用例

### 测试 1: Session内记忆（Working Memory）

**目的**: 验证 Working Memory 在当前 Session 内工作正常

**步骤**:

1. 打开浏览器访问 http://localhost:3001
2. 在对话框输入：
   ```
   我喜欢吃pizza
   ```
3. 等待 AI 回复
4. 继续输入：
   ```
   我刚才说喜欢吃什么？
   ```

**预期结果**:
```
✅ AI应该回复类似：
   "您刚才提到喜欢吃pizza"
   
✅ 这验证了Working Memory（Session scope）正常工作
```

**如果失败**:
- ❌ 如果AI回复"不知道"，说明Session内记忆也有问题
- 🔧 检查后端日志：`tail -f agentmen/logs/*.log`

---

### 测试 2: 跨 Session 记忆（Episodic Memory） ⭐ 重点

**目的**: 验证 Phase 1 的 Episodic-first 检索策略

**步骤**:

1. 在对话框输入：
   ```
   我的生日是1990年1月1日
   ```
2. 等待 AI 确认
3. **刷新页面**（这会创建新的 Session）
4. 在新 Session 输入：
   ```
   我的生日是哪天？
   ```

**预期结果** (Phase 1 修复后):
```
✅ AI应该回复类似：
   "您的生日是1990年1月1日"
   
✅ 这验证了Episodic Memory（User scope）跨Session工作！
✅ 这就是Phase 1的核心改进！
```

**如果失败** (修复前的行为):
```
❌ AI回复：
   "抱歉，我不知道您的生日"
   
❌ 说明还在使用旧的Session优先策略
🔧 需要检查代码是否正确调用 retrieve_episodic_first
```

---

### 测试 3: 认知架构日志验证 🧠

**目的**: 验证 Episodic-first 策略正确执行

**步骤**:

1. 在另一个终端窗口打开日志：
   ```bash
   cd agentmen
   tail -f logs/*.log | grep -E "(Episodic|Working|Priority)"
   ```

2. 在前端进行对话（可以是测试2）

3. 观察日志输出

**预期日志** (Phase 1 实施后):
```
✅ 应该看到类似：
   INFO 🔍 Episodic-first retrieval: agent=xxx, user=xxx, ...
   INFO Priority 1: Querying Episodic Memory (User scope)
   INFO Priority 1: Episodic Memory returned X memories
   INFO Priority 2: Querying Working Memory (Session scope) as context
   INFO Priority 2: Working Memory added Y memories
   INFO ✅ Retrieval complete: Z memories (Episodic: X, Working: Y)
```

**如果没有看到**:
```
❌ 如果只看到：
   "retrieve_relevant_memories_with_session"
   
❌ 说明还在使用旧方法
🔧 检查 orchestrator/mod.rs 是否调用了新方法
```

---

### 测试 4: 多轮对话连续性

**目的**: 验证复杂场景下的记忆连贯性

**步骤**:

1. **第一轮对话** (Session A):
   ```
   用户: 我最喜欢的编程语言是Rust
   AI: [确认]
   
   用户: 我正在开发一个记忆管理平台
   AI: [确认]
   ```

2. **刷新页面** (创建 Session B)

3. **第二轮对话** (Session B):
   ```
   用户: 我最喜欢什么编程语言？
   AI: [应该回复"Rust"] ✅
   
   用户: 我在开发什么项目？
   AI: [应该回复"记忆管理平台"] ✅
   ```

4. **再次刷新** (创建 Session C)

5. **第三轮对话** (Session C):
   ```
   用户: 总结一下我之前告诉你的信息
   AI: [应该包含Rust和记忆管理平台] ✅
   ```

**预期结果**:
```
✅ 所有历史信息都能被检索到
✅ 跨多个Session保持连贯性
✅ 这证明Episodic Memory是主要检索来源
```

---

## 🔍 调试技巧

### 1. 查看完整日志

```bash
cd agentmen
tail -100 logs/*.log
```

### 2. 检查数据库记忆

```bash
cd agentmen
sqlite3 agentmem.db "SELECT id, content, memory_type, scope FROM memories LIMIT 10;"
```

### 3. 实时监控API请求

在浏览器开发者工具中：
1. 打开 Network 标签
2. 筛选 XHR/Fetch
3. 观察 `/api/v1/agents/*/chat` 请求
4. 查看请求/响应内容

### 4. 检查记忆检索数量

在日志中搜索：
```bash
grep "Retrieved.*memories" logs/*.log | tail -20
```

**期望看到**:
```
✅ "Retrieved 10 memories (Episodic-first) for user=xxx"
```

**不应该看到**:
```
❌ "Retrieved 0 memories for session=xxx"
```

---

## 📊 验证清单

### Phase 1 + 1.5 核心功能

- [ ] **编译验证**: ✅ 已完成（100%通过）
- [ ] **服务健康**: ✅ 已完成（后端+前端正常）
- [ ] **Session内记忆**: ⏳ 需要UI测试（测试1）
- [ ] **跨Session记忆**: ⏳ 需要UI测试（测试2）
- [ ] **认知架构日志**: ⏳ 需要查看日志（测试3）
- [ ] **多轮连续性**: ⏳ 需要UI测试（测试4）

### 理论一致性验证

- [ ] **Episodic优先**: ⏳ 查看日志确认
- [ ] **Working Memory补充**: ⏳ 查看日志确认
- [ ] **权重可配置**: ✅ 代码已验证

---

## 🎯 成功标准

### 最小成功标准 (必须)
- ✅ 测试2通过（跨Session记忆）
- ✅ 日志显示"Episodic-first"

### 完整成功标准 (推荐)
- ✅ 所有4个测试用例通过
- ✅ 认知架构日志完整
- ✅ 多轮对话连贯性

---

## 🚨 常见问题

### Q1: 刷新页面后AI"失忆"了

**原因**: 可能还在使用旧的 `retrieve_relevant_memories_with_session`

**解决**:
```bash
# 检查是否使用了新方法
cd agentmen/crates/agent-mem-core/src/orchestrator
grep "retrieve_episodic_first" mod.rs

# 应该看到：
# let memories = self.memory_integrator.retrieve_episodic_first(...)
```

### Q2: 没有看到"Episodic-first"日志

**原因**: 日志级别可能设置为WARN或ERROR

**解决**:
```bash
# 检查日志配置
export RUST_LOG=info
# 重启服务
```

### Q3: AI回复不包含历史信息

**可能原因**:
1. 记忆确实不存在（检查数据库）
2. LLM没有利用检索到的记忆
3. 检索阈值太高

**解决**:
```bash
# 查看检索到的记忆数量
grep "Retrieved.*memories" logs/*.log | tail -5

# 如果数量为0，检查数据库
sqlite3 agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id IS NOT NULL;"
```

---

## 📝 测试结果报告模板

完成测试后，请记录结果：

```markdown
## 测试结果

**测试时间**: 2025-11-07
**测试人**: [您的名字]

### 测试1: Session内记忆
- 状态: [ ] 通过 / [ ] 失败
- 备注: 

### 测试2: 跨Session记忆 ⭐
- 状态: [ ] 通过 / [ ] 失败
- 备注:

### 测试3: 认知架构日志
- 状态: [ ] 通过 / [ ] 失败
- 看到的日志:
```

### 测试4: 多轮连续性
- 状态: [ ] 通过 / [ ] 失败
- 备注:

### 总结
- 通过的测试数: X/4
- Phase 1 + 1.5 是否成功: [ ] 是 / [ ] 否
- 建议:
```

---

## 🎉 预期结果

如果所有测试通过，您应该看到：

```
✅ AgentMem 现在拥有真正的"长期记忆"！
✅ 跨Session对话连贯性实现
✅ Episodic Memory成为主要检索来源
✅ Working Memory正确定位为补充角色
✅ 认知架构符合Atkinson-Shiffrin模型
```

这就是 **Phase 1 + 1.5 的核心价值**！

---

**下一步**: 
- 如果测试通过 → 更新 agentmem61.md 标记为"✅ 全部验证通过"
- 如果测试失败 → 分析日志，调试问题
- 可选 → 实施 Phase 2（策略配置）和 Phase 3（智能优化）

---

**文档版本**: 1.0
**创建时间**: 2025-11-07
**相关文档**:
- `agentmem61.md` - 主文档
- `COMPREHENSIVE_VERIFICATION_REPORT.md` - 综合验证报告
- `COMPREHENSIVE_MEMORY_ARCHITECTURE_ANALYSIS.md` - 架构分析

