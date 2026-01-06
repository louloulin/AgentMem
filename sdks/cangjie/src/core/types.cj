/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 核心类型定义 - AgentMem 仓颉 SDK 的基础数据类型
 */

package agentmem.core

import agentmem.utils.{SimpleMap, ArrayUtils}

/// 记忆类型枚举
public enum MemoryType {
    | Episodic      // 情景记忆
    | Semantic      // 语义记忆
    | Procedural    // 程序记忆
    | Working       // 工作记忆

    /// 转换为字符串表示
    public func toString(): String {
        return match (this) {
            case Episodic => "episodic"
            case Semantic => "semantic"
            case Procedural => "procedural"
            case Working => "working"
        }
    }

    /// 比较操作
    public operator func ==(right: MemoryType): Bool {
        return match (this) {
            case Episodic => match (right) {
                case Episodic => true
                case _ => false
            }
            case Semantic => match (right) {
                case Semantic => true
                case _ => false
            }
            case Procedural => match (right) {
                case Procedural => true
                case _ => false
            }
            case Working => match (right) {
                case Working => true
                case _ => false
            }
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

    public static func fromUInt32(value: UInt32): Option<MemoryType> {
        return match (value) {
            case 0 => Some(Episodic)
            case 1 => Some(Semantic)
            case 2 => Some(Procedural)
            case 3 => Some(Working)
            case _ => None
        }
    }

    public func toUInt32(): UInt32 {
        return match (this) {
            case Episodic => 0
            case Semantic => 1
            case Procedural => 2
            case Working => 3
        }
    }
}

/// 重要性级别枚举
public enum ImportanceLevel {
    | Low           // 低重要性
    | Medium        // 中等重要性
    | High          // 高重要性
    | Critical      // 关键重要性

    /// 转换为浮点数表示
    public func toFloat32(): Float32 {
        return match (this) {
            case Low => 0.25f32
            case Medium => 0.5f32
            case High => 0.75f32
            case Critical => 1.0f32
        }
    }

    /// 从分数值创建重要性级别
    public static func fromScore(score: Float32): ImportanceLevel {
        if (score >= 0.8f32) {
            return Critical
        } else if (score >= 0.6f32) {
            return High
        } else if (score >= 0.4f32) {
            return Medium
        } else {
            return Low
        }
    }

    /// 转换为字符串
    public func toString(): String {
        return match (this) {
            case Low => "low"
            case Medium => "medium"
            case High => "high"
            case Critical => "critical"
        }
    }
}

/// 搜索过滤器
public class SearchFilter {
    public var memoryTypes: Option<Array<MemoryType>> = None
    public var importanceRange: Option<(Float32, Float32)> = None
    public var timeRange: Option<(UInt64, UInt64)> = None
    public var agentId: Option<String> = None
    public var userId: Option<String> = None
    public var sessionId: Option<String> = None
    public var tags: Option<Array<String>> = None
    public var limit: Option<UInt32> = None
    public var metadata: SimpleMap = SimpleMap()
    public var agentIds: Option<Array<String>> = None
    public var userIds: Option<Array<String>> = None

    public init() {}
}

/// 记忆搜索结果
public class MemorySearchResult {
    public var memory: Memory
    public var score: Float32
    public var relevance: Float32

    public init(memory: Memory, score: Float32, relevance: Float32) {
        this.memory = memory
        this.score = score
        this.relevance = relevance
    }
}

// 注意：仓颉不支持typealias，直接使用基础类型
// Vector = Array<Float32>
// Timestamp = UInt64

/// 内存统计信息
public class MemoryStats {
    public var totalMemories: UInt64
    public var episodicCount: UInt64
    public var semanticCount: UInt64
    public var proceduralCount: UInt64
    public var workingCount: UInt64
    public var averageImportance: Float32
    public var totalSize: UInt64
    public var lastUpdated: UInt64

    public init(
        totalMemories: UInt64,
        episodicCount: UInt64,
        semanticCount: UInt64,
        proceduralCount: UInt64,
        workingCount: UInt64,
        averageImportance: Float32,
        totalSize: UInt64,
        lastUpdated: UInt64
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
}

/// 批处理结果
public class BatchResult<T> {
    public var total: UInt32
    public var successes: Array<T>
    public var failures: Array<String>

    public init() {
        this.total = 0
        this.successes = Array<T>()
        this.failures = Array<String>()
    }

    public func addSuccess(item: T): Unit {
        this.successes = ArrayUtils.append(this.successes, item)
        return ()
    }

    public func addFailure(error: String): Unit {
        this.failures = ArrayUtils.append(this.failures, error)
        return ()
    }
}

/// 分页参数
public class PaginationParams {
    public var page: UInt32
    public var pageSize: UInt32
    public var sortBy: Option<String>
    public var sortOrder: Option<String>

    public init(page: UInt32, pageSize: UInt32) {
        this.page = page
        this.pageSize = pageSize
        this.sortBy = None<String>
        this.sortOrder = None<String>
    }
}

/// 分页结果
public class PaginatedResult<T> {
    public var items: Array<T>
    public var totalCount: UInt64
    public var currentPage: UInt32
    public var totalPages: UInt32
    public var hasNext: Bool
    public var hasPrevious: Bool

    public init(
        items: Array<T>,
        totalCount: UInt64,
        currentPage: UInt32,
        totalPages: UInt32,
        hasNext: Bool,
        hasPrevious: Bool
    ) {
        this.items = items
        this.totalCount = totalCount
        this.currentPage = currentPage
        this.totalPages = totalPages
        this.hasNext = hasNext
        this.hasPrevious = hasPrevious
    }
}
