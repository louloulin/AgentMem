//! 批量操作功能测试
//!
//! 验证 AgentMem 的批量操作 API

use agent_mem::{AddMemoryOptions, Memory};

#[tokio::test]
async fn test_add_batch_basic() {
    // 测试基本的批量添加
    let mem = Memory::new().await.expect("初始化失败");

    let contents = vec![
        "我喜欢喝咖啡".to_string(),
        "我喜欢读书".to_string(),
        "我在学习 Rust".to_string(),
    ];

    let results = mem
        .add_batch(contents, AddMemoryOptions::default())
        .await
        .expect("批量添加失败");

    assert_eq!(results.len(), 3, "应该成功添加 3 条记忆");

    for (i, result) in results.iter().enumerate() {
        assert!(
            !result.results.is_empty(),
            "结果 {} 应该包含事件",
            i + 1
        );
    }

    println!("✅ 批量添加基本测试通过");
}

#[tokio::test]
async fn test_add_batch_with_user_scope() {
    // 测试带用户作用域的批量添加
    let mem = Memory::new().await.expect("初始化失败");

    let contents = vec![
        "记忆1".to_string(),
        "记忆2".to_string(),
    ];

    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        ..Default::default()
    };

    let results = mem.add_batch(contents, options).await.expect("批量添加失败");

    assert_eq!(results.len(), 2);
    println!("✅ 带用户作用域的批量添加测试通过");
}

#[tokio::test]
async fn test_add_batch_performance() {
    // 测试批量添加的性能
    let mem = Memory::new().await.expect("初始化失败");

    let contents: Vec<String> = (0..10)
        .map(|i| format!("测试记忆 {}", i))
        .collect();

    let start = std::time::Instant::now();
    let results = mem
        .add_batch(contents, AddMemoryOptions::default())
        .await
        .expect("批量添加失败");
    let duration = start.elapsed();

    assert_eq!(results.len(), 10);

    println!("✅ 批量添加性能测试:");
    println!("   - 记忆数: 10");
    println!("   - 总耗时: {:?}", duration);
    println!("   - 平均: {:?}/个", duration / 10);
}

#[tokio::test]
async fn test_add_batch_with_infer_false() {
    // 测试批量添加（禁用智能功能）
    let mem = Memory::new().await.expect("初始化失败");

    let contents = vec![
        "原始内容1".to_string(),
        "原始内容2".to_string(),
    ];

    let options = AddMemoryOptions {
        infer: false,
        ..Default::default()
    };

    let results = mem.add_batch(contents, options).await.expect("批量添加失败");

    assert_eq!(results.len(), 2);
    println!("✅ 批量添加（简单模式）测试通过");
}

#[tokio::test]
async fn test_add_batch_empty() {
    // 测试空批量添加
    let mem = Memory::new().await.expect("初始化失败");

    let contents: Vec<String> = vec![];

    let results = mem
        .add_batch(contents, AddMemoryOptions::default())
        .await
        .expect("空批量添加应该成功");

    assert_eq!(results.len(), 0, "空批量应该返回空结果");
    println!("✅ 空批量添加测试通过");
}

