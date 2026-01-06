/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 内存管理 - AgentMem 仓颉 SDK 的FFI内存管理工具
 */

package agentmem.ffi

import agentmem.core.*
import std.collection.HashMap

/// FFI内存管理器 - 负责C内存的安全管理
public class FFIMemoryManager {
    /// 跟踪分配的C内存指针
    private var _allocatedPointers: HashMap<UIntNative, Bool>
    /// 是否启用内存跟踪
    private var _trackingEnabled: Bool

    public init(trackingEnabled: Bool = true) {
        this._allocatedPointers = HashMap<UIntNative, Bool>()
        this._trackingEnabled = trackingEnabled
    }

    /// 注册分配的指针
    public func registerPointer(ptr: CPointer<Unit>): Unit {
        if (this._trackingEnabled && !ptr.isNull()) {
            let address = ptr.toUIntNative()
            this._allocatedPointers[address] = true
        }
    }

    /// 注销指针
    public func unregisterPointer(ptr: CPointer<Unit>): Unit {
        if (this._trackingEnabled && !ptr.isNull()) {
            let address = ptr.toUIntNative()
            this._allocatedPointers.remove(address)
        }
    }

    /// 检查指针是否已注册
    public func isRegistered(ptr: CPointer<Unit>): Bool {
        if (!this._trackingEnabled || ptr.isNull()) {
            return false
        }
        let address = ptr.toUIntNative()
        return this._allocatedPointers.get(address).getOrElse(false)
    }

    /// 获取已分配指针数量
    public func getAllocatedCount(): UIntNative {
        return UIntNative(this._allocatedPointers.size)
    }

    /// 清理所有跟踪的指针（仅用于调试）
    public func clearTracking(): Unit {
        this._allocatedPointers.clear()
    }
}

/// 全局FFI内存管理器实例
let globalFFIMemoryManager = FFIMemoryManager()

/// 安全的C字符串转换工具
public class CStringConverter {
    /// 将仓颉字符串转换为C字符串
    public static func toCString(str: String): CString {
        return unsafe { LibC.mallocCString(str) }
    }

    /// 将C字符串转换为仓颉字符串
    public static func toString(cstr: CString): String {
        if (cstr.isNull()) {
            return ""
        }
        return cstr.toString()
    }

    /// 安全释放C字符串
    public static func freeCString(cstr: CString): Unit {
        if (!cstr.isNull()) {
            unsafe { LibC.free(cstr.getChars()) }
        }
    }

    /// 创建C字符串数组
    public static func createCStringArray(strings: Array<String>): CPointer<CString> {
        let count = strings.size
        if (count == 0) {
            return CPointer<CString>()
        }

        let arrayPtr = unsafe { LibC.malloc(UIntNative(count * 8)) } // 假设指针大小为8字节
        let cstringArray = unsafe { CPointer<CString>(arrayPtr) }

        for (i in 0..count) {
            let cstr = CStringConverter.toCString(strings[i])
            unsafe { (cstringArray + UIntNative(i)).write(cstr) }
        }

        return cstringArray
    }

    /// 释放C字符串数组
    public static func freeCStringArray(arr: CPointer<CString>, count: UIntNative): Unit {
        if (arr.isNull() || count == 0) {
            return
        }

        for (i in 0..count) {
            let cstr = unsafe { (arr + i).read() }
            CStringConverter.freeCString(cstr)
        }

        unsafe { LibC.free(arr) }
    }
}

/// 记忆对象转换工具
public class MemoryConverter {
    /// 将仓颉Memory转换为C Memory
    public static func toCMemory(memory: Memory): CMemory {
        var cMemory = CMemory()
        
        cMemory.id = CStringConverter.toCString(memory.id)
        cMemory.agent_id = CStringConverter.toCString(memory.agentId)
        
        if (memory.userId.isSome()) {
            cMemory.user_id = CStringConverter.toCString(memory.userId.getOrThrow())
        } else {
            cMemory.user_id = CStringConverter.toCString("")
        }
        
        cMemory.memory_type = memory.memoryType.toUInt32()
        cMemory.content = CStringConverter.toCString(memory.content)
        cMemory.importance = memory.importance
        cMemory.created_at = memory.createdAt
        cMemory.last_accessed_at = memory.lastAccessedAt
        cMemory.access_count = memory.accessCount
        
        if (memory.expiresAt.isSome()) {
            cMemory.expires_at = memory.expiresAt.getOrThrow()
        } else {
            cMemory.expires_at = 0
        }
        
        cMemory.version = memory.version
        
        // 处理嵌入向量
        if (memory.embedding.isSome()) {
            let embedding = memory.embedding.getOrThrow()
            cMemory.has_embedding = true
            cMemory.embedding_len = UInt32(embedding.size)
            
            // 分配C内存存储向量数据
            let vectorSize = UIntNative(embedding.size * 4) // Float32 = 4 bytes
            let vectorPtr = unsafe { LibC.malloc(vectorSize) }
            cMemory.embedding_data = unsafe { CPointer<Float32>(vectorPtr) }
            
            // 复制向量数据
            for (i in 0..embedding.size) {
                unsafe { (cMemory.embedding_data + UIntNative(i)).write(embedding[i]) }
            }
        } else {
            cMemory.has_embedding = false
            cMemory.embedding_len = 0
            cMemory.embedding_data = CPointer<Float32>()
        }
        
        return cMemory
    }

    /// 将C Memory转换为仓颉Memory
    public static func fromCMemory(cMemory: CMemory): Memory {
        let id = CStringConverter.toString(cMemory.id)
        let agentId = CStringConverter.toString(cMemory.agent_id)
        let content = CStringConverter.toString(cMemory.content)
        
        let memoryType = MemoryType.fromUInt32(cMemory.memory_type).getOrElse(MemoryType.Semantic)
        
        let userId = if (cMemory.user_id.isEmpty()) {
            None
        } else {
            Some(CStringConverter.toString(cMemory.user_id))
        }
        
        let expiresAt = if (cMemory.expires_at == 0) {
            None
        } else {
            Some(cMemory.expires_at)
        }
        
        // 处理嵌入向量
        let embedding = if (cMemory.has_embedding && cMemory.embedding_len > 0) {
            var vector = Array<Float32>()
            for (i in 0..cMemory.embedding_len) {
                let value = unsafe { (cMemory.embedding_data + UIntNative(i)).read() }
                vector.append(value)
            }
            Some(vector)
        } else {
            None
        }
        
        let metadata = HashMap<String, String>() // 元数据需要单独处理
        
        return Memory(
            id,
            agentId,
            userId,
            memoryType,
            content,
            cMemory.importance,
            embedding,
            cMemory.created_at,
            cMemory.last_accessed_at,
            cMemory.access_count,
            expiresAt,
            metadata,
            cMemory.version
        )
    }

    /// 释放C Memory对象
    public static func freeCMemory(cMemory: CMemory): Unit {
        CStringConverter.freeCString(cMemory.id)
        CStringConverter.freeCString(cMemory.agent_id)
        CStringConverter.freeCString(cMemory.user_id)
        CStringConverter.freeCString(cMemory.content)
        
        if (cMemory.has_embedding && !cMemory.embedding_data.isNull()) {
            unsafe { LibC.free(cMemory.embedding_data) }
        }
    }

    /// 释放C Memory数组
    public static func freeCMemoryArray(arr: CMemoryArray): Unit {
        if (!arr.memories.isNull() && arr.count > 0) {
            for (i in 0..arr.count) {
                let cMemory = unsafe { (arr.memories + i).read() }
                MemoryConverter.freeCMemory(cMemory)
            }
            unsafe { LibC.free(arr.memories) }
        }
    }
}

/// 搜索结果转换工具
public class SearchResultConverter {
    /// 将C搜索结果转换为仓颉搜索结果
    public static func fromCSearchResult(cResult: CSearchResult): MemorySearchResult {
        let memory = MemoryConverter.fromCMemory(cResult.memory)
        return MemorySearchResult(memory, cResult.score, cResult.rank)
    }

    /// 将C搜索结果数组转换为仓颉数组
    public static func fromCSearchResultArray(cArray: CSearchResultArray): Array<MemorySearchResult> {
        var results = Array<MemorySearchResult>()
        
        if (!cArray.results.isNull() && cArray.count > 0) {
            for (i in 0..cArray.count) {
                let cResult = unsafe { (cArray.results + i).read() }
                let result = SearchResultConverter.fromCSearchResult(cResult)
                results.append(result)
            }
        }
        
        return results
    }

    /// 释放C搜索结果数组
    public static func freeCSearchResultArray(cArray: CSearchResultArray): Unit {
        if (!cArray.results.isNull() && cArray.count > 0) {
            for (i in 0..cArray.count) {
                let cResult = unsafe { (cArray.results + i).read() }
                MemoryConverter.freeCMemory(cResult.memory)
            }
            unsafe { LibC.free(cArray.results) }
        }
    }
}

/// 资源自动管理器 - RAII模式
public class ResourceGuard<T> {
    private var _resource: Option<T>
    private var _cleanup: (T) -> Unit

    public init(resource: T, cleanup: (T) -> Unit) {
        this._resource = Some(resource)
        this._cleanup = cleanup
    }

    /// 获取资源
    public func get(): Option<T> {
        return this._resource
    }

    /// 手动释放资源
    public func release(): Unit {
        if (this._resource.isSome()) {
            let resource = this._resource.getOrThrow()
            this._cleanup(resource)
            this._resource = None
        }
    }

    /// 析构函数 - 自动释放资源
    public func finalize(): Unit {
        this.release()
    }
}

/// 创建资源守护者的便利函数
public func createResourceGuard<T>(resource: T, cleanup: (T) -> Unit): ResourceGuard<T> {
    return ResourceGuard<T>(resource, cleanup)
}

/// 错误处理工具
public class FFIErrorHandler {
    /// 获取最后的FFI错误
    public static func getLastError(): AgentMemError {
        let errorCode = unsafe { agentmem_get_last_error_code() }
        let errorMessage = unsafe { agentmem_get_last_error() }
        let message = CStringConverter.toString(errorMessage)
        
        // 释放错误消息字符串
        unsafe { agentmem_free_string(errorMessage) }
        
        return AgentMemError.fromErrorCode(errorCode, message).getOrElse(
            AgentMemError.InternalError(message)
        )
    }

    /// 清除最后的错误
    public static func clearLastError(): Unit {
        unsafe { agentmem_clear_last_error() }
    }

    /// 检查FFI调用结果并转换错误
    public static func checkResult(result: Int32): AgentMemResult<Unit> {
        if (result == 0) {
            return AgentMemResult<Unit>.Ok(Unit())
        } else {
            let error = FFIErrorHandler.getLastError()
            return AgentMemResult<Unit>.Err(error)
        }
    }
}
