# LlamaIndex-AgentMem Quick Start Guide

This guide will help you get started with the Llama-AgentMem integration in 5 minutes.

## Prerequisites

1. **AgentMem Server**: Make sure you have an AgentMem server running
   ```bash
   # Start AgentMem server (default: http://localhost:8080)
   agentmem-server --port 8080
   ```

2. **Python Environment**: Python 3.8 or higher
   ```bash
   python --version  # Should be >= 3.8
   ```

## Installation

```bash
# Install the integration
pip install llamaindex-agentmem

# Or install with all LlamaIndex dependencies
pip install llamaindex-agentmem[all]
```

## Your First Memory Operation

### 1. Basic Memory Storage and Retrieval

Create a file `first_memory.py`:

```python
from llama_index.core import Document
from llamaindex_agentmem import AgentMemory
from agentmem import Config

# Initialize memory
config = Config(
    api_base_url="http://localhost:8080",
)

memory = AgentMemory(
    config=config,
    agent_id="my_first_agent",
)

# Add some documents
documents = [
    Document(text="Python is a high-level programming language"),
    Document(text="JavaScript is used for web development"),
    Document(text="Rust provides memory safety"),
]

print("Storing documents...")
memory.put(documents)

# Search for relevant content
print("\nSearching for 'programming':")
results = memory.get("programming")

for i, doc in enumerate(results, 1):
    print(f"{i}. {doc.text}")
    print(f"   Relevance: {doc.metadata['score']:.2f}")

print(f"\nFound {len(results)} relevant documents")
```

Run it:
```bash
python first_memory.py
```

### 2. Using with Query Engine

Create a file `query_example.py`:

```python
from llama_index.core import Document, VectorStoreIndex
from llamaindex_agentmem import AgentMemory
from agentmem import Config

# Initialize
config = Config(api_base_url="http://localhost:8080")
memory = AgentMemory(config=config, agent_id="qa_agent")

# Add knowledge base
kb_docs = [
    Document(
        text="AgentMem provides semantic, episodic, and procedural memory types",
        metadata={"category": "features"}
    ),
    Document(
        text="LlamaIndex is a data framework for LLM applications",
        metadata={"category": "overview"}
    ),
    Document(
        text="Vector databases enable efficient semantic search",
        metadata={"category": "technology"}
    ),
]

memory.put(kb_docs)

# Query the knowledge base
query = "What memory types are available?"

# Retrieve relevant context
relevant_docs = memory.get(query, limit=2)

print(f"Query: {query}\n")

# Create index and query engine
index = VectorStoreIndex.from_documents(relevant_docs)
query_engine = index.as_query_engine()

# Get answer
response = query_engine.query(query)
print(f"Answer: {response}")
```

### 3. Building a Chat Application

Create a file `chat_example.py`:

```python
from llama_index.core import Document
from llamaindex_agentmem import AgentMemory
from agentmem import Config
import asyncio

async def chat_demo():
    # Initialize episodic memory for conversations
    config = Config(api_base_url="http://localhost:8080")
    memory = AgentMemory(
        config=config,
        agent_id="chatbot",
        user_id="user_123",
        memory_type="episodic",
    )

    # Simulate a conversation
    conversation_turns = [
        ("Hi, I'm learning Python", "Hello! Python is great to learn."),
        ("What should I start with?", "Start with variables and loops."),
        ("How about data structures?", "Lists and dictionaries are essential."),
    ]

    # Store conversation
    for user_msg, bot_msg in conversation_turns:
        doc = Document(
            text=f"User: {user_msg}\nBot: {bot_msg}",
            metadata={"type": "conversation"},
        )
        await memory._async_put([doc])

    # Retrieve conversation context
    context = await memory._async_get("learning Python resources")

    print("Conversation History:")
    for doc in context:
        print(f"\n{doc.text}")

    # Cleanup
    await memory._async_clear()

if __name__ == "__main__":
    asyncio.run(chat_demo())
```

## Next Steps

### Explore Advanced Features

**Metadata Filtering:**
```python
results = memory.get(
    "query",
    filters={"category": "AI", "difficulty": "advanced"}
)
```

**Importance-Based Retrieval:**
```python
memory.put(documents, importance=0.9)
results = memory.get("query", min_importance=0.7)
```

**Document Reader:**
```python
from llamaindex_agentmem import AgentMemReader

reader = AgentMemReader(connection_string="http://localhost:8080")
documents = reader.load_data(
    query="machine learning",
    agent_id="research_agent",
    limit=10
)
```

### Run Examples

The package includes comprehensive examples:

```bash
cd examples/

# Basic usage
python basic_usage.py

# Query engine integration
python query_engine.py

# Chat engine integration
python chat_engine.py
```

### Run Tests

```bash
# Clone the repository
git clone https://github.com/agentmem/agentmem.git
cd agentmem/sdks/llamaindex-agentmem

# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run with coverage
pytest --cov=llamaindex_agentmem --cov-report=html
```

## Common Use Cases

### 1. Document Q&A System
```python
memory = AgentMemory(agent_id="qa_system")
memory.put(your_documents)

# Query your documents
results = memory.get(user_question)
```

### 2. Chatbot with Memory
```python
memory = AgentMemory(
    agent_id="chatbot",
    user_id=user_id,
    memory_type="episodic"
)

# Store conversations
memory.put([Document(text=conversation)])

# Retrieve context
context = memory.get(user_query)
```

### 3. Knowledge Management
```python
memory = AgentMemory(
    agent_id="knowledge_base",
    memory_type="semantic"
)

# Store knowledge with metadata
memory.put([
    Document(
        text="Knowledge content",
        metadata={"topic": "ML", "level": "advanced"}
    )
])

# Filtered retrieval
results = memory.get(
    "query",
    filters={"topic": "ML"}
)
```

## Troubleshooting

### Connection Issues
```python
# Verify AgentMem server is running
import httpx
response = httpx.get("http://localhost:8080/health")
print(response.json())
```

### Authentication Errors
```python
# Provide API key if required
config = Config(
    api_base_url="http://localhost:8080",
    api_key="your-api-key"
)
```

### Import Errors
```bash
# Reinstall with correct dependencies
pip install --upgrade llamaindex-agentmem
pip install --upgrade llama-index-core agentmem
```

## Resources

- **Full Documentation**: [README.md](README.md)
- **API Reference**: [README.md#api-reference](README.md#api-reference)
- **Examples**: [examples/](examples/)
- **Issues**: [GitHub Issues](https://github.com/agentmem/agentmem/issues)
- **Community**: [Discord](https://discord.gg/agentmem)

## Support

If you need help:
1. Check the [README.md](README.md) for detailed documentation
2. Review [examples/](examples/) for usage patterns
3. Search [GitHub Issues](https://github.com/agentmem/agentmem/issues)
4. Contact: support@agentmem.dev

Happy coding with AgentMem and LlamaIndex!
