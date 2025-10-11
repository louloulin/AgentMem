# AgentDB 仓颉 FFI 集成计划

## 项目概述

基于对 tree-sitter.cj 项目 FFI 实现的深度分析，制定为 AgentDB 添加仓颉（Cangjie）语言 FFI 支持的完整计划。该计划将在现有的 Rust+Zig+C FFI 架构基础上，增加仓颉语言绑定，形成四语言混合架构。

## 技术分析

### 仓颉 FFI 特性分析

基于 tree-sitter.cj 的实现，仓颉 FFI 具有以下特点：

1. **Foreign 函数声明**：使用 `foreign func` 关键字声明 C 函数
2. **类型安全**：支持 C 类型到仓颉类型的安全映射
3. **内存管理**：提供手动内存管理和安全包装
4. **结构体映射**：使用 `@C` 注解映射 C 结构体
5. **动态库加载**：支持 dlopen/dlsym 动态加载
6. **包装函数**：通过 public 函数封装 unsafe FFI 调用

### AgentDB 现有 FFI 架构

```
当前架构：
应用层 → Zig API → C FFI → Rust 核心

目标架构：
应用层 → 仓颉 API → C FFI → Rust 核心
       → Zig API   ↗
```

## 实施计划

### 阶段一：基础架构搭建（第1-2周）

#### 1.1 项目结构创建
```
AgentDB/
├── agent-db-cangjie/           # 新增仓颉模块
│   ├── src/
│   │   ├── core/               # 核心类型定义
│   │   ├── ffi/                # FFI 绑定层
│   │   ├── api/                # 高级 API 层
│   │   ├── examples/           # 使用示例
│   │   └── tests/              # 测试套件
│   ├── cjpm.toml               # 仓颉包配置
│   └── README.md               # 模块文档
```

#### 1.2 核心类型定义 (src/core/types.cj)
```cangjie
package agentdb.core

// 错误码枚举
public enum AgentDbErrorCode {
    | Success
    | InvalidParam
    | NotFound
    | IoError
    | MemoryError
    | InternalError
    | SerializationError
    | NetworkError
    | AuthenticationError
    | PermissionDenied

    public func toInt32(): Int32 {
        return match (this) {
            case Success => 0
            case InvalidParam => -1
            case NotFound => -2
            case IoError => -3
            case MemoryError => -4
            case InternalError => -5
            case SerializationError => -6
            case NetworkError => -7
            case AuthenticationError => -8
            case PermissionDenied => -9
        }
    }
}

// 状态类型枚举
public enum StateType {
    | WorkingMemory
    | LongTermMemory
    | Context
    | TaskState
    | Relationship
    | Embedding

    public func toUInt32(): UInt32 {
        return match (this) {
            case WorkingMemory => 0
            case LongTermMemory => 1
            case Context => 2
            case TaskState => 3
            case Relationship => 4
            case Embedding => 5
        }
    }
}

// 记忆类型枚举
public enum MemoryType {
    | Episodic
    | Semantic
    | Procedural
    | Working

    public func toUInt32(): UInt32 {
        return match (this) {
            case Episodic => 0
            case Semantic => 1
            case Procedural => 2
            case Working => 3
        }
    }
}
```

#### 1.3 FFI 绑定层 (src/ffi/bindings.cj)
```cangjie
package agentdb.ffi

// C 结构体声明
@C
public struct CAgentStateDB {
    public var inner: CPointer<Unit> = CPointer<Unit>()
}

// FFI 函数声明
foreign func agent_db_create(db_path: CString): CPointer<CAgentStateDB>
foreign func agent_db_destroy(db: CPointer<CAgentStateDB>): Unit
foreign func agent_db_save_state(
    db: CPointer<CAgentStateDB>,
    agent_id: UInt64,
    state_type: UInt32,
    data: CString,
    data_len: UIntNative
): Int32
foreign func agent_db_load_state(
    db: CPointer<CAgentStateDB>,
    agent_id: UInt64,
    out_data: CPointer<CPointer<UInt8>>,
    out_len: CPointer<UIntNative>
): Int32
foreign func agent_db_vector_search(
    db: CPointer<CAgentStateDB>,
    query_vector: CPointer<Float32>,
    vector_len: UIntNative,
    limit: UIntNative,
    out_results: CPointer<CPointer<UInt64>>,
    out_count: CPointer<UIntNative>
): Int32
foreign func agent_db_free_memory(ptr: CPointer<UInt8>): Unit
foreign func agent_db_free_results(results: CPointer<UInt64>, count: UIntNative): Unit
foreign func agent_db_get_error_message(error_code: Int32): CString
foreign func agent_db_version(): CString
```

### 阶段二：高级 API 实现（第3-4周）

#### 2.1 数据库管理类 (src/api/database.cj)
```cangjie
package agentdb.api

import agentdb.core.*
import agentdb.ffi.*

public class AgentDatabase {
    private var _handle: ?CPointer<CAgentStateDB> = Option<CPointer<CAgentStateDB>>.None
    private var _path: String = ""
    private var _initialized: Bool = false

    public init(dbPath: String) {
        this._path = dbPath
    }

    public func open(): Result<Unit, AgentDbError> {
        let cPath = stringToCString(this._path)
        let handle = unsafe { agent_db_create(cPath) }
        unsafe { LibC.free(cPath) }

        if (handle.isNull()) {
            return Result<Unit, AgentDbError>.Err(
                AgentDbError("Failed to create database")
            )
        }

        this._handle = Option<CPointer<CAgentStateDB>>.Some(handle)
        this._initialized = true
        return Result<Unit, AgentDbError>.Ok(Unit())
    }

    public func close(): Unit {
        if (this._initialized && this._handle.isSome()) {
            let handle = this._handle.unwrap()
            unsafe { agent_db_destroy(handle) }
            this._handle = Option<CPointer<CAgentStateDB>>.None
            this._initialized = false
        }
    }
}
```

#### 2.2 Agent 状态管理 (src/api/agent_state.cj)
```cangjie
package agentdb.api

public struct AgentState {
    public var agentId: UInt64
    public var sessionId: UInt64
    public var stateType: StateType
    public var data: Array<UInt8>
    public var metadata: HashMap<String, String>
    public var timestamp: Int64

    public init(
        agentId: UInt64,
        sessionId: UInt64,
        stateType: StateType,
        data: Array<UInt8>
    ) {
        this.agentId = agentId
        this.sessionId = sessionId
        this.stateType = stateType
        this.data = data
        this.metadata = HashMap<String, String>()
        this.timestamp = getCurrentTimestamp()
    }

    public func setMetadata(key: String, value: String): Unit {
        this.metadata[key] = value
    }

    public func getMetadata(key: String): ?String {
        return this.metadata.get(key)
    }
}

extend AgentDatabase {
    public func saveState(state: AgentState): Result<Unit, AgentDbError> {
        if (!this._initialized || this._handle.isNone()) {
            return Result<Unit, AgentDbError>.Err(
                AgentDbError("Database not initialized")
            )
        }

        let handle = this._handle.unwrap()
        let dataStr = arrayToString(state.data)
        let cData = stringToCString(dataStr)

        let result = unsafe {
            agent_db_save_state(
                handle,
                state.agentId,
                state.stateType.toUInt32(),
                cData,
                dataStr.size
            )
        }

        unsafe { LibC.free(cData) }

        if (result == 0) {
            return Result<Unit, AgentDbError>.Ok(Unit())
        } else {
            return Result<Unit, AgentDbError>.Err(
                AgentDbError("Failed to save state")
            )
        }
    }

    public func loadState(agentId: UInt64): Result<?AgentState, AgentDbError> {
        if (!this._initialized || this._handle.isNone()) {
            return Result<?AgentState, AgentDbError>.Err(
                AgentDbError("Database not initialized")
            )
        }

        let handle = this._handle.unwrap()
        var outData: CPointer<UInt8> = CPointer<UInt8>()
        var outLen: UIntNative = 0

        let result = unsafe {
            agent_db_load_state(
                handle,
                agentId,
                CPointer.addressOf(outData),
                CPointer.addressOf(outLen)
            )
        }

        if (result == 0 && !outData.isNull()) {
            let data = cstringToArray(outData, outLen)
            unsafe { agent_db_free_memory(outData) }

            let state = AgentState(
                agentId,
                0,
                StateType.WorkingMemory,
                data
            )
            return Result<?AgentState, AgentDbError>.Ok(Option<AgentState>.Some(state))
        } else {
            return Result<?AgentState, AgentDbError>.Ok(Option<AgentState>.None)
        }
    }
}
```

### 阶段三：高级功能实现（第5-6周）

#### 3.1 记忆管理 (src/api/memory.cj)
#### 3.2 向量搜索 (src/api/vector.cj)
#### 3.3 RAG 引擎 (src/api/rag.cj)
#### 3.4 安全管理 (src/api/security.cj)

### 阶段四：工具和示例（第7-8周）

#### 4.1 内存管理工具 (src/ffi/memory.cj)
#### 4.2 字符串转换工具 (src/ffi/utils.cj)
#### 4.3 使用示例 (src/examples/)
#### 4.4 测试套件 (src/tests/)

### 阶段五：集成和优化（第9-10周）

#### 5.1 构建系统集成
#### 5.2 文档完善
#### 5.3 性能优化
#### 5.4 错误处理完善

## 技术要点

### 1. 内存管理策略
- 使用 RAII 模式管理资源
- 提供安全的字符串转换函数
- 实现自动内存清理机制

### 2. 错误处理机制
- 使用 Result 类型处理错误
- 提供详细的错误信息
- 实现错误传播机制

### 3. 类型安全保证
- 使用强类型系统
- 提供类型转换函数
- 避免直接使用 unsafe 操作

### 4. 性能优化
- 最小化 FFI 调用开销
- 使用批量操作接口
- 实现连接池管理

## 预期成果

### 功能完整性
- 完整的 AgentDB 功能覆盖
- 类型安全的 API 接口
- 丰富的使用示例

### 性能指标
- FFI 调用延迟 < 1ms
- 内存使用效率 > 95%
- 错误处理覆盖率 100%

### 文档质量
- 完整的 API 文档
- 详细的使用指南
- 丰富的示例代码

## 风险评估

### 技术风险
- 仓颉 FFI 兼容性问题
- 内存管理复杂性
- 类型转换开销

### 缓解措施
- 充分的兼容性测试
- 完善的内存管理工具
- 性能基准测试

## 总结

该计划将为 AgentDB 提供完整的仓颉语言支持，形成业界首个四语言混合架构的 AI Agent 数据库。通过借鉴 tree-sitter.cj 的成功经验，确保实现的稳定性和性能。预计10周内完成全部开发工作，为仓颉生态系统贡献重要的基础设施组件。
