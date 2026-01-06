# Clone Optimization Guide for AgentMem

## Current State
- Total clones: 4,109
- Target: Reduce by 50-70% through systematic optimization

## Clone Reduction Strategies

### 1. Use References Instead of Cloning

**Before:**
```rust
fn process_string(s: String) {
    println!("{}", s);
}

let s = String::from("hello");
process_string(s.clone());
```

**After:**
```rust
fn process_string(s: &str) {
    println!("{}", s);
}

let s = String::from("hello");
process_string(&s);
```

### 2. Use Borrowing in Loops

**Before:**
```rust
for item in items {
    process(item.clone());
}
```

**After:**
```rust
for item in &items {
    process(item);
}
```

### 3. Use Cow (Clone-on-Write)

**Before:**
```rust
fn maybe_modify(s: String) -> String {
    if condition {
        s.to_uppercase()
    } else {
        s
    }
}
```

**After:**
```rust
use std::borrow::Cow;

fn maybe_modify(s: &str) -> Cow<str> {
    if condition {
        Cow::Owned(s.to_uppercase())
    } else {
        Cow::Borrowed(s)
    }
}
```

### 4. Use Arc for Shared Data

**Before:**
```rust
struct SharedConfig {
    data: Vec<u8>,
}

impl Clone for SharedConfig {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(), // Expensive!
        }
    }
}
```

**After:**
```rust
use std::sync::Arc;

struct SharedConfig {
    data: Arc<Vec<u8>>,
}

// Clone is cheap (just increments reference count)
```

### 5. Use Rc for Single-Threaded Scenarios

**Before:**
```rust
struct TreeNode {
    value: String,
    children: Vec<TreeNode>,
}
```

**After:**
```rust
use std::rc::Rc;

struct TreeNode {
    value: String,
    children: Vec<Rc<TreeNode>>,
}
```

### 6. Avoid Unnecessary Struct Field Clones

**Before:**
```rust
impl MyStruct {
    fn get_field(&self) -> String {
        self.field.clone()
    }
}
```

**After:**
```rust
impl MyStruct {
    fn get_field(&self) -> &str {
        &self.field
    }
}
```

### 7. Use Iterators Instead of collect+clone

**Before:**
```rust
let items: Vec<_> = items.iter().cloned().collect();
for item in items {
    process(item);
}
```

**After:**
```rust
for item in &items {
    process(item);
}
```

### 8. Copy Small Types

**Before:**
```rust
#[derive(Clone)]
struct Point {
    x: u32,
    y: u32,
}

// Clones Point even though it could be copied
```

**After:**
```rust
#[derive(Clone, Copy)]
struct Point {
    x: u32,
    y: u32,
}

// Now uses cheap memcpy
```

## Common Patterns in AgentMem

### Pattern 1: Memory Access

**Problem:**
```rust
// In agent-mem-core/src/memory.rs
impl Memory {
    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}
```

**Solution:**
```rust
impl Memory {
    pub fn get_content(&self) -> &str {
        &self.content
    }
}
```

### Pattern 2: Vector Operations

**Problem:**
```rust
for memory in &memories {
    self.store.add(memory.clone()).await?;
}
```

**Solution:**
```rust
for memory in &memories {
    self.store.add(memory).await?; // Takes reference
}
```

### Pattern 3: Error Handling

**Problem:**
```rust
Err(AgentMemError::NotFound {
    message: format!("Memory {} not found", id.clone()),
})
```

**Solution:**
```rust
Err(AgentMemError::NotFound {
    message: format!("Memory {} not found", id), // No clone needed
})
```

### Pattern 4: HashMap Access

**Problem:**
```rust
let value = map.get(&key).cloned().unwrap();
```

**Solution:**
```rust
let value = map.get(&key).ok_or_else(|| Error::NotFound)?;
// Or if you really need it:
let value = map.get(&key).copied().unwrap(); // for Copy types
```

## Automated Detection

Run this script to find clone hotspots:

```bash
#!/bin/bash
# Find files with most clones

echo "🔍 Finding clone hotspots..."
echo ""

for file in $(find crates -name "*.rs"); do
    count=$(grep -c "\.clone()" "$file" || echo "0")
    if [ "$count" -gt 10 ]; then
        echo "$file: $count clones"
    fi
done | sort -t: -k2 -rn | head -20
```

## Performance Impact

| Optimization | Effort | Impact | Priority |
|-------------|--------|--------|----------|
| Use references | Low | High | P0 |
| Arc for shared data | Medium | High | P0 |
| Borrow in loops | Low | Medium | P1 |
| Cow for conditionals | Medium | Medium | P1 |
| Copy for small types | Low | Low | P2 |
| Rc in single-threaded | Low | Low | P2 |

## Testing After Optimization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_unnecessary_clones() {
        // This test should compile and pass
        let data = String::from("hello");
        let result = process(&data);
        assert_eq!(result, "hello");
        assert_eq!(data, "hello"); // Original still valid
    }
}
```

## Measurement

Before and after optimization:

```rust
use std::time::Instant;

fn benchmark_clones() {
    let start = Instant::now();

    // Your code here

    let duration = start.elapsed();
    println!("Time: {:?}", duration);
}
```

## Summary Checklist

- [ ] Audit all `.clone()` calls
- [ ] Replace with references where possible
- [ ] Use `Arc<T>` for shared ownership
- [ ] Use `&str` instead of `String` in function parameters
- [ ] Use `&[T]` instead of `Vec<T>` where possible
- [ ] Add `#[derive(Copy)]` to small structs
- [ ] Run benchmarks to verify improvements
- [ ] Update tests to verify correctness

## Expected Results

- **Initial**: 4,109 clones
- **Target (P0)**: ~2,000 clones (50% reduction)
- **Target (P1)**: ~1,200 clones (70% reduction)
- **Stretch**: <500 clones (88% reduction)

## Resources

- [Rust Ownership Guide](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Cow Documentation](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [Performance Guide](https://nnethercote.github.io/perf-book/introduction.html)
