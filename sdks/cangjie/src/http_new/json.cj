/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file json.cj
 * JSON工具 - 简单的JSON序列化/反序列化实现
 */

package agentmem_http

/// JSON构建器 - 用于序列化
public class JsonBuilder {
    private var fields: Array<(String, String)>
    
    /// 构造函数
    public init() {
        this.fields = Array<(String, String)>()
    }
    
    /// 添加字符串字段
    public func addString(key: String, value: String): JsonBuilder {
        let escapedValue = escapeJson(value)
        this.fields.append((key, "\"${escapedValue}\""))
        return this
    }
    
    /// 添加数字字段
    public func addNumber(key: String, value: Float64): JsonBuilder {
        this.fields.append((key, "${value}"))
        return this
    }
    
    /// 添加整数字段
    public func addInt(key: String, value: Int64): JsonBuilder {
        this.fields.append((key, "${value}"))
        return this
    }
    
    /// 添加布尔字段
    public func addBool(key: String, value: Bool): JsonBuilder {
        let boolStr = if (value) { "true" } else { "false" }
        this.fields.append((key, boolStr))
        return this
    }
    
    /// 添加可选字符串字段
    public func addOptionalString(key: String, value: Option<String>): JsonBuilder {
        match (value) {
            | Some(v) => this.addString(key, v)
            | None => this
        }
        return this
    }
    
    /// 构建JSON字符串
    public func build(): String {
        if (this.fields.size() == 0) {
            return "{}"
        }
        
        var parts = Array<String>(this.fields.size(), item: "")
        for (i in 0..this.fields.size()) {
            let (k, v) = this.fields[i]
            parts[i] = "\"${k}\":${v}"
        }
        
        var result = "{"
        for (i in 0..parts.size()) {
            result += parts[i]
            if (i < parts.size() - 1) {
                result += ","
            }
        }
        result += "}"
        
        return result
    }
}

/// JSON转义函数
public func escapeJson(s: String): String {
    var result = s
    result = result.replace("\\", "\\\\")
    result = result.replace("\"", "\\\"")
    result = result.replace("\n", "\\n")
    result = result.replace("\r", "\\r")
    result = result.replace("\t", "\\t")
    return result
}

/// 从JSON提取字段（简单实现）
public func extractField(json: String, fieldName: String): String {
    // 简化实现：查找 "fieldName":"value" 模式
    let pattern = "\"${fieldName}\":"
    let startIdx = json.indexOf(pattern)
    
    if (startIdx < 0) {
        return ""
    }
    
    // 跳过字段名和冒号
    var valueStartIdx = startIdx + pattern.size()
    
    // 跳过空格
    while (valueStartIdx < json.size() && json[valueStartIdx] == ' ') {
        valueStartIdx += 1
    }
    
    // 检查是否是字符串值（以"开头）
    if (valueStartIdx < json.size() && json[valueStartIdx] == '"') {
        valueStartIdx += 1  // 跳过开始的引号
        var valueEndIdx = valueStartIdx
        
        // 查找结束的引号
        while (valueEndIdx < json.size() && json[valueEndIdx] != '"') {
            if (json[valueEndIdx] == '\\') {
                valueEndIdx += 2  // 跳过转义字符
            } else {
                valueEndIdx += 1
            }
        }
        
        return json.substring(valueStartIdx, valueEndIdx)
    }
    
    // 数字或布尔值
    var valueEndIdx = valueStartIdx
    while (valueEndIdx < json.size() && 
           json[valueEndIdx] != ',' && 
           json[valueEndIdx] != '}' && 
           json[valueEndIdx] != ' ') {
        valueEndIdx += 1
    }
    
    return json.substring(valueStartIdx, valueEndIdx)
}

/// 从JSON提取整数字段
public func extractIntField(json: String, fieldName: String): Int64 {
    let valueStr = extractField(json, fieldName)
    if (valueStr.isEmpty()) {
        return 0
    }
    
    // 简单解析整数
    try {
        return Int64.parse(valueStr)
    } catch {
        return 0
    }
}

/// 从JSON提取浮点数字段
public func extractFloatField(json: String, fieldName: String): Float64 {
    let valueStr = extractField(json, fieldName)
    if (valueStr.isEmpty()) {
        return 0.0
    }
    
    // 简单解析浮点数
    try {
        return Float64.parse(valueStr)
    } catch {
        return 0.0
    }
}

/// 从JSON提取布尔字段
public func extractBoolField(json: String, fieldName: String): Bool {
    let valueStr = extractField(json, fieldName)
    return valueStr == "true"
}

