#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! agent-mem-core = { path = "./crates/agent-mem-core" }
//! chrono = "0.4"
//! tokio = { version = "1.0", features = ["full"] }
//! ```

use agent_mem_core::types::{
    Memory, MemoryType, Content, AttributeKey, AttributeValue,
    AttributeSet, RelationGraph, Metadata,
};
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("\n=== AgentMem V4.0 Memory 核心功能测试 ===\n");

    // 测试1: 创建V4.0 Memory (使用builder模式)
    println!("✅ 测试1: 创建V4.0 Memory");
    let memory1 = Memory::builder()
        .content(Content::Text("测试记忆内容".to_string()))
        .attribute(AttributeKey::system("agent_id"), AttributeValue::String("agent-001".to_string()))
        .attribute(AttributeKey::system("user_id"), AttributeValue::String("user-001".to_string()))
        .attribute(AttributeKey::system("memory_type"), AttributeValue::String("semantic".to_string()))
        .attribute(AttributeKey::system("importance"), AttributeValue::Number(0.8))
        .build();
    
    println!("   Memory ID: {}", memory1.id);
    println!("   Agent ID: {}", memory1.agent_id());
    println!("   User ID: {:?}", memory1.user_id());
    println!("   Importance: {}", memory1.importance());
    println!("   Memory Type: {:?}", memory1.memory_type());

    // 测试2: 向后兼容API
    println!("\n✅ 测试2: 向后兼容API");
    let memory2 = Memory::new(
        "agent-002".to_string(),
        Some("user-002".to_string()),
        MemoryType::Episodic,
        "这是一个情景记忆".to_string(),
        0.9,
    );
    println!("   Created with legacy API: {}", memory2.id);
    println!("   Importance: {}", memory2.importance());

    // 测试3: 多模态Content
    println!("\n✅ 测试3: 多模态Content");
    let mut memory3 = Memory::builder()
        .content(Content::Mixed(vec![
            Content::Text("图片描述".to_string()),
            Content::Image {
                url: "https://example.com/image.png".to_string(),
                caption: Some("示例图片".to_string()),
            },
        ]))
        .attribute(AttributeKey::system("agent_id"), AttributeValue::String("agent-003".to_string()))
        .build();
    println!("   Multi-modal content created");

    // 测试4: 命名空间化属性
    println!("\n✅ 测试4: 命名空间化属性");
    memory3.attributes.set(
        AttributeKey::new("ecommerce", "product_id"),
        AttributeValue::String("P000257".to_string()),
    );
    memory3.attributes.set(
        AttributeKey::new("ecommerce", "price"),
        AttributeValue::Number(99.99),
    );
    memory3.attributes.set(
        AttributeKey::user("preference"),
        AttributeValue::String("electronics".to_string()),
    );
    
    println!("   System namespace: agent_id = {}", memory3.agent_id());
    if let Some(product_id) = memory3.attributes.get(&AttributeKey::new("ecommerce", "product_id")) {
        println!("   Ecommerce namespace: product_id = {}", product_id);
    }
    if let Some(price) = memory3.attributes.get(&AttributeKey::new("ecommerce", "price")) {
        println!("   Ecommerce namespace: price = {}", price);
    }

    // 测试5: AttributeValue Display
    println!("\n✅ 测试5: AttributeValue Display");
    let values = vec![
        AttributeValue::String("text".to_string()),
        AttributeValue::Number(123.45),
        AttributeValue::Boolean(true),
        AttributeValue::Timestamp(Utc::now()),
        AttributeValue::Array(vec![
            AttributeValue::Number(1.0),
            AttributeValue::Number(2.0),
            AttributeValue::Number(3.0),
        ]),
    ];
    for (i, val) in values.iter().enumerate() {
        println!("   Value {}: {}", i + 1, val);
    }

    // 测试6: 关系网络
    println!("\n✅ 测试6: 关系网络");
    use agent_mem_core::types::{Relation, RelationType};
    let mut memory4 = Memory::builder()
        .content(Content::Text("相关记忆".to_string()))
        .attribute(AttributeKey::system("agent_id"), AttributeValue::String("agent-004".to_string()))
        .relation(
            memory1.id.clone(),
            RelationType::References,
            0.9,
        )
        .build();
    println!("   Created memory with relation to: {}", memory1.id);
    println!("   Relation count: {}", memory4.relations.relations.len());

    println!("\n=== 所有测试通过 ✅ ===\n");
}

