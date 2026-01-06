/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 单元测试 - AgentMem 仓颉 SDK 的单元测试套件
 */

package agentmem.tests

import agentmem.api.*
import agentmem.core.*
import agentmem.ffi.*

/// 测试结果
public struct TestResult {
    public var name: String
    public var passed: Bool
    public var message: String
    public var duration: Int64

    public init(name: String, passed: Bool, message: String, duration: Int64) {
        this.name = name
        this.passed = passed
        this.message = message
        this.duration = duration
    }
}

/// 测试套件
public class TestSuite {
    private var _results: Array<TestResult>
    private var _totalTests: Int32
    private var _passedTests: Int32

    public init() {
        this._results = Array<TestResult>()
        this._totalTests = 0
        this._passedTests = 0
    }

    /// 运行测试
    public func runTest(name: String, testFunc: () -> Bool): Unit {
        let startTime = getCurrentTimestamp()
        var passed = false
        var message = ""

        try {
            passed = testFunc()
            message = if (passed) { "PASS" } else { "FAIL" }
        } catch (e: Exception) {
            passed = false
            message = "ERROR: ${e}"
        }

        let endTime = getCurrentTimestamp()
        let duration = endTime - startTime

        let result = TestResult(name, passed, message, duration)
        this._results.append(result)
        this._totalTests = this._totalTests + 1
        
        if (passed) {
            this._passedTests = this._passedTests + 1
        }

        let status = if (passed) { "✅" } else { "❌" }
        println("${status} ${name} (${duration}ms) - ${message}")
    }

    /// 断言相等
    public func assertEqual<T>(actual: T, expected: T, message: String = ""): Bool {
        let isEqual = actual == expected
        if (!isEqual && !message.isEmpty()) {
            println("   Expected: ${expected}, Actual: ${actual} - ${message}")
        }
        return isEqual
    }

    /// 断言为真
    public func assertTrue(condition: Bool, message: String = ""): Bool {
        if (!condition && !message.isEmpty()) {
            println("   ${message}")
        }
        return condition
    }

    /// 断言为假
    public func assertFalse(condition: Bool, message: String = ""): Bool {
        return this.assertTrue(!condition, message)
    }

    /// 断言不为空
    public func assertNotNull<T>(value: Option<T>, message: String = ""): Bool {
        let isNotNull = value.isSome()
        if (!isNotNull && !message.isEmpty()) {
            println("   ${message}")
        }
        return isNotNull
    }

    /// 打印测试报告
    public func printReport(): Unit {
        println("\n" + "=" * 60)
        println("📊 测试报告")
        println("=" * 60)
        println("总测试数: ${this._totalTests}")
        println("通过数量: ${this._passedTests}")
        println("失败数量: ${this._totalTests - this._passedTests}")
        println("通过率: ${Float32(this._passedTests) / Float32(this._totalTests) * 100.0:.1f}%")
        
        let totalDuration = this._results.fold(Int64(0), { acc, result => acc + result.duration })
        println("总耗时: ${totalDuration}ms")
        
        if (this._passedTests < this._totalTests) {
            println("\n❌ 失败的测试:")
            for (result in this._results) {
                if (!result.passed) {
                    println("   ${result.name}: ${result.message}")
                }
            }
        }
        
        println("=" * 60)
    }

    /// 获取测试结果
    public func getResults(): Array<TestResult> {
        return this._results
    }

    /// 是否所有测试都通过
    public func allPassed(): Bool {
        return this._passedTests == this._totalTests
    }
}

/// 单元测试主函数
main() {
    println("🧪 AgentMem 仓颉 SDK 单元测试")
    println("=" * 50)
    
    let testSuite = TestSuite()
    
    // 1. 核心类型测试
    println("\n📋 1. 核心类型测试")
    
    testSuite.runTest("MemoryType 枚举测试") {
        let episodic = MemoryType.Episodic
        let semantic = MemoryType.Semantic
        
        return testSuite.assertEqual(episodic.toString(), "Episodic") &&
               testSuite.assertEqual(semantic.toUInt32(), UInt32(1)) &&
               testSuite.assertTrue(episodic != semantic)
    }
    
    testSuite.runTest("ImportanceLevel 枚举测试") {
        let low = ImportanceLevel.Low
        let high = ImportanceLevel.High
        
        return testSuite.assertEqual(low.toString(), "Low") &&
               testSuite.assertEqual(high.toFloat32(), 0.8) &&
               testSuite.assertTrue(low != high)
    }
    
    testSuite.runTest("Memory 结构体测试") {
        let memory = Memory(
            "test-id",
            "test-agent",
            "test-user",
            "test-session",
            "测试内容",
            MemoryType.Semantic,
            0.8,
            getCurrentTimestamp(),
            getCurrentTimestamp(),
            0,
            HashMap<String, String>(),
            Array<Float32>()
        )
        
        return testSuite.assertEqual(memory.id, "test-id") &&
               testSuite.assertEqual(memory.content, "测试内容") &&
               testSuite.assertEqual(memory.memoryType, MemoryType.Semantic) &&
               testSuite.assertEqual(memory.importance, 0.8)
    }
    
    // 2. MemoryBuilder 测试
    println("\n🏗️ 2. MemoryBuilder 测试")
    
    testSuite.runTest("MemoryBuilder 基础构建测试") {
        let memory = MemoryBuilder()
            .withAgentId("test-agent")
            .withContent("测试内容")
            .withMemoryType(MemoryType.Episodic)
            .withImportance(0.9)
            .build()
        
        return testSuite.assertEqual(memory.agentId, "test-agent") &&
               testSuite.assertEqual(memory.content, "测试内容") &&
               testSuite.assertEqual(memory.memoryType, MemoryType.Episodic) &&
               testSuite.assertEqual(memory.importance, 0.9)
    }
    
    testSuite.runTest("MemoryBuilder 元数据测试") {
        let memory = MemoryBuilder()
            .withAgentId("test-agent")
            .withContent("测试内容")
            .withMetadata("key1", "value1")
            .withMetadata("key2", "value2")
            .build()
        
        return testSuite.assertTrue(memory.metadata.containsKey("key1")) &&
               testSuite.assertEqual(memory.metadata["key1"], "value1") &&
               testSuite.assertTrue(memory.metadata.containsKey("key2")) &&
               testSuite.assertEqual(memory.metadata["key2"], "value2")
    }
    
    testSuite.runTest("MemoryBuilder 向量测试") {
        let embedding = [0.1, 0.2, 0.3, 0.4, 0.5]
        let memory = MemoryBuilder()
            .withAgentId("test-agent")
            .withContent("测试内容")
            .withEmbedding(embedding)
            .build()
        
        return testSuite.assertEqual(memory.embedding.size, 5) &&
               testSuite.assertEqual(memory.embedding[0], 0.1) &&
               testSuite.assertEqual(memory.embedding[4], 0.5)
    }
    
    // 3. 配置类测试
    println("\n⚙️ 3. 配置类测试")
    
    testSuite.runTest("ClientConfig 基础测试") {
        let config = ClientConfig("http://localhost:8080")
        
        return testSuite.assertEqual(config.serverUrl, "http://localhost:8080") &&
               testSuite.assertEqual(config.timeout, 30) &&
               testSuite.assertEqual(config.retryCount, 3) &&
               testSuite.assertFalse(config.debugMode)
    }
    
    testSuite.runTest("ClientConfig 链式配置测试") {
        let config = ClientConfig("http://localhost:8080")
            .withApiKey("test-key")
            .withTimeout(60)
            .withRetryCount(5)
            .withCache(true, 200)
            .withDebugMode(true)
        
        return testSuite.assertEqual(config.apiKey.getOrElse(""), "test-key") &&
               testSuite.assertEqual(config.timeout, 60) &&
               testSuite.assertEqual(config.retryCount, 5) &&
               testSuite.assertTrue(config.enableCache) &&
               testSuite.assertEqual(config.cacheSize, 200) &&
               testSuite.assertTrue(config.debugMode)
    }
    
    testSuite.runTest("SearchConfig 测试") {
        var searchConfig = SearchConfig()
        searchConfig.maxResults = 50
        searchConfig.similarityThreshold = 0.8
        searchConfig.enableSemanticSearch = true
        
        return testSuite.assertEqual(searchConfig.maxResults, UInt32(50)) &&
               testSuite.assertEqual(searchConfig.similarityThreshold, 0.8) &&
               testSuite.assertTrue(searchConfig.enableSemanticSearch)
    }
    
    // 4. 错误处理测试
    println("\n❌ 4. 错误处理测试")
    
    testSuite.runTest("AgentMemError 创建测试") {
        let error1 = AgentMemError.InvalidInput("测试错误")
        let error2 = AgentMemError.NetworkError("网络错误")
        let error3 = AgentMemError.InternalError("内部错误")
        
        return testSuite.assertEqual(error1.getMessage(), "测试错误") &&
               testSuite.assertEqual(error2.getMessage(), "网络错误") &&
               testSuite.assertEqual(error3.getMessage(), "内部错误")
    }
    
    testSuite.runTest("Result 类型测试") {
        let successResult = AgentMemResult<String>.Ok("成功")
        let errorResult = AgentMemResult<String>.Err(AgentMemError.InvalidInput("失败"))
        
        var successValue = ""
        var errorMessage = ""
        
        match (successResult) {
            case Ok(value) => successValue = value
            case Err(_) => {}
        }
        
        match (errorResult) {
            case Ok(_) => {}
            case Err(error) => errorMessage = error.getMessage()
        }
        
        return testSuite.assertEqual(successValue, "成功") &&
               testSuite.assertEqual(errorMessage, "失败")
    }
    
    // 5. 搜索过滤器测试
    println("\n🔍 5. 搜索过滤器测试")
    
    testSuite.runTest("SearchFilter 基础测试") {
        var filter = SearchFilter()
        filter.agentIds = Some(["agent1", "agent2"])
        filter.memoryTypes = Some([MemoryType.Semantic, MemoryType.Episodic])
        filter.importanceRange = Some((0.5, 1.0))
        
        return testSuite.assertNotNull(filter.agentIds) &&
               testSuite.assertEqual(filter.agentIds.getOrThrow().size, 2) &&
               testSuite.assertNotNull(filter.memoryTypes) &&
               testSuite.assertEqual(filter.memoryTypes.getOrThrow().size, 2) &&
               testSuite.assertNotNull(filter.importanceRange)
    }
    
    testSuite.runTest("PaginationParams 测试") {
        let pagination = PaginationParams(2, 20)
        
        return testSuite.assertEqual(pagination.page, UInt32(2)) &&
               testSuite.assertEqual(pagination.pageSize, UInt32(20))
    }
    
    // 6. 批量结果测试
    println("\n📦 6. 批量结果测试")
    
    testSuite.runTest("BatchResult 测试") {
        var batchResult = BatchResult<String>()
        batchResult.addSuccess("success1")
        batchResult.addSuccess("success2")
        batchResult.addFailure("error1")
        
        return testSuite.assertEqual(batchResult.total, UInt32(3)) &&
               testSuite.assertEqual(batchResult.successes.size, 2) &&
               testSuite.assertEqual(batchResult.failures.size, 1) &&
               testSuite.assertEqual(batchResult.successes[0], "success1") &&
               testSuite.assertEqual(batchResult.failures[0], "error1")
    }
    
    testSuite.runTest("PaginatedResult 测试") {
        let memories = [
            MemoryBuilder().withAgentId("test").withContent("内容1").build(),
            MemoryBuilder().withAgentId("test").withContent("内容2").build()
        ]
        
        let paginatedResult = PaginatedResult<Memory>(memories, 1, 10, 100)
        
        return testSuite.assertEqual(paginatedResult.data.size, 2) &&
               testSuite.assertEqual(paginatedResult.page, UInt32(1)) &&
               testSuite.assertEqual(paginatedResult.pageSize, UInt32(10)) &&
               testSuite.assertEqual(paginatedResult.totalCount, UInt32(100))
    }
    
    // 7. FFI 工具测试
    println("\n🔧 7. FFI 工具测试")
    
    testSuite.runTest("CStringConverter 测试") {
        let originalString = "测试字符串"
        let cString = CStringConverter.toCString(originalString)
        let convertedString = CStringConverter.toString(cString)
        CStringConverter.freeCString(cString)
        
        return testSuite.assertEqual(convertedString, originalString)
    }
    
    testSuite.runTest("MemoryConverter 测试") {
        let originalMemory = MemoryBuilder()
            .withAgentId("test-agent")
            .withContent("测试内容")
            .withMemoryType(MemoryType.Semantic)
            .withImportance(0.8)
            .build()
        
        let cMemory = MemoryConverter.toCMemory(originalMemory)
        let convertedMemory = MemoryConverter.fromCMemory(cMemory)
        MemoryConverter.freeCMemory(cMemory)
        
        return testSuite.assertEqual(convertedMemory.agentId, originalMemory.agentId) &&
               testSuite.assertEqual(convertedMemory.content, originalMemory.content) &&
               testSuite.assertEqual(convertedMemory.memoryType, originalMemory.memoryType) &&
               testSuite.assertEqual(convertedMemory.importance, originalMemory.importance)
    }
    
    // 8. 缓存测试
    println("\n💾 8. 缓存测试")
    
    testSuite.runTest("SimpleCache 基础测试") {
        let cache = SimpleCache<String, String>(3)
        
        cache.put("key1", "value1")
        cache.put("key2", "value2")
        cache.put("key3", "value3")
        
        return testSuite.assertEqual(cache.get("key1").getOrElse(""), "value1") &&
               testSuite.assertEqual(cache.get("key2").getOrElse(""), "value2") &&
               testSuite.assertEqual(cache.get("key3").getOrElse(""), "value3") &&
               testSuite.assertEqual(cache.size(), 3)
    }
    
    testSuite.runTest("SimpleCache LRU 测试") {
        let cache = SimpleCache<String, String>(2)
        
        cache.put("key1", "value1")
        cache.put("key2", "value2")
        cache.put("key3", "value3")  // 应该淘汰 key1
        
        return testSuite.assertTrue(cache.get("key1").isNone()) &&
               testSuite.assertEqual(cache.get("key2").getOrElse(""), "value2") &&
               testSuite.assertEqual(cache.get("key3").getOrElse(""), "value3") &&
               testSuite.assertEqual(cache.size(), 2)
    }
    
    // 9. 性能监控测试
    println("\n⚡ 9. 性能监控测试")
    
    testSuite.runTest("PerformanceMonitor 测试") {
        let monitor = PerformanceMonitor("test_operation")
        
        // 模拟一些工作
        let startTime = getCurrentTimestamp()
        while (getCurrentTimestamp() - startTime < 10) {
            // 等待10ms
        }
        
        let duration = monitor.finish()
        
        return testSuite.assertTrue(duration >= 10) &&
               testSuite.assertTrue(duration < 100)  // 应该不会超过100ms
    }
    
    // 打印测试报告
    testSuite.printReport()
    
    // 返回测试结果
    if (testSuite.allPassed()) {
        println("\n🎉 所有单元测试通过！")
    } else {
        println("\n❌ 部分单元测试失败，请检查代码！")
    }
}
