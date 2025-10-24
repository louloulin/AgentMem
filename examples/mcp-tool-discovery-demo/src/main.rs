//! MCP 工具发现和注册功能演示
//!
//! 演示动态工具发现、加载和注册

use agent_mem_tools::mcp::{ToolDiscovery, ToolMetadata, ToolType, HttpToolLoader, ToolLoader};
use std::collections::HashMap;
use tracing::error;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n🚀 MCP 工具发现和注册功能演示");
    println!("============================================================\n");

    // 演示 1: 工具元数据注册
    demo_tool_metadata_registration().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 2: 工具搜索和过滤
    demo_tool_search_and_filter().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 3: 工具加载器
    demo_tool_loader().await;

    println!("\n------------------------------------------------------------\n");

    // 演示 4: 工具依赖管理
    demo_tool_dependencies().await;

    println!("\n============================================================");
    println!("✅ 所有演示完成！");
}

/// 演示工具元数据注册
async fn demo_tool_metadata_registration() {
    println!("📋 演示 1: 工具元数据注册");
    println!("------------------------------------------------------------");

    let discovery = ToolDiscovery::new();

    // 1. 注册 HTTP 工具
    println!("\n1️⃣ 注册 HTTP 工具:");
    let http_tool = ToolMetadata {
        name: "weather_api".to_string(),
        version: "1.0.0".to_string(),
        description: "Get weather information from API".to_string(),
        source: "http://api.weather.com".to_string(),
        tool_type: ToolType::Http,
        dependencies: vec![],
        tags: vec!["weather".to_string(), "api".to_string()],
        metadata: HashMap::new(),
    };

    match discovery.register_metadata(http_tool.clone()).await {
        Ok(_) => {
            println!("  ✅ HTTP 工具注册成功");
            println!("  名称: {}", http_tool.name);
            println!("  版本: {}", http_tool.version);
            println!("  类型: {:?}", http_tool.tool_type);
            println!("  来源: {}", http_tool.source);
        }
        Err(e) => {
            error!("  ❌ 注册失败: {}", e);
        }
    }

    // 2. 注册 Stdio 工具
    println!("\n2️⃣ 注册 Stdio 工具:");
    let stdio_tool = ToolMetadata {
        name: "file_processor".to_string(),
        version: "2.0.0".to_string(),
        description: "Process files using stdio interface".to_string(),
        source: "/usr/local/bin/file-processor".to_string(),
        tool_type: ToolType::Stdio,
        dependencies: vec![],
        tags: vec!["file".to_string(), "processing".to_string()],
        metadata: HashMap::new(),
    };

    match discovery.register_metadata(stdio_tool.clone()).await {
        Ok(_) => {
            println!("  ✅ Stdio 工具注册成功");
            println!("  名称: {}", stdio_tool.name);
            println!("  版本: {}", stdio_tool.version);
            println!("  类型: {:?}", stdio_tool.tool_type);
        }
        Err(e) => {
            error!("  ❌ 注册失败: {}", e);
        }
    }

    // 3. 注册本地工具
    println!("\n3️⃣ 注册本地工具:");
    let local_tool = ToolMetadata {
        name: "data_analyzer".to_string(),
        version: "1.5.0".to_string(),
        description: "Analyze data locally".to_string(),
        source: "local://data-analyzer".to_string(),
        tool_type: ToolType::Local,
        dependencies: vec![],
        tags: vec!["data".to_string(), "analysis".to_string()],
        metadata: HashMap::new(),
    };

    match discovery.register_metadata(local_tool.clone()).await {
        Ok(_) => {
            println!("  ✅ 本地工具注册成功");
            println!("  名称: {}", local_tool.name);
            println!("  版本: {}", local_tool.version);
        }
        Err(e) => {
            error!("  ❌ 注册失败: {}", e);
        }
    }

    // 4. 列出所有已注册的工具
    println!("\n4️⃣ 列出所有已注册的工具:");
    let all_metadata = discovery.list_metadata().await;
    println!("  找到 {} 个工具:", all_metadata.len());
    for metadata in &all_metadata {
        println!("    - {} (v{}) - {:?}", metadata.name, metadata.version, metadata.tool_type);
    }
}

/// 演示工具搜索和过滤
async fn demo_tool_search_and_filter() {
    println!("📋 演示 2: 工具搜索和过滤");
    println!("------------------------------------------------------------");

    let discovery = ToolDiscovery::new();

    // 注册一些测试工具
    let tools = vec![
        ToolMetadata {
            name: "weather_api".to_string(),
            version: "1.0.0".to_string(),
            description: "Get weather information".to_string(),
            source: "http://api.weather.com".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["weather".to_string(), "api".to_string()],
            metadata: HashMap::new(),
        },
        ToolMetadata {
            name: "news_api".to_string(),
            version: "1.0.0".to_string(),
            description: "Get news articles".to_string(),
            source: "http://api.news.com".to_string(),
            tool_type: ToolType::Http,
            dependencies: vec![],
            tags: vec!["news".to_string(), "api".to_string()],
            metadata: HashMap::new(),
        },
        ToolMetadata {
            name: "file_processor".to_string(),
            version: "2.0.0".to_string(),
            description: "Process files".to_string(),
            source: "/usr/local/bin/file-processor".to_string(),
            tool_type: ToolType::Stdio,
            dependencies: vec![],
            tags: vec!["file".to_string(), "processing".to_string()],
            metadata: HashMap::new(),
        },
    ];

    for tool in tools {
        discovery.register_metadata(tool).await.unwrap();
    }

    // 1. 搜索工具
    println!("\n1️⃣ 搜索包含 'api' 的工具:");
    let results = discovery.search_tools("api").await;
    println!("  找到 {} 个工具:", results.len());
    for tool in &results {
        println!("    - {}: {}", tool.name, tool.description);
    }

    // 2. 按标签过滤
    println!("\n2️⃣ 按标签 'api' 过滤:");
    let results = discovery.filter_by_tags(&["api".to_string()]).await;
    println!("  找到 {} 个工具:", results.len());
    for tool in &results {
        println!("    - {}: {:?}", tool.name, tool.tags);
    }

    // 3. 按标签 'file' 过滤
    println!("\n3️⃣ 按标签 'file' 过滤:");
    let results = discovery.filter_by_tags(&["file".to_string()]).await;
    println!("  找到 {} 个工具:", results.len());
    for tool in &results {
        println!("    - {}: {:?}", tool.name, tool.tags);
    }

    // 4. 获取特定工具的元数据
    println!("\n4️⃣ 获取特定工具的元数据:");
    if let Some(metadata) = discovery.get_metadata("weather_api").await {
        println!("  ✅ 找到工具: {}", metadata.name);
        println!("  版本: {}", metadata.version);
        println!("  描述: {}", metadata.description);
        println!("  来源: {}", metadata.source);
        println!("  标签: {:?}", metadata.tags);
    } else {
        println!("  ❌ 工具未找到");
    }
}

/// 演示工具加载器
async fn demo_tool_loader() {
    println!("📋 演示 3: 工具加载器");
    println!("------------------------------------------------------------");

    let loader = HttpToolLoader::new();

    // 1. 加载 HTTP 工具
    println!("\n1️⃣ 加载 HTTP 工具:");
    let http_tool = ToolMetadata {
        name: "api_client".to_string(),
        version: "1.0.0".to_string(),
        description: "HTTP API client".to_string(),
        source: "http://api.example.com".to_string(),
        tool_type: ToolType::Http,
        dependencies: vec![],
        tags: vec![],
        metadata: HashMap::new(),
    };

    match loader.load(&http_tool).await {
        Ok(loaded) => {
            if loaded {
                println!("  ✅ HTTP 工具加载成功");
                println!("  工具名称: {}", http_tool.name);
                println!("  来源: {}", http_tool.source);
            } else {
                println!("  ⚠️  工具类型不匹配");
            }
        }
        Err(e) => {
            error!("  ❌ 加载失败: {}", e);
        }
    }

    // 2. 检查工具是否已加载
    println!("\n2️⃣ 检查工具是否已加载:");
    let is_loaded = loader.is_loaded("api_client").await;
    println!("  api_client 已加载: {is_loaded}");

    // 3. 尝试加载非 HTTP 工具
    println!("\n3️⃣ 尝试加载非 HTTP 工具:");
    let stdio_tool = ToolMetadata {
        name: "stdio_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Stdio tool".to_string(),
        source: "/usr/local/bin/tool".to_string(),
        tool_type: ToolType::Stdio,
        dependencies: vec![],
        tags: vec![],
        metadata: HashMap::new(),
    };

    match loader.load(&stdio_tool).await {
        Ok(loaded) => {
            if loaded {
                println!("  ✅ 工具加载成功");
            } else {
                println!("  ⚠️  工具类型不匹配（预期行为）");
            }
        }
        Err(e) => {
            error!("  ❌ 加载失败: {}", e);
        }
    }

    // 4. 卸载工具
    println!("\n4️⃣ 卸载工具:");
    match loader.unload("api_client").await {
        Ok(_) => {
            println!("  ✅ 工具卸载成功");
            let is_loaded = loader.is_loaded("api_client").await;
            println!("  api_client 已加载: {is_loaded}");
        }
        Err(e) => {
            error!("  ❌ 卸载失败: {}", e);
        }
    }
}

/// 演示工具依赖管理
async fn demo_tool_dependencies() {
    println!("📋 演示 4: 工具依赖管理");
    println!("------------------------------------------------------------");

    let discovery = ToolDiscovery::new();

    // 1. 注册基础工具（依赖）
    println!("\n1️⃣ 注册基础工具:");
    let base_tool = ToolMetadata {
        name: "http_client".to_string(),
        version: "1.0.0".to_string(),
        description: "HTTP client library".to_string(),
        source: "http://lib.example.com/http-client".to_string(),
        tool_type: ToolType::Local,
        dependencies: vec![],
        tags: vec!["library".to_string()],
        metadata: HashMap::new(),
    };

    discovery.register_metadata(base_tool.clone()).await.unwrap();
    println!("  ✅ 基础工具注册成功: {}", base_tool.name);

    // 2. 注册依赖于基础工具的工具
    println!("\n2️⃣ 注册依赖工具:");
    let dependent_tool = ToolMetadata {
        name: "rest_api_client".to_string(),
        version: "2.0.0".to_string(),
        description: "REST API client".to_string(),
        source: "http://api.example.com/rest-client".to_string(),
        tool_type: ToolType::Http,
        dependencies: vec!["http_client".to_string()],
        tags: vec!["api".to_string()],
        metadata: HashMap::new(),
    };

    match discovery.register_metadata(dependent_tool.clone()).await {
        Ok(_) => {
            println!("  ✅ 依赖工具注册成功: {}", dependent_tool.name);
            println!("  依赖: {:?}", dependent_tool.dependencies);
        }
        Err(e) => {
            error!("  ❌ 注册失败: {}", e);
        }
    }

    // 3. 尝试注册缺少依赖的工具
    println!("\n3️⃣ 尝试注册缺少依赖的工具:");
    let missing_dep_tool = ToolMetadata {
        name: "advanced_tool".to_string(),
        version: "1.0.0".to_string(),
        description: "Advanced tool with missing dependency".to_string(),
        source: "http://api.example.com/advanced".to_string(),
        tool_type: ToolType::Http,
        dependencies: vec!["missing_dependency".to_string()],
        tags: vec![],
        metadata: HashMap::new(),
    };

    match discovery.register_metadata(missing_dep_tool.clone()).await {
        Ok(_) => {
            println!("  ⚠️  工具注册成功（但有缺失的依赖）");
            println!("  工具名称: {}", missing_dep_tool.name);
            println!("  缺失依赖: {:?}", missing_dep_tool.dependencies);
        }
        Err(e) => {
            error!("  ❌ 注册失败: {}", e);
        }
    }

    // 4. 列出所有工具及其依赖
    println!("\n4️⃣ 列出所有工具及其依赖:");
    let all_tools = discovery.list_metadata().await;
    println!("  总共 {} 个工具:", all_tools.len());
    for tool in &all_tools {
        if tool.dependencies.is_empty() {
            println!("    - {} (无依赖)", tool.name);
        } else {
            println!("    - {} (依赖: {:?})", tool.name, tool.dependencies);
        }
    }
}

