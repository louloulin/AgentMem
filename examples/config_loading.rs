//! Configuration Loading Example
//! Week 3-4: Demonstrate how to eliminate hardcoding using config files

use agent_mem_core::config::AgentMemConfig;

fn main() -> anyhow::Result<()> {
    println!("=== AgentMem V4.0 Configuration Loading Example ===\n");
    
    // 方式1: 使用默认配置（所有默认值）
    println!("1. Default Configuration:");
    let default_config = AgentMemConfig::default();
    println!("   Vector weight: {}", default_config.hybrid_search.vector_weight);
    println!("   Fulltext weight: {}", default_config.hybrid_search.fulltext_weight);
    println!("   RRF k: {}", default_config.hybrid_search.rrf_k);
    println!("   Recency weight: {}", default_config.importance_scorer.recency_weight);
    
    // 方式2: 从文件加载配置
    println!("\n2. Loading from file:");
    let config_path = "config/agentmem.example.toml";
    match AgentMemConfig::from_file(config_path) {
        Ok(config) => {
            println!("   ✓ Loaded config from {}", config_path);
            println!("   Vector weight: {}", config.hybrid_search.vector_weight);
            println!("   Max memories: {}", config.memory_integration.max_memories);
        }
        Err(e) => {
            println!("   ✗ Failed to load: {} (this is expected if file doesn't exist)", e);
            println!("   💡 Use default config as fallback");
        }
    }
    
    // 方式3: 从TOML字符串加载
    println!("\n3. Loading from TOML string:");
    let toml_str = r#"
[hybrid_search]
vector_weight = 0.8
fulltext_weight = 0.2
rrf_k = 50.0

[importance_scorer]
recency_weight = 0.30
frequency_weight = 0.25
relevance_weight = 0.20
emotional_weight = 0.15
context_weight = 0.05
interaction_weight = 0.05
"#;
    let config = AgentMemConfig::from_toml_str(toml_str)?;
    println!("   ✓ Parsed custom config");
    println!("   Vector weight: {}", config.hybrid_search.vector_weight);
    println!("   Recency weight: {}", config.importance_scorer.recency_weight);
    
    // 方式4: 环境变量覆盖
    println!("\n4. Environment variable overrides:");
    std::env::set_var("AGENTMEM_VECTOR_WEIGHT", "0.75");
    std::env::set_var("AGENTMEM_FULLTEXT_WEIGHT", "0.25");
    
    let mut config = AgentMemConfig::default();
    config.apply_env_overrides();
    println!("   ✓ Applied env overrides");
    println!("   Vector weight (from env): {}", config.hybrid_search.vector_weight);
    println!("   Fulltext weight (from env): {}", config.hybrid_search.fulltext_weight);
    
    std::env::remove_var("AGENTMEM_VECTOR_WEIGHT");
    std::env::remove_var("AGENTMEM_FULLTEXT_WEIGHT");
    
    // 方式5: 配置验证
    println!("\n5. Configuration validation:");
    let valid_config = AgentMemConfig::default();
    match valid_config.validate() {
        Ok(_) => println!("   ✓ Default config is valid"),
        Err(e) => println!("   ✗ Validation failed: {}", e),
    }
    
    // 无效配置示例
    let mut invalid_config = AgentMemConfig::default();
    invalid_config.hybrid_search.vector_weight = 0.9;  // 总和 > 1.0
    match invalid_config.validate() {
        Ok(_) => println!("   ✗ Should have failed validation"),
        Err(e) => println!("   ✓ Caught invalid config: {}", e),
    }
    
    // 方式6: 生成配置文件模板
    println!("\n6. Generating config template:");
    let template_path = "/tmp/agentmem_generated.toml";
    match default_config.save_to_file(template_path) {
        Ok(_) => println!("   ✓ Saved template to {}", template_path),
        Err(e) => println!("   ✗ Failed to save: {}", e),
    }
    
    println!("\n=== Summary ===");
    println!("✅ Configuration can be loaded from:");
    println!("   - Default values (hardcoded → configurable)");
    println!("   - TOML files");
    println!("   - Environment variables");
    println!("   - Inline TOML strings");
    println!("\n✅ All configurations are validated before use");
    println!("✅ Zero hardcoding in production code!");
    
    Ok(())
}

