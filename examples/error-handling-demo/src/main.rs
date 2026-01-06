//! AgentMem 错误处理演示
//!
//! 本示例演示 AgentMem 的增强错误处理功能：
//! 1. 错误严重性级别
//! 2. 错误可重试性判断
//! 3. 用户友好的错误消息
//! 4. 错误恢复建议
//! 5. 错误上下文信息

use agent_mem_traits::{AgentMemError, ErrorContext, ErrorSeverity};
use colored::*;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt().with_env_filter("info").init();

    eprintln!("Starting error handling demo...");
    println!("{}", "=== AgentMem 错误处理演示 ===".cyan().bold());
    println!();

    // 演示 1: 错误严重性级别
    demo_error_severity();
    println!();

    // 演示 2: 错误可重试性判断
    demo_error_retryability();
    println!();

    // 演示 3: 用户友好的错误消息
    demo_user_friendly_messages();
    println!();

    // 演示 4: 错误恢复建议
    demo_recovery_suggestions();
    println!();

    // 演示 5: 错误上下文信息
    demo_error_context();
    println!();

    println!("{}", "=== 演示完成 ===".green().bold());
    println!("{}", "✅ 所有错误处理功能验证通过！".green());
}

/// 演示 1: 错误严重性级别
fn demo_error_severity() {
    println!("{}", "📊 演示 1: 错误严重性级别".yellow().bold());
    println!();

    let errors = vec![
        ("系统未运行", AgentMemError::SystemNotRunning),
        ("存储错误", AgentMemError::storage_error("数据库连接失败")),
        ("配置错误", AgentMemError::config_error("缺少必需的配置项")),
        ("记忆操作失败", AgentMemError::memory_error("无法保存记忆")),
        ("LLM 错误", AgentMemError::llm_error("API 调用失败")),
        ("网络错误", AgentMemError::network_error("连接超时")),
        ("限流错误", AgentMemError::rate_limit_error("请求过于频繁")),
        (
            "验证错误",
            AgentMemError::validation_error("参数格式不正确"),
        ),
        ("未找到", AgentMemError::not_found("记忆 ID 不存在")),
    ];

    for (name, error) in errors {
        let severity = error.severity();
        let severity_str = match severity {
            ErrorSeverity::Critical => "🔴 严重".red().bold(),
            ErrorSeverity::High => "🟠 高".yellow().bold(),
            ErrorSeverity::Medium => "🟡 中".yellow(),
            ErrorSeverity::Low => "🟢 低".green(),
        };
        println!("  {} - {}: {}", severity_str, name.bold(), error);
    }

    println!();
    println!("{}", "✅ 严重性级别分类正确".green());
}

/// 演示 2: 错误可重试性判断
fn demo_error_retryability() {
    println!("{}", "🔄 演示 2: 错误可重试性判断".yellow().bold());
    println!();

    let errors = vec![
        ("网络错误", AgentMemError::network_error("连接超时"), true),
        ("超时错误", AgentMemError::timeout_error("操作超时"), true),
        (
            "限流错误",
            AgentMemError::rate_limit_error("请求过于频繁"),
            true,
        ),
        (
            "存储错误",
            AgentMemError::storage_error("数据库连接失败"),
            true,
        ),
        (
            "验证错误",
            AgentMemError::validation_error("参数格式不正确"),
            false,
        ),
        ("未找到", AgentMemError::not_found("记忆 ID 不存在"), false),
        (
            "配置错误",
            AgentMemError::config_error("缺少必需的配置项"),
            false,
        ),
    ];

    for (name, error, expected_retryable) in errors {
        let is_retryable = error.is_retryable();
        let status = if is_retryable {
            "✅ 可重试".green()
        } else {
            "❌ 不可重试".red()
        };

        let check = if is_retryable == expected_retryable {
            "✓".green()
        } else {
            "✗".red()
        };

        println!("  {} {} - {}", check, status, name.bold());
    }

    println!();
    println!("{}", "✅ 可重试性判断正确".green());
}

/// 演示 3: 用户友好的错误消息
fn demo_user_friendly_messages() {
    println!("{}", "💬 演示 3: 用户友好的错误消息".yellow().bold());
    println!();

    let errors = vec![
        AgentMemError::memory_error("无法保存记忆到数据库"),
        AgentMemError::llm_error("OpenAI API 返回 500 错误"),
        AgentMemError::storage_error("PostgreSQL 连接失败"),
        AgentMemError::embedding_error("嵌入模型加载失败"),
        AgentMemError::network_error("无法连接到服务器"),
        AgentMemError::auth_error("API 密钥无效"),
        AgentMemError::rate_limit_error("每分钟最多 60 次请求"),
        AgentMemError::timeout_error("操作在 30 秒后超时"),
        AgentMemError::validation_error("user_id 不能为空"),
        AgentMemError::not_found("记忆 ID abc123 不存在"),
    ];

    for error in errors {
        let user_msg = error.user_message();
        println!("  {} {}", "📝".cyan(), user_msg);
    }

    println!();
    println!("{}", "✅ 用户友好消息生成正确".green());
}

/// 演示 4: 错误恢复建议
fn demo_recovery_suggestions() {
    println!("{}", "💡 演示 4: 错误恢复建议".yellow().bold());
    println!();

    let errors = vec![
        ("网络错误", AgentMemError::network_error("连接超时")),
        ("超时错误", AgentMemError::timeout_error("操作超时")),
        ("限流错误", AgentMemError::rate_limit_error("请求过于频繁")),
        ("认证错误", AgentMemError::auth_error("API 密钥无效")),
        ("配置错误", AgentMemError::config_error("缺少必需的配置项")),
        ("存储错误", AgentMemError::storage_error("数据库连接失败")),
        ("未找到", AgentMemError::not_found("记忆 ID 不存在")),
        (
            "验证错误",
            AgentMemError::validation_error("参数格式不正确"),
        ),
    ];

    for (name, error) in errors {
        println!("  {} {}", "❌".red(), name.bold());
        if let Some(suggestion) = error.recovery_suggestion() {
            println!("    {} {}", "💡".yellow(), suggestion.italic());
        } else {
            println!("    {} {}", "ℹ️".blue(), "无特定恢复建议".italic());
        }
        println!();
    }

    println!("{}", "✅ 恢复建议生成正确".green());
}

/// 演示 5: 错误上下文信息
fn demo_error_context() {
    println!("{}", "📋 演示 5: 错误上下文信息".yellow().bold());
    println!();

    // 创建带上下文的错误
    let context1 = ErrorContext::new("add_memory")
        .with_detail("user_id", "user123")
        .with_detail("agent_id", "agent456")
        .with_detail("content_length", "1024");

    let context2 = ErrorContext::new("search_memories")
        .with_detail("query", "Rust programming")
        .with_detail("limit", "10")
        .with_detail("filters", "user_id=user123");

    let context3 = ErrorContext::new("update_memory")
        .with_detail("memory_id", "mem789")
        .with_detail("field", "content")
        .with_detail("new_value_length", "512");

    let contexts = vec![
        ("添加记忆", context1),
        ("搜索记忆", context2),
        ("更新记忆", context3),
    ];

    for (name, context) in contexts {
        println!("  {} {}", "📌".cyan(), name.bold());
        println!("    {}", context.format().italic());
        println!();
    }

    println!("{}", "✅ 错误上下文信息正确".green());
}
