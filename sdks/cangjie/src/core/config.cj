/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 配置类型定义 - AgentMem 仓颉 SDK 的配置管理
 */

package agentmem.core

// import std.collection.HashMap
import agentmem.utils.{SimpleMap, StringUtils}

/// 客户端配置
public class ClientConfig {
    /// 服务器URL
    public var serverUrl: String
    /// 代理ID
    public var agentId: String
    /// API密钥
    public var apiKey: Option<String>
    /// 超时时间（秒）
    public var timeout: UInt32
    /// 重试次数
    public var retryCount: UInt32
    /// 启用缓存
    public var enableCache: Bool
    /// 缓存大小（MB）
    public var cacheSize: UInt32
    /// 连接池大小
    public var connectionPoolSize: UInt32
    /// 启用压缩
    public var enableCompression: Bool
    /// 用户代理字符串
    public var userAgent: String
    /// 自定义头部
    public var customHeaders: SimpleMap
    /// 启用调试模式
    public var debugMode: Bool
    /// 日志级别
    public var logLevel: LogLevel

    /// 构造函数 - 基本配置
    public init(serverUrl: String, agentId: String) {
        this.serverUrl = serverUrl
        this.agentId = agentId
        this.apiKey = None
        this.timeout = 30
        this.retryCount = 3
        this.enableCache = true
        this.cacheSize = 100
        this.connectionPoolSize = 10
        this.enableCompression = true
        this.userAgent = "AgentMem-Cangjie-SDK/1.0.0"
        this.customHeaders = SimpleMap()
        this.debugMode = false
        this.logLevel = LogLevel.Info
    }

    /// 构造函数 - 完整配置
    public init(
        serverUrl: String,
        agentId: String,
        apiKey: Option<String>,
        timeout: UInt32,
        retryCount: UInt32,
        enableCache: Bool,
        cacheSize: UInt32,
        connectionPoolSize: UInt32,
        enableCompression: Bool,
        userAgent: String,
        customHeaders: SimpleMap,
        debugMode: Bool,
        logLevel: LogLevel
    ) {
        this.serverUrl = serverUrl
        this.agentId = agentId
        this.apiKey = apiKey
        this.timeout = timeout
        this.retryCount = retryCount
        this.enableCache = enableCache
        this.cacheSize = cacheSize
        this.connectionPoolSize = connectionPoolSize
        this.enableCompression = enableCompression
        this.userAgent = userAgent
        this.customHeaders = customHeaders
        this.debugMode = debugMode
        this.logLevel = logLevel
    }

    /// 设置API密钥
    public func withApiKey(apiKey: String): ClientConfig {
        this.apiKey = Some(apiKey)
        return this
    }

    /// 设置超时时间
    public func withTimeout(timeout: UInt32): ClientConfig {
        this.timeout = timeout
        return this
    }

    /// 设置重试次数
    public func withRetryCount(retryCount: UInt32): ClientConfig {
        this.retryCount = retryCount
        return this
    }

    /// 启用/禁用缓存
    public func withCache(enable: Bool, size: UInt32): ClientConfig {
        this.enableCache = enable
        this.cacheSize = size
        return this
    }

    /// 启用/禁用缓存（默认大小）
    public func withCache(enable: Bool): ClientConfig {
        this.enableCache = enable
        this.cacheSize = 100
        return this
    }

    /// 设置连接池大小
    public func withConnectionPoolSize(size: UInt32): ClientConfig {
        this.connectionPoolSize = size
        return this
    }

    /// 启用/禁用压缩
    public func withCompression(enable: Bool): ClientConfig {
        this.enableCompression = enable
        return this
    }

    /// 设置用户代理
    public func withUserAgent(userAgent: String): ClientConfig {
        this.userAgent = userAgent
        return this
    }

    /// 添加自定义头部
    public func withCustomHeader(key: String, value: String): ClientConfig {
        this.customHeaders.set(key, value)
        return this
    }

    /// 启用/禁用调试模式
    public func withDebugMode(enable: Bool): ClientConfig {
        this.debugMode = enable
        return this
    }

    /// 设置日志级别
    public func withLogLevel(level: LogLevel): ClientConfig {
        this.logLevel = level
        return this
    }

    /// 验证配置
    public func validate(): Bool {
        if (this.serverUrl.isEmpty()) {
            return false
        }
        if (this.timeout == 0) {
            return false
        }
        if (this.connectionPoolSize == 0) {
            return false
        }
        return true
    }

    /// 转换为JSON字符串（用于FFI）
    public func toJson(): String {
        var json = "{"
        json += "\"server_url\":\"${this.serverUrl}\""
        
        if (this.apiKey.isSome()) {
            json += ",\"api_key\":\"${this.apiKey.getOrThrow()}\""
        }
        
        json += ",\"timeout\":${this.timeout}"
        json += ",\"retry_count\":${this.retryCount}"
        json += ",\"enable_cache\":${this.enableCache}"
        json += ",\"cache_size\":${this.cacheSize}"
        json += ",\"connection_pool_size\":${this.connectionPoolSize}"
        json += ",\"enable_compression\":${this.enableCompression}"
        json += ",\"user_agent\":\"${this.userAgent}\""
        json += ",\"debug_mode\":${this.debugMode}"
        json += ",\"log_level\":\"${this.logLevel.toString()}\""
        
        if (!this.customHeaders.isEmpty()) {
            json += ",\"custom_headers\":"
            json += this.customHeaders.toString()
        }
        
        json += "}"
        return json
    }

    /// 克隆配置
    public func clone(): ClientConfig {
        let newHeaders = SimpleMap()
        // 简化：直接创建新的SimpleMap
        // for ((key, value) in this.customHeaders) {
        //     newHeaders.set(key, value)
        // }

        return ClientConfig(
            this.serverUrl,
            this.agentId,
            this.apiKey,
            this.timeout,
            this.retryCount,
            this.enableCache,
            this.cacheSize,
            this.connectionPoolSize,
            this.enableCompression,
            this.userAgent,
            newHeaders,
            this.debugMode,
            this.logLevel
        )
    }
}

/// 日志级别枚举
public enum LogLevel {
    | Trace      // 跟踪级别
    | Debug      // 调试级别
    | Info       // 信息级别
    | Warn       // 警告级别
    | Error      // 错误级别
    | Fatal      // 致命级别

    /// 转换为字符串
    public func toString(): String {
        return match (this) {
            case Trace => "trace"
            case Debug => "debug"
            case Info => "info"
            case Warn => "warn"
            case Error => "error"
            case Fatal => "fatal"
        }
    }

    /// 从字符串创建
    public static func fromString(s: String): Option<LogLevel> {
        return match (StringUtils.toLowerCase(s)) {
            case "trace" => Some(Trace)
            case "debug" => Some(Debug)
            case "info" => Some(Info)
            case "warn" => Some(Warn)
            case "error" => Some(Error)
            case "fatal" => Some(Fatal)
            case _ => None
        }
    }

    /// 转换为数值（用于FFI）
    public func toUInt32(): UInt32 {
        return match (this) {
            case Trace => 0
            case Debug => 1
            case Info => 2
            case Warn => 3
            case Error => 4
            case Fatal => 5
        }
    }

    /// 从数值创建（用于FFI）
    public static func fromUInt32(value: UInt32): Option<LogLevel> {
        return match (value) {
            case 0 => Some(Trace)
            case 1 => Some(Debug)
            case 2 => Some(Info)
            case 3 => Some(Warn)
            case 4 => Some(Error)
            case 5 => Some(Fatal)
            case _ => None
        }
    }
}

/// 搜索配置
public struct SearchConfig {
    /// 最大结果数量
    public var maxResults: UInt32
    /// 相似度阈值
    public var similarityThreshold: Float32
    /// 启用语义搜索
    public var enableSemanticSearch: Bool
    /// 启用全文搜索
    public var enableFullTextSearch: Bool
    /// 搜索权重配置
    public var searchWeights: SearchWeights
    /// 结果排序方式
    public var sortBy: SortBy
    /// 排序方向
    public var sortOrder: SortOrder

    public init() {
        this.maxResults = 10
        this.similarityThreshold = 0.7
        this.enableSemanticSearch = true
        this.enableFullTextSearch = true
        this.searchWeights = SearchWeights()
        this.sortBy = SortBy.Relevance
        this.sortOrder = SortOrder.Descending
    }
}

/// 搜索权重配置
public struct SearchWeights {
    /// 内容权重
    public var contentWeight: Float32
    /// 重要性权重
    public var importanceWeight: Float32
    /// 时间权重
    public var timeWeight: Float32
    /// 访问频率权重
    public var accessFrequencyWeight: Float32

    public init() {
        this.contentWeight = 0.6
        this.importanceWeight = 0.2
        this.timeWeight = 0.1
        this.accessFrequencyWeight = 0.1
    }
}

/// 排序方式枚举
public enum SortBy {
    | Relevance      // 相关性
    | Importance     // 重要性
    | CreatedTime    // 创建时间
    | AccessTime     // 访问时间
    | AccessCount    // 访问次数

    public func toString(): String {
        return match (this) {
            case Relevance => "relevance"
            case Importance => "importance"
            case CreatedTime => "created_time"
            case AccessTime => "access_time"
            case AccessCount => "access_count"
        }
    }
}

/// 排序方向枚举
public enum SortOrder {
    | Ascending     // 升序
    | Descending    // 降序

    public func toString(): String {
        return match (this) {
            case Ascending => "asc"
            case Descending => "desc"
        }
    }
}

/// 缓存配置
public struct CacheConfig {
    /// 启用缓存
    public var enabled: Bool
    /// 缓存大小（MB）
    public var sizeInMB: UInt32
    /// 缓存过期时间（秒）
    public var ttlInSeconds: UInt32
    /// 最大缓存项数量
    public var maxItems: UInt32
    /// 缓存策略
    public var evictionPolicy: EvictionPolicy

    public init() {
        this.enabled = true
        this.sizeInMB = 100
        this.ttlInSeconds = 3600  // 1小时
        this.maxItems = 1000
        this.evictionPolicy = EvictionPolicy.LRU
    }
}

/// 缓存淘汰策略
public enum EvictionPolicy {
    | LRU       // 最近最少使用
    | LFU       // 最少使用频率
    | FIFO      // 先进先出
    | Random    // 随机淘汰

    public func toString(): String {
        return match (this) {
            case LRU => "lru"
            case LFU => "lfu"
            case FIFO => "fifo"
            case Random => "random"
        }
    }
}
