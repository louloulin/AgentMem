/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file types.cj
 * HTTP类型定义 - AgentMem 仓颉 SDK HTTP方案
 */

package agentmem_http

/// HTTP错误类型
public enum AgentMemError {
    /// 网络错误
    | NetworkError(String)
    /// HTTP状态码错误
    | HttpError(Int32, String)
    /// JSON解析错误
    | JsonParseError(String)
    /// 数据验证错误
    | ValidationError(String)
    /// 404错误
    | NotFoundError(String)
    /// 401/403错误
    | UnauthorizedError(String)
    /// 500服务器错误
    | ServerError(String)
    
    /// 获取错误消息
    public func getMessage(): String {
        match (this) {
            | NetworkError(msg) => "网络错误: ${msg}"
            | HttpError(code, msg) => "HTTP错误 ${code}: ${msg}"
            | JsonParseError(msg) => "JSON解析错误: ${msg}"
            | ValidationError(msg) => "验证错误: ${msg}"
            | NotFoundError(msg) => "未找到: ${msg}"
            | UnauthorizedError(msg) => "未授权: ${msg}"
            | ServerError(msg) => "服务器错误: ${msg}"
        }
    }
}

/// HTTP响应结果类型
public enum Result<T, E> {
    | Ok(T)
    | Err(E)
    
    /// 是否成功
    public func isOk(): Bool {
        match (this) {
            | Ok(_) => true
            | Err(_) => false
        }
    }
    
    /// 是否失败
    public func isErr(): Bool {
        !this.isOk()
    }
    
    /// 解包，失败时抛出异常
    public func getOrThrow(): T {
        match (this) {
            | Ok(value) => value
            | Err(e) => {
                // 简化实现：返回默认值或抛出错误
                // 注：实际应使用异常机制
                throw Exception("Result unwrap failed")
            }
        }
    }
}

/// HTTP状态码映射
public func mapHttpError(code: Int32, message: String): AgentMemError {
    if (code >= 400 && code < 500) {
        if (code == 401 || code == 403) {
            return UnauthorizedError(message)
        } else if (code == 404) {
            return NotFoundError(message)
        } else {
            return ValidationError(message)
        }
    } else if (code >= 500 && code < 600) {
        return ServerError(message)
    } else {
        return HttpError(code, message)
    }
}

