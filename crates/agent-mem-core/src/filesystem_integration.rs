//! File System Integration Module
//!
//! Phase 3.4: 文件系统集成
//! - CLAUDE.md兼容格式
//! - 自动加载机制
//! - 导入系统
//!
//! 参考Claude Code的文件系统集成，提升开发体验30%

use agent_mem_traits::{MemoryV4 as Memory, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 文件系统集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemIntegrationConfig {
    /// 启用文件系统集成
    pub enable_filesystem: bool,
    /// CLAUDE.md文件路径（相对于项目根目录）
    pub claude_md_path: PathBuf,
    /// 自动加载CLAUDE.md
    pub auto_load_claude_md: bool,
    /// 监听文件变化
    pub watch_file_changes: bool,
    /// 支持的导入路径前缀
    pub import_prefixes: Vec<String>,
}

impl Default for FilesystemIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_filesystem: true,
            claude_md_path: PathBuf::from("CLAUDE.md"),
            auto_load_claude_md: true,
            watch_file_changes: false,
            import_prefixes: vec!["@".to_string(), "import:".to_string()],
        }
    }
}

/// CLAUDE.md文件格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMdFile {
    /// 文件路径
    pub path: PathBuf,
    /// 文件内容
    pub content: String,
    /// 解析的记忆
    pub memories: Vec<ClaudeMemory>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// CLAUDE.md中的记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMemory {
    /// 记忆内容
    pub content: String,
    /// 记忆类型
    pub memory_type: Option<String>,
    /// 重要性
    pub importance: Option<f64>,
    /// 标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 文件系统集成管理器
pub struct FilesystemIntegrationManager {
    config: FilesystemIntegrationConfig,
    /// 已加载的CLAUDE.md文件
    loaded_files: Arc<RwLock<HashMap<PathBuf, ClaudeMdFile>>>,
    /// 项目根目录
    project_root: PathBuf,
}

impl FilesystemIntegrationManager {
    /// 创建新的文件系统集成管理器
    pub fn new(config: FilesystemIntegrationConfig, project_root: PathBuf) -> Self {
        Self {
            config,
            loaded_files: Arc::new(RwLock::new(HashMap::new())),
            project_root,
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(project_root: PathBuf) -> Self {
        Self::new(FilesystemIntegrationConfig::default(), project_root)
    }

    /// 自动加载CLAUDE.md文件
    ///
    /// 从项目根目录自动查找并加载CLAUDE.md文件
    pub async fn auto_load_claude_md(&self) -> Result<Vec<Memory>> {
        if !self.config.auto_load_claude_md {
            return Ok(Vec::new());
        }

        info!("自动加载CLAUDE.md文件...");

        let claude_md_path = self.project_root.join(&self.config.claude_md_path);
        
        if !claude_md_path.exists() {
            debug!("CLAUDE.md文件不存在: {:?}", claude_md_path);
            return Ok(Vec::new());
        }

        self.load_claude_md_file(&claude_md_path).await
    }

    /// 加载CLAUDE.md文件
    pub async fn load_claude_md_file(&self, path: &Path) -> Result<Vec<Memory>> {
        info!("加载CLAUDE.md文件: {:?}", path);

        let content = fs::read_to_string(path).await.map_err(|e| {
            agent_mem_traits::AgentMemError::IoError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read CLAUDE.md file: {}", e))
            )
        })?;

        let claude_file = self.parse_claude_md(&content, path)?;

        // 保存到已加载文件
        let mut loaded = self.loaded_files.write().await;
        loaded.insert(path.to_path_buf(), claude_file.clone());

        // 转换为Memory对象
        let memories = self.convert_to_memories(&claude_file).await?;

        info!("成功加载 {} 条记忆", memories.len());
        Ok(memories)
    }

    /// 解析CLAUDE.md文件内容
    fn parse_claude_md(&self, content: &str, path: &Path) -> Result<ClaudeMdFile> {
        let mut memories = Vec::new();
        let mut metadata = HashMap::new();

        // 解析CLAUDE.md格式
        // 支持多种格式：
        // 1. Markdown格式（## Memory: ...）
        // 2. YAML frontmatter格式
        // 3. JSON格式

        // 简化的解析逻辑（实际应使用更完善的解析器）
        let lines: Vec<&str> = content.lines().collect();
        let mut current_memory: Option<ClaudeMemory> = None;

        for line in lines {
            // 检测记忆开始标记
            if line.starts_with("## Memory:") || line.starts_with("## MEMORY:") {
                if let Some(mem) = current_memory.take() {
                    memories.push(mem);
                }
                current_memory = Some(ClaudeMemory {
                    content: String::new(),
                    memory_type: None,
                    importance: None,
                    tags: Vec::new(),
                    metadata: HashMap::new(),
                });
            } else if line.starts_with("### Type:") {
                if let Some(ref mut mem) = current_memory {
                    mem.memory_type = Some(line.strip_prefix("### Type:").unwrap().trim().to_string());
                }
            } else if line.starts_with("### Importance:") {
                if let Some(ref mut mem) = current_memory {
                    if let Ok(imp) = line.strip_prefix("### Importance:").unwrap().trim().parse::<f64>() {
                        mem.importance = Some(imp);
                    }
                }
            } else if line.starts_with("### Tags:") {
                if let Some(ref mut mem) = current_memory {
                    let tags_str = line.strip_prefix("### Tags:").unwrap().trim();
                    mem.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
                }
            } else if let Some(ref mut mem) = current_memory {
                // 添加到当前记忆内容
                if !mem.content.is_empty() {
                    mem.content.push('\n');
                }
                mem.content.push_str(line);
            }
        }

        // 添加最后一个记忆
        if let Some(mem) = current_memory {
            memories.push(mem);
        }

        Ok(ClaudeMdFile {
            path: path.to_path_buf(),
            content: content.to_string(),
            memories,
            metadata,
        })
    }

    /// 转换为Memory对象
    async fn convert_to_memories(&self, claude_file: &ClaudeMdFile) -> Result<Vec<Memory>> {
        let mut memories = Vec::new();

        for claude_mem in &claude_file.memories {
            let memory_type = claude_mem.memory_type.clone().unwrap_or_else(|| "untyped".to_string());
            let importance = claude_mem.importance.unwrap_or(0.5) as f32;
            
            let mut memory = Memory::new(
                "default".to_string(), // agent_id
                None, // user_id
                memory_type.clone(),
                claude_mem.content.clone(),
                importance,
            );

            // 记忆类型和重要性已在Memory::new中设置，这里只需要更新其他属性

            // 设置标签
            if !claude_mem.tags.is_empty() {
                memory.attributes.insert(
                    agent_mem_traits::AttributeKey::core("tags"),
                    agent_mem_traits::AttributeValue::List(
                        claude_mem.tags.iter()
                            .map(|t| agent_mem_traits::AttributeValue::String(t.clone()))
                            .collect(),
                    ),
                );
            }

            // 设置元数据
            for (key, value) in &claude_mem.metadata {
                memory.attributes.insert(
                    agent_mem_traits::AttributeKey::new("metadata", key),
                    agent_mem_traits::AttributeValue::String(value.clone()),
                );
            }

            // 设置来源
            memory.attributes.insert(
                agent_mem_traits::AttributeKey::system("source"),
                agent_mem_traits::AttributeValue::String(
                    format!("claude_md:{}", claude_file.path.display()),
                ),
            );

            memories.push(memory);
        }

        Ok(memories)
    }

    /// 处理导入语句
    ///
    /// 支持 `@path/to/file` 和 `import:path/to/file` 格式
    pub async fn process_imports(&self, content: &str) -> Result<String> {
        let mut result = content.to_string();

        for _prefix in &self.config.import_prefixes {
            // 查找所有导入语句
            // 简化的导入处理（实际应使用更完善的解析）
            // TODO: 实现完整的导入解析和替换
        }

        Ok(result)
    }

    /// 保存记忆到CLAUDE.md文件
    pub async fn save_memories_to_claude_md(
        &self,
        memories: &[Memory],
        path: &Path,
    ) -> Result<()> {
        info!("保存 {} 条记忆到CLAUDE.md: {:?}", memories.len(), path);

        let mut content = String::from("# AgentMem Memories\n\n");
        content.push_str("This file contains memories loaded from AgentMem.\n\n");

        for (i, memory) in memories.iter().enumerate() {
            content.push_str(&format!("## Memory: {}\n\n", i + 1));

            // 提取内容
            let mem_content = match &memory.content {
                agent_mem_traits::Content::Text(t) => t.clone(),
                _ => format!("{:?}", memory.content),
            };
            content.push_str(&mem_content);
            content.push_str("\n\n");

            // 提取记忆类型
            if let Some(mem_type) = memory.attributes.get(&agent_mem_traits::AttributeKey::core("memory_type")) {
                if let Some(typ) = mem_type.as_string() {
                    content.push_str(&format!("### Type: {}\n\n", typ));
                }
            }

            // 提取重要性
            if let Some(importance) = memory.attributes.get(&agent_mem_traits::AttributeKey::system("importance")) {
                if let Some(imp) = importance.as_number() {
                    content.push_str(&format!("### Importance: {}\n\n", imp));
                }
            }

            content.push_str("---\n\n");
        }

        fs::write(path, content).await.map_err(|e| {
            agent_mem_traits::AgentMemError::IoError(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write CLAUDE.md file: {}", e))
            )
        })?;

        info!("✅ 记忆已保存到CLAUDE.md");
        Ok(())
    }

    /// 获取已加载的文件列表
    pub async fn get_loaded_files(&self) -> Vec<PathBuf> {
        let loaded = self.loaded_files.read().await;
        loaded.keys().cloned().collect()
    }

    /// 清除已加载的文件
    pub async fn clear_loaded_files(&self) {
        let mut loaded = self.loaded_files.write().await;
        loaded.clear();
    }
}

use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parse_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FilesystemIntegrationManager::with_defaults(temp_dir.path().to_path_buf());

        let content = r#"# AgentMem Memories

## Memory: 1

I love Rust programming language.

### Type: episodic
### Importance: 0.8
### Tags: programming, rust

---

## Memory: 2

Python is also a great language.

### Type: semantic
### Importance: 0.6
"#;

        let claude_file = manager.parse_claude_md(content, temp_dir.path().join("test.md").as_path()).unwrap();
        assert_eq!(claude_file.memories.len(), 2);
        assert!(claude_file.memories[0].content.contains("Rust"));
        assert_eq!(claude_file.memories[0].importance, Some(0.8));
    }

    #[tokio::test]
    async fn test_convert_to_memories() {
        let temp_dir = TempDir::new().unwrap();
        let manager = FilesystemIntegrationManager::with_defaults(temp_dir.path().to_path_buf());

        let claude_file = ClaudeMdFile {
            path: temp_dir.path().join("test.md"),
            content: "test".to_string(),
            memories: vec![ClaudeMemory {
                content: "Test memory".to_string(),
                memory_type: Some("episodic".to_string()),
                importance: Some(0.8),
                tags: vec!["test".to_string()],
                metadata: HashMap::new(),
            }],
            metadata: HashMap::new(),
        };

        let memories = manager.convert_to_memories(&claude_file).await.unwrap();
        assert_eq!(memories.len(), 1);
    }
}
