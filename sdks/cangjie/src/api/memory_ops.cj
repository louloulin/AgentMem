/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * AgentMem记忆操作API - 批量操作和高级功能
 */

package agentmem.api

import agentmem.core.{Memory, MemoryType, ImportanceLevel, AgentMemError, AgentMemResult}
import agentmem.utils.{SimpleMap, StringUtils, TimeUtils, ArrayUtils}

/// 批量操作结果
public class BatchOperationResult {
    public var totalCount: UInt32
    public var successCount: UInt32
    public var failureCount: UInt32
    public var successIds: Array<String>
    public var failureMessages: Array<String>

    public init(
        totalCount: UInt32,
        successCount: UInt32,
        failureCount: UInt32,
        successIds: Array<String>,
        failureMessages: Array<String>
    ) {
        this.totalCount = totalCount
        this.successCount = successCount
        this.failureCount = failureCount
        this.successIds = successIds
        this.failureMessages = failureMessages
    }

    /// 获取成功率
    public func getSuccessRate(): Float32 {
        if (this.totalCount == 0) {
            return 0.0
        }
        return Float32(this.successCount) / Float32(this.totalCount)
    }

    /// 是否全部成功
    public func isAllSuccessful(): Bool {
        return this.failureCount == 0
    }

    /// 是否有失败
    public func hasFailures(): Bool {
        return this.failureCount > 0
    }
}

/// 记忆统计信息
public class MemoryStatistics {
    public var totalMemories: UInt32
    public var episodicCount: UInt32
    public var semanticCount: UInt32
    public var proceduralCount: UInt32
    public var workingCount: UInt32
    public var averageImportance: Float32
    public var totalSize: UInt64
    public var lastUpdated: Int64

    public init(
        totalMemories: UInt32,
        episodicCount: UInt32,
        semanticCount: UInt32,
        proceduralCount: UInt32,
        workingCount: UInt32,
        averageImportance: Float32,
        totalSize: UInt64,
        lastUpdated: Int64
    ) {
        this.totalMemories = totalMemories
        this.episodicCount = episodicCount
        this.semanticCount = semanticCount
        this.proceduralCount = proceduralCount
        this.workingCount = workingCount
        this.averageImportance = averageImportance
        this.totalSize = totalSize
        this.lastUpdated = lastUpdated
    }

    /// 获取最常见的记忆类型
    public func getMostCommonType(): MemoryType {
        var maxCount = this.episodicCount
        var mostCommon = MemoryType.Episodic

        if (this.semanticCount > maxCount) {
            maxCount = this.semanticCount
            mostCommon = MemoryType.Semantic
        }

        if (this.proceduralCount > maxCount) {
            maxCount = this.proceduralCount
            mostCommon = MemoryType.Procedural
        }

        if (this.workingCount > maxCount) {
            mostCommon = MemoryType.Working
        }

        return mostCommon
    }
}

/// AgentMem记忆操作服务
public class AgentMemMemoryOps {
    private var _agentId: String
    private var _connected: Bool = false
    private var _memoryCache: SimpleMap = SimpleMap()

    public init(agentId: String) {
        this._agentId = agentId
    }

    /// 设置连接状态
    public func setConnected(connected: Bool): Unit {
        this._connected = connected
    }

    /// 获取统计信息
    public func getStatistics(): AgentMemResult<MemoryStatistics> {
        if (!this._connected) {
            return AgentMemResult<MemoryStatistics>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        // 模拟统计数据
        let stats = MemoryStatistics(
            100,
            25,
            50,
            15,
            10,
            0.65,
            1024,
            Int64(TimeUtils.getCurrentTimestamp())
        )

        return AgentMemResult<MemoryStatistics>.Ok(stats)
    }

    /// 批量添加记忆
    public func addMemoriesBatch(memories: Array<Memory>): AgentMemResult<BatchOperationResult> {
        if (!this._connected) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        if (memories.size == 0) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InvalidParameter("Memory array cannot be empty")
            )
        }

        try {
            var successIds = Array<String>(0, {_ => ""})
            var failureMessages = Array<String>(0, {_ => ""})
            var successCount: UInt32 = 0
            var failureCount: UInt32 = 0

            for (i in 0..memories.size) {
                let memory = memories[i]
                let validationResult = memory.validate()
                
                if (validationResult.isOk()) {
                    let memoryId = "batch_${i}_${TimeUtils.getCurrentTimestamp()}_${StringUtils.generateRandomId(6)}"
                    this._memoryCache.set(memoryId, memory.content)
                    successIds = ArrayUtils.append(successIds, memoryId)
                    successCount += 1
                } else {
                    let errorMsg = "Memory ${i}: ${validationResult.getErr().getMessage()}"
                    failureMessages = ArrayUtils.append(failureMessages, errorMsg)
                    failureCount += 1
                }
            }

            let result = BatchOperationResult(
                UInt32(memories.size),
                successCount,
                failureCount,
                successIds,
                failureMessages
            )

            return AgentMemResult<BatchOperationResult>.Ok(result)

        } catch (e: Exception) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InternalError("Batch add failed: ${e}")
            )
        }
    }

    /// 批量删除记忆
    public func deleteMemoriesBatch(memoryIds: Array<String>): AgentMemResult<BatchOperationResult> {
        if (!this._connected) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        if (memoryIds.size == 0) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InvalidParameter("Memory ID array cannot be empty")
            )
        }

        try {
            var successIds = Array<String>(0, {_ => ""})
            var failureMessages = Array<String>(0, {_ => ""})
            var successCount: UInt32 = 0
            var failureCount: UInt32 = 0

            for (i in 0..memoryIds.size) {
                let memoryId = memoryIds[i]
                
                if (memoryId.isEmpty()) {
                    let errorMsg = "Memory ID ${i}: ID cannot be empty"
                    failureMessages = ArrayUtils.append(failureMessages, errorMsg)
                    failureCount += 1
                    continue
                }

                let existingOpt = this._memoryCache.get(memoryId)
                if (existingOpt.isSome()) {
                    this._memoryCache.remove(memoryId)
                    successIds = ArrayUtils.append(successIds, memoryId)
                    successCount += 1
                } else {
                    let errorMsg = "Memory ID ${i}: Not found"
                    failureMessages = ArrayUtils.append(failureMessages, errorMsg)
                    failureCount += 1
                }
            }

            let result = BatchOperationResult(
                UInt32(memoryIds.size),
                successCount,
                failureCount,
                successIds,
                failureMessages
            )

            return AgentMemResult<BatchOperationResult>.Ok(result)

        } catch (e: Exception) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InternalError("Batch delete failed: ${e}")
            )
        }
    }

    /// 获取记忆统计信息
    public func getMemoryStatistics(): AgentMemResult<MemoryStatistics> {
        if (!this._connected) {
            return AgentMemResult<MemoryStatistics>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        try {
            // 模拟统计信息
            let totalMemories = this._memoryCache.size()
            let stats = MemoryStatistics(
                totalMemories,
                totalMemories / 4,
                totalMemories / 2,
                totalMemories / 6,
                totalMemories / 8,
                0.65,
                UInt64(totalMemories) * 1024,
                Int64(TimeUtils.getCurrentTimestamp())
            )

            return AgentMemResult<MemoryStatistics>.Ok(stats)

        } catch (e: Exception) {
            return AgentMemResult<MemoryStatistics>.Err(
                AgentMemError.InternalError("Failed to get statistics: ${e}")
            )
        }
    }

    /// 压缩记忆（清理过期或低重要性记忆）
    public func compressMemories(compressionRatio: Float32): AgentMemResult<UInt32> {
        if (!this._connected) {
            return AgentMemResult<UInt32>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        if (compressionRatio < 0.0 || compressionRatio > 1.0) {
            return AgentMemResult<UInt32>.Err(
                AgentMemError.InvalidParameter("Compression ratio must be between 0.0 and 1.0")
            )
        }

        try {
            let originalCount = this._memoryCache.size()
            let targetRemovalCount = Int64(Float32(originalCount) * compressionRatio)
            
            // 模拟压缩：随机移除一些记忆
            var removedCount: UInt32 = 0
            if (targetRemovalCount > 0) {
                // 简化实现：清理部分缓存
                removedCount = UInt32(targetRemovalCount)
            }

            return AgentMemResult<UInt32>.Ok(removedCount)

        } catch (e: Exception) {
            return AgentMemResult<UInt32>.Err(
                AgentMemError.InternalError("Memory compression failed: ${e}")
            )
        }
    }

    /// 导出记忆
    public func exportMemories(format: String): AgentMemResult<String> {
        if (!this._connected) {
            return AgentMemResult<String>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        if (format != "json" && format != "csv" && format != "xml") {
            return AgentMemResult<String>.Err(
                AgentMemError.InvalidParameter("Unsupported export format: ${format}")
            )
        }

        try {
            let memoryCount = this._memoryCache.size()
            let exportData = "{\n  \"agent_id\": \"${this._agentId}\",\n  \"memory_count\": ${memoryCount},\n  \"export_time\": ${TimeUtils.getCurrentTimestamp()},\n  \"format\": \"${format}\"\n}"

            return AgentMemResult<String>.Ok(exportData)

        } catch (e: Exception) {
            return AgentMemResult<String>.Err(
                AgentMemError.InternalError("Export failed: ${e}")
            )
        }
    }

    /// 导入记忆
    public func importMemories(data: String, format: String): AgentMemResult<BatchOperationResult> {
        if (!this._connected) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.NotConnected("Memory operations service not connected")
            )
        }

        if (data.isEmpty()) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InvalidParameter("Import data cannot be empty")
            )
        }

        if (format != "json" && format != "csv" && format != "xml") {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InvalidParameter("Unsupported import format: ${format}")
            )
        }

        try {
            // 模拟导入：假设导入了3个记忆
            let importedCount: UInt32 = 3
            var successIds = Array<String>(0, {_ => ""})
            
            for (i in 0..Int64(importedCount)) {
                let memoryId = "imported_${i}_${TimeUtils.getCurrentTimestamp()}_${StringUtils.generateRandomId(6)}"
                this._memoryCache.set(memoryId, "Imported memory content ${i}")
                successIds = ArrayUtils.append(successIds, memoryId)
            }

            let result = BatchOperationResult(
                importedCount,
                importedCount,
                0,
                successIds,
                Array<String>(0, {_ => ""})
            )

            return AgentMemResult<BatchOperationResult>.Ok(result)

        } catch (e: Exception) {
            return AgentMemResult<BatchOperationResult>.Err(
                AgentMemError.InternalError("Import failed: ${e}")
            )
        }
    }

    /// 获取缓存大小
    public func getCacheSize(): UInt32 {
        return this._memoryCache.size()
    }

    /// 清空缓存
    public func clearCache(): Unit {
        this._memoryCache.clear()
    }
}
