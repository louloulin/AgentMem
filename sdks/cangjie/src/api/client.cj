/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * AgentMem客户端API - 高级接口实现
 */

package agentmem.api

import agentmem.core.{Memory, MemoryType, ImportanceLevel, AgentMemError, AgentMemResult, ClientConfig}
import agentmem.utils.{SimpleMap, StringUtils, TimeUtils}

/// AgentMem客户端 - 主要API接口
public class AgentMemClient {
    private var _config: ClientConfig
    private var _connected: Bool = false
    private var _agentId: String = ""
    private var _memoryCache: SimpleMap = SimpleMap()

    /// 构造函数
    public init(config: ClientConfig) {
        this._config = config
        this._agentId = config.agentId
    }

    /// 连接到AgentMem服务
    public func connect(): AgentMemResult<Bool> {
        try {
            // 模拟连接逻辑
            if (this._config.serverUrl.isEmpty()) {
                return AgentMemResult<Bool>.Err(
                    AgentMemError.InvalidParameter("Server URL cannot be empty")
                )
            }

            // 简单的URL验证
            if (!this._config.serverUrl.startsWith("http://") && 
                !this._config.serverUrl.startsWith("https://")) {
                return AgentMemResult<Bool>.Err(
                    AgentMemError.InvalidParameter("Invalid server URL format")
                )
            }

            this._connected = true
            return AgentMemResult<Bool>.Ok(true)

        } catch (e: Exception) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Connection failed: ${e}")
            )
        }
    }

    /// 断开连接
    public func disconnect(): Unit {
        this._connected = false
        this._memoryCache.clear()
    }

    /// 检查连接状态
    public func isConnected(): Bool {
        return this._connected
    }

    /// 设置连接状态（用于测试）
    public func setConnected(connected: Bool): Unit {
        this._connected = connected
    }

    /// 添加记忆
    public func addMemory(memory: Memory): AgentMemResult<String> {
        if (!this._connected) {
            return AgentMemResult<String>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        try {
            // 验证记忆
            let validationResult = memory.validate()
            if (validationResult.isErr()) {
                return AgentMemResult<String>.Err(validationResult.getErr())
            }

            // 生成记忆ID（模拟）
            let memoryId = "mem_${TimeUtils.getCurrentTimestamp()}_${StringUtils.generateRandomId(8)}"
            
            // 缓存记忆
            this._memoryCache.set(memoryId, memory.content)

            return AgentMemResult<String>.Ok(memoryId)

        } catch (e: Exception) {
            return AgentMemResult<String>.Err(
                AgentMemError.InternalError("Failed to add memory: ${e}")
            )
        }
    }

    /// 获取记忆
    public func getMemory(memoryId: String): AgentMemResult<Memory> {
        if (!this._connected) {
            return AgentMemResult<Memory>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        if (memoryId.isEmpty()) {
            return AgentMemResult<Memory>.Err(
                AgentMemError.InvalidParameter("Memory ID cannot be empty")
            )
        }

        try {
            // 从缓存获取（模拟）
            let contentOpt = this._memoryCache.get(memoryId)
            if (contentOpt.isNone()) {
                return AgentMemResult<Memory>.Err(
                    AgentMemError.NotFound("Memory not found: ${memoryId}")
                )
            }

            let content = contentOpt.getOrThrow()
            
            // 创建记忆对象
            let memory = Memory(
                memoryId,
                this._agentId,
                Some("user-123"),
                MemoryType.Semantic,
                content,
                ImportanceLevel.Medium.toFloat32(),
                None,
                TimeUtils.getCurrentTimestamp(),
                TimeUtils.getCurrentTimestamp(),
                1,
                None,
                SimpleMap(),
                1
            )

            return AgentMemResult<Memory>.Ok(memory)

        } catch (e: Exception) {
            return AgentMemResult<Memory>.Err(
                AgentMemError.InternalError("Failed to get memory: ${e}")
            )
        }
    }

    /// 更新记忆
    public func updateMemory(memoryId: String, content: String): AgentMemResult<Bool> {
        if (!this._connected) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        if (memoryId.isEmpty()) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InvalidParameter("Memory ID cannot be empty")
            )
        }

        if (content.isEmpty()) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InvalidParameter("Content cannot be empty")
            )
        }

        try {
            // 检查记忆是否存在
            let existingOpt = this._memoryCache.get(memoryId)
            if (existingOpt.isNone()) {
                return AgentMemResult<Bool>.Err(
                    AgentMemError.NotFound("Memory not found: ${memoryId}")
                )
            }

            // 更新缓存
            this._memoryCache.set(memoryId, content)

            return AgentMemResult<Bool>.Ok(true)

        } catch (e: Exception) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Failed to update memory: ${e}")
            )
        }
    }

    /// 删除记忆
    public func deleteMemory(memoryId: String): AgentMemResult<Bool> {
        if (!this._connected) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        if (memoryId.isEmpty()) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InvalidParameter("Memory ID cannot be empty")
            )
        }

        try {
            // 检查记忆是否存在
            let existingOpt = this._memoryCache.get(memoryId)
            if (existingOpt.isNone()) {
                return AgentMemResult<Bool>.Err(
                    AgentMemError.NotFound("Memory not found: ${memoryId}")
                )
            }

            // 从缓存删除
            this._memoryCache.remove(memoryId)

            return AgentMemResult<Bool>.Ok(true)

        } catch (e: Exception) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Failed to delete memory: ${e}")
            )
        }
    }

    /// 搜索记忆（简化版本）
    public func searchMemories(query: String, _limit: UInt32): AgentMemResult<Array<Memory>> {
        if (!this._connected) {
            return AgentMemResult<Array<Memory>>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        if (query.isEmpty()) {
            return AgentMemResult<Array<Memory>>.Err(
                AgentMemError.InvalidParameter("Query cannot be empty")
            )
        }

        try {
            // 简单的搜索实现（模拟）
            let results = Array<Memory>(0, {i => 
                Memory(
                    "search-result-${i}",
                    this._agentId,
                    Some("user-123"),
                    MemoryType.Semantic,
                    "Search result for: ${query}",
                    ImportanceLevel.Medium.toFloat32(),
                    None,
                    TimeUtils.getCurrentTimestamp(),
                    TimeUtils.getCurrentTimestamp(),
                    1,
                    None,
                    SimpleMap(),
                    1
                )
            })

            return AgentMemResult<Array<Memory>>.Ok(results)

        } catch (e: Exception) {
            return AgentMemResult<Array<Memory>>.Err(
                AgentMemError.InternalError("Failed to search memories: ${e}")
            )
        }
    }

    /// 获取客户端配置
    public func getConfig(): ClientConfig {
        return this._config
    }

    /// 获取代理ID
    public func getAgentId(): String {
        return this._agentId
    }

    /// 获取缓存的记忆数量
    public func getCachedMemoryCount(): UInt32 {
        return this._memoryCache.size()
    }

    /// 健康检查
    public func healthCheck(): AgentMemResult<Bool> {
        if (!this._connected) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.NotConnected("Client not connected")
            )
        }

        try {
            // 模拟健康检查
            return AgentMemResult<Bool>.Ok(true)

        } catch (e: Exception) {
            return AgentMemResult<Bool>.Err(
                AgentMemError.InternalError("Health check failed: ${e}")
            )
        }
    }
}

/// 客户端构建器
public class AgentMemClientBuilder {
    private var _serverUrl: String = ""
    private var _agentId: String = ""
    private var _apiKey: String = ""
    private var _timeout: UInt32 = 30000
    private var _retryCount: UInt32 = 3

    public init() {}

    /// 设置服务器URL
    public func serverUrl(url: String): AgentMemClientBuilder {
        this._serverUrl = url
        return this
    }

    /// 设置代理ID
    public func agentId(id: String): AgentMemClientBuilder {
        this._agentId = id
        return this
    }

    /// 设置代理ID（别名方法）
    public func setAgentId(id: String): AgentMemClientBuilder {
        this._agentId = id
        return this
    }

    /// 设置API密钥
    public func apiKey(key: String): AgentMemClientBuilder {
        this._apiKey = key
        return this
    }

    /// 设置API密钥（别名方法）
    public func setApiKey(key: String): AgentMemClientBuilder {
        this._apiKey = key
        return this
    }

    /// 设置超时时间
    public func timeout(timeoutMs: UInt32): AgentMemClientBuilder {
        this._timeout = timeoutMs
        return this
    }

    /// 设置重试次数
    public func retryCount(count: UInt32): AgentMemClientBuilder {
        this._retryCount = count
        return this
    }

    /// 构建客户端
    public func build(): AgentMemResult<AgentMemClient> {
        if (this._serverUrl.isEmpty()) {
            return AgentMemResult<AgentMemClient>.Err(
                AgentMemError.InvalidParameter("Server URL is required")
            )
        }

        if (this._agentId.isEmpty()) {
            return AgentMemResult<AgentMemClient>.Err(
                AgentMemError.InvalidParameter("Agent ID is required")
            )
        }

        let config = ClientConfig(this._serverUrl, this._agentId)

        let client = AgentMemClient(config)
        return AgentMemResult<AgentMemClient>.Ok(client)
    }
}
