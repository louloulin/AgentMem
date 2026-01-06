/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * AgentMem 仓颉 SDK 使用示例
 * 
 * 这个示例展示了如何使用AgentMem静态库进行智能记忆管理。
 */

package agentmem_example

import agentmem.*

main() {
    println("🚀 AgentMem 仓颉 SDK 使用示例")
    println("=" * 50)
    
    // 示例1：基本记忆操作
    basicMemoryExample()
    
    // 示例2：客户端使用
    clientExample()
    
    // 示例3：搜索功能
    searchExample()
    
    // 示例4：错误处理
    errorHandlingExample()
    
    println("\n🎉 所有示例运行完成！")
}

/// 基本记忆操作示例
func basicMemoryExample(): Unit {
    println("\n📚 基本记忆操作示例")
    println("-" * 30)
    
    // 创建记忆
    let memory = Memory(
        "example-001",
        "agent-demo",
        None,
        MemoryType.Semantic,
        "这是一个使用AgentMem库的示例记忆",
        ImportanceLevel.Medium.toFloat32(),
        None,
        1640995200000,
        1640995200000,
        0,
        None,
        SimpleMap(),
        1
    )
    
    println("✓ 记忆创建成功")
    println("  ID: ${memory.id}")
    println("  内容: ${memory.content}")
    println("  类型: ${memory.memoryType.toString()}")
    println("  重要性: ${memory.importance}")
    
    // 验证记忆
    let validateResult = memory.validate()
    match (validateResult) {
        case Ok(isValid) =>
            if (isValid) {
                println("✓ 记忆验证通过")
            } else {
                println("❌ 记忆验证失败")
            }
        case Err(error) => println("❌ 记忆验证错误: ${error.getMessage()}")
    }
}

/// 客户端使用示例
func clientExample(): Unit {
    println("\n🔌 客户端使用示例")
    println("-" * 30)
    
    // 创建客户端配置
    let config = ClientConfig("http://localhost:8080", "demo-agent")

    // 构建客户端
    let clientResult = AgentMemClientBuilder().build()
    
    match (clientResult) {
        case Ok(client) =>
            println("✓ 客户端创建成功")
            println("  服务器: ${config.serverUrl}")
            println("  超时: ${config.timeout}ms")

            // 演示添加记忆
            let memory = Memory(
                "client-example-001",
                "demo-agent",
                None,
                MemoryType.Episodic,
                "通过客户端添加的记忆",
                ImportanceLevel.High.toFloat32(),
                None,
                1640995200000,
                1640995200000,
                0,
                None,
                SimpleMap(),
                1
            )

            let addResult = client.addMemory(memory)
            match (addResult) {
                case Ok(id) => println("✓ 记忆添加成功: ${id}")
                case Err(error) => println("⚠ 记忆添加失败: ${error.getMessage()}")
            }
        case Err(error) =>
            println("❌ 客户端创建失败: ${error.getMessage()}")
    }
}

/// 搜索功能示例
func searchExample(): Unit {
    println("\n🔍 搜索功能示例")
    println("-" * 30)
    
    // 创建搜索服务
    let searchService = AgentMemSearchService("demo-agent")
    
    // 执行文本搜索
    let searchResult = searchService.searchByText("示例", None, None)
    match (searchResult) {
        case Ok(results) =>
            println("✓ 搜索完成，找到 ${results.totalCount} 个结果")
            for (result in results.results) {
                println("  - ${result.memory.content} (分数: ${result.score})")
            }
        case Err(error) =>
            println("❌ 搜索失败: ${error.getMessage()}")
    }
}

/// 错误处理示例
func errorHandlingExample(): Unit {
    println("\n❌ 错误处理示例")
    println("-" * 30)
    
    // 演示不同类型的错误
    let errors = Array<AgentMemError>(4, { i =>
        match (i) {
            case 0 => AgentMemError.NetworkError("网络连接超时")
            case 1 => AgentMemError.InvalidParameter("参数不能为空")
            case 2 => AgentMemError.NotFound("记忆不存在")
            case _ => AgentMemError.InternalError("内部服务错误")
        }
    })
    
    for (error in errors) {
        println("错误类型: ${error.getMessage()}")
        println("错误代码: ${error.getErrorCode()}")
        println("可重试: ${if (error.isRetryable()) { "是" } else { "否" }}")
        println()
    }
}
