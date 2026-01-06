/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 记忆操作 - AgentMem 仓颉 SDK 的记忆管理操作
 */

package agentmem.api

import agentmem.core.*
import agentmem.ffi.*

/// 记忆操作扩展 - 为AgentMemClient提供高级记忆操作功能
extend AgentMemClient {
    /// 批量添加记忆
    public func addMemoriesBatch(memories: Array<Memory>): AgentMemResult<BatchResult<String>> {
        if (!this._initialized) {
            return AgentMemResult<BatchResult<String>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        if (memories.isEmpty()) {
            let emptyResult = BatchResult<String>()
            return AgentMemResult<BatchResult<String>>.Ok(emptyResult)
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("add_memories_batch"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            
            // 转换为C内存数组
            let count = UIntNative(memories.size)
            let cMemories = FFIUtils.allocateArray<CMemory>(count, UIntNative(sizeof(CMemory)))
            
            for (i in 0..memories.size) {
                let cMemory = MemoryConverter.toCMemory(memories[i])
                unsafe { (cMemories + UIntNative(i)).write(cMemory) }
            }

            var cBatchResult = CBatchResult()
            let result = unsafe {
                agentmem_add_memories_batch(handle, cMemories, count, CPointer.addressOf(cBatchResult))
            }

            // 释放C内存
            for (i in 0..memories.size) {
                let cMemory = unsafe { (cMemories + UIntNative(i)).read() }
                MemoryConverter.freeCMemory(cMemory)
            }
            FFIUtils.freeArray(cMemories)

            if (result == 0) {
                let batchResult = this.convertCBatchResult(cBatchResult)
                unsafe { agentmem_free_batch_result(CPointer.addressOf(cBatchResult)) }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<BatchResult<String>>.Ok(batchResult)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<BatchResult<String>>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<BatchResult<String>>.Err(
                AgentMemError.InternalError("Failed to add memories batch: ${e}")
            )
        }
    }

    /// 批量删除记忆
    public func deleteMemoriesBatch(memoryIds: Array<String>): AgentMemResult<BatchResult<Unit>> {
        if (!this._initialized) {
            return AgentMemResult<BatchResult<Unit>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        if (memoryIds.isEmpty()) {
            let emptyResult = BatchResult<Unit>()
            return AgentMemResult<BatchResult<Unit>>.Ok(emptyResult)
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("delete_memories_batch"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            
            // 转换为C字符串数组
            let cStringArray = CStringConverter.createCStringArray(memoryIds)
            let count = UIntNative(memoryIds.size)

            var cBatchResult = CBatchResult()
            let result = unsafe {
                agentmem_delete_memories_batch(handle, cStringArray, count, CPointer.addressOf(cBatchResult))
            }

            // 释放C字符串数组
            CStringConverter.freeCStringArray(cStringArray, count)

            if (result == 0) {
                let batchResult = this.convertCBatchResultUnit(cBatchResult)
                unsafe { agentmem_free_batch_result(CPointer.addressOf(cBatchResult)) }

                // 从缓存中移除删除的记忆
                if (this._cache.isSome()) {
                    let cache = this._cache.getOrThrow()
                    for (memoryId in memoryIds) {
                        cache.remove(memoryId)
                    }
                }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<BatchResult<Unit>>.Ok(batchResult)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<BatchResult<Unit>>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<BatchResult<Unit>>.Err(
                AgentMemError.InternalError("Failed to delete memories batch: ${e}")
            )
        }
    }

    /// 分页获取记忆
    public func getMemoriesPaginated(
        agentId: String, 
        pagination: PaginationParams
    ): AgentMemResult<PaginatedResult<Memory>> {
        if (!this._initialized) {
            return AgentMemResult<PaginatedResult<Memory>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("get_memories_paginated"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cAgentId = CStringConverter.toCString(agentId)
            var cPaginatedResult = CPaginatedResult()

            let result = unsafe {
                agentmem_get_memories_paginated(
                    handle, 
                    cAgentId, 
                    pagination.page, 
                    pagination.pageSize, 
                    CPointer.addressOf(cPaginatedResult)
                )
            }
            CStringConverter.freeCString(cAgentId)

            if (result == 0) {
                let paginatedResult = this.convertCPaginatedResult(cPaginatedResult)
                unsafe { agentmem_free_paginated_result(CPointer.addressOf(cPaginatedResult)) }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<PaginatedResult<Memory>>.Ok(paginatedResult)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<PaginatedResult<Memory>>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<PaginatedResult<Memory>>.Err(
                AgentMemError.InternalError("Failed to get memories paginated: ${e}")
            )
        }
    }

    /// 按类型分页获取记忆
    public func getMemoriesByTypePaginated(
        agentId: String, 
        memoryType: MemoryType, 
        pagination: PaginationParams
    ): AgentMemResult<PaginatedResult<Memory>> {
        if (!this._initialized) {
            return AgentMemResult<PaginatedResult<Memory>>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("get_memories_by_type_paginated"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cAgentId = CStringConverter.toCString(agentId)
            var cPaginatedResult = CPaginatedResult()

            let result = unsafe {
                agentmem_get_memories_by_type_paginated(
                    handle, 
                    cAgentId, 
                    memoryType.toUInt32(), 
                    pagination.page, 
                    pagination.pageSize, 
                    CPointer.addressOf(cPaginatedResult)
                )
            }
            CStringConverter.freeCString(cAgentId)

            if (result == 0) {
                let paginatedResult = this.convertCPaginatedResult(cPaginatedResult)
                unsafe { agentmem_free_paginated_result(CPointer.addressOf(cPaginatedResult)) }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<PaginatedResult<Memory>>.Ok(paginatedResult)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<PaginatedResult<Memory>>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<PaginatedResult<Memory>>.Err(
                AgentMemError.InternalError("Failed to get memories by type paginated: ${e}")
            )
        }
    }

    /// 获取记忆统计信息
    public func getMemoryStats(agentId: String): AgentMemResult<MemoryStats> {
        if (!this._initialized) {
            return AgentMemResult<MemoryStats>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("get_memory_stats"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            let cAgentId = CStringConverter.toCString(agentId)
            var cStats = CMemoryStats()

            let result = unsafe {
                agentmem_get_memory_stats(handle, cAgentId, CPointer.addressOf(cStats))
            }
            CStringConverter.freeCString(cAgentId)

            if (result == 0) {
                let stats = this.convertCMemoryStats(cStats)
                unsafe { agentmem_free_memory_stats(CPointer.addressOf(cStats)) }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<MemoryStats>.Ok(stats)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<MemoryStats>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<MemoryStats>.Err(
                AgentMemError.InternalError("Failed to get memory stats: ${e}")
            )
        }
    }

    /// 获取全局统计信息
    public func getGlobalStats(): AgentMemResult<MemoryStats> {
        if (!this._initialized) {
            return AgentMemResult<MemoryStats>.Err(
                AgentMemError.InternalError("Client not initialized")
            )
        }

        let monitor = if (this._performanceEnabled) {
            Some(PerformanceMonitor("get_global_stats"))
        } else {
            None
        }

        try {
            let handle = this._handle.getOrThrow()
            var cStats = CMemoryStats()

            let result = unsafe {
                agentmem_get_global_stats(handle, CPointer.addressOf(cStats))
            }

            if (result == 0) {
                let stats = this.convertCMemoryStats(cStats)
                unsafe { agentmem_free_memory_stats(CPointer.addressOf(cStats)) }

                if (monitor.isSome()) {
                    monitor.getOrThrow().finishAndLog()
                }

                return AgentMemResult<MemoryStats>.Ok(stats)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<MemoryStats>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<MemoryStats>.Err(
                AgentMemError.InternalError("Failed to get global stats: ${e}")
            )
        }
    }

    // 私有辅助方法
    private func convertCBatchResult(cResult: CBatchResult): BatchResult<String> {
        var batchResult = BatchResult<String>()
        batchResult.total = cResult.total_count
        
        // 这里简化处理，实际应该从C结果中提取成功的ID列表
        for (i in 0..cResult.success_count) {
            batchResult.addSuccess("success_${i}")
        }
        
        // 提取错误消息
        if (!cResult.error_messages.isNull() && cResult.error_count > 0) {
            for (i in 0..cResult.error_count) {
                let errorCStr = unsafe { (cResult.error_messages + i).read() }
                let errorMsg = CStringConverter.toString(errorCStr)
                batchResult.addFailure(errorMsg)
            }
        }
        
        return batchResult
    }

    private func convertCBatchResultUnit(cResult: CBatchResult): BatchResult<Unit> {
        var batchResult = BatchResult<Unit>()
        batchResult.total = cResult.total_count
        
        for (i in 0..cResult.success_count) {
            batchResult.addSuccess(Unit())
        }
        
        if (!cResult.error_messages.isNull() && cResult.error_count > 0) {
            for (i in 0..cResult.error_count) {
                let errorCStr = unsafe { (cResult.error_messages + i).read() }
                let errorMsg = CStringConverter.toString(errorCStr)
                batchResult.addFailure(errorMsg)
            }
        }
        
        return batchResult
    }

    private func convertCPaginatedResult(cResult: CPaginatedResult): PaginatedResult<Memory> {
        var memories = Array<Memory>()
        
        if (!cResult.memories.memories.isNull() && cResult.memories.count > 0) {
            for (i in 0..cResult.memories.count) {
                let cMemory = unsafe { (cResult.memories.memories + i).read() }
                let memory = MemoryConverter.fromCMemory(cMemory)
                memories.append(memory)
            }
        }
        
        return PaginatedResult<Memory>(
            memories,
            cResult.page,
            cResult.page_size,
            cResult.total_count
        )
    }

    private func convertCMemoryStats(cStats: CMemoryStats): MemoryStats {
        var stats = MemoryStats()
        stats.totalMemories = cStats.total_memories
        stats.averageImportance = cStats.average_importance
        stats.maxImportance = cStats.max_importance
        stats.minImportance = cStats.min_importance
        stats.lastCreatedAt = cStats.last_created_at
        stats.lastAccessedAt = cStats.last_accessed_at
        
        // 转换类型计数
        for (i in 0..4) {
            let memoryType = MemoryType.fromUInt32(UInt32(i)).getOrElse(MemoryType.Semantic)
            let count = unsafe { cStats.type_counts[i] }
            stats.memoryTypeCount[memoryType] = count
        }
        
        return stats
    }
}
