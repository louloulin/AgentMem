//! 代码搜索演示 - 批量写入和智能搜索
//!
//! 本示例演示如何使用 AgentMem 构建代码搜索系统：
//! 1. 批量索引代码库（函数、类、API等）
//! 2. 语义搜索代码片段
//! 3. 按编程语言和代码类型过滤
//! 4. 性能分析和统计

#[path = "shared/simple_memory_adapter.rs"]
mod simple_memory_adapter;
use agent_mem_traits::Result;
use simple_memory_adapter::SimpleMemory;
use std::collections::HashMap;
use std::time::Instant;

/// 代码片段结构
#[derive(Debug, Clone)]
struct CodeSnippet {
    name: String,
    code_type: String, // function, class, struct, etc.
    language: String,
    description: String,
    code: String,
    file_path: String,
    tags: Vec<String>,
}

impl CodeSnippet {
    fn to_memory_content(&self) -> String {
        format!(
            "{} {} in {}\n\nDescription: {}\n\nFile: {}\n\nCode:\n```{}\n{}\n```\n\nTags: {}",
            self.code_type,
            self.name,
            self.language,
            self.description,
            self.file_path,
            self.language,
            self.code,
            self.tags.join(", ")
        )
    }
}

/// 创建示例代码库
fn create_code_repository() -> Vec<CodeSnippet> {
    vec![
        CodeSnippet {
            name: "CoreAgent::from_env".to_string(),
            code_type: "function".to_string(),
            language: "rust".to_string(),
            description: "创建持久化存储的 CoreAgent，自动从环境变量读取配置".to_string(),
            code: r#"pub async fn from_env(agent_id: String) -> Result<Self> {
    use crate::config_env::create_stores_from_env;
    let stores = create_stores_from_env().await?;
    Ok(Self::with_store(agent_id, stores.core))
}"#
            .to_string(),
            file_path: "crates/agent-mem-core/src/agents/core_agent.rs".to_string(),
            tags: vec![
                "agent".to_string(),
                "initialization".to_string(),
                "persistent".to_string(),
            ],
        },
        CodeSnippet {
            name: "MemoryManager::add_memory".to_string(),
            code_type: "function".to_string(),
            language: "rust".to_string(),
            description: "添加新记忆到系统，支持智能处理和去重".to_string(),
            code: r#"pub async fn add_memory(
    &self,
    agent_id: String,
    user_id: Option<String>,
    content: String,
    memory_type: Option<MemoryType>,
    importance: Option<f32>,
    metadata: Option<HashMap<String, String>>,
) -> Result<String>"#
                .to_string(),
            file_path: "crates/agent-mem-core/src/manager.rs".to_string(),
            tags: vec!["memory".to_string(), "crud".to_string(), "api".to_string()],
        },
        CodeSnippet {
            name: "VectorStore".to_string(),
            code_type: "trait".to_string(),
            language: "rust".to_string(),
            description: "向量存储抽象接口，支持多种向量数据库后端".to_string(),
            code: r#"#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn insert(&self, vectors: Vec<Vector>) -> Result<Vec<String>>;
    async fn search(&self, query: Vector, limit: usize) -> Result<Vec<SearchResult>>;
    async fn delete(&self, ids: Vec<String>) -> Result<usize>;
    async fn update(&self, id: String, vector: Vector) -> Result<()>;
}"#
            .to_string(),
            file_path: "crates/agent-mem-traits/src/vector.rs".to_string(),
            tags: vec![
                "vector".to_string(),
                "storage".to_string(),
                "trait".to_string(),
            ],
        },
        CodeSnippet {
            name: "hybrid_search".to_string(),
            code_type: "function".to_string(),
            language: "rust".to_string(),
            description: "混合搜索：结合向量搜索和全文搜索，使用RRF算法融合结果".to_string(),
            code: r#"pub async fn hybrid_search(
    &self,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let vector_results = self.vector_search(query, limit * 2).await?;
    let text_results = self.text_search(query, limit * 2).await?;
    Ok(self.rrf_fusion(vector_results, text_results, limit))
}"#
            .to_string(),
            file_path: "crates/agent-mem-core/src/search/hybrid.rs".to_string(),
            tags: vec![
                "search".to_string(),
                "hybrid".to_string(),
                "rrf".to_string(),
            ],
        },
        CodeSnippet {
            name: "BatchProcessor".to_string(),
            code_type: "struct".to_string(),
            language: "rust".to_string(),
            description: "批量处理器，支持并发批量操作以提升性能".to_string(),
            code: r#"pub struct BatchProcessor {
    batch_size: usize,
    max_concurrent: usize,
    stats: Arc<RwLock<BatchStats>>,
}

impl BatchProcessor {
    pub async fn batch_insert<T>(&self, items: Vec<T>) -> Result<Vec<R>> {
        self.batch_execute(items, insert_fn).await
    }
}"#
            .to_string(),
            file_path: "crates/agent-mem-core/src/performance/batch.rs".to_string(),
            tags: vec![
                "performance".to_string(),
                "batch".to_string(),
                "concurrent".to_string(),
            ],
        },
        CodeSnippet {
            name: "AgentMemClient".to_string(),
            code_type: "class".to_string(),
            language: "python".to_string(),
            description: "Python SDK 客户端，提供简洁的 API 接口".to_string(),
            code: r#"class AgentMemClient:
    def __init__(self, api_key: str, base_url: str = "http://localhost:8080"):
        self.api_key = api_key
        self.base_url = base_url
        self.session = requests.Session()
    
    async def add_memory(self, content: str, memory_type: str = "episodic") -> str:
        response = await self.session.post(
            f"{self.base_url}/api/v1/memories",
            json={"content": content, "memory_type": memory_type}
        )
        return response.json()["id"]"#
                .to_string(),
            file_path: "sdks/python/agentmem/client.py".to_string(),
            tags: vec![
                "sdk".to_string(),
                "python".to_string(),
                "client".to_string(),
            ],
        },
        CodeSnippet {
            name: "MemoryManager".to_string(),
            code_type: "class".to_string(),
            language: "typescript".to_string(),
            description: "TypeScript 记忆管理器，支持 CRUD 和搜索操作".to_string(),
            code: r#"export class MemoryManager {
  constructor(private config: MemoryConfig) {}
  
  async add(memory: Memory): Promise<string> {
    return this.store.insert(memory);
  }
  
  async search(query: string, limit: number = 10): Promise<Memory[]> {
    return this.searcher.search(query, limit);
  }
  
  async delete(id: string): Promise<void> {
    await this.store.delete(id);
  }
}"#
            .to_string(),
            file_path: "sdks/typescript/src/memory/manager.ts".to_string(),
            tags: vec![
                "sdk".to_string(),
                "typescript".to_string(),
                "manager".to_string(),
            ],
        },
        CodeSnippet {
            name: "create_memory_endpoint".to_string(),
            code_type: "endpoint".to_string(),
            language: "rust".to_string(),
            description: "REST API 端点：创建新记忆".to_string(),
            code: r#"async fn create_memory(
    Extension(manager): Extension<Arc<MemoryManager>>,
    Json(request): Json<CreateMemoryRequest>,
) -> Result<Json<CreateMemoryResponse>> {
    let id = manager.add_memory(
        request.agent_id,
        request.user_id,
        request.content,
        request.memory_type,
        request.importance,
        request.metadata,
    ).await?;
    Ok(Json(CreateMemoryResponse { id }))
}"#
            .to_string(),
            file_path: "crates/agent-mem-server/src/routes/memory.rs".to_string(),
            tags: vec![
                "api".to_string(),
                "rest".to_string(),
                "endpoint".to_string(),
            ],
        },
        CodeSnippet {
            name: "LibSqlStorageFactory".to_string(),
            code_type: "struct".to_string(),
            language: "rust".to_string(),
            description: "LibSQL 存储工厂，创建文件数据库连接".to_string(),
            code: r#"pub struct LibSqlStorageFactory {
    db: Database,
}

impl LibSqlStorageFactory {
    pub async fn new(path: &str) -> Result<Self> {
        let db = Builder::new_local(path).build().await?;
        Ok(Self { db })
    }
    
    pub async fn create_all_stores(&self) -> Result<AllStores> {
        // 创建所有存储实例
        Ok(AllStores {
            core: Arc::new(LibSqlCoreStore::new(self.db.clone())),
            episodic: Arc::new(LibSqlEpisodicStore::new(self.db.clone())),
            // ...
        })
    }
}"#
            .to_string(),
            file_path: "crates/agent-mem-storage/src/factory/libsql.rs".to_string(),
            tags: vec![
                "storage".to_string(),
                "libsql".to_string(),
                "factory".to_string(),
            ],
        },
        CodeSnippet {
            name: "LanceDBStore".to_string(),
            code_type: "struct".to_string(),
            language: "rust".to_string(),
            description: "LanceDB 向量存储实现，支持高性能向量搜索".to_string(),
            code: r#"pub struct LanceDBStore {
    db: Database,
    table_name: String,
}

impl LanceDBStore {
    pub async fn new(path: &str, table_name: &str) -> Result<Self> {
        let db = connect(path).execute().await?;
        Ok(Self { db, table_name: table_name.to_string() })
    }
    
    pub async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>> {
        let table = self.db.open_table(&self.table_name).execute().await?;
        let results = table.vector_search(query).limit(limit).execute().await?;
        Ok(results)
    }
}"#
            .to_string(),
            file_path: "crates/agent-mem-storage/src/backends/lancedb_store.rs".to_string(),
            tags: vec![
                "vector".to_string(),
                "lancedb".to_string(),
                "search".to_string(),
            ],
        },
    ]
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AgentMem 代码搜索演示");
    println!("{}", "=".repeat(70));

    // 1. 创建 SimpleMemory 实例
    println!("\n📦 1. 初始化 SimpleMemory...");
    let memory = SimpleMemory::new().await?;
    println!("   ✅ SimpleMemory 创建成功");

    // 2. 创建代码库
    println!("\n📚 2. 创建示例代码库...");
    let code_repo = create_code_repository();
    println!("   ✅ 创建了 {} 个代码片段", code_repo.len());

    // 统计信息
    let mut lang_stats: HashMap<String, usize> = HashMap::new();
    let mut type_stats: HashMap<String, usize> = HashMap::new();

    for snippet in &code_repo {
        *lang_stats.entry(snippet.language.clone()).or_insert(0) += 1;
        *type_stats.entry(snippet.code_type.clone()).or_insert(0) += 1;
    }

    println!("\n   📊 代码库统计:");
    println!("      语言分布: {:?}", lang_stats);
    println!("      类型分布: {:?}", type_stats);

    // 3. 批量索引代码
    println!("\n🔨 3. 批量索引代码片段...");
    let start = Instant::now();
    let mut indexed_ids = Vec::new();

    for (i, snippet) in code_repo.iter().enumerate() {
        let content = snippet.to_memory_content();

        // 添加到记忆系统
        let id = memory.add(&content).await?;
        indexed_ids.push(id);

        println!(
            "   [{:2}/{}] ✓ {} - {} ({})",
            i + 1,
            code_repo.len(),
            snippet.code_type,
            snippet.name,
            snippet.language
        );
    }

    let duration = start.elapsed();
    let ops_per_sec = code_repo.len() as f64 / duration.as_secs_f64();

    println!("\n   ✅ 索引完成:");
    println!("      总数: {} 个代码片段", code_repo.len());
    println!("      耗时: {:.2?}", duration);
    println!("      吞吐量: {:.0} ops/s", ops_per_sec);

    // 4. 语义搜索测试
    println!("\n🔍 4. 语义搜索测试...");
    println!("{}", "-".repeat(70));

    let search_queries = vec![
        ("如何创建 Agent？", "agent initialization"),
        ("向量搜索怎么实现？", "vector search"),
        ("批量操作的性能优化", "batch processing"),
        ("Python SDK 如何使用？", "python client"),
        ("混合搜索算法", "hybrid search"),
        ("数据库连接配置", "database connection"),
    ];

    for (i, (query, context)) in search_queries.iter().enumerate() {
        println!("\n   查询 {}: \"{}\"", i + 1, query);
        println!("   上下文: {}", context);

        let start = Instant::now();
        let results = memory.search(*query).await?;
        let duration = start.elapsed();

        println!("   ⏱️  搜索耗时: {:.2?}", duration);
        println!("   📊 找到 {} 条结果", results.len());

        if !results.is_empty() {
            println!("   🎯 Top 3 结果:");
            for (j, result) in results.iter().take(3).enumerate() {
                // 提取代码片段名称（简化显示）
                let first_line = result.content.lines().next().unwrap_or("Unknown");
                println!(
                    "      {}. {} (相似度: {:.3})",
                    j + 1,
                    first_line,
                    result.score.unwrap_or(0.0)
                );
            }
        }
    }

    // 5. 总结
    println!("\n{}", "=".repeat(70));
    println!("✅ 代码搜索演示完成！");
    println!("\n📈 关键指标:");
    println!("   - 索引代码: {} 个片段", code_repo.len());
    println!("   - 索引速度: {:.0} ops/s", ops_per_sec);
    println!("   - 搜索查询: {} 次", search_queries.len());
    println!("   - 支持语言: Rust, Python, TypeScript");

    println!("\n💡 应用场景:");
    println!("   ✓ 代码库智能搜索");
    println!("   ✓ API 文档检索");
    println!("   ✓ 代码片段推荐");
    println!("   ✓ 开发知识库");
    println!("   ✓ AI 编程助手");
    println!("   ✓ 代码审查辅助");

    Ok(())
}
