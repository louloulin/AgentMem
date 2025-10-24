//! LibSQL 存储测试
//!
//! 测试 LibSQL 嵌入式存储的基本功能

#[cfg(feature = "libsql")]
mod tests {
    use agent_mem_storage::backends::LibSQLStore;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_libsql_basic_crud() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap();

        // 创建 LibSQL 存储
        let store = LibSQLStore::new(db_path_str).await.unwrap();

        // 测试插入
        let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
            id: "test-1".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: Some("user-1".to_string()),
            content: "Test memory content".to_string(),
            memory_type: "episodic".to_string(),
            importance: 0.8,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        store.insert(&record).await.unwrap();

        // 测试查询
        let retrieved = store.get("test-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.content, "Test memory content");
        assert_eq!(retrieved.importance, 0.8);

        // 测试更新
        let mut updated_record = retrieved.clone();
        updated_record.content = "Updated content".to_string();
        updated_record.importance = 0.9;
        store.update(&updated_record).await.unwrap();

        let retrieved = store.get("test-1").await.unwrap().unwrap();
        assert_eq!(retrieved.content, "Updated content");
        assert_eq!(retrieved.importance, 0.9);

        // 测试计数
        let count = store.count().await.unwrap();
        assert_eq!(count, 1);

        // 测试删除
        let deleted = store.delete("test-1").await.unwrap();
        assert!(deleted);

        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_libsql_memory_mode() {
        // 测试内存模式
        let store = LibSQLStore::new(":memory:").await.unwrap();

        let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
            id: "mem-1".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: None,
            content: "In-memory test".to_string(),
            memory_type: "semantic".to_string(),
            importance: 0.7,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        store.insert(&record).await.unwrap();
        let retrieved = store.get("mem-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "In-memory test");
    }

    #[tokio::test]
    async fn test_libsql_search() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_search.db");
        let db_path_str = db_path.to_str().unwrap();

        let store = LibSQLStore::new(db_path_str).await.unwrap();

        // 插入多条记录
        for i in 1..=5 {
            let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
                id: format!("test-{i}"),
                agent_id: "agent-1".to_string(),
                user_id: Some("user-1".to_string()),
                content: format!("Memory content {i}"),
                memory_type: "episodic".to_string(),
                importance: 0.5 + (i as f32 * 0.1),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            store.insert(&record).await.unwrap();
        }

        // 测试搜索
        let results = store
            .search(Some("agent-1"), Some("user-1"), None, 3)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);

        // 验证按创建时间倒序排序（最新的在前）
        assert!(results[0].created_at >= results[1].created_at);
        assert!(results[1].created_at >= results[2].created_at);
    }

    #[tokio::test]
    async fn test_libsql_clear() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_clear.db");
        let db_path_str = db_path.to_str().unwrap();

        let store = LibSQLStore::new(db_path_str).await.unwrap();

        // 插入一些记录
        for i in 1..=5 {
            let record = agent_mem_storage::backends::libsql_store::MemoryRecord {
                id: format!("test-{i}"),
                agent_id: "agent-1".to_string(),
                user_id: Some("user-1".to_string()),
                content: format!("Content {i}"),
                memory_type: "episodic".to_string(),
                importance: 0.5,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            store.insert(&record).await.unwrap();
        }

        let count = store.count().await.unwrap();
        assert_eq!(count, 5);

        // 清空
        store.clear().await.unwrap();

        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }
}

