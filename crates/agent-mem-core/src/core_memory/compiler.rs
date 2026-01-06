//! Core Memory Compiler - 将多个 Block 编译成提示词
//!
//! 类似 MIRIX 的 Memory.compile() 功能

use crate::core_memory::{BlockType, TemplateContext, TemplateEngine};
use crate::storage::models::Block;
use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};

/// 编译器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// 默认模板
    pub default_template: String,
    /// 是否包含元数据
    pub include_metadata: bool,
    /// 是否包含统计信息
    pub include_stats: bool,
    /// 分隔符
    pub separator: String,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            default_template: DEFAULT_CORE_MEMORY_TEMPLATE.to_string(),
            include_metadata: false,
            include_stats: false,
            separator: "\n\n".to_string(),
        }
    }
}

/// 默认的 Core Memory 模板
const DEFAULT_CORE_MEMORY_TEMPLATE: &str = r#"# Core Memory

{% if persona_blocks %}
## Persona
{% for block in persona_blocks %}
{{block}}
{% endfor %}
{% endif %}

{% if human_blocks %}
## Human
{% for block in human_blocks %}
{{block}}
{% endfor %}
{% endif %}

{% if system_blocks %}
## System
{% for block in system_blocks %}
{{block}}
{% endfor %}
{% endif %}
"#;

/// 编译结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    /// 编译后的提示词
    pub prompt: String,
    /// 使用的 Block 数量
    pub blocks_used: usize,
    /// 总字符数
    pub total_characters: usize,
    /// 编译时间（毫秒）
    pub compilation_time_ms: u64,
}

/// Core Memory 编译器
pub struct CoreMemoryCompiler {
    config: CompilerConfig,
    template_engine: TemplateEngine,
}

impl CoreMemoryCompiler {
    /// 创建新的编译器
    pub fn new(config: CompilerConfig) -> Self {
        Self {
            config,
            template_engine: TemplateEngine::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(CompilerConfig::default())
    }

    /// 编译 Blocks 为提示词
    pub fn compile(&self, blocks: &[Block]) -> Result<CompilationResult> {
        let start_time = std::time::Instant::now();

        // 按类型分组
        let mut persona_blocks = Vec::new();
        let mut human_blocks = Vec::new();
        let mut system_blocks = Vec::new();

        for block in blocks {
            match block.label.as_str() {
                "persona" => persona_blocks.push(block),
                "human" => human_blocks.push(block),
                "system" => system_blocks.push(block),
                _ => {}
            }
        }

        // 构建模板上下文
        let mut context = TemplateContext::new();

        // 添加 persona blocks
        if !persona_blocks.is_empty() {
            let persona_values: Vec<String> = persona_blocks
                .iter()
                .map(|b| self.format_block(b))
                .collect();
            context.set_list("persona_blocks".to_string(), persona_values);
        }

        // 添加 human blocks
        if !human_blocks.is_empty() {
            let human_values: Vec<String> =
                human_blocks.iter().map(|b| self.format_block(b)).collect();
            context.set_list("human_blocks".to_string(), human_values);
        }

        // 添加 system blocks
        if !system_blocks.is_empty() {
            let system_values: Vec<String> =
                system_blocks.iter().map(|b| self.format_block(b)).collect();
            context.set_list("system_blocks".to_string(), system_values);
        }

        // 渲染模板
        let prompt = self
            .template_engine
            .render(&self.config.default_template, &context)?;

        let compilation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            total_characters: prompt.len(),
            blocks_used: blocks.len(),
            prompt,
            compilation_time_ms,
        })
    }

    /// 使用自定义模板编译
    pub fn compile_with_template(
        &self,
        blocks: &[Block],
        template: &str,
    ) -> Result<CompilationResult> {
        let start_time = std::time::Instant::now();

        // 构建上下文
        let mut context = TemplateContext::new();

        // 添加所有 blocks
        let all_values: Vec<String> = blocks.iter().map(|b| self.format_block(b)).collect();
        context.set_list("blocks".to_string(), all_values);

        // 按类型分组
        let persona_blocks: Vec<String> = blocks
            .iter()
            .filter(|b| b.label == "persona")
            .map(|b| self.format_block(b))
            .collect();
        let human_blocks: Vec<String> = blocks
            .iter()
            .filter(|b| b.label == "human")
            .map(|b| self.format_block(b))
            .collect();
        let system_blocks: Vec<String> = blocks
            .iter()
            .filter(|b| b.label == "system")
            .map(|b| self.format_block(b))
            .collect();

        if !persona_blocks.is_empty() {
            context.set_list("persona_blocks".to_string(), persona_blocks);
        }
        if !human_blocks.is_empty() {
            context.set_list("human_blocks".to_string(), human_blocks);
        }
        if !system_blocks.is_empty() {
            context.set_list("system_blocks".to_string(), system_blocks);
        }

        // 渲染模板
        let prompt = self.template_engine.render(template, &context)?;

        let compilation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            total_characters: prompt.len(),
            blocks_used: blocks.len(),
            prompt,
            compilation_time_ms,
        })
    }

    /// 编译为简单字符串（不使用模板）
    pub fn compile_simple(&self, blocks: &[Block]) -> Result<String> {
        let mut result = String::new();

        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                result.push_str(&self.config.separator);
            }

            result.push_str(&self.format_block(block));
        }

        Ok(result)
    }

    /// 格式化单个 Block
    fn format_block(&self, block: &Block) -> String {
        let mut formatted = String::new();

        // 添加描述（如果有）
        if let Some(ref description) = block.description {
            if !description.is_empty() {
                formatted.push_str(&format!("[{}]\n", description));
            }
        }

        // 添加内容
        formatted.push_str(&block.value);

        // 添加元数据（如果启用）
        if self.config.include_metadata {
            if let Some(ref metadata) = block.metadata_ {
                formatted.push_str(&format!("\n[Metadata: {}]", metadata));
            }
        }

        formatted
    }

    /// 验证编译结果
    pub fn validate_compilation(
        &self,
        result: &CompilationResult,
        max_length: Option<usize>,
    ) -> Result<()> {
        // 检查是否为空
        if result.prompt.is_empty() {
            return Err(AgentMemError::validation_error("Compiled prompt is empty"));
        }

        // 检查长度限制
        if let Some(max_len) = max_length {
            if result.total_characters > max_len {
                return Err(AgentMemError::validation_error(format!(
                    "Compiled prompt exceeds max length: {} > {}",
                    result.total_characters, max_len
                )));
            }
        }

        Ok(())
    }

    /// 获取编译统计信息
    pub fn get_compilation_stats(&self, blocks: &[Block]) -> CompilationStats {
        let mut stats = CompilationStats::default();

        stats.total_blocks = blocks.len();

        for block in blocks {
            stats.total_characters += block.value.len();

            match block.label.as_str() {
                "persona" => stats.persona_blocks += 1,
                "human" => stats.human_blocks += 1,
                "system" => stats.system_blocks += 1,
                _ => {}
            }
        }

        if !blocks.is_empty() {
            stats.average_block_size = stats.total_characters / blocks.len();
        }

        stats
    }
}

/// 编译统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompilationStats {
    pub total_blocks: usize,
    pub persona_blocks: usize,
    pub human_blocks: usize,
    pub system_blocks: usize,
    pub total_characters: usize,
    pub average_block_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::Block;

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
    fn test_compile_simple() {
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
    fn test_compile_with_template() {
        let compiler = CoreMemoryCompiler::with_default_config();

        let blocks = vec![
            create_test_block("persona", "I am a helpful assistant"),
            create_test_block("human", "User prefers concise answers"),
        ];

        let result = compiler.compile(&blocks).unwrap();

        assert!(result.blocks_used == 2);
        assert!(result.total_characters > 0);
        assert!(result.prompt.contains("Persona"));
        assert!(result.prompt.contains("Human"));
    }

    #[test]
    fn test_compilation_stats() {
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
    }
}
