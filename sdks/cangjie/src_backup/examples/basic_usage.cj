/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 基础使用示例 - AgentMem 仓颉 SDK 的基本功能演示
 */

package agentmem.examples

import agentmem.api.*
import agentmem.core.*

/// 基础使用示例
main() {
    println("🚀 AgentMem 仓颉 SDK 基础使用示例")
    println("=" * 50)
    
    try {
        // 1. 创建客户端配置
        println("\n📋 1. 创建客户端配置")
        let config = ClientConfig("http://localhost:8080")
            .withApiKey("demo-api-key")
            .withTimeout(30)
            .withRetryCount(3)
            .withCache(true, 100)
            .withDebugMode(true)
            .withLogLevel(LogLevel.Info)
        
        println("✅ 配置创建成功")
        println("   服务器地址: ${config.serverUrl}")
        println("   缓存启用: ${config.enableCache}")
        println("   调试模式: ${config.debugMode}")
        
        // 2. 创建客户端
        println("\n🔌 2. 创建并初始化客户端")
        let client = AgentMemClient(config)
        
        let initResult = client.initialize()
        match (initResult) {
            case Ok(_) => {
                println("✅ 客户端初始化成功")
                println("   版本: ${client.getVersion()}")
                println("   连接状态: ${client.isConnected()}")
            }
            case Err(error) => {
                println("❌ 客户端初始化失败: ${error.getMessage()}")
                return
            }
        }
        
        // 3. 创建记忆
        println("\n💭 3. 创建和添加记忆")
        let memory1 = MemoryBuilder()
            .withAgentId("demo-agent")
            .withUserId("demo-user")
            .withContent("我喜欢喝咖啡，特别是在早晨工作的时候")
            .withMemoryType(MemoryType.Semantic)
            .withImportance(0.8)
            .withMetadata("category", "preference")
            .withMetadata("source", "conversation")
            .build()
        
        let addResult = client.addMemory(memory1)
        var memoryId = ""
        match (addResult) {
            case Ok(id) => {
                memoryId = id
                println("✅ 记忆添加成功")
                println("   记忆ID: ${id}")
                println("   内容: ${memory1.content}")
                println("   重要性: ${memory1.importance}")
            }
            case Err(error) => {
                println("❌ 记忆添加失败: ${error.getMessage()}")
                return
            }
        }
        
        // 4. 添加更多记忆
        println("\n📚 4. 添加更多记忆")
        let memories = [
            MemoryBuilder()
                .withAgentId("demo-agent")
                .withContent("今天天气很好，适合户外活动")
                .withMemoryType(MemoryType.Episodic)
                .withImportance(0.6)
                .build(),
            MemoryBuilder()
                .withAgentId("demo-agent")
                .withContent("学习仓颉编程语言的FFI功能")
                .withMemoryType(MemoryType.Procedural)
                .withImportance(0.9)
                .build(),
            MemoryBuilder()
                .withAgentId("demo-agent")
                .withContent("明天有重要会议需要准备")
                .withMemoryType(MemoryType.Working)
                .withImportance(0.95)
                .build()
        ]
        
        for ((index, memory) in memories.enumerate()) {
            let result = client.addMemory(memory)
            match (result) {
                case Ok(id) => {
                    println("✅ 记忆 ${index + 1} 添加成功: ${id}")
                }
                case Err(error) => {
                    println("❌ 记忆 ${index + 1} 添加失败: ${error.getMessage()}")
                }
            }
        }
        
        // 5. 搜索记忆
        println("\n🔍 5. 搜索记忆")
        let searchQueries = ["咖啡", "学习", "会议", "天气"]
        
        for (query in searchQueries) {
            let searchResult = client.searchMemories(query, 3)
            match (searchResult) {
                case Ok(results) => {
                    println("🔍 搜索 '${query}' 找到 ${results.size} 条结果:")
                    for ((index, result) in results.enumerate()) {
                        println("   ${index + 1}. [${result.score:.2f}] ${result.memory.content}")
                    }
                }
                case Err(error) => {
                    println("❌ 搜索 '${query}' 失败: ${error.getMessage()}")
                }
            }
        }
        
        // 6. 获取记忆
        println("\n📖 6. 获取特定记忆")
        let getResult = client.getMemory(memoryId)
        match (getResult) {
            case Ok(memoryOpt) => {
                if (memoryOpt.isSome()) {
                    let memory = memoryOpt.getOrThrow()
                    println("✅ 记忆获取成功:")
                    println("   ID: ${memory.id}")
                    println("   内容: ${memory.content}")
                    println("   类型: ${memory.memoryType.toString()}")
                    println("   重要性: ${memory.importance}")
                    println("   创建时间: ${memory.createdAt}")
                    println("   访问次数: ${memory.accessCount}")
                } else {
                    println("⚠️ 记忆不存在")
                }
            }
            case Err(error) => {
                println("❌ 记忆获取失败: ${error.getMessage()}")
            }
        }
        
        // 7. 更新记忆
        println("\n✏️ 7. 更新记忆内容")
        let updateResult = client.updateMemory(memoryId, "我非常喜欢喝咖啡，特别是在早晨工作的时候，它能让我保持专注")
        match (updateResult) {
            case Ok(_) => {
                println("✅ 记忆更新成功")
                
                // 验证更新
                let verifyResult = client.getMemory(memoryId)
                match (verifyResult) {
                    case Ok(memoryOpt) => {
                        if (memoryOpt.isSome()) {
                            let updatedMemory = memoryOpt.getOrThrow()
                            println("   更新后内容: ${updatedMemory.content}")
                        }
                    }
                    case Err(_) => {}
                }
            }
            case Err(error) => {
                println("❌ 记忆更新失败: ${error.getMessage()}")
            }
        }
        
        // 8. 高级搜索
        println("\n🔍 8. 高级搜索功能")
        
        // 按重要性搜索
        var filter = SearchFilter()
        filter.importanceRange = Some((0.8, 1.0))
        filter.agentIds = Some(["demo-agent"])
        
        let filteredSearchResult = client.searchMemoriesFiltered("*", filter, 5)
        match (filteredSearchResult) {
            case Ok(results) => {
                println("🎯 高重要性记忆 (>= 0.8) 找到 ${results.size} 条:")
                for ((index, result) in results.enumerate()) {
                    println("   ${index + 1}. [${result.memory.importance:.2f}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ 高级搜索失败: ${error.getMessage()}")
            }
        }
        
        // 9. 获取统计信息
        println("\n📊 9. 获取记忆统计信息")
        let statsResult = client.getMemoryStats("demo-agent")
        match (statsResult) {
            case Ok(stats) => {
                println("✅ 统计信息获取成功:")
                println("   总记忆数量: ${stats.totalMemories}")
                println("   平均重要性: ${stats.averageImportance:.2f}")
                println("   最高重要性: ${stats.maxImportance:.2f}")
                println("   最低重要性: ${stats.minImportance:.2f}")
                
                println("   按类型分布:")
                for ((memoryType, count) in stats.memoryTypeCount) {
                    println("     ${memoryType.toString()}: ${count}")
                }
            }
            case Err(error) => {
                println("❌ 统计信息获取失败: ${error.getMessage()}")
            }
        }
        
        // 10. 健康检查
        println("\n🏥 10. 系统健康检查")
        let healthResult = client.healthCheck()
        match (healthResult) {
            case Ok(isHealthy) => {
                println("✅ 系统健康检查: ${if (isHealthy) { "健康" } else { "异常" }}")
            }
            case Err(error) => {
                println("❌ 健康检查失败: ${error.getMessage()}")
            }
        }
        
        // 11. 清理演示数据（可选）
        println("\n🧹 11. 清理演示数据")
        println("是否要删除演示记忆？(y/n)")
        // 这里简化处理，直接删除第一个记忆作为演示
        let deleteResult = client.deleteMemory(memoryId)
        match (deleteResult) {
            case Ok(_) => {
                println("✅ 演示记忆删除成功")
            }
            case Err(error) => {
                println("❌ 记忆删除失败: ${error.getMessage()}")
            }
        }
        
        // 12. 关闭客户端
        println("\n🔌 12. 关闭客户端连接")
        client.close()
        println("✅ 客户端已关闭")
        
        println("\n🎉 基础使用示例完成！")
        println("=" * 50)
        
    } catch (e: Exception) {
        println("💥 示例执行出错: ${e}")
    }
}
