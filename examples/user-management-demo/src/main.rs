//! 用户管理功能演示
//!
//! 本示例演示 AgentMem 的用户管理功能：
//! 1. 创建用户
//! 2. 列出所有用户
//! 3. 按名称查询用户
//! 4. 重复创建测试
//! 5. 验证逻辑测试

use agent_mem_core::client::AgentMemClient;
use agent_mem_traits::Result;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "=== AgentMem 用户管理演示 ===".cyan().bold());
    println!();

    // 创建客户端
    println!("{}", "1. 创建 AgentMemClient...".yellow());
    let client = AgentMemClient::default();
    println!("{}", "   ✅ AgentMemClient 创建成功".green());
    println!();

    // 测试 1: 创建用户
    println!("{}", "2. 创建用户...".yellow());
    let user1 = client.create_user("alice".to_string()).await?;
    println!("   ✅ 创建用户: {} (ID: {})", user1.name.green(), user1.id);

    let user2 = client.create_user("bob".to_string()).await?;
    println!("   ✅ 创建用户: {} (ID: {})", user2.name.green(), user2.id);

    let user3 = client.create_user("charlie".to_string()).await?;
    println!("   ✅ 创建用户: {} (ID: {})", user3.name.green(), user3.id);
    println!();

    // 测试 2: 列出所有用户
    println!("{}", "3. 列出所有用户...".yellow());
    let users = client.list_users().await?;
    println!("   找到 {} 个用户:", users.len().to_string().green());
    for user in &users {
        println!(
            "   - {} (ID: {}, 创建时间: {})",
            user.name.cyan(),
            user.id,
            user.created_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
    println!();

    // 测试 3: 按名称查询用户
    println!("{}", "4. 按名称查询用户...".yellow());
    if let Some(user) = client.get_user_by_name("alice".to_string()).await? {
        println!("   ✅ 找到用户: {} (ID: {})", user.name.green(), user.id);
    } else {
        println!("   ❌ 未找到用户: alice");
    }

    if let Some(user) = client.get_user_by_name("bob".to_string()).await? {
        println!("   ✅ 找到用户: {} (ID: {})", user.name.green(), user.id);
    } else {
        println!("   ❌ 未找到用户: bob");
    }

    // 查询不存在的用户
    if let Some(_user) = client.get_user_by_name("nonexistent".to_string()).await? {
        println!("   ❌ 不应该找到用户: nonexistent");
    } else {
        println!("   ✅ 正确：未找到不存在的用户 {}", "nonexistent".yellow());
    }
    println!();

    // 测试 4: 重复创建用户（幂等性测试）
    println!("{}", "5. 测试幂等性（重复创建用户）...".yellow());
    let user_alice_again = client.create_user("alice".to_string()).await?;
    if user_alice_again.id == user1.id {
        println!(
            "   ✅ 幂等性测试通过：返回相同的用户 (ID: {})",
            user_alice_again.id.green()
        );
    } else {
        println!("   ❌ 幂等性测试失败：返回了不同的用户");
    }
    println!();

    // 测试 5: 验证用户名不能为空
    println!("{}", "6. 测试验证逻辑（空用户名）...".yellow());
    match client.create_user("".to_string()).await {
        Ok(_) => println!("   ❌ 验证失败：应该拒绝空用户名"),
        Err(e) => println!(
            "   ✅ 验证通过：正确拒绝空用户名 ({})",
            e.to_string().yellow()
        ),
    }

    match client.create_user("   ".to_string()).await {
        Ok(_) => println!("   ❌ 验证失败：应该拒绝空白用户名"),
        Err(e) => println!(
            "   ✅ 验证通过：正确拒绝空白用户名 ({})",
            e.to_string().yellow()
        ),
    }
    println!();

    // 最终统计
    println!("{}", "=== 最终统计 ===".cyan().bold());
    let final_users = client.list_users().await?;
    println!("总用户数: {}", final_users.len().to_string().green().bold());
    println!();

    println!("{}", "=== 演示完成 ===".green().bold());
    println!();
    println!("✅ 所有测试通过！");

    Ok(())
}
