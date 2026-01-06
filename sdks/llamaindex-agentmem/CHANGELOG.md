# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-01-XX

### Added
- Initial release of llamaindex-agentmem integration
- `AgentMemory` class implementing LlamaIndex's `BaseMemory` interface
- `AgentMemReader` for loading documents from AgentMem
- `AgentMemNodeParser` for chunking documents with AgentMem storage
- Full async/sync API support
- Metadata filtering capabilities
- Importance-based memory ranking
- Multi-agent memory management
- Comprehensive test suite with >80% coverage
- Complete examples:
  - Basic usage demonstrations
  - Query engine integration
  - Chat engine integration
- Full documentation and API reference

### Features
- Persistent memory storage with semantic search
- Support for all AgentMem memory types (semantic, episodic, procedural, untyped)
- Configurable chunk sizes and overlap for node parsing
- Lazy client initialization for optimal resource usage
- Context manager support for automatic cleanup
- Comprehensive error handling and logging

### Dependencies
- agentmem >= 7.0.0
- llama-index-core >= 0.10.0
- Python >= 3.8

## [Unreleased]

### Planned
- Streaming query support
- Advanced caching strategies
- Performance optimizations
- Additional example notebooks
- Integration with popular LLM providers
