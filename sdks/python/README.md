# AgentMem Python SDK

Official Python client library for AgentMem - Enterprise-grade memory management for AI agents.

## 🎯 Features

- ✅ **Complete CRUD Operations**: Add, get, update, delete memories
- ✅ **Advanced Search**: Semantic and hybrid search with filtering
- ✅ **Batch Operations**: Bulk add and delete for efficiency
- ✅ **Memory History**: Track changes over time
- ✅ **Statistics**: Get insights into memory usage
- ✅ **Type Safety**: Full type hints and validation
- ✅ **Async Support**: Built on `httpx` for async/await
- ✅ **Retry Logic**: Automatic retry with exponential backoff
- ✅ **Caching**: Optional response caching for GET requests
- ✅ **Error Handling**: Comprehensive error types

## 📦 Installation

```bash
pip install agentmem
```

Or install from source:

```bash
git clone https://github.com/louloulin/agentmem
cd agentmem/sdks/python
pip install -e .
```

## 🚀 Quick Start

```python
import asyncio
from agentmem import AgentMemClient, Config, MemoryType

async def main():
    # Initialize client
    config = Config(
        api_base_url="http://localhost:8080",
        api_key="your_api_key",
    )
    
    async with AgentMemClient(config) as client:
        # Add a memory
        memory_id = await client.add_memory(
            content="I love pizza",
            agent_id="agent_1",
            user_id="alice",
            memory_type=MemoryType.EPISODIC,
            importance=0.8,
        )
        print(f"Memory created: {memory_id}")
        
        # Search memories
        from agentmem import SearchQuery
        results = await client.search_memories(
            SearchQuery(
                query="pizza",
                user_id="alice",
                limit=10,
                threshold=0.7,
            )
        )
        print(f"Found {len(results)} memories")
        
        # Update memory
        await client.update_memory(
            memory_id,
            content="I love pasta",
            importance=0.9,
        )
        
        # Delete memory
        await client.delete_memory(memory_id)

asyncio.run(main())
```

## 📚 API Reference

### Core Methods

#### `add_memory()`
Add a new memory to the system.

```python
memory_id = await client.add_memory(
    content="Important information",
    agent_id="agent_1",
    user_id="alice",
    memory_type=MemoryType.SEMANTIC,
    importance=0.8,
    metadata={"source": "conversation"},
)
```

#### `get_memory()`
Retrieve a specific memory by ID.

```python
memory = await client.get_memory(memory_id)
print(memory.content)
```

#### `update_memory()`
Update an existing memory.

```python
await client.update_memory(
    memory_id,
    content="Updated content",
    importance=0.9,
)
```

#### `delete_memory()`
Delete a memory.

```python
await client.delete_memory(memory_id)
```

#### `search_memories()`
Search for memories using semantic search.

```python
from agentmem import SearchQuery

results = await client.search_memories(
    SearchQuery(
        query="project requirements",
        user_id="alice",
        limit=10,
        threshold=0.7,
    )
)
```

### Batch Operations

#### `batch_add_memories()`
Add multiple memories at once.

```python
memories = [
    {
        "content": "Memory 1",
        "agent_id": "agent_1",
        "user_id": "alice",
    },
    {
        "content": "Memory 2",
        "agent_id": "agent_1",
        "user_id": "alice",
    },
]

ids = await client.batch_add_memories(memories)
```

#### `batch_delete_memories()`
Delete multiple memories at once.

```python
await client.batch_delete_memories([id1, id2, id3])
```

### Advanced Features

#### `get_memory_history()`
Get the change history of a memory.

```python
history = await client.get_memory_history(memory_id)
print(history["history"])
```

#### `get_all_memories()`
Get all memories with optional filters.

```python
memories = await client.get_all_memories(
    user_id="alice",
    limit=100,
)
```

#### `get_memory_stats()`
Get statistics about memory usage.

```python
stats = await client.get_memory_stats()
print(f"Total memories: {stats.total_memories}")
```

### Monitoring

#### `health_check()`
Check the health of the AgentMem service.

```python
health = await client.health_check()
print(health["status"])
```

#### `get_metrics()`
Get system metrics.

```python
metrics = await client.get_metrics()
```

## 🔧 Configuration

```python
from agentmem import Config

config = Config(
    api_base_url="http://localhost:8080",
    api_key="your_api_key",
    timeout=30.0,
    max_retries=3,
    retry_delay=1.0,
    enable_caching=True,
    cache_ttl=60,
    enable_logging=True,
    log_level="INFO",
)
```

### Environment Variables

You can also configure using environment variables:

```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key
export AGENTMEM_TIMEOUT=30
export AGENTMEM_MAX_RETRIES=3
```

Then:

```python
config = Config.from_env()
client = AgentMemClient(config)
```

## 🎯 Memory Types

```python
from agentmem import MemoryType

# Available memory types:
MemoryType.EPISODIC    # Event-based memories
MemoryType.SEMANTIC    # Factual knowledge
MemoryType.PROCEDURAL  # Skills and procedures
MemoryType.WORKING     # Short-term working memory
MemoryType.CORE        # Core/persistent memories
MemoryType.UNTYPED     # Unclassified memories
```

## ⚡ Advanced Usage

### Context Manager

```python
async with AgentMemClient(config) as client:
    memory_id = await client.add_memory(...)
    # Client will automatically close on exit
```

### Error Handling

```python
from agentmem import (
    AgentMemError,
    AuthenticationError,
    ValidationError,
    NotFoundError,
    RateLimitError,
    NetworkError,
)

try:
    await client.add_memory(...)
except AuthenticationError:
    print("Invalid API key")
except ValidationError as e:
    print(f"Invalid request: {e}")
except RateLimitError:
    print("Rate limit exceeded")
except NotFoundError:
    print("Memory not found")
except NetworkError as e:
    print(f"Network error: {e}")
```

### Caching

```python
# Enable caching for GET requests
config = Config(
    api_base_url="http://localhost:8080",
    api_key="your_api_key",
    enable_caching=True,
    cache_ttl=60,  # 60 seconds
)

client = AgentMemClient(config)

# First call hits the API
memory = await client.get_memory(memory_id)

# Second call uses cache (within TTL)
memory = await client.get_memory(memory_id)  # Cached!
```

## 📊 What's New in v7.0.0

### 🎉 Server统一API兼容 (2025-10-23)

- ✅ **API端点更新**: 所有端点已更新为`/api/v1/*`格式
- ✅ **新增方法**: 
  - `batch_delete_memories()` - 批量删除记忆
  - `get_memory_history()` - 获取记忆变更历史
  - `get_all_memories()` - 获取所有记忆（带过滤）
- ✅ **完全兼容**: 与Server Memory统一API 100%兼容
- ✅ **向后兼容**: 保持与旧版本的兼容性

### 架构改进

- Server已迁移到Memory统一API
- 全栈使用相同的Memory接口
- 自动智能功能集成
- 类型安全增强

## 🧪 Testing

Run tests:

```bash
cd sdks/python
python verify_sdk_structure.py  # 结构验证
pytest tests/                    # 单元测试（需要安装pytest）
```

## 📖 Documentation

- [API Documentation](https://agentmem.cc)
- [Examples](./examples/)
- [Main Project](https://github.com/louloulin/agentmem)

## 🤝 Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔗 Links

- **GitHub**: https://github.com/louloulin/agentmem
- **Documentation**: https://agentmem.cc
- **Issues**: https://github.com/louloulin/agentmem/issues

## 📞 Support

- Email: support@agentmem.dev
- Discord: https://discord.gg/agentmem
- GitHub Issues: https://github.com/louloulin/agentmem/issues

---

**AgentMem** - Enterprise-grade memory management for AI agents 🚀
