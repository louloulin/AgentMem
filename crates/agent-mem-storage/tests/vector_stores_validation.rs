//! 向量服务验证测试
//!
//! 此文件用于验证所有向量服务是否正确实现了 VectorStore trait
//! 这是编译时验证，确保所有向量服务都实现了必要的方法

use agent_mem_traits::{VectorData, VectorSearchResult, VectorStore};

/// 验证向量服务是否实现了 VectorStore trait
/// 这是一个编译时检查，如果任何向量服务没有正确实现 trait，编译将失败
#[allow(dead_code)]
fn verify_vector_store_trait<T: VectorStore>() {
    // 这个函数不会被调用，只是用于编译时类型检查
}

#[test]
fn test_all_vector_stores_implement_trait() {
    // 1. Memory Store - 内存向量存储
    #[cfg(feature = "memory")]
    {
        use agent_mem_storage::backends::memory::MemoryVectorStore;
        verify_vector_store_trait::<MemoryVectorStore>();
        println!("✅ MemoryVectorStore implements VectorStore trait");
    }

    // 2. LanceDB Store - 嵌入式向量数据库
    #[cfg(feature = "lancedb")]
    {
        use agent_mem_storage::backends::lancedb_store::LanceDBStore;
        verify_vector_store_trait::<LanceDBStore>();
        println!("✅ LanceDBStore implements VectorStore trait");
    }

    // 3. Chroma Store - 开源向量数据库
    #[cfg(feature = "chroma")]
    {
        use agent_mem_storage::backends::chroma::ChromaStore;
        verify_vector_store_trait::<ChromaStore>();
        println!("✅ ChromaStore implements VectorStore trait");
    }

    // 4. Pinecone Store - 云端向量数据库
    #[cfg(feature = "pinecone")]
    {
        use agent_mem_storage::backends::pinecone::PineconeStore;
        verify_vector_store_trait::<PineconeStore>();
        println!("✅ PineconeStore implements VectorStore trait");
    }

    // 5. Qdrant Store - 高性能向量搜索引擎
    #[cfg(feature = "qdrant")]
    {
        use agent_mem_storage::backends::qdrant::QdrantStore;
        verify_vector_store_trait::<QdrantStore>();
        println!("✅ QdrantStore implements VectorStore trait");
    }

    // 6. Milvus Store - 分布式向量数据库
    #[cfg(feature = "milvus")]
    {
        use agent_mem_storage::backends::milvus::MilvusStore;
        verify_vector_store_trait::<MilvusStore>();
        println!("✅ MilvusStore implements VectorStore trait");
    }

    // 7. Weaviate Store - 知识图谱向量数据库
    #[cfg(feature = "weaviate")]
    {
        use agent_mem_storage::backends::weaviate::WeaviateStore;
        verify_vector_store_trait::<WeaviateStore>();
        println!("✅ WeaviateStore implements VectorStore trait");
    }

    // 8. Elasticsearch Store - 搜索引擎向量支持
    #[cfg(feature = "elasticsearch")]
    {
        use agent_mem_storage::backends::elasticsearch::ElasticsearchStore;
        verify_vector_store_trait::<ElasticsearchStore>();
        println!("✅ ElasticsearchStore implements VectorStore trait");
    }

    // 9. Redis Store - 内存数据库向量支持
    #[cfg(feature = "redis")]
    {
        use agent_mem_storage::backends::redis::RedisStore;
        verify_vector_store_trait::<RedisStore>();
        println!("✅ RedisStore implements VectorStore trait");
    }

    // 10. MongoDB Store - 文档数据库向量支持
    #[cfg(feature = "mongodb")]
    {
        use agent_mem_storage::backends::mongodb::MongoDBStore;
        verify_vector_store_trait::<MongoDBStore>();
        println!("✅ MongoDBStore implements VectorStore trait");
    }

    // 11. Supabase Store - PostgreSQL + pgvector
    #[cfg(feature = "supabase")]
    {
        use agent_mem_storage::backends::supabase::SupabaseStore;
        verify_vector_store_trait::<SupabaseStore>();
        println!("✅ SupabaseStore implements VectorStore trait");
    }

    // 12. FAISS Store - 本地高性能向量搜索
    #[cfg(feature = "faiss")]
    {
        use agent_mem_storage::backends::faiss::FaissStore;
        verify_vector_store_trait::<FaissStore>();
        println!("✅ FaissStore implements VectorStore trait");
    }

    // 13. Azure AI Search Store - 企业级搜索服务
    #[cfg(feature = "azure-ai-search")]
    {
        use agent_mem_storage::backends::azure_ai_search::AzureAISearchStore;
        verify_vector_store_trait::<AzureAISearchStore>();
        println!("✅ AzureAISearchStore implements VectorStore trait");
    }

    println!("\n🎉 所有启用的向量服务都正确实现了 VectorStore trait！");
}

/// 验证向量服务的核心方法签名
#[test]
fn test_vector_store_method_signatures() {
    // 这个测试验证 VectorStore trait 的方法签名是否正确
    use agent_mem_traits::VectorStore;
    use std::future::Future;

    // 验证 add_vectors 方法
    fn check_add_vectors<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<Vec<String>>> + '_ {
        store.add_vectors(vec![])
    }

    // 验证 search_vectors 方法
    fn check_search_vectors<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<Vec<VectorSearchResult>>> + '_ {
        store.search_vectors(vec![], 10, None)
    }

    // 验证 delete_vectors 方法
    fn check_delete_vectors<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<()>> + '_ {
        store.delete_vectors(vec![])
    }

    // 验证 update_vectors 方法
    fn check_update_vectors<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<()>> + '_ {
        store.update_vectors(vec![])
    }

    // 验证 get_vector 方法
    fn check_get_vector<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<Option<VectorData>>> + '_ {
        store.get_vector("")
    }

    // 验证 count_vectors 方法
    fn check_count_vectors<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<usize>> + '_ {
        store.count_vectors()
    }

    // 验证 health_check 方法
    fn check_health_check<T: VectorStore>(
        store: &T,
    ) -> impl Future<Output = agent_mem_traits::Result<agent_mem_traits::HealthStatus>> + '_ {
        store.health_check()
    }

    println!("✅ 所有 VectorStore trait 方法签名验证通过");
}

/// 统计已实现的向量服务数量
#[test]
fn test_count_implemented_vector_stores() {
    let mut count = 0;
    let mut services = Vec::new();

    #[cfg(feature = "memory")]
    {
        count += 1;
        services.push("Memory");
    }

    #[cfg(feature = "lancedb")]
    {
        count += 1;
        services.push("LanceDB");
    }

    #[cfg(feature = "chroma")]
    {
        count += 1;
        services.push("Chroma");
    }

    #[cfg(feature = "pinecone")]
    {
        count += 1;
        services.push("Pinecone");
    }

    #[cfg(feature = "qdrant")]
    {
        count += 1;
        services.push("Qdrant");
    }

    #[cfg(feature = "milvus")]
    {
        count += 1;
        services.push("Milvus");
    }

    #[cfg(feature = "weaviate")]
    {
        count += 1;
        services.push("Weaviate");
    }

    #[cfg(feature = "elasticsearch")]
    {
        count += 1;
        services.push("Elasticsearch");
    }

    #[cfg(feature = "redis")]
    {
        count += 1;
        services.push("Redis");
    }

    #[cfg(feature = "mongodb")]
    {
        count += 1;
        services.push("MongoDB");
    }

    #[cfg(feature = "supabase")]
    {
        count += 1;
        services.push("Supabase");
    }

    #[cfg(feature = "faiss")]
    {
        count += 1;
        services.push("FAISS");
    }

    #[cfg(feature = "azure-ai-search")]
    {
        count += 1;
        services.push("Azure AI Search");
    }

    println!("\n📊 向量服务统计:");
    println!("   已启用的向量服务数量: {count}");
    println!("   已启用的向量服务列表:");
    for service in &services {
        println!("   - {service}");
    }

    // 验证至少有 Memory 和 LanceDB 两个服务
    assert!(count >= 2, "至少应该有 Memory 和 LanceDB 两个向量服务");
}
