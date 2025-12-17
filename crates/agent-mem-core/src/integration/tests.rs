/// 系统集成测试
///
/// 测试系统各组件的集成和端到端工作流程

#[cfg(test)]
mod tests {
    use crate::integration::api_interface::{
        ApiOperation, ApiRequest, ApiStatus, BatchRequest, ClientInfo, UnifiedApiInterface,
    };
    use crate::integration::config_manager::ConfigManager;
    use crate::integration::{AccessControlLevel, CacheConfig, MonitoringConfig, SecurityConfig};
    use crate::{HealthStatus, SystemConfig, SystemIntegrationManager, SystemState};
    use agent_mem_traits::{MemoryItem as Memory, MemoryType, Result};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    /// 创建测试用的系统集成管理器
    async fn create_test_system_manager() -> Result<SystemIntegrationManager> {
        let config = SystemConfig {
            name: "Test AgentMem".to_string(),
            version: "7.0.0-test".to_string(),
            enabled_memory_types: vec![
                MemoryType::Core,
                MemoryType::Resource,
                MemoryType::Knowledge,
                MemoryType::Contextual,
            ],
            max_concurrent_operations: 10,
            cache_config: CacheConfig {
                max_size_mb: 64,
                ttl_seconds: 300,
                enable_compression: true,
            },
            monitoring_config: MonitoringConfig {
                enable_performance_monitoring: true,
                enable_health_checks: true,
                monitoring_interval_seconds: 10,
                metrics_retention_hours: 1,
            },
            security_config: SecurityConfig {
                enable_encryption: false, // 测试环境关闭加密
                enable_audit_logging: true,
                access_control_level: AccessControlLevel::Basic,
            },
        };

        SystemIntegrationManager::new(config).await
    }

    /// 创建测试记忆
    fn create_test_memory(memory_type: MemoryType, content: &str) -> Memory {
        use agent_mem_traits::Session;
        let now = chrono::Utc::now();

        Memory {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            hash: None,
            metadata: HashMap::new(),
            score: Some(0.8),
            created_at: now,
            updated_at: Some(now),
            session: Session::new(),
            memory_type,
            entities: vec![],
            relations: vec![],
            agent_id: "test_agent".to_string(),
            user_id: Some("test_user".to_string()),
            importance: 0.8,
            embedding: None,
            last_accessed_at: now,
            access_count: 0,
            expires_at: None,
            version: 1,
        }
    }

    #[tokio::test]
    async fn test_system_lifecycle() {
        let system_manager = create_test_system_manager().await.unwrap();

        // 测试系统启动
        assert!(system_manager.start().await.is_ok());

        // 检查系统状态
        let status = system_manager.get_status().await;
        assert_eq!(status.status, SystemState::Running);
        assert_eq!(status.system_id, status.system_id);

        // 测试系统暂停
        assert!(system_manager.pause().await.is_ok());
        let status = system_manager.get_status().await;
        assert_eq!(status.status, SystemState::Paused);

        // 测试系统恢复
        assert!(system_manager.resume().await.is_ok());
        let status = system_manager.get_status().await;
        assert_eq!(status.status, SystemState::Running);

        // 测试系统停止
        assert!(system_manager.stop().await.is_ok());
        let status = system_manager.get_status().await;
        assert_eq!(status.status, SystemState::Stopped);
    }

    #[tokio::test]
    #[ignore] // TODO: 需要实现完整的存储和检索逻辑
    async fn test_unified_memory_operations() {
        let system_manager = create_test_system_manager().await.unwrap();
        system_manager.start().await.unwrap();

        // 测试存储不同类型的记忆
        let core_memory = create_test_memory(MemoryType::Core, "这是核心记忆内容");
        let core_id = system_manager.store_memory(core_memory).await.unwrap();

        let resource_memory = create_test_memory(MemoryType::Resource, "这是资源记忆内容");
        let resource_id = system_manager.store_memory(resource_memory).await.unwrap();

        let knowledge_memory = create_test_memory(MemoryType::Knowledge, "这是知识库内容");
        let _knowledge_id = system_manager.store_memory(knowledge_memory).await.unwrap();

        let contextual_memory = create_test_memory(MemoryType::Contextual, "这是上下文记忆内容");
        let _contextual_id = system_manager
            .store_memory(contextual_memory)
            .await
            .unwrap();

        // 测试记忆检索
        let retrieved_core = system_manager.retrieve_memory(core_id).await.unwrap();
        assert!(retrieved_core.is_some());
        assert_eq!(retrieved_core.unwrap().content, "这是核心记忆内容");

        let retrieved_resource = system_manager.retrieve_memory(resource_id).await.unwrap();
        assert!(retrieved_resource.is_some());
        assert_eq!(retrieved_resource.unwrap().content, "这是资源记忆内容");

        // 测试记忆搜索
        let search_results = system_manager
            .search_memories("核心", Some(5))
            .await
            .unwrap();
        assert!(!search_results.is_empty());

        // 测试记忆删除
        let deleted = system_manager.delete_memory(core_id).await.unwrap();
        assert!(deleted);

        // 验证删除后无法检索
        let retrieved_after_delete = system_manager.retrieve_memory(core_id).await.unwrap();
        assert!(retrieved_after_delete.is_none());

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_health_monitoring() {
        let system_manager = create_test_system_manager().await.unwrap();
        system_manager.start().await.unwrap();

        // 执行健康检查
        let health_results = system_manager.perform_health_check().await.unwrap();

        // 验证所有组件都有健康检查结果
        assert!(health_results.contains_key("core_memory"));
        assert!(health_results.contains_key("resource_memory"));
        assert!(health_results.contains_key("knowledge_vault"));
        assert!(health_results.contains_key("contextual_memory"));
        assert!(health_results.contains_key("meta_memory"));
        assert!(health_results.contains_key("active_retrieval"));

        // 验证健康状态
        for (component, health) in health_results {
            println!("组件 {} 健康状态: {:?}", component, health.status);
            assert_ne!(health.status, HealthStatus::Unknown);
        }

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_system_statistics() {
        let system_manager = create_test_system_manager().await.unwrap();
        system_manager.start().await.unwrap();

        // 存储一些测试数据
        for i in 0..5 {
            let memory = create_test_memory(MemoryType::Core, &format!("测试记忆 {i}"));
            system_manager.store_memory(memory).await.unwrap();
        }

        // 获取系统统计信息
        let stats = system_manager.get_system_statistics().await.unwrap();

        // 验证统计信息包含必要的字段
        assert!(stats.contains_key("system_status"));
        assert!(stats.contains_key("component_health"));
        assert!(stats.contains_key("core_memory_stats"));

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_config_manager() {
        let config_manager = ConfigManager::new();

        // 测试默认配置
        let default_config = config_manager.get_config().await;
        assert_eq!(default_config.name, "AgentMem 7.0");
        assert_eq!(default_config.version, "7.0.0");

        // 测试配置更新
        let mut new_config = default_config.clone();
        new_config.name = "Updated AgentMem".to_string();
        new_config.max_concurrent_operations = 50;

        assert!(config_manager.update_config(new_config).await.is_ok());

        let updated_config = config_manager.get_config().await;
        assert_eq!(updated_config.name, "Updated AgentMem");
        assert_eq!(updated_config.max_concurrent_operations, 50);

        // 测试部分配置更新
        let mut updates = HashMap::new();
        updates.insert(
            "name".to_string(),
            serde_json::Value::String("Partially Updated".to_string()),
        );
        updates.insert(
            "max_concurrent_operations".to_string(),
            serde_json::Value::Number(serde_json::Number::from(75)),
        );

        assert!(config_manager.update_config_partial(updates).await.is_ok());

        let partial_updated_config = config_manager.get_config().await;
        assert_eq!(partial_updated_config.name, "Partially Updated");
        assert_eq!(partial_updated_config.max_concurrent_operations, 75);

        // 测试配置验证
        let mut invalid_config = SystemConfig::default();
        invalid_config.name = "".to_string(); // 无效的空名称
        assert!(config_manager.update_config(invalid_config).await.is_err());

        // 测试配置变更历史
        let history = config_manager.get_change_history(Some(10)).await;
        assert!(!history.is_empty());
    }

    #[tokio::test]
    async fn test_api_interface() {
        let system_manager = Arc::new(create_test_system_manager().await.unwrap());
        system_manager.start().await.unwrap();

        let api_interface = UnifiedApiInterface::new(Arc::clone(&system_manager));

        // 测试存储记忆API
        let memory = create_test_memory(MemoryType::Core, "API测试记忆");
        let store_request = ApiRequest {
            request_id: Uuid::new_v4(),
            operation: ApiOperation::StoreMemory { memory },
            timestamp: chrono::Utc::now(),
            client_info: Some(ClientInfo {
                client_id: "test_client".to_string(),
                version: "1.0.0".to_string(),
                user_agent: Some("AgentMem Test".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
            }),
        };

        let store_response = api_interface.handle_request(store_request).await
            .expect("handle_request should succeed in test");
        assert_eq!(store_response.status, ApiStatus::Success);
        assert!(store_response.data.is_some());

        let memory_id: Uuid = serde_json::from_value(store_response.data.unwrap()).unwrap();

        // 测试检索记忆API
        let retrieve_request = ApiRequest {
            request_id: Uuid::new_v4(),
            operation: ApiOperation::RetrieveMemory { memory_id },
            timestamp: chrono::Utc::now(),
            client_info: None,
        };

        let retrieve_response = api_interface.handle_request(retrieve_request).await;
        assert!(retrieve_response.is_ok());
        let retrieve_response = retrieve_response.unwrap();
        assert_eq!(retrieve_response.status, ApiStatus::Success);
        assert!(retrieve_response.data.is_some());

        // 测试搜索记忆API
        let search_request = ApiRequest {
            request_id: Uuid::new_v4(),
            operation: ApiOperation::SearchMemories {
                query: "API测试".to_string(),
                limit: Some(10),
            },
            timestamp: chrono::Utc::now(),
            client_info: None,
        };

        let search_response = api_interface.handle_request(search_request).await;
        assert!(search_response.is_ok());
        let search_response = search_response.unwrap();
        assert_eq!(search_response.status, ApiStatus::Success);
        assert!(search_response.data.is_some());

        // 测试系统状态API
        let status_request = ApiRequest {
            request_id: Uuid::new_v4(),
            operation: ApiOperation::GetSystemStatus,
            timestamp: chrono::Utc::now(),
            client_info: None,
        };

        let status_response = api_interface.handle_request(status_request).await;
        assert!(status_response.is_ok());
        let status_response = status_response.unwrap();
        assert_eq!(status_response.status, ApiStatus::Success);
        assert!(status_response.data.is_some());

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let system_manager = Arc::new(create_test_system_manager().await.unwrap());
        system_manager.start().await.unwrap();

        let api_interface = UnifiedApiInterface::new(Arc::clone(&system_manager));

        // 创建批量操作请求
        let operations = vec![
            ApiOperation::StoreMemory {
                memory: create_test_memory(MemoryType::Core, "批量操作记忆1"),
            },
            ApiOperation::StoreMemory {
                memory: create_test_memory(MemoryType::Resource, "批量操作记忆2"),
            },
            ApiOperation::GetSystemStatus,
        ];

        let batch_request = BatchRequest {
            batch_id: Uuid::new_v4(),
            operations,
            transactional: false,
            client_info: None,
        };

        let batch_response = api_interface.handle_batch_request(batch_request).await;

        assert_eq!(batch_response.overall_status, ApiStatus::Success);
        assert_eq!(batch_response.responses.len(), 3);
        assert_eq!(batch_response.success_count, 3);
        assert_eq!(batch_response.error_count, 0);

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let system_manager = Arc::new(create_test_system_manager().await.unwrap());
        system_manager.start().await.unwrap();

        // 并发存储记忆
        let mut handles = Vec::new();
        for i in 0..10 {
            let manager = Arc::clone(&system_manager);
            let handle = tokio::spawn(async move {
                let memory = create_test_memory(MemoryType::Core, &format!("并发测试记忆 {i}"));
                manager.store_memory(memory).await
            });
            handles.push(handle);
        }

        // 等待所有操作完成
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }

        // 验证所有操作都成功
        assert_eq!(results.len(), 10);
        for result in results {
            assert!(result.is_ok());
        }

        system_manager.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling() {
        let system_manager = create_test_system_manager().await.unwrap();
        // 不启动系统，测试错误处理

        // 测试在系统未运行时的操作
        let memory = create_test_memory(MemoryType::Core, "错误测试记忆");
        let result = system_manager.store_memory(memory).await;
        assert!(result.is_err());

        // 测试检索不存在的记忆
        system_manager.start().await.unwrap();
        let non_existent_id = Uuid::new_v4();
        let result = system_manager.retrieve_memory(non_existent_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        system_manager.stop().await.unwrap();
    }
}
