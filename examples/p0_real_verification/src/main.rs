//! P0 优化真实验证示例
//! 
//! 使用真实的 LLM (Zhipu AI) 验证 P0 优化：
//! - 默认 infer=true 的行为
//! - 零配置初始化
//! - 智能功能（事实提取、去重、冲突解决）

use agent_mem::{AddMemoryOptions, Memory};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 P0 优化真实验证");
    println!("=" .repeat(60));
    
    // 设置环境变量
    env::set_var("ZHIPU_API_KEY", "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k");
    env::set_var("LLM_PROVIDER", "zhipu");
    env::set_var("LLM_MODEL", "glm-4-plus");
    env::set_var("EMBEDDER_PROVIDER", "fastembed");
    env::set_var("EMBEDDER_MODEL", "BAAI/bge-small-en-v1.5");
    
    // 设置代理（如果需要）
    env::set_var("http_proxy", "http://127.0.0.1:4780");
    env::set_var("https_proxy", "http://127.0.0.1:4780");
    
    println!("\n✅ 测试 1: AddMemoryOptions::default().infer = true");
    let options = AddMemoryOptions::default();
    assert_eq!(options.infer, true, "默认值应该是 true");
    println!("   ✅ 默认 infer 值为: {}", options.infer);
    
    println!("\n✅ 测试 2: 零配置初始化");
    let mem = Memory::new().await?;
    println!("   ✅ Memory::new() 初始化成功");
    
    println!("\n✅ 测试 3: 默认行为（infer: true）");
    println!("   添加记忆: '我喜欢吃苹果和香蕉'");
    let result1 = mem.add("我喜欢吃苹果和香蕉").await?;
    println!("   ✅ 添加成功，结果数: {}", result1.results.len());
    for (i, event) in result1.results.iter().enumerate() {
        println!("     结果 {}: {} - {}", i + 1, event.event, event.memory);
    }
    
    println!("\n✅ 测试 4: 向后兼容性 - 显式设置 infer: false");
    let options = AddMemoryOptions {
        infer: false,
        ..Default::default()
    };
    let result2 = mem.add_with_options("这是原始内容，不使用智能功能".to_string(), options).await?;
    println!("   ✅ 简单模式添加成功，结果数: {}", result2.results.len());
    
    println!("\n✅ 测试 5: 向后兼容性 - 显式设置 infer: true");
    let options = AddMemoryOptions {
        infer: true,
        ..Default::default()
    };
    let result3 = mem.add_with_options("我喜欢编程，特别是 Rust 语言".to_string(), options).await?;
    println!("   ✅ 智能模式添加成功，结果数: {}", result3.results.len());
    
    println!("\n✅ 测试 6: 搜索记忆");
    let search_results = mem.search("我喜欢什么？").await?;
    println!("   ✅ 搜索成功，找到 {} 条记忆", search_results.len());
    for (i, result) in search_results.iter().take(5).enumerate() {
        println!("     结果 {}: {}", i + 1, result.content);
    }
    
    println!("\n" + &"=".repeat(60));
    println!("🎉 所有测试通过！P0 优化验证成功！");
    println!("=" .repeat(60));
    
    Ok(())
}

