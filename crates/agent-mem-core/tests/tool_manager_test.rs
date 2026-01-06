//! Tool Manager 测试
//!
//! 测试工具管理器的核心功能

#![cfg(feature = "postgres")]

use agent_mem_core::managers::{
    CreateToolRequest, ToolManager, ToolManagerConfig, ToolType, UpdateToolRequest,
};
use agent_mem_tools::ToolExecutor;
use sqlx::PgPool;
use std::sync::Arc;

/// 创建测试用的 ToolManager
async fn create_test_manager(pool: &PgPool) -> ToolManager {
    let executor = Arc::new(ToolExecutor::new());
    ToolManager::with_default_config(Arc::new(pool.clone()), executor)
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_tool_type_conversion() {
    assert_eq!(ToolType::Core.as_str(), "core");
    assert_eq!(ToolType::Memory.as_str(), "memory");
    assert_eq!(ToolType::Custom.as_str(), "custom");
    assert_eq!(ToolType::Mcp.as_str(), "mcp");
    assert_eq!(ToolType::External.as_str(), "external");

    assert_eq!(ToolType::from_str("core").unwrap(), ToolType::Core);
    assert_eq!(ToolType::from_str("memory").unwrap(), ToolType::Memory);
    assert_eq!(ToolType::from_str("custom").unwrap(), ToolType::Custom);
    assert_eq!(ToolType::from_str("mcp").unwrap(), ToolType::Mcp);
    assert_eq!(ToolType::from_str("external").unwrap(), ToolType::External);

    assert!(ToolType::from_str("invalid").is_err());
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_tool_manager_config_default() {
    let config = ToolManagerConfig::default();
    assert!(config.enable_cache);
    assert_eq!(config.cache_ttl_seconds, 3600);
    assert!(config.auto_register_builtin);
    assert_eq!(config.max_concurrent_executions, 10);
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_create_tool() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "test_calculator".to_string(),
        description: Some("A test calculator tool".to_string()),
        tool_type: "custom".to_string(),
        source_code: Some("def calculate(a, b): return a + b".to_string()),
        source_type: Some("python".to_string()),
        json_schema: Some(serde_json::json!({
            "type": "function",
            "function": {
                "name": "test_calculator",
                "description": "A test calculator tool",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["a", "b"]
                }
            }
        })),
        tags: Some(vec!["math".to_string(), "calculator".to_string()]),
    };

    let tool = manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    assert_eq!(tool.name, "test_calculator");
    assert_eq!(tool.description.unwrap(), "A test calculator tool");
    assert!(!tool.is_deleted);
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_create_or_update_tool() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "test_tool".to_string(),
        description: Some("Original description".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    // 第一次创建
    let tool1 = manager
        .create_or_update_tool("org_test", "user_test", request.clone())
        .await
        .unwrap();

    assert_eq!(tool1.description.unwrap(), "Original description");

    // 第二次更新
    let request2 = CreateToolRequest {
        name: "test_tool".to_string(),
        description: Some("Updated description".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    let tool2 = manager
        .create_or_update_tool("org_test", "user_test", request2)
        .await
        .unwrap();

    assert_eq!(tool2.id, tool1.id); // 同一个工具
    assert_eq!(tool2.description.unwrap(), "Updated description");
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_get_tool() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "get_test_tool".to_string(),
        description: Some("Test tool for get".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    let created = manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    let retrieved = manager.get_tool(&created.id, "user_test").await.unwrap();

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, "get_test_tool");
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_get_tool_by_name() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "unique_tool_name".to_string(),
        description: Some("Test tool".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    let found = manager
        .get_tool_by_name("org_test", "unique_tool_name")
        .await
        .unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "unique_tool_name");

    let not_found = manager
        .get_tool_by_name("org_test", "nonexistent_tool")
        .await
        .unwrap();

    assert!(not_found.is_none());
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_list_tools() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    // 创建多个工具
    for i in 1..=5 {
        let request = CreateToolRequest {
            name: format!("tool_{}", i),
            description: Some(format!("Tool number {}", i)),
            tool_type: "custom".to_string(),
            source_code: None,
            source_type: None,
            json_schema: None,
            tags: None,
        };

        manager
            .create_tool("org_test", "user_test", request)
            .await
            .unwrap();
    }

    let tools = manager
        .list_tools("org_test", Some(10), Some(0))
        .await
        .unwrap();

    assert!(tools.len() >= 5);
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_update_tool() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "update_test_tool".to_string(),
        description: Some("Original".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: Some(vec!["tag1".to_string()]),
    };

    let created = manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    let update_req = UpdateToolRequest {
        description: Some("Updated description".to_string()),
        source_code: Some("new code".to_string()),
        json_schema: None,
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        is_enabled: None,
    };

    let updated = manager
        .update_tool(&created.id, "user_test", update_req)
        .await
        .unwrap();

    assert_eq!(updated.description.unwrap(), "Updated description");
    assert_eq!(updated.source_code.unwrap(), "new code");
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_delete_tool() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "delete_test_tool".to_string(),
        description: Some("To be deleted".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    let created = manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    manager.delete_tool(&created.id, "user_test").await.unwrap();

    // 验证工具已被软删除
    let result = manager.get_tool(&created.id, "user_test").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_get_stats() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    // 创建不同类型的工具
    for i in 1..=3 {
        let request = CreateToolRequest {
            name: format!("stats_tool_{}", i),
            description: Some(format!("Tool {}", i)),
            tool_type: "custom".to_string(),
            source_code: None,
            source_type: None,
            json_schema: None,
            tags: Some(vec!["custom".to_string()]),
        };

        manager
            .create_tool("org_test", "user_test", request)
            .await
            .unwrap();
    }

    let stats = manager.get_stats("org_test").await.unwrap();

    assert!(stats.total_tools >= 3);
    assert!(stats.enabled_count >= 3);
}

#[tokio::test]
#[ignore] // 需要数据库
async fn test_cache_functionality() {
    let pool = create_test_pool().await;
    let manager = create_test_manager(&pool).await;

    let request = CreateToolRequest {
        name: "cache_test_tool".to_string(),
        description: Some("Test caching".to_string()),
        tool_type: "custom".to_string(),
        source_code: None,
        source_type: None,
        json_schema: None,
        tags: None,
    };

    let created = manager
        .create_tool("org_test", "user_test", request)
        .await
        .unwrap();

    // 第一次获取（从数据库）
    let tool1 = manager.get_tool(&created.id, "user_test").await.unwrap();

    // 第二次获取（从缓存）
    let tool2 = manager.get_tool(&created.id, "user_test").await.unwrap();

    assert_eq!(tool1.id, tool2.id);

    // 清除缓存
    manager.clear_cache().await;
}

// 辅助函数：创建测试数据库连接池
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/agentmem_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}
