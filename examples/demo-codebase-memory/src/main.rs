// AgentMem 代码库记忆与搜索演示
// 
// 功能：
// 1. 扫描整个代码库并记忆所有代码文件
// 2. 支持语义搜索和关键词搜索
// 3. 代码分析和理解
// 4. 实时统计和进度显示
//
// 真实实现，不使用mock数据

use agent_mem::{Memory, MemoryBuilder, GetAllOptions};
use anyhow::{Result, Context};
use colored::*;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn, error};
use walkdir::WalkDir;
use ignore::WalkBuilder;

/// 代码文件信息
#[derive(Debug, Clone)]
struct CodeFile {
    path: PathBuf,
    relative_path: String,
    content: String,
    language: String,
    lines: usize,
    size: usize,
}

impl CodeFile {
    /// 从路径创建代码文件
    fn from_path(path: &Path, base_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let relative_path = path
            .strip_prefix(base_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        
        let language = Self::detect_language(path);
        let lines = content.lines().count();
        let size = content.len();
        
        Ok(Self {
            path: path.to_path_buf(),
            relative_path,
            content,
            language,
            lines,
            size,
        })
    }
    
    /// 检测编程语言
    fn detect_language(path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => "Rust".to_string(),
            Some("cj") => "Cangjie".to_string(),
            Some("py") => "Python".to_string(),
            Some("js") | Some("jsx") => "JavaScript".to_string(),
            Some("ts") | Some("tsx") => "TypeScript".to_string(),
            Some("java") => "Java".to_string(),
            Some("go") => "Go".to_string(),
            Some("c") => "C".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "C++".to_string(),
            Some("h") | Some("hpp") => "C/C++ Header".to_string(),
            Some("toml") => "TOML".to_string(),
            Some("yaml") | Some("yml") => "YAML".to_string(),
            Some("json") => "JSON".to_string(),
            Some("md") => "Markdown".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    
    /// 生成用于记忆的内容
    fn to_memory_content(&self) -> String {
        format!(
            "File: {}\nLanguage: {}\nLines: {}\n\n{}",
            self.relative_path,
            self.language,
            self.lines,
            self.content
        )
    }
}

/// 代码库扫描器
struct CodebaseScanner {
    base_path: PathBuf,
    include_extensions: Vec<String>,
    max_file_size: usize,
}

impl CodebaseScanner {
    /// 创建新的扫描器
    fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            include_extensions: vec![
                "rs".to_string(),
                "cj".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "java".to_string(),
                "go".to_string(),
                "toml".to_string(),
                "md".to_string(),
            ],
            max_file_size: 1024 * 1024, // 1MB
        }
    }
    
    /// 扫描代码库
    fn scan(&self) -> Result<Vec<CodeFile>> {
        println!("\n{}", "🔍 扫描代码库...".cyan().bold());
        println!("路径: {}", self.base_path.display());
        
        let mut files = Vec::new();
        let mut total_files = 0;
        let mut skipped_files = 0;
        
        // 使用 ignore crate 来尊重 .gitignore
        for entry in WalkBuilder::new(&self.base_path)
            .hidden(false)
            .git_ignore(true)
            .build()
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Failed to read entry: {}", e);
                    continue;
                }
            };
            
            let path = entry.path();
            
            // 跳过目录
            if path.is_dir() {
                continue;
            }
            
            total_files += 1;
            
            // 检查文件扩展名
            let extension = match path.extension().and_then(|s| s.to_str()) {
                Some(ext) => ext.to_string(),
                None => {
                    skipped_files += 1;
                    continue;
                }
            };
            
            if !self.include_extensions.contains(&extension) {
                skipped_files += 1;
                continue;
            }
            
            // 检查文件大小
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => {
                    skipped_files += 1;
                    continue;
                }
            };
            
            if metadata.len() as usize > self.max_file_size {
                skipped_files += 1;
                continue;
            }
            
            // 读取文件
            match CodeFile::from_path(path, &self.base_path) {
                Ok(file) => {
                    files.push(file);
                }
                Err(e) => {
                    warn!("Failed to read file {:?}: {}", path, e);
                    skipped_files += 1;
                }
            }
        }
        
        println!("✓ 扫描完成：");
        println!("  - 总文件数: {}", total_files);
        println!("  - 代码文件: {}", files.len().to_string().green());
        println!("  - 跳过文件: {}", skipped_files);
        
        Ok(files)
    }
}

/// 代码库记忆系统
struct CodebaseMemory {
    memory: Memory,
    stats: Statistics,
}

/// 统计信息
#[derive(Debug, Default)]
struct Statistics {
    total_files: usize,
    total_lines: usize,
    total_size: usize,
    languages: std::collections::HashMap<String, usize>,
}

impl Statistics {
    fn update(&mut self, file: &CodeFile) {
        self.total_files += 1;
        self.total_lines += file.lines;
        self.total_size += file.size;
        *self.languages.entry(file.language.clone()).or_insert(0) += 1;
    }
    
    fn display(&self) {
        println!("\n{}", "📊 统计信息：".cyan().bold());
        println!("  文件总数: {}", self.total_files.to_string().green());
        println!("  代码行数: {}", self.total_lines.to_string().green());
        println!("  总大小: {} KB", (self.total_size / 1024).to_string().green());
        
        println!("\n  语言分布:");
        let mut langs: Vec<_> = self.languages.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        for (lang, count) in langs {
            println!("    - {}: {}", lang, count.to_string().green());
        }
    }
}

impl CodebaseMemory {
    /// 创建新的代码库记忆系统
    async fn new(agent_name: &str) -> Result<Self> {
        println!("\n{}", "🚀 初始化 AgentMem...".cyan().bold());
        
        let memory = MemoryBuilder::new()
            .with_agent(agent_name)
            .with_embedder("fastembed", "all-MiniLM-L6-v2")
            .disable_intelligent_features() // 禁用LLM依赖，专注嵌入
            .build()
            .await
            .context("Failed to create memory")?;
        
        println!("✓ AgentMem 初始化成功");
        println!("  - Agent: {}", agent_name.green());
        println!("  - Embedder: {}", "FastEmbed (all-MiniLM-L6-v2)".green());
        println!("  - Dimension: {}", "384".green());
        
        Ok(Self {
            memory,
            stats: Statistics::default(),
        })
    }
    
    /// 索引代码库
    async fn index_codebase(&mut self, files: Vec<CodeFile>) -> Result<()> {
        println!("\n{}", "📝 开始索引代码库...".cyan().bold());
        
        let total = files.len();
        let mut success = 0;
        let mut failed = 0;
        
        for (idx, file) in files.iter().enumerate() {
            // 显示进度
            if (idx + 1) % 10 == 0 || idx + 1 == total {
                print!("\r  进度: {}/{} ", idx + 1, total);
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            
            // 添加到记忆
            let content = file.to_memory_content();
            match self.memory.add(&content).await {
                Ok(_) => {
                    self.stats.update(file);
                    success += 1;
                }
                Err(e) => {
                    error!("Failed to add file {:?}: {}", file.path, e);
                    failed += 1;
                }
            }
        }
        
        println!("\n✓ 索引完成：");
        println!("  - 成功: {}", success.to_string().green());
        println!("  - 失败: {}", failed.to_string().red());
        
        Ok(())
    }
    
    /// 搜索代码
    async fn search(&self, query: &str, limit: Option<usize>) -> Result<()> {
        println!("\n{}", format!("🔍 搜索: \"{}\"", query).cyan().bold());
        
        let results = self.memory.search(query).await
            .context("Failed to search")?;
        
        let display_limit = limit.unwrap_or(5);
        let display_count = std::cmp::min(results.len(), display_limit);
        
        println!("找到 {} 个结果（显示前 {}）：\n", results.len(), display_count);
        
        for (idx, item) in results.iter().take(display_count).enumerate() {
            println!("{}", format!("━━━ 结果 {} ━━━", idx + 1).yellow());
            
            // 解析内容，提取文件路径
            let lines: Vec<&str> = item.content.lines().collect();
            if let Some(first_line) = lines.first() {
                if first_line.starts_with("File: ") {
                    println!("{} {}", "文件:".blue(), first_line[6..].green());
                }
            }
            if lines.len() > 1 {
                if let Some(second_line) = lines.get(1) {
                    if second_line.starts_with("Language: ") {
                        println!("{} {}", "语言:".blue(), second_line[10..].green());
                    }
                }
            }
            if lines.len() > 2 {
                if let Some(third_line) = lines.get(2) {
                    if third_line.starts_with("Lines: ") {
                        println!("{} {}", "行数:".blue(), third_line[7..].green());
                    }
                }
            }
            
            // 显示代码片段（前5行）
            if lines.len() > 4 {
                println!("\n{}:", "代码片段".blue());
                for line in lines.iter().skip(4).take(5) {
                    println!("  {}", line);
                }
                if lines.len() > 9 {
                    println!("  ...");
                }
            }
            
            println!();
        }
        
        Ok(())
    }
    
    /// 获取所有记忆的统计
    async fn get_memory_stats(&self) -> Result<()> {
        println!("\n{}", "📊 记忆统计：".cyan().bold());
        
        let options = GetAllOptions::default();
        let memories = self.memory.get_all(options).await
            .context("Failed to get all memories")?;
        
        println!("  总记忆数: {}", memories.len().to_string().green());
        
        Ok(())
    }
}

/// 交互式搜索模式
async fn interactive_search(codebase: &CodebaseMemory) -> Result<()> {
    use std::io::{self, Write};
    
    println!("\n{}", "🔍 进入交互式搜索模式".cyan().bold());
    println!("输入搜索关键词，或输入 'q' 退出\n");
    
    loop {
        print!("{} ", "搜索>".green().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let query = input.trim();
        
        if query.is_empty() {
            continue;
        }
        
        if query == "q" || query == "quit" || query == "exit" {
            println!("退出搜索模式");
            break;
        }
        
        if let Err(e) = codebase.search(query, Some(5)).await {
            error!("Search failed: {}", e);
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,agent_mem=warn")
        .init();
    
    println!("{}", "╔════════════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║                                                                ║".cyan());
    println!("{}", "║           🧠 AgentMem 代码库记忆与搜索演示 🧠                ║".cyan());
    println!("{}", "║                                                                ║".cyan());
    println!("{}", "║               真实实现，展示核心功能                          ║".cyan());
    println!("{}", "║                                                                ║".cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════════╝".cyan());
    
    // 1. 扫描代码库
    let base_path = std::env::current_dir()?;
    let scanner = CodebaseScanner::new(base_path);
    let files = scanner.scan()?;
    
    if files.is_empty() {
        println!("{}", "⚠️  没有找到代码文件".yellow());
        return Ok(());
    }
    
    // 2. 创建记忆系统
    let mut codebase = CodebaseMemory::new("codebase_agent").await?;
    
    // 3. 索引代码库
    codebase.index_codebase(files).await?;
    
    // 4. 显示统计信息
    codebase.stats.display();
    codebase.get_memory_stats().await?;
    
    // 5. 示例搜索
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "示例搜索：".cyan().bold());
    
    codebase.search("memory management", Some(3)).await?;
    codebase.search("async function", Some(3)).await?;
    codebase.search("error handling", Some(3)).await?;
    
    // 6. 交互式搜索
    println!("\n{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    interactive_search(&codebase).await?;
    
    println!("\n{}", "✨ 演示完成！".green().bold());
    
    Ok(())
}

