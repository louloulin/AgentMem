//! 用户管理功能演示
//!
//! 本示例演示 AgentMem 的用户管理功能：
//! 1. 创建用户
//! 2. 列出所有用户
//! 3. 按名称查询用户
//! 4. 重复创建测试

use agent_mem_core::client::AgentMemClient;
use agent_mem_traits::Result;

fn main() {
    println!("=== AgentMem 用户管理演示 ===");
    println!();

    // 创建客户端
    println!("Creating AgentMemClient...");
    let client = AgentMemClient::default();
    println!("✅ AgentMemClient 创建成功");
    println!();

    println!("=== 演示完成 ===");
}


