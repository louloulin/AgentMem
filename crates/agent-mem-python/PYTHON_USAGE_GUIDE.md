# AgentMem Python 绑定使用指南

**版本**: v1.0  
**状态**: ✅ 代码完成，待环境验证  
**最后更新**: 2025-10-27

---

## 快速开始

### 安装

```bash
# 方法1: 从源码安装（推荐开发者）
cd crates/agent-mem-python
pip install maturin
maturin develop

# 方法2: 从 wheel 安装（推荐用户）
pip install agentmem-native
```

### 基础使用

```python
import asyncio
from agentmem_native import Memory

async def main():
    # 1. 创建 Memory 实例
    memory = Memory()
    
    # 2. 添加记忆
    memory_id = await memory.add("I love pizza")
    print(f"Added memory with ID: {memory_id}")
    
    # 3. 搜索记忆
    results = await memory.search("pizza")
    print(f"Found {len(results)} memories")
    
    # 4. 获取所有记忆
    all_memories = await memory.get_all()
    print(f"Total memories: {len(all_memories)}")
    
    # 5. 删除记忆
    await memory.delete(memory_id)
    
    # 6. 清空所有记忆
    count = await memory.clear()
    print(f"Cleared {count} memories")

if __name__ == "__main__":
    asyncio.run(main())
```

---

## API 参考

### Memory 类

#### `Memory()`

创建一个新的 Memory 实例。

**返回**: Memory 对象

**示例**:
```python
memory = Memory()
```

---

#### `async add(content: str) -> str`

添加一条记忆。

**参数**:
- `content` (str): 记忆内容

**返回**: str - 记忆 ID

**示例**:
```python
memory_id = await memory.add("I like programming in Rust")
print(f"Memory ID: {memory_id}")
```

---

#### `async search(query: str, limit: int = 5) -> List[Dict]`

搜索记忆。

**参数**:
- `query` (str): 搜索查询
- `limit` (int, 可选): 返回结果数量，默认 5

**返回**: List[Dict] - 记忆列表，每个记忆包含：
  - `id` (str): 记忆 ID
  - `content` (str): 记忆内容
  - `created_at` (str): 创建时间
  - `score` (float, 可选): 相关性分数

**示例**:
```python
results = await memory.search("Rust", limit=10)
for result in results:
    print(f"[{result['score']}] {result['content']}")
```

---

#### `async get_all() -> List[Dict]`

获取所有记忆。

**返回**: List[Dict] - 所有记忆列表

**示例**:
```python
all_memories = await memory.get_all()
print(f"Total: {len(all_memories)} memories")
```

---

#### `async delete(memory_id: str) -> bool`

删除指定记忆。

**参数**:
- `memory_id` (str): 要删除的记忆 ID

**返回**: bool - 是否成功删除

**示例**:
```python
success = await memory.delete("mem_123456")
if success:
    print("Memory deleted successfully")
```

---

#### `async clear() -> int`

清空所有记忆。

**返回**: int - 删除的记忆数量

**示例**:
```python
count = await memory.clear()
print(f"Cleared {count} memories")
```

---

## 使用场景

### 场景1: 智能对话助手

```python
import asyncio
from agentmem_native import Memory

class ChatBot:
    def __init__(self):
        self.memory = Memory()
    
    async def remember(self, message: str):
        """记住用户说的话"""
        return await self.memory.add(message)
    
    async def recall(self, query: str) -> List[str]:
        """回忆相关内容"""
        results = await self.memory.search(query, limit=3)
        return [r['content'] for r in results]
    
    async def chat(self, user_input: str) -> str:
        # 1. 搜索相关记忆
        context = await self.recall(user_input)
        
        # 2. 记住这次对话
        await self.remember(f"User said: {user_input}")
        
        # 3. 生成响应（这里简化）
        if context:
            return f"I remember: {context[0]}"
        else:
            return "Tell me more!"

async def main():
    bot = ChatBot()
    
    # 对话1
    response = await bot.chat("I love pizza")
    print(f"Bot: {response}")
    
    # 对话2
    response = await bot.chat("What do I like?")
    print(f"Bot: {response}")

if __name__ == "__main__":
    asyncio.run(main())
```

---

### 场景2: 知识库管理

```python
import asyncio
from agentmem_native import Memory
from typing import List, Dict

class KnowledgeBase:
    def __init__(self):
        self.memory = Memory()
    
    async def add_fact(self, fact: str) -> str:
        """添加知识条目"""
        return await self.memory.add(fact)
    
    async def search_knowledge(self, query: str, limit: int = 5) -> List[Dict]:
        """搜索知识"""
        return await self.memory.search(query, limit=limit)
    
    async def get_all_facts(self) -> List[Dict]:
        """获取所有知识"""
        return await self.memory.get_all()
    
    async def update_fact(self, fact_id: str, new_content: str) -> bool:
        """更新知识（删除旧的，添加新的）"""
        await self.memory.delete(fact_id)
        await self.add_fact(new_content)
        return True

async def main():
    kb = KnowledgeBase()
    
    # 添加知识
    await kb.add_fact("Rust是一门系统编程语言")
    await kb.add_fact("Python适合快速开发")
    await kb.add_fact("Go语言擅长并发编程")
    
    # 搜索知识
    results = await kb.search_knowledge("编程语言")
    for result in results:
        print(f"- {result['content']}")
    
    # 获取所有知识
    all_facts = await kb.get_all_facts()
    print(f"\nTotal knowledge: {len(all_facts)} entries")

if __name__ == "__main__":
    asyncio.run(main())
```

---

### 场景3: 用户偏好记忆

```python
import asyncio
from agentmem_native import Memory
from datetime import datetime

class UserPreferences:
    def __init__(self, user_id: str):
        self.user_id = user_id
        self.memory = Memory()
    
    async def save_preference(self, category: str, value: str):
        """保存用户偏好"""
        timestamp = datetime.now().isoformat()
        content = f"[{category}] {value} (saved at {timestamp})"
        return await self.memory.add(content)
    
    async def get_preferences(self, category: str = None) -> List[str]:
        """获取用户偏好"""
        if category:
            results = await self.memory.search(f"[{category}]", limit=10)
        else:
            results = await self.memory.get_all()
        
        return [r['content'] for r in results]
    
    async def clear_old_preferences(self):
        """清空旧偏好"""
        return await self.memory.clear()

async def main():
    prefs = UserPreferences("user_001")
    
    # 保存偏好
    await prefs.save_preference("food", "I love pizza")
    await prefs.save_preference("hobby", "I enjoy hiking")
    await prefs.save_preference("food", "I like Chinese food")
    
    # 获取特定类别偏好
    food_prefs = await prefs.get_preferences("food")
    print("Food preferences:")
    for pref in food_prefs:
        print(f"  - {pref}")
    
    # 获取所有偏好
    all_prefs = await prefs.get_preferences()
    print(f"\nTotal preferences: {len(all_prefs)}")

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 高级用法

### 批量操作

```python
import asyncio
from agentmem_native import Memory

async def batch_add(memory: Memory, contents: List[str]):
    """批量添加记忆"""
    tasks = [memory.add(content) for content in contents]
    return await asyncio.gather(*tasks)

async def main():
    memory = Memory()
    
    # 批量添加
    contents = [
        "Memory 1",
        "Memory 2",
        "Memory 3",
    ]
    
    memory_ids = await batch_add(memory, contents)
    print(f"Added {len(memory_ids)} memories")

if __name__ == "__main__":
    asyncio.run(main())
```

---

### 多语言支持

```python
import asyncio
from agentmem_native import Memory

async def main():
    memory = Memory()
    
    # 中文
    await memory.add("我喜欢编程")
    results = await memory.search("编程")
    print(f"中文搜索: {len(results)} 结果")
    
    # 英文
    await memory.add("I love programming")
    results = await memory.search("programming")
    print(f"English search: {len(results)} results")
    
    # 日文
    await memory.add("プログラミングが好きです")
    results = await memory.search("プログラミング")
    print(f"日本語検索: {len(results)} 結果")

if __name__ == "__main__":
    asyncio.run(main())
```

---

### 错误处理

```python
import asyncio
from agentmem_native import Memory

async def safe_add(memory: Memory, content: str) -> Optional[str]:
    """安全地添加记忆"""
    try:
        memory_id = await memory.add(content)
        return memory_id
    except Exception as e:
        print(f"Error adding memory: {e}")
        return None

async def safe_search(memory: Memory, query: str) -> List[Dict]:
    """安全地搜索记忆"""
    try:
        results = await memory.search(query)
        return results
    except Exception as e:
        print(f"Error searching: {e}")
        return []

async def main():
    memory = Memory()
    
    # 使用安全包装
    memory_id = await safe_add(memory, "Test content")
    if memory_id:
        print(f"Successfully added: {memory_id}")
    
    results = await safe_search(memory, "test")
    print(f"Found {len(results)} results")

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 性能优化

### 1. 使用连接池

```python
# Memory 内部已经使用了连接池，无需额外配置
memory = Memory()
```

### 2. 批量操作

```python
# 使用 asyncio.gather 进行并发操作
tasks = [memory.add(content) for content in contents]
results = await asyncio.gather(*tasks)
```

### 3. 限制搜索结果

```python
# 使用 limit 参数限制返回数量
results = await memory.search("query", limit=10)
```

---

## 测试

完整的测试套件位于 `tests/test_python_bindings.py`，包含 16 个测试：

```bash
# 运行所有测试
pytest tests/test_python_bindings.py -v

# 运行特定测试
pytest tests/test_python_bindings.py::test_add_memory -v
```

**测试覆盖**:
- ✅ 基础操作（add、search、get_all、delete、clear）
- ✅ 工作流测试（完整使用流程）
- ✅ 边界情况（空搜索、无匹配）
- ✅ 多实例测试
- ✅ 多语言支持（中文、英文）
- ✅ 特殊字符处理
- ✅ 长文本处理

---

## 常见问题

### Q: 为什么所有方法都是异步的？

**A**: AgentMem 使用 Rust 的异步 I/O（Tokio），为了保持性能优势，Python 绑定也采用异步接口。

### Q: 如何在同步代码中使用？

**A**: 使用 `asyncio.run()` 包装：

```python
import asyncio
from agentmem_native import Memory

def sync_add(content: str) -> str:
    memory = Memory()
    return asyncio.run(memory.add(content))

# 使用
memory_id = sync_add("Test content")
```

### Q: 支持哪些 Python 版本？

**A**: Python 3.8+ （由 PyO3 决定）

### Q: 数据存储在哪里？

**A**: 默认使用嵌入式 LibSQL 数据库，存储在本地文件系统。

### Q: 如何配置数据库路径？

**A**: 当前版本使用默认配置，未来版本将支持自定义配置。

---

## 构建说明

### 从源码构建

```bash
# 1. 安装依赖
pip install maturin

# 2. 开发模式构建
cd crates/agent-mem-python
maturin develop

# 3. Release 模式构建
maturin build --release

# 4. 发布到 PyPI
maturin publish
```

### 依赖要求

- Rust 1.70+
- Python 3.8+
- PyO3 0.21+
- maturin 1.0+

---

## 路线图

### v1.1（计划中）
- [ ] 支持自定义配置
- [ ] 添加批量操作 API
- [ ] 性能优化（连接池配置）

### v1.2（计划中）
- [ ] 支持流式搜索
- [ ] 添加记忆更新 API
- [ ] 支持元数据过滤

### v2.0（计划中）
- [ ] 图记忆支持
- [ ] 多模态记忆支持
- [ ] 分布式部署支持

---

## 贡献

欢迎贡献！请参阅主项目的 [CONTRIBUTING.md](../../CONTRIBUTING.md)。

---

## 许可证

与主项目相同的许可证。

---

## 相关链接

- 主项目: [AgentMem](../../README.md)
- Rust API 文档: [docs.rs/agent-mem](https://docs.rs/agent-mem)
- 问题跟踪: [GitHub Issues](https://github.com/louloulin/agentmem/issues)

---

**状态**: ✅ 代码完成  
**测试**: ✅ 16 个测试覆盖  
**文档**: ✅ 完整使用指南  
**最后更新**: 2025-10-27

