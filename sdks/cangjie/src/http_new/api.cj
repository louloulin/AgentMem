/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file api.cj
 * Memory API实现（基于HTTP）
 */

package agentmem_http

/// Memory API客户端
public class MemoryApi {
    private let client: AgentMemHttpClient
    
    /// 构造函数
    public init(client: AgentMemHttpClient) {
        this.client = client
    }
    
    /// 添加记忆
    public func addMemory(memory: Memory): Result<String, AgentMemError> {
        let body = JsonBuilder()
            .addString("agent_id", memory.agentId)
            .addString("content", memory.content)
            .addString("memory_type", memory.memoryType.toString())
            .addNumber("importance", memory.importance)
            .build()
        
        let response = this.client.post("/api/v1/memories", body)
        match (response) {
            | Ok(jsonStr) => {
                let memoryId = extractField(jsonStr, "id")
                return Ok(memoryId)
            }
            | Err(e) => return Err(e)
        }
    }
    
    /// 获取记忆
    public func getMemory(memoryId: String): Result<Memory, AgentMemError> {
        let response = this.client.get("/api/v1/memories/${memoryId}")
        match (response) {
            | Ok(jsonStr) => {
                let memory = parseMemory(jsonStr)
                return Ok(memory)
            }
            | Err(e) => return Err(e)
        }
    }
    
    /// 更新记忆
    public func updateMemory(memoryId: String, content: String): Result<Unit, AgentMemError> {
        let body = JsonBuilder()
            .addString("content", content)
            .build()
        
        let response = this.client.put("/api/v1/memories/${memoryId}", body)
        match (response) {
            | Ok(_) => return Ok(())
            | Err(e) => return Err(e)
        }
    }
    
    /// 删除记忆
    public func deleteMemory(memoryId: String): Result<Unit, AgentMemError> {
        let response = this.client.delete("/api/v1/memories/${memoryId}")
        match (response) {
            | Ok(_) => return Ok(())
            | Err(e) => return Err(e)
        }
    }
    
    /// 搜索记忆
    public func searchMemories(query: String, limit: Int32): Result<Array<SearchResult>, AgentMemError> {
        let body = JsonBuilder()
            .addString("query", query)
            .addInt("limit", Int64(limit))
            .build()
        
        let response = this.client.post("/api/v1/memories/search", body)
        match (response) {
            | Ok(jsonStr) => {
                // 简化实现：返回空数组
                // 实际需要解析JSON数组
                let results = Array<SearchResult>()
                return Ok(results)
            }
            | Err(e) => return Err(e)
        }
    }
    
    /// 批量添加记忆
    public func addMemoriesBatch(memories: Array<Memory>): Result<Int32, AgentMemError> {
        // 构建批量请求体
        var memoriesJson = Array<String>(memories.size(), item: "")
        for (i in 0..memories.size()) {
            memoriesJson[i] = memories[i].toJson()
        }
        
        var bodyStr = "{\"memories\":["
        for (i in 0..memoriesJson.size()) {
            bodyStr += memoriesJson[i]
            if (i < memoriesJson.size() - 1) {
                bodyStr += ","
            }
        }
        bodyStr += "]}"
        
        let response = this.client.post("/api/v1/memories/batch", bodyStr)
        match (response) {
            | Ok(jsonStr) => {
                let successCount = extractIntField(jsonStr, "success_count")
                return Ok(Int32(successCount))
            }
            | Err(e) => return Err(e)
        }
    }
    
    /// 获取记忆统计
    public func getMemoryStats(agentId: String): Result<MemoryStats, AgentMemError> {
        let response = this.client.get("/api/v1/agents/${agentId}/stats")
        match (response) {
            | Ok(jsonStr) => {
                let stats = MemoryStats(
                    Int32(extractIntField(jsonStr, "total_memories")),
                    extractFloatField(jsonStr, "average_importance")
                )
                return Ok(stats)
            }
            | Err(e) => return Err(e)
        }
    }
}

/// 记忆统计信息
public class MemoryStats {
    public var totalMemories: Int32
    public var averageImportance: Float64
    
    public init(totalMemories: Int32, averageImportance: Float64) {
        this.totalMemories = totalMemories
        this.averageImportance = averageImportance
    }
}

