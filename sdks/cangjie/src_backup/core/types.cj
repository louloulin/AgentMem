/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 核心类型定义 - AgentMem 仓颉 SDK 的基础数据类型
 */

package agentmem.core

import std.collection.HashMap
import std.time.DateTime

/// 记忆类型枚举
public enum MemoryType {
    | Episodic      // 情景记忆 - 特定事件和经历
    | Semantic      // 语义记忆 - 事实和一般知识
    | Procedural    // 程序记忆 - 技能和程序
    | Working       // 工作记忆 - 临时信息

    /// 转换为字符串表示
    public func toString(): String {
        return match (this) {
            case Episodic => "episodic"
            case Semantic => "semantic"
            case Procedural => "procedural"
            case Working => "working"
        }
    }

    /// 从字符串创建记忆类型
    public static func fromString(s: String): Option<MemoryType> {
        return match (s) {
            case "episodic" => Some(Episodic)
            case "semantic" => Some(Semantic)
            case "procedural" => Some(Procedural)
            case "working" => Some(Working)
            case _ => None
        }
    }

    /// 转换为数值表示（用于FFI）
    public func toUInt32(): UInt32 {
        return match (this) {
            case Episodic => 0
            case Semantic => 1
            case Procedural => 2
            case Working => 3
        }
    }

    /// 从数值创建记忆类型（用于FFI）
    public static func fromUInt32(value: UInt32): Option<MemoryType> {
        return match (value) {
            case 0 => Some(Episodic)
            case 1 => Some(Semantic)
            case 2 => Some(Procedural)
            case 3 => Some(Working)
            case _ => None
        }
    }
}

/// 重要性级别枚举
public enum ImportanceLevel {
    | Low = 1        // 低重要性
    | Medium = 2     // 中等重要性
    | High = 3       // 高重要性
    | Critical = 4   // 关键重要性

    /// 转换为分数值
    public func toScore(): Float32 {
        return match (this) {
            case Low => 0.25
            case Medium => 0.5
            case High => 0.75
            case Critical => 1.0
        }
    }

    /// 从分数值创建重要性级别
    public static func fromScore(score: Float32): ImportanceLevel {
        if (score >= 0.8) {
            return Critical
        } else if (score >= 0.6) {
            return High
        } else if (score >= 0.4) {
            return Medium
        } else {
            return Low
        }
    }

    /// 转换为字符串表示
    public func toString(): String {
        return match (this) {
            case Low => "low"
            case Medium => "medium"
            case High => "high"
            case Critical => "critical"
        }
    }
}

/// 向量类型别名
public type Vector = Array<Float32>

/// 时间戳类型别名
public type Timestamp = Int64

/// 获取当前时间戳
public func getCurrentTimestamp(): Timestamp {
    return DateTime.now().toUnixTimestamp()
}

/// 记忆搜索结果
public struct MemorySearchResult {
    /// 记忆对象
    public var memory: Memory
    /// 相似度分数 (0.0 到 1.0)
    public var score: Float32
    /// 搜索排名
    public var rank: UInt32

    public init(memory: Memory, score: Float32, rank: UInt32) {
        this.memory = memory
        this.score = score
        this.rank = rank
    }
}

/// 批量操作结果
public struct BatchResult<T> {
    /// 成功的结果
    public var successes: Array<T>
    /// 失败的结果
    public var failures: Array<String>
    /// 总数量
    public var total: UInt32

    public init() {
        this.successes = Array<T>()
        this.failures = Array<String>()
        this.total = 0
    }

    /// 添加成功结果
    public func addSuccess(item: T): Unit {
        this.successes.append(item)
        this.total = this.total + 1
    }

    /// 添加失败结果
    public func addFailure(error: String): Unit {
        this.failures.append(error)
        this.total = this.total + 1
    }

    /// 获取成功率
    public func getSuccessRate(): Float32 {
        if (this.total == 0) {
            return 0.0
        }
        return Float32(this.successes.size) / Float32(this.total)
    }
}

/// 搜索过滤器
public struct SearchFilter {
    /// 记忆类型过滤
    public var memoryTypes: Option<Array<MemoryType>>
    /// 重要性范围过滤
    public var importanceRange: Option<(Float32, Float32)>
    /// 时间范围过滤
    public var timeRange: Option<(Timestamp, Timestamp)>
    /// 元数据过滤
    public var metadata: HashMap<String, String>
    /// 代理ID过滤
    public var agentIds: Option<Array<String>>
    /// 用户ID过滤
    public var userIds: Option<Array<String>>

    public init() {
        this.memoryTypes = None
        this.importanceRange = None
        this.timeRange = None
        this.metadata = HashMap<String, String>()
        this.agentIds = None
        this.userIds = None
    }

    /// 设置记忆类型过滤
    public func withMemoryTypes(types: Array<MemoryType>): SearchFilter {
        this.memoryTypes = Some(types)
        return this
    }

    /// 设置重要性范围过滤
    public func withImportanceRange(min: Float32, max: Float32): SearchFilter {
        this.importanceRange = Some((min, max))
        return this
    }

    /// 设置时间范围过滤
    public func withTimeRange(start: Timestamp, end: Timestamp): SearchFilter {
        this.timeRange = Some((start, end))
        return this
    }

    /// 添加元数据过滤
    public func withMetadata(key: String, value: String): SearchFilter {
        this.metadata[key] = value
        return this
    }
}

/// 分页参数
public struct PaginationParams {
    /// 页码（从1开始）
    public var page: UInt32
    /// 每页大小
    public var pageSize: UInt32
    /// 偏移量
    public var offset: UInt32

    public init(page: UInt32 = 1, pageSize: UInt32 = 10) {
        this.page = page
        this.pageSize = pageSize
        this.offset = (page - 1) * pageSize
    }

    /// 计算偏移量
    public func calculateOffset(): UInt32 {
        return (this.page - 1) * this.pageSize
    }
}

/// 分页结果
public struct PaginatedResult<T> {
    /// 数据项
    public var items: Array<T>
    /// 当前页码
    public var page: UInt32
    /// 每页大小
    public var pageSize: UInt32
    /// 总数量
    public var totalCount: UInt32
    /// 总页数
    public var totalPages: UInt32

    public init(items: Array<T>, page: UInt32, pageSize: UInt32, totalCount: UInt32) {
        this.items = items
        this.page = page
        this.pageSize = pageSize
        this.totalCount = totalCount
        this.totalPages = if (pageSize > 0) { (totalCount + pageSize - 1) / pageSize } else { 0 }
    }

    /// 是否有下一页
    public func hasNext(): Bool {
        return this.page < this.totalPages
    }

    /// 是否有上一页
    public func hasPrevious(): Bool {
        return this.page > 1
    }
}

/// 统计信息
public struct MemoryStats {
    /// 总记忆数量
    public var totalMemories: UInt32
    /// 按类型分组的数量
    public var memoryTypeCount: HashMap<MemoryType, UInt32>
    /// 平均重要性
    public var averageImportance: Float32
    /// 最高重要性
    public var maxImportance: Float32
    /// 最低重要性
    public var minImportance: Float32
    /// 最近创建时间
    public var lastCreatedAt: Timestamp
    /// 最近访问时间
    public var lastAccessedAt: Timestamp

    public init() {
        this.totalMemories = 0
        this.memoryTypeCount = HashMap<MemoryType, UInt32>()
        this.averageImportance = 0.0
        this.maxImportance = 0.0
        this.minImportance = 1.0
        this.lastCreatedAt = 0
        this.lastAccessedAt = 0
    }
}
