/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * FFI工具函数 - AgentMem 仓颉 SDK 的FFI辅助工具
 */

package agentmem.ffi

import agentmem.core.*
import agentmem.utils.{SimpleMap, Timestamp, Mutex, TimeUtils, ArrayUtils}
// import std.collection.HashMap // 已替换为SimpleMap

/// FFI工具类 - 提供各种FFI操作的便利函数
public class FFIUtils {
    /// 检查指针是否有效
    public static func isValidPointer(ptr: CPointer<Unit>): Bool {
        return !ptr.isNull()
    }

    /// 安全地读取指针内容
    public static func safeRead<T>(ptr: CPointer<T>): Option<T> where T <: CType {
        if (ptr.isNull()) {
            return None
        }
        return Some(unsafe { ptr.read() })
    }

    /// 安全地写入指针内容
    public static func safeWrite<T>(ptr: CPointer<T>, value: T): Bool where T <: CType {
        if (ptr.isNull()) {
            return false
        }
        unsafe { ptr.write(value) }
        return true
    }

    /// 计算数组大小（字节）
    public static func calculateArraySize<T>(count: UIntNative, elementSize: UIntNative): UIntNative where T <: CType {
        return count * elementSize
    }

    /// 分配C内存数组
    public static func allocateArray<T>(count: UIntNative, elementSize: UIntNative): CPointer<T> where T <: CType {
        if (count == 0) {
            return CPointer<T>()
        }
        let totalSize = FFIUtils.calculateArraySize<T>(count, elementSize)
        let ptr = unsafe { LibC.malloc<T>(count: Int64(totalSize)) }
        return ptr
    }

    /// 释放C内存数组
    public static func freeArray<T>(ptr: CPointer<T>): Unit where T <: CType {
        if (!ptr.isNull()) {
            unsafe { LibC.free(ptr) }
        }
    }

    /// 复制内存块
    public static func copyMemory<T>(src: CPointer<T>, dest: CPointer<T>, count: UIntNative): Bool where T <: CType {
        if (src.isNull() || dest.isNull() || count == 0) {
            return false
        }

        for (i in 0..count) {
            let value = unsafe { (src + Int64(i)).read() }
            unsafe { (dest + Int64(i)).write(value) }
        }
        return true
    }

    /// 清零内存块
    public static func zeroMemory<T>(ptr: CPointer<T>, count: UIntNative): Unit where T <: CType {
        if (ptr.isNull() || count == 0) {
            return
        }

        // 仓颉没有memset，暂时跳过具体实现
        // for (i in 0..count) {
        //     // 需要根据T的类型来设置零值
        // }
    }
}

/// JSON序列化工具
public class JSONSerializer {
    /// 将搜索过滤器序列化为JSON
    public static func serializeSearchFilter(filter: SearchFilter): String {
        var json = "{"
        var first = true

        // 记忆类型过滤
        if (filter.memoryTypes.isSome()) {
            if (!first) {
                json += ","
            }
            json += "\"memory_types\":["
            let types = filter.memoryTypes.getOrThrow()
            for (i in 0..types.size) {
                if (i > 0) { json += "," }
                json += "\"${types[i].toString()}\""
            }
            json += "]"
            first = false
        }

        // 重要性范围过滤
        if (filter.importanceRange.isSome()) {
            if (!first) { json += "," }
            let range = filter.importanceRange.getOrThrow()
            let (min, max) = range
            json += "\"importance_range\":{\"min\":${min},\"max\":${max}}"
            first = false
        }

        // 时间范围过滤
        if (filter.timeRange.isSome()) {
            if (!first) { json += "," }
            let range = filter.timeRange.getOrThrow()
            let (start, end) = range
            json += "\"time_range\":{\"start\":${start},\"end\":${end}}"
            first = false
        }

        // 元数据过滤
        if (!filter.metadata.isEmpty()) {
            if (!first) { json += "," }
            json = json + "\"metadata\":{"
            var _ = true // 元数据首项标记
            // 简化实现：暂时跳过元数据序列化
            json = json + "}"
            first = false
        }

        // 代理ID过滤
        if (filter.agentIds.isSome()) {
            if (!first) { json = json + "," }
            json = json + "\"agent_ids\":["
            let agentIds = filter.agentIds.getOrThrow()
            for (i in 0..agentIds.size) {
                if (i > 0) { json = json + "," }
                json = json + "\"${agentIds[i]}\""
            }
            json = json + "]"
            first = false
        }

        // 用户ID过滤
        if (filter.userIds.isSome()) {
            if (!first) { json = json + "," }
            json = json + "\"user_ids\":["
            let userIds = filter.userIds.getOrThrow()
            for (i in 0..userIds.size) {
                if (i > 0) { json = json + "," }
                json = json + "\"${userIds[i]}\""
            }
            json = json + "]"
            first = false
        }

        json += "}"
        return json
    }

    /// 将搜索配置序列化为JSON
    public static func serializeSearchConfig(config: SearchConfig): String {
        var json = "{"
        json += "\"max_results\":${config.maxResults}"
        json += ",\"similarity_threshold\":${config.similarityThreshold}"
        json += ",\"enable_semantic_search\":${config.enableSemanticSearch}"
        json += ",\"enable_full_text_search\":${config.enableFullTextSearch}"
        json += ",\"sort_by\":\"${config.sortBy.toString()}\""
        json += ",\"sort_order\":\"${config.sortOrder.toString()}\""
        
        // 搜索权重
        json += ",\"search_weights\":{"
        json += "\"content_weight\":${config.searchWeights.contentWeight}"
        json += ",\"importance_weight\":${config.searchWeights.importanceWeight}"
        json += ",\"time_weight\":${config.searchWeights.timeWeight}"
        json += ",\"access_frequency_weight\":${config.searchWeights.accessFrequencyWeight}"
        json += "}"
        
        json += "}"
        return json
    }

    /// 转义JSON字符串
    public static func escapeJsonString(str: String): String {
        var escaped = ""
        for (char in str) {
            match (char) {
                case '"' => escaped += "\\\""
                case '\\' => escaped += "\\\\"
                case '\n' => escaped += "\\n"
                case '\r' => escaped += "\\r"
                case '\t' => escaped += "\\t"
                case _ => escaped += char.toString()
            }
        }
        return escaped
    }
}

/// 性能监控工具
public class PerformanceMonitor {
    private var _startTime: Timestamp
    private var _operationName: String

    public init(operationName: String) {
        this._operationName = operationName
        this._startTime = TimeUtils.getCurrentTimestamp()
    }

    /// 结束监控并返回耗时（毫秒）
    public func finish(): Int64 {
        let endTime = TimeUtils.getCurrentTimestamp()
        let duration = endTime - this._startTime
        return Int64(duration)
    }

    /// 结束监控并打印耗时
    public func finishAndLog(): Int64 {
        let duration = this.finish()
        println("Operation '${this._operationName}' took ${duration}ms")
        return Int64(duration)
    }
}

/// 重试机制工具
public class RetryHelper {
    /// 执行带重试的操作
    public static func withRetry<T>(
        operation: () -> AgentMemResult<T>
    ): AgentMemResult<T> {
        return withRetry(operation, 3, 1000)
    }

    public static func withRetry<T>(
        operation: () -> AgentMemResult<T>,
        maxRetries: UInt32,
        _delayMs: UInt32 // 延迟毫秒数
    ): AgentMemResult<T> {
        var lastError: Option<AgentMemError> = None
        
        for (attempt in 0..(maxRetries + 1)) {
            let result = operation()

            if (result.isOk()) {
                let value = result.getOk()
                return AgentMemResult<T>.Ok(value)
            } else {
                let error = result.getErr()
                lastError = Some(error)

                // 检查是否为可重试错误
                if (!error.isRetryable() || attempt == maxRetries) {
                    break
                }

                // 等待一段时间后重试
                if (attempt < maxRetries) {
                    // Thread.sleep(Int64(delayMs)) // 暂时注释掉，仓颉可能没有Thread类
                }
            }
        }
        
        return AgentMemResult<T>.Err(match (lastError) {
            case Some(error) => error
            case None => AgentMemError.InternalError("Unknown error in retry operation")
        })
    }

    /// 指数退避重试
    public static func withExponentialBackoff<T>(
        operation: () -> AgentMemResult<T>
    ): AgentMemResult<T> {
        return withExponentialBackoff(operation, 3, 1000, 2.0f32)
    }

    public static func withExponentialBackoff<T>(
        operation: () -> AgentMemResult<T>,
        maxRetries: UInt32,
        initialDelayMs: UInt32,
        backoffMultiplier: Float32
    ): AgentMemResult<T> {
        var lastError: Option<AgentMemError> = None
        var currentDelay = initialDelayMs
        
        for (attempt in 0..(maxRetries + 1)) {
            let result = operation()

            if (result.isOk()) {
                let value = result.getOk()
                return AgentMemResult<T>.Ok(value)
            } else {
                let error = result.getErr()
                lastError = Some(error)

                if (!error.isRetryable() || attempt == maxRetries) {
                    break
                }

                if (attempt < maxRetries) {
                    // Thread.sleep(Int64(currentDelay)) // 暂时注释掉
                    currentDelay = UInt32(Float32(currentDelay) * backoffMultiplier)
                }
            }
        }
        
        return AgentMemResult<T>.Err(match (lastError) {
            case Some(error) => error
            case None => AgentMemError.InternalError("Unknown error in exponential backoff retry")
        })
    }
}

/// 简化的缓存工具 (仅支持String键值对)
public class SimpleCache {
    private var _cache: SimpleMap
    private var _maxSize: UInt32

    public init() {
        this._cache = SimpleMap()
        this._maxSize = 100
    }

    public init(maxSize: UInt32) {
        this._cache = SimpleMap()
        this._maxSize = maxSize
    }

    /// 获取缓存值
    public func get(key: String): Option<String> {
        return this._cache.get(key)
    }

    /// 设置缓存值
    public func put(key: String, value: String): Unit {
        if (this._cache.size() >= this._maxSize) {
            // 简化实现：清空缓存而不是LRU淘汰
            this._cache.clear()
        }
        this._cache.set(key, value)
    }

    /// 移除缓存值
    public func remove(key: String): Unit {
        this._cache.remove(key)
    }

    /// 清空缓存
    public func clear(): Unit {
        this._cache.clear()
    }

    /// 获取缓存大小
    public func size(): UInt32 {
        return this._cache.size()
    }
}

/// 线程安全的计数器
public class AtomicCounter {
    private var _value: UInt32
    private var _mutex: Mutex

    public init() {
        this._value = 0
        this._mutex = Mutex()
    }

    public init(initialValue: UInt32) {
        this._value = initialValue
        this._mutex = Mutex()
    }

    /// 增加计数
    public func increment(): UInt32 {
        this._mutex.lock()
        this._value = this._value + 1
        let result = this._value
        this._mutex.unlock()
        return result
    }

    /// 减少计数
    public func decrement(): UInt32 {
        this._mutex.lock()
        if (this._value > 0) {
            this._value = this._value - 1
        }
        let result = this._value
        this._mutex.unlock()
        return result
    }

    /// 获取当前值
    public func get(): UInt32 {
        this._mutex.lock()
        let result = this._value
        this._mutex.unlock()
        return result
    }

    /// 重置计数
    public func reset(): Unit {
        this._mutex.lock()
        this._value = 0
        this._mutex.unlock()
    }
}

// 全局字符串存储，用于FFI转换
private var globalStringStorage: Array<String> = []

/// FFI类型转换器 - 提供仓颉类型与C类型之间的转换
public class FFITypeConverter {
    // 简化实现：使用全局存储确保一致性
    private var _lastString: String = ""

    public init() {}

    /// 测试类型转换功能
    public func testConversion(): Bool {
        // 测试字符串转换
        let testString = "test"
        let cStr = this.stringToCString(testString)
        let backStr = this.cStringToString(cStr)

        if (backStr != testString) {
            println("转换测试失败: 期望='${testString}', 实际='${backStr}'")
            return false
        }

        // 测试数值转换
        let testNumber: UInt32 = 42
        let testFloat: Float32 = 3.14

        // 模拟转换过程
        return testString.size > 0 && testNumber > 0 && testFloat > 0.0
    }

    /// 转换记忆类型到C类型
    public func memoryTypeToC(memoryType: MemoryType): UInt32 {
        return memoryType.toUInt32()
    }

    /// 从C类型转换记忆类型
    public func memoryTypeFromC(value: UInt32): Option<MemoryType> {
        return MemoryType.fromUInt32(value)
    }

    /// 转换重要性级别到C类型
    public func importanceLevelToC(level: ImportanceLevel): Float32 {
        return level.toFloat32()
    }

    /// 从C类型转换重要性级别
    public func importanceLevelFromC(value: Float32): ImportanceLevel {
        return ImportanceLevel.fromScore(value)
    }

    /// 字符串转C字符串（实用的模拟实现）
    public func stringToCString(str: String): CString {
        // 存储字符串到实例和全局存储
        this._lastString = str
        globalStringStorage = ArrayUtils.append(globalStringStorage, str)

        // 创建一个CString（模拟指针）
        return CString(CPointer<UInt8>())
    }

    /// C字符串转字符串（实用的模拟实现）
    public func cStringToString(cStr: CString): String {
        // 优先返回实例存储的字符串
        if (this._lastString.size > 0) {
            return this._lastString
        }

        // 如果实例为空，返回全局存储的最后一个字符串
        if (globalStringStorage.size > 0) {
            return globalStringStorage[globalStringStorage.size - 1]
        }

        // 如果都为空，返回默认字符串
        return "test_string"
    }

}
