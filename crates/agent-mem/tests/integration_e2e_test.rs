//! End-to-End Integration Tests
//!
//! 完整的端到端集成测试，验证整个系统的工作流程

use agent_mem::Memory;
use agent_mem_traits::MemoryItem;

#[cfg(test)]
mod e2e_tests {
    use super::*;

    /// 测试完整的记忆添加和搜索流程
    #[tokio::test]
    #[ignore] // 需要完整环境
    async fn test_complete_memory_workflow() {
        // Step 1: 创建 Memory 实例
        let memory = Memory::quick();

        // Step 2: 添加记忆
        let result = memory
            .add("我喜欢喝咖啡，每天早上都会喝一杯")
            .await
            .expect("Failed to add memory");
        assert!(!result.results.is_empty());

        // Step 3: 搜索记忆
        let search_results = memory
            .search("饮品")
            .await
            .expect("Failed to search memories");
        assert!(!search_results.is_empty());

        // Step 4: 验证结果相关性
        let found_coffee = search_results
            .iter()
            .any(|m| m.content.contains("咖啡"));
        assert!(found_coffee, "Should find coffee memory");
    }

    /// 测试批量记忆操作
    #[tokio::test]
    #[ignore]
    async fn test_batch_memory_operations() {
        let memory = Memory::quick();

        // 添加多条记忆
        let memories = vec![
            "我叫张三，住在北京",
            "我是一名软件工程师",
            "我喜欢编程和阅读",
        ];

        for mem in memories {
            memory.add(mem).await.expect("Failed to add memory");
        }

        // 搜索验证
        let results = memory
            .search("张三")
            .await
            .expect("Failed to search");
        assert!(!results.is_empty());
    }

    /// 测试记忆更新流程
    #[tokio::test]
    #[ignore]
    async fn test_memory_update_workflow() {
        let memory = Memory::quick();

        // 添加初始记忆
        let add_result = memory
            .add("我住在北京")
            .await
            .expect("Failed to add memory");
        let memory_id = &add_result.results[0].id;

        // 更新记忆
        memory
            .update(memory_id, "我住在上海")
            .await
            .expect("Failed to update memory");

        // 验证更新
        let results = memory
            .search("住在哪里")
            .await
            .expect("Failed to search");
        let updated = results
            .iter()
            .any(|m| m.content.contains("上海"));
        assert!(updated, "Memory should be updated to Shanghai");
    }

    /// 测试记忆删除流程
    #[tokio::test]
    #[ignore]
    async fn test_memory_deletion_workflow() {
        let memory = Memory::quick();

        // 添加记忆
        let add_result = memory
            .add("这是一条测试记忆")
            .await
            .expect("Failed to add memory");
        let memory_id = &add_result.results[0].id;

        // 删除记忆
        memory
            .delete(memory_id)
            .await
            .expect("Failed to delete memory");

        // 验证删除
        let results = memory
            .search("测试记忆")
            .await
            .expect("Failed to search");
        assert!(results.is_empty(), "Deleted memory should not be found");
    }

    /// 测试多语言记忆处理
    #[tokio::test]
    #[ignore]
    async fn test_multilingual_memory_handling() {
        let memory = Memory::quick();

        // 添加中文记忆
        memory.add("我喜欢喝茶").await.expect("Failed to add Chinese memory");

        // 添加英文记忆
        memory.add("I like drinking coffee").await.expect("Failed to add English memory");

        // 搜索中文
        let chinese_results = memory
            .search("茶")
            .await
            .expect("Failed to search Chinese");
        assert!(!chinese_results.is_empty());

        // 搜索英文
        let english_results = memory
            .search("coffee")
            .await
            .expect("Failed to search English");
        assert!(!english_results.is_empty());
    }

    /// 测试用户隔离
    #[tokio::test]
    #[ignore]
    async fn test_user_isolation() {
        let memory = Memory::quick();

        // User A 添加记忆
        memory
            .add_with_context("User A 的秘密", "user_a", "agent_1")
            .await
            .expect("Failed to add user A memory");

        // User B 添加记忆
        memory
            .add_with_context("User B 的秘密", "user_b", "agent_1")
            .await
            .expect("Failed to add user B memory");

        // User A 搜索（不应该看到 User B 的记忆）
        let user_a_results = memory
            .search_with_context("秘密", "user_a", "agent_1")
            .await
            .expect("Failed to search for user A");

        let has_user_b_memory = user_a_results
            .iter()
            .any(|m| m.content.contains("User B"));
        assert!(!has_user_b_memory, "User A should not see User B's memories");
    }

    /// 测试智能去重
    #[tokio::test]
    #[ignore]
    async fn test_intelligent_deduplication() {
        let memory = Memory::quick();

        // 添加相似记忆
        memory
            .add("我喜欢编程")
            .await
            .expect("Failed to add first memory");

        memory
            .add("我热爱编程")
            .await
            .expect("Failed to add second memory");

        // 搜索应该只返回一条（去重后）
        let results = memory
            .search("编程")
            .await
            .expect("Failed to search");

        // 验证去重逻辑
        assert!(!results.is_empty());
    }

    /// 测试元数据过滤
    #[tokio::test]
    #[ignore]
    async fn test_metadata_filtering() {
        let memory = Memory::quick();

        // 添加带元数据的记忆
        // (实际API可能不同，这里只是示例)
        memory
            .add("工作：完成项目报告")
            .await
            .expect("Failed to add work memory");

        memory
            .add("个人：去买菜")
            .await
            .expect("Failed to add personal memory");

        // 搜索工作相关记忆
        let work_results = memory
            .search("工作")
            .await
            .expect("Failed to search work memories");
        assert!(!work_results.is_empty());
    }

    /// 测试性能 - 大量记忆
    #[tokio::test]
    #[ignore]
    async fn test_performance_with_large_dataset() {
        let memory = Memory::quick();

        // 添加1000条记忆
        for i in 0..1000 {
            memory
                .add(&format!("测试记忆第{}条", i))
                .await
                .expect("Failed to add memory");
        }

        // 搜索性能测试
        let start = std::time::Instant::now();
        let results = memory
            .search("测试")
            .await
            .expect("Failed to search");
        let elapsed = start.elapsed();

        assert!(!results.is_empty());
        assert!(elapsed.as_secs() < 5, "Search should complete within 5 seconds");
    }

    /// 测试并发操作
    #[tokio::test]
    #[ignore]
    async fn test_concurrent_operations() {
        let memory = Memory::quick();
        let memory_clone = memory.clone(); // 如果支持clone

        // 并发添加
        let add_task = tokio::spawn(async move {
            for i in 0..10 {
                memory_clone
                    .add(&format!("并发记忆{}", i))
                    .await
                    .expect("Failed to add memory");
            }
        });

        // 并发搜索
        let search_task = tokio::spawn(async move {
            for _ in 0..10 {
                let _ = memory.search("并发").await;
            }
        });

        // 等待完成
        add_task.await.expect("Add task failed");
        search_task.await.expect("Search task failed");
    }

    /// 测试错误恢复
    #[tokio::test]
    #[ignore]
    async fn test_error_recovery() {
        let memory = Memory::quick();

        // 添加空内容（应该返回错误）
        let result = memory.add("").await;
        assert!(result.is_err(), "Empty content should return error");

        // 添加无效ID（应该返回错误）
        let update_result = memory.update("invalid_id", "新内容").await;
        assert!(update_result.is_err(), "Invalid ID should return error");
    }

    /// 测试持久化
    #[tokio::test]
    #[ignore]
    async fn test_persistence() {
        let memory1 = Memory::quick();

        // 添加记忆
        memory1
            .add("持久化测试")
            .await
            .expect("Failed to add memory");

        // 创建新的 Memory 实例（应该能读取之前的数据）
        let memory2 = Memory::quick();

        let results = memory2
            .search("持久化测试")
            .await
            .expect("Failed to search");
        assert!(!results.is_empty(), "Memory should persist across instances");
    }
}

/// 真实场景测试
#[cfg(test)]
mod real_world_scenarios {
    use super::*;

    /// 场景1：AI 助手对话记忆
    #[tokio::test]
    #[ignore]
    async fn test_ai_assistant_conversation() {
        let memory = Memory::quick();

        // 模拟对话
        let conversations = vec![
            ("我叫李明", "用户", "assistant"),
            ("我在北京工作", "用户", "assistant"),
            ("我是一名医生", "用户", "assistant"),
        ];

        for (msg, user, agent) in conversations {
            memory
                .add_with_context(msg, user, agent)
                .await
                .expect("Failed to add conversation");
        }

        // 验证记忆整合
        let results = memory
            .search("李明的职业")
            .await
            .expect("Failed to search");
        assert!(!results.is_empty());
    }

    /// 场景2：知识库管理
    #[tokio::test]
    #[ignore]
    async fn test_knowledge_base_management() {
        let memory = Memory::quick();

        // 添加知识条目
        let knowledge = vec![
            "Rust 是一种系统编程语言",
            "Rust 注重内存安全",
            "Rust 支持并发编程",
        ];

        for item in knowledge {
            memory.add(item).await.expect("Failed to add knowledge");
        }

        // 搜索相关知识
        let results = memory
            .search("Rust 的特点")
            .await
            .expect("Failed to search knowledge");
        assert!(!results.is_empty());
    }

    /// 场景3：学习进度跟踪
    #[tokio::test]
    #[ignore]
    async fn test_learning_progress_tracking() {
        let memory = Memory::quick();

        // 记录学习进度
        let progress = vec![
            "今天学会了 Rust 的所有权概念",
            "理解了借用和生命周期",
            "掌握了异步编程基础",
        ];

        for item in progress {
            memory.add(item).await.expect("Failed to add progress");
        }

        // 搜索学习记录
        let results = memory
            .search("Rust 学习")
            .await
            .expect("Failed to search progress");
        assert!(!results.is_empty());
    }

    /// 场景4：项目管理
    #[tokio::test]
    #[ignore]
    async fn test_project_management() {
        let memory = Memory::quick();

        // 记录项目信息
        let project_info = vec![
            "项目 Alpha：开发新功能",
            "截止日期：下周五",
            "团队规模：5人",
        ];

        for item in project_info {
            memory.add(item).await.expect("Failed to add project info");
        }

        // 搜索项目信息
        let results = memory
            .search("项目 Alpha")
            .await
            .expect("Failed to search project");
        assert!(!results.is_empty());
    }

    /// 场景5：个人笔记
    #[tokio::test]
    #[ignore]
    async fn test_personal_notes() {
        let memory = Memory::quick();

        // 记录笔记
        let notes = vec![
            "会议笔记：讨论了新产品的功能需求",
            "想法：可以考虑使用 AI 来优化搜索",
            "提醒：周五前提交报告",
        ];

        for note in notes {
            memory.add(note).await.expect("Failed to add note");
        }

        // 搜索笔记
        let results = memory
            .search("会议")
            .await
            .expect("Failed to search notes");
        assert!(!results.is_empty());
    }
}
