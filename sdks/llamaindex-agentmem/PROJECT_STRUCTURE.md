# LlamaIndex-AgentMem Project Structure

This document provides an overview of the project structure and organization.

## Directory Structure

```
llamaindex-agentmem/
├── llamaindex_agentmem/          # Main package
│   ├── __init__.py               # Package initialization
│   ├── memory.py                 # AgentMemory class (BaseMemory implementation)
│   ├── reader.py                 # AgentMemReader class (BaseReader implementation)
│   ├── node_parser.py            # AgentMemNodeParser class
│   └── py.typed                  # Type checking marker
│
├── tests/                        # Test suite
│   ├── __init__.py               # Test package initialization
│   ├── test_memory.py            # Tests for AgentMemory
│   ├── test_reader.py            # Tests for AgentMemReader
│   ├── test_node_parser.py       # Tests for AgentMemNodeParser
│   └── test_integration.py       # Integration tests
│
├── examples/                     # Usage examples
│   ├── basic_usage.py            # Basic memory operations
│   ├── query_engine.py           # Query engine integration
│   └── chat_engine.py            # Chat engine integration
│
├── README.md                     # Main documentation
├── QUICKSTART.md                 # Quick start guide
├── CHANGELOG.md                  # Version history
├── LICENSE                       # MIT License
├── .gitignore                    # Git ignore patterns
├── pyproject.toml                # Project configuration
└── PROJECT_STRUCTURE.md          # This file
```

## Component Overview

### Core Components (`llamaindex_agentmem/`)

#### `__init__.py`
- Package initialization
- Exports main classes: AgentMemory, AgentMemReader, AgentMemNodeParser
- Version information

#### `memory.py`
- **AgentMemory**: Main class implementing LlamaIndex's BaseMemory interface
- Provides:
  - Document storage and retrieval
  - Semantic search capabilities
  - Metadata filtering
  - Importance-based ranking
  - Multi-agent memory management
- Supports both sync and async operations

#### `reader.py`
- **AgentMemReader**: LlamaIndex BaseReader implementation
- Provides:
  - Load documents from AgentMem by search query
  - Filter by agent, user, session, and metadata
  - Support for importance-based filtering
  - Returns LlamaIndex Document objects

#### `node_parser.py`
- **AgentMemNodeParser**: LlamaIndex NodeParser with AgentMem integration
- Provides:
  - Automatic document chunking
  - Chunk storage in AgentMem
  - Configurable chunk sizes and overlap
  - Metadata preservation

### Tests (`tests/`)

#### `test_memory.py`
- Unit tests for AgentMemory class
- Tests:
  - Initialization
  - Document operations (add, get, put, set, delete, clear)
  - Metadata handling
  - Serialization/deserialization

#### `test_reader.py`
- Unit tests for AgentMemReader class
- Tests:
  - Document loading by query
  - Metadata filtering
  - Loading all memories
  - Async context manager

#### `test_node_parser.py`
- Unit tests for AgentMemNodeParser class
- Tests:
  - Text splitting
  - Document chunking
  - Memory storage
  - Multi-document handling

#### `test_integration.py`
- Integration tests
- End-to-end scenarios:
  - Full memory workflow
  - Chatbot memory scenario
  - Knowledge base scenario

### Examples (`examples/`)

#### `basic_usage.py`
- Demonstrates:
  - Basic memory operations
  - Document storage and retrieval
  - Metadata filtering
  - Multi-agent setup

#### `query_engine.py`
- Demonstrates:
  - Integration with LlamaIndex QueryEngine
  - Context retrieval
  - Metadata filtering in queries
  - Streaming queries

#### `chat_engine.py`
- Demonstrates:
  - Building conversational AI applications
  - Episodic memory for conversations
  - Multi-turn chat with memory
  - Knowledge base integration

## Key Design Patterns

### 1. Lazy Initialization
All components use lazy client initialization:
```python
@property
def client(self) -> AgentMemClient:
    if self._client is None:
        self._client = AgentMemClient(self.config)
    return self._client
```

### 2. Async/Sync Dual API
All operations support both sync and async:
```python
# Sync
memory.get("query")

# Async
await memory._async_get("query")
```

### 3. Context Manager Support
All components support async context managers:
```python
async with memory:
    # Use memory
    pass  # Automatically cleaned up
```

### 4. Type Hints
Complete type annotations throughout:
```python
def get(self, input_str: str, **kwargs) -> List[Document]:
    ...
```

## Configuration

### Project Configuration (`pyproject.toml`)
- Package metadata
- Dependencies
- Build configuration
- Tool configurations (black, isort, mypy, pytest, coverage)

### AgentMem Configuration
Uses standard AgentMem Config:
```python
config = Config(
    api_base_url="http://localhost:8080",
    api_key="optional-key",
)
```

## Testing Strategy

### Unit Tests
- Mock all AgentMem client calls
- Test component logic in isolation
- Achieve >80% code coverage

### Integration Tests
- Test component interactions
- End-to-end workflows
- Realistic usage scenarios

### Test Coverage
```bash
pytest --cov=llamaindex_agentmem --cov-report=html
```

## Documentation Structure

### User Documentation
- **README.md**: Comprehensive guide with API reference
- **QUICKSTART.md**: 5-minute getting started guide
- **examples/**: Working code examples

### Developer Documentation
- **PROJECT_STRUCTURE.md**: This file
- Inline docstrings: Detailed API documentation
- Type hints: IDE-friendly development

## Build and Distribution

### Building
```bash
python -m build
```

### Publishing
```bash
twine upload dist/*
```

### Versioning
Follows Semantic Versioning (MAJOR.MINOR.PATCH)
- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes

## Dependencies

### Required
- `agentmem >= 7.0.0`: AgentMem Python SDK
- `llama-index-core >= 0.10.0`: LlamaIndex core library

### Development
- `pytest >= 7.0.0`: Testing framework
- `pytest-asyncio >= 0.21.0`: Async test support
- `pytest-cov >= 4.0.0`: Coverage reporting
- `black >= 23.0.0`: Code formatting
- `isort >= 5.12.0`: Import sorting
- `mypy >= 1.0.0`: Type checking

### Optional
- `llama-index >= 0.10.0`: Full LlamaIndex suite
- `llama-index-llms-openai`: OpenAI LLM integration
- `llama-index-embeddings-openai`: OpenAI embeddings

## Development Workflow

### Setting Up Development Environment
```bash
# Clone repository
git clone https://github.com/louloulin/agentmem.git
cd agentmem/sdks/llamaindex-agentmem

# Install in editable mode with dev dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black llamaindex_agentmem tests
isort llamaindex_agentmem tests

# Type check
mypy llamaindex_agentmem
```

### Code Quality Standards
- **Style**: PEP 8 with Black formatting
- **Imports**: isort for import organization
- **Type Safety**: mypy for type checking
- **Testing**: pytest with >80% coverage
- **Documentation**: Comprehensive docstrings

## Release Process

1. Update version in `pyproject.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create git tag
5. Build distribution packages
6. Upload to PyPI
7. Create GitHub release

## Future Enhancements

### Planned Features
- [ ] Streaming query support
- [ ] Advanced caching strategies
- [ ] Performance optimizations
- [ ] Additional example notebooks
- [ ] Integration with popular LLM providers
- [ ] Batch operation optimizations
- [ ] Enhanced error messages
- [ ] Performance benchmarking

### Contributing
See CONTRIBUTING.md for guidelines on contributing to this project.

## Support

- **Issues**: https://github.com/louloulin/agentmem/issues
- **Email**: support@agentmem.dev
- **Documentation**: https://agentmem.cc

## License

MIT License - see LICENSE file for details.
