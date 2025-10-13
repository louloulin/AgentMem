use agent_mem_traits::{AgentMemError, ErrorContext, ErrorSeverity};

fn main() {
    println!("=== AgentMem 错误处理测试 ===");
    
    // 测试 1: 创建错误
    let error = AgentMemError::memory_error("测试错误");
    println!("错误: {}", error);
    
    // 测试 2: 错误严重性
    let severity = error.severity();
    println!("严重性: {:?}", severity);
    
    // 测试 3: 错误上下文
    let context = ErrorContext::new("test_operation")
        .with_detail("key", "value");
    println!("上下文: {}", context.format());
    
    println!("测试完成！");
}

