/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file memory.cj
 * 简化的Memory类型定义（无FFI依赖）
 */

package agentmem_http

/// 记忆类型枚举
public enum MemoryType {
    | Episodic    // 情景记忆
    | Semantic    // 语义记忆
    | Procedural  // 程序记忆
    | Working     // 工作记忆
    
    /// 转换为字符串
    public func toString(): String {
        match (this) {
            | Episodic => "episodic"
            | Semantic => "semantic"
            | Procedural => "procedural"
            | Working => "working"
        }
    }
    
    /// 从字符串解析
    public static func fromString(s: String): MemoryType {
        match (s.toLower()) {
            | "episodic" => Episodic
            | "semantic" => Semantic
            | "procedural" => Procedural
            | "working" => Working
            | _ => Episodic  // 默认
        }
    }
}

/// 记忆对象（简化版本）
public class Memory {
    /// 唯一标识符
    public var id: String
    /// 代理标识符
    public var agentId: String
    /// 用户标识符（可选）
    public var userId: Option<String>
    /// 记忆类型
    public var memoryType: MemoryType
    /// 记忆内容
    public var content: String
    /// 重要性分数 (0.0 到 1.0)
    public var importance: Float64
    /// 创建时间戳
    public var createdAt: Int64
    /// 最后访问时间戳
    public var lastAccessedAt: Int64
    /// 访问次数
    public var accessCount: Int32
    
    /// 构造函数
    public init(
        id: String,
        agentId: String,
        content: String,
        memoryType: MemoryType = MemoryType.Episodic
    ) {
        this.id = id
        this.agentId = agentId
        this.userId = None
        this.memoryType = memoryType
        this.content = content
        this.importance = 0.5
        this.createdAt = getCurrentTimestamp()
        this.lastAccessedAt = this.createdAt
        this.accessCount = 0
    }
    
    /// 完整构造函数
    public init(
        id: String,
        agentId: String,
        userId: Option<String>,
        memoryType: MemoryType,
        content: String,
        importance: Float64,
        createdAt: Int64,
        lastAccessedAt: Int64,
        accessCount: Int32
    ) {
        this.id = id
        this.agentId = agentId
        this.userId = userId
        this.memoryType = memoryType
        this.content = content
        this.importance = importance
        this.createdAt = createdAt
        this.lastAccessedAt = lastAccessedAt
        this.accessCount = accessCount
    }
    
    /// 创建新记忆（工厂方法）
    public static func create(agentId: String, content: String): Memory {
        let id = generateId()
        return Memory(id, agentId, content, MemoryType.Episodic)
    }
    
    /// 转换为JSON
    public func toJson(): String {
        let builder = JsonBuilder()
        builder.addString("id", this.id)
        builder.addString("agent_id", this.agentId)
        builder.addString("content", this.content)
        builder.addString("memory_type", this.memoryType.toString())
        builder.addNumber("importance", this.importance)
        builder.addInt("created_at", this.createdAt)
        builder.addInt("last_accessed_at", this.lastAccessedAt)
        builder.addInt("access_count", Int64(this.accessCount))
        
        match (this.userId) {
            | Some(uid) => builder.addString("user_id", uid)
            | None => {}
        }
        
        return builder.build()
    }
}

/// 从JSON解析Memory对象
public func parseMemory(json: String): Memory {
    let id = extractField(json, "id")
    let agentId = extractField(json, "agent_id")
    let content = extractField(json, "content")
    let memoryTypeStr = extractField(json, "memory_type")
    let importance = extractFloatField(json, "importance")
    let createdAt = extractIntField(json, "created_at")
    let lastAccessedAt = extractIntField(json, "last_accessed_at")
    let accessCount = Int32(extractIntField(json, "access_count"))
    
    let userIdStr = extractField(json, "user_id")
    let userId = if (userIdStr.isEmpty()) { None } else { Some(userIdStr) }
    
    let memoryType = MemoryType.fromString(memoryTypeStr)
    
    return Memory(
        id,
        agentId,
        userId,
        memoryType,
        content,
        importance,
        createdAt,
        lastAccessedAt,
        accessCount
    )
}

/// 从JSON解析Memory数组
public func parseMemories(json: String): Array<Memory> {
    // 简化实现：假设格式为 {"results":[...]}
    var memories = Array<Memory>()
    
    // 查找results数组
    let resultsStart = json.indexOf("\"results\":[")
    if (resultsStart < 0) {
        return memories
    }
    
    // 简单实现：分割JSON对象
    // 实际应使用更健壮的JSON解析器
    // 这里仅作演示
    
    return memories
}

/// 搜索结果
public class SearchResult {
    public var memory: Memory
    public var score: Float64
    public var rank: Int32
    
    public init(memory: Memory, score: Float64, rank: Int32 = 0) {
        this.memory = memory
        this.score = score
        this.rank = rank
    }
}

/// 生成唯一ID（简化实现）
func generateId(): String {
    let timestamp = getCurrentTimestamp()
    let random = Int32(timestamp % 10000)
    return "mem-${timestamp}-${random}"
}

/// 获取当前时间戳（毫秒）
func getCurrentTimestamp(): Int64 {
    // 简化实现：使用系统时间
    // 实际应使用 std.time 库
    return System.currentTimeMillis()
}

