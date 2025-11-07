//! MCP 传输层功能演示
//!
//! 演示 HTTP 和 SSE 传输层的使用

use agent_mem_tools::mcp::{HttpTransport, SseTransport, Transport};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tracing::error;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🚀 MCP 传输层功能演示");
    println!("============================================================\n");

    // 演示 1: HTTP 传输
    demo_http_transport().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 2: SSE 传输
    demo_sse_transport().await;

    println!("\n============================================================");
    println!("✅ 所有演示完成！");
}

/// 演示 HTTP 传输功能
async fn demo_http_transport() {
    println!("📋 演示 1: HTTP 传输层");
    println!("------------------------------------------------------------");

    // 1. 创建 HTTP 传输
    println!("\n1️⃣ 创建 HTTP 传输:");
    let mut headers = HashMap::new();
    headers.insert("X-API-Key".to_string(), "demo-api-key".to_string());
    headers.insert("X-Client-Version".to_string(), "1.0.0".to_string());

    let mut transport = HttpTransport::new("http://localhost:8080".to_string(), Some(headers))
        .with_timeout(Duration::from_secs(60))
        .add_header("X-Custom-Header", "custom-value");

    println!("  ✅ HTTP 传输创建成功");
    println!("  URL: http://localhost:8080");
    println!("  超时: 60 秒");
    println!("  自定义请求头: 3 个");

    // 2. 连接到服务器
    println!("\n2️⃣ 连接到服务器:");
    match transport.connect().await {
        Ok(_) => {
            println!("  ✅ 连接成功");
            println!(
                "  连接状态: {}",
                if transport.is_connected() {
                    "已连接"
                } else {
                    "未连接"
                }
            );
        }
        Err(e) => {
            error!("  ❌ 连接失败: {}", e);
            println!("  ⚠️  这是预期的（演示环境没有运行服务器）");
        }
    }

    // 3. 发送请求（模拟）
    println!("\n3️⃣ 发送 JSON-RPC 请求:");
    let request_params = json!({
        "method": "list_tools",
        "params": {}
    });

    println!("  请求方法: list_tools");
    println!("  请求参数: {request_params}");

    if transport.is_connected() {
        match transport.send_request("list_tools", json!({})).await {
            Ok(response) => {
                println!("  ✅ 请求成功");
                println!("  响应: {response}");
            }
            Err(e) => {
                println!("  ⚠️  请求失败: {e}");
                println!("  （演示环境没有运行服务器）");
            }
        }
    } else {
        println!("  ⚠️  未连接到服务器，跳过请求");
    }

    // 4. 断开连接
    println!("\n4️⃣ 断开连接:");
    match transport.disconnect().await {
        Ok(_) => {
            println!("  ✅ 断开连接成功");
            println!(
                "  连接状态: {}",
                if transport.is_connected() {
                    "已连接"
                } else {
                    "未连接"
                }
            );
        }
        Err(e) => {
            error!("  ❌ 断开连接失败: {}", e);
        }
    }

    // 5. 测试未连接状态下的请求
    println!("\n5️⃣ 测试未连接状态下的请求:");
    match transport.send_request("test_method", json!({})).await {
        Ok(_) => {
            println!("  ❌ 不应该成功");
        }
        Err(e) => {
            println!("  ✅ 正确返回错误: {e}");
        }
    }
}

/// 演示 SSE 传输功能
async fn demo_sse_transport() {
    println!("📋 演示 2: SSE 传输层");
    println!("------------------------------------------------------------");

    // 1. 创建 SSE 传输
    println!("\n1️⃣ 创建 SSE 传输:");
    let mut transport = SseTransport::new("http://localhost:8080/events".to_string());

    println!("  ✅ SSE 传输创建成功");
    println!("  URL: http://localhost:8080/events");
    println!(
        "  连接状态: {}",
        if transport.is_connected() {
            "已连接"
        } else {
            "未连接"
        }
    );

    // 2. 连接到服务器
    println!("\n2️⃣ 连接到 SSE 服务器:");
    match transport.connect().await {
        Ok(_) => {
            println!("  ✅ 连接成功");
            println!(
                "  连接状态: {}",
                if transport.is_connected() {
                    "已连接"
                } else {
                    "未连接"
                }
            );
            println!("  事件监听: 已启动");
        }
        Err(e) => {
            error!("  ❌ 连接失败: {}", e);
            println!("  ⚠️  这是预期的（演示环境没有运行服务器）");
        }
    }

    // 3. 模拟接收事件
    println!("\n3️⃣ 接收 SSE 事件:");
    println!("  ⚠️  SSE 是单向通信（服务器 → 客户端）");
    println!("  在实际应用中，事件会通过 receive_event() 接收");
    println!("  （演示环境中，我们跳过事件接收测试）");

    // 4. 测试发送请求（SSE 不支持）
    println!("\n4️⃣ 测试发送请求（SSE 不支持双向通信）:");
    if transport.is_connected() {
        match transport.send_request("test_method", json!({})).await {
            Ok(response) => {
                println!("  ⚠️  返回模拟响应: {response}");
                println!("  （SSE 不支持客户端到服务器的请求）");
            }
            Err(e) => {
                println!("  ❌ 错误: {e}");
            }
        }
    } else {
        println!("  ⚠️  未连接到服务器，跳过请求");
    }

    // 5. 断开连接
    println!("\n5️⃣ 断开连接:");
    match transport.disconnect().await {
        Ok(_) => {
            println!("  ✅ 断开连接成功");
            println!(
                "  连接状态: {}",
                if transport.is_connected() {
                    "已连接"
                } else {
                    "未连接"
                }
            );
        }
        Err(e) => {
            error!("  ❌ 断开连接失败: {}", e);
        }
    }
}
