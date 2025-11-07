//! 商品搜索演示 - 基于agent-mem-llm和混合检索系统
//! 
//! 这个演示展示了如何结合：
//! 1. agent-mem-llm - 理解用户查询意图
//! 2. EnhancedHybridSearchEngineV2 - 执行混合检索
//! 3. 实际的商品数据 - 电商场景

use agent_mem_core::search::{
    EnhancedHybridSearchEngineV2, QueryClassifier, AdaptiveThresholdCalculator,
    EnhancedHybridConfig, QueryClassifierConfig, AdaptiveThresholdConfig,
};
use agent_mem_storage::backends::{LibSQLFTS5Store, FTS5Config, LanceDBVectorStore};
use agent_mem_llm::{LLMFactory, LLMClient, LLMConfig, LLMProvider, Message};
use agent_mem_embeddings::{EmbeddingProvider, EmbeddingConfig};
use anyhow::{Result, Context};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// 商品信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    category: String,
    brand: String,
    description: String,
    price: f64,
    tags: Vec<String>,
    rating: f32,
    reviews_count: u32,
}

/// 搜索结果
#[derive(Debug, Serialize, Deserialize)]
struct ProductSearchResult {
    product: Product,
    score: f64,
    match_type: String, // "vector", "bm25", "hybrid"
    snippet: Option<String>,
}

/// 商品搜索引擎
struct ProductSearchEngine {
    hybrid_engine: EnhancedHybridSearchEngineV2,
    llm_client: Option<LLMClient>,
    products: Vec<Product>,
}

impl ProductSearchEngine {
    /// 创建新的商品搜索引擎
    async fn new(use_llm: bool) -> Result<Self> {
        info!("🚀 初始化商品搜索引擎...");
        
        // 1. 创建FTS5存储
        let fts5_config = FTS5Config {
            table_name: "products".to_string(),
            ..Default::default()
        };
        let fts5_store = LibSQLFTS5Store::new(":memory:", fts5_config).await?;
        
        // 2. 创建向量存储（内存模式）
        let vector_store = Arc::new(Mutex::new(
            LanceDBVectorStore::new_memory("product_vectors").await?
        ));
        
        // 3. 创建查询分类器
        let classifier = QueryClassifier::new(QueryClassifierConfig::default());
        
        // 4. 创建自适应阈值计算器
        let threshold_calc = AdaptiveThresholdCalculator::new(
            AdaptiveThresholdConfig::default()
        );
        
        // 5. 创建混合搜索引擎
        let config = EnhancedHybridConfig {
            vector_weight: 0.6,
            bm25_weight: 0.4,
            enable_parallel: true,
            rrf_k: 60.0,
            ..Default::default()
        };
        
        let hybrid_engine = EnhancedHybridSearchEngineV2::new(
            vector_store,
            fts5_store,
            classifier,
            threshold_calc,
            config,
        );
        
        // 6. 可选：创建LLM客户端
        let llm_client = if use_llm {
            info!("🤖 启用LLM查询理解...");
            let llm_config = LLMConfig {
                provider: LLMProvider::OpenAI,
                api_key: std::env::var("OPENAI_API_KEY").ok(),
                model: "gpt-3.5-turbo".to_string(),
                ..Default::default()
            };
            
            match LLMFactory::create_client(llm_config).await {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("⚠️  无法创建LLM客户端: {}，将使用基础搜索", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            hybrid_engine,
            llm_client,
            products: Vec::new(),
        })
    }
    
    /// 加载商品数据
    async fn load_products(&mut self) -> Result<()> {
        info!("📦 加载商品数据...");
        
        // 创建示例商品数据
        let products = vec![
            Product {
                id: "P001".to_string(),
                name: "iPhone 15 Pro".to_string(),
                category: "手机".to_string(),
                brand: "Apple".to_string(),
                description: "搭载A17 Pro芯片的旗舰手机，支持ProMotion 120Hz显示，钛金属边框，强大的相机系统".to_string(),
                price: 7999.0,
                tags: vec!["旗舰".to_string(), "5G".to_string(), "高端".to_string()],
                rating: 4.8,
                reviews_count: 15234,
            },
            Product {
                id: "P002".to_string(),
                name: "小米13 Ultra".to_string(),
                category: "手机".to_string(),
                brand: "Xiaomi".to_string(),
                description: "徕卡专业影像，骁龙8 Gen 2处理器，120W快充，2K AMOLED屏幕".to_string(),
                price: 5999.0,
                tags: vec!["拍照".to_string(), "性能".to_string(), "快充".to_string()],
                rating: 4.6,
                reviews_count: 8921,
            },
            Product {
                id: "P003".to_string(),
                name: "MacBook Pro 16".to_string(),
                category: "笔记本".to_string(),
                brand: "Apple".to_string(),
                description: "M3 Max芯片，64GB内存，16.2英寸Liquid Retina XDR显示屏，专业创作利器".to_string(),
                price: 25999.0,
                tags: vec!["专业".to_string(), "创作".to_string(), "高性能".to_string()],
                rating: 4.9,
                reviews_count: 3421,
            },
            Product {
                id: "P004".to_string(),
                name: "戴尔XPS 15".to_string(),
                category: "笔记本".to_string(),
                brand: "Dell".to_string(),
                description: "英特尔i7-13700H，NVIDIA RTX 4060，4K OLED触控屏，轻薄设计".to_string(),
                price: 12999.0,
                tags: vec!["高性能".to_string(), "触控".to_string(), "4K".to_string()],
                rating: 4.5,
                reviews_count: 2156,
            },
            Product {
                id: "P005".to_string(),
                name: "索尼WH-1000XM5".to_string(),
                category: "耳机".to_string(),
                brand: "Sony".to_string(),
                description: "业界领先的降噪技术，LDAC高解析音质，30小时续航，舒适佩戴".to_string(),
                price: 2799.0,
                tags: vec!["降噪".to_string(), "无线".to_string(), "音质".to_string()],
                rating: 4.7,
                reviews_count: 12453,
            },
            Product {
                id: "P006".to_string(),
                name: "AirPods Pro 2".to_string(),
                category: "耳机".to_string(),
                brand: "Apple".to_string(),
                description: "自适应降噪，空间音频，H2芯片，无线充电盒，完美适配苹果生态".to_string(),
                price: 1899.0,
                tags: vec!["降噪".to_string(), "无线".to_string(), "生态".to_string()],
                rating: 4.8,
                reviews_count: 23451,
            },
            Product {
                id: "P007".to_string(),
                name: "华为Mate 60 Pro".to_string(),
                category: "手机".to_string(),
                brand: "Huawei".to_string(),
                description: "麒麟9000S芯片，卫星通信，超光变XMAGE影像，昆仑玻璃".to_string(),
                price: 6999.0,
                tags: vec!["卫星".to_string(), "拍照".to_string(), "旗舰".to_string()],
                rating: 4.7,
                reviews_count: 18234,
            },
            Product {
                id: "P008".to_string(),
                name: "ThinkPad X1 Carbon".to_string(),
                category: "笔记本".to_string(),
                brand: "Lenovo".to_string(),
                description: "商务旗舰，碳纤维机身，i7-1365U，32GB内存，军标认证耐用".to_string(),
                price: 13999.0,
                tags: vec!["商务".to_string(), "轻薄".to_string(), "耐用".to_string()],
                rating: 4.6,
                reviews_count: 5432,
            },
        ];
        
        // 将商品数据插入FTS5和向量数据库
        for product in &products {
            // 构建搜索文档
            let doc_text = format!(
                "{} {} {} {} {}",
                product.name, product.brand, product.category,
                product.description, product.tags.join(" ")
            );
            
            // 插入FTS5
            self.hybrid_engine.fts5_store.insert_document(
                &product.id,
                &doc_text,
                None,
            ).await?;
            
            info!("  ✓ 已加载: {} ({})", product.name, product.id);
        }
        
        self.products = products;
        info!("✅ 成功加载 {} 个商品", self.products.len());
        
        Ok(())
    }
    
    /// 使用LLM理解查询意图
    async fn understand_query(&self, query: &str) -> Result<String> {
        if let Some(llm) = &self.llm_client {
            let prompt = format!(
                r#"你是一个电商搜索助手。用户查询："{}"

请分析用户的搜索意图，提取关键信息：
1. 商品类型
2. 品牌偏好
3. 关键特性
4. 价格范围

请用简洁的关键词总结，用于商品搜索。只输出关键词，用空格分隔。"#,
                query
            );
            
            let messages = vec![Message::user(prompt)];
            
            match llm.chat(messages, None).await {
                Ok(response) => {
                    info!("🤖 LLM理解: {} -> {}", query, response.content);
                    Ok(response.content)
                }
                Err(e) => {
                    warn!("⚠️  LLM理解失败: {}，使用原始查询", e);
                    Ok(query.to_string())
                }
            }
        } else {
            Ok(query.to_string())
        }
    }
    
    /// 搜索商品
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<ProductSearchResult>> {
        println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
        println!("{} {}", "🔍 搜索查询:".bright_cyan().bold(), query.bright_white().bold());
        
        // 1. 可选：使用LLM理解查询
        let enhanced_query = self.understand_query(query).await?;
        if enhanced_query != query {
            println!("{} {}", "🤖 增强查询:".bright_magenta(), enhanced_query.bright_white());
        }
        
        // 2. 执行混合搜索
        let search_results = self.hybrid_engine
            .search(&enhanced_query, top_k, None)
            .await?;
        
        // 3. 转换为商品搜索结果
        let mut results = Vec::new();
        for result in search_results.results {
            if let Some(product) = self.products.iter().find(|p| p.id == result.id) {
                results.push(ProductSearchResult {
                    product: product.clone(),
                    score: result.score,
                    match_type: "hybrid".to_string(),
                    snippet: Some(result.content),
                });
            }
        }
        
        // 4. 显示搜索统计
        let stats = &search_results.stats;
        println!("\n{}", "📊 搜索统计:".bright_yellow());
        println!("  • 总耗时: {:.2}ms", stats.total_time_ms);
        println!("  • 向量搜索: {:.2}ms", stats.vector_search_time_ms);
        println!("  • BM25搜索: {:.2}ms", stats.bm25_search_time_ms);
        println!("  • 融合时间: {:.2}ms", stats.fusion_time_ms);
        println!("  • 向量结果数: {}", stats.vector_results_count);
        println!("  • BM25结果数: {}", stats.bm25_results_count);
        println!("  • 融合结果数: {}", stats.fused_results_count);
        
        Ok(results)
    }
    
    /// 显示搜索结果
    fn display_results(&self, results: &[ProductSearchResult]) {
        println!("\n{}", "🎯 搜索结果:".bright_green().bold());
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
        
        if results.is_empty() {
            println!("{}", "  ❌ 未找到匹配的商品".bright_red());
            return;
        }
        
        for (idx, result) in results.iter().enumerate() {
            let product = &result.product;
            println!("\n{} {}", format!("{}.", idx + 1).bright_cyan().bold(), product.name.bright_white().bold());
            println!("  📱 品牌: {}", product.brand.bright_yellow());
            println!("  📂 分类: {}", product.category.bright_blue());
            println!("  💰 价格: ¥{:.2}", product.price);
            println!("  ⭐ 评分: {}/5.0 ({} 评价)", product.rating, product.reviews_count);
            println!("  🏷️  标签: {}", product.tags.join(", ").bright_magenta());
            println!("  📝 描述: {}", product.description.bright_white());
            println!("  🎯 匹配度: {:.2}%", result.score * 100.0);
            
            if let Some(snippet) = &result.snippet {
                if snippet.len() > 100 {
                    println!("  💬 匹配片段: {}...", &snippet[..100].bright_black());
                }
            }
        }
        println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,agent_mem=debug")
        .init();
    
    println!("\n{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                                                           ║".bright_cyan());
    println!("{}", "║     🛒 商品搜索演示 - AgentMem混合检索系统               ║".bright_cyan());
    println!("{}", "║     结合 agent-mem-llm + 混合检索                        ║".bright_cyan());
    println!("{}", "║                                                           ║".bright_cyan());
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan());
    
    // 检查是否启用LLM
    let use_llm = std::env::var("OPENAI_API_KEY").is_ok();
    if use_llm {
        println!("\n{}", "✅ 检测到 OPENAI_API_KEY，将使用LLM增强查询理解".bright_green());
    } else {
        println!("\n{}", "ℹ️  未设置 OPENAI_API_KEY，将使用基础搜索".bright_yellow());
    }
    
    // 创建搜索引擎
    let mut engine = ProductSearchEngine::new(use_llm)
        .await
        .context("创建搜索引擎失败")?;
    
    // 加载商品数据
    engine.load_products()
        .await
        .context("加载商品数据失败")?;
    
    println!("\n{}", "准备就绪！开始搜索演示...".bright_green().bold());
    
    // 演示场景
    let test_queries = vec![
        ("苹果手机", "精确品牌搜索"),
        ("拍照好的手机", "特性搜索"),
        ("专业笔记本电脑", "功能+类别搜索"),
        ("降噪耳机", "功能搜索"),
        ("5000元左右的手机", "价格范围搜索"),
        ("轻薄商务本", "多特征搜索"),
    ];
    
    for (query, description) in test_queries {
        println!("\n\n{} {}", "📋 测试场景:".bright_magenta().bold(), description.bright_white());
        
        match engine.search(query, 3).await {
            Ok(results) => engine.display_results(&results),
            Err(e) => {
                println!("{} {}", "❌ 搜索失败:".bright_red(), e);
            }
        }
        
        // 短暂延迟，便于观察
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    
    println!("\n\n{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    println!("{}", "✅ 商品搜索演示完成！".bright_green().bold());
    println!("{}", "═══════════════════════════════════════════════════════════".bright_cyan());
    
    println!("\n{}", "💡 核心功能展示:".bright_yellow().bold());
    println!("  1. ✅ 混合检索 (向量+BM25)");
    println!("  2. ✅ LLM查询理解 (可选)");
    println!("  3. ✅ 智能查询分类");
    println!("  4. ✅ 自适应阈值");
    println!("  5. ✅ 性能统计");
    
    Ok(())
}

