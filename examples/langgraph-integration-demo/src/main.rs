//! LangGraph 集成演示

use agent_mem_core::{AgentMemClient, MemoryType, Messages};
use agent_mem_traits::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// 对话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationState {
    messages: Vec<Message>,
    user_id: String,
    user_name: String,
    session_id: String,
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    
    println!("{}", "=== AgentMem LangGraph 集成演示 ===".cyan().bold());
    
    let client = Arc::new(AgentMemClient::default());
    println!("{}", "✅ AgentMemClient 创建成功".green());
    
    println!("{}", "=== 演示完成 ===".green().bold());
    
    Ok(())
}
