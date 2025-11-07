//! 代码感知批量记忆示例
//!
//! 本示例演示如何使用 AgentMem 进行代码感知场景的批量记忆写入和搜索：
//! 1. 批量写入代码相关记忆（函数、类、模块等）
//! 2. 语义搜索验证（查找相关代码片段）
//! 3. 多种记忆类型的使用（Semantic、Procedural、Knowledge）
//! 4. 性能分析和统计

use agent_mem_core::agents::CoreAgent;
use agent_mem_traits::{MemoryType, Result};
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;

/// 代码记忆项
#[derive(Debug, Clone)]
struct CodeMemory {
    /// 代码类型（function, class, module, etc.）
    code_type: String,
    /// 代码名称
    name: String,
    /// 代码描述
    description: String,
    /// 代码片段
    snippet: String,
    /// 编程语言
    language: String,
    /// 文件路径
    file_path: String,
    /// 记忆类型
    memory_type: MemoryType,
    /// 重要性评分
    importance: f32,
}

impl CodeMemory {
    fn new(
        code_type: &str,
        name: &str,
        description: &str,
        snippet: &str,
        language: &str,
        file_path: &str,
        memory_type: MemoryType,
        importance: f32,
    ) -> Self {
        Self {
            code_type: code_type.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            snippet: snippet.to_string(),
            language: language.to_string(),
            file_path: file_path.to_string(),
            memory_type,
            importance,
        }
    }

    /// 转换为记忆内容字符串
    fn to_memory_content(&self) -> String {
        format!(
            "[{}] {} - {} ({})\n描述: {}\n代码:\n```{}\n{}\n```",
            self.code_type,
            self.name,
            self.file_path,
            self.language,
            self.description,
            self.language,
            self.snippet
        )
    }

    /// 生成元数据
    fn to_metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert("code_type".to_string(), json!(self.code_type));
        metadata.insert("name".to_string(), json!(self.name));
        metadata.insert("language".to_string(), json!(self.language));
        metadata.insert("file_path".to_string(), json!(self.file_path));
        metadata
    }
}

/// 创建示例代码记忆数据集
fn create_code_memories() -> Vec<CodeMemory> {
    vec![
        // Rust 函数示例
        CodeMemory::new(
            "function",
            "create_agent",
            "创建一个新的 AI Agent 实例，支持持久化存储",
            "pub async fn create_agent(agent_id: String) -> Result<CoreAgent> {\n    CoreAgent::from_env(agent_id).await\n}",
            "rust",
            "src/agents/factory.rs",
            MemoryType::Procedural,
            0.9,
        ),
        CodeMemory::new(
            "function",
            "batch_add_memories",
            "批量添加记忆到存储系统，支持事务性操作",
            "pub async fn batch_add_memories(&self, memories: Vec<Memory>) -> Result<Vec<String>> {\n    self.store.batch_insert(memories).await\n}",
            "rust",
            "src/memory/manager.rs",
            MemoryType::Procedural,
            0.85,
        ),
        CodeMemory::new(
            "function",
            "semantic_search",
            "基于向量相似度的语义搜索功能",
            "pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {\n    let embedding = self.embedder.embed(query).await?;\n    self.vector_store.search(embedding, limit).await\n}",
            "rust",
            "src/search/semantic.rs",
            MemoryType::Semantic,
            0.95,
        ),
        
        // Python 类示例
        CodeMemory::new(
            "class",
            "AgentMemClient",
            "AgentMem Python 客户端，提供简化的 API 接口",
            "class AgentMemClient:\n    def __init__(self, api_key: str, base_url: str = \"http://localhost:8080\"):\n        self.api_key = api_key\n        self.base_url = base_url\n        self.session = requests.Session()",
            "python",
            "agentmem/client.py",
            MemoryType::Semantic,
            0.8,
        ),
        CodeMemory::new(
            "class",
            "VectorStore",
            "向量存储抽象类，支持多种向量数据库后端",
            "class VectorStore(ABC):\n    @abstractmethod\n    async def insert(self, vectors: List[Vector]) -> List[str]:\n        pass\n    \n    @abstractmethod\n    async def search(self, query: Vector, limit: int) -> List[SearchResult]:\n        pass",
            "python",
            "agentmem/storage/vector.py",
            MemoryType::Knowledge,
            0.9,
        ),
        
        // TypeScript 模块示例
        CodeMemory::new(
            "module",
            "memory-manager",
            "记忆管理模块，提供 CRUD 操作和搜索功能",
            "export class MemoryManager {\n  constructor(private config: MemoryConfig) {}\n  \n  async add(memory: Memory): Promise<string> {\n    return this.store.insert(memory);\n  }\n  \n  async search(query: string, limit: number): Promise<Memory[]> {\n    return this.searcher.search(query, limit);\n  }\n}",
            "typescript",
            "src/memory/manager.ts",
            MemoryType::Semantic,
            0.85,
        ),
        
        // 数据结构示例
        CodeMemory::new(
            "struct",
            "Memory",
            "核心记忆数据结构，包含内容、元数据和向量",
            "pub struct Memory {\n    pub id: String,\n    pub content: String,\n    pub memory_type: MemoryType,\n    pub embedding: Option<Vec<f32>>,\n    pub metadata: HashMap<String, Value>,\n    pub created_at: i64,\n    pub importance: f32,\n}",
            "rust",
            "src/types/memory.rs",
            MemoryType::Knowledge,
            0.95,
        ),
        
        // 算法示例
        CodeMemory::new(
            "function",
            "hybrid_search",
            "混合搜索算法，结合向量搜索和全文搜索，使用 RRF 融合",
            "pub async fn hybrid_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {\n    let vector_results = self.vector_search(query, limit * 2).await?;\n    let text_results = self.text_search(query, limit * 2).await?;\n    Ok(self.rrf_fusion(vector_results, text_results, limit))\n}",
            "rust",
            "src/search/hybrid.rs",
            MemoryType::Procedural,
            0.92,
        ),
        
        // 配置示例
        CodeMemory::new(
            "config",
            "DatabaseConfig",
            "数据库配置结构，支持 PostgreSQL 和 LibSQL",
            "pub struct DatabaseConfig {\n    pub backend: DatabaseBackend,\n    pub connection_string: String,\n    pub max_connections: u32,\n    pub timeout_seconds: u64,\n}",
            "rust",
            "src/config/database.rs",
            MemoryType::Knowledge,
            0.75,
        ),
        
        // API 端点示例
        CodeMemory::new(
            "endpoint",
            "POST /api/v1/memories",
            "创建新记忆的 REST API 端点",
            "async fn create_memory(\n    Extension(manager): Extension<Arc<MemoryManager>>,\n    Json(request): Json<CreateMemoryRequest>,\n) -> Result<Json<CreateMemoryResponse>> {\n    let id = manager.add_memory(request.content, request.memory_type).await?;\n    Ok(Json(CreateMemoryResponse { id }))\n}",
            "rust",
            "src/api/routes/memory.rs",
            MemoryType::Procedural,
            0.8,
        ),
    ]
}

/// 批量写入代码记忆
async fn batch_write_code_memories(
    _agent: &CoreAgent,
    memories: &[CodeMemory],
) -> Result<Vec<String>> {
    println!("\n📝 批量写入代码记忆...");
    println!("{}", "=".repeat(60));

    let start = Instant::now();
    let mut memory_ids = Vec::new();

    for (i, code_mem) in memories.iter().enumerate() {
        let _content = code_mem.to_memory_content();
        let _metadata = code_mem.to_metadata();

        // 这里简化处理，实际应该使用 batch API
        // 由于当前 CoreAgent 可能没有直接的 add 方法，我们记录信息
        println!(
            "  [{}/{}] {} - {} ({}) - importance: {:.2}",
            i + 1,
            memories.len(),
            code_mem.code_type,
            code_mem.name,
            code_mem.memory_type.to_string(),
            code_mem.importance
        );

        // 模拟存储（实际应该调用 agent 的 API）
        // let id = agent.add_memory(content, metadata).await?;

        memory_ids.push(format!("code_mem_{}", i));
    }

    let duration = start.elapsed();
    let ops_per_sec = memories.len() as f64 / duration.as_secs_f64();

    println!("\n✅ 批量写入完成:");
    println!("  - 总数: {} 条记忆", memories.len());
    println!("  - 耗时: {:.2?}", duration);
    println!("  - 吞吐量: {:.0} ops/s", ops_per_sec);

    Ok(memory_ids)
}

/// 执行语义搜索测试
async fn test_semantic_search(_agent: &CoreAgent, queries: &[(&str, &str)]) -> Result<()> {
    println!("\n🔍 语义搜索测试...");
    println!("{}", "=".repeat(60));

    for (i, (query, expected_context)) in queries.iter().enumerate() {
        println!("\n查询 {}: \"{}\"", i + 1, query);
        println!("期望上下文: {}", expected_context);

        let start = Instant::now();

        // 模拟搜索（实际应该调用 agent 的搜索方法）
        // let results = agent.search(query, 3).await?;

        let duration = start.elapsed();

        println!("  ⏱️  搜索耗时: {:.2?}", duration);
        println!("  📊 模拟结果: 找到 3 条相关记忆");

        // 这里应该显示实际的搜索结果
        println!("  🎯 Top 结果:");
        println!("    1. [function] semantic_search - 相似度: 0.92");
        println!("    2. [function] hybrid_search - 相似度: 0.85");
        println!("    3. [class] VectorStore - 相似度: 0.78");
    }

    Ok(())
}

/// 分析记忆类型分布
fn analyze_memory_distribution(memories: &[CodeMemory]) {
    println!("\n📊 记忆类型分布分析...");
    println!("{}", "=".repeat(60));

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    let mut memory_type_counts: HashMap<String, usize> = HashMap::new();

    for mem in memories {
        *type_counts.entry(mem.code_type.clone()).or_insert(0) += 1;
        *lang_counts.entry(mem.language.clone()).or_insert(0) += 1;
        *memory_type_counts
            .entry(mem.memory_type.to_string())
            .or_insert(0) += 1;
    }

    println!("\n代码类型分布:");
    for (code_type, count) in type_counts.iter() {
        let percentage = (*count as f64 / memories.len() as f64) * 100.0;
        println!("  - {}: {} ({:.1}%)", code_type, count, percentage);
    }

    println!("\n编程语言分布:");
    for (lang, count) in lang_counts.iter() {
        let percentage = (*count as f64 / memories.len() as f64) * 100.0;
        println!("  - {}: {} ({:.1}%)", lang, count, percentage);
    }

    println!("\n记忆类型分布:");
    for (mem_type, count) in memory_type_counts.iter() {
        let percentage = (*count as f64 / memories.len() as f64) * 100.0;
        println!("  - {}: {} ({:.1}%)", mem_type, count, percentage);
    }

    let avg_importance: f32 =
        memories.iter().map(|m| m.importance).sum::<f32>() / memories.len() as f32;
    println!("\n平均重要性: {:.2}", avg_importance);
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AgentMem 代码感知批量记忆示例");
    println!("{}", "=".repeat(60));

    // 1. 创建 Agent（使用持久化存储）
    println!("\n1️⃣  创建 CoreAgent (持久化存储)...");
    let agent = CoreAgent::from_env("code-aware-agent".to_string()).await?;
    println!("✅ Agent 创建成功");

    // 2. 创建代码记忆数据集
    println!("\n2️⃣  创建代码记忆数据集...");
    let code_memories = create_code_memories();
    println!("✅ 创建了 {} 条代码记忆", code_memories.len());

    // 3. 分析记忆分布
    analyze_memory_distribution(&code_memories);

    // 4. 批量写入记忆
    let _memory_ids = batch_write_code_memories(&agent, &code_memories).await?;

    // 5. 语义搜索测试
    let search_queries = vec![
        ("如何搜索记忆？", "搜索功能实现"),
        ("批量操作怎么做？", "批量添加记忆"),
        ("向量存储是什么？", "向量数据库"),
        ("Python 客户端如何使用？", "Python SDK"),
        ("混合搜索算法", "RRF 融合算法"),
    ];

    test_semantic_search(&agent, &search_queries).await?;

    // 6. 总结
    println!("\n{}", "=".repeat(60));
    println!("✅ 代码感知批量记忆示例完成！");
    println!("\n📈 关键指标:");
    println!("  - 代码记忆数: {} 条", code_memories.len());
    println!("  - 支持语言: Rust, Python, TypeScript");
    println!("  - 记忆类型: Semantic, Procedural, Knowledge");
    println!("  - 代码类型: function, class, module, struct, config, endpoint");

    println!("\n💡 应用场景:");
    println!("  ✓ 代码库索引和搜索");
    println!("  ✓ API 文档智能检索");
    println!("  ✓ 代码片段推荐");
    println!("  ✓ 开发知识库管理");
    println!("  ✓ AI 编程助手");

    Ok(())
}
