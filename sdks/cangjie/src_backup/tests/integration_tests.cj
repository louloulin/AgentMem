/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 集成测试 - AgentMem 仓颉 SDK 的集成测试套件
 */

package agentmem.tests

import agentmem.api.*
import agentmem.core.*

/// 集成测试套件
public class IntegrationTestSuite {
    private var _client: Option<AgentMemClient>
    private var _testResults: Array<TestResult>
    private var _totalTests: Int32
    private var _passedTests: Int32

    public init() {
        this._client = None
        this._testResults = Array<TestResult>()
        this._totalTests = 0
        this._passedTests = 0
    }

    /// 设置测试环境
    public func setUp(): Bool {
        try {
            println("🔧 设置集成测试环境...")
            
            let config = ClientConfig("http://localhost:8080")
                .withApiKey("integration-test-key")
                .withTimeout(30)
                .withRetryCount(3)
                .withCache(true, 100)
                .withDebugMode(true)
            
            let client = AgentMemClient(config)
            let initResult = client.initialize()
            
            match (initResult) {
                case Ok(_) => {
                    this._client = Some(client)
                    println("✅ 测试环境设置成功")
                    return true
                }
                case Err(error) => {
                    println("❌ 测试环境设置失败: ${error.getMessage()}")
                    return false
                }
            }
            
        } catch (e: Exception) {
            println("❌ 测试环境设置异常: ${e}")
            return false
        }
    }

    /// 清理测试环境
    public func tearDown(): Unit {
        if (this._client.isSome()) {
            this._client.getOrThrow().close()
            this._client = None
            println("🧹 测试环境清理完成")
        }
    }

    /// 运行集成测试
    public func runTest(name: String, testFunc: (AgentMemClient) -> Bool): Unit {
        if (this._client.isNone()) {
            println("❌ ${name} - 客户端未初始化")
            return
        }

        let startTime = getCurrentTimestamp()
        var passed = false
        var message = ""

        try {
            passed = testFunc(this._client.getOrThrow())
            message = if (passed) { "PASS" } else { "FAIL" }
        } catch (e: Exception) {
            passed = false
            message = "ERROR: ${e}"
        }

        let endTime = getCurrentTimestamp()
        let duration = endTime - startTime

        let result = TestResult(name, passed, message, duration)
        this._testResults.append(result)
        this._totalTests = this._totalTests + 1
        
        if (passed) {
            this._passedTests = this._passedTests + 1
        }

        let status = if (passed) { "✅" } else { "❌" }
        println("${status} ${name} (${duration}ms) - ${message}")
    }

    /// 打印测试报告
    public func printReport(): Unit {
        println("\n" + "=" * 60)
        println("📊 集成测试报告")
        println("=" * 60)
        println("总测试数: ${this._totalTests}")
        println("通过数量: ${this._passedTests}")
        println("失败数量: ${this._totalTests - this._passedTests}")
        println("通过率: ${Float32(this._passedTests) / Float32(this._totalTests) * 100.0:.1f}%")
        
        let totalDuration = this._testResults.fold(Int64(0), { acc, result => acc + result.duration })
        println("总耗时: ${totalDuration}ms")
        
        if (this._passedTests < this._totalTests) {
            println("\n❌ 失败的测试:")
            for (result in this._testResults) {
                if (!result.passed) {
                    println("   ${result.name}: ${result.message}")
                }
            }
        }
        
        println("=" * 60)
    }

    /// 是否所有测试都通过
    public func allPassed(): Bool {
        return this._passedTests == this._totalTests
    }
}

/// 集成测试主函数
main() {
    println("🔗 AgentMem 仓颉 SDK 集成测试")
    println("=" * 50)
    
    let testSuite = IntegrationTestSuite()
    
    // 设置测试环境
    if (!testSuite.setUp()) {
        println("❌ 无法设置测试环境，跳过集成测试")
        return
    }
    
    var testMemoryIds = Array<String>()
    
    try {
        // 1. 客户端连接测试
        println("\n🔌 1. 客户端连接测试")
        
        testSuite.runTest("客户端初始化和连接测试") { client =>
            let isConnected = client.isConnected()
            let version = client.getVersion()
            
            return isConnected && !version.isEmpty()
        }
        
        testSuite.runTest("健康检查测试") { client =>
            let healthResult = client.healthCheck()
            match (healthResult) {
                case Ok(isHealthy) => isHealthy
                case Err(_) => false
            }
        }
        
        // 2. 基础CRUD操作测试
        println("\n📝 2. 基础CRUD操作测试")
        
        testSuite.runTest("添加记忆测试") { client =>
            let memory = MemoryBuilder()
                .withAgentId("integration-test")
                .withUserId("test-user")
                .withContent("这是一条集成测试记忆")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.8)
                .withMetadata("test", "integration")
                .build()
            
            let addResult = client.addMemory(memory)
            match (addResult) {
                case Ok(memoryId) => {
                    testMemoryIds.append(memoryId)
                    return true
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("获取记忆测试") { client =>
            if (testMemoryIds.isEmpty()) {
                return false
            }
            
            let memoryId = testMemoryIds[0]
            let getResult = client.getMemory(memoryId)
            match (getResult) {
                case Ok(memoryOpt) => {
                    if (memoryOpt.isSome()) {
                        let memory = memoryOpt.getOrThrow()
                        return memory.id == memoryId && 
                               memory.agentId == "integration-test" &&
                               memory.content == "这是一条集成测试记忆"
                    } else {
                        return false
                    }
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("更新记忆测试") { client =>
            if (testMemoryIds.isEmpty()) {
                return false
            }
            
            let memoryId = testMemoryIds[0]
            let newContent = "这是一条更新后的集成测试记忆"
            
            let updateResult = client.updateMemory(memoryId, newContent)
            match (updateResult) {
                case Ok(_) => {
                    // 验证更新
                    let getResult = client.getMemory(memoryId)
                    match (getResult) {
                        case Ok(memoryOpt) => {
                            if (memoryOpt.isSome()) {
                                return memoryOpt.getOrThrow().content == newContent
                            } else {
                                return false
                            }
                        }
                        case Err(_) => false
                    }
                }
                case Err(_) => false
            }
        }
        
        // 3. 搜索功能测试
        println("\n🔍 3. 搜索功能测试")
        
        // 添加更多测试数据
        let testMemories = [
            MemoryBuilder()
                .withAgentId("integration-test")
                .withContent("人工智能是未来的发展方向")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.9)
                .build(),
            MemoryBuilder()
                .withAgentId("integration-test")
                .withContent("机器学习算法很有趣")
                .withMemoryType(MemoryType.Episodic)
                .withImportance(0.7)
                .build(),
            MemoryBuilder()
                .withAgentId("integration-test")
                .withContent("深度学习需要大量数据")
                .withMemoryType(MemoryType.Procedural)
                .withImportance(0.8)
                .build()
        ]
        
        testSuite.runTest("批量添加测试数据") { client =>
            var successCount = 0
            for (memory in testMemories) {
                let addResult = client.addMemory(memory)
                match (addResult) {
                    case Ok(memoryId) => {
                        testMemoryIds.append(memoryId)
                        successCount = successCount + 1
                    }
                    case Err(_) => {}
                }
            }
            return successCount == testMemories.size
        }
        
        testSuite.runTest("基础搜索测试") { client =>
            let searchResult = client.searchMemories("人工智能", 5)
            match (searchResult) {
                case Ok(results) => {
                    return !results.isEmpty() && 
                           results.any { result => result.memory.content.contains("人工智能") }
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("过滤搜索测试") { client =>
            var filter = SearchFilter()
            filter.memoryTypes = Some([MemoryType.Semantic])
            filter.agentIds = Some(["integration-test"])
            
            let searchResult = client.searchMemoriesFiltered("*", filter, 10)
            match (searchResult) {
                case Ok(results) => {
                    return results.all { result => result.memory.memoryType == MemoryType.Semantic }
                }
                case Err(_) => false
            }
        }
        
        // 4. 批量操作测试
        println("\n📦 4. 批量操作测试")
        
        testSuite.runTest("批量添加记忆测试") { client =>
            let batchMemories = Array<Memory>()
            for (i in 0..5) {
                let memory = MemoryBuilder()
                    .withAgentId("integration-test")
                    .withContent("批量测试记忆 ${i + 1}")
                    .withMemoryType(MemoryType.Working)
                    .withImportance(0.6)
                    .build()
                batchMemories.append(memory)
            }
            
            let batchResult = client.addMemoriesBatch(batchMemories)
            match (batchResult) {
                case Ok(result) => {
                    // 保存ID用于后续清理
                    for (id in result.successes) {
                        testMemoryIds.append(id)
                    }
                    return result.successes.size == batchMemories.size
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("分页获取记忆测试") { client =>
            let pagination = PaginationParams(1, 5)
            let pageResult = client.getMemoriesPaginated("integration-test", pagination)
            match (pageResult) {
                case Ok(result) => {
                    return !result.data.isEmpty() && 
                           result.data.size <= 5 &&
                           result.page == 1 &&
                           result.pageSize == 5
                }
                case Err(_) => false
            }
        }
        
        // 5. 统计信息测试
        println("\n📊 5. 统计信息测试")
        
        testSuite.runTest("获取记忆统计测试") { client =>
            let statsResult = client.getMemoryStats("integration-test")
            match (statsResult) {
                case Ok(stats) => {
                    return stats.totalMemories > 0 &&
                           stats.averageImportance >= 0.0 &&
                           stats.averageImportance <= 1.0 &&
                           !stats.memoryTypeCount.isEmpty()
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("获取全局统计测试") { client =>
            let globalStatsResult = client.getGlobalStats()
            match (globalStatsResult) {
                case Ok(stats) => {
                    return stats.totalMemories >= 0
                }
                case Err(_) => false
            }
        }
        
        // 6. 高级功能测试
        println("\n🚀 6. 高级功能测试")
        
        testSuite.runTest("搜索管理器测试") { client =>
            let searchConfig = SearchConfig()
            let searchManager = SearchManager(client, searchConfig)
            
            let smartSearchResult = searchManager.smartSearch("学习", None, Some(3))
            match (smartSearchResult) {
                case Ok(results) => {
                    return results.size <= 3
                }
                case Err(_) => false
            }
        }
        
        testSuite.runTest("管理功能测试") { client =>
            let adminManager = AdminManager(client)
            
            let healthResult = adminManager.systemHealthCheck()
            match (healthResult) {
                case Ok(health) => {
                    return !health.version.isEmpty()
                }
                case Err(_) => false
            }
        }
        
        // 7. 错误处理测试
        println("\n❌ 7. 错误处理测试")
        
        testSuite.runTest("无效输入错误处理测试") { client =>
            let invalidMemory = MemoryBuilder()
                .withAgentId("")  // 空的agent ID应该导致错误
                .withContent("")  // 空内容应该导致错误
                .build()
            
            let addResult = client.addMemory(invalidMemory)
            match (addResult) {
                case Ok(_) => false  // 应该失败
                case Err(_) => true   // 预期的错误
            }
        }
        
        testSuite.runTest("不存在记忆获取测试") { client =>
            let getResult = client.getMemory("non-existent-id")
            match (getResult) {
                case Ok(memoryOpt) => memoryOpt.isNone()
                case Err(_) => true  // 也可能返回错误
            }
        }
        
        // 8. 性能测试
        println("\n⚡ 8. 性能测试")
        
        testSuite.runTest("搜索性能测试") { client =>
            let startTime = getCurrentTimestamp()
            var successCount = 0
            
            for (i in 0..10) {
                let searchResult = client.searchMemories("测试", 5)
                match (searchResult) {
                    case Ok(_) => successCount = successCount + 1
                    case Err(_) => {}
                }
            }
            
            let endTime = getCurrentTimestamp()
            let duration = endTime - startTime
            let averageTime = duration / 10
            
            println("   平均搜索时间: ${averageTime}ms")
            return successCount == 10 && averageTime < 1000  // 平均响应时间应该小于1秒
        }
        
        // 9. 清理测试数据
        println("\n🧹 9. 清理测试数据")
        
        testSuite.runTest("批量删除测试数据") { client =>
            if (testMemoryIds.isEmpty()) {
                return true
            }
            
            let batchDeleteResult = client.deleteMemoriesBatch(testMemoryIds)
            match (batchDeleteResult) {
                case Ok(result) => {
                    println("   删除了 ${result.successes.size}/${testMemoryIds.size} 条测试记忆")
                    return result.successes.size > 0
                }
                case Err(_) => {
                    // 尝试单个删除
                    var deletedCount = 0
                    for (memoryId in testMemoryIds) {
                        let deleteResult = client.deleteMemory(memoryId)
                        match (deleteResult) {
                            case Ok(_) => deletedCount = deletedCount + 1
                            case Err(_) => {}
                        }
                    }
                    println("   单个删除了 ${deletedCount}/${testMemoryIds.size} 条测试记忆")
                    return deletedCount > 0
                }
            }
        }
        
    } catch (e: Exception) {
        println("💥 集成测试执行异常: ${e}")
    } finally {
        // 清理测试环境
        testSuite.tearDown()
    }
    
    // 打印测试报告
    testSuite.printReport()
    
    // 返回测试结果
    if (testSuite.allPassed()) {
        println("\n🎉 所有集成测试通过！")
    } else {
        println("\n❌ 部分集成测试失败，请检查服务器连接和配置！")
    }
}
