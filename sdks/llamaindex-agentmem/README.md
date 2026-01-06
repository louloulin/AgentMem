# LlamaIndex-AgentMem Integration

[![PyPI version](https://badge.fury.io/py/llamaindex-agentmem.svg)](https://badge.fury.io/py/llamaindex-agentmem)
[![Python](https://img.shields.io/pypi/pyversions/llamaindex-agentmem.svg)](https://pypi.org/project/llamaindex-agentmem/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Official integration adapter between [LlamaIndex](https://github.com/run-llama/llama_index) and [AgentMem](https://github.com/louloulin/agentmem) for enterprise-grade memory management in AI applications.

## Features

- **Persistent Memory Storage**: Store and retrieve documents with semantic search
- **Multi-Agent Memory Management**: Isolated memory spaces for different agents
- **Importance-Based Ranking**: Prioritize important memories during retrieval
- **Metadata Filtering**: Advanced filtering capabilities for targeted queries
- **Automatic Deduplication**: Prevent duplicate memory entries
- **Async/Sync APIs**: Full support for both async and sync operations
- **LlamaIndex Integration**: Seamless integration with VectorStoreIndex, QueryEngine, and ChatEngine

## Installation

```bash
# Basic installation
pip install llamaindex-agentmem

# With all LlamaIndex dependencies
pip install llamaindex-agentmem[all]
```

## Quick Start

### Basic Memory Usage

```python
from llama_index.core import Document
from llamaindex_agentmem import AgentMemory
from agentmem import Config

# Initialize AgentMemory
config = Config(
    api_base_url="http://localhost:8080",
    api_key="your-api-key"  # Optional if not using authentication
)

memory = AgentMemory(
    config=config,
    agent_id="my_agent",
    memory_type="semantic",
)

# Add documents
documents = [
    Document(text="Python is a programming language"),
    Document(text="LlamaIndex is a data framework"),
]
memory.put(documents)

# Retrieve relevant documents
results = memory.get("programming languages")
for doc in results:
    print(f"{doc.text} (score: {doc.metadata['score']})")
```

### Using Document Reader

```python
from llamaindex_agentmem import AgentMemReader

# Initialize reader
reader = AgentMemReader(
    connection_string="http://localhost:8080",
    api_key="your-api-key"
)

# Load documents by query
documents = reader.load_data(
    query="machine learning",
    agent_id="research_agent",
    limit=10
)

# Use with LlamaIndex
from llama_index.core import VectorStoreIndex
index = VectorStoreIndex.from_documents(documents)
```

### Using with Query Engine

```python
from llamaindex_agentmem import AgentMemory
from llama_index.core import VectorStoreIndex

# Initialize memory
memory = AgentMemory(agent_id="qa_agent")

# Add knowledge base
documents = [Document(text="Your knowledge here")]
memory.put(documents)

# Retrieve and query
relevant_docs = memory.get("search query")
index = VectorStoreIndex.from_documents(relevant_docs)
query_engine = index.as_query_engine()

response = query_engine.query("Your question")
print(response)
```

### Using with Chat Engine

```python
from llamaindex_agentmem import AgentMemory

# Initialize episodic memory for conversations
memory = AgentMemory(
    agent_id="chat_agent",
    user_id="user_123",
    memory_type="episodic",
)

# Store conversation
from llama_index.core import Document
conversation = Document(
    text="User: What is Python?\nAssistant: Python is a programming language.",
    metadata={"type": "conversation"}
)
memory.put([conversation])

# Retrieve conversation context
context = memory.get("Python programming")
```

## Advanced Usage

### Metadata Filtering

```python
# Add documents with metadata
documents = [
    Document(
        text="Machine learning concepts",
        metadata={"category": "ML", "difficulty": "advanced"}
    ),
    Document(
        text="Python basics",
        metadata={"category": "Python", "difficulty": "beginner"}
    ),
]
memory.put(documents)

# Search with metadata filters
results = memory.get(
    "learning resources",
    filters={"category": "ML", "difficulty": "advanced"}
)
```

### Importance-Based Retrieval

```python
# Add documents with importance scores
memory.put(
    documents,
    importance=0.9  # High importance
)

# Retrieve with minimum importance threshold
results = memory.get(
    "query",
    min_importance=0.7
)
```

### Node Parsing with Storage

```python
from llamaindex_agentmem import AgentMemNodeParser

# Initialize parser
parser = AgentMemNodeParser(
    agent_id="parser_agent",
    chunk_size=512,
    chunk_overlap=50,
)

# Parse and store documents
documents = [Document(text="Long document text...")]
nodes = parser.get_nodes_from_documents(documents)

# Nodes are automatically stored in AgentMem
```

## API Reference

### AgentMemory

Main memory class implementing LlamaIndex's `BaseMemory` interface.

**Parameters:**
- `config` (Config, optional): AgentMem configuration
- `connection_string` (str): AgentMem API URL
- `api_key` (str, optional): API key for authentication
- `user_id` (str): User identifier
- `agent_id` (str, optional): Agent identifier
- `memory_type` (str): Type of memory ("semantic", "episodic", "procedural")
- `top_k` (int): Default number of results to retrieve

**Methods:**
- `get(input_str, **kwargs)`: Retrieve relevant documents
- `get_all(**kwargs)`: Get all documents
- `put(documents, **kwargs)`: Add documents
- `set(documents, **kwargs)`: Replace all documents
- `clear()`: Clear all documents

### AgentMemReader

Document reader for loading from AgentMem.

**Methods:**
- `load_data(query, **kwargs)`: Load documents by search query
- `load_all_memories(**kwargs)`: Load all memories

### AgentMemNodeParser

Node parser with AgentMem storage integration.

**Parameters:**
- `chunk_size` (int): Maximum chunk size
- `chunk_overlap` (int): Overlap between chunks
- `importance` (float): Default importance score

## Memory Types

AgentMem supports four memory types:

1. **Semantic**: General knowledge and facts
2. **Episodic**: Events and experiences with temporal context
3. **Procedural**: Skills, workflows, and procedures
4. **Untyped**: Unspecified memory type

Choose the appropriate type based on your use case.

## Examples

See the `examples/` directory for complete examples:

- `basic_usage.py`: Basic memory operations
- `query_engine.py`: Integration with QueryEngine
- `chat_engine.py`: Building conversational AI applications

## Testing

```bash
# Run tests
pytest

# Run with coverage
pytest --cov=llamaindex_agentmem --cov-report=html

# Run specific test file
pytest tests/test_memory.py
```

## Requirements

- Python >= 3.8
- agentmem >= 7.0.0
- llama-index-core >= 0.10.0

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## Support

- GitHub Issues: https://github.com/louloulin/agentmem/issues
- Documentation: https://docs.agentmem.dev
- Email: support@agentmem.dev

## Related Projects

- [AgentMem Python SDK](https://github.com/louloulin/agentmem)
- [LlamaIndex](https://github.com/run-llama/llama_index)
