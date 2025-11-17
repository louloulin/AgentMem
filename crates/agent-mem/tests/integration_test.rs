//! 集成测试 - Memory 统一 API

use agent_mem::types::GetAllOptions;
use agent_mem::Memory;
use chrono::Utc;
use serde_json::json;

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

#[tokio::test]
async fn test_add_text_convenience_api() {
    let mem = Memory::new().await.expect("初始化失败");

    let user_id = format!("user-text-{}", Utc::now().timestamp_nanos_opt().unwrap());
    let agent_id = format!("agent-text-{}", Utc::now().timestamp_millis());

    let result = mem
        .add_text("User loves latte art", &agent_id, Some(&user_id))
        .await;
    assert!(result.is_ok(), "add_text() 应该成功");

    let options = GetAllOptions {
        user_id: Some(user_id.clone()),
        agent_id: Some(agent_id.clone()),
        run_id: None,
        limit: None,
    };

    let memories = mem.get_all(options).await.expect("get_all 失败");
    assert!(
        memories.iter().any(|m| m.content.contains("latte art")),
        "应当找到 add_text 写入的内容"
    );
    assert!(
        memories.iter().all(|m| m.user_id.as_deref() == Some(&user_id)),
        "返回结果应绑定到指定 user_id"
    );
    assert!(
        memories.iter().all(|m| m.agent_id == agent_id),
        "返回结果应绑定到指定 agent_id"
    );
}

#[tokio::test]
async fn test_add_structured_convenience_api() {
    let mem = Memory::new().await.expect("初始化失败");

    let user_id = format!(
        "user-structured-{}",
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    let agent_id = format!("agent-structured-{}", Utc::now().timestamp_millis());
    let payload = json!({
        "type": "user_profile",
        "name": "Alice",
        "preferences": ["coffee", "latte"]
    });

    let result = mem
        .add_structured(payload.clone(), &agent_id, Some(&user_id))
        .await;
    assert!(result.is_ok(), "add_structured() 应该成功");

    let options = GetAllOptions {
        user_id: Some(user_id.clone()),
        agent_id: Some(agent_id.clone()),
        run_id: None,
        limit: Some(10),
    };

    let memories = mem.get_all(options).await.expect("get_all 失败");
    assert!(
        memories.iter().any(|m| m.content.contains("user_profile")),
        "应当包含结构化内容被序列化后的字段"
    );

    let structured_metadata = memories
        .iter()
        .flat_map(|m| m.metadata.get("content_format"))
        .find(|value| value.as_str() == Some("structured_json"));

    assert!(
        structured_metadata.is_some(),
        "metadata 中应包含 content_format=structured_json"
    );
}
