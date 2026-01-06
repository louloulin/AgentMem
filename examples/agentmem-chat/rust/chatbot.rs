//! AgentMem 聊天机器人示例
//!
//! 这个示例演示了如何构建一个智能聊天机器人：
//! - 对话历史管理
//! - 上下文检索
//! - 个性化回复
//! - 多轮对话
//!
//! # 运行方式
//!
//! ```bash
//! export OPENAI_API_KEY=sk-...
//! cargo run --example chatbot
//! ```
//!
//! # 预期输出
//!
//! ```text
//! 🤖 AgentMem 聊天机器人示例
//!
//! ✅ 初始化完成
//!
//! 💬 对话 1:
//!    用户: 我叫 Alice
//!    🤖: 很高兴认识你，Alice！
//!    ✅ 已保存记忆
//!
//! 💬 对话 2:
//!    用户: 我喜欢编程
//!    🤖: 编程很棒！
//!    ✅ 已保存记忆
//!
//! 💬 对话 3:
//!    用户: 我叫什么名字？
//!    🤖: 你叫 Alice。
//!    ✅ 从记忆中检索到: 我叫 Alice
//!
//! 💬 对话 4:
//!    用户: 我有什么爱好？
//!    🤖: 你喜欢编程。
//!    ✅ 从记忆中检索到: 我喜欢编程
//!
//! 🎉 对话结束！
//! ```

use agent_mem::{GetAllOptions, Memory};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AgentMem 聊天机器人示例\n");
    println!("这个示例演示了:");
    println!("  1. 对话历史管理");
    println!("  2. 上下文检索");
    println!("  3. 个性化回复");
    println!("  4. 多轮对话");
    println!();

    // 初始化
    let mem = Memory::new().await?;
    println!("✅ 初始化完成\n");

    // ============================================
    // 演示 1: 简单对话
    // ============================================
    println!("💬 演示 1: 简单对话");
    println!("---");

    let conversations = vec![
        "我叫 Alice",
        "我喜欢编程",
        "我住在上海",
    ];

    for msg in conversations {
        println!("   用户: {}", msg);

        // 添加用户消息到记忆
        let result = mem.add(msg).await?;
        println!("   ✅ 已保存: {}", result.id);

        // 生成简单回复
        let reply = generate_simple_reply(&msg, &mem).await?;
        println!("   🤖: {}", reply);
        println!();
    }

    // ============================================
    // 演示 2: 基于记忆的回复
    // ============================================
    println!("💬 演示 2: 基于记忆的回复");
    println!("---");

    let questions = vec![
        "我叫什么名字？",
        "我有什么爱好？",
        "我住在哪里？",
    ];

    for question in questions {
        println!("   用户: {}", question);

        // 搜索相关记忆
        let context = search_context(&mem, question).await?;

        // 基于上下文生成回复
        let reply = if let Some(ctx) = context {
            println!("   ✅ 检索到上下文: {}", ctx);
            format!("根据记忆，{}", ctx)
        } else {
            "抱歉，我不记得了。".to_string()
        };

        println!("   🤖: {}", reply);
        println!();
    }

    // ============================================
    // 演示 3: 交互式对话
    // ============================================
    println!("💬 演示 3: 交互式对话（可选）");
    println!("---");
    println!("   输入消息与机器人对话（输入 'quit' 退出）");
    println!("   或者直接按 Enter 跳过交互式演示");
    print!("   > ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim() != "quit" && !input.trim().is_empty() {
        // 交互式对话
        let mut turn = 1;

        loop {
            print!("   用户[{}]: ", turn);
            io::stdout().flush()?;

            let mut user_msg = String::new();
            io::stdin().read_line(&mut user_msg)?;
            let user_msg = user_msg.trim();

            if user_msg == "quit" {
                println!("   👋 再见！");
                break;
            }

            if user_msg.is_empty() {
                continue;
            }

            // 保存用户消息
            mem.add(user_msg).await?;

            // 搜索上下文并生成回复
            let context = search_context(&mem, user_msg).await?;
            let reply = generate_reply_with_context(user_msg, context.as_deref()).await;

            println!("   🤖: {}", reply);

            turn += 1;
        }
    } else {
        println!("   跳过交互式演示");
    }

    // ============================================
    // 总结
    // ============================================
    println!("\n🎉 对话演示完成！");

    let all_memories = mem.get_all(GetAllOptions::default()).await?;
    println!("\n📊 对话统计:");
    println!("   总记忆数: {}", all_memories.len());

    println!("\n💡 构建聊天机器人的关键:");
    println!("   1. 保存每条用户消息到记忆");
    println!("   2. 搜索相关上下文");
    println!("   3. 基于上下文生成个性化回复");
    println!("   4. 维护对话历史");
    println!("   5. 处理多轮对话");

    Ok(())
}

// ============================================
// 辅助函数
// ============================================

/// 生成简单回复
async fn generate_simple_reply(msg: &str, mem: &Memory) -> Result<String, Box<dyn std::error::Error>> {
    Ok(if msg.contains("我叫") {
        let name = msg.replace("我叫", "").trim().to_string();
        format!("很高兴认识你，{}！", name)
    } else if msg.contains("我喜欢") {
        let hobby = msg.replace("我喜欢", "").trim().to_string();
        format!("{}很棒！", hobby)
    } else if msg.contains("我住在") {
        let place = msg.replace("我住在", "").trim().to_string();
        format!("{}是个好地方！", place)
    } else {
        "我明白了！".to_string()
    })
}

/// 搜索上下文
async fn search_context(
    mem: &Memory,
    query: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let results = mem.search(query).await?;

    if let Some(result) = results.first() {
        Ok(Some(result.content.clone()))
    } else {
        Ok(None)
    }
}

/// 基于上下文生成回复
async fn generate_reply_with_context(question: &str, context: Option<&str>) -> String {
    if let Some(ctx) = context {
        // 基于上下文回答
        if question.contains("名字") {
            format!("你的名字是{}", ctx.replace("我叫", ""))
        } else if question.contains("爱好") {
            format!("你喜欢{}", ctx.replace("我喜欢", ""))
        } else if question.contains("住") {
            format!("你住在{}", ctx.replace("我住在", ""))
        } else {
            format!("我记得：{}", ctx)
        }
    } else {
        "抱歉，我不记得了。".to_string()
    }
}

// ============================================
// 高级示例: 使用 LLM 生成回复
// ============================================
//
// 如果你想使用真实的 LLM 生成回复，可以这样:
//
// ```rust
// use agent_mem_llm::LLMProvider;
//
// async fn generate_llm_reply(
//     question: &str,
//     context: Option<&str>,
//     llm: &LLMProvider,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let prompt = if let Some(ctx) = context {
//         format!(
//             "基于以下上下文回答问题:\n\n上下文: {}\n\n问题: {}",
//             ctx, question
//         )
//     } else {
//         question.to_string()
//     };
//
//     let response = llm.generate(&prompt).await?;
//     Ok(response)
// }
// ```
//
// 然后在对话循环中:
//
// ```rust
// let llm = LLMProvider::new();
// let reply = generate_llm_reply(user_msg, context.as_deref(), &llm).await?;
// ```
