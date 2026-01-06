/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 高级搜索示例 - AgentMem 仓颉 SDK 的高级搜索功能演示
 */

package agentmem.examples

import agentmem.api.*
import agentmem.core.*

/// 高级搜索示例
main() {
    println("🔍 AgentMem 仓颉 SDK 高级搜索示例")
    println("=" * 50)
    
    try {
        // 1. 初始化客户端
        println("\n🔌 1. 初始化客户端")
        let config = ClientConfig("http://localhost:8080")
            .withApiKey("demo-api-key")
            .withCache(true, 200)
            .withDebugMode(true)
        
        let client = AgentMemClient(config)
        let initResult = client.initialize()
        match (initResult) {
            case Ok(_) => println("✅ 客户端初始化成功")
            case Err(error) => {
                println("❌ 客户端初始化失败: ${error.getMessage()}")
                return
            }
        }
        
        // 2. 创建搜索管理器
        println("\n🎯 2. 创建搜索管理器")
        var searchConfig = SearchConfig()
        searchConfig.maxResults = 20
        searchConfig.similarityThreshold = 0.6
        searchConfig.enableSemanticSearch = true
        searchConfig.enableFullTextSearch = true
        
        let searchManager = SearchManager(client, searchConfig)
        println("✅ 搜索管理器创建成功")
        
        // 3. 准备测试数据
        println("\n📚 3. 准备测试数据")
        let testMemories = [
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("人工智能是计算机科学的一个分支，致力于创建能够执行通常需要人类智能的任务的系统")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.9)
                .withMetadata("topic", "AI")
                .withMetadata("domain", "technology")
                .build(),
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("机器学习是人工智能的一个子集，使计算机能够在没有明确编程的情况下学习")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.85)
                .withMetadata("topic", "ML")
                .withMetadata("domain", "technology")
                .build(),
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("深度学习使用神经网络来模拟人脑的工作方式")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.8)
                .withMetadata("topic", "DL")
                .withMetadata("domain", "technology")
                .build(),
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("今天我学习了仓颉编程语言的FFI功能，感觉很有趣")
                .withMemoryType(MemoryType.Episodic)
                .withImportance(0.7)
                .withMetadata("topic", "learning")
                .withMetadata("domain", "programming")
                .build(),
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("明天需要完成AgentMem SDK的文档编写工作")
                .withMemoryType(MemoryType.Working)
                .withImportance(0.95)
                .withMetadata("topic", "work")
                .withMetadata("domain", "development")
                .build(),
            MemoryBuilder()
                .withAgentId("search-demo")
                .withContent("咖啡是我最喜欢的饮品，特别是在编程的时候")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.6)
                .withMetadata("topic", "preference")
                .withMetadata("domain", "lifestyle")
                .build()
        ]
        
        var memoryIds = Array<String>()
        for ((index, memory) in testMemories.enumerate()) {
            let result = client.addMemory(memory)
            match (result) {
                case Ok(id) => {
                    memoryIds.append(id)
                    println("✅ 测试记忆 ${index + 1} 添加成功")
                }
                case Err(error) => {
                    println("❌ 测试记忆 ${index + 1} 添加失败: ${error.getMessage()}")
                }
            }
        }
        
        // 4. 基础搜索
        println("\n🔍 4. 基础搜索测试")
        let basicQueries = ["人工智能", "学习", "编程", "咖啡"]
        
        for (query in basicQueries) {
            let result = searchManager.smartSearch(query, None, Some(3))
            match (result) {
                case Ok(results) => {
                    println("🔍 搜索 '${query}' 找到 ${results.size} 条结果:")
                    for ((index, searchResult) in results.enumerate()) {
                        println("   ${index + 1}. [${searchResult.score:.3f}] ${searchResult.memory.content}")
                    }
                }
                case Err(error) => {
                    println("❌ 搜索 '${query}' 失败: ${error.getMessage()}")
                }
            }
            println()
        }
        
        // 5. 过滤搜索
        println("\n🎯 5. 过滤搜索测试")
        
        // 按记忆类型过滤
        println("📋 按记忆类型过滤 (Semantic):")
        var typeFilter = SearchFilter()
        typeFilter.memoryTypes = Some([MemoryType.Semantic])
        typeFilter.agentIds = Some(["search-demo"])
        
        let typeSearchResult = searchManager.smartSearch("*", Some(typeFilter), Some(5))
        match (typeSearchResult) {
            case Ok(results) => {
                for ((index, result) in results.enumerate()) {
                    println("   ${index + 1}. [${result.memory.memoryType.toString()}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ 类型过滤搜索失败: ${error.getMessage()}")
            }
        }
        
        // 按重要性过滤
        println("\n⭐ 按重要性过滤 (>= 0.8):")
        var importanceFilter = SearchFilter()
        importanceFilter.importanceRange = Some((0.8, 1.0))
        importanceFilter.agentIds = Some(["search-demo"])
        
        let importanceSearchResult = searchManager.smartSearch("*", Some(importanceFilter), Some(5))
        match (importanceSearchResult) {
            case Ok(results) => {
                for ((index, result) in results.enumerate()) {
                    println("   ${index + 1}. [${result.memory.importance:.2f}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ 重要性过滤搜索失败: ${error.getMessage()}")
            }
        }
        
        // 6. 多条件搜索
        println("\n🔗 6. 多条件搜索测试")
        let multiQueries = ["人工智能", "学习", "编程"]
        
        // AND 搜索
        println("🔗 AND 搜索 (交集):")
        let andResult = searchManager.advancedSearch(multiQueries, SearchOperator.And, None, 5)
        match (andResult) {
            case Ok(results) => {
                println("   找到 ${results.size} 条同时包含所有关键词的记忆:")
                for ((index, result) in results.enumerate()) {
                    println("   ${index + 1}. [${result.score:.3f}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ AND 搜索失败: ${error.getMessage()}")
            }
        }
        
        // OR 搜索
        println("\n🔗 OR 搜索 (并集):")
        let orResult = searchManager.advancedSearch(multiQueries, SearchOperator.Or, None, 8)
        match (orResult) {
            case Ok(results) => {
                println("   找到 ${results.size} 条包含任一关键词的记忆:")
                for ((index, result) in results.enumerate()) {
                    println("   ${index + 1}. [${result.score:.3f}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ OR 搜索失败: ${error.getMessage()}")
            }
        }
        
        // 7. 时间范围搜索
        println("\n⏰ 7. 时间范围搜索测试")
        let currentTime = getCurrentTimestamp()
        let oneHourAgo = currentTime - 3600  // 1小时前
        
        let timeResult = searchManager.searchByTimeRange(oneHourAgo, currentTime, Some("search-demo"), 10)
        match (timeResult) {
            case Ok(results) => {
                println("🕐 最近1小时内的记忆 (${results.size} 条):")
                for ((index, result) in results.enumerate()) {
                    let ageMinutes = (currentTime - result.memory.createdAt) / 60
                    println("   ${index + 1}. [${ageMinutes}分钟前] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ 时间范围搜索失败: ${error.getMessage()}")
            }
        }
        
        // 8. 相似记忆搜索
        println("\n🔄 8. 相似记忆搜索测试")
        if (!memoryIds.isEmpty()) {
            let firstMemoryId = memoryIds[0]
            let similarResult = searchManager.findSimilarMemories(firstMemoryId, 0.5, 3)
            match (similarResult) {
                case Ok(results) => {
                    println("🔄 与第一条记忆相似的记忆 (${results.size} 条):")
                    for ((index, result) in results.enumerate()) {
                        println("   ${index + 1}. [${result.score:.3f}] ${result.memory.content}")
                    }
                }
                case Err(error) => {
                    println("❌ 相似记忆搜索失败: ${error.getMessage()}")
                }
            }
        }
        
        // 9. 搜索建议
        println("\n💡 9. 搜索建议测试")
        let partialQueries = ["人工", "学", "编"]
        
        for (partial in partialQueries) {
            let suggestionsResult = searchManager.getSearchSuggestions(partial, 3)
            match (suggestionsResult) {
                case Ok(suggestions) => {
                    println("💡 '${partial}' 的搜索建议:")
                    for ((index, suggestion) in suggestions.enumerate()) {
                        println("   ${index + 1}. ${suggestion}")
                    }
                }
                case Err(error) => {
                    println("❌ 搜索建议失败: ${error.getMessage()}")
                }
            }
        }
        
        // 10. 性能测试
        println("\n⚡ 10. 搜索性能测试")
        let performanceQueries = ["AI", "学习", "编程", "咖啡", "工作"]
        let startTime = getCurrentTimestamp()
        var totalResults = 0
        
        for (query in performanceQueries) {
            let result = searchManager.smartSearch(query, None, Some(5))
            match (result) {
                case Ok(results) => {
                    totalResults = totalResults + results.size
                }
                case Err(_) => {}
            }
        }
        
        let endTime = getCurrentTimestamp()
        let duration = endTime - startTime
        println("⚡ 性能测试结果:")
        println("   查询数量: ${performanceQueries.size}")
        println("   总结果数: ${totalResults}")
        println("   总耗时: ${duration}ms")
        println("   平均耗时: ${duration / Int64(performanceQueries.size)}ms/查询")
        
        // 11. 清理缓存
        println("\n🧹 11. 清理搜索缓存")
        searchManager.clearCache()
        println("✅ 搜索缓存已清理")
        
        // 12. 清理测试数据
        println("\n🗑️ 12. 清理测试数据")
        var deletedCount = 0
        for (memoryId in memoryIds) {
            let deleteResult = client.deleteMemory(memoryId)
            match (deleteResult) {
                case Ok(_) => deletedCount = deletedCount + 1
                case Err(_) => {}
            }
        }
        println("✅ 已删除 ${deletedCount} 条测试记忆")
        
        // 13. 关闭客户端
        println("\n🔌 13. 关闭客户端")
        client.close()
        println("✅ 客户端已关闭")
        
        println("\n🎉 高级搜索示例完成！")
        println("=" * 50)
        
    } catch (e: Exception) {
        println("💥 示例执行出错: ${e}")
    }
}
