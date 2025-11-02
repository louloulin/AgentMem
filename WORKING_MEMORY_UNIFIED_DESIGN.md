# Working Memory 统一架构设计

## 问题分析

### 当前错误方案
```rust
// ❌ 错误：暴露底层连接，破坏抽象
pub struct Repositories {
    pub libsql_conn: Option<Arc<Mutex<Connection>>>, // 破坏抽象层
}
```

### 正确方案：WorkingMemoryStore 应该和其他 Repositories 平级

```rust
// ✅ 正确：WorkingMemoryStore 是一个独立的存储抽象
pub struct Repositories {
    pub users: Arc<dyn UserRepositoryTrait>,
    pub memories: Arc<dyn MemoryRepositoryTrait>,
    pub working_memory: Arc<dyn WorkingMemoryStore>, // ✅ 平级抽象
}
```

## 架构原则

1. **抽象层次一致**
   - `UserRepositoryTrait`, `MemoryRepositoryTrait`, `WorkingMemoryStore` 都是存储抽象
   - 它们应该在同一抽象层次

2. **高内聚低耦合**
   - `WorkingMemoryStore` 内部使用 memories 表（实现细节）
   - 外部只依赖 trait，不依赖具体实现

3. **统一工厂模式**
   - `RepositoryFactory` 负责创建所有存储抽象
   - 包括 Working Memory

## 实施方案

### 1. 在 Repositories 中添加 working_memory

```rust
#[derive(Clone)]
pub struct Repositories {
    pub users: Arc<dyn UserRepositoryTrait>,
    pub organizations: Arc<dyn OrganizationRepositoryTrait>,
    pub agents: Arc<dyn AgentRepositoryTrait>,
    pub messages: Arc<dyn MessageRepositoryTrait>,
    pub tools: Arc<dyn ToolRepositoryTrait>,
    pub api_keys: Arc<dyn ApiKeyRepositoryTrait>,
    pub memories: Arc<dyn MemoryRepositoryTrait>,
    pub blocks: Arc<dyn BlockRepositoryTrait>,
    pub associations: Arc<dyn AssociationRepositoryTrait>,
    pub working_memory: Arc<dyn WorkingMemoryStore>, // ✅ 新增
}
```

### 2. 在 RepositoryFactory 中创建 Working Memory

```rust
impl RepositoryFactory {
    async fn create_libsql_repositories(config: &DatabaseConfig) -> Result<Repositories> {
        let conn = create_libsql_pool(&config.url).await?;
        
        if config.auto_migrate {
            run_migrations(conn.clone()).await?;
        }

        Ok(Repositories {
            // ... 其他 repositories ...
            memories: Arc::new(LibSqlMemoryRepository::new(conn.clone())),
            working_memory: Arc::new(LibSqlWorkingStore::new(conn.clone())), // ✅ 统一创建
        })
    }
}
```

### 3. 在 orchestrator_factory 中直接使用

```rust
pub async fn create_orchestrator(
    agent: &Agent,
    repositories: &Arc<Repositories>,
) -> ServerResult<AgentOrchestrator> {
    // ...
    
    // ✅ 直接从 repositories 获取，保持抽象
    let working_store = Some(repositories.working_memory.clone());
    
    let orchestrator = AgentOrchestrator::new(
        orchestrator_config,
        memory_engine,
        message_repo,
        llm_client,
        tool_executor,
        working_store, // ✅ 传递抽象
    );
    
    Ok(orchestrator)
}
```

## 优势

1. **抽象一致**：所有存储都是 trait
2. **解耦合**：orchestrator 不知道底层是 LibSQL 还是 PostgreSQL
3. **可测试**：可以轻松 mock WorkingMemoryStore
4. **可扩展**：未来可以添加其他存储后端

## 关键洞察

Working Memory 使用 memories 表是**实现细节**，不应该暴露给上层。上层只需要知道：
- 有一个 `WorkingMemoryStore` trait
- 可以通过 `Repositories` 获取

这就是**高内聚低耦合**的体现！

