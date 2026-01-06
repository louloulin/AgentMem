//! Query抽象演示

use agent_mem_core::{Query, QueryIntent, Constraint, AttributeKey, AttributeValue};

fn main() {
    println!("=== AgentMem Query抽象演示 ===\n");

    // 示例1: 从字符串创建Query
    println!("1. 从字符串创建Query:");
    let query1 = Query::from_string("hello world");
    println!("   Query: {:?}", query1.intent);
    println!();

    // 示例2: ID查找
    println!("2. ID查找:");
    let query2 = Query::from_string("find U123456");
    println!("   Query: {:?}", query2.intent);
    println!();

    // 示例3: 属性过滤
    println!("3. 属性过滤:");
    let query3 = Query::from_string("search user::name=john");
    println!("   Intent: {:?}", query3.intent);
    println!("   Constraints: {} items", query3.constraints.len());
    println!();

    // 示例4: 使用Builder
    println!("4. 使用Builder构建Query:");
    let query4 = Query::builder()
        .intent(QueryIntent::SemanticSearch {
            text: "important memories".to_string(),
            semantic_vector: None,
        })
        .constraint(Constraint::AttributeMatch {
            key: AttributeKey::system("importance"),
            value: AttributeValue::Number(0.8),
        })
        .constraint(Constraint::Limit(10))
        .user_id("user1")
        .agent_id("agent1")
        .build();
    println!("   User ID: {:?}", query4.context.user_id);
    println!("   Constraints: {} items", query4.constraints.len());
    println!();

    println!("=== 演示完成 ===");
}

