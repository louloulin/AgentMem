//! AgentMem 完整功能演示
//! 
//! 这个示例展示了 AgentMem 的所有核心功能，包括：
//! - 记忆的创建、检索、更新和删除
//! - 智能搜索和语义检索
//! - 批量操作
//! - 性能监控
//! - 错误处理

use agent_mem_client::{
    AsyncAgentMemClient, ClientConfig,
    models::*,
    error::ClientResult,
};
use agent_mem_traits::{Memory, MemoryType, MemoryScope};
use chrono::{DateTime, Utc};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();
    
    println!("🧠 AgentMem 完整功能演示");
    println!("========================");
    
    // 创建客户端配置
    let config = ClientConfig::builder()
        .base_url("http://localhost:8080")
        .api_key("demo-api-key")
        .timeout(Duration::from_secs(30))
        .max_retries(3)
        .enable_logging(true)
        .build();
    
    // 创建客户端
    let client = AsyncAgentMemClient::new(config)?;
    
    // 检查服务健康状态
    println!("\n🔍 检查服务状态...");
    match client.health_check().await {
        Ok(health) => {
            println!("✅ 服务状态: {:?}", health.status);
            println!("   版本: {}", health.version);
            println!("   运行时间: {}s", health.uptime_seconds);
        }
        Err(e) => {
            println!("❌ 服务不可用: {}", e);
            return Ok(());
        }
    }
    
    // 演示基础记忆操作
    println!("\n📝 演示基础记忆操作");
    println!("==================");
    
    let demo_memories = create_demo_memories().await?;
    let mut memory_ids = Vec::new();
    
    // 创建记忆
    for (i, memory_request) in demo_memories.into_iter().enumerate() {
        println!("创建记忆 {}: {}", i + 1, memory_request.content);
        
        match client.add_memory(memory_request).await {
            Ok(response) => {
                println!("✅ 记忆已创建，ID: {}", response.memory.id);
                memory_ids.push(response.memory.id);
            }
            Err(e) => {
                println!("❌ 创建失败: {}", e);
            }
        }
    }
    
    // 检索记忆
    println!("\n🔍 检索记忆");
    println!("==========");
    
    if let Some(memory_id) = memory_ids.first() {
        match client.get_memory(memory_id).await {
            Ok(memory) => {
                println!("✅ 检索到记忆:");
                println!("   ID: {}", memory.id);
                println!("   内容: {}", memory.content);
                println!("   类型: {:?}", memory.memory_type);
                println!("   重要性: {:.2}", memory.importance);
                println!("   创建时间: {}", memory.created_at);
            }
            Err(e) => {
                println!("❌ 检索失败: {}", e);
            }
        }
    }
    
    // 更新记忆
    println!("\n✏️  更新记忆");
    println!("==========");
    
    if let Some(memory_id) = memory_ids.first() {
        let update_request = UpdateMemoryRequest {
            content: Some("更新后的记忆内容：今天深入学习了 Rust 的异步编程".to_string()),
            importance: Some(0.9),
            tags: Some(vec!["学习".to_string(), "Rust".to_string(), "异步编程".to_string()]),
            metadata: None,
        };
        
        match client.update_memory(memory_id, update_request).await {
            Ok(response) => {
                println!("✅ 记忆已更新:");
                println!("   新内容: {}", response.memory.content);
                println!("   新重要性: {:.2}", response.memory.importance);
            }
            Err(e) => {
                println!("❌ 更新失败: {}", e);
            }
        }
    }
    
    // 演示搜索功能
    println!("\n🔍 演示搜索功能");
    println!("==============");
    
    // 基础文本搜索
    let search_request = SearchMemoriesRequest {
        query: "Rust 编程".to_string(),
        search_type: Some(SearchType::Hybrid),
        limit: Some(5),
        offset: Some(0),
        filters: Some(SearchFilters {
            memory_types: Some(vec![MemoryType::Episodic, MemoryType::Semantic]),
            importance_range: Some((0.5, 1.0)),
            date_range: None,
            tags: Some(vec!["学习".to_string()]),
            exclude_tags: None,
        }),
        sort_by: Some(SortBy::Relevance),
        sort_order: Some(SortOrder::Desc),
    };
    
    match client.search_memories(search_request).await {
        Ok(response) => {
            println!("✅ 搜索结果 ({} 条):", response.results.len());
            for (i, result) in response.results.iter().enumerate() {
                println!("   {}. {} (相关性: {:.2})", 
                        i + 1, result.memory.content, result.relevance_score);
            }
            println!("   查询时间: {}ms", response.query_time_ms);
        }
        Err(e) => {
            println!("❌ 搜索失败: {}", e);
        }
    }
    
    // 语义搜索
    println!("\n🧠 语义搜索演示");
    println!("==============");
    
    let semantic_search = SearchMemoriesRequest {
        query: "如何提高编程技能".to_string(),
        search_type: Some(SearchType::Semantic),
        limit: Some(3),
        offset: Some(0),
        filters: None,
        sort_by: Some(SortBy::Relevance),
        sort_order: Some(SortOrder::Desc),
    };
    
    match client.search_memories(semantic_search).await {
        Ok(response) => {
            println!("✅ 语义搜索结果:");
            for (i, result) in response.results.iter().enumerate() {
                println!("   {}. {} (语义相似度: {:.2})", 
                        i + 1, result.memory.content, result.relevance_score);
            }
        }
        Err(e) => {
            println!("❌ 语义搜索失败: {}", e);
        }
    }
    
    // 演示批量操作
    println!("\n📦 演示批量操作");
    println!("==============");
    
    let batch_memories = vec![
        AddMemoryRequest {
            content: "批量创建的记忆 1".to_string(),
            memory_type: MemoryType::Episodic,
            scope: MemoryScope::User {
                agent_id: "demo_agent".to_string(),
                user_id: "demo_user".to_string(),
            },
            importance: 0.6,
            tags: vec!["批量".to_string(), "测试".to_string()],
            metadata: std::collections::HashMap::new(),
            context: None,
        },
        AddMemoryRequest {
            content: "批量创建的记忆 2".to_string(),
            memory_type: MemoryType::Semantic,
            scope: MemoryScope::User {
                agent_id: "demo_agent".to_string(),
                user_id: "demo_user".to_string(),
            },
            importance: 0.7,
            tags: vec!["批量".to_string(), "测试".to_string()],
            metadata: std::collections::HashMap::new(),
            context: None,
        },
    ];
    
    let batch_request = BatchAddMemoriesRequest {
        memories: batch_memories,
    };
    
    match client.batch_add_memories(batch_request).await {
        Ok(response) => {
            println!("✅ 批量创建成功:");
            println!("   成功: {} 条", response.successful);
            println!("   失败: {} 条", response.failed);
            for id in &response.memory_ids {
                memory_ids.push(id.clone());
            }
        }
        Err(e) => {
            println!("❌ 批量创建失败: {}", e);
        }
    }
    
    // 性能测试
    println!("\n⚡ 性能测试");
    println!("==========");
    
    let start_time = std::time::Instant::now();
    let mut successful_operations = 0;
    let total_operations = 10;
    
    for i in 0..total_operations {
        let search_request = SearchMemoriesRequest {
            query: format!("测试查询 {}", i),
            search_type: Some(SearchType::Fuzzy),
            limit: Some(5),
            offset: Some(0),
            filters: None,
            sort_by: Some(SortBy::Relevance),
            sort_order: Some(SortOrder::Desc),
        };
        
        if client.search_memories(search_request).await.is_ok() {
            successful_operations += 1;
        }
    }
    
    let elapsed = start_time.elapsed();
    let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();
    
    println!("✅ 性能测试结果:");
    println!("   总操作数: {}", total_operations);
    println!("   成功操作: {}", successful_operations);
    println!("   总耗时: {:.2}s", elapsed.as_secs_f64());
    println!("   平均 QPS: {:.2}", ops_per_second);
    
    // 获取系统指标
    println!("\n📊 系统指标");
    println!("==========");
    
    match client.get_metrics().await {
        Ok(metrics) => {
            println!("✅ 系统指标:");
            println!("   总记忆数: {}", metrics.total_memories);
            println!("   活跃连接: {}", metrics.active_connections);
            println!("   平均响应时间: {:.2}ms", metrics.avg_response_time_ms);
            println!("   内存使用: {:.1}MB", metrics.memory_usage_mb);
            println!("   CPU 使用率: {:.1}%", metrics.cpu_usage_percent);
        }
        Err(e) => {
            println!("❌ 获取指标失败: {}", e);
        }
    }
    
    // 清理演示数据
    println!("\n🧹 清理演示数据");
    println!("==============");
    
    let cleanup_ids: Vec<String> = memory_ids.into_iter().take(3).collect();
    
    match client.batch_delete_memories(cleanup_ids.clone()).await {
        Ok(response) => {
            println!("✅ 批量删除成功:");
            println!("   删除数量: {}", response.successful);
        }
        Err(e) => {
            println!("❌ 批量删除失败: {}", e);
        }
    }
    
    println!("\n🎉 演示完成！");
    println!("============");
    println!("AgentMem 提供了完整的记忆管理功能：");
    println!("• 智能记忆存储和检索");
    println!("• 多种搜索模式（精确、模糊、语义）");
    println!("• 高性能批量操作");
    println!("• 实时性能监控");
    println!("• 完善的错误处理");
    
    Ok(())
}

/// 创建演示用的记忆数据
async fn create_demo_memories() -> ClientResult<Vec<AddMemoryRequest>> {
    let memories = vec![
        AddMemoryRequest {
            content: "今天学习了 Rust 的所有权机制，理解了借用和生命周期的概念".to_string(),
            memory_type: MemoryType::Episodic,
            scope: MemoryScope::User {
                agent_id: "demo_agent".to_string(),
                user_id: "demo_user".to_string(),
            },
            importance: 0.8,
            tags: vec!["学习".to_string(), "Rust".to_string(), "编程".to_string()],
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source".to_string(), "学习笔记".to_string());
                map.insert("category".to_string(), "技术".to_string());
                map
            },
            context: Some(MemoryContext {
                location: Some("办公室".to_string()),
                time_of_day: Some("上午".to_string()),
                mood: Some("专注".to_string()),
                activity: Some("编程学习".to_string()),
                people_present: None,
                environment: Some("安静".to_string()),
            }),
        },
        AddMemoryRequest {
            content: "参加了团队会议，讨论了新项目的架构设计和技术选型".to_string(),
            memory_type: MemoryType::Episodic,
            scope: MemoryScope::User {
                agent_id: "demo_agent".to_string(),
                user_id: "demo_user".to_string(),
            },
            importance: 0.7,
            tags: vec!["工作".to_string(), "会议".to_string(), "架构".to_string()],
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source".to_string(), "工作会议".to_string());
                map.insert("category".to_string(), "工作".to_string());
                map
            },
            context: Some(MemoryContext {
                location: Some("会议室".to_string()),
                time_of_day: Some("下午".to_string()),
                mood: Some("积极".to_string()),
                activity: Some("团队协作".to_string()),
                people_present: Some(vec!["张三".to_string(), "李四".to_string()]),
                environment: Some("正式".to_string()),
            }),
        },
        AddMemoryRequest {
            content: "微服务架构的核心原则：单一职责、服务自治、去中心化治理".to_string(),
            memory_type: MemoryType::Semantic,
            scope: MemoryScope::Global,
            importance: 0.9,
            tags: vec!["架构".to_string(), "微服务".to_string(), "设计原则".to_string()],
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source".to_string(), "技术文档".to_string());
                map.insert("category".to_string(), "知识".to_string());
                map
            },
            context: None,
        },
        AddMemoryRequest {
            content: "解决 Rust 编译错误的步骤：1. 仔细阅读错误信息 2. 检查类型匹配 3. 验证生命周期".to_string(),
            memory_type: MemoryType::Procedural,
            scope: MemoryScope::User {
                agent_id: "demo_agent".to_string(),
                user_id: "demo_user".to_string(),
            },
            importance: 0.8,
            tags: vec!["Rust".to_string(), "调试".to_string(), "流程".to_string()],
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source".to_string(), "实践经验".to_string());
                map.insert("category".to_string(), "技能".to_string());
                map
            },
            context: None,
        },
    ];
    
    Ok(memories)
}

/// 记忆上下文信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryContext {
    pub location: Option<String>,
    pub time_of_day: Option<String>,
    pub mood: Option<String>,
    pub activity: Option<String>,
    pub people_present: Option<Vec<String>>,
    pub environment: Option<String>,
}
