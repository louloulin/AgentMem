//! Phase 1 集成演示 - 展示核心存储后端的真实实现
//!
//! 这个演示展示了：
//! 1. Chroma 向量存储的真实连接和操作
//! 2. OpenAI 嵌入服务的真实 API 调用
//! 3. Neo4j 图数据库的真实连接和操作
//! 4. 各组件之间的集成工作

// 移除未使用的导入
use agent_mem_config::memory::GraphStoreConfig;
use agent_mem_embeddings::config::EmbeddingConfig;
use agent_mem_embeddings::providers::OpenAIEmbedder;
use agent_mem_storage::backends::ChromaStore;
use agent_mem_storage::graph::Neo4jStore;
use agent_mem_traits::{
    Embedder, Entity, GraphStore, Relation, Session, VectorData, VectorStore, VectorStoreConfig,
};
// 移除未使用的导入
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    info!("🚀 启动 Phase 1 集成演示");

    // 检查环境变量
    let demo_mode = check_environment();

    match demo_mode {
        DemoMode::Full => {
            info!("🔥 运行完整演示（需要真实服务）");
            run_full_demo().await?;
        }
        DemoMode::Mock => {
            info!("🧪 运行模拟演示（不需要真实服务）");
            run_mock_demo().await?;
        }
        DemoMode::Partial => {
            info!("⚡ 运行部分演示（部分真实服务）");
            run_partial_demo().await?;
        }
    }

    info!("✅ Phase 1 集成演示完成");
    Ok(())
}

#[derive(Debug)]
enum DemoMode {
    Full,    // 所有服务都可用
    Mock,    // 所有服务都模拟
    Partial, // 部分服务可用
}

fn check_environment() -> DemoMode {
    let has_openai = std::env::var("OPENAI_API_KEY").is_ok();
    let has_chroma = std::env::var("CHROMA_TEST_ENABLED").is_ok();
    let has_neo4j = std::env::var("NEO4J_TEST_ENABLED").is_ok();

    info!("🔍 环境检查:");
    info!("  OpenAI API Key: {}", if has_openai { "✅" } else { "❌" });
    info!("  Chroma 服务: {}", if has_chroma { "✅" } else { "❌" });
    info!("  Neo4j 服务: {}", if has_neo4j { "✅" } else { "❌" });

    if has_openai && has_chroma && has_neo4j {
        DemoMode::Full
    } else if has_openai || has_chroma || has_neo4j {
        DemoMode::Partial
    } else {
        DemoMode::Mock
    }
}

async fn run_full_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 开始完整演示");

    // 1. 测试 OpenAI 嵌入服务
    info!("📝 测试 OpenAI 嵌入服务");
    let embedder = create_openai_embedder().await?;
    let test_text = "这是一个测试文本，用于验证嵌入功能";
    let embedding = embedder.embed(test_text).await?;
    info!("✅ 成功生成嵌入向量，维度: {}", embedding.len());

    // 2. 测试 Chroma 向量存储
    info!("🗄️ 测试 Chroma 向量存储");
    let chroma_store = create_chroma_store().await?;

    let vector_data = VectorData {
        id: Uuid::new_v4().to_string(),
        vector: embedding.clone(),
        metadata: {
            let mut map = HashMap::new();
            map.insert("content".to_string(), test_text.to_string());
            map.insert("type".to_string(), "test".to_string());
            map
        },
    };

    let ids = chroma_store.add_vectors(vec![vector_data.clone()]).await?;
    info!("✅ 成功存储向量，ID: {:?}", ids);

    // 搜索测试
    let search_results = chroma_store.search_vectors(embedding, 5, Some(0.7)).await?;
    info!("✅ 搜索到 {} 个相似向量", search_results.len());

    // 3. 测试 Neo4j 图数据库
    info!("🕸️ 测试 Neo4j 图数据库");
    let neo4j_store = create_neo4j_store().await?;

    let session = Session::new()
        .with_agent_id(Some("demo-agent".to_string()))
        .with_user_id(Some("demo-user".to_string()));

    // 创建测试实体
    let entities = vec![
        Entity {
            id: "person-1".to_string(),
            entity_type: "Person".to_string(),
            name: "张三".to_string(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("age".to_string(), serde_json::Value::Number(30.into()));
                attrs.insert(
                    "city".to_string(),
                    serde_json::Value::String("北京".to_string()),
                );
                attrs
            },
        },
        Entity {
            id: "person-2".to_string(),
            entity_type: "Person".to_string(),
            name: "李四".to_string(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("age".to_string(), serde_json::Value::Number(25.into()));
                attrs.insert(
                    "city".to_string(),
                    serde_json::Value::String("上海".to_string()),
                );
                attrs
            },
        },
    ];

    neo4j_store.add_entities(&entities, &session).await?;
    info!("✅ 成功添加 {} 个实体", entities.len());

    // 创建关系
    let relations = vec![Relation {
        relation_type: "朋友".to_string(),
        source: "person-1".to_string(),
        target: "person-2".to_string(),
        confidence: 0.9,
    }];

    neo4j_store.add_relations(&relations, &session).await?;
    info!("✅ 成功添加 {} 个关系", relations.len());

    // 图搜索测试
    let search_results = neo4j_store.search_graph("张三", &session).await?;
    info!("✅ 图搜索找到 {} 个结果", search_results.len());

    info!("🎉 完整演示成功完成！");
    Ok(())
}

async fn run_mock_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 开始模拟演示");

    // 模拟嵌入向量生成
    info!("📝 模拟嵌入向量生成");
    let mock_embedding = vec![0.1; 1536]; // 模拟 OpenAI ada-002 维度
    info!("✅ 生成模拟嵌入向量，维度: {}", mock_embedding.len());

    // 模拟向量存储操作
    info!("🗄️ 模拟向量存储操作");
    let vector_data = VectorData {
        id: Uuid::new_v4().to_string(),
        vector: mock_embedding.clone(),
        metadata: {
            let mut map = HashMap::new();
            map.insert("content".to_string(), "模拟测试内容".to_string());
            map.insert("type".to_string(), "mock".to_string());
            map
        },
    };
    info!("✅ 创建模拟向量数据: {}", vector_data.id);

    // 模拟图数据库操作
    info!("🕸️ 模拟图数据库操作");
    let mock_entity = Entity {
        id: "mock-entity-1".to_string(),
        entity_type: "MockEntity".to_string(),
        name: "模拟实体".to_string(),
        attributes: HashMap::new(),
    };
    info!("✅ 创建模拟实体: {}", mock_entity.name);

    info!("🎉 模拟演示成功完成！");
    Ok(())
}

async fn run_partial_demo() -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ 开始部分演示");

    // 根据可用服务运行相应的测试
    if std::env::var("OPENAI_API_KEY").is_ok() {
        info!("📝 测试 OpenAI 嵌入服务");
        match create_openai_embedder().await {
            Ok(embedder) => {
                let embedding = embedder.embed("部分演示测试").await?;
                info!("✅ OpenAI 嵌入服务正常，维度: {}", embedding.len());
            }
            Err(e) => {
                warn!("⚠️ OpenAI 嵌入服务测试失败: {}", e);
            }
        }
    }

    if std::env::var("CHROMA_TEST_ENABLED").is_ok() {
        info!("🗄️ 测试 Chroma 向量存储");
        match create_chroma_store().await {
            Ok(store) => {
                let count = store.count_vectors().await?;
                info!("✅ Chroma 向量存储正常，当前向量数: {}", count);
            }
            Err(e) => {
                warn!("⚠️ Chroma 向量存储测试失败: {}", e);
            }
        }
    }

    if std::env::var("NEO4J_TEST_ENABLED").is_ok() {
        info!("🕸️ 测试 Neo4j 图数据库");
        match create_neo4j_store().await {
            Ok(_store) => {
                info!("✅ Neo4j 图数据库连接正常");
            }
            Err(e) => {
                warn!("⚠️ Neo4j 图数据库测试失败: {}", e);
            }
        }
    }

    info!("🎉 部分演示成功完成！");
    Ok(())
}

async fn create_openai_embedder() -> Result<OpenAIEmbedder, Box<dyn std::error::Error>> {
    let config = EmbeddingConfig::openai(std::env::var("OPENAI_API_KEY").ok());

    Ok(OpenAIEmbedder::new(config).await?)
}

async fn create_chroma_store() -> Result<ChromaStore, Box<dyn std::error::Error>> {
    let config = VectorStoreConfig {
        provider: "chroma".to_string(),
        url: Some("http://localhost:8000".to_string()),
        collection_name: Some("phase1_demo".to_string()),
        dimension: Some(1536),
        ..Default::default()
    };

    Ok(ChromaStore::new(config).await?)
}

async fn create_neo4j_store() -> Result<Neo4jStore, Box<dyn std::error::Error>> {
    let config = GraphStoreConfig {
        provider: "neo4j".to_string(),
        uri: "bolt://localhost:7687".to_string(),
        username: Some("neo4j".to_string()),
        password: Some("password".to_string()),
        database: Some("neo4j".to_string()),
    };

    Ok(Neo4jStore::new(config).await?)
}
