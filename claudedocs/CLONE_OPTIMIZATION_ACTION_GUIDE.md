# Clone 优化实战指南

## 🎯 目标

**当前**: 4,109 clones
**目标**: ~1,200 clones (-70%)
**时间**: 2周 (分2轮)

---

## 📊 第1轮: 低风险快速优化 (本周)

### Pattern 1: 函数签名 String → &str

**影响**: 高
**风险**: 极低
**工具**: 半自动

#### 查找热点

```bash
# 找到 clone 最多的函数
grep -rn "String" crates/agent-mem-core/src --include="*.rs" | \
  grep "fn.*String" | \
  head -20
```

#### 修复模板

```rust
// Before ❌
pub async fn add_memory(
    &self,
    user_id: String,  // ← caller clone()
    content: String,  // ← caller clone()
    metadata: Option<HashMap<String, String>>,
) -> CoreResult<String> {
    self.storage.add(user_id, content, metadata).await
}

// After ✅
pub async fn add_memory(
    &self,
    user_id: &str,  // ← zero copy
    content: &str,  // ← zero copy
    metadata: Option<&HashMap<String, String>>,
) -> CoreResult<String> {
    self.storage.add(user_id, content, metadata).await
}

// Caller side - no change needed!
client.add_memory("user123", "Hello", None).await?
```

### Pattern 2: Vec → &[T]

**影响**: 中
**风险**: 低
**工具**: 手动

```rust
// Before ❌
pub fn filter_memories(
    &self,
    items: Vec<Memory>,  // ← ownership transfer
) -> Vec<Memory> {
    items.into_iter().filter(|m| m.active).collect()
}

// After ✅
pub fn filter_memories(
    &self,
    items: &[Memory],  // ← borrow, zero copy
) -> Vec<Memory> {
    items.iter().filter(|m| m.active).cloned().collect()
}
```

### Pattern 3: .clone().deref() → .as_ref()

**影响**: 低
**风险**: 极低
**工具**: 自动

```bash
find crates/agent-mem-core/src -name "*.rs" -type f -exec sed -i '' \
    's/\.clone()\.deref()/.as_ref()/g' {} \;
```

### Pattern 4: 循环中的不必要 clone

**影响**: 高
**风险**: 中
**工具**: 手动

```rust
// Before ❌
for item in &items {
    process(item.clone()).await?;  // ← clone per iteration
}

// After ✅
for item in &items {
    process(item).await?;  // ← pass reference
}

// Or if process needs ownership:
for item in items.into_iter() {
    process(item).await?;  // ← move, no clone
}
```

---

## 📊 第2轮: 架构级优化 (下周)

### Pattern 5: Arc<T> 共享所有权

**影响**: 极高
**风险**: 中

```rust
// Before ❌
pub struct MemoryManager {
    config: ManagerConfig,
    embeddings: Vec<Embedding>,
    lookup: HashMap<String, Memory>,
}

impl MemoryManager {
    pub async fn search(&self) -> Vec<Memory> {
        // 每次都 clone
        self.embeddings.clone()
        self.lookup.values().cloned().collect()
    }
}

// After ✅
use std::sync::Arc;

pub struct MemoryManager {
    config: Arc<ManagerConfig>,  // ← shared, cheap clone
    embeddings: Arc<Vec<Embedding>>,  // ← shared
    lookup: Arc<HashMap<String, Memory>>,  // ← shared
}
```

### Pattern 6: Cow<T> 条件克隆

```rust
// Before ❌
pub fn normalize(mut input: String) -> String {
    if needs_cleanup(&input) {
        input = cleanup(input);
    }
    input  // ← always allocated
}

// After ✅
use std::borrow::Cow;

pub fn normalize(input: &str) -> Cow<'_, str> {
    if needs_cleanup(input) {
        Cow::Owned(cleanup(input.to_string()))
    } else {
        Cow::Borrowed(input)  // ← zero allocation
    }
}
```

---

## 🚀 执行计划

### 第1轮 (本周 - 3天)

| Day | 任务 | 预期减少 |
|-----|------|---------|
| **Day 1** | Pattern 1: String→&str | -300 |
| **Day 2** | Pattern 2: Vec→&[T] + Pattern 3 | -200 |
| **Day 3** | Pattern 4: 循环优化 | -200 |
| **总计** | | **-700 (-17%)** |

### 第2轮 (下周 - 4天)

| Day | 任务 | 预期减少 |
|-----|------|---------|
| **Day 1** | Pattern 5: Arc重构 | -400 |
| **Day 2** | Pattern 6: Cow优化 | -150 |
| **Day 3** | Pattern 7: 迭代器 | -150 |
| **Day 4** | 验证+benchmark | - |
| **总计** | | **-700 (-34% 累计)** |

---

## 📈 进度追踪

### 每日检查

```bash
# Count current clones
echo "当前 clone 数:"
grep -r "\.clone()" crates/agent-mem-core/src --include="*.rs" | wc -l

# Compare to baseline (4,109)
echo "减少数量: $((4109 - $(grep -r "\.clone()" crates/agent-mem-core/src --include="*.rs" | wc -l | tr -d ' ')))"
```

---

## ⚠️ 风险管理

### 禁止自动修复的模式

```rust
// ❌ Don't change:
- unsafe code
- FFI boundaries
- trait implementations
- public API (without semver bump)

// ✅ Safe to change:
- internal functions
- private methods
- local variables
- loops
```

---

## 📊 成功指标

### Round 1 完成 (本周)

- [ ] Clone: 4,109 → ~3,400 (-17%)
- [ ] Tests: 100% passing
- [ ] Benchmark: +10% throughput

### Round 2 完成 (下周)

- [ ] Clone: ~3,400 → ~2,700 (-34% 累计)
- [ ] Tests: 100% passing
- [ ] Benchmark: +20% throughput
- [ ] Memory: -15% RSS

### Phase 1 完成 (2周)

- [ ] Clone: 4,109 → ~1,200 (-70%)
- [ ] Tests: 100% passing
- [ ] Benchmark: +30% throughput
- [ ] Memory: -25% RSS

---

## 🔧 实用工具

### Hotspot finder

```bash
cat > scripts/find_clone_hotspots.sh << 'EOF'
#!/bin/bash
echo "🔍 Clone hotspots 分析"
echo ""

for file in crates/agent-mem-core/src/**/*.rs; do
    count=$(grep -c "\.clone()" "$file" 2>/dev/null || echo 0)
    if [ "$count" -gt 10 ]; then
        echo "$file: $count clones"
    fi
done | sort -t: -k2 -rn | head -20
EOF

chmod +x scripts/find_clone_hotspots.sh
./scripts/find_clone_hotspots.sh
```

---

**下一步**: 执行 `./scripts/find_clone_hotspots.sh` 找到热点,开始Day 1优化

**时间**: 本周一、二、三
**目标**: -700 clones (-17%)
