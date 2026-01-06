/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 批量操作示例 - AgentMem 仓颉 SDK 的批量操作功能演示
 */

package agentmem.examples

import agentmem.api.*
import agentmem.core.*

/// 批量操作示例
main() {
    println("📦 AgentMem 仓颉 SDK 批量操作示例")
    println("=" * 50)
    
    try {
        // 1. 初始化客户端
        println("\n🔌 1. 初始化客户端")
        let config = ClientConfig("http://localhost:8080")
            .withApiKey("demo-api-key")
            .withTimeout(60)  // 批量操作需要更长超时时间
            .withRetryCount(3)
            .withCache(true, 500)
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
        
        // 2. 准备大量测试数据
        println("\n📚 2. 准备批量测试数据")
        let batchSize = 50
        var batchMemories = Array<Memory>()
        
        let topics = ["AI", "ML", "DL", "编程", "算法", "数据结构", "软件工程", "系统设计"]
        let domains = ["technology", "science", "education", "research"]
        let memoryTypes = [MemoryType.Semantic, MemoryType.Episodic, MemoryType.Procedural, MemoryType.Working]
        
        for (i in 0..batchSize) {
            let topic = topics[i % topics.size]
            let domain = domains[i % domains.size]
            let memoryType = memoryTypes[i % memoryTypes.size]
            let importance = 0.5 + (Float32(i % 50) / 100.0)  // 0.5 到 1.0
            
            let memory = MemoryBuilder()
                .withAgentId("batch-demo")
                .withUserId("batch-user")
                .withContent("这是关于${topic}的第${i + 1}条记忆内容，包含了详细的技术信息和实践经验")
                .withMemoryType(memoryType)
                .withImportance(importance)
                .withMetadata("topic", topic)
                .withMetadata("domain", domain)
                .withMetadata("batch_id", "batch-001")
                .withMetadata("index", "${i}")
                .build()
            
            batchMemories.append(memory)
        }
        
        println("✅ 准备了 ${batchMemories.size} 条测试记忆")
        
        // 3. 批量添加记忆
        println("\n📥 3. 批量添加记忆")
        let startTime = getCurrentTimestamp()
        
        let batchAddResult = client.addMemoriesBatch(batchMemories)
        match (batchAddResult) {
            case Ok(batchResult) => {
                let endTime = getCurrentTimestamp()
                let duration = endTime - startTime
                
                println("✅ 批量添加完成:")
                println("   总数量: ${batchResult.total}")
                println("   成功数量: ${batchResult.successes.size}")
                println("   失败数量: ${batchResult.failures.size}")
                println("   耗时: ${duration}ms")
                println("   平均速度: ${Float32(batchResult.successes.size) / Float32(duration) * 1000.0:.2f} 记忆/秒")
                
                if (!batchResult.failures.isEmpty()) {
                    println("   失败原因:")
                    for ((index, error) in batchResult.failures.enumerate()) {
                        if (index < 5) {  // 只显示前5个错误
                            println("     ${index + 1}. ${error}")
                        }
                    }
                    if (batchResult.failures.size > 5) {
                        println("     ... 还有 ${batchResult.failures.size - 5} 个错误")
                    }
                }
            }
            case Err(error) => {
                println("❌ 批量添加失败: ${error.getMessage()}")
                return
            }
        }
        
        // 4. 分页获取记忆
        println("\n📄 4. 分页获取记忆")
        let pageSize: UInt32 = 10
        var currentPage: UInt32 = 1
        var totalRetrieved = 0
        
        while (true) {
            let pagination = PaginationParams(currentPage, pageSize)
            let pageResult = client.getMemoriesPaginated("batch-demo", pagination)
            
            match (pageResult) {
                case Ok(paginatedResult) => {
                    println("📄 第 ${currentPage} 页 (${paginatedResult.data.size} 条):")
                    
                    for ((index, memory) in paginatedResult.data.enumerate()) {
                        let globalIndex = (currentPage - 1) * pageSize + UInt32(index) + 1
                        println("   ${globalIndex}. [${memory.importance:.2f}] ${memory.content[0..50]}...")
                    }
                    
                    totalRetrieved = totalRetrieved + paginatedResult.data.size
                    
                    println("   页面信息: ${currentPage}/${(paginatedResult.totalCount + pageSize - 1) / pageSize}")
                    println("   总计: ${paginatedResult.totalCount} 条记忆")
                    
                    // 检查是否还有更多页面
                    if (currentPage * pageSize >= paginatedResult.totalCount) {
                        break
                    }
                    
                    currentPage = currentPage + 1
                    
                    // 限制演示只显示前3页
                    if (currentPage > 3) {
                        println("   ... (为了演示，只显示前3页)")
                        break
                    }
                }
                case Err(error) => {
                    println("❌ 分页获取失败: ${error.getMessage()}")
                    break
                }
            }
        }
        
        println("📊 分页获取总结: 已检索 ${totalRetrieved} 条记忆")
        
        // 5. 按类型分页获取
        println("\n🏷️ 5. 按类型分页获取记忆")
        for (memoryType in memoryTypes) {
            let typePagination = PaginationParams(1, 5)
            let typePageResult = client.getMemoriesByTypePaginated("batch-demo", memoryType, typePagination)
            
            match (typePageResult) {
                case Ok(result) => {
                    println("🏷️ ${memoryType.toString()} 类型记忆 (${result.data.size} 条):")
                    for ((index, memory) in result.data.enumerate()) {
                        println("   ${index + 1}. ${memory.content[0..40]}...")
                    }
                }
                case Err(error) => {
                    println("❌ 按类型获取失败: ${error.getMessage()}")
                }
            }
        }
        
        // 6. 批量搜索
        println("\n🔍 6. 批量搜索测试")
        let searchQueries = ["AI", "编程", "算法", "数据", "系统"]
        var allSearchResults = Array<Array<MemorySearchResult>>()
        
        let batchSearchStart = getCurrentTimestamp()
        for (query in searchQueries) {
            let searchResult = client.searchMemories(query, 10)
            match (searchResult) {
                case Ok(results) => {
                    allSearchResults.append(results)
                    println("🔍 '${query}' 搜索结果: ${results.size} 条")
                }
                case Err(error) => {
                    println("❌ '${query}' 搜索失败: ${error.getMessage()}")
                    allSearchResults.append(Array<MemorySearchResult>())
                }
            }
        }
        let batchSearchEnd = getCurrentTimestamp()
        
        let totalSearchResults = allSearchResults.fold(0, { acc, results => acc + results.size })
        println("📊 批量搜索总结:")
        println("   查询数量: ${searchQueries.size}")
        println("   总结果数: ${totalSearchResults}")
        println("   总耗时: ${batchSearchEnd - batchSearchStart}ms")
        
        // 7. 获取统计信息
        println("\n📊 7. 获取详细统计信息")
        let statsResult = client.getMemoryStats("batch-demo")
        match (statsResult) {
            case Ok(stats) => {
                println("📊 Agent 统计信息:")
                println("   总记忆数量: ${stats.totalMemories}")
                println("   平均重要性: ${stats.averageImportance:.3f}")
                println("   重要性范围: ${stats.minImportance:.3f} - ${stats.maxImportance:.3f}")
                
                println("   按类型分布:")
                var totalByType: UInt32 = 0
                for ((memoryType, count) in stats.memoryTypeCount) {
                    let percentage = Float32(count) / Float32(stats.totalMemories) * 100.0
                    println("     ${memoryType.toString()}: ${count} (${percentage:.1f}%)")
                    totalByType = totalByType + count
                }
                println("   验证总计: ${totalByType}")
            }
            case Err(error) => {
                println("❌ 统计信息获取失败: ${error.getMessage()}")
            }
        }
        
        // 8. 批量更新测试（更新前10条记忆）
        println("\n✏️ 8. 批量更新测试")
        let updatePagination = PaginationParams(1, 10)
        let updatePageResult = client.getMemoriesPaginated("batch-demo", updatePagination)
        
        match (updatePageResult) {
            case Ok(result) => {
                var updateCount = 0
                for ((index, memory) in result.data.enumerate()) {
                    let newContent = "${memory.content} [已更新]"
                    let updateResult = client.updateMemory(memory.id, newContent)
                    match (updateResult) {
                        case Ok(_) => {
                            updateCount = updateCount + 1
                            println("✅ 记忆 ${index + 1} 更新成功")
                        }
                        case Err(error) => {
                            println("❌ 记忆 ${index + 1} 更新失败: ${error.getMessage()}")
                        }
                    }
                }
                println("📊 批量更新完成: ${updateCount}/${result.data.size} 条记忆更新成功")
            }
            case Err(error) => {
                println("❌ 获取待更新记忆失败: ${error.getMessage()}")
            }
        }
        
        // 9. 数据导出测试
        println("\n📤 9. 数据导出测试")
        let exportPath = "/tmp/agentmem_batch_export.json"
        let exportResult = client.exportMemories("batch-demo", "json", exportPath)
        match (exportResult) {
            case Ok(_) => {
                println("✅ 数据导出成功")
                println("   导出路径: ${exportPath}")
                println("   导出格式: JSON")
            }
            case Err(error) => {
                println("❌ 数据导出失败: ${error.getMessage()}")
            }
        }
        
        // 10. 性能压力测试
        println("\n⚡ 10. 性能压力测试")
        let stressTestQueries = Array<String>()
        for (i in 0..20) {
            stressTestQueries.append("测试查询${i}")
        }
        
        let stressTestStart = getCurrentTimestamp()
        var successCount = 0
        var failCount = 0
        
        for (query in stressTestQueries) {
            let result = client.searchMemories(query, 5)
            match (result) {
                case Ok(_) => successCount = successCount + 1
                case Err(_) => failCount = failCount + 1
            }
        }
        
        let stressTestEnd = getCurrentTimestamp()
        let stressDuration = stressTestEnd - stressTestStart
        
        println("⚡ 压力测试结果:")
        println("   查询数量: ${stressTestQueries.size}")
        println("   成功数量: ${successCount}")
        println("   失败数量: ${failCount}")
        println("   总耗时: ${stressDuration}ms")
        println("   平均响应时间: ${stressDuration / Int64(stressTestQueries.size)}ms")
        println("   QPS: ${Float32(stressTestQueries.size) / Float32(stressDuration) * 1000.0:.2f}")
        
        // 11. 批量删除测试
        println("\n🗑️ 11. 批量删除测试")
        
        // 获取要删除的记忆ID列表
        let deletePagination = PaginationParams(1, 20)
        let deletePageResult = client.getMemoriesPaginated("batch-demo", deletePagination)
        
        match (deletePageResult) {
            case Ok(result) => {
                var memoryIdsToDelete = Array<String>()
                for (memory in result.data) {
                    memoryIdsToDelete.append(memory.id)
                }
                
                if (!memoryIdsToDelete.isEmpty()) {
                    let batchDeleteResult = client.deleteMemoriesBatch(memoryIdsToDelete)
                    match (batchDeleteResult) {
                        case Ok(batchResult) => {
                            println("✅ 批量删除完成:")
                            println("   总数量: ${batchResult.total}")
                            println("   成功数量: ${batchResult.successes.size}")
                            println("   失败数量: ${batchResult.failures.size}")
                            
                            if (!batchResult.failures.isEmpty()) {
                                println("   失败原因:")
                                for ((index, error) in batchResult.failures.enumerate()) {
                                    if (index < 3) {
                                        println("     ${index + 1}. ${error}")
                                    }
                                }
                            }
                        }
                        case Err(error) => {
                            println("❌ 批量删除失败: ${error.getMessage()}")
                        }
                    }
                } else {
                    println("⚠️ 没有找到要删除的记忆")
                }
            }
            case Err(error) => {
                println("❌ 获取待删除记忆失败: ${error.getMessage()}")
            }
        }
        
        // 12. 最终统计
        println("\n📊 12. 最终统计信息")
        let finalStatsResult = client.getMemoryStats("batch-demo")
        match (finalStatsResult) {
            case Ok(finalStats) => {
                println("📊 最终统计:")
                println("   剩余记忆数量: ${finalStats.totalMemories}")
                println("   平均重要性: ${finalStats.averageImportance:.3f}")
            }
            case Err(error) => {
                println("❌ 最终统计获取失败: ${error.getMessage()}")
            }
        }
        
        // 13. 清理剩余数据
        println("\n🧹 13. 清理剩余数据")
        let cleanupPagination = PaginationParams(1, 100)
        let cleanupPageResult = client.getMemoriesPaginated("batch-demo", cleanupPagination)
        
        match (cleanupPageResult) {
            case Ok(result) => {
                if (!result.data.isEmpty()) {
                    var cleanupIds = Array<String>()
                    for (memory in result.data) {
                        cleanupIds.append(memory.id)
                    }
                    
                    let cleanupResult = client.deleteMemoriesBatch(cleanupIds)
                    match (cleanupResult) {
                        case Ok(batchResult) => {
                            println("✅ 清理完成: 删除了 ${batchResult.successes.size} 条记忆")
                        }
                        case Err(error) => {
                            println("❌ 清理失败: ${error.getMessage()}")
                        }
                    }
                } else {
                    println("✅ 没有需要清理的数据")
                }
            }
            case Err(error) => {
                println("❌ 获取清理数据失败: ${error.getMessage()}")
            }
        }
        
        // 14. 关闭客户端
        println("\n🔌 14. 关闭客户端")
        client.close()
        println("✅ 客户端已关闭")
        
        println("\n🎉 批量操作示例完成！")
        println("=" * 50)
        
    } catch (e: Exception) {
        println("💥 示例执行出错: ${e}")
    }
}
