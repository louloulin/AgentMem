# AgentMem 仓颉 SDK 最佳实践指南

## 🎯 概述

本指南提供使用 AgentMem 仓颉 SDK 的最佳实践，包括错误处理、性能优化、内存管理和安全性考虑。

## 🛡️ 安全最佳实践

### 1. FFI调用安全

**✅ 推荐做法**:
```cangjie
// 所有FFI调用都应该在unsafe块中
unsafe {
    let result = ffi_function_call(param)
    // 立即检查结果
    if (result.isNull()) {
        return Err(AgentMemError.InternalError("FFI调用失败"))
    }
}
```

**❌ 避免做法**:
```cangjie
// 不要在unsafe块外调用FFI函数
let result = ffi_function_call(param)  // 编译错误
```

### 2. 内存管理

**✅ 推荐做法**:
```cangjie
// 使用RAII模式管理资源
public class ResourceManager {
    private var resource: CPointer<UInt8>
    
    public init(size: USize) {
        unsafe {
            this.resource = allocate(size)
        }
    }
    
    // 析构函数自动清理资源
    public func finalize() {
        unsafe {
            if (!this.resource.isNull()) {
                deallocate(this.resource)
            }
        }
    }
}
```

**❌ 避免做法**:
```cangjie
// 不要忘记释放分配的内存
let ptr = allocate(1024)
// 忘记调用 deallocate(ptr) - 内存泄漏！
```

### 3. 输入验证

**✅ 推荐做法**:
```cangjie
public func addMemory(memory: Memory): AgentMemResult<String> {
    // 验证输入参数
    if (memory.id.isEmpty()) {
        return Err(AgentMemError.InvalidParameter("记忆ID不能为空"))
    }
    
    if (memory.content.size > MAX_CONTENT_SIZE) {
        return Err(AgentMemError.InvalidParameter("记忆内容过长"))
    }
    
    // 继续处理...
}
```

## ⚡ 性能最佳实践

### 1. 批量操作

**✅ 推荐做法**:
```cangjie
// 使用批量操作减少网络往返
let memories = [memory1, memory2, memory3, memory4, memory5]
match (client.addMemories(memories)) {
    case Ok(ids) => println("批量添加成功")
    case Err(error) => println("批量添加失败: ${error.getMessage()}")
}
```

**❌ 避免做法**:
```cangjie
// 避免循环中的单个操作
for (memory in memories) {
    client.addMemory(memory)  // 每次都是一个网络请求
}
```

### 2. 连接复用

**✅ 推荐做法**:
```cangjie
// 创建一个长期存在的客户端实例
let client = AgentMemClientBuilder()
    .withServerUrl("https://api.agentmem.com")
    .withApiKey("your-api-key")
    .withTimeout(30000)  // 30秒超时
    .build()

// 在整个应用生命周期中复用这个客户端
```

**❌ 避免做法**:
```cangjie
// 避免为每个操作创建新客户端
func addMemory(memory: Memory) {
    let client = AgentMemClientBuilder().build()  // 低效！
    client.addMemory(memory)
}
```

### 3. 缓存策略

**✅ 推荐做法**:
```cangjie
public class MemoryCache {
    private var cache: SimpleMap = SimpleMap()
    private let maxSize: Int64 = 1000
    
    public func getMemory(id: String): AgentMemResult<Memory> {
        // 先检查缓存
        if (cache.contains(id)) {
            return Ok(cache.get(id) as Memory)
        }
        
        // 缓存未命中，从服务器获取
        match (client.getMemory(id)) {
            case Ok(memory) => {
                // 添加到缓存
                if (cache.size < maxSize) {
                    cache.put(id, memory)
                }
                return Ok(memory)
            }
            case Err(error) => return Err(error)
        }
    }
}
```

## 🔧 错误处理最佳实践

### 1. 统一错误处理

**✅ 推荐做法**:
```cangjie
public func handleMemoryOperation<T>(
    operation: () -> AgentMemResult<T>
): AgentMemResult<T> {
    match (operation()) {
        case Ok(result) => return Ok(result)
        case Err(error) => {
            // 统一的错误日志记录
            logError("操作失败", error)
            
            // 根据错误类型决定是否重试
            if (error.isRetryable()) {
                return retryOperation(operation)
            }
            
            return Err(error)
        }
    }
}
```

### 2. 重试机制

**✅ 推荐做法**:
```cangjie
public func retryOperation<T>(
    operation: () -> AgentMemResult<T>,
    maxRetries: UInt32 = 3,
    delayMs: UInt32 = 1000
): AgentMemResult<T> {
    var attempts: UInt32 = 0
    
    while (attempts < maxRetries) {
        match (operation()) {
            case Ok(result) => return Ok(result)
            case Err(error) => {
                attempts += 1
                
                if (!error.isRetryable() || attempts >= maxRetries) {
                    return Err(error)
                }
                
                // 指数退避延迟
                let delay = delayMs * (2 ** (attempts - 1))
                Thread.sleep(delay)
            }
        }
    }
    
    return Err(AgentMemError.InternalError("重试次数已用完"))
}
```

### 3. 错误恢复

**✅ 推荐做法**:
```cangjie
public func robustMemoryRetrieval(id: String): AgentMemResult<Memory> {
    // 首先尝试从主存储获取
    match (client.getMemory(id)) {
        case Ok(memory) => return Ok(memory)
        case Err(error) => {
            // 如果是网络错误，尝试从本地缓存获取
            if (error is NetworkError) {
                match (localCache.getMemory(id)) {
                    case Ok(memory) => {
                        println("从本地缓存恢复记忆: ${id}")
                        return Ok(memory)
                    }
                    case Err(_) => {
                        // 缓存也没有，返回原始错误
                        return Err(error)
                    }
                }
            }
            
            return Err(error)
        }
    }
}
```

## 🧪 测试最佳实践

### 1. 单元测试结构

**✅ 推荐做法**:
```cangjie
public class MemoryServiceTests {
    private var service: MemoryService
    private var mockClient: MockAgentMemClient
    
    public func setUp() {
        mockClient = MockAgentMemClient()
        service = MemoryService(mockClient)
    }
    
    public func testAddMemorySuccess() {
        // Arrange
        let memory = createTestMemory()
        mockClient.setExpectedResult(Ok("memory-001"))
        
        // Act
        let result = service.addMemory(memory)
        
        // Assert
        match (result) {
            case Ok(id) => {
                assert(id == "memory-001", "ID应该匹配")
            }
            case Err(_) => {
                assert(false, "操作应该成功")
            }
        }
    }
    
    public func testAddMemoryFailure() {
        // Arrange
        let memory = createTestMemory()
        let expectedError = AgentMemError.InvalidParameter("测试错误")
        mockClient.setExpectedResult(Err(expectedError))
        
        // Act
        let result = service.addMemory(memory)
        
        // Assert
        match (result) {
            case Ok(_) => {
                assert(false, "操作应该失败")
            }
            case Err(error) => {
                assert(error.getMessage() == "测试错误", "错误消息应该匹配")
            }
        }
    }
}
```

### 2. 集成测试

**✅ 推荐做法**:
```cangjie
public class IntegrationTests {
    private var client: AgentMemClient
    
    public func setUp() {
        // 使用测试环境配置
        client = AgentMemClientBuilder()
            .withServerUrl("https://test-api.agentmem.com")
            .withApiKey("test-api-key")
            .withTimeout(5000)
            .build()
    }
    
    public func testEndToEndWorkflow() {
        // 创建记忆
        let memory = createTestMemory()
        let addResult = client.addMemory(memory)
        
        match (addResult) {
            case Ok(id) => {
                // 验证记忆已创建
                match (client.getMemory(id)) {
                    case Ok(retrievedMemory) => {
                        assert(retrievedMemory.content == memory.content)
                        
                        // 清理测试数据
                        client.deleteMemory(id)
                    }
                    case Err(error) => {
                        assert(false, "应该能够检索记忆: ${error.getMessage()}")
                    }
                }
            }
            case Err(error) => {
                assert(false, "应该能够添加记忆: ${error.getMessage()}")
            }
        }
    }
}
```

## 📊 监控和日志最佳实践

### 1. 结构化日志

**✅ 推荐做法**:
```cangjie
public class Logger {
    public static func logMemoryOperation(
        operation: String,
        memoryId: String,
        agentId: String,
        duration: Int64,
        success: Bool
    ) {
        let logEntry = SimpleMap()
        logEntry.put("timestamp", TimeUtils.getCurrentTimestamp())
        logEntry.put("operation", operation)
        logEntry.put("memory_id", memoryId)
        logEntry.put("agent_id", agentId)
        logEntry.put("duration_ms", duration)
        logEntry.put("success", success)
        
        println("MEMORY_OP: ${logEntry.toJson()}")
    }
}
```

### 2. 性能监控

**✅ 推荐做法**:
```cangjie
public class PerformanceMonitor {
    public static func measureOperation<T>(
        operationName: String,
        operation: () -> AgentMemResult<T>
    ): AgentMemResult<T> {
        let startTime = TimeUtils.getCurrentTimestamp()
        let result = operation()
        let endTime = TimeUtils.getCurrentTimestamp()
        let duration = endTime - startTime
        
        // 记录性能指标
        recordMetric(operationName, duration, result.isOk())
        
        // 如果操作耗时过长，记录警告
        if (duration > SLOW_OPERATION_THRESHOLD) {
            println("警告: 操作 ${operationName} 耗时 ${duration}ms")
        }
        
        return result
    }
}
```

## 🔄 版本兼容性最佳实践

### 1. API版本处理

**✅ 推荐做法**:
```cangjie
public class VersionCompatibility {
    public static func checkApiVersion(serverVersion: String): Bool {
        let clientVersion = "1.0.0"
        let serverMajor = extractMajorVersion(serverVersion)
        let clientMajor = extractMajorVersion(clientVersion)
        
        // 主版本号必须匹配
        return serverMajor == clientMajor
    }
    
    public static func handleVersionMismatch(
        clientVersion: String,
        serverVersion: String
    ): AgentMemResult<Void> {
        if (!checkApiVersion(serverVersion)) {
            return Err(AgentMemError.InternalError(
                "API版本不兼容: 客户端=${clientVersion}, 服务器=${serverVersion}"
            ))
        }
        
        return Ok(())
    }
}
```

## 📝 代码风格最佳实践

### 1. 命名约定

**✅ 推荐做法**:
```cangjie
// 类名使用PascalCase
public class MemoryManager

// 函数名使用camelCase
public func addMemory()

// 常量使用UPPER_SNAKE_CASE
public let MAX_MEMORY_SIZE: Int64 = 1024 * 1024

// 变量名使用camelCase
let memoryId: String = "memory-001"
```

### 2. 文档注释

**✅ 推荐做法**:
```cangjie
/**
 * 添加新记忆到存储系统
 * 
 * @param memory 要添加的记忆对象
 * @return 成功时返回记忆ID，失败时返回错误
 * 
 * @throws AgentMemError.InvalidParameter 当记忆对象无效时
 * @throws AgentMemError.NetworkError 当网络连接失败时
 * 
 * @example
 * ```cangjie
 * let memory = Memory(...)
 * match (client.addMemory(memory)) {
 *     case Ok(id) => println("记忆已添加: ${id}")
 *     case Err(error) => println("添加失败: ${error.getMessage()}")
 * }
 * ```
 */
public func addMemory(memory: Memory): AgentMemResult<String>
```

## 🚀 部署最佳实践

### 1. 配置管理

**✅ 推荐做法**:
```cangjie
public class ConfigManager {
    private var config: SimpleMap
    
    public init() {
        // 从环境变量或配置文件加载配置
        config = loadConfiguration()
    }
    
    public func getServerUrl(): String {
        return config.get("server_url") as String? ?? "https://api.agentmem.com"
    }
    
    public func getApiKey(): String {
        let apiKey = config.get("api_key") as String?
        if (apiKey == None || apiKey.isEmpty()) {
            throw RuntimeError("API密钥未配置")
        }
        return apiKey
    }
}
```

### 2. 健康检查

**✅ 推荐做法**:
```cangjie
public class HealthChecker {
    private var client: AgentMemClient
    
    public func checkHealth(): AgentMemResult<HealthStatus> {
        // 检查服务器连接
        match (client.getMemoryStats()) {
            case Ok(_) => {
                return Ok(HealthStatus.Healthy)
            }
            case Err(error) => {
                return Ok(HealthStatus.Unhealthy(error.getMessage()))
            }
        }
    }
}
```

---

**指南版本**: v1.0  
**最后更新**: 2024-09-25  
**适用版本**: AgentMem SDK v1.0+
