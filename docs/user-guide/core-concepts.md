# Core Concepts

Understanding AgentMem's core concepts will help you use it effectively.

## Memory Architecture

### Memory Scopes

AgentMem provides 6 levels of memory isolation:

#### 1. Global Scope
```rust
let scope = MemoryScope::Global;
memory.add_with_scope("Universal knowledge", scope).await?;
```
- **Use case**: Shared knowledge across all users and agents
- **Example**: Company policies, general facts

#### 2. Organization Scope
```rust
let scope = MemoryScope::Organization {
    org_id: "acme-corp".to_string()
};
memory.add_with_scope("Company-wide settings", scope).await?;
```
- **Use case**: Multi-tenant enterprise applications
- **Example**: Organization-specific configurations

#### 3. User Scope
```rust
let scope = MemoryScope::User {
    user_id: "alice".to_string()
};
memory.add_with_scope("User preferences", scope).await?;
```
- **Use case**: Single-user AI assistant
- **Example**: User preferences, personal information

#### 4. Agent Scope
```rust
let scope = MemoryScope::Agent {
    user_id: "alice".to_string(),
    agent_id: "coding-assistant".to_string()
};
memory.add_with_scope("Agent-specific context", scope).await?;
```
- **Use case**: Multi-agent systems
- **Example**: Agent behavior patterns, specialized knowledge

#### 5. Session Scope
```rust
let scope = MemoryScope::Session {
    user_id: "alice".to_string(),
    session_id: "window-1".to_string()
};
memory.add_with_scope("Session context", scope).await?;
```
- **Use case**: Multi-window or multi-device conversations
- **Example**: Current conversation state

#### 6. Run Scope
```rust
let scope = MemoryScope::Run {
    user_id: "alice".to_string(),
    run_id: "run-12345".to_string()
};
memory.add_with_scope("Temporary context", scope).await?;
```
- **Use case**: Temporary, single-execution contexts
- **Example**: Batch processing jobs

### Memory Hierarchy

```
Global (shared by all)
    ↓
Organization (shared by org members)
    ↓
User (user-specific)
    ↓
Agent (agent-specific)
    ↓
Session (conversation-specific)
    ↓
Run (single execution)
```

## Memory Content Types

### 1. Text Memory
```rust
memory.add("I love pizza").await?;
```
- Plain text content
- Automatically embedded for semantic search
- Best for: Natural language, facts, preferences

### 2. Structured Memory
```rust
use serde_json::json;

memory.add_structured(
    json!({
        "type": "user_profile",
        "name": "Alice",
        "age": 30,
        "skills": ["Rust", "Python"]
    }),
    "assistant",
    Some("user123")
).await?;
```
- JSON-structured data
- Preserves data types
- Best for: Profiles, configurations, metadata

### 3. Multimodal Memory
```rust
// Image memory
let image_data = std::fs::read("photo.jpg")?;
memory.add_binary(
    image_data,
    BinaryMetadata {
        content_type: "image/jpeg".to_string(),
        format: BinaryFormat::Raw
    }
).await?;

// Text + Image combination
memory.add_multimodal(vec![
    MultimodalContent::Text("A beautiful sunset".to_string()),
    MultimodalContent::Binary(image_data)
]).await?;
```
- Images, audio, video
- Cross-modal search
- Best for: Rich media, visual content

## Search Engines

AgentMem provides 5 search engines:

### 1. Vector Search (Semantic)
```rust
let results = memory.search(
    "What are the user's preferences?",
    "user123"
).await?;
```
- **How it works**: Embedding-based similarity
- **Best for**: Semantic understanding, concepts
- **Performance**: < 100ms

### 2. BM25 Search (Keyword)
```rust
use agent_mem::SearchEngine;

let results = memory.search_with_engine(
    "pizza pasta",
    "user123",
    SearchEngine::BM25
).await?;
```
- **How it works**: TF-IDF ranking
- **Best for**: Exact keyword matches
- **Performance**: < 10ms

### 3. Full-Text Search
```rust
let results = memory.search_with_engine(
    "\"love pizza\"",  // Phrase query
    "user123",
    SearchEngine::FullText
).await?;
```
- **How it works**: PostgreSQL full-text search
- **Best for**: Exact phrase matching
- **Performance**: < 5ms

### 4. Fuzzy Match
```rust
let results = memory.search_with_engine(
    "califonia",  // Typo: "california"
    "user123",
    SearchEngine::Fuzzy
).await?;
```
- **How it works**: Levenshtein distance
- **Best for**: Typos, misspellings
- **Performance**: < 15ms

### 5. Hybrid Search (RRF)
```rust
let results = memory.search_hybrid(
    "pizza preferences",
    "user123",
    HybridConfig {
        vector_weight: 0.7,
        bm25_weight: 0.3
    }
).await?;
```
- **How it works**: Reciprocal Rank Fusion
- **Best for**: Combined semantic + keyword
- **Performance**: < 50ms

## Intelligent Features

### Automatic Fact Extraction

When you add memories, AgentMem automatically extracts facts:

```rust
memory.add("Hi, I'm Alice from SF, I work at Google").await?;

// Automatically extracted:
// - Name: Alice
// - Location: San Francisco
// - Company: Google
```

### Smart Deduplication

```rust
memory.add("I love pizza").await?;
memory.add("My favorite food is pizza").await?;
memory.add("Pizza is my favorite").await?;

// AgentMem recognizes these as duplicates
// and merges them intelligently
```

### Conflict Resolution

```rust
memory.add("I live in New York").await?;
memory.add("I moved to San Francisco").await?;

// AgentMem detects the contradiction
// and resolves it using timestamps and context
```

## Memory Lifecycle

### Creation
```rust
let id = memory.add("New memory").await?;
```

### Retrieval
```rust
let memory = memory.get(&id).await?;
```

### Update
```rust
memory.update(&id, "Updated content").await?;
```

### Deletion
```rust
memory.delete(&id).await?;
```

### Archival
```rust
memory.archive(&id).await?;
```

## Metadata and Attributes

### Adding Metadata
```rust
use agent_mem::AddMemoryOptions;
use serde_json::json;

let options = AddMemoryOptions {
    metadata: Some(json!({
        "source": "user_chat",
        "confidence": 0.95,
        "timestamp": "2024-01-01T10:00:00Z"
    })),
    ..Default::default()
};

memory.add_with_options("User message", options).await?;
```

### Importance Scoring
```rust
let options = AddMemoryOptions {
    importance: Some(0.9),  // 0.0 to 1.0
    ..Default::default()
};

memory.add_with_options("Critical information", options).await?;
```

## Best Practices

### 1. Use Appropriate Scopes
```rust
// ✅ Good: User-specific data
memory.add_with_scope("User preference", MemoryScope::User { user_id }).await?;

// ❌ Bad: Using global scope for user data
memory.add_with_scope("User preference", MemoryScope::Global).await?;
```

### 2. Structure Your Data
```rust
// ✅ Good: Structured
memory.add_structured(json!({"name": "Alice", "age": 30})).await?;

// ❌ Bad: Unstructured
memory.add("Name: Alice, Age: 30").await?;
```

### 3. Use Search Engines Appropriately
```rust
// Semantic concepts → Vector search
memory.search("user feelings about product").await?;

// Exact keywords → BM25
memory.search_with_engine("error code 404", user, SearchEngine::BM25).await?;
```

### 4. Handle Conflicts Gracefully
```rust
// Check for conflicts before adding
let existing = memory.search("user location", user).await?;
if !existing.is_empty() {
    // Update instead of add
    memory.update(&existing[0].id, new_location).await?;
} else {
    memory.add(new_location).await?;
}
```

## Performance Considerations

### Memory Limits
- **Storage**: Unlimited (database-backed)
- **Cache**: 10,000 memories by default
- **Search**: No practical limit

### Optimization Tips
1. **Use caching**: Enabled by default
2. **Batch operations**: Use `add_batch()` for multiple memories
3. **Choose the right search engine**: BM25 is faster than vector
4. **Set importance scores**: Important memories are searched first

## Next Steps

- 📖 [Advanced Search](./advanced-search.md) - Deep dive into search engines
- 🧠 [Intelligent Memory](./intelligent-memory.md) - AI-powered features
- 🔌 [Plugin System](../developer-guide/plugin-development.md) - Extending AgentMem
