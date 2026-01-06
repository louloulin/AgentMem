# AgentMem Database Schema Documentation

## Overview

AgentMem now has a complete production-grade database schema inspired by MIRIX's design, implemented in Rust with SQLx. This document describes the database structure, relationships, and usage.

## Database Tables

### 1. Organizations Table

The top-level entity in the multi-tenant hierarchy.

```sql
CREATE TABLE organizations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);
```

**Purpose**: Represents an organization (tenant) in the system.

### 2. Users Table

Represents users within an organization.

```sql
CREATE TABLE users (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    timezone VARCHAR(100) NOT NULL DEFAULT 'UTC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);
```

**Purpose**: Represents users who interact with agents.

### 3. Agents Table

Represents AI agents with their configuration.

```sql
CREATE TABLE agents (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    agent_type VARCHAR(100),
    name VARCHAR(255),
    description TEXT,
    system TEXT,                    -- System prompt
    topic TEXT,                     -- Current conversation topic
    message_ids JSONB,              -- In-context message IDs
    metadata_ JSONB,
    llm_config JSONB,               -- LLM configuration
    embedding_config JSONB,         -- Embedding configuration
    tool_rules JSONB,               -- Tool usage rules
    mcp_tools JSONB,                -- MCP server names
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);
```

**Purpose**: Represents AI agents with their configuration and state.

### 4. Messages Table

Stores conversation messages between users and agents.

```sql
CREATE TABLE messages (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL,      -- "user", "assistant", "system", "tool"
    text TEXT,
    content JSONB,                  -- Message content parts
    model VARCHAR(255),
    name VARCHAR(255),
    tool_calls JSONB,               -- Tool calls made by assistant
    tool_call_id VARCHAR(255),      -- ID of tool call this message responds to
    step_id VARCHAR(255),
    otid VARCHAR(255),              -- Offline threading ID
    tool_returns JSONB,
    group_id VARCHAR(255),
    sender_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);
```

**Purpose**: Stores conversation history for agents.

**Key Index**: `idx_messages_agent_created_at` on `(agent_id, created_at)` for efficient message retrieval.

### 5. Blocks Table (Core Memory)

Stores core memory blocks for agents.

```sql
CREATE TABLE blocks (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    template_name VARCHAR(255),
    description TEXT,
    label VARCHAR(50) NOT NULL,     -- "human", "persona", "system"
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    value TEXT NOT NULL,            -- Block content
    "limit" BIGINT NOT NULL DEFAULT 2000,  -- Character limit
    metadata_ JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255),
    CONSTRAINT unique_block_id_label UNIQUE (id, label)
);
```

**Purpose**: Stores core memory blocks (human, persona, system) for agents.

**Validation**: Block value length is validated against the limit before insert/update.

### 6. Tools Table

Stores tools that can be used by agents.

```sql
CREATE TABLE tools (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    json_schema JSONB,              -- Tool parameter schema
    source_type VARCHAR(100),
    source_code TEXT,
    tags JSONB,
    metadata_ JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);
```

**Purpose**: Stores tool definitions for agent use.

### 7. Memories Table (Enhanced)

Enhanced version of the memories table with multi-tenant support.

```sql
CREATE TABLE memories (
    id VARCHAR(255) PRIMARY KEY,
    organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    hash VARCHAR(64),
    metadata JSONB NOT NULL DEFAULT '{}',
    score REAL,
    memory_type VARCHAR(50) NOT NULL,
    scope VARCHAR(50) NOT NULL,
    level VARCHAR(50) NOT NULL,
    importance REAL NOT NULL DEFAULT 0.0,
    access_count BIGINT NOT NULL DEFAULT 0,
    last_accessed TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_by_id VARCHAR(255),
    last_updated_by_id VARCHAR(255)
);
```

**Purpose**: Stores episodic and semantic memories with full-text search support.

**Key Index**: `idx_memories_content_fts` using PostgreSQL's GIN index for full-text search.

### 8. Junction Tables

#### blocks_agents

Many-to-many relationship between blocks and agents.

```sql
CREATE TABLE blocks_agents (
    block_id VARCHAR(255) NOT NULL,
    block_label VARCHAR(50) NOT NULL,
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    PRIMARY KEY (block_id, block_label, agent_id),
    FOREIGN KEY (block_id, block_label) REFERENCES blocks(id, label) ON DELETE CASCADE
);
```

#### tools_agents

Many-to-many relationship between tools and agents.

```sql
CREATE TABLE tools_agents (
    tool_id VARCHAR(255) NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    PRIMARY KEY (tool_id, agent_id)
);
```

## Key Design Principles

### 1. Multi-Tenancy

All tables (except organizations) have an `organization_id` foreign key for tenant isolation.

### 2. Soft Deletes

All tables have an `is_deleted` flag for soft deletion. Queries filter by `is_deleted = FALSE`.

### 3. Audit Trail

All tables have:
- `created_at`: Timestamp of creation
- `updated_at`: Timestamp of last update
- `created_by_id`: User who created the record
- `last_updated_by_id`: User who last updated the record

### 4. Cascade Deletes

Foreign keys use `ON DELETE CASCADE` to maintain referential integrity.

### 5. JSONB Columns

Complex data structures are stored as JSONB for flexibility:
- `llm_config`: LLM configuration
- `embedding_config`: Embedding configuration
- `tool_rules`: Tool usage rules
- `content`: Message content parts
- `tool_calls`: Tool call data
- `metadata`: Flexible metadata

### 6. Indexes

Critical indexes for query performance:
- `idx_messages_agent_created_at`: For message retrieval by agent
- `idx_memories_content_fts`: For full-text search
- `idx_memories_agent_id`: For memory retrieval by agent
- `idx_agents_organization_id`: For agent listing by organization

## Usage Examples

### Creating an Organization and User

```rust
use agent_mem_core::storage::{
    models::{Organization, User},
    repository::{OrganizationRepository, UserRepository, Repository},
};

// Create organization
let org = Organization::new("Acme Corp".to_string());
let org = org_repo.create(&org).await?;

// Create user
let user = User::new(org.id.clone(), "John Doe".to_string(), "UTC".to_string());
let user = user_repo.create(&user).await?;
```

### Creating an Agent with Core Memory

```rust
use agent_mem_core::storage::{
    models::{Agent, Block},
    agent_repository::AgentRepository,
    block_repository::BlockRepository,
    repository::Repository,
};

// Create agent
let agent = Agent::new(org.id.clone(), Some("Assistant".to_string()));
let agent = agent_repo.create(&agent).await?;

// Create core memory blocks
let human_block = Block::new(
    org.id.clone(),
    user.id.clone(),
    "human".to_string(),
    "User is a software engineer".to_string(),
    2000,
);
let human_block = block_repo.create_validated(&human_block).await?;

// Link block to agent
agent_repo.add_block(&agent.id, &human_block.id, &human_block.label).await?;
```

### Storing and Retrieving Messages

```rust
use agent_mem_core::storage::{
    models::Message,
    message_repository::MessageRepository,
    repository::Repository,
};

// Create message
let message = Message::new(
    org.id.clone(),
    user.id.clone(),
    agent.id.clone(),
    "user".to_string(),
    Some("Hello!".to_string()),
);
let message = message_repo.create(&message).await?;

// Get recent messages for agent
let messages = message_repo.get_recent_messages(&agent.id, 10).await?;
```

## Running Tests

Integration tests require a PostgreSQL database:

```bash
# Set database URL
export DATABASE_URL="postgresql://agentmem:password@localhost:5432/agentmem_test"

# Run tests
cargo test --package agent-mem-core --test database_integration_test -- --ignored
```

## Migration

To run migrations on a new database:

```rust
use agent_mem_core::storage::{postgres::PostgresStorage, PostgresConfig};

let config = PostgresConfig {
    url: "postgresql://agentmem:password@localhost:5432/agentmem".to_string(),
    max_connections: 10,
    connection_timeout: 30,
    query_timeout: 30,
    ssl: false,
};

let storage = PostgresStorage::new(config).await?;
storage.migrate().await?;
```

## Next Steps

- [ ] Add authentication and authorization layer
- [ ] Implement row-level security policies
- [ ] Add database connection pooling optimization
- [ ] Implement retry mechanisms for transient failures
- [ ] Add database backup and restore procedures
- [ ] Implement database migration versioning

