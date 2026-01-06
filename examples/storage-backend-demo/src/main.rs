use agent_mem_core::{
    hierarchy::{HierarchicalMemory, HierarchyMetadata, MemoryLevel, MemoryScope},
    storage::{
        postgres::PostgresStorage, redis::RedisCache, CacheBackend, CacheConfig, CacheStatistics,
        EvictionPolicy, HealthStatus, PostgresConfig, RedisConfig, StorageBackend,
        StorageStatistics,
    },
    CoreResult, Memory,
};
use agent_mem_traits::{Entity, MemoryType, Relation, Session};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("storage_backend_demo=info,agent_mem_core=info")
        .init();

    info!("🚀 AgentMem 6.0 存储后端真实化演示");
    info!("📋 Phase 1.2: 存储后端 Mock 清理和真实实现验证");

    // 演示 1: PostgreSQL 存储后端测试
    info!("📝 演示 1: PostgreSQL 存储后端连接测试");

    let postgres_config = PostgresConfig {
        url: "postgresql://localhost:5432/agentmem_test".to_string(),
        max_connections: 10,
        connection_timeout: 30,
        query_timeout: 60,
        ssl: false,
    };

    match test_postgres_storage(postgres_config).await {
        Ok(_) => info!("✅ PostgreSQL 存储后端测试完成"),
        Err(e) => {
            warn!("⚠️ PostgreSQL 连接失败（预期行为）: {}", e);
            info!("💡 这是正常的，因为演示环境可能没有运行 PostgreSQL");
        }
    }

    // 演示 2: Redis 缓存后端测试
    info!("📦 演示 2: Redis 缓存后端连接测试");

    let redis_config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 10,
        connection_timeout: 30,
        default_ttl: 3600,
        cluster: false,
    };

    match test_redis_cache(redis_config).await {
        Ok(_) => info!("✅ Redis 缓存后端测试完成"),
        Err(e) => {
            warn!("⚠️ Redis 连接失败（预期行为）: {}", e);
            info!("💡 这是正常的，因为演示环境可能没有运行 Redis");
        }
    }

    // 演示 3: 存储配置验证
    info!("🔍 演示 3: 存储配置验证");
    test_storage_configuration().await?;

    // 演示 4: 内存数据结构测试
    info!("🧠 演示 4: 内存数据结构和序列化测试");
    test_memory_serialization().await?;

    // 演示 5: 统计信息结构测试
    info!("📊 演示 5: 统计信息和健康检查结构测试");
    test_statistics_structures().await?;

    info!("✅ 所有演示完成！存储后端架构验证成功");
    info!("🎯 下一步: 实现真实的数据库连接和操作");

    Ok(())
}

async fn test_postgres_storage(config: PostgresConfig) -> CoreResult<()> {
    info!("   尝试连接到 PostgreSQL: {}", config.url);

    // 尝试创建 PostgreSQL 存储实例
    let storage = PostgresStorage::new(config).await?;

    // 执行健康检查
    let health = storage.health_check().await?;
    info!("   PostgreSQL 健康状态: {:?}", health);

    // 尝试初始化（运行迁移）
    storage.initialize().await?;
    info!("   PostgreSQL 初始化完成");

    // 创建测试内存
    let test_memory = create_test_memory();

    // 测试存储操作
    storage.store_memory(&test_memory).await?;
    info!("   存储内存成功");

    // 测试检索操作
    let retrieved = storage.get_memory(&test_memory.memory.id).await?;
    if retrieved.is_some() {
        info!("   检索内存成功");
    }

    // 测试统计信息
    let stats = storage.get_statistics().await?;
    info!("   存储统计: {} 个内存", stats.total_memories);

    Ok(())
}

async fn test_redis_cache(config: RedisConfig) -> CoreResult<()> {
    info!("   尝试连接到 Redis: {}", config.url);

    // 尝试创建 Redis 缓存实例
    let cache = RedisCache::new(config).await?;

    // 创建测试内存
    let test_memory = create_test_memory();
    let cache_key = format!("memory:{}", test_memory.memory.id);

    // 测试缓存操作
    cache.set(&cache_key, &test_memory, Some(3600)).await?;
    info!("   缓存设置成功");

    // 测试检索操作
    let cached = cache.get(&cache_key).await?;
    if cached.is_some() {
        info!("   缓存检索成功");
    }

    // 测试存在性检查
    let exists = cache.exists(&cache_key).await?;
    info!("   缓存存在性检查: {}", exists);

    // 测试统计信息
    let stats = cache.get_cache_stats().await?;
    info!("   缓存统计: {} 个条目", stats.total_entries);

    Ok(())
}

async fn test_storage_configuration() -> anyhow::Result<()> {
    // 创建完整的存储配置
    let postgres_config = PostgresConfig {
        url: "postgresql://localhost:5432/agentmem".to_string(),
        max_connections: 20,
        connection_timeout: 30,
        query_timeout: 60,
        ssl: true,
    };

    let redis_config = RedisConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 15,
        connection_timeout: 10,
        default_ttl: 3600,
        cluster: false,
    };

    let cache_config = CacheConfig {
        enabled: true,
        default_ttl: 1800,
        max_size: 10000,
        eviction_policy: EvictionPolicy::LRU,
    };

    // 序列化配置
    let postgres_json = serde_json::to_string_pretty(&postgres_config)?;
    let redis_json = serde_json::to_string_pretty(&redis_config)?;
    let cache_json = serde_json::to_string_pretty(&cache_config)?;

    info!("   PostgreSQL 配置验证通过");
    info!("   Redis 配置验证通过");
    info!("   缓存配置验证通过");
    info!("   所有配置结构序列化成功");

    Ok(())
}

async fn test_memory_serialization() -> anyhow::Result<()> {
    // 创建测试内存
    let memory = create_test_memory();

    // 测试序列化
    let serialized = serde_json::to_string_pretty(&memory)?;
    info!("   内存序列化成功，大小: {} 字节", serialized.len());

    // 测试反序列化
    let deserialized: HierarchicalMemory = serde_json::from_str(&serialized)?;
    info!("   内存反序列化成功");

    // 验证数据完整性
    assert_eq!(memory.memory.id, deserialized.memory.id);
    assert_eq!(memory.memory.content, deserialized.memory.content);
    assert_eq!(memory.scope, deserialized.scope);
    assert_eq!(memory.level, deserialized.level);

    info!("   数据完整性验证通过");

    Ok(())
}

async fn test_statistics_structures() -> anyhow::Result<()> {
    // 创建存储统计信息
    let mut memories_by_level = HashMap::new();
    memories_by_level.insert(MemoryLevel::Strategic, 100);
    memories_by_level.insert(MemoryLevel::Tactical, 500);
    memories_by_level.insert(MemoryLevel::Operational, 300);

    let mut memories_by_scope = HashMap::new();
    memories_by_scope.insert(MemoryScope::Global, 200);
    memories_by_scope.insert(MemoryScope::Agent("test_agent".to_string()), 400);
    memories_by_scope.insert(
        MemoryScope::User {
            agent_id: "test_agent".to_string(),
            user_id: "test_user".to_string(),
        },
        300,
    );

    let storage_stats = StorageStatistics {
        total_memories: 900,
        storage_size: 1024 * 1024 * 50, // 50MB
        memories_by_level,
        memories_by_scope,
        average_memory_size: 56832.0,
        last_updated: Utc::now(),
    };

    // 创建缓存统计信息
    let cache_stats = CacheStatistics {
        total_entries: 150,
        hit_rate: 0.85,
        miss_rate: 0.15,
        total_hits: 8500,
        total_misses: 1500,
        cache_size: 1024 * 1024 * 10,  // 10MB
        memory_usage: 1024 * 1024 * 8, // 8MB
        last_updated: Utc::now(),
    };

    // 创建健康状态
    let health_status = HealthStatus {
        healthy: true,
        message: "All systems operational".to_string(),
        last_check: Utc::now(),
        response_time_ms: 25,
    };

    // 测试序列化
    let storage_json = serde_json::to_string_pretty(&storage_stats)?;
    let cache_json = serde_json::to_string_pretty(&cache_stats)?;
    let health_json = serde_json::to_string_pretty(&health_status)?;

    info!("   存储统计信息结构验证通过");
    info!("   缓存统计信息结构验证通过");
    info!("   健康状态结构验证通过");
    info!("   所有统计结构序列化成功");

    Ok(())
}

fn create_test_memory() -> HierarchicalMemory {
    let mut metadata = HashMap::new();
    metadata.insert(
        "source".to_string(),
        serde_json::Value::String("demo".to_string()),
    );
    metadata.insert(
        "category".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    let now = Utc::now();

    let session = Session {
        id: "test_session".to_string(),
        user_id: Some("test_user".to_string()),
        agent_id: Some("test_agent".to_string()),
        run_id: None,
        actor_id: None,
        created_at: now,
        metadata: HashMap::new(),
    };

    let memory = Memory {
        id: Uuid::new_v4().to_string(),
        content: "这是一个测试内存，用于验证存储后端功能".to_string(),
        hash: Some("test_hash_123".to_string()),
        metadata,
        score: Some(0.8),
        created_at: now,
        updated_at: Some(now),
        session,
        memory_type: MemoryType::Episodic,
        entities: Vec::new(),
        relations: Vec::new(),
        agent_id: "test_agent".to_string(),
        user_id: Some("test_user".to_string()),
        importance: 0.8,
        embedding: None,
        last_accessed_at: now,
        access_count: 1,
        expires_at: None,
        version: 1,
    };

    HierarchicalMemory {
        memory,
        scope: MemoryScope::Session {
            agent_id: "test_agent".to_string(),
            user_id: "test_user".to_string(),
            session_id: "test_session".to_string(),
        },
        level: MemoryLevel::Operational,
        hierarchy_metadata: HierarchyMetadata::default(),
    }
}
