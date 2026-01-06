/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * FFI工具函数 - AgentMem 仓颉 SDK 的FFI辅助工具
 */

package agentmem.ffi

import agentmem.core.*
import std.collection.HashMap

/// FFI工具类 - 提供各种FFI操作的便利函数
public class FFIUtils {
    /// 检查指针是否有效
    public static func isValidPointer(ptr: CPointer<Unit>): Bool {
        return !ptr.isNull()
    }

    /// 安全地读取指针内容
    public static func safeRead<T>(ptr: CPointer<T>): Option<T> {
        if (ptr.isNull()) {
            return None
        }
        return Some(unsafe { ptr.read() })
    }

    /// 安全地写入指针内容
    public static func safeWrite<T>(ptr: CPointer<T>, value: T): Bool {
        if (ptr.isNull()) {
            return false
        }
        unsafe { ptr.write(value) }
        return true
    }

    /// 计算数组大小（字节）
    public static func calculateArraySize<T>(count: UIntNative, elementSize: UIntNative): UIntNative {
        return count * elementSize
    }

    /// 分配C内存数组
    public static func allocateArray<T>(count: UIntNative, elementSize: UIntNative): CPointer<T> {
        if (count == 0) {
            return CPointer<T>()
        }
        let totalSize = FFIUtils.calculateArraySize(count, elementSize)
        let ptr = unsafe { LibC.malloc(totalSize) }
        return unsafe { CPointer<T>(ptr) }
    }

    /// 释放C内存数组
    public static func freeArray<T>(ptr: CPointer<T>): Unit {
        if (!ptr.isNull()) {
            unsafe { LibC.free(ptr) }
        }
    }

    /// 复制内存块
    public static func copyMemory<T>(src: CPointer<T>, dest: CPointer<T>, count: UIntNative): Bool {
        if (src.isNull() || dest.isNull() || count == 0) {
            return false
        }
        
        for (i in 0..count) {
            let value = unsafe { (src + i).read() }
            unsafe { (dest + i).write(value) }
        }
        return true
    }

    /// 清零内存块
    public static func zeroMemory<T>(ptr: CPointer<T>, count: UIntNative): Unit {
        if (ptr.isNull() || count == 0) {
            return
        }
        
        // 这里需要根据T的类型来设置零值
        // 简化实现，假设T是数值类型
        for (i in 0..count) {
            unsafe { LibC.memset(ptr + i, 0, 1) }
        }
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
            if (!first) { json += "," }
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
            json += "\"importance_range\":{\"min\":${range.0},\"max\":${range.1}}"
            first = false
        }

        // 时间范围过滤
        if (filter.timeRange.isSome()) {
            if (!first) { json += "," }
            let range = filter.timeRange.getOrThrow()
            json += "\"time_range\":{\"start\":${range.0},\"end\":${range.1}}"
            first = false
        }

        // 元数据过滤
        if (!filter.metadata.isEmpty()) {
            if (!first) { json += "," }
            json += "\"metadata\":{"
            var metaFirst = true
            for ((key, value) in filter.metadata) {
                if (!metaFirst) { json += "," }
                json += "\"${key}\":\"${value}\""
                metaFirst = false
            }
            json += "}"
            first = false
        }

        // 代理ID过滤
        if (filter.agentIds.isSome()) {
            if (!first) { json += "," }
            json += "\"agent_ids\":["
            let agentIds = filter.agentIds.getOrThrow()
            for (i in 0..agentIds.size) {
                if (i > 0) { json += "," }
                json += "\"${agentIds[i]}\""
            }
            json += "]"
            first = false
        }

        // 用户ID过滤
        if (filter.userIds.isSome()) {
            if (!first) { json += "," }
            json += "\"user_ids\":["
            let userIds = filter.userIds.getOrThrow()
            for (i in 0..userIds.size) {
                if (i > 0) { json += "," }
                json += "\"${userIds[i]}\""
            }
            json += "]"
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
        this._startTime = getCurrentTimestamp()
    }

    /// 结束监控并返回耗时（毫秒）
    public func finish(): Int64 {
        let endTime = getCurrentTimestamp()
        let duration = endTime - this._startTime
        return duration
    }

    /// 结束监控并打印耗时
    public func finishAndLog(): Int64 {
        let duration = this.finish()
        println("Operation '${this._operationName}' took ${duration}ms")
        return duration
    }
}

/// 重试机制工具
public class RetryHelper {
    /// 执行带重试的操作
    public static func withRetry<T>(
        operation: () -> AgentMemResult<T>,
        maxRetries: UInt32 = 3,
        delayMs: UInt32 = 1000
    ): AgentMemResult<T> {
        var lastError: Option<AgentMemError> = None
        
        for (attempt in 0..(maxRetries + 1)) {
            let result = operation()
            
            match (result) {
                case Ok(value) => return AgentMemResult<T>.Ok(value)
                case Err(error) => {
                    lastError = Some(error)
                    
                    // 检查是否为可重试错误
                    if (!error.isRetryable() || attempt == maxRetries) {
                        break
                    }
                    
                    // 等待一段时间后重试
                    if (attempt < maxRetries) {
                        Thread.sleep(Int64(delayMs))
                    }
                }
            }
        }
        
        return AgentMemResult<T>.Err(lastError.getOrElse(
            AgentMemError.InternalError("Unknown error in retry operation")
        ))
    }

    /// 指数退避重试
    public static func withExponentialBackoff<T>(
        operation: () -> AgentMemResult<T>,
        maxRetries: UInt32 = 3,
        initialDelayMs: UInt32 = 1000,
        backoffMultiplier: Float32 = 2.0
    ): AgentMemResult<T> {
        var lastError: Option<AgentMemError> = None
        var currentDelay = initialDelayMs
        
        for (attempt in 0..(maxRetries + 1)) {
            let result = operation()
            
            match (result) {
                case Ok(value) => return AgentMemResult<T>.Ok(value)
                case Err(error) => {
                    lastError = Some(error)
                    
                    if (!error.isRetryable() || attempt == maxRetries) {
                        break
                    }
                    
                    if (attempt < maxRetries) {
                        Thread.sleep(Int64(currentDelay))
                        currentDelay = UInt32(Float32(currentDelay) * backoffMultiplier)
                    }
                }
            }
        }
        
        return AgentMemResult<T>.Err(lastError.getOrElse(
            AgentMemError.InternalError("Unknown error in exponential backoff retry")
        ))
    }
}

/// 缓存工具
public class SimpleCache<K, V> {
    private var _cache: HashMap<K, V>
    private var _maxSize: UInt32
    private var _accessOrder: Array<K>

    public init(maxSize: UInt32 = 100) {
        this._cache = HashMap<K, V>()
        this._maxSize = maxSize
        this._accessOrder = Array<K>()
    }

    /// 获取缓存值
    public func get(key: K): Option<V> {
        let value = this._cache.get(key)
        if (value.isSome()) {
            this.updateAccessOrder(key)
        }
        return value
    }

    /// 设置缓存值
    public func put(key: K, value: V): Unit {
        if (this._cache.containsKey(key)) {
            this._cache[key] = value
            this.updateAccessOrder(key)
        } else {
            if (UInt32(this._cache.size) >= this._maxSize) {
                this.evictLRU()
            }
            this._cache[key] = value
            this._accessOrder.append(key)
        }
    }

    /// 移除缓存值
    public func remove(key: K): Unit {
        this._cache.remove(key)
        this.removeFromAccessOrder(key)
    }

    /// 清空缓存
    public func clear(): Unit {
        this._cache.clear()
        this._accessOrder.clear()
    }

    /// 获取缓存大小
    public func size(): UInt32 {
        return UInt32(this._cache.size)
    }

    /// 更新访问顺序
    private func updateAccessOrder(key: K): Unit {
        this.removeFromAccessOrder(key)
        this._accessOrder.append(key)
    }

    /// 从访问顺序中移除
    private func removeFromAccessOrder(key: K): Unit {
        var newOrder = Array<K>()
        for (k in this._accessOrder) {
            if (k != key) {
                newOrder.append(k)
            }
        }
        this._accessOrder = newOrder
    }

    /// 淘汰最近最少使用的项
    private func evictLRU(): Unit {
        if (!this._accessOrder.isEmpty()) {
            let lruKey = this._accessOrder[0]
            this.remove(lruKey)
        }
    }
}

/// 线程安全的计数器
public class AtomicCounter {
    private var _value: UInt32
    private var _mutex: Mutex

    public init(initialValue: UInt32 = 0) {
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
