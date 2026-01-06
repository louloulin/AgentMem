/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 性能测试 - AgentMem 仓颉 SDK 的性能测试套件
 */

package agentmem.tests

import agentmem.api.*
import agentmem.core.*

/// 性能测试结果
public struct PerformanceResult {
    public var testName: String
    public var totalOperations: Int32
    public var totalDuration: Int64  // 毫秒
    public var averageLatency: Float32  // 毫秒
    public var throughput: Float32  // 操作/秒
    public var minLatency: Int64  // 毫秒
    public var maxLatency: Int64  // 毫秒
    public var p95Latency: Int64  // 毫秒
    public var successRate: Float32  // 成功率 (0-1)

    public init(
        testName: String,
        totalOperations: Int32,
        totalDuration: Int64,
        latencies: Array<Int64>,
        successCount: Int32
    ) {
        this.testName = testName
        this.totalOperations = totalOperations
        this.totalDuration = totalDuration
        this.averageLatency = Float32(totalDuration) / Float32(totalOperations)
        this.throughput = Float32(totalOperations) / Float32(totalDuration) * 1000.0
        this.successRate = Float32(successCount) / Float32(totalOperations)
        
        if (!latencies.isEmpty()) {
            this.minLatency = latencies.min()
            this.maxLatency = latencies.max()
            
            // 计算P95延迟
            let sortedLatencies = latencies.sorted()
            let p95Index = Int32(Float32(sortedLatencies.size) * 0.95)
            this.p95Latency = if (p95Index < sortedLatencies.size) {
                sortedLatencies[p95Index]
            } else {
                sortedLatencies[sortedLatencies.size - 1]
            }
        } else {
            this.minLatency = 0
            this.maxLatency = 0
            this.p95Latency = 0
        }
    }

    public func printReport(): Unit {
        println("📊 ${this.testName} 性能测试报告:")
        println("   总操作数: ${this.totalOperations}")
        println("   总耗时: ${this.totalDuration}ms")
        println("   平均延迟: ${this.averageLatency:.2f}ms")
        println("   吞吐量: ${this.throughput:.2f} ops/sec")
        println("   成功率: ${this.successRate * 100.0:.1f}%")
        println("   延迟分布:")
        println("     最小值: ${this.minLatency}ms")
        println("     最大值: ${this.maxLatency}ms")
        println("     P95: ${this.p95Latency}ms")
    }
}

/// 性能测试套件
public class PerformanceTestSuite {
    private var _client: Option<AgentMemClient>
    private var _results: Array<PerformanceResult>

    public init() {
        this._client = None
        this._results = Array<PerformanceResult>()
    }

    /// 设置测试环境
    public func setUp(): Bool {
        try {
            println("🔧 设置性能测试环境...")
            
            let config = ClientConfig("http://localhost:8080")
                .withApiKey("performance-test-key")
                .withTimeout(60)  // 性能测试需要更长超时
                .withRetryCount(1)  // 减少重试以获得真实性能数据
                .withCache(false, 0)  // 禁用缓存以测试真实性能
                .withDebugMode(false)  // 禁用调试模式以减少开销
            
            let client = AgentMemClient(config)
            let initResult = client.initialize()
            
            match (initResult) {
                case Ok(_) => {
                    this._client = Some(client)
                    println("✅ 性能测试环境设置成功")
                    return true
                }
                case Err(error) => {
                    println("❌ 性能测试环境设置失败: ${error.getMessage()}")
                    return false
                }
            }
            
        } catch (e: Exception) {
            println("❌ 性能测试环境设置异常: ${e}")
            return false
        }
    }

    /// 清理测试环境
    public func tearDown(): Unit {
        if (this._client.isSome()) {
            this._client.getOrThrow().close()
            this._client = None
            println("🧹 性能测试环境清理完成")
        }
    }

    /// 运行性能测试
    public func runPerformanceTest(
        testName: String,
        operationCount: Int32,
        operation: (AgentMemClient) -> Bool
    ): PerformanceResult {
        if (this._client.isNone()) {
            println("❌ ${testName} - 客户端未初始化")
            return PerformanceResult(testName, 0, 0, Array<Int64>(), 0)
        }

        println("🚀 开始性能测试: ${testName}")
        println("   操作数量: ${operationCount}")
        
        let client = this._client.getOrThrow()
        var latencies = Array<Int64>()
        var successCount = 0
        
        let totalStartTime = getCurrentTimestamp()
        
        for (i in 0..operationCount) {
            let operationStartTime = getCurrentTimestamp()
            
            let success = try {
                operation(client)
            } catch (e: Exception) {
                false
            }
            
            let operationEndTime = getCurrentTimestamp()
            let latency = operationEndTime - operationStartTime
            latencies.append(latency)
            
            if (success) {
                successCount = successCount + 1
            }
            
            // 每100次操作打印进度
            if ((i + 1) % 100 == 0) {
                let progress = Float32(i + 1) / Float32(operationCount) * 100.0
                println("   进度: ${progress:.1f}% (${i + 1}/${operationCount})")
            }
        }
        
        let totalEndTime = getCurrentTimestamp()
        let totalDuration = totalEndTime - totalStartTime
        
        let result = PerformanceResult(testName, operationCount, totalDuration, latencies, successCount)
        this._results.append(result)
        
        result.printReport()
        return result
    }

    /// 打印所有测试结果
    public func printSummary(): Unit {
        println("\n" + "=" * 80)
        println("📊 性能测试总结报告")
        println("=" * 80)
        
        for (result in this._results) {
            println("${result.testName}:")
            println("  吞吐量: ${result.throughput:.2f} ops/sec")
            println("  平均延迟: ${result.averageLatency:.2f}ms")
            println("  P95延迟: ${result.p95Latency}ms")
            println("  成功率: ${result.successRate * 100.0:.1f}%")
            println()
        }
        
        println("=" * 80)
    }
}

/// 性能测试主函数
main() {
    println("⚡ AgentMem 仓颉 SDK 性能测试")
    println("=" * 50)
    
    let testSuite = PerformanceTestSuite()
    
    // 设置测试环境
    if (!testSuite.setUp()) {
        println("❌ 无法设置性能测试环境，跳过性能测试")
        return
    }
    
    var testMemoryIds = Array<String>()
    
    try {
        // 1. 记忆添加性能测试
        println("\n📝 1. 记忆添加性能测试")
        
        testSuite.runPerformanceTest("单个记忆添加", 100) { client =>
            let memory = MemoryBuilder()
                .withAgentId("perf-test")
                .withContent("性能测试记忆内容 ${getCurrentTimestamp()}")
                .withMemoryType(MemoryType.Semantic)
                .withImportance(0.7)
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
        
        // 2. 批量添加性能测试
        println("\n📦 2. 批量添加性能测试")
        
        testSuite.runPerformanceTest("批量记忆添加 (10个/批)", 20) { client =>
            var batchMemories = Array<Memory>()
            for (i in 0..10) {
                let memory = MemoryBuilder()
                    .withAgentId("perf-test")
                    .withContent("批量性能测试记忆 ${i} - ${getCurrentTimestamp()}")
                    .withMemoryType(MemoryType.Working)
                    .withImportance(0.6)
                    .build()
                batchMemories.append(memory)
            }
            
            let batchResult = client.addMemoriesBatch(batchMemories)
            match (batchResult) {
                case Ok(result) => {
                    for (id in result.successes) {
                        testMemoryIds.append(id)
                    }
                    return result.successes.size == batchMemories.size
                }
                case Err(_) => false
            }
        }
        
        // 3. 记忆搜索性能测试
        println("\n🔍 3. 记忆搜索性能测试")
        
        let searchQueries = [
            "性能测试", "记忆内容", "测试数据", "搜索查询", "人工智能",
            "机器学习", "深度学习", "算法", "数据结构", "编程语言"
        ]
        
        testSuite.runPerformanceTest("记忆搜索", 200) { client =>
            let query = searchQueries[getCurrentTimestamp() % Int64(searchQueries.size)]
            let searchResult = client.searchMemories(query, 10)
            match (searchResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 4. 记忆获取性能测试
        println("\n📖 4. 记忆获取性能测试")
        
        testSuite.runPerformanceTest("单个记忆获取", 150) { client =>
            if (testMemoryIds.isEmpty()) {
                return false
            }
            
            let randomIndex = getCurrentTimestamp() % Int64(testMemoryIds.size)
            let memoryId = testMemoryIds[Int32(randomIndex)]
            
            let getResult = client.getMemory(memoryId)
            match (getResult) {
                case Ok(memoryOpt) => memoryOpt.isSome()
                case Err(_) => false
            }
        }
        
        // 5. 分页获取性能测试
        println("\n📄 5. 分页获取性能测试")
        
        testSuite.runPerformanceTest("分页获取记忆", 50) { client =>
            let randomPage = UInt32((getCurrentTimestamp() % 10) + 1)
            let pagination = PaginationParams(randomPage, 20)
            
            let pageResult = client.getMemoriesPaginated("perf-test", pagination)
            match (pageResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 6. 记忆更新性能测试
        println("\n✏️ 6. 记忆更新性能测试")
        
        testSuite.runPerformanceTest("记忆更新", 80) { client =>
            if (testMemoryIds.isEmpty()) {
                return false
            }
            
            let randomIndex = getCurrentTimestamp() % Int64(testMemoryIds.size)
            let memoryId = testMemoryIds[Int32(randomIndex)]
            let newContent = "更新的性能测试内容 ${getCurrentTimestamp()}"
            
            let updateResult = client.updateMemory(memoryId, newContent)
            match (updateResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 7. 过滤搜索性能测试
        println("\n🎯 7. 过滤搜索性能测试")
        
        testSuite.runPerformanceTest("过滤搜索", 100) { client =>
            var filter = SearchFilter()
            filter.agentIds = Some(["perf-test"])
            filter.memoryTypes = Some([MemoryType.Semantic, MemoryType.Working])
            filter.importanceRange = Some((0.5, 1.0))
            
            let searchResult = client.searchMemoriesFiltered("*", filter, 15)
            match (searchResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 8. 统计信息获取性能测试
        println("\n📊 8. 统计信息获取性能测试")
        
        testSuite.runPerformanceTest("获取统计信息", 50) { client =>
            let statsResult = client.getMemoryStats("perf-test")
            match (statsResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 9. 并发模拟测试
        println("\n🔄 9. 并发模拟测试")
        
        testSuite.runPerformanceTest("混合操作模拟", 200) { client =>
            let operationType = getCurrentTimestamp() % 4
            
            match (operationType) {
                case 0 => {
                    // 添加记忆
                    let memory = MemoryBuilder()
                        .withAgentId("perf-test")
                        .withContent("并发测试记忆 ${getCurrentTimestamp()}")
                        .withMemoryType(MemoryType.Episodic)
                        .withImportance(0.7)
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
                case 1 => {
                    // 搜索记忆
                    let searchResult = client.searchMemories("并发测试", 5)
                    match (searchResult) {
                        case Ok(_) => true
                        case Err(_) => false
                    }
                }
                case 2 => {
                    // 获取记忆
                    if (!testMemoryIds.isEmpty()) {
                        let randomIndex = getCurrentTimestamp() % Int64(testMemoryIds.size)
                        let memoryId = testMemoryIds[Int32(randomIndex)]
                        
                        let getResult = client.getMemory(memoryId)
                        match (getResult) {
                            case Ok(_) => true
                            case Err(_) => false
                        }
                    } else {
                        return true
                    }
                }
                case _ => {
                    // 获取统计信息
                    let statsResult = client.getMemoryStats("perf-test")
                    match (statsResult) {
                        case Ok(_) => true
                        case Err(_) => false
                    }
                }
            }
        }
        
        // 10. 压力测试
        println("\n💪 10. 压力测试")
        
        testSuite.runPerformanceTest("高频搜索压力测试", 500) { client =>
            let queries = ["压力", "测试", "性能", "搜索", "记忆"]
            let query = queries[getCurrentTimestamp() % Int64(queries.size)]
            
            let searchResult = client.searchMemories(query, 3)
            match (searchResult) {
                case Ok(_) => true
                case Err(_) => false
            }
        }
        
        // 11. 清理性能测试数据
        println("\n🧹 11. 清理性能测试数据")
        
        if (!testMemoryIds.isEmpty()) {
            println("清理 ${testMemoryIds.size} 条测试记忆...")
            
            // 分批删除以避免超时
            let batchSize = 50
            var deletedCount = 0
            
            for (i in 0..(testMemoryIds.size / batchSize + 1)) {
                let startIndex = i * batchSize
                let endIndex = Int32.min(startIndex + batchSize, testMemoryIds.size)
                
                if (startIndex >= testMemoryIds.size) {
                    break
                }
                
                var batchIds = Array<String>()
                for (j in startIndex..endIndex) {
                    batchIds.append(testMemoryIds[j])
                }
                
                if (!batchIds.isEmpty()) {
                    testSuite.runPerformanceTest("批量删除 (${batchIds.size}个)", 1) { client =>
                        let deleteResult = client.deleteMemoriesBatch(batchIds)
                        match (deleteResult) {
                            case Ok(result) => {
                                deletedCount = deletedCount + result.successes.size
                                return result.successes.size > 0
                            }
                            case Err(_) => false
                        }
                    }
                }
            }
            
            println("✅ 清理完成，删除了 ${deletedCount} 条测试记忆")
        }
        
    } catch (e: Exception) {
        println("💥 性能测试执行异常: ${e}")
    } finally {
        // 清理测试环境
        testSuite.tearDown()
    }
    
    // 打印性能测试总结
    testSuite.printSummary()
    
    println("\n🎉 性能测试完成！")
    println("💡 建议的性能基准:")
    println("   - 单个记忆添加: < 50ms 平均延迟")
    println("   - 记忆搜索: < 100ms 平均延迟")
    println("   - 记忆获取: < 20ms 平均延迟")
    println("   - 吞吐量: > 100 ops/sec")
    println("   - 成功率: > 95%")
}
