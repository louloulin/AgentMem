# Configuration Migration Guide (Week 3-4)

## Overview

AgentMem V4.0 eliminates all hardcoded values by moving them to configurable parameters. This document shows how to migrate from hardcoded values to configuration-driven code.

## Migration Examples

### Before (❌ Hardcoded)

```rust
// search/hybrid.rs
impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,      // ❌ Hardcoded
            fulltext_weight: 0.3,    // ❌ Hardcoded
            rrf_k: 60.0,             // ❌ Hardcoded
            enable_parallel: true,
            enable_cache: false,
        }
    }
}
```

### After (✅ Configurable)

```rust
// Load from config file
let config = AgentMemConfig::from_file("config/agentmem.toml")?;

// Use loaded values
let hybrid_config = config.hybrid_search;
assert_eq!(hybrid_config.vector_weight, 0.7);  // From config
assert_eq!(hybrid_config.fulltext_weight, 0.3); // From config
```

## Configuration File Structure

```toml
# config/agentmem.toml

[hybrid_search]
vector_weight = 0.7
fulltext_weight = 0.3
rrf_k = 60.0
enable_parallel = true
enable_cache = false

[importance_scorer]
recency_weight = 0.25
frequency_weight = 0.20
relevance_weight = 0.25
emotional_weight = 0.15
context_weight = 0.10
interaction_weight = 0.05

[memory_integration]
max_memories = 10
relevance_threshold = 0.1
include_timestamp = true
sort_by_importance = true
episodic_weight = 1.2
working_weight = 1.0
semantic_weight = 0.9
```

## Usage Patterns

### Pattern 1: Default + File Override

```rust
// Start with defaults
let mut config = AgentMemConfig::default();

// Override from file if exists
if let Ok(file_config) = AgentMemConfig::from_file("agentmem.toml") {
    config = file_config;
}

// Apply environment overrides
config.apply_env_overrides();

// Validate before use
config.validate()?;
```

### Pattern 2: Environment Variable Override

```bash
# Set environment variables
export AGENTMEM_VECTOR_WEIGHT=0.8
export AGENTMEM_FULLTEXT_WEIGHT=0.2
export AGENTMEM_RECENCY_WEIGHT=0.3
```

```rust
let mut config = AgentMemConfig::default();
config.apply_env_overrides();
```

### Pattern 3: Production vs Development

```toml
# config/development.toml
[hybrid_search]
vector_weight = 0.7
enable_cache = false  # Disable in dev

# config/production.toml
[hybrid_search]
vector_weight = 0.7
enable_cache = true   # Enable in prod
```

```rust
let env = std::env::var("ENVIRONMENT").unwrap_or("development".to_string());
let config_file = format!("config/{}.toml", env);
let config = AgentMemConfig::from_file(config_file)?;
```

## Eliminated Hardcodings

### Search Module
- ✅ `vector_weight: 0.7` → `config.hybrid_search.vector_weight`
- ✅ `fulltext_weight: 0.3` → `config.hybrid_search.fulltext_weight`
- ✅ `rrf_k: 60.0` → `config.hybrid_search.rrf_k`

### Importance Scorer
- ✅ `recency_weight: 0.25` → `config.importance_scorer.recency_weight`
- ✅ `frequency_weight: 0.20` → `config.importance_scorer.frequency_weight`
- ✅ `relevance_weight: 0.25` → `config.importance_scorer.relevance_weight`
- ✅ `emotional_weight: 0.15` → `config.importance_scorer.emotional_weight`
- ✅ `context_weight: 0.10` → `config.importance_scorer.context_weight`
- ✅ `interaction_weight: 0.05` → `config.importance_scorer.interaction_weight`

### Memory Integration
- ✅ `max_memories: 10` → `config.memory_integration.max_memories`
- ✅ `relevance_threshold: 0.1` → `config.memory_integration.relevance_threshold`
- ✅ `episodic_weight: 1.2` → `config.memory_integration.episodic_weight`
- ✅ `working_weight: 1.0` → `config.memory_integration.working_weight`
- ✅ `semantic_weight: 0.9` → `config.memory_integration.semantic_weight`

### Orchestrator
- ✅ `max_tool_rounds: 5` → `config.orchestrator.max_tool_rounds`
- ✅ `tool_timeout_seconds: 30` → `config.orchestrator.tool_timeout_seconds`

### Compression
- ✅ `min_importance_threshold: 0.3` → `config.compression.min_importance_threshold`
- ✅ `target_compression_ratio: 0.7` → `config.compression.target_compression_ratio`
- ✅ `semantic_similarity_threshold: 0.85` → `config.compression.semantic_similarity_threshold`

### Adaptive Threshold
- ✅ `base_threshold: 0.1-0.5` → `config.adaptive_threshold.base_thresholds`
- ✅ `length_factor: 0.3` → `config.adaptive_threshold.length_factor`
- ✅ `complexity_factor: 0.2` → `config.adaptive_threshold.complexity_factor`

## Benefits

1. **Zero Hardcoding**: All magic numbers moved to configuration
2. **Environment-Specific**: Different configs for dev/staging/prod
3. **Runtime Adjustable**: Change without recompiling
4. **Validated**: All configs validated before use
5. **Documented**: Clear documentation of all parameters

## Testing

```rust
#[test]
fn test_config_loading() {
    let config = AgentMemConfig::default();
    assert!(config.validate().is_ok());
    
    // Test weight sums
    let total = config.hybrid_search.vector_weight 
              + config.hybrid_search.fulltext_weight;
    assert!((total - 1.0).abs() < 0.01);
}
```

## See Also

- `examples/config_loading.rs` - Complete working example
- `config/agentmem.example.toml` - Full configuration template
- `src/config.rs` - Configuration loader implementation

