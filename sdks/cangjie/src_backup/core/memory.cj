/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 记忆相关类型定义 - AgentMem 仓颉 SDK 的记忆数据结构
 */

package agentmem.core

import std.collection.HashMap
import std.time.DateTime
import std.uuid.Uuid

/// 核心记忆结构体
public struct Memory {
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
    public var importance: Float32
    /// 向量嵌入（可选）
    public var embedding: Option<Vector>
    /// 创建时间戳
    public var createdAt: Timestamp
    /// 最后访问时间戳
    public var lastAccessedAt: Timestamp
    /// 访问次数
    public var accessCount: UInt32
    /// 过期时间戳（可选）
    public var expiresAt: Option<Timestamp>
    /// 元数据
    public var metadata: HashMap<String, String>
    /// 版本号
    public var version: UInt32

    /// 构造函数 - 创建新记忆
    public init(
        id: String,
        agentId: String,
        content: String,
        memoryType: MemoryType = MemoryType.Semantic
    ) {
        this.id = id
        this.agentId = agentId
        this.userId = None
        this.memoryType = memoryType
        this.content = content
        this.importance = 0.5
        this.embedding = None
        this.createdAt = getCurrentTimestamp()
        this.lastAccessedAt = this.createdAt
        this.accessCount = 0
        this.expiresAt = None
        this.metadata = HashMap<String, String>()
        this.version = 1
    }

    /// 构造函数 - 完整参数
    public init(
        id: String,
        agentId: String,
        userId: Option<String>,
        memoryType: MemoryType,
        content: String,
        importance: Float32,
        embedding: Option<Vector>,
        createdAt: Timestamp,
        lastAccessedAt: Timestamp,
        accessCount: UInt32,
        expiresAt: Option<Timestamp>,
        metadata: HashMap<String, String>,
        version: UInt32
    ) {
        this.id = id
        this.agentId = agentId
        this.userId = userId
        this.memoryType = memoryType
        this.content = content
        this.importance = importance
        this.embedding = embedding
        this.createdAt = createdAt
        this.lastAccessedAt = lastAccessedAt
        this.accessCount = accessCount
        this.expiresAt = expiresAt
        this.metadata = metadata
        this.version = version
    }

    /// 生成新的记忆ID
    public static func generateId(): String {
        return Uuid.random().toString()
    }

    /// 设置用户ID
    public func setUserId(userId: String): Memory {
        this.userId = Some(userId)
        return this
    }

    /// 设置重要性
    public func setImportance(importance: Float32): Memory {
        this.importance = if (importance < 0.0) { 0.0 } else if (importance > 1.0) { 1.0 } else { importance }
        return this
    }

    /// 设置嵌入向量
    public func setEmbedding(embedding: Vector): Memory {
        this.embedding = Some(embedding)
        return this
    }

    /// 设置过期时间
    public func setExpiresAt(expiresAt: Timestamp): Memory {
        this.expiresAt = Some(expiresAt)
        return this
    }

    /// 添加元数据
    public func addMetadata(key: String, value: String): Memory {
        this.metadata[key] = value
        return this
    }

    /// 获取元数据
    public func getMetadata(key: String): Option<String> {
        return this.metadata.get(key)
    }

    /// 移除元数据
    public func removeMetadata(key: String): Memory {
        this.metadata.remove(key)
        return this
    }

    /// 记录访问
    public func markAccessed(): Memory {
        this.lastAccessedAt = getCurrentTimestamp()
        this.accessCount = this.accessCount + 1
        return this
    }

    /// 更新内容
    public func updateContent(content: String): Memory {
        this.content = content
        this.version = this.version + 1
        return this
    }

    /// 检查是否过期
    public func isExpired(): Bool {
        if (this.expiresAt.isSome()) {
            let expireTime = this.expiresAt.getOrThrow()
            return getCurrentTimestamp() > expireTime
        }
        return false
    }

    /// 获取重要性级别
    public func getImportanceLevel(): ImportanceLevel {
        return ImportanceLevel.fromScore(this.importance)
    }

    /// 获取年龄（天数）
    public func getAgeInDays(): UInt32 {
        let currentTime = getCurrentTimestamp()
        let ageInSeconds = currentTime - this.createdAt
        return UInt32(ageInSeconds / (24 * 60 * 60))
    }

    /// 获取最后访问间隔（天数）
    public func getLastAccessAgeInDays(): UInt32 {
        let currentTime = getCurrentTimestamp()
        let ageInSeconds = currentTime - this.lastAccessedAt
        return UInt32(ageInSeconds / (24 * 60 * 60))
    }

    /// 克隆记忆
    public func clone(): Memory {
        let newMetadata = HashMap<String, String>()
        for ((key, value) in this.metadata) {
            newMetadata[key] = value
        }

        let newEmbedding = if (this.embedding.isSome()) {
            let originalEmbedding = this.embedding.getOrThrow()
            let newVec = Array<Float32>()
            for (value in originalEmbedding) {
                newVec.append(value)
            }
            Some(newVec)
        } else {
            None
        }

        return Memory(
            this.id,
            this.agentId,
            this.userId,
            this.memoryType,
            this.content,
            this.importance,
            newEmbedding,
            this.createdAt,
            this.lastAccessedAt,
            this.accessCount,
            this.expiresAt,
            newMetadata,
            this.version
        )
    }

    /// 转换为字符串表示
    public func toString(): String {
        return "Memory{id=${this.id}, agentId=${this.agentId}, type=${this.memoryType.toString()}, content=\"${this.content}\", importance=${this.importance}}"
    }

    /// 比较两个记忆是否相等
    public func equals(other: Memory): Bool {
        return this.id == other.id && 
               this.agentId == other.agentId &&
               this.content == other.content &&
               this.memoryType == other.memoryType
    }

    /// 计算哈希值
    public func hashCode(): Int32 {
        // 简单的哈希计算
        var hash = this.id.hashCode()
        hash = hash * 31 + this.agentId.hashCode()
        hash = hash * 31 + this.content.hashCode()
        return hash
    }
}

/// 记忆构建器
public class MemoryBuilder {
    private var _id: Option<String> = None
    private var _agentId: Option<String> = None
    private var _userId: Option<String> = None
    private var _memoryType: MemoryType = MemoryType.Semantic
    private var _content: Option<String> = None
    private var _importance: Float32 = 0.5
    private var _embedding: Option<Vector> = None
    private var _expiresAt: Option<Timestamp> = None
    private var _metadata: HashMap<String, String> = HashMap<String, String>()

    public init() {}

    /// 设置ID
    public func withId(id: String): MemoryBuilder {
        this._id = Some(id)
        return this
    }

    /// 设置代理ID
    public func withAgentId(agentId: String): MemoryBuilder {
        this._agentId = Some(agentId)
        return this
    }

    /// 设置用户ID
    public func withUserId(userId: String): MemoryBuilder {
        this._userId = Some(userId)
        return this
    }

    /// 设置记忆类型
    public func withMemoryType(memoryType: MemoryType): MemoryBuilder {
        this._memoryType = memoryType
        return this
    }

    /// 设置内容
    public func withContent(content: String): MemoryBuilder {
        this._content = Some(content)
        return this
    }

    /// 设置重要性
    public func withImportance(importance: Float32): MemoryBuilder {
        this._importance = if (importance < 0.0) { 0.0 } else if (importance > 1.0) { 1.0 } else { importance }
        return this
    }

    /// 设置嵌入向量
    public func withEmbedding(embedding: Vector): MemoryBuilder {
        this._embedding = Some(embedding)
        return this
    }

    /// 设置过期时间
    public func withExpiresAt(expiresAt: Timestamp): MemoryBuilder {
        this._expiresAt = Some(expiresAt)
        return this
    }

    /// 添加元数据
    public func withMetadata(key: String, value: String): MemoryBuilder {
        this._metadata[key] = value
        return this
    }

    /// 构建记忆对象
    public func build(): Memory {
        let id = this._id.getOrElse(Memory.generateId())
        let agentId = this._agentId.getOrThrow()
        let content = this._content.getOrThrow()

        let memory = Memory(id, agentId, content, this._memoryType)
        memory.userId = this._userId
        memory.importance = this._importance
        memory.embedding = this._embedding
        memory.expiresAt = this._expiresAt
        memory.metadata = this._metadata

        return memory
    }
}

/// 记忆历史记录
public struct MemoryHistory {
    /// 记忆ID
    public var memoryId: String
    /// 操作类型
    public var operation: HistoryOperation
    /// 操作时间戳
    public var timestamp: Timestamp
    /// 操作前的内容
    public var oldContent: Option<String>
    /// 操作后的内容
    public var newContent: Option<String>
    /// 操作描述
    public var description: String
    /// 操作者
    public var operator: Option<String>

    public init(
        memoryId: String,
        operation: HistoryOperation,
        description: String,
        oldContent: Option<String> = None,
        newContent: Option<String> = None,
        operator: Option<String> = None
    ) {
        this.memoryId = memoryId
        this.operation = operation
        this.timestamp = getCurrentTimestamp()
        this.oldContent = oldContent
        this.newContent = newContent
        this.description = description
        this.operator = operator
    }
}

/// 历史操作类型
public enum HistoryOperation {
    | Created      // 创建
    | Updated      // 更新
    | Accessed     // 访问
    | Deleted      // 删除
    | Archived     // 归档
    | Restored     // 恢复

    public func toString(): String {
        return match (this) {
            case Created => "created"
            case Updated => "updated"
            case Accessed => "accessed"
            case Deleted => "deleted"
            case Archived => "archived"
            case Restored => "restored"
        }
    }
}
