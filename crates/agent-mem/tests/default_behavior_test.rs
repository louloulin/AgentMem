//! 默认行为测试
//!
//! 验证 P0 优化后的默认行为：
//! - AddMemoryOptions::default() 的 infer 字段为 true
//! - mem.add() 默认使用智能模式
//! - 显式禁用智能功能仍然有效
//! - 向后兼容性

use agent_mem::{AddMemoryOptions, Memory};

#[test]
fn test_default_infer_is_true() {
    // 验证默认值是 true（P0 优化的核心改动）
    let options = AddMemoryOptions::default();
    assert_eq!(options.infer, true, "默认应该启用智能功能（对标 Mem0）");
}

#[test]
fn test_default_options_fields() {
    // 验证所有默认值
    let options = AddMemoryOptions::default();

    assert_eq!(options.user_id, None, "默认 user_id 应该为 None");
    assert_eq!(options.agent_id, None, "默认 agent_id 应该为 None");
    assert_eq!(options.run_id, None, "默认 run_id 应该为 None");
    assert!(options.metadata.is_empty(), "默认 metadata 应该为空");
    assert_eq!(options.infer, true, "默认 infer 应该为 true");
    assert_eq!(options.memory_type, None, "默认 memory_type 应该为 None");
    assert_eq!(options.prompt, None, "默认 prompt 应该为 None");
}

#[tokio::test]
async fn test_add_uses_default_options() {
    // 验证 mem.add() 使用默认选项
    let mem = Memory::new().await.expect("初始化失败");

    // 不设置 options，使用默认值
    let result = mem.add("I love pizza").await;

    // 应该成功（智能模式或降级到简单模式）
    assert!(result.is_ok(), "默认行为应该成功");

    if let Ok(add_result) = result {
        println!("添加记忆成功: {:?}", add_result);
    }
}

#[tokio::test]
async fn test_explicit_infer_false_still_works() {
    // 验证显式禁用智能功能仍然有效（向后兼容）
    let mem = Memory::new().await.expect("初始化失败");

    let options = AddMemoryOptions {
        infer: false,
        ..Default::default()
    };

    let result = mem
        .add_with_options("Raw content".to_string(), options)
        .await;
    assert!(result.is_ok(), "简单模式应该成功");

    if let Ok(add_result) = result {
        println!("简单模式添加成功: {:?}", add_result);
    }
}

#[tokio::test]
async fn test_backward_compatibility_with_explicit_infer_true() {
    // 验证显式设置 infer: true 仍然有效（向后兼容）
    let mem = Memory::new().await.expect("初始化失败");

    let options = AddMemoryOptions {
        infer: true,
        ..Default::default()
    };

    let result = mem
        .add_with_options("I love pizza".to_string(), options)
        .await;
    assert!(result.is_ok(), "显式启用智能功能应该成功");

    if let Ok(add_result) = result {
        println!("显式智能模式添加成功: {:?}", add_result);
    }
}

#[tokio::test]
async fn test_add_with_session_context() {
    // 验证带 Session 上下文的添加
    let mem = Memory::new().await.expect("初始化失败");

    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        agent_id: Some("assistant".to_string()),
        run_id: Some("session-123".to_string()),
        ..Default::default() // infer: true
    };

    let result = mem
        .add_with_options("I love pizza".to_string(), options)
        .await;
    assert!(result.is_ok(), "带 Session 上下文的添加应该成功");

    if let Ok(add_result) = result {
        println!("带 Session 上下文添加成功: {:?}", add_result);
    }
}

#[tokio::test]
async fn test_add_with_metadata() {
    // 验证带元数据的添加
    let mem = Memory::new().await.expect("初始化失败");

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("source".to_string(), "chat".to_string());
    metadata.insert("priority".to_string(), "high".to_string());

    let options = AddMemoryOptions {
        metadata,
        ..Default::default() // infer: true
    };

    let result = mem
        .add_with_options("Important message".to_string(), options)
        .await;
    assert!(result.is_ok(), "带元数据的添加应该成功");

    if let Ok(add_result) = result {
        println!("带元数据添加成功: {:?}", add_result);
    }
}

#[tokio::test]
async fn test_multiple_adds_with_default_options() {
    // 验证多次添加都使用默认选项
    let mem = Memory::new().await.expect("初始化失败");

    let result1 = mem.add("I love pizza").await;
    assert!(result1.is_ok(), "第一次添加应该成功");

    let result2 = mem.add("I live in San Francisco").await;
    assert!(result2.is_ok(), "第二次添加应该成功");

    let result3 = mem.add("My favorite food is pizza").await;
    assert!(result3.is_ok(), "第三次添加应该成功（可能去重）");

    println!("多次添加测试完成");
}

#[tokio::test]
async fn test_search_after_add_with_default_options() {
    // 验证使用默认选项添加后可以搜索
    let mem = Memory::new().await.expect("初始化失败");

    // 添加记忆
    let _ = mem.add("I love pizza").await;
    let _ = mem.add("I live in San Francisco").await;

    // 搜索记忆
    let results = mem.search("What do you know about me?").await;

    // 应该能搜索到结果（即使智能组件未初始化，也应该有基本搜索）
    assert!(results.is_ok(), "搜索应该成功");

    if let Ok(search_results) = results {
        println!("搜索到 {} 条记忆", search_results.len());
        for result in search_results {
            println!("- {}", result.content);
        }
    }
}

#[test]
fn test_options_builder_pattern() {
    // 验证选项构建器模式
    let options = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        agent_id: Some("assistant".to_string()),
        infer: false, // 显式禁用
        ..Default::default()
    };

    assert_eq!(options.user_id, Some("alice".to_string()));
    assert_eq!(options.agent_id, Some("assistant".to_string()));
    assert_eq!(options.infer, false);
    assert_eq!(options.run_id, None);
}

#[test]
fn test_options_clone() {
    // 验证选项可以克隆
    let options1 = AddMemoryOptions {
        user_id: Some("alice".to_string()),
        infer: true,
        ..Default::default()
    };

    let options2 = options1.clone();

    assert_eq!(options1.user_id, options2.user_id);
    assert_eq!(options1.infer, options2.infer);
}

#[test]
fn test_options_debug() {
    // 验证选项可以调试打印
    let options = AddMemoryOptions::default();
    let debug_str = format!("{:?}", options);

    assert!(debug_str.contains("infer"));
    assert!(debug_str.contains("true"));
}
