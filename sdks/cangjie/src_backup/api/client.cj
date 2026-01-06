/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 主客户端 - AgentMem 仓颉 SDK 的主要API接口
 */

package agentmem.api

import agentmem.core.*
import agentmem.ffi.*
import std.collection.HashMap

/// AgentMem 客户端 - 主要的API接口类
public class AgentMemClient {
    /// C客户端句柄
    private var _handle: Option<CPointer<Unit>> = None
    /// 客户端配置
    private var _config: ClientConfig
    /// 是否已初始化
    private var _initialized: Bool = false
    /// 内存管理器
    private var _memoryManager: FFIMemoryManager
    /// 性能监控器
    private var _performanceEnabled: Bool = false
    /// 缓存
    private var _cache: Option<SimpleCache<String, Memory>> = None

    /// 构造函数
    public init(config: ClientConfig) {
        this._config = config
        this._memoryManager = FFIMemoryManager(true)
        this._performanceEnabled = config.debugMode
        
        if (config.enableCache) {
            this._cache = Some(SimpleCache<String, Memory>(config.cacheSize))
        }
    }

    /// 初始化客户端
    public func initialize(): AgentMemResult<Unit> {
        if (this._initialized) {
            return AgentMemResult<Unit>.Ok(Unit())
        }

        // 验证配置
        if (!this._config.validate()) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.ConfigurationError("Invalid client configuration")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("client_initialize"))
        } else {
            None
        }

        try {
            // 序列化配置为JSON
            let configJson = this._config.toJson()
            let cConfigJson = CStringConverter.toCString(configJson)
            
            // 创建C客户端
            let handle = unsafe { agentmem_client_new(cConfigJson) }
            CStringConverter.freeCString(cConfigJson)

            if (handle.isNull()) {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Unit>.Err(error)
            }

            this._handle = Some(handle)
            this._memoryManager.registerPointer(handle)
            this._initialized = true

            if (monitor.isSome()) {
                monitor.getOrThrow().finishAndLog()
            }

            return AgentMemResult<Unit>.Ok(Unit())
            
        } catch (e: Exception) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Failed to initialize client: ${e}")
            )
        }
    }

    /// 关闭客户端
    public func close(): Unit {
        if (this._initialized && this._handle.isSome()) {
            let handle = this._handle.getOrThrow()
            
            unsafe { agentmem_client_destroy(handle) }
            this._memoryManager.unregisterPointer(handle)
            
            this._handle = None
            this._initialized = false
            
            if (this._cache.isSome()) {
                this._cache.getOrThrow().clear()
            }
        }
    }

    /// 检查客户端是否已连接
    public func isConnected(): Bool {
        if (!this._initialized || this._handle.isNone()) {
            return false
        }
        
        let handle = this._handle.getOrThrow()
        return unsafe { agentmem_client_is_connected(handle) }
    }

    /// 健康检查
    public func healthCheck(): AgentMemResult<Bool> {
        if (!this._initialized) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let handle = this._handle.getOrThrow()
        let isHealthy = unsafe { agentmem_health_check(handle) }
        return AgentMemResult<Bool>.Ok(isHealthy)
    }

    /// 添加记忆
    public func addMemory(memory: Memory): AgentMemResult<String> {
        if (!this._initialized) {
            return AgentMemResult<String>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("add_memory"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemory = MemoryConverter.toCMemory(memory)
            
            let resultId = unsafe { agentmem_add_memory(handle, CPointer.addressOf(cMemory)) }
            let memoryId = CStringConverter.toString(resultId)
            
            // 释放C内存
            unsafe { agentmem_free_string(resultId) }
            MemoryConverter.freeCMemory(cMemory)

            if (memoryId.isEmpty()) {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<String>.Err(error)
            }

            // 更新缓存
            if (this._cache.isSome()) {
                var updatedMemory = memory.clone()
                updatedMemory.id = memoryId
                this._cache.getOrThrow().put(memoryId, updatedMemory)
            }

            if (monitor.isSome()) {
                monitor.getOrThrow().finishAndLog()
            }

            return AgentMemResult<String>.Ok(memoryId)
            
        } catch (e: Exception) {
            return AgentMemResult<String>.Err(
                AgentMemError.InternalError("Failed to add memory: ${e}")
            )
        }
    }

    /// 获取记忆
    public func getMemory(memoryId: String): AgentMemResult<Option<Memory>> {
        if (!this._initialized) {
            return AgentMemResult<Option<Memory>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        // 检查缓存
        if (this._cache.isSome()) {
            let cached = this._cache.getOrThrow().get(memoryId)
            if (cached.isSome()) {
                return AgentMemResult<Option<Memory>>.Ok(cached)
            }
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("get_memory"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemoryId = CStringConverter.toCString(memoryId)
            var cMemory = CMemory()

            let result = unsafe { 
                agentmem_get_memory(handle, cMemoryId, CPointer.addressOf(cMemory))
            }
            CStringConverter.freeCString(cMemoryId)

            if (result == 0) {
                let memory = MemoryConverter.fromCMemory(cMemory)
                MemoryConverter.freeCMemory(cMemory)
                
                // 更新缓存
                if (this._cache.isSome()) {
                    this._cache.getOrThrow().put(memoryId, memory)
                }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<Option<Memory>>.Ok(Some(memory))
            } else if (result == -2) {
                return AgentMemResult<Option<Memory>>.Ok(None)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Option<Memory>>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<Option<Memory>>.Err(
                AgentMemError.InternalError("Failed to get memory: ${e}")
            )
        }
    }

    /// 更新记忆内容
    public func updateMemory(memoryId: String, content: String): AgentMemResult<Unit> {
        if (!this._initialized) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("update_memory"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemoryId = CStringConverter.toCString(memoryId)
            let cContent = CStringConverter.toCString(content)

            let result = unsafe { agentmem_update_memory(handle, cMemoryId, cContent) }
            CStringConverter.freeCString(cMemoryId)
            CStringConverter.freeCString(cContent)

            // 清除缓存中的旧数据
            if (this._cache.isSome()) {
                this._cache.getOrThrow().remove(memoryId)
            }

            if (monitor.isSome()) {
                monitor.getOrThrow().finishAndLog()
            }

            return FFIErrorHandler.checkResult(result)
            
        } catch (e: Exception) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Failed to update memory: ${e}")
            )
        }
    }

    /// 删除记忆
    public func deleteMemory(memoryId: String): AgentMemResult<Unit> {
        if (!this._initialized) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("delete_memory"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemoryId = CStringConverter.toCString(memoryId)

            let result = unsafe { agentmem_delete_memory(handle, cMemoryId) }
            CStringConverter.freeCString(cMemoryId)

            // 从缓存中移除
            if (this._cache.isSome()) {
                this._cache.getOrThrow().remove(memoryId)
            }

            if (monitor.isSome()) {
                monitor.getOrThrow().finishAndLog()
            }

            return FFIErrorHandler.checkResult(result)
            
        } catch (e: Exception) {
            return AgentMemResult<Unit>.Err(
                AgentMemError.InternalError("Failed to delete memory: ${e}")
            )
        }
    }

    /// 检查记忆是否存在
    public func memoryExists(memoryId: String): AgentMemResult<Bool> {
        if (!this._initialized) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        // 检查缓存
        if (this._cache.isSome()) {
            let cached = this._cache.getOrThrow().get(memoryId)
            if (cached.isSome()) {
                return AgentMemResult<Bool>.Ok(true)
            }
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemoryId = CStringConverter.toCString(memoryId)

            let exists = unsafe { agentmem_memory_exists(handle, cMemoryId) }
            CStringConverter.freeCString(cMemoryId)

            return AgentMemResult<Bool>.Ok(exists)
            
        } catch (e: Exception) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Failed to check memory existence: ${e}")
            )
        }
    }

    /// 获取客户端配置
    public func getConfig(): ClientConfig {
        return this._config.clone()
    }

    /// 获取SDK版本
    public func getVersion(): String {
        let versionCStr = unsafe { agentmem_get_version() }
        let version = CStringConverter.toString(versionCStr)
        unsafe { agentmem_free_string(versionCStr) }
        return version
    }

    /// 设置日志级别
    public func setLogLevel(level: LogLevel): Unit {
        unsafe { agentmem_set_log_level(level.toUInt32()) }
    }

    /// 搜索记忆
    public func searchMemories(query: String, limit: UInt32 = 10): AgentMemResult<Array<MemorySearchResult>> {
        if (!this._initialized) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("search_memories"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cQuery = CStringConverter.toCString(query)
            var cResults = CSearchResultArray()

            let result = unsafe {
                agentmem_search_memories(handle, cQuery, limit, CPointer.addressOf(cResults))
            }
            CStringConverter.freeCString(cQuery)

            if (result == 0) {
                let searchResults = SearchResultConverter.fromCSearchResultArray(cResults)
                SearchResultConverter.freeCSearchResultArray(cResults)

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<Array<MemorySearchResult>>.Ok(searchResults)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Array<MemorySearchResult>>.Err(error)
            }

        } catch (e: Exception) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Failed to search memories: ${e}")
            )
        }
    }

    /// 带过滤器的搜索记忆
    public func searchMemoriesFiltered(
        query: String,
        filter: SearchFilter,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        if (!this._initialized) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("search_memories_filtered"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cQuery = CStringConverter.toCString(query)
            let filterJson = JSONSerializer.serializeSearchFilter(filter)
            let cFilterJson = CStringConverter.toCString(filterJson)
            var cResults = CSearchResultArray()

            let result = unsafe {
                agentmem_search_memories_filtered(handle, cQuery, cFilterJson, limit, CPointer.addressOf(cResults))
            }
            CStringConverter.freeCString(cQuery)
            CStringConverter.freeCString(cFilterJson)

            if (result == 0) {
                let searchResults = SearchResultConverter.fromCSearchResultArray(cResults)
                SearchResultConverter.freeCSearchResultArray(cResults)

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<Array<MemorySearchResult>>.Ok(searchResults)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Array<MemorySearchResult>>.Err(error)
            }

        } catch (e: Exception) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Failed to search memories with filter: ${e}")
            )
        }
    }

    /// 搜索相似记忆
    public func searchSimilarMemories(
        memoryId: String,
        limit: UInt32 = 10,
        threshold: Float32 = 0.7
    ): AgentMemResult<Array<MemorySearchResult>> {
        if (!this._initialized) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("search_similar_memories"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cMemoryId = CStringConverter.toCString(memoryId)
            var cResults = CSearchResultArray()

            let result = unsafe {
                agentmem_search_similar_memories(handle, cMemoryId, limit, threshold, CPointer.addressOf(cResults))
            }
            CStringConverter.freeCString(cMemoryId)

            if (result == 0) {
                let searchResults = SearchResultConverter.fromCSearchResultArray(cResults)
                SearchResultConverter.freeCSearchResultArray(cResults)

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<Array<MemorySearchResult>>.Ok(searchResults)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Array<MemorySearchResult>>.Err(error)
            }

        } catch (e: Exception) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Failed to search similar memories: ${e}")
            )
        }
    }
}
