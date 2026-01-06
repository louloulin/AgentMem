/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * FFI绑定层 - AgentMem 仓颉 SDK 的C函数绑定
 */

package agentmem.ffi

import agentmem.core.*

/// C 结构体映射 - 记忆对象
@C
public struct CMemory {
    /// 记忆ID
    public var id: CString = CString()
    /// 代理ID
    public var agent_id: CString = CString()
    /// 用户ID（可选，空字符串表示None）
    public var user_id: CString = CString()
    /// 记忆类型（数值表示）
    public var memory_type: UInt32 = 0
    /// 记忆内容
    public var content: CString = CString()
    /// 重要性分数
    public var importance: Float32 = 0.0
    /// 创建时间戳
    public var created_at: Int64 = 0
    /// 最后访问时间戳
    public var last_accessed_at: Int64 = 0
    /// 访问次数
    public var access_count: UInt32 = 0
    /// 过期时间戳（0表示不过期）
    public var expires_at: Int64 = 0
    /// 版本号
    public var version: UInt32 = 0
    /// 是否有嵌入向量
    public var has_embedding: Bool = false
    /// 嵌入向量长度
    public var embedding_len: UInt32 = 0
    /// 嵌入向量数据指针
    public var embedding_data: CPointer<Float32> = CPointer<Float32>()

    public init() {}
}

/// C 结构体映射 - 记忆数组
@C
public struct CMemoryArray {
    /// 记忆数组指针
    public var memories: CPointer<CMemory> = CPointer<CMemory>()
    /// 数组长度
    public var count: UIntNative = 0
    /// 数组容量
    public var capacity: UIntNative = 0

    public init() {}
}

/// C 结构体映射 - 搜索结果
@C
public struct CSearchResult {
    /// 记忆对象
    public var memory: CMemory = CMemory()
    /// 相似度分数
    public var score: Float32 = 0.0
    /// 搜索排名
    public var rank: UInt32 = 0

    public init() {}
}

/// C 结构体映射 - 搜索结果数组
@C
public struct CSearchResultArray {
    /// 搜索结果数组指针
    public var results: CPointer<CSearchResult> = CPointer<CSearchResult>()
    /// 数组长度
    public var count: UIntNative = 0
    /// 数组容量
    public var capacity: UIntNative = 0

    public init() {}
}

/// C 结构体映射 - 批量操作结果
@C
public struct CBatchResult {
    /// 成功数量
    public var success_count: UInt32 = 0
    /// 失败数量
    public var failure_count: UInt32 = 0
    /// 总数量
    public var total_count: UInt32 = 0
    /// 错误消息数组
    public var error_messages: CPointer<CString> = CPointer<CString>()
    /// 错误消息数量
    public var error_count: UIntNative = 0

    public init() {}
}

/// C 结构体映射 - 统计信息
@C
public struct CMemoryStats {
    /// 总记忆数量
    public var total_memories: UInt32 = 0
    /// 各类型记忆数量（按MemoryType枚举顺序）
    public var type_counts: VArray<UInt32, $4> = VArray<UInt32, $4>(repeat: 0)
    /// 平均重要性
    public var average_importance: Float32 = 0.0
    /// 最高重要性
    public var max_importance: Float32 = 0.0
    /// 最低重要性
    public var min_importance: Float32 = 0.0
    /// 最近创建时间
    public var last_created_at: Int64 = 0
    /// 最近访问时间
    public var last_accessed_at: Int64 = 0

    public init() {}
}

/// C 结构体映射 - 分页结果
@C
public struct CPaginatedResult {
    /// 记忆数组
    public var memories: CMemoryArray = CMemoryArray()
    /// 当前页码
    public var page: UInt32 = 0
    /// 每页大小
    public var page_size: UInt32 = 0
    /// 总数量
    public var total_count: UInt32 = 0
    /// 总页数
    public var total_pages: UInt32 = 0

    public init() {}
}

/// FFI 函数声明 - 客户端管理
foreign func agentmem_client_new(config_json: CString): CPointer<Unit>
foreign func agentmem_client_destroy(client: CPointer<Unit>): Unit
foreign func agentmem_client_is_connected(client: CPointer<Unit>): Bool

/// FFI 函数声明 - 记忆操作
foreign func agentmem_add_memory(
    client: CPointer<Unit>,
    memory: CPointer<CMemory>
): CString

foreign func agentmem_get_memory(
    client: CPointer<Unit>,
    memory_id: CString,
    out_memory: CPointer<CMemory>
): Int32

foreign func agentmem_update_memory(
    client: CPointer<Unit>,
    memory_id: CString,
    content: CString
): Int32

foreign func agentmem_update_memory_full(
    client: CPointer<Unit>,
    memory: CPointer<CMemory>
): Int32

foreign func agentmem_delete_memory(
    client: CPointer<Unit>,
    memory_id: CString
): Int32

foreign func agentmem_memory_exists(
    client: CPointer<Unit>,
    memory_id: CString
): Bool

/// FFI 函数声明 - 搜索操作
foreign func agentmem_search_memories(
    client: CPointer<Unit>,
    query: CString,
    limit: UInt32,
    out_results: CPointer<CSearchResultArray>
): Int32

foreign func agentmem_search_memories_filtered(
    client: CPointer<Unit>,
    query: CString,
    filter_json: CString,
    limit: UInt32,
    out_results: CPointer<CSearchResultArray>
): Int32

foreign func agentmem_search_similar_memories(
    client: CPointer<Unit>,
    memory_id: CString,
    limit: UInt32,
    threshold: Float32,
    out_results: CPointer<CSearchResultArray>
): Int32

/// FFI 函数声明 - 批量操作
foreign func agentmem_add_memories_batch(
    client: CPointer<Unit>,
    memories: CPointer<CMemory>,
    count: UIntNative,
    out_result: CPointer<CBatchResult>
): Int32

foreign func agentmem_delete_memories_batch(
    client: CPointer<Unit>,
    memory_ids: CPointer<CString>,
    count: UIntNative,
    out_result: CPointer<CBatchResult>
): Int32

/// FFI 函数声明 - 分页查询
foreign func agentmem_get_memories_paginated(
    client: CPointer<Unit>,
    agent_id: CString,
    page: UInt32,
    page_size: UInt32,
    out_result: CPointer<CPaginatedResult>
): Int32

foreign func agentmem_get_memories_by_type_paginated(
    client: CPointer<Unit>,
    agent_id: CString,
    memory_type: UInt32,
    page: UInt32,
    page_size: UInt32,
    out_result: CPointer<CPaginatedResult>
): Int32

/// FFI 函数声明 - 统计信息
foreign func agentmem_get_memory_stats(
    client: CPointer<Unit>,
    agent_id: CString,
    out_stats: CPointer<CMemoryStats>
): Int32

foreign func agentmem_get_global_stats(
    client: CPointer<Unit>,
    out_stats: CPointer<CMemoryStats>
): Int32

/// FFI 函数声明 - 内存管理
foreign func agentmem_free_string(str: CString): Unit
foreign func agentmem_free_memory_array(arr: CPointer<CMemoryArray>): Unit
foreign func agentmem_free_search_result_array(arr: CPointer<CSearchResultArray>): Unit
foreign func agentmem_free_batch_result(result: CPointer<CBatchResult>): Unit
foreign func agentmem_free_paginated_result(result: CPointer<CPaginatedResult>): Unit
foreign func agentmem_free_memory_stats(stats: CPointer<CMemoryStats>): Unit

/// FFI 函数声明 - 错误处理
foreign func agentmem_get_last_error(): CString
foreign func agentmem_get_last_error_code(): UInt32
foreign func agentmem_clear_last_error(): Unit

/// FFI 函数声明 - 配置和调试
foreign func agentmem_set_log_level(level: UInt32): Unit
foreign func agentmem_get_version(): CString
foreign func agentmem_health_check(client: CPointer<Unit>): Bool

/// FFI 函数声明 - 高级功能
foreign func agentmem_compress_memories(
    client: CPointer<Unit>,
    agent_id: CString,
    compression_ratio: Float32
): Int32

foreign func agentmem_export_memories(
    client: CPointer<Unit>,
    agent_id: CString,
    format: CString,
    output_path: CString
): Int32

foreign func agentmem_import_memories(
    client: CPointer<Unit>,
    agent_id: CString,
    format: CString,
    input_path: CString,
    out_result: CPointer<CBatchResult>
): Int32

/// FFI 函数声明 - 向量操作
foreign func agentmem_generate_embedding(
    client: CPointer<Unit>,
    text: CString,
    out_embedding: CPointer<CPointer<Float32>>,
    out_length: CPointer<UInt32>
): Int32

foreign func agentmem_free_embedding(embedding: CPointer<Float32>): Unit

/// FFI 函数声明 - 关系和图操作
foreign func agentmem_add_memory_relation(
    client: CPointer<Unit>,
    from_memory_id: CString,
    to_memory_id: CString,
    relation_type: CString,
    strength: Float32
): Int32

foreign func agentmem_get_related_memories(
    client: CPointer<Unit>,
    memory_id: CString,
    relation_type: CString,
    max_depth: UInt32,
    out_results: CPointer<CSearchResultArray>
): Int32

foreign func agentmem_find_memory_path(
    client: CPointer<Unit>,
    from_memory_id: CString,
    to_memory_id: CString,
    max_depth: UInt32,
    out_path: CPointer<CPointer<CString>>,
    out_path_length: CPointer<UInt32>
): Int32

foreign func agentmem_free_string_array(arr: CPointer<CString>, length: UInt32): Unit
