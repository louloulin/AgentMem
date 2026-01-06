# AgentMem Python SDK

Python bindings and integrations for AgentMem.

## Features

- 🚀 **Native Python API**: Simple, intuitive interface
- 🔗 **LangChain Integration**: Drop-in replacement for LangChain memory
- ⚡ **High Performance**: Async support for concurrent operations
- 🎯 **Type Hints**: Full type annotations for IDE support

## Installation

```bash
pip install agentmem
```

## Quick Start

### Native Python API

```python
import agentmem

# Initialize memory
memory = agentmem.Memory()

# Add memories
memory.add("I love pizza")
memory.add("I live in San Francisco")

# Search memories
results = memory.search("What do you know about me?")
for result in results:
    print(f"- {result['content']}")
```

### LangChain Integration

```python
from langchain.chains import ConversationChain
from agentmem.langchain import AgentMemMemory

# Initialize AgentMem memory
memory = AgentMemMemory(session_id="user-123")

# Use with LangChain
conversation = ConversationChain(
    llm=your_llm,
    memory=memory,
    verbose=True
)

result = conversation.run("Hello, I'm John!")
```

## API Reference

### Memory Client

#### `MemoryClient`

Synchronous client for AgentMem.

```python
from agentmem import MemoryClient

client = MemoryClient(
    base_url="http://localhost:8080",
    session_id="user-123"
)

# Add memory
client.add("I love coding", metadata={"category": "hobby"})

# Search memories
results = client.search("programming", top_k=5, threshold=0.7)

# Get all memories
all_memories = client.get_all()

# Delete memory
client.delete(memory_id="123")
```

#### `AsyncMemoryClient`

Async client for high-performance scenarios.

```python
import asyncio
from agentmem import AsyncMemoryClient

async def main():
    client = AsyncMemoryClient()
    
    # Add memory
    await client.add("Async memory operations")
    
    # Search
    results = await client.search("async")
    
    # Cleanup
    await client.close()

asyncio.run(main())
```

### LangChain Integration

#### `AgentMemMemory`

Main memory class for LangChain.

```python
from agentmem.langchain import AgentMemMemory

memory = AgentMemMemory(
    session_id="user-123",
    backend_url="http://localhost:8080",
    top_k=5,
    threshold=0.7
)
```

#### `AgentMemBufferMemory`

Buffer-style memory with fixed-size recent history.

```python
from agentmem.langchain import AgentMemBufferMemory

memory = AgentMemBufferMemory(
    session_id="user-123",
    k=10  # Keep last 10 turns
)
```

#### `AgentMemSummaryMemory`

Memory with automatic summarization.

```python
from agentmem.langchain import AgentMemSummaryMemory

memory = AgentMemSummaryMemory(
    session_id="user-123",
    max_tokens=2000,
    summary_frequency=10
)
```

#### Factory Function

```python
from agentmem.langchain import create_agentmem_memory

memory = create_agentmem_memory(
    session_id="user-123",
    memory_type="summary",  # "default", "buffer", or "summary"
    top_k=10
)
```

## Advanced Usage

### Custom Metadata

```python
memory.add(
    "I prefer Python over JavaScript",
    metadata={
        "category": "preference",
        "topic": "programming",
        "sentiment": "positive"
    }
)
```

### Memory Scopes

```python
# Session-scoped (default)
memory.add("Session-specific info", scope="session")

# User-scoped
memory.add("User preference", scope="user")

# Global-scoped
memory.add("Shared knowledge", scope="global")
```

### Async with LangChain

```python
from agentmem import AsyncMemoryClient

client = AsyncMemoryClient()

# Use in async context
async def chat_turn(user_input: str):
    # Add user input
    await client.add(f"User: {user_input}")
    
    # Get relevant context
    context = await client.search(user_input, top_k=5)
    
    # Generate response
    response = await generate_response(user_input, context)
    
    # Add AI response
    await client.add(f"AI: {response}")
    
    return response
```

## Examples

See the `examples/` directory for complete examples:
- Basic usage
- LangChain integration
- Async operations
- Multi-tenant scenarios

## License

MIT OR Apache-2.0
