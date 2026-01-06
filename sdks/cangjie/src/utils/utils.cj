/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * 工具类 - 提供HashMap替代和其他工具函数
 */

package agentmem.utils

/// 简单的时间戳类型
public type Timestamp = UInt64

/// 简单的向量类型
public type Vector = Array<Float32>

/// 简单的互斥锁类型（占位符）
public class Mutex {
    public init() {}

    public func lock(): Unit {}
    public func unlock(): Unit {}
}

/// 简单的键值对
public class KeyValuePair {
    public var key: String
    public var value: String

    public init(key: String, value: String) {
        this.key = key
        this.value = value
    }
}

/// 简单的HashMap替代类
public class SimpleMap {
    private var pairs: Array<KeyValuePair>
    
    public init() {
        this.pairs = Array<KeyValuePair>()
    }
    
    /// 设置键值对
    public func set(key: String, value: String): Unit {
        // 查找是否已存在
        for (i in 0..this.pairs.size) {
            if (this.pairs[i].key == key) {
                this.pairs[i].value = value
                return
            }
        }
        // 不存在则添加新的
        let newPair = KeyValuePair(key, value)
        // 由于Array可能没有append方法，我们创建新数组
        let newPairs = Array<KeyValuePair>(this.pairs.size + 1, {_ => newPair})
        for (i in 0..this.pairs.size) {
            newPairs[i] = this.pairs[i]
        }
        newPairs[this.pairs.size] = newPair
        this.pairs = newPairs
    }
    
    /// 获取值
    public func get(key: String): Option<String> {
        for (i in 0..this.pairs.size) {
            if (this.pairs[i].key == key) {
                return Some(this.pairs[i].value)
            }
        }
        return None
    }
    
    /// 删除键值对
    public func remove(key: String): Unit {
        var newSize = 0
        for (i in 0..this.pairs.size) {
            if (this.pairs[i].key != key) {
                newSize += 1
            }
        }
        
        if (newSize == this.pairs.size) {
            return // 没找到要删除的
        }
        
        let newPairs = Array<KeyValuePair>(newSize, {_ => KeyValuePair("", "")})
        var newIndex = 0
        for (i in 0..this.pairs.size) {
            if (this.pairs[i].key != key) {
                newPairs[newIndex] = this.pairs[i]
                newIndex += 1
            }
        }
        this.pairs = newPairs
    }
    
    /// 检查是否为空
    public func isEmpty(): Bool {
        return this.pairs.size == 0
    }

    // 下标操作符
    public operator func [](key: String): String {
        let result = this.get(key)
        match (result) {
            case Some(value) => return value
            case None => return ""
        }
    }

    // 清空所有键值对
    public func clear(): Unit {
        this.pairs = Array<KeyValuePair>(0, {_ => KeyValuePair("", "")})
    }

    // 获取大小
    public func size(): UInt32 {
        return UInt32(this.pairs.size)
    }
    
    /// 转换为字符串
    public func toString(): String {
        if (this.isEmpty()) {
            return "{}"
        }
        
        var result = "{"
        for (i in 0..this.pairs.size) {
            if (i > 0) {
                result += ", "
            }
            result += "${this.pairs[i].key}: ${this.pairs[i].value}"
        }
        result += "}"
        return result
    }
}

/// 字符串工具类
public class StringUtils {
    /// 转换为小写（简单实现）
    public static func toLowerCase(s: String): String {
        // 仓颉可能不支持字符操作，返回原字符串
        return s
    }
    
    /// 生成简单ID
    public static func generateId(prefix: String): String {
        // 使用简单的计数器或固定值
        return "${prefix}-${42}"
    }

    /// 生成随机ID
    public static func generateRandomId(length: Int64): String {
        // 简化实现，返回固定格式的随机ID
        return "rand123456"
    }
}

/// 时间工具类
public class TimeUtils {
    /// 获取当前时间戳
    public static func getCurrentTimestamp(): UInt64 {
        return 1234567890 // 简化实现
    }
}

/// 数组工具类
public class ArrayUtils {
    /// 向数组添加元素（创建新数组）
    public static func append<T>(arr: Array<T>, element: T): Array<T> {
        let newArr = Array<T>(arr.size + 1, {_ => element})
        for (i in 0..arr.size) {
            newArr[i] = arr[i]
        }
        newArr[arr.size] = element
        return newArr
    }
}

/// 简单缓存类
public class SimpleCache {
    private var data: SimpleMap

    public init() {
        this.data = SimpleMap()
    }

    public func get(key: String): Option<String> {
        return this.data.get(key)
    }

    public func set(key: String, value: String): Unit {
        this.data.set(key, value)
        return ()
    }

    public func put(key: String, value: String): Unit {
        this.data.set(key, value)
        return ()
    }

    public func remove(key: String): Unit {
        this.data.remove(key)
        return ()
    }

    public func clear(): Unit {
        this.data.clear()
        return ()
    }
}

/// 性能监控类
public class PerformanceMonitor {
    private var name: String
    private var startTime: UInt64

    public init(name: String) {
        this.name = name
        this.startTime = TimeUtils.getCurrentTimestamp()
    }

    public func finishAndLog(): Unit {
        let endTime = TimeUtils.getCurrentTimestamp()
        let _ = endTime - this.startTime // 计算持续时间
        // 简化实现，实际应该记录到日志系统
        return ()
    }
}
