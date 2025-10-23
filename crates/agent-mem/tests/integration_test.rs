//! 集成测试 - Memory 统一 API

use agent_mem::Memory;

#[tokio::test]
async fn test_zero_config_initialization() {
    // 测试零配置初始化
    let result = Memory::new().await;
    assert!(result.is_ok(), "Memory::new() 应该成功");

    let mem = result.unwrap();
    println!("✅ Memory 零配置初始化成功");
}

#[tokio::test]
async fn test_builder_pattern() {
    // 测试 Builder 模式
    let result = Memory::builder()
        .with_storage("libsql://test_builder.db")
        .with_user("test_user")
        .with_agent("test_agent")
        .build()
        .await;

    assert!(result.is_ok(), "Memory::builder().build() 应该成功");

    let mem = result.unwrap();
    println!("✅ Memory Builder 模式初始化成功");
}

#[tokio::test]
async fn test_add_memory() {
    // 测试添加记忆
    let mem = Memory::new().await.expect("初始化失败");

    let result = mem.add("I love pizza").await;
    assert!(result.is_ok(), "add() 应该成功");

    let add_result = result.unwrap();
    assert!(!add_result.results.is_empty(), "记忆 ID 不应为空");

    println!("✅ 添加记忆成功: {:?}", add_result);
}

#[tokio::test]
async fn test_search_memory() {
    // 测试搜索记忆
    let mem = Memory::new().await.expect("初始化失败");

    // 先添加一些记忆
    mem.add("I love pizza").await.expect("添加失败");
    mem.add("I work at Google").await.expect("添加失败");

    // 搜索
    let result = mem.search("What do you know about me?").await;
    assert!(result.is_ok(), "search() 应该成功");

    let memories = result.unwrap();
    println!("✅ 搜索成功，找到 {} 条记忆", memories.len());
}

#[tokio::test]
async fn test_get_all_memories() {
    // 测试获取所有记忆
    let mem = Memory::new().await.expect("初始化失败");

    // 添加记忆
    mem.add("Memory 1").await.expect("添加失败");
    mem.add("Memory 2").await.expect("添加失败");
    mem.add("Memory 3").await.expect("添加失败");

    // 获取所有记忆
    use agent_mem::types::GetAllOptions;
    let result = mem.get_all(GetAllOptions::default()).await;
    assert!(result.is_ok(), "get_all() 应该成功");

    let memories = result.unwrap();
    println!("✅ 获取所有记忆成功，总共 {} 条", memories.len());
}

#[tokio::test]
async fn test_get_stats() {
    // 测试获取统计信息
    let mem = Memory::new().await.expect("初始化失败");

    // 添加一些记忆
    mem.add("Test memory 1").await.expect("添加失败");
    mem.add("Test memory 2").await.expect("添加失败");

    // 获取统计信息
    let result = mem.get_stats().await;
    assert!(result.is_ok(), "get_stats() 应该成功");

    let stats = result.unwrap();
    println!("✅ 获取统计信息成功:");
    println!("   - 总记忆数: {}", stats.total_memories);
    println!("   - 平均重要性: {:.2}", stats.average_importance);
}
