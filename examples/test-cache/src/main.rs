//! 测试缓存功能
//!
//! 这个示例程序测试 LRU 缓存的基本功能

use agent_mem_traits::{
    CacheStats, ExtractedFact, IntelligenceCache, MemoryActionType, MemoryDecision,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 简单的内存缓存实现（用于测试）
pub struct SimpleCache {
    facts: Arc<RwLock<HashMap<String, Vec<ExtractedFact>>>>,
    decisions: Arc<RwLock<HashMap<String, MemoryDecision>>>,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl Default for SimpleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleCache {
    pub fn new() -> Self {
        Self {
            facts: Arc::new(RwLock::new(HashMap::new())),
            decisions: Arc::new(RwLock::new(HashMap::new())),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait]
impl IntelligenceCache for SimpleCache {
    async fn get_facts(&self, key: &str) -> Option<Vec<ExtractedFact>> {
        let facts = self.facts.read().await;
        if let Some(f) = facts.get(key) {
            *self.hits.write().await += 1;
            Some(f.clone())
        } else {
            *self.misses.write().await += 1;
            None
        }
    }

    async fn set_facts(&self, key: &str, facts: Vec<ExtractedFact>) {
        self.facts.write().await.insert(key.to_string(), facts);
    }

    async fn get_decision(&self, key: &str) -> Option<MemoryDecision> {
        let decisions = self.decisions.read().await;
        if let Some(d) = decisions.get(key) {
            *self.hits.write().await += 1;
            Some(d.clone())
        } else {
            *self.misses.write().await += 1;
            None
        }
    }

    async fn set_decision(&self, key: &str, decision: MemoryDecision) {
        self.decisions
            .write()
            .await
            .insert(key.to_string(), decision);
    }

    async fn clear(&self) {
        self.facts.write().await.clear();
        self.decisions.write().await.clear();
        *self.hits.write().await = 0;
        *self.misses.write().await = 0;
    }

    async fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let facts_size = self.facts.read().await.len();
        let decisions_size = self.decisions.read().await.len();

        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            size: facts_size + decisions_size,
            hit_rate,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🧪 测试缓存功能\n");

    // 创建缓存
    let cache: Arc<dyn IntelligenceCache> = Arc::new(SimpleCache::new());

    // 测试 1: 事实缓存
    println!("📝 测试 1: 事实缓存");
    test_facts_cache(cache.clone()).await?;

    // 测试 2: 决策缓存
    println!("\n🤖 测试 2: 决策缓存");
    test_decision_cache(cache.clone()).await?;

    // 测试 3: 缓存统计
    println!("\n📊 测试 3: 缓存统计");
    test_cache_stats(cache.clone()).await?;

    // 测试 4: 缓存清空
    println!("\n🗑️  测试 4: 缓存清空");
    test_cache_clear(cache.clone()).await?;

    println!("\n✅ 所有测试通过！");
    Ok(())
}

/// 测试事实缓存
async fn test_facts_cache(cache: Arc<dyn IntelligenceCache>) -> anyhow::Result<()> {
    // 创建测试事实
    let facts = vec![
        ExtractedFact {
            content: "用户喜欢 Rust 编程".to_string(),
            confidence: 0.9,
            category: "preference".to_string(),
            metadata: HashMap::new(),
        },
        ExtractedFact {
            content: "用户住在北京".to_string(),
            confidence: 0.95,
            category: "location".to_string(),
            metadata: HashMap::new(),
        },
    ];

    // 测试缓存未命中
    let result = cache.get_facts("test_key_1").await;
    assert!(result.is_none(), "首次查询应该未命中");
    println!("  ✓ 缓存未命中测试通过");

    // 设置缓存
    cache.set_facts("test_key_1", facts.clone()).await;
    println!("  ✓ 设置缓存成功");

    // 测试缓存命中
    let result = cache.get_facts("test_key_1").await;
    assert!(result.is_some(), "第二次查询应该命中");
    let cached_facts = result.unwrap();
    assert_eq!(cached_facts.len(), 2, "缓存的事实数量应该是 2");
    println!("  ✓ 缓存命中测试通过");
    println!("  ✓ 提取到 {} 个事实", cached_facts.len());

    Ok(())
}

/// 测试决策缓存
async fn test_decision_cache(cache: Arc<dyn IntelligenceCache>) -> anyhow::Result<()> {
    // 创建测试决策
    let decision = MemoryDecision {
        action: MemoryActionType::Add {
            content: "新记忆内容".to_string(),
            importance: 0.8,
            metadata: HashMap::new(),
        },
        confidence: 0.9,
        reasoning: "这是一个新的重要信息".to_string(),
    };

    // 测试缓存未命中
    let result = cache.get_decision("decision_key_1").await;
    assert!(result.is_none(), "首次查询应该未命中");
    println!("  ✓ 缓存未命中测试通过");

    // 设置缓存
    cache.set_decision("decision_key_1", decision.clone()).await;
    println!("  ✓ 设置缓存成功");

    // 测试缓存命中
    let result = cache.get_decision("decision_key_1").await;
    assert!(result.is_some(), "第二次查询应该命中");
    let cached_decision = result.unwrap();
    assert!(cached_decision.confidence > 0.0, "决策置信度应该大于 0");
    println!("  ✓ 缓存命中测试通过");
    println!("  ✓ 决策置信度: {:.2}", cached_decision.confidence);

    Ok(())
}

/// 测试缓存统计
async fn test_cache_stats(cache: Arc<dyn IntelligenceCache>) -> anyhow::Result<()> {
    let stats = cache.stats().await;

    println!("  缓存统计:");
    println!("    命中次数: {}", stats.hits);
    println!("    未命中次数: {}", stats.misses);
    println!("    缓存大小: {}", stats.size);
    println!("    命中率: {:.2}%", stats.hit_rate * 100.0);

    assert!(stats.hits > 0, "应该有缓存命中");
    assert!(stats.misses > 0, "应该有缓存未命中");
    assert!(stats.hit_rate > 0.0, "命中率应该大于 0");
    assert!(stats.hit_rate < 1.0, "命中率应该小于 1");

    println!("  ✓ 统计信息正确");

    Ok(())
}

/// 测试缓存清空
async fn test_cache_clear(cache: Arc<dyn IntelligenceCache>) -> anyhow::Result<()> {
    // 清空缓存
    cache.clear().await;
    println!("  ✓ 缓存已清空");

    // 验证缓存已清空
    let result = cache.get_facts("test_key_1").await;
    assert!(result.is_none(), "清空后查询应该未命中");
    println!("  ✓ 验证缓存已清空");

    // 验证统计信息已重置
    let stats = cache.stats().await;
    assert_eq!(stats.hits, 0, "命中次数应该重置为 0");
    assert_eq!(stats.size, 0, "缓存大小应该为 0");
    println!("  ✓ 统计信息已重置");

    Ok(())
}
