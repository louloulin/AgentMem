//! 用户管理功能集成测试

use agent_mem_core::client::AgentMemClient;
use agent_mem_traits::Result;

#[tokio::test]
async fn test_create_user() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let user = client.create_user("test_user".to_string()).await?;

    // 验证用户属性
    assert_eq!(user.name, "test_user");
    assert!(!user.id.is_empty());

    println!("✅ test_create_user passed");
    Ok(())
}

#[tokio::test]
async fn test_create_user_idempotent() -> Result<()> {
    let client = AgentMemClient::default();

    // 第一次创建
    let user1 = client.create_user("alice".to_string()).await?;

    // 第二次创建相同用户名
    let user2 = client.create_user("alice".to_string()).await?;

    // 应该返回相同的用户
    assert_eq!(user1.id, user2.id);
    assert_eq!(user1.name, user2.name);

    println!("✅ test_create_user_idempotent passed");
    Ok(())
}

#[tokio::test]
async fn test_list_users() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建几个用户
    client.create_user("user1".to_string()).await?;
    client.create_user("user2".to_string()).await?;
    client.create_user("user3".to_string()).await?;

    // 列出所有用户
    let users = client.list_users().await?;

    // 应该有3个用户
    assert_eq!(users.len(), 3);

    // 验证用户名
    let names: Vec<String> = users.iter().map(|u| u.name.clone()).collect();
    assert!(names.contains(&"user1".to_string()));
    assert!(names.contains(&"user2".to_string()));
    assert!(names.contains(&"user3".to_string()));

    println!("✅ test_list_users passed");
    Ok(())
}

#[tokio::test]
async fn test_get_user_by_name() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建用户
    let created_user = client.create_user("bob".to_string()).await?;

    // 按名称查询
    let found_user = client.get_user_by_name("bob".to_string()).await?;

    // 应该找到用户
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.name, "bob");

    println!("✅ test_get_user_by_name passed");
    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_user() -> Result<()> {
    let client = AgentMemClient::default();

    // 查询不存在的用户
    let user = client.get_user_by_name("nonexistent".to_string()).await?;

    // 应该返回 None
    assert!(user.is_none());

    println!("✅ test_get_nonexistent_user passed");
    Ok(())
}

#[tokio::test]
async fn test_create_user_empty_name() {
    let client = AgentMemClient::default();

    // 尝试创建空用户名
    let result = client.create_user("".to_string()).await;

    // 应该失败
    assert!(result.is_err());

    println!("✅ test_create_user_empty_name passed");
}

#[tokio::test]
async fn test_create_user_whitespace_name() {
    let client = AgentMemClient::default();

    // 尝试创建空白用户名
    let result = client.create_user("   ".to_string()).await;

    // 应该失败
    assert!(result.is_err());

    println!("✅ test_create_user_whitespace_name passed");
}

#[tokio::test]
async fn test_multiple_users() -> Result<()> {
    let client = AgentMemClient::default();

    // 创建多个用户
    let users_to_create = vec!["alice", "bob", "charlie", "david", "eve"];

    for name in &users_to_create {
        client.create_user(name.to_string()).await?;
    }

    // 列出所有用户
    let users = client.list_users().await?;
    assert_eq!(users.len(), users_to_create.len());

    // 验证每个用户都能被查询到
    for name in &users_to_create {
        let user = client.get_user_by_name(name.to_string()).await?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, *name);
    }

    println!("✅ test_multiple_users passed");
    Ok(())
}
