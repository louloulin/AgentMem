# AgentMem 仓颉 SDK API 参考文档

## 📚 目录

- [核心类型](#核心类型)
- [客户端API](#客户端api)
- [记忆管理](#记忆管理)
- [搜索功能](#搜索功能)
- [错误处理](#错误处理)
- [FFI绑定](#ffi绑定)
- [工具函数](#工具函数)

## 核心类型

### Memory

记忆对象的核心数据结构。

```cangjie
public class Memory {
    public let id: String                    // 记忆唯一标识符
    public let agentId: String              // 关联的智能体ID
    public let parentId: Option<String>     // 父记忆ID（可选）
    public let memoryType: MemoryType       // 记忆类型
    public let content: String              // 记忆内容
    public let importance: Float32          // 重要性级别
    public let embedding: Option<Array<Float32>>  // 向量嵌入（可选）
    public let createdAt: Int64            // 创建时间戳
    public let updatedAt: Int64            // 更新时间戳
    public let accessCount: Int64          // 访问次数
    public let metadata: Option<String>    // 元数据（可选）
    public let tags: SimpleMap             // 标签映射
    public let version: Int64              // 版本号
}
```

**构造函数**:
```cangjie
public init(
    id: String,
    agentId: String, 
    parentId: Option<String>,
    memoryType: MemoryType,
    content: String,
    importance: Float32,
    embedding: Option<Array<Float32>>,
    createdAt: Int64,
    updatedAt: Int64,
    accessCount: Int64,
    metadata: Option<String>,
    tags: SimpleMap,
    version: Int64
)
```

**方法**:
- `validate(): AgentMemResult<Bool>` - 验证记忆对象的有效性
- `toJson(): String` - 转换为JSON字符串
- `clone(): Memory` - 创建记忆对象的副本

### MemoryType

记忆类型枚举。

```cangjie
public enum MemoryType {
    | Episodic      // 情节记忆
    | Semantic      // 语义记忆
    | Procedural    // 程序记忆
    | Working       // 工作记忆
    | Declarative   // 陈述记忆
}
```

**方法**:
- `toUInt32(): UInt32` - 转换为数值表示
- `fromUInt32(value: UInt32): Option<MemoryType>` - 从数值创建

### ImportanceLevel

重要性级别枚举。

```cangjie
public enum ImportanceLevel {
    | Critical      // 关键 (1.0)
    | High          // 高 (0.8)
    | Medium        // 中 (0.6)
    | Low           // 低 (0.4)
    | Minimal       // 最低 (0.2)
}
```

**方法**:
- `toFloat32(): Float32` - 转换为浮点数值
- `fromFloat32(value: Float32): ImportanceLevel` - 从浮点数创建

## 客户端API

### AgentMemClient

主要的客户端类，提供所有记忆管理功能。

```cangjie
public class AgentMemClient {
    public init(config: AgentMemConfig)
}
```

**核心方法**:

#### 记忆操作
```cangjie
// 添加记忆
public func addMemory(memory: Memory): AgentMemResult<String>

// 获取记忆
public func getMemory(id: String): AgentMemResult<Memory>

// 更新记忆
public func updateMemory(memory: Memory): AgentMemResult<Bool>

// 删除记忆
public func deleteMemory(id: String): AgentMemResult<Bool>

// 批量添加记忆
public func addMemories(memories: Array<Memory>): AgentMemResult<Array<String>>
```

#### 搜索功能
```cangjie
// 搜索记忆
public func searchMemories(query: String, limit: UInt32): AgentMemResult<Array<Memory>>

// 语义搜索
public func semanticSearch(
    query: String, 
    limit: UInt32, 
    threshold: Float32
): AgentMemResult<Array<Memory>>

// 按类型搜索
public func searchByType(
    memoryType: MemoryType, 
    limit: UInt32
): AgentMemResult<Array<Memory>>
```

#### 统计信息
```cangjie
// 获取记忆统计
public func getMemoryStats(): AgentMemResult<MemoryStats>

// 获取智能体记忆数量
public func getMemoryCount(agentId: String): AgentMemResult<Int64>
```

### AgentMemClientBuilder

客户端构建器，用于配置和创建客户端实例。

```cangjie
public class AgentMemClientBuilder {
    public init()
    
    // 设置服务器URL
    public func withServerUrl(url: String): AgentMemClientBuilder
    
    // 设置API密钥
    public func withApiKey(key: String): AgentMemClientBuilder
    
    // 设置超时时间
    public func withTimeout(timeoutMs: UInt32): AgentMemClientBuilder
    
    // 启用重试机制
    public func withRetry(maxRetries: UInt32): AgentMemClientBuilder
    
    // 构建客户端
    public func build(): AgentMemResult<AgentMemClient>
}
```

## 记忆管理

### MemoryStats

记忆统计信息结构。

```cangjie
public struct MemoryStats {
    public let totalMemories: Int64         // 总记忆数量
    public let memoriesByType: SimpleMap    // 按类型分组的数量
    public let averageImportance: Float32   // 平均重要性
    public let oldestMemory: Int64         // 最早记忆时间戳
    public let newestMemory: Int64         // 最新记忆时间戳
}
```

### MemoryFilter

记忆过滤器，用于高级搜索。

```cangjie
public struct MemoryFilter {
    public let memoryType: Option<MemoryType>      // 记忆类型过滤
    public let minImportance: Option<Float32>      // 最小重要性
    public let maxImportance: Option<Float32>      // 最大重要性
    public let dateRange: Option<DateRange>        // 日期范围
    public let tags: Option<Array<String>>         // 标签过滤
}
```

## 搜索功能

### SearchService

搜索服务类，提供高级搜索功能。

```cangjie
public class SearchService {
    public init(client: AgentMemClient)
    
    // 复合搜索
    public func complexSearch(
        query: String,
        filter: MemoryFilter,
        limit: UInt32
    ): AgentMemResult<Array<Memory>>
    
    // 相似性搜索
    public func similaritySearch(
        targetMemory: Memory,
        limit: UInt32,
        threshold: Float32
    ): AgentMemResult<Array<Memory>>
}
```

## 错误处理

### AgentMemError

统一的错误类型枚举。

```cangjie
public enum AgentMemError {
    | InvalidParameter(String)      // 无效参数
    | NotFound(String)             // 未找到
    | NetworkError(String)         // 网络错误
    | SerializationError(String)   // 序列化错误
    | DatabaseError(String)        // 数据库错误
    | AuthenticationError(String)  // 认证错误
    | RateLimitError(String)       // 限流错误
    | InternalError(String)        // 内部错误
    | NotConnected(String)         // 未连接
}
```

**方法**:
- `getMessage(): String` - 获取错误消息
- `getErrorCode(): String` - 获取错误代码
- `isRetryable(): Bool` - 判断是否可重试

### AgentMemResult<T>

结果类型，用于错误处理。

```cangjie
public enum AgentMemResult<T> {
    | Ok(T)                    // 成功结果
    | Err(AgentMemError)       // 错误结果
}
```

**使用示例**:
```cangjie
match (client.getMemory("memory-001")) {
    case Ok(memory) => {
        println("获取记忆成功: ${memory.content}")
    }
    case Err(error) => {
        println("获取记忆失败: ${error.getMessage()}")
    }
}
```

## FFI绑定

### FFITypeConverter

FFI类型转换器，处理仓颉类型与C类型之间的转换。

```cangjie
public class FFITypeConverter {
    public init()
    
    // 字符串转换
    public func stringToCString(str: String): CString
    public func cStringToString(cStr: CString): String
    
    // 类型转换
    public func memoryTypeToC(memoryType: MemoryType): UInt32
    public func memoryTypeFromC(value: UInt32): Option<MemoryType>
    
    // 重要性级别转换
    public func importanceLevelToC(level: ImportanceLevel): Float32
    public func importanceLevelFromC(value: Float32): ImportanceLevel
    
    // 测试转换功能
    public func testConversion(): Bool
}
```

### FFIMemoryManager

FFI内存管理器，确保内存安全。

```cangjie
public class FFIMemoryManager {
    public init()
    
    // 分配内存
    public func allocate(size: USize): CPointer<UInt8>
    
    // 释放内存
    public func deallocate(ptr: CPointer<UInt8>)
    
    // 内存统计
    public func getMemoryUsage(): USize
    public func getActiveAllocations(): USize
}
```

## 工具函数

### TimeUtils

时间工具类。

```cangjie
public class TimeUtils {
    // 获取当前时间戳
    public static func getCurrentTimestamp(): Int64
    
    // 格式化时间戳
    public static func formatTimestamp(timestamp: Int64): String
    
    // 解析时间字符串
    public static func parseTimestamp(timeStr: String): Int64
}
```

### ValidationUtils

验证工具类。

```cangjie
public class ValidationUtils {
    // 验证记忆ID
    public static func isValidMemoryId(id: String): Bool
    
    // 验证智能体ID
    public static func isValidAgentId(id: String): Bool
    
    // 验证重要性级别
    public static func isValidImportance(importance: Float32): Bool
}
```

## 📝 使用最佳实践

### 1. 错误处理
```cangjie
// 推荐：使用match表达式处理结果
match (client.addMemory(memory)) {
    case Ok(id) => println("记忆添加成功，ID: ${id}")
    case Err(error) => {
        if (error.isRetryable()) {
            // 可重试的错误，实现重试逻辑
            retryAddMemory(memory)
        } else {
            // 不可重试的错误，记录日志
            logError(error.getMessage())
        }
    }
}
```

### 2. 资源管理
```cangjie
// 推荐：使用RAII模式管理资源
{
    let client = AgentMemClientBuilder()
        .withServerUrl("https://api.agentmem.com")
        .withApiKey("your-api-key")
        .build()
    
    match (client) {
        case Ok(c) => {
            // 使用客户端
            let result = c.addMemory(memory)
            // 客户端会在作用域结束时自动清理
        }
        case Err(error) => println("客户端创建失败: ${error.getMessage()}")
    }
}
```

### 3. 批量操作
```cangjie
// 推荐：使用批量操作提高性能
let memories = [memory1, memory2, memory3]
match (client.addMemories(memories)) {
    case Ok(ids) => println("批量添加成功，数量: ${ids.size}")
    case Err(error) => println("批量添加失败: ${error.getMessage()}")
}
```

---

**文档版本**: v1.0  
**最后更新**: 2024-09-25  
**兼容版本**: 仓颉 0.60.5+
