# AgentMem MCP Tools Reference

**Version**: 2.0.0  
**Last Updated**: 2025-01-05

---

## Overview

AgentMem provides **5 core MCP tools** that enable seamless memory management, search, and intelligent conversation capabilities through the Model Context Protocol.

---

## Tool List

| Tool Name | Description | Status |
|-----------|-------------|--------|
| `agentmem_add_memory` | Add a new memory to the system | ✅ Production Ready |
| `agentmem_search_memories` | Search memories semantically | ✅ Production Ready |
| `agentmem_chat` | Intelligent chat with memory context | ✅ Production Ready |
| `agentmem_get_system_prompt` | Get personalized system prompt | ✅ Production Ready |
| `agentmem_list_agents` | List all available agents | ✅ Production Ready |

---

## Tool Details

### 1. agentmem_add_memory

Add a new memory to the AgentMem system with automatic fact extraction and deduplication.

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `content` | string | ✅ Yes | The memory content to store |
| `user_id` | string | ✅ Yes | User identifier for scoping |
| `agent_id` | string | ❌ No | Agent identifier (for agent/run/session scope) |
| `run_id` | string | ❌ No | Run identifier (for run scope) |
| `session_id` | string | ❌ No | Session identifier (for session scope) |
| `org_id` | string | ❌ No | Organization identifier (for organization scope) |
| `scope_type` | string | ❌ No | Scope type: `user`, `agent`, `run`, `session`, `organization` |
| `memory_type` | string | ❌ No | Memory type: `Episodic`, `Semantic`, `Procedural`, `Factual`, `Core`, `Working`, `Resource`, `Knowledge`, `Contextual` (default: `Episodic`) |
| `metadata` | string | ❌ No | Additional metadata as JSON string |

#### Example Request

```json
{
  "content": "User prefers dark mode and uses Rust for backend development",
  "user_id": "user_123",
  "agent_id": "coding_assistant",
  "memory_type": "Factual",
  "metadata": "{\"source\": \"conversation\", \"timestamp\": \"2025-01-05T10:00:00Z\"}"
}
```

#### Example Response

```json
{
  "success": true,
  "memory_id": "mem_abc123",
  "user_id": "user_123",
  "content": "User prefers dark mode and uses Rust for backend development",
  "memory_type": "Factual",
  "created_at": "2025-01-05T10:00:00Z",
  "importance_score": 0.85
}
```

#### Features

- ✅ Automatic fact extraction using LLM
- ✅ Deduplication of similar memories
- ✅ Importance scoring
- ✅ Conflict resolution for contradictory information
- ✅ Multi-scope support (user, agent, run, session, organization)

---

### 2. agentmem_search_memories

Search memories using semantic search with multiple search engines.

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | ✅ Yes | Search query text |
| `user_id` | string | ✅ Yes | User identifier for scoping |
| `agent_id` | string | ❌ No | Agent identifier (optional filtering) |
| `limit` | integer | ❌ No | Maximum number of results (default: 10, max: 100) |
| `search_type` | string | ❌ No | Search engine: `vector`, `bm25`, `fulltext`, `fuzzy`, `hybrid` (default: `hybrid`) |
| `memory_type` | string | ❌ No | Filter by memory type |
| `min_score` | float | ❌ No | Minimum relevance score (0.0-1.0) |

#### Example Request

```json
{
  "query": "user preferences for development",
  "user_id": "user_123",
  "limit": 5,
  "search_type": "hybrid"
}
```

#### Example Response

```json
{
  "success": true,
  "results": [
    {
      "memory_id": "mem_abc123",
      "content": "User prefers dark mode and uses Rust for backend development",
      "score": 0.92,
      "memory_type": "Factual",
      "created_at": "2025-01-05T10:00:00Z"
    },
    {
      "memory_id": "mem_def456",
      "content": "User prefers functional programming paradigms",
      "score": 0.78,
      "memory_type": "Semantic",
      "created_at": "2025-01-04T15:30:00Z"
    }
  ],
  "total": 2,
  "search_time_ms": 45
}
```

#### Features

- ✅ 5 search engines: Vector, BM25, Full-Text, Fuzzy, Hybrid (RRF)
- ✅ Semantic understanding
- ✅ Relevance scoring
- ✅ Fast retrieval (<100ms P95)
- ✅ Multi-scope filtering

---

### 3. agentmem_chat

Intelligent chat with automatic memory context retrieval and personalized responses.

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `message` | string | ✅ Yes | User message/query |
| `user_id` | string | ✅ Yes | User identifier |
| `agent_id` | string | ❌ No | Agent identifier (required for full functionality) |
| `include_memories` | boolean | ❌ No | Include relevant memories in context (default: true) |
| `max_memories` | integer | ❌ No | Maximum memories to include (default: 5) |

#### Example Request

```json
{
  "message": "What programming languages do I prefer?",
  "user_id": "user_123",
  "agent_id": "coding_assistant",
  "include_memories": true,
  "max_memories": 3
}
```

#### Example Response

```json
{
  "success": true,
  "response": "Based on your saved memories, you prefer Rust for backend development and enjoy functional programming paradigms.",
  "memories_used": [
    {
      "memory_id": "mem_abc123",
      "content": "User prefers dark mode and uses Rust for backend development",
      "relevance": 0.92
    }
  ],
  "context_length": 256,
  "response_time_ms": 120
}
```

#### Features

- ✅ Automatic memory retrieval
- ✅ Context-aware responses
- ✅ Personalized system prompts
- ✅ Multi-turn conversation support
- ✅ LLM provider flexibility (OpenAI, Anthropic, DeepSeek, etc.)

---

### 4. agentmem_get_system_prompt

Generate a personalized system prompt based on user memories and preferences.

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | ✅ Yes | User identifier |
| `agent_id` | string | ❌ No | Agent identifier (optional) |
| `include_preferences` | boolean | ❌ No | Include user preferences (default: true) |
| `include_history` | boolean | ❌ No | Include conversation history (default: false) |
| `max_context` | integer | ❌ No | Maximum context items (default: 10) |

#### Example Request

```json
{
  "user_id": "user_123",
  "agent_id": "coding_assistant",
  "include_preferences": true,
  "max_context": 5
}
```

#### Example Response

```json
{
  "success": true,
  "system_prompt": "You are a helpful coding assistant. The user prefers dark mode and uses Rust for backend development. They enjoy functional programming paradigms. Please tailor your responses accordingly.",
  "user_id": "user_123",
  "context": {
    "preferences": ["dark mode", "Rust", "functional programming"],
    "memory_count": 15,
    "last_active": "2025-01-05T10:00:00Z"
  },
  "timestamp": "2025-01-05T10:00:00Z"
}
```

#### Features

- ✅ Personalized prompts
- ✅ Context aggregation
- ✅ Preference extraction
- ✅ Dynamic generation
- ✅ Multi-agent support

---

### 5. agentmem_list_agents

List all available agents in the system.

#### Parameters

None required.

#### Example Request

```json
{}
```

#### Example Response

```json
{
  "success": true,
  "agents": [
    {
      "agent_id": "coding_assistant",
      "name": "Coding Assistant",
      "description": "Helps with programming tasks",
      "created_at": "2025-01-01T00:00:00Z",
      "memory_count": 150,
      "status": "active"
    },
    {
      "agent_id": "writing_assistant",
      "name": "Writing Assistant",
      "description": "Assists with writing tasks",
      "created_at": "2025-01-02T00:00:00Z",
      "memory_count": 89,
      "status": "active"
    }
  ],
  "total": 2
}
```

#### Features

- ✅ Complete agent listing
- ✅ Agent metadata
- ✅ Status information
- ✅ Memory statistics
- ✅ Fast retrieval

---

## Error Handling

All tools return consistent error responses:

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| `INVALID_PARAMETERS` | Missing or invalid parameters | Check parameter types and required fields |
| `BACKEND_UNAVAILABLE` | AgentMem backend not reachable | Start the backend server or check connection |
| `AGENT_NOT_FOUND` | Specified agent does not exist | Create the agent first or use a valid agent_id |
| `MEMORY_NOT_FOUND` | Requested memory does not exist | Check memory_id or search parameters |
| `RATE_LIMIT_EXCEEDED` | Too many requests | Wait and retry |
| `AUTHENTICATION_FAILED` | Invalid credentials | Check API keys or JWT tokens |

---

## Best Practices

### 1. Memory Scoping

Use appropriate scope types for better organization:

```json
// User-level memory (persistent across all agents)
{
  "content": "User prefers dark mode",
  "user_id": "user_123",
  "scope_type": "user"
}

// Agent-specific memory
{
  "content": "User prefers Rust for backend",
  "user_id": "user_123",
  "agent_id": "coding_assistant",
  "scope_type": "agent"
}

// Session-specific memory (temporary)
{
  "content": "Current task: refactoring authentication",
  "user_id": "user_123",
  "agent_id": "coding_assistant",
  "session_id": "session_abc",
  "scope_type": "session"
}
```

### 2. Memory Types

Choose appropriate memory types:

- **Episodic**: Specific events or experiences
- **Semantic**: General knowledge or facts
- **Procedural**: How-to information or processes
- **Factual**: Verifiable facts
- **Core**: Essential user information
- **Working**: Temporary context
- **Resource**: External resources or links
- **Knowledge**: Domain knowledge
- **Contextual**: Context-dependent information

### 3. Search Optimization

- Use `hybrid` search for best results (combines multiple engines)
- Set appropriate `limit` to balance performance and relevance
- Use `min_score` to filter low-relevance results
- Filter by `memory_type` when searching specific categories

### 4. Chat Context

- Set `include_memories: true` for personalized responses
- Adjust `max_memories` based on context window limits
- Use `agent_id` for agent-specific behavior

---

## Performance

| Tool | Average Latency | P95 Latency | Throughput |
|------|----------------|-------------|------------|
| `agentmem_add_memory` | 20ms | 50ms | 5,000 ops/s |
| `agentmem_search_memories` | 45ms | 100ms | 10,000 ops/s |
| `agentmem_chat` | 120ms | 300ms | 1,000 ops/s |
| `agentmem_get_system_prompt` | 30ms | 80ms | 2,000 ops/s |
| `agentmem_list_agents` | 10ms | 25ms | 5,000 ops/s |

*Benchmarks on Apple M2 Pro, 32GB RAM, LibSQL backend*

---

## Integration Examples

### Claude Code

```bash
# In Claude Code, simply use natural language:
"Remember that I prefer dark mode"
"Search for my preferences about programming"
"What do you know about me?"
```

### Python SDK

```python
from agentmem import Memory

memory = Memory()
memory.add("User prefers dark mode", user_id="user_123")
results = memory.search("user preferences", user_id="user_123")
```

### HTTP API

```bash
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "User prefers dark mode",
    "user_id": "user_123"
  }'
```

---

## See Also

- [MCP Complete Guide](mcp-complete-guide.md) - Full integration guide
- [Claude Code Quickstart](../getting-started/claude-code-quickstart.md) - 5-minute setup
- [MCP Commands Reference](mcp-commands.md) - Command-line usage
- [API Reference](../api/) - Complete API documentation

---

**Last Updated**: 2025-01-05  
**Version**: 2.0.0

