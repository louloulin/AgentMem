/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 错误处理 - AgentMem 仓颉 SDK 的错误类型和处理机制
 */

package agentmem.core

// import std.result.Result
import agentmem.utils.{SimpleMap, StringUtils, Timestamp, TimeUtils}

/// AgentMem 错误类型枚举
public enum AgentMemError {
    /// 无效参数错误
    | InvalidParameter(String)
    /// 资源未找到错误
    | NotFound(String)
    /// 网络连接错误
    | NetworkError(String)
    /// 未连接错误
    | NotConnected(String)
    /// 序列化/反序列化错误
    | SerializationError(String)
    /// 身份验证错误
    | AuthenticationError(String)
    /// 授权错误
    | AuthorizationError(String)
    /// 内部服务器错误
    | InternalError(String)
    /// FFI调用错误
    | FFIError(String)
    /// 超时错误
    | TimeoutError(String)
    /// 配置错误
    | ConfigurationError(String)
    /// 缓存错误
    | CacheError(String)
    /// 数据库错误
    | DatabaseError(String)
    /// 向量操作错误
    | VectorError(String)
    /// 内存不足错误
    | OutOfMemoryError(String)
    /// 并发错误
    | ConcurrencyError(String)
    /// 版本冲突错误
    | VersionConflictError(String)
    /// 限流错误
    | RateLimitError(String)
    /// 服务不可用错误
    | ServiceUnavailableError(String)

    /// 获取错误消息
    public func getMessage(): String {
        return match (this) {
            case InvalidParameter(msg) => "Invalid parameter: ${msg}"
            case NotFound(msg) => "Not found: ${msg}"
            case NetworkError(msg) => "Network error: ${msg}"
            case NotConnected(msg) => "Not connected: ${msg}"
            case SerializationError(msg) => "Serialization error: ${msg}"
            case AuthenticationError(msg) => "Authentication error: ${msg}"
            case AuthorizationError(msg) => "Authorization error: ${msg}"
            case InternalError(msg) => "Internal error: ${msg}"
            case FFIError(msg) => "FFI error: ${msg}"
            case TimeoutError(msg) => "Timeout error: ${msg}"
            case ConfigurationError(msg) => "Configuration error: ${msg}"
            case CacheError(msg) => "Cache error: ${msg}"
            case DatabaseError(msg) => "Database error: ${msg}"
            case VectorError(msg) => "Vector error: ${msg}"
            case OutOfMemoryError(msg) => "Out of memory error: ${msg}"
            case ConcurrencyError(msg) => "Concurrency error: ${msg}"
            case VersionConflictError(msg) => "Version conflict error: ${msg}"
            case RateLimitError(msg) => "Rate limit error: ${msg}"
            case ServiceUnavailableError(msg) => "Service unavailable error: ${msg}"
        }
    }

    /// 获取错误代码
    public func getErrorCode(): UInt32 {
        return match (this) {
            case InvalidParameter(_) => 1001
            case NotFound(_) => 1002
            case NetworkError(_) => 1003
            case NotConnected(_) => 1019
            case SerializationError(_) => 1004
            case AuthenticationError(_) => 1005
            case AuthorizationError(_) => 1006
            case InternalError(_) => 1007
            case FFIError(_) => 1008
            case TimeoutError(_) => 1009
            case ConfigurationError(_) => 1010
            case CacheError(_) => 1011
            case DatabaseError(_) => 1012
            case VectorError(_) => 1013
            case OutOfMemoryError(_) => 1014
            case ConcurrencyError(_) => 1015
            case VersionConflictError(_) => 1016
            case RateLimitError(_) => 1017
            case ServiceUnavailableError(_) => 1018
        }
    }

    /// 获取错误类型名称
    public func getErrorType(): String {
        return match (this) {
            case InvalidParameter(_) => "InvalidParameter"
            case NotFound(_) => "NotFound"
            case NetworkError(_) => "NetworkError"
            case NotConnected(_) => "NotConnected"
            case SerializationError(_) => "SerializationError"
            case AuthenticationError(_) => "AuthenticationError"
            case AuthorizationError(_) => "AuthorizationError"
            case InternalError(_) => "InternalError"
            case FFIError(_) => "FFIError"
            case TimeoutError(_) => "TimeoutError"
            case ConfigurationError(_) => "ConfigurationError"
            case CacheError(_) => "CacheError"
            case DatabaseError(_) => "DatabaseError"
            case VectorError(_) => "VectorError"
            case OutOfMemoryError(_) => "OutOfMemoryError"
            case ConcurrencyError(_) => "ConcurrencyError"
            case VersionConflictError(_) => "VersionConflictError"
            case RateLimitError(_) => "RateLimitError"
            case ServiceUnavailableError(_) => "ServiceUnavailableError"
        }
    }

    /// 判断是否为可重试错误
    public func isRetryable(): Bool {
        return match (this) {
            case NetworkError(_) => true
            case TimeoutError(_) => true
            case InternalError(_) => true
            case ServiceUnavailableError(_) => true
            case RateLimitError(_) => true
            case _ => false
        }
    }

    /// 判断是否为临时错误
    public func isTemporary(): Bool {
        return match (this) {
            case NetworkError(_) => true
            case TimeoutError(_) => true
            case ServiceUnavailableError(_) => true
            case RateLimitError(_) => true
            case ConcurrencyError(_) => true
            case _ => false
        }
    }

    /// 转换为字符串表示
    public func toString(): String {
        return "${this.getErrorType()}(${this.getErrorCode()}): ${this.getMessage()}"
    }

    /// 从错误代码创建错误
    public static func fromErrorCode(code: UInt32, message: String): Option<AgentMemError> {
        return match (code) {
            case 1001 => Some(InvalidParameter(message))
            case 1002 => Some(NotFound(message))
            case 1003 => Some(NetworkError(message))
            case 1004 => Some(SerializationError(message))
            case 1005 => Some(AuthenticationError(message))
            case 1006 => Some(AuthorizationError(message))
            case 1007 => Some(InternalError(message))
            case 1008 => Some(FFIError(message))
            case 1009 => Some(TimeoutError(message))
            case 1010 => Some(ConfigurationError(message))
            case 1011 => Some(CacheError(message))
            case 1012 => Some(DatabaseError(message))
            case 1013 => Some(VectorError(message))
            case 1014 => Some(OutOfMemoryError(message))
            case 1015 => Some(ConcurrencyError(message))
            case 1016 => Some(VersionConflictError(message))
            case 1017 => Some(RateLimitError(message))
            case 1018 => Some(ServiceUnavailableError(message))
            case _ => None
        }
    }
}

/// AgentMem 结果类型
public enum AgentMemResult<T> {
    | Ok(T)
    | Err(AgentMemError)

    /// 判断是否成功
    public func isOk(): Bool {
        return match (this) {
            case Ok(_) => true
            case Err(_) => false
        }
    }

    /// 判断是否失败
    public func isErr(): Bool {
        return !this.isOk()
    }

    /// 获取错误值（如果存在）
    public func getErr(): AgentMemError {
        return this.unwrapErr()
    }

    /// 获取成功值（如果存在）
    public func getOk(): T {
        return this.unwrap()
    }

    /// 获取成功值（如果存在）
    public func unwrap(): T {
        return match (this) {
            case Ok(value) => value
            case Err(error) => throw Exception(error.getMessage())
        }
    }

    /// 获取错误（如果存在）
    public func unwrapErr(): AgentMemError {
        return match (this) {
            case Ok(_) => throw Exception("Called unwrapErr on Ok value")
            case Err(error) => error
        }
    }
}

/// 错误上下文信息
public class ErrorContext {
    /// 操作名称
    public var operation: String
    /// 时间戳
    public var timestamp: Timestamp
    /// 请求ID
    public var requestId: Option<String>
    /// 用户ID
    public var userId: Option<String>
    /// 代理ID
    public var agentId: Option<String>
    /// 额外信息
    public var metadata: SimpleMap

    public init(operation: String) {
        this.operation = operation
        this.timestamp = TimeUtils.getCurrentTimestamp()
        this.requestId = None
        this.userId = None
        this.agentId = None
        this.metadata = SimpleMap()
    }

    /// 设置请求ID
    public func withRequestId(requestId: String): ErrorContext {
        this.requestId = Some(requestId)
        return this
    }

    /// 设置用户ID
    public func withUserId(userId: String): ErrorContext {
        this.userId = Some(userId)
        return this
    }

    /// 设置代理ID
    public func withAgentId(agentId: String): ErrorContext {
        this.agentId = Some(agentId)
        return this
    }

    /// 添加元数据
    public func withMetadata(key: String, value: String): ErrorContext {
        this.metadata.set(key, value)
        return this
    }
}

/// 错误详情
public class ErrorDetails {
    /// 错误对象
    public var error: AgentMemError
    /// 错误上下文
    public var context: ErrorContext
    /// 堆栈跟踪
    public var stackTrace: Option<String>
    /// 内部错误
    public var innerError: Option<AgentMemError>

    public init(error: AgentMemError, context: ErrorContext) {
        this.error = error
        this.context = context
        this.stackTrace = None
        this.innerError = None
    }

    /// 设置堆栈跟踪
    public func withStackTrace(stackTrace: String): ErrorDetails {
        this.stackTrace = Some(stackTrace)
        return this
    }

    /// 设置内部错误
    public func withInnerError(innerError: AgentMemError): ErrorDetails {
        this.innerError = Some(innerError)
        return this
    }

    /// 转换为字符串表示
    public func toString(): String {
        var result = "Error: ${this.error.toString()}\n"
        result += "Operation: ${this.context.operation}\n"
        result += "Timestamp: ${this.context.timestamp}\n"
        
        if (this.context.requestId.isSome()) {
            result += "Request ID: ${this.context.requestId.getOrThrow()}\n"
        }
        
        if (this.context.userId.isSome()) {
            result += "User ID: ${this.context.userId.getOrThrow()}\n"
        }
        
        if (this.context.agentId.isSome()) {
            result += "Agent ID: ${this.context.agentId.getOrThrow()}\n"
        }
        
        if (!this.context.metadata.isEmpty()) {
            result += "Metadata:\n"
            result += "  ${this.context.metadata.toString()}\n"
        }
        
        if (this.stackTrace.isSome()) {
            result += "Stack Trace:\n${this.stackTrace.getOrThrow()}\n"
        }
        
        if (this.innerError.isSome()) {
            result += "Inner Error: ${this.innerError.getOrThrow().toString()}\n"
        }
        
        return result
    }
}

/// 错误处理器接口
public interface ErrorHandler {
    /// 处理错误
    func handleError(error: ErrorDetails): Unit
}

/// 默认错误处理器
public class DefaultErrorHandler <: ErrorHandler {
    private var _logLevel: LogLevel

    public init(logLevel: LogLevel) {
        this._logLevel = logLevel
    }

    public init() {
        this._logLevel = LogLevel.Error
    }

    public func handleError(error: ErrorDetails): Unit {
        // 根据日志级别决定是否输出错误信息
        if (this.shouldLog(error.error)) {
            println(error.toString())
        }
    }

    private func shouldLog(error: AgentMemError): Bool {
        return match (this._logLevel) {
            case LogLevel.Trace => true
            case LogLevel.Debug => true
            case LogLevel.Info => true
            case LogLevel.Warn => this.isWarningOrHigher(error)
            case LogLevel.Error => this.isErrorOrHigher(error)
            case LogLevel.Fatal => this.isFatal(error)
        }
    }

    private func isWarningOrHigher(error: AgentMemError): Bool {
        return match (error) {
            case InvalidParameter(_) => true
            case NotFound(_) => false
            case NetworkError(_) => true
            case NotConnected(_) => true
            case SerializationError(_) => true
            case AuthenticationError(_) => true
            case AuthorizationError(_) => true
            case InternalError(_) => true
            case FFIError(_) => true
            case TimeoutError(_) => true
            case ConfigurationError(_) => true
            case CacheError(_) => false
            case DatabaseError(_) => true
            case VectorError(_) => true
            case OutOfMemoryError(_) => true
            case ConcurrencyError(_) => true
            case VersionConflictError(_) => true
            case RateLimitError(_) => false
            case ServiceUnavailableError(_) => true
        }
    }

    private func isErrorOrHigher(error: AgentMemError): Bool {
        return match (error) {
            case InvalidParameter(_) => true
            case NotFound(_) => false
            case NetworkError(_) => true
            case NotConnected(_) => true
            case SerializationError(_) => true
            case AuthenticationError(_) => true
            case AuthorizationError(_) => true
            case InternalError(_) => true
            case FFIError(_) => true
            case TimeoutError(_) => false
            case ConfigurationError(_) => true
            case CacheError(_) => false
            case DatabaseError(_) => true
            case VectorError(_) => true
            case OutOfMemoryError(_) => true
            case ConcurrencyError(_) => true
            case VersionConflictError(_) => false
            case RateLimitError(_) => false
            case ServiceUnavailableError(_) => false
        }
    }

    private func isFatal(error: AgentMemError): Bool {
        return match (error) {
            case OutOfMemoryError(_) => true
            case FFIError(_) => true
            case ConfigurationError(_) => true
            case _ => false
        }
    }
}
