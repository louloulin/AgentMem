# AgentMem 仓颉 SDK API 参考文档

## 概述

AgentMem 仓颉 SDK 是一个企业级智能记忆管理平台的仓颉语言接口，提供了完整的记忆管理、搜索、分析和维护功能。

## 核心类型

### Memory 结构体

记忆的核心数据结构。

```cangjie
public struct Memory {
    public var id: String                    // 记忆唯一标识符
    public var agentId: String              // 所属Agent ID
    public var userId: String               // 所属用户ID
    public var sessionId: String            // 会话ID
    public var content: String              // 记忆内容
    public var memoryType: MemoryType       // 记忆类型
    public var importance: Float32          // 重要性 (0.0-1.0)
    public var createdAt: Timestamp         // 创建时间
    public var lastAccessedAt: Timestamp    // 最后访问时间
    public var accessCount: UInt32          // 访问次数
    public var metadata: HashMap<String, String>  // 元数据
    public var embedding: Vector            // 嵌入向量
}
```

### MemoryType 枚举

记忆类型分类。

```cangjie
public enum MemoryType {
    | Episodic    // 情景记忆：具体事件和经历
    | Semantic    // 语义记忆：概念和知识
    | Procedural  // 程序记忆：技能和过程
    | Working     // 工作记忆：临时信息

    public func toString(): String
    public func toUInt32(): UInt32
}
```

### ImportanceLevel 枚举

重要性级别。

```cangjie
public enum ImportanceLevel {
    | Low      // 低重要性 (0.2)
    | Medium   // 中等重要性 (0.5)
    | High     // 高重要性 (0.8)
    | Critical // 关键重要性 (1.0)

    public func toString(): String
    public func toFloat32(): Float32
}
```

## 主要类

### AgentMemClient

主客户端类，提供所有核心功能。

#### 构造函数

```cangjie
public init(config: ClientConfig)
```

#### 初始化和连接

```cangjie
// 初始化客户端
public func initialize(): AgentMemResult<Unit>

// 检查连接状态
public func isConnected(): Bool

// 健康检查
public func healthCheck(): AgentMemResult<Bool>

// 获取版本信息
public func getVersion(): String

// 关闭客户端
public func close(): Unit
```

#### 记忆管理

```cangjie
// 添加记忆
public func addMemory(memory: Memory): AgentMemResult<String>

// 获取记忆
public func getMemory(memoryId: String): AgentMemResult<Option<Memory>>

// 更新记忆
public func updateMemory(memoryId: String, newContent: String): AgentMemResult<Unit>

// 删除记忆
public func deleteMemory(memoryId: String): AgentMemResult<Unit>
```

#### 搜索功能

```cangjie
// 基础搜索
public func searchMemories(query: String, limit: UInt32): AgentMemResult<Array<MemorySearchResult>>

// 过滤搜索
public func searchMemoriesFiltered(
    query: String, 
    filter: SearchFilter, 
    limit: UInt32
): AgentMemResult<Array<MemorySearchResult>>

// 相似记忆搜索
public func searchSimilarMemories(
    memoryId: String, 
    limit: UInt32, 
    threshold: Float32
): AgentMemResult<Array<MemorySearchResult>>
```

#### 批量操作

```cangjie
// 批量添加记忆
public func addMemoriesBatch(memories: Array<Memory>): AgentMemResult<BatchResult<String>>

// 批量删除记忆
public func deleteMemoriesBatch(memoryIds: Array<String>): AgentMemResult<BatchResult<Unit>>

// 分页获取记忆
public func getMemoriesPaginated(
    agentId: String, 
    pagination: PaginationParams
): AgentMemResult<PaginatedResult<Memory>>
```

#### 统计和分析

```cangjie
// 获取记忆统计
public func getMemoryStats(agentId: String): AgentMemResult<MemoryStats>

// 获取全局统计
public func getGlobalStats(): AgentMemResult<MemoryStats>
```

#### 数据管理

```cangjie
// 压缩记忆
public func compressMemories(agentId: String, compressionRatio: Float32): AgentMemResult<Unit>

// 导出记忆
public func exportMemories(
    agentId: String, 
    format: String, 
    outputPath: String
): AgentMemResult<Unit>

// 导入记忆
public func importMemories(
    agentId: String, 
    format: String, 
    inputPath: String
): AgentMemResult<BatchResult<String>>
```

### MemoryBuilder

记忆构建器，提供链式API构建记忆对象。

```cangjie
public class MemoryBuilder {
    public init()
    
    // 设置基本信息
    public func withAgentId(agentId: String): MemoryBuilder
    public func withUserId(userId: String): MemoryBuilder
    public func withSessionId(sessionId: String): MemoryBuilder
    public func withContent(content: String): MemoryBuilder
    
    // 设置类型和重要性
    public func withMemoryType(memoryType: MemoryType): MemoryBuilder
    public func withImportance(importance: Float32): MemoryBuilder
    public func withImportanceLevel(level: ImportanceLevel): MemoryBuilder
    
    // 设置元数据和向量
    public func withMetadata(key: String, value: String): MemoryBuilder
    public func withEmbedding(embedding: Vector): MemoryBuilder
    
    // 构建记忆对象
    public func build(): Memory
}
```

### SearchManager

搜索管理器，提供高级搜索功能。

```cangjie
public class SearchManager {
    public init(client: AgentMemClient, config: SearchConfig = SearchConfig())
    
    // 智能搜索
    public func smartSearch(
        query: String, 
        filter: Option<SearchFilter> = None,
        limit: Option<UInt32> = None
    ): AgentMemResult<Array<MemorySearchResult>>
    
    // 语义搜索
    public func semanticSearch(
        query: String, 
        threshold: Float32 = 0.7,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>>
    
    // 多条件搜索
    public func advancedSearch(
        queries: Array<String>,
        operator: SearchOperator = SearchOperator.And,
        filter: Option<SearchFilter> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>>
    
    // 时间范围搜索
    public func searchByTimeRange(
        startTime: Timestamp,
        endTime: Timestamp,
        agentId: Option<String> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>>
    
    // 按重要性搜索
    public func searchByImportance(
        minImportance: Float32,
        maxImportance: Float32 = 1.0,
        agentId: Option<String> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>>
}
```

### AdminManager

管理功能管理器，提供系统管理和维护功能。

```cangjie
public class AdminManager {
    public init(client: AgentMemClient)
    
    // 系统健康检查
    public func systemHealthCheck(): AgentMemResult<SystemHealth>
    
    // 数据备份
    public func backupData(
        agentId: Option<String> = None,
        outputPath: String,
        format: BackupFormat = BackupFormat.Json
    ): AgentMemResult<BackupInfo>
    
    // 数据恢复
    public func restoreData(
        agentId: String,
        inputPath: String,
        format: BackupFormat = BackupFormat.Json,
        overwrite: Bool = false
    ): AgentMemResult<RestoreInfo>
    
    // 数据压缩
    public func compressData(
        agentId: String,
        compressionRatio: Float32 = 0.8
    ): AgentMemResult<CompressionInfo>
    
    // 数据清理
    public func cleanupData(
        agentId: String,
        criteria: CleanupCriteria
    ): AgentMemResult<CleanupInfo>
    
    // 性能监控
    public func getPerformanceMetrics(
        agentId: Option<String> = None
    ): AgentMemResult<PerformanceMetrics>
}
```

## 配置类

### ClientConfig

客户端配置。

```cangjie
public class ClientConfig {
    public init(serverUrl: String)
    
    // 链式配置方法
    public func withApiKey(apiKey: String): ClientConfig
    public func withTimeout(timeout: Int32): ClientConfig
    public func withRetryCount(retryCount: Int32): ClientConfig
    public func withCache(enabled: Bool, size: Int32): ClientConfig
    public func withDebugMode(enabled: Bool): ClientConfig
    public func withLogLevel(level: LogLevel): ClientConfig
}
```

### SearchConfig

搜索配置。

```cangjie
public struct SearchConfig {
    public var maxResults: UInt32 = 10
    public var similarityThreshold: Float32 = 0.7
    public var enableSemanticSearch: Bool = true
    public var enableFullTextSearch: Bool = true
    public var enableCache: Bool = true
}
```

## 结果类型

### AgentMemResult

统一的结果类型。

```cangjie
public enum AgentMemResult<T> {
    | Ok(T)
    | Err(AgentMemError)
}
```

### AgentMemError

错误类型。

```cangjie
public enum AgentMemError {
    | InvalidInput(String)
    | NetworkError(String)
    | AuthenticationError(String)
    | NotFound(String)
    | InternalError(String)
    | TimeoutError(String)
    
    public func getMessage(): String
}
```

### BatchResult

批量操作结果。

```cangjie
public struct BatchResult<T> {
    public var total: UInt32
    public var successes: Array<T>
    public var failures: Array<String>
    
    public func addSuccess(item: T): Unit
    public func addFailure(error: String): Unit
}
```

### PaginatedResult

分页结果。

```cangjie
public struct PaginatedResult<T> {
    public var data: Array<T>
    public var page: UInt32
    public var pageSize: UInt32
    public var totalCount: UInt32
    
    public init(data: Array<T>, page: UInt32, pageSize: UInt32, totalCount: UInt32)
}
```

## 使用示例

### 基础使用

```cangjie
// 1. 创建配置
let config = ClientConfig("http://localhost:8080")
    .withApiKey("your-api-key")
    .withTimeout(30)
    .withCache(true, 100)

// 2. 创建客户端
let client = AgentMemClient(config)
let initResult = client.initialize()

// 3. 添加记忆
let memory = MemoryBuilder()
    .withAgentId("my-agent")
    .withContent("这是一条测试记忆")
    .withMemoryType(MemoryType.Semantic)
    .withImportance(0.8)
    .build()

let addResult = client.addMemory(memory)

// 4. 搜索记忆
let searchResult = client.searchMemories("测试", 10)

// 5. 关闭客户端
client.close()
```

### 高级搜索

```cangjie
// 创建搜索管理器
let searchManager = SearchManager(client)

// 智能搜索
let smartResult = searchManager.smartSearch("人工智能", None, Some(5))

// 过滤搜索
var filter = SearchFilter()
filter.memoryTypes = Some([MemoryType.Semantic])
filter.importanceRange = Some((0.8, 1.0))

let filteredResult = searchManager.smartSearch("*", Some(filter), Some(10))
```

### 批量操作

```cangjie
// 批量添加
let memories = [memory1, memory2, memory3]
let batchResult = client.addMemoriesBatch(memories)

// 分页获取
let pagination = PaginationParams(1, 20)
let pageResult = client.getMemoriesPaginated("my-agent", pagination)
```

## 错误处理

所有API方法都返回 `AgentMemResult<T>` 类型，使用模式匹配处理结果：

```cangjie
match (result) {
    case Ok(value) => {
        // 处理成功结果
        println("操作成功: ${value}")
    }
    case Err(error) => {
        // 处理错误
        println("操作失败: ${error.getMessage()}")
    }
}
```

## 性能优化建议

1. **启用缓存**：对于频繁访问的数据启用客户端缓存
2. **批量操作**：使用批量API减少网络往返
3. **分页查询**：对于大量数据使用分页避免内存问题
4. **合理设置超时**：根据网络环境调整超时时间
5. **连接复用**：重用客户端连接避免频繁初始化

## 最佳实践

1. **记忆内容**：保持记忆内容简洁明确
2. **重要性设置**：合理设置重要性帮助搜索排序
3. **元数据使用**：充分利用元数据进行分类和过滤
4. **错误处理**：始终处理可能的错误情况
5. **资源清理**：使用完毕后及时关闭客户端连接
