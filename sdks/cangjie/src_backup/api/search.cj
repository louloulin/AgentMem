/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 搜索功能 - AgentMem 仓颉 SDK 的搜索功能专门实现
 */

package agentmem.api

import agentmem.core.*
import agentmem.ffi.*

/// 搜索管理器 - 提供高级搜索功能
public class SearchManager {
    /// 客户端引用
    private var _client: AgentMemClient
    /// 搜索配置
    private var _config: SearchConfig
    /// 搜索缓存
    private var _searchCache: Option<SimpleCache<String, Array<MemorySearchResult>>>

    public init(client: AgentMemClient, config: SearchConfig = SearchConfig()) {
        this._client = client
        this._config = config
        
        if (client.getConfig().enableCache) {
            this._searchCache = Some(SimpleCache<String, Array<MemorySearchResult>>(50))
        } else {
            this._searchCache = None
        }
    }

    /// 智能搜索 - 结合语义搜索和全文搜索
    public func smartSearch(
        query: String, 
        filter: Option<SearchFilter> = None,
        limit: Option<UInt32> = None
    ): AgentMemResult<Array<MemorySearchResult>> {
        let searchLimit = limit.getOrElse(this._config.maxResults)
        let cacheKey = this.generateCacheKey(query, filter, searchLimit)
        
        // 检查缓存
        if (this._searchCache.isSome()) {
            let cached = this._searchCache.getOrThrow().get(cacheKey)
            if (cached.isSome()) {
                return AgentMemResult<Array<MemorySearchResult>>.Ok(cached.getOrThrow())
            }
        }

        let monitor = PerformanceMonitor("smart_search")

        // 执行搜索
        let searchResult = if (filter.isSome()) {
            this._client.searchMemoriesFiltered(query, filter.getOrThrow(), searchLimit)
        } else {
            this._client.searchMemories(query, searchLimit)
        }

        match (searchResult) {
            case Ok(results) => {
                // 应用搜索配置进行后处理
                let processedResults = this.postProcessResults(results)
                
                // 缓存结果
                if (this._searchCache.isSome()) {
                    this._searchCache.getOrThrow().put(cacheKey, processedResults)
                }
                
                monitor.finishAndLog()
                return AgentMemResult<Array<MemorySearchResult>>.Ok(processedResults)
            }
            case Err(error) => {
                return AgentMemResult<Array<MemorySearchResult>>.Err(error)
            }
        }
    }

    /// 语义搜索 - 基于向量相似度的搜索
    public func semanticSearch(
        query: String, 
        threshold: Float32 = 0.7,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        let monitor = PerformanceMonitor("semantic_search")

        try {
            // 首先生成查询的嵌入向量
            let embeddingResult = this.generateQueryEmbedding(query)
            match (embeddingResult) {
                case Ok(embedding) => {
                    // 使用嵌入向量进行搜索
                    let searchResult = this.searchByEmbedding(embedding, threshold, limit)
                    monitor.finishAndLog()
                    return searchResult
                }
                case Err(error) => {
                    return AgentMemResult<Array<MemorySearchResult>>.Err(error)
                }
            }
            
        } catch (e: Exception) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Failed to perform semantic search: ${e}")
            )
        }
    }

    /// 相似记忆搜索
    public func findSimilarMemories(
        memoryId: String, 
        threshold: Float32 = 0.7,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        return this._client.searchSimilarMemories(memoryId, limit, threshold)
    }

    /// 多条件搜索
    public func advancedSearch(
        queries: Array<String>,
        operator: SearchOperator = SearchOperator.And,
        filter: Option<SearchFilter> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        if (queries.isEmpty()) {
            return AgentMemResult<Array<MemorySearchResult>>.Ok(Array<MemorySearchResult>())
        }

        let monitor = PerformanceMonitor("advanced_search")

        try {
            var allResults = Array<Array<MemorySearchResult>>()
            
            // 对每个查询执行搜索
            for (query in queries) {
                let searchResult = if (filter.isSome()) {
                    this._client.searchMemoriesFiltered(query, filter.getOrThrow(), limit * 2)
                } else {
                    this._client.searchMemories(query, limit * 2)
                }
                
                match (searchResult) {
                    case Ok(results) => allResults.append(results)
                    case Err(error) => return AgentMemResult<Array<MemorySearchResult>>.Err(error)
                }
            }
            
            // 合并结果
            let mergedResults = this.mergeSearchResults(allResults, operator, limit)
            monitor.finishAndLog()
            
            return AgentMemResult<Array<MemorySearchResult>>.Ok(mergedResults)
            
        } catch (e: Exception) {
            return AgentMemResult<Array<MemorySearchResult>>.Err(
                AgentMemError.InternalError("Failed to perform advanced search: ${e}")
            )
        }
    }

    /// 时间范围搜索
    public func searchByTimeRange(
        startTime: Timestamp,
        endTime: Timestamp,
        agentId: Option<String> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        var filter = SearchFilter()
        filter.timeRange = Some((startTime, endTime))
        
        if (agentId.isSome()) {
            filter.agentIds = Some([agentId.getOrThrow()])
        }
        
        return this.smartSearch("*", Some(filter), Some(limit))
    }

    /// 按重要性搜索
    public func searchByImportance(
        minImportance: Float32,
        maxImportance: Float32 = 1.0,
        agentId: Option<String> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        var filter = SearchFilter()
        filter.importanceRange = Some((minImportance, maxImportance))
        
        if (agentId.isSome()) {
            filter.agentIds = Some([agentId.getOrThrow()])
        }
        
        return this.smartSearch("*", Some(filter), Some(limit))
    }

    /// 按记忆类型搜索
    public func searchByMemoryType(
        memoryTypes: Array<MemoryType>,
        agentId: Option<String> = None,
        limit: UInt32 = 10
    ): AgentMemResult<Array<MemorySearchResult>> {
        var filter = SearchFilter()
        filter.memoryTypes = Some(memoryTypes)
        
        if (agentId.isSome()) {
            filter.agentIds = Some([agentId.getOrThrow()])
        }
        
        return this.smartSearch("*", Some(filter), Some(limit))
    }

    /// 搜索建议 - 基于查询历史提供搜索建议
    public func getSearchSuggestions(
        partialQuery: String,
        limit: UInt32 = 5
    ): AgentMemResult<Array<String>> {
        // 这里可以实现基于历史搜索的建议功能
        // 简化实现，返回基于部分查询的建议
        var suggestions = Array<String>()
        
        if (partialQuery.length >= 2) {
            suggestions.append("${partialQuery} 相关内容")
            suggestions.append("${partialQuery} 详细信息")
            suggestions.append("${partialQuery} 历史记录")
        }
        
        return AgentMemResult<Array<String>>.Ok(suggestions)
    }

    /// 更新搜索配置
    public func updateConfig(config: SearchConfig): Unit {
        this._config = config
    }

    /// 获取搜索配置
    public func getConfig(): SearchConfig {
        return this._config
    }

    /// 清除搜索缓存
    public func clearCache(): Unit {
        if (this._searchCache.isSome()) {
            this._searchCache.getOrThrow().clear()
        }
    }

    // 私有辅助方法
    private func generateCacheKey(
        query: String, 
        filter: Option<SearchFilter>, 
        limit: UInt32
    ): String {
        var key = "q:${query}:l:${limit}"
        if (filter.isSome()) {
            key += ":f:${filter.getOrThrow().hashCode()}"
        }
        return key
    }

    private func postProcessResults(results: Array<MemorySearchResult>): Array<MemorySearchResult> {
        var processedResults = results
        
        // 应用相似度阈值过滤
        if (this._config.similarityThreshold > 0.0) {
            var filteredResults = Array<MemorySearchResult>()
            for (result in processedResults) {
                if (result.score >= this._config.similarityThreshold) {
                    filteredResults.append(result)
                }
            }
            processedResults = filteredResults
        }
        
        // 应用排序
        processedResults = this.sortResults(processedResults)
        
        // 限制结果数量
        if (UInt32(processedResults.size) > this._config.maxResults) {
            var limitedResults = Array<MemorySearchResult>()
            for (i in 0..this._config.maxResults) {
                limitedResults.append(processedResults[Int32(i)])
            }
            processedResults = limitedResults
        }
        
        return processedResults
    }

    private func sortResults(results: Array<MemorySearchResult>): Array<MemorySearchResult> {
        // 简化的排序实现
        // 实际应该根据配置的排序方式进行排序
        return results
    }

    private func generateQueryEmbedding(query: String): AgentMemResult<Vector> {
        try {
            let handle = this._client._handle.getOrThrow()
            let cQuery = CStringConverter.toCString(query)
            var embeddingPtr = CPointer<Float32>()
            var length: UInt32 = 0

            let result = unsafe {
                agentmem_generate_embedding(
                    handle, 
                    cQuery, 
                    CPointer.addressOf(embeddingPtr), 
                    CPointer.addressOf(length)
                )
            }
            CStringConverter.freeCString(cQuery)

            if (result == 0) {
                var embedding = Array<Float32>()
                for (i in 0..length) {
                    let value = unsafe { (embeddingPtr + UIntNative(i)).read() }
                    embedding.append(value)
                }
                
                unsafe { agentmem_free_embedding(embeddingPtr) }
                return AgentMemResult<Vector>.Ok(embedding)
            } else {
                let error = FFIErrorHandler.getLastError()
                return AgentMemResult<Vector>.Err(error)
            }
            
        } catch (e: Exception) {
            return AgentMemResult<Vector>.Err(
                AgentMemError.InternalError("Failed to generate query embedding: ${e}")
            )
        }
    }

    private func searchByEmbedding(
        embedding: Vector, 
        threshold: Float32, 
        limit: UInt32
    ): AgentMemResult<Array<MemorySearchResult>> {
        // 这里需要实现基于嵌入向量的搜索
        // 简化实现，返回空结果
        return AgentMemResult<Array<MemorySearchResult>>.Ok(Array<MemorySearchResult>())
    }

    private func mergeSearchResults(
        allResults: Array<Array<MemorySearchResult>>,
        operator: SearchOperator,
        limit: UInt32
    ): Array<MemorySearchResult> {
        if (allResults.isEmpty()) {
            return Array<MemorySearchResult>()
        }

        match (operator) {
            case SearchOperator.And => {
                // 交集：只返回在所有结果中都出现的记忆
                return this.intersectResults(allResults, limit)
            }
            case SearchOperator.Or => {
                // 并集：返回在任何结果中出现的记忆
                return this.unionResults(allResults, limit)
            }
        }
    }

    private func intersectResults(
        allResults: Array<Array<MemorySearchResult>>,
        limit: UInt32
    ): Array<MemorySearchResult> {
        if (allResults.isEmpty()) {
            return Array<MemorySearchResult>()
        }

        var intersection = allResults[0]
        for (i in 1..allResults.size) {
            intersection = this.intersectTwoResults(intersection, allResults[i])
        }

        return this.limitResults(intersection, limit)
    }

    private func unionResults(
        allResults: Array<Array<MemorySearchResult>>,
        limit: UInt32
    ): Array<MemorySearchResult> {
        var union = Array<MemorySearchResult>()
        var seenIds = HashMap<String, Bool>()

        for (results in allResults) {
            for (result in results) {
                if (!seenIds.containsKey(result.memory.id)) {
                    union.append(result)
                    seenIds[result.memory.id] = true
                }
            }
        }

        return this.limitResults(union, limit)
    }

    private func intersectTwoResults(
        results1: Array<MemorySearchResult>,
        results2: Array<MemorySearchResult>
    ): Array<MemorySearchResult> {
        var intersection = Array<MemorySearchResult>()
        var ids2 = HashMap<String, MemorySearchResult>()

        for (result in results2) {
            ids2[result.memory.id] = result
        }

        for (result in results1) {
            if (ids2.containsKey(result.memory.id)) {
                intersection.append(result)
            }
        }

        return intersection
    }

    private func limitResults(
        results: Array<MemorySearchResult>,
        limit: UInt32
    ): Array<MemorySearchResult> {
        if (UInt32(results.size) <= limit) {
            return results
        }

        var limitedResults = Array<MemorySearchResult>()
        for (i in 0..limit) {
            limitedResults.append(results[Int32(i)])
        }
        return limitedResults
    }
}

/// 搜索操作符枚举
public enum SearchOperator {
    | And    // 交集
    | Or     // 并集

    public func toString(): String {
        return match (this) {
            case And => "and"
            case Or => "or"
        }
    }
}
