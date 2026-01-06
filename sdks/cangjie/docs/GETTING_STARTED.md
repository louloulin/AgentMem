# AgentMem 仓颉 SDK 快速开始指南

## 概述

AgentMem 仓颉 SDK 是一个企业级智能记忆管理平台的仓颉语言接口，提供了完整的记忆存储、检索、分析和管理功能。本指南将帮助您快速上手使用 SDK。

## 系统要求

- **仓颉编程语言**: 0.60.5 或更高版本
- **操作系统**: Linux, macOS, Windows
- **内存**: 最少 512MB RAM
- **网络**: 需要访问 AgentMem 服务器

## 安装

### 1. 添加依赖

在您的 `cjpm.toml` 文件中添加 AgentMem SDK 依赖：

```toml
[dependencies]
agentmem-cangjie-sdk = { path = "path/to/agentmem/sdks/cangjie" }
```

### 2. 导入包

在您的仓颉代码中导入所需的包：

```cangjie
import agentmem.api.*
import agentmem.core.*
```

## 快速开始

### 第一步：创建客户端

```cangjie
// 创建客户端配置
let config = ClientConfig("http://localhost:8080")
    .withApiKey("your-api-key")
    .withTimeout(30)
    .withRetryCount(3)
    .withCache(true, 100)
    .withDebugMode(true)

// 创建客户端
let client = AgentMemClient(config)

// 初始化客户端
let initResult = client.initialize()
match (initResult) {
    case Ok(_) => println("✅ 客户端初始化成功")
    case Err(error) => {
        println("❌ 客户端初始化失败: ${error.getMessage()}")
        return
    }
}
```

### 第二步：添加记忆

```cangjie
// 使用 MemoryBuilder 创建记忆
let memory = MemoryBuilder()
    .withAgentId("my-agent")
    .withUserId("user-123")
    .withContent("我喜欢喝咖啡，特别是在早晨工作的时候")
    .withMemoryType(MemoryType.Semantic)
    .withImportance(0.8)
    .withMetadata("category", "preference")
    .withMetadata("source", "conversation")
    .build()

// 添加记忆到系统
let addResult = client.addMemory(memory)
match (addResult) {
    case Ok(memoryId) => {
        println("✅ 记忆添加成功，ID: ${memoryId}")
    }
    case Err(error) => {
        println("❌ 记忆添加失败: ${error.getMessage()}")
    }
}
```

### 第三步：搜索记忆

```cangjie
// 基础搜索
let searchResult = client.searchMemories("咖啡", 5)
match (searchResult) {
    case Ok(results) => {
        println("🔍 找到 ${results.size} 条相关记忆:")
        for ((index, result) in results.enumerate()) {
            println("${index + 1}. [${result.score:.2f}] ${result.memory.content}")
        }
    }
    case Err(error) => {
        println("❌ 搜索失败: ${error.getMessage()}")
    }
}
```

### 第四步：获取和更新记忆

```cangjie
// 获取特定记忆
let getResult = client.getMemory(memoryId)
match (getResult) {
    case Ok(memoryOpt) => {
        if (memoryOpt.isSome()) {
            let memory = memoryOpt.getOrThrow()
            println("📖 记忆内容: ${memory.content}")
            println("📊 重要性: ${memory.importance}")
            println("🏷️ 类型: ${memory.memoryType.toString()}")
        } else {
            println("⚠️ 记忆不存在")
        }
    }
    case Err(error) => {
        println("❌ 获取记忆失败: ${error.getMessage()}")
    }
}

// 更新记忆内容
let updateResult = client.updateMemory(memoryId, "我非常喜欢喝咖啡，它能让我保持专注")
match (updateResult) {
    case Ok(_) => println("✅ 记忆更新成功")
    case Err(error) => println("❌ 记忆更新失败: ${error.getMessage()}")
}
```

### 第五步：清理资源

```cangjie
// 关闭客户端连接
client.close()
println("✅ 客户端已关闭")
```

## 完整示例

```cangjie
package my.app

import agentmem.api.*
import agentmem.core.*

main() {
    try {
        // 1. 创建和初始化客户端
        let config = ClientConfig("http://localhost:8080")
            .withApiKey("demo-api-key")
            .withCache(true, 100)
            .withDebugMode(true)
        
        let client = AgentMemClient(config)
        let initResult = client.initialize()
        match (initResult) {
            case Ok(_) => println("✅ 客户端初始化成功")
            case Err(error) => {
                println("❌ 初始化失败: ${error.getMessage()}")
                return
            }
        }
        
        // 2. 添加记忆
        let memory = MemoryBuilder()
            .withAgentId("demo-agent")
            .withContent("学习仓颉编程语言的FFI功能")
            .withMemoryType(MemoryType.Procedural)
            .withImportance(0.9)
            .withMetadata("topic", "programming")
            .build()
        
        var memoryId = ""
        let addResult = client.addMemory(memory)
        match (addResult) {
            case Ok(id) => {
                memoryId = id
                println("✅ 记忆添加成功: ${id}")
            }
            case Err(error) => {
                println("❌ 添加失败: ${error.getMessage()}")
                return
            }
        }
        
        // 3. 搜索记忆
        let searchResult = client.searchMemories("仓颉", 3)
        match (searchResult) {
            case Ok(results) => {
                println("🔍 搜索结果 (${results.size} 条):")
                for ((index, result) in results.enumerate()) {
                    println("${index + 1}. [${result.score:.2f}] ${result.memory.content}")
                }
            }
            case Err(error) => {
                println("❌ 搜索失败: ${error.getMessage()}")
            }
        }
        
        // 4. 获取统计信息
        let statsResult = client.getMemoryStats("demo-agent")
        match (statsResult) {
            case Ok(stats) => {
                println("📊 统计信息:")
                println("   总记忆数: ${stats.totalMemories}")
                println("   平均重要性: ${stats.averageImportance:.2f}")
            }
            case Err(error) => {
                println("❌ 获取统计失败: ${error.getMessage()}")
            }
        }
        
        // 5. 清理演示数据
        let deleteResult = client.deleteMemory(memoryId)
        match (deleteResult) {
            case Ok(_) => println("✅ 演示记忆已删除")
            case Err(error) => println("❌ 删除失败: ${error.getMessage()}")
        }
        
        // 6. 关闭客户端
        client.close()
        println("✅ 演示完成")
        
    } catch (e: Exception) {
        println("💥 程序异常: ${e}")
    }
}
```

## 高级功能

### 高级搜索

```cangjie
// 创建搜索管理器
let searchManager = SearchManager(client)

// 智能搜索
let smartResult = searchManager.smartSearch("人工智能", None, Some(5))

// 过滤搜索
var filter = SearchFilter()
filter.memoryTypes = Some([MemoryType.Semantic])
filter.importanceRange = Some((0.8, 1.0))
filter.agentIds = Some(["my-agent"])

let filteredResult = searchManager.smartSearch("*", Some(filter), Some(10))
```

### 批量操作

```cangjie
// 批量添加记忆
let memories = [
    MemoryBuilder().withAgentId("agent1").withContent("内容1").build(),
    MemoryBuilder().withAgentId("agent1").withContent("内容2").build(),
    MemoryBuilder().withAgentId("agent1").withContent("内容3").build()
]

let batchResult = client.addMemoriesBatch(memories)
match (batchResult) {
    case Ok(result) => {
        println("✅ 批量添加完成:")
        println("   成功: ${result.successes.size}")
        println("   失败: ${result.failures.size}")
    }
    case Err(error) => {
        println("❌ 批量添加失败: ${error.getMessage()}")
    }
}
```

### 分页查询

```cangjie
// 分页获取记忆
let pagination = PaginationParams(1, 20)  // 第1页，每页20条
let pageResult = client.getMemoriesPaginated("my-agent", pagination)

match (pageResult) {
    case Ok(result) => {
        println("📄 第 ${result.page} 页 (共 ${result.totalCount} 条):")
        for ((index, memory) in result.data.enumerate()) {
            println("${index + 1}. ${memory.content}")
        }
    }
    case Err(error) => {
        println("❌ 分页查询失败: ${error.getMessage()}")
    }
}
```

### 管理功能

```cangjie
// 创建管理器
let adminManager = AdminManager(client)

// 系统健康检查
let healthResult = adminManager.systemHealthCheck()
match (healthResult) {
    case Ok(health) => {
        println("🏥 系统健康状态:")
        println("   连接状态: ${health.isConnected}")
        println("   版本: ${health.version}")
    }
    case Err(error) => {
        println("❌ 健康检查失败: ${error.getMessage()}")
    }
}

// 数据备份
let backupResult = adminManager.backupData(
    Some("my-agent"), 
    "/tmp/backup.json", 
    BackupFormat.Json
)
```

## 配置选项

### 客户端配置

```cangjie
let config = ClientConfig("http://localhost:8080")
    .withApiKey("your-api-key")          // API密钥
    .withTimeout(30)                     // 超时时间(秒)
    .withRetryCount(3)                   // 重试次数
    .withCache(true, 100)                // 启用缓存，缓存大小100
    .withDebugMode(true)                 // 启用调试模式
    .withLogLevel(LogLevel.Info)         // 日志级别
```

### 搜索配置

```cangjie
var searchConfig = SearchConfig()
searchConfig.maxResults = 20                    // 最大结果数
searchConfig.similarityThreshold = 0.7         // 相似度阈值
searchConfig.enableSemanticSearch = true       // 启用语义搜索
searchConfig.enableFullTextSearch = true       // 启用全文搜索

let searchManager = SearchManager(client, searchConfig)
```

## 错误处理

### 常见错误类型

```cangjie
match (result) {
    case Ok(value) => {
        // 处理成功结果
    }
    case Err(error) => {
        match (error) {
            case AgentMemError.InvalidInput(msg) => {
                println("输入错误: ${msg}")
            }
            case AgentMemError.NetworkError(msg) => {
                println("网络错误: ${msg}")
            }
            case AgentMemError.AuthenticationError(msg) => {
                println("认证错误: ${msg}")
            }
            case AgentMemError.NotFound(msg) => {
                println("未找到: ${msg}")
            }
            case AgentMemError.TimeoutError(msg) => {
                println("超时错误: ${msg}")
            }
            case AgentMemError.InternalError(msg) => {
                println("内部错误: ${msg}")
            }
        }
    }
}
```

## 性能优化建议

1. **启用缓存**: 对于频繁访问的数据启用客户端缓存
2. **批量操作**: 使用批量API减少网络往返次数
3. **分页查询**: 对于大量数据使用分页避免内存问题
4. **合理超时**: 根据网络环境调整超时时间
5. **连接复用**: 重用客户端连接避免频繁初始化

## 最佳实践

1. **记忆内容**: 保持记忆内容简洁明确，避免过长文本
2. **重要性设置**: 合理设置重要性帮助搜索排序和过滤
3. **元数据使用**: 充分利用元数据进行分类和标记
4. **错误处理**: 始终处理可能的错误情况
5. **资源清理**: 使用完毕后及时关闭客户端连接

## 故障排除

### 常见问题

**Q: 客户端初始化失败**
A: 检查服务器地址、API密钥和网络连接

**Q: 搜索结果为空**
A: 确认记忆已正确添加，检查搜索关键词和过滤条件

**Q: 操作超时**
A: 增加超时时间或检查网络连接稳定性

**Q: 内存使用过高**
A: 使用分页查询，避免一次性加载大量数据

### 调试技巧

1. 启用调试模式查看详细日志
2. 使用健康检查验证连接状态
3. 检查返回的错误信息
4. 监控网络请求和响应

## 下一步

- 查看 [API 参考文档](API_REFERENCE.md) 了解完整API
- 运行 [示例程序](../src/examples/) 学习具体用法
- 阅读 [架构文档](ARCHITECTURE.md) 了解内部实现
- 参考 [性能测试](../src/tests/performance_tests.cj) 进行性能调优

## 获取帮助

如果您在使用过程中遇到问题，可以：

1. 查看本文档和API参考
2. 运行示例程序学习用法
3. 查看测试用例了解预期行为
4. 提交Issue报告问题

祝您使用愉快！🎉
