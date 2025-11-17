#![cfg(feature = "postgres")]

//! Core Memory 系统集成测试

use agent_mem_core::core_memory::{
    AutoRewriter, AutoRewriterConfig, BlockManager, BlockManagerConfig, BlockType, CompilerConfig,
    CoreMemoryCompiler, RewriteStrategy, TemplateContext, TemplateEngine,
};
use agent_mem_core::storage::models::Block;

// ============================================================================
// Template Engine Tests
// ============================================================================

#[test]
fn test_template_engine_variable_substitution() {
    let engine = TemplateEngine::new();
    let mut context = TemplateContext::new();
    context.set("name".to_string(), "Alice".to_string());
    context.set("role".to_string(), "Assistant".to_string());

    let template = "Hello {{name}}, you are a {{role}}.";
    let result = engine.render(template, &context).unwrap();

    assert_eq!(result, "Hello Alice, you are a Assistant.");
}

#[test]
fn test_template_engine_filters() {
    let engine = TemplateEngine::new();
    let mut context = TemplateContext::new();
    context.set("name".to_string(), "alice".to_string());

    let template = "Hello {{name|upper}}!";
    let result = engine.render(template, &context).unwrap();

    assert_eq!(result, "Hello ALICE!");
}

#[test]
fn test_template_engine_conditionals() {
    let engine = TemplateEngine::new();
    let mut context = TemplateContext::new();
    context.set("show_greeting".to_string(), "yes".to_string());

    let template = "{% if show_greeting %}Hello World!{% endif %}";
    let result = engine.render(template, &context).unwrap();

    assert_eq!(result, "Hello World!");
}

#[test]
fn test_template_engine_loops() {
    let engine = TemplateEngine::new();
    let mut context = TemplateContext::new();
    context.set_list(
        "items".to_string(),
        vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ],
    );

    let template = "{% for item in items %}{{item}}, {% endfor %}";
    let result = engine.render(template, &context).unwrap();

    assert_eq!(result, "apple, banana, cherry, ");
}

#[test]
fn test_template_engine_strict_mode() {
    let engine = TemplateEngine::new().with_strict_mode(true);
    let context = TemplateContext::new();

    let template = "Hello {{undefined_var}}!";
    let result = engine.render(template, &context);

    assert!(result.is_err());
}

// ============================================================================
// Auto Rewriter Tests
// ============================================================================

#[tokio::test]
async fn test_auto_rewriter_preserve_important() {
    let rewriter = AutoRewriter::with_default_config();
    let content = "Short line\nThis is a much longer line with more information\nMedium line here";
    let target_length = 50;

    let result = rewriter
        .rewrite(content, target_length, None)
        .await
        .unwrap();

    assert!(result.len() <= target_length);
    assert!(result.contains("much longer line"));
}

#[tokio::test]
async fn test_auto_rewriter_preserve_recent() {
    let mut config = AutoRewriterConfig::default();
    config.strategy = RewriteStrategy::PreserveRecent;

    let rewriter = AutoRewriter::new(config);
    let content = "Old information at the start. Recent information at the end.";
    let target_length = 30;

    let result = rewriter
        .rewrite(content, target_length, None)
        .await
        .unwrap();

    assert!(result.len() <= target_length);
    assert!(result.contains("end"));
}

#[tokio::test]
async fn test_auto_rewriter_validation() {
    let rewriter = AutoRewriter::with_default_config();
    let original = "This is the original content";
    let rewritten = "This is rewritten";
    let target_length = 50;

    let result = rewriter.validate_rewrite(original, rewritten, target_length);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auto_rewriter_quality_score() {
    let rewriter = AutoRewriter::with_default_config();
    let original = "The quick brown fox jumps over the lazy dog";
    let rewritten = "The quick brown fox jumps";

    let score = rewriter.calculate_quality_score(original, rewritten);
    assert!(score > 0.0 && score <= 1.0);
}

// ============================================================================
// Core Memory Compiler Tests
// ============================================================================

fn create_test_block(label: &str, value: &str) -> Block {
    Block::new(
        "org-123".to_string(),
        "user-456".to_string(),
        label.to_string(),
        value.to_string(),
        2000,
    )
}

#[test]
fn test_compiler_simple() {
    let compiler = CoreMemoryCompiler::with_default_config();

    let blocks = vec![
        create_test_block("persona", "I am a helpful assistant"),
        create_test_block("human", "User prefers concise answers"),
    ];

    let result = compiler.compile_simple(&blocks).unwrap();

    assert!(result.contains("helpful assistant"));
    assert!(result.contains("concise answers"));
}

#[test]
fn test_compiler_with_template() {
    let compiler = CoreMemoryCompiler::with_default_config();

    let blocks = vec![
        create_test_block("persona", "I am a helpful assistant"),
        create_test_block("human", "User prefers concise answers"),
    ];

    let result = compiler.compile(&blocks).unwrap();

    println!("Default template result:\n{}", result.prompt);
    assert_eq!(result.blocks_used, 2);
    assert!(result.total_characters > 0);
    assert!(result.prompt.contains("Persona"));
    assert!(result.prompt.contains("Human"));
}

#[test]
fn test_compiler_custom_template() {
    let compiler = CoreMemoryCompiler::with_default_config();

    let blocks = vec![
        create_test_block("persona", "I am a helpful assistant"),
        create_test_block("human", "User prefers concise answers"),
    ];

    let custom_template = "{% for block in blocks %}{{block}}\n{% endfor %}";
    let result = compiler
        .compile_with_template(&blocks, custom_template)
        .unwrap();

    println!("Generated prompt:\n{}", result.prompt);
    assert_eq!(result.blocks_used, 2);
    assert!(result.prompt.contains("helpful assistant"));
}

#[test]
fn test_compiler_stats() {
    let compiler = CoreMemoryCompiler::with_default_config();

    let blocks = vec![
        create_test_block("persona", "Test persona"),
        create_test_block("human", "Test human"),
        create_test_block("system", "Test system"),
    ];

    let stats = compiler.get_compilation_stats(&blocks);

    assert_eq!(stats.total_blocks, 3);
    assert_eq!(stats.persona_blocks, 1);
    assert_eq!(stats.human_blocks, 1);
    assert_eq!(stats.system_blocks, 1);
    assert!(stats.total_characters > 0);
}

#[test]
fn test_compiler_validation() {
    let compiler = CoreMemoryCompiler::with_default_config();

    let blocks = vec![create_test_block("persona", "Test")];

    let result = compiler.compile(&blocks).unwrap();

    // Should pass validation
    assert!(compiler.validate_compilation(&result, Some(10000)).is_ok());

    // Should fail validation (too long)
    assert!(compiler.validate_compilation(&result, Some(10)).is_err());
}

// ============================================================================
// Block Type Tests
// ============================================================================

#[test]
fn test_block_type_conversion() {
    assert_eq!(BlockType::Persona.as_str(), "persona");
    assert_eq!(BlockType::Human.as_str(), "human");
    assert_eq!(BlockType::System.as_str(), "system");

    assert_eq!(BlockType::from_str("persona"), Some(BlockType::Persona));
    assert_eq!(BlockType::from_str("HUMAN"), Some(BlockType::Human));
    assert_eq!(BlockType::from_str("System"), Some(BlockType::System));
    assert_eq!(BlockType::from_str("invalid"), None);
}

// ============================================================================
// Integration Tests (require database)
// ============================================================================

#[test]
fn test_block_manager_config() {
    let config = BlockManagerConfig::default();

    assert_eq!(config.persona_default_limit, 2000);
    assert_eq!(config.human_default_limit, 2000);
    assert_eq!(config.system_default_limit, 1000);
    assert_eq!(config.auto_rewrite_threshold, 0.9);
    assert!(config.enable_auto_rewrite);
}

#[test]
fn test_compiler_config() {
    let config = CompilerConfig::default();

    assert!(!config.include_metadata);
    assert!(!config.include_stats);
    assert_eq!(config.separator, "\n\n");
}

#[test]
fn test_auto_rewriter_config() {
    let config = AutoRewriterConfig::default();

    assert_eq!(config.strategy, RewriteStrategy::PreserveImportant);
    assert_eq!(config.retention_ratio, 0.8);
    assert_eq!(config.buffer_ratio, 0.1);
    assert_eq!(config.llm_model, "gpt-4");
    assert_eq!(config.llm_temperature, 0.3);
    assert_eq!(config.max_retries, 3);
}

// ============================================================================
// End-to-End Workflow Test
// ============================================================================

#[test]
fn test_end_to_end_workflow() {
    // 1. Create blocks
    let blocks = vec![
        create_test_block("persona", "I am a helpful AI assistant named Claude."),
        create_test_block(
            "human",
            "User is a software developer who prefers concise answers.",
        ),
        create_test_block("system", "Always be respectful and professional."),
    ];

    // 2. Compile to prompt
    let compiler = CoreMemoryCompiler::with_default_config();
    let result = compiler.compile(&blocks).unwrap();

    // 3. Validate
    assert!(compiler.validate_compilation(&result, Some(10000)).is_ok());
    assert_eq!(result.blocks_used, 3);
    assert!(result.prompt.contains("Claude"));
    assert!(result.prompt.contains("software developer"));
    assert!(result.prompt.contains("respectful"));

    // 4. Get stats
    let stats = compiler.get_compilation_stats(&blocks);
    assert_eq!(stats.total_blocks, 3);
    assert_eq!(stats.persona_blocks, 1);
    assert_eq!(stats.human_blocks, 1);
    assert_eq!(stats.system_blocks, 1);
}
