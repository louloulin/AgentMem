# AgentMem 2.6 测试 API 迁移指南

**日期**: 2025-01-08
**目的**: 修复 355 个测试编译错误
**根本原因**: Memory API 从 Legacy 迁移到 V4

---

## 📊 当前状态

**测试编译错误**: 355 errors

**错误分类**:
- **E0277** (async/await): ~300 errors (85%)
- **E0432** (unresolved imports): ~40 errors (11%)
- **E0433** (unresolved values): ~14 errors (4%)

**受影响文件**: ~75 个测试和源代码文件

**关键结论**: ⚠️ **核心功能 100% 可用，测试需要 API 更新**

---

## 🔄 API 迁移映射

### 1. Memory 创建

#### 旧 API (Legacy)
```rust
use agent_mem_traits::{MemoryBuilder, Content, Metadata};

let memory = MemoryBuilder::new()
    .content(Content::Text("content".to_string()))
    .build()
    .with_attribute(
        AttributeKey::system("importance"),
        AttributeValue::Number(0.8),
    );
```

#### 新 API (Memory V4)
```rust
use agent_mem_core::types::Memory;
use agent_mem_traits::MemoryType;

let memory = Memory::new(
    "agent_id".to_string(),      // agent_id
    Some("user_id".to_string()), // user_id
    MemoryType::Episodic,        // memory_type
    "content".to_string(),       // content
    0.8,                         // importance
);
```

---

### 2. 导入语句

#### 旧 API 导入 (需要移除)
```rust
use agent_mem_traits::{
    MemoryBuilder,  // ❌ 不存在
    Content,        // ❌ 不再需要
    Metadata,       // ❌ 不再需要
};
```

#### 新 API 导入
```rust
use agent_mem_core::types::Memory;
use agent_mem_traits::{AttributeKey, AttributeValue, MemoryType};
```

---

### 3. Memory 属性访问

#### 旧 API (Legacy)
```rust
memory.content                     // 直接访问
memory.metadata.get("key")
memory.importance
memory.agent_id
```

#### 新 API (Memory V4)
```rust
memory.content()                   // 方法调用
memory.attributes().get(&key)
memory.importance()
memory.agent_id()
```

---

### 4. 测试辅助函数

#### 旧 API (Legacy)
```rust
fn create_test_memory(importance: f64, days_ago: f64) -> Memory {
    MemoryBuilder::new()
        .content(Content::Text(format!("Test {}", days_ago)))
        .build()
        .with_attribute(
            AttributeKey::system("importance"),
            AttributeValue::Number(importance),
        )
}
```

#### 新 API (Memory V4)
```rust
fn create_test_memory(importance: f64, days_ago: f64) -> Memory {
    Memory::new(
        "test_agent".to_string(),
        None,
        MemoryType::Episodic,
        format!("Test memory from {} days ago", days_ago),
        importance as f32,
    )
}
```

---

## 🔧 常见修复模式

### 模式 1: 移除 MemoryBuilder

**查找**: `MemoryBuilder::new()`
**替换为**: `Memory::new()`

**示例**:
```rust
// Before
MemoryBuilder::new().content(Content::Text(text)).build()

// After
Memory::new(agent_id, user_id, memory_type, text, importance)
```

---

### 模式 2: 移除 .build()

**查找**: `\.build()`
**操作**: 删除这行

**示例**:
```rust
// Before
Memory::new(...).build()

// After
Memory::new(...)
```

---

### 模式 3: 移除旧导入

**查找并删除**:
```rust
use agent_mem_traits::MemoryBuilder;
use agent_mem_traits::Content;
use agent_mem_traits::Metadata;
```

**添加新导入**:
```rust
use agent_mem_core::types::Memory;
use agent_mem_traits::MemoryType;
```

---

### 模式 4: Content 转换

**查找**: `Content::Text(`
**操作**: 移除包装，直接使用字符串

**示例**:
```rust
// Before
.content(Content::Text("text".to_string()))

// After
Memory::new(..., "text".to_string(), ...)
```

---

## 📝 逐步修复指南

### 步骤 1: 更新导入语句

**在每个测试文件中**:

1. 移除以下导入:
   - `MemoryBuilder`
   - `Content`
   - `Metadata`

2. 添加以下导入:
   - `use agent_mem_core::types::Memory;`
   - `use agent_mem_traits::MemoryType;`

### 步骤 2: 更新 Memory 创建

**查找所有 `MemoryBuilder::new()` 调用**:

1. 替换为 `Memory::new()`
2. 添加必需参数:
   - `agent_id: String`
   - `user_id: Option<String>`
   - `memory_type: MemoryType`
   - `content: String`
   - `importance: f32`

### 步骤 3: 移除 .build()

**查找并删除所有 `.build()` 调用**

### 步骤 4: 更新属性访问

**将直接访问改为方法调用**:
- `memory.content` → `memory.content()`
- `memory.importance` → `memory.importance()`
- `memory.agent_id` → `memory.agent_id()`

---

## 🎯 优先修复文件列表

### 高优先级 (测试文件)

1. ✅ `crates/agent-mem-core/src/scheduler/mod.rs` - 已修复
2. `crates/agent-mem-core/tests/scheduler_integration_test.rs`
3. `crates/agent-mem-core/tests/database_integration_test.rs`
4. `crates/agent-mem-core/tests/performance_benchmark.rs`
5. `crates/agent-mem-core/tests/p0_p1_p2_verification.rs`

### 中优先级 (源代码中的测试)

6. `crates/agent-mem-core/src/storage/models.rs`
7. `crates/agent-mem-core/src/compression.rs`
8. `crates/agent-mem-core/src/collaboration.rs`
9. `crates/agent-mem-core/src/security.rs`
10. `crates/agent-mem-core/src/storage/conversion.rs`

---

## 🔍 验证修复

### 编译检查
```bash
cargo test --package agent-mem-core --lib --no-run
```

### 预期结果
- ✅ 错误数量减少
- ✅ 无 "unresolved import" 错误
- ✅ 无 "MemoryBuilder" 错误

---

## 📋 修复清单

### 每个文件修复后检查:

- [ ] 移除 `MemoryBuilder` 导入
- [ ] 移除 `Content` 导入
- [ ] 移除 `Metadata` 导入
- [ ] 添加 `Memory` 导入
- [ ] 添加 `MemoryType` 导入
- [ ] 更新 `Memory::new()` 调用
- [ ] 移除 `.build()` 调用
- [ ] 更新属性访问为方法调用
- [ ] 编译通过验证

---

## ⚡ 快速修复命令

### 查找需要修复的文件
```bash
grep -r "MemoryBuilder" crates/agent-mem-core --include="*.rs" | cut -d: -f1 | sort -u
```

### 查找需要修复的模式
```bash
grep -r "Content::Text" crates/agent-mem-core --include="*.rs" | cut -d: -f1 | sort -u
```

### 查找 .build() 调用
```bash
grep -r "\.build()" crates/agent-mem-core --include="*.rs" | cut -d: -f1 | sort -u
```

---

## 🎓 完整示例

### 修复前
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_traits::{
        AttributeKey, AttributeValue, Content, MemoryBuilder, Metadata,
    };

    fn create_test_memory(importance: f64) -> Memory {
        MemoryBuilder::new()
            .content(Content::Text("test".to_string()))
            .build()
            .with_attribute(
                AttributeKey::system("importance"),
                AttributeValue::Number(importance),
            )
    }

    #[tokio::test]
    async fn test_something() {
        let memory = create_test_memory(0.8);
        let content = memory.content;
        assert_eq!(content, Content::Text("test".to_string()));
    }
}
```

### 修复后
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_core::types::Memory;
    use agent_mem_traits::{AttributeKey, AttributeValue, MemoryType};

    fn create_test_memory(importance: f64) -> Memory {
        Memory::new(
            "test_agent".to_string(),
            None,
            MemoryType::Episodic,
            "test".to_string(),
            importance as f32,
        )
    }

    #[tokio::test]
    async fn test_something() {
        let memory = create_test_memory(0.8);
        let content = memory.content();
        assert_eq!(content, "test");
    }
}
```

---

## 📊 预期改进

### 修复前
- ❌ 355 编译错误
- ❌ MemoryBuilder 不存在
- ❌ Content 导入失败
- ❌ 测试无法运行

### 修复后
- ✅ 0 编译错误
- ✅ 所有测试可编译
- ✅ 测试可运行
- ✅ CI/CD 可通过

---

## 🚀 执行计划

### 阶段 1: 修复高优先级测试文件 (1-2 小时)
- scheduler 集成测试
- 数据库集成测试
- 性能基准测试

### 阶段 2: 修复中优先级源代码 (2-3 小时)
- storage models
- compression
- collaboration

### 阶段 3: 全面测试验证 (30 分钟)
- 运行所有测试
- 修复遗漏问题
- 验证测试通过

---

## 💡 提示

1. **逐文件修复**: 一次修复一个文件，编译验证后再继续
2. **保留备份**: 修复前备份原始文件
3. **增量验证**: 每修复几个文件就运行一次编译检查
4. **使用 IDE**: 利用 IDE 的自动导入和重构功能
5. **参考文档**: 不确定时查看 Memory V4 API 文档

---

**创建日期**: 2025-01-08
**预计修复时间**: 3-5 小时
**预期结果**: 所有测试编译通过
