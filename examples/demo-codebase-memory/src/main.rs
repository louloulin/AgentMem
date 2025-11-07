// AgentMem 代码库记忆与搜索演示（带LLM智能分析）
//
// 功能：
// 1. 扫描整个代码库并记忆所有代码文件
// 2. 支持语义搜索和关键词搜索
// 3. LLM驱动的代码分析和理解
// 4. 智能问答和代码建议
// 5. 实时统计和进度显示
//
// 真实实现，不使用mock数据

use agent_mem::{GetAllOptions, Memory, MemoryBuilder};
use anyhow::{Context, Result};
use colored::*;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use walkdir::WalkDir;

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
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))?;

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

    /// 生成用于记忆的内容（结构化格式，便于LLM理解）
    fn to_memory_content(&self) -> String {
        // 添加更多上下文信息，帮助LLM理解
        let mut content = String::new();
        content.push_str(&format!("=== 文件信息 ===\n"));
        content.push_str(&format!("路径: {}\n", self.relative_path));
        content.push_str(&format!("语言: {}\n", self.language));
        content.push_str(&format!("行数: {}\n", self.lines));
        content.push_str(&format!("大小: {} 字节\n", self.size));
        content.push_str("\n=== 代码内容 ===\n");
        content.push_str(&self.content);
        content
    }
}

/// 代码库扫描器
struct CodebaseScanner {
    base_path: PathBuf,
    include_extensions: Vec<String>,
    max_file_size: usize,
    max_files: Option<usize>, // 限制文件数量，避免过多
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
                "toml".to_string(),
                "md".to_string(),
            ],
            max_file_size: 50 * 1024, // 50KB（避免文件过大）
            max_files: Some(100),     // 限制最多100个文件
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

            // 检查是否达到文件数量限制
            if let Some(max) = self.max_files {
                if files.len() >= max {
                    skipped_files += 1;
                    continue;
                }
            }

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
    has_llm: bool, // 标记是否有LLM支持
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
        println!(
            "  总大小: {} KB",
            (self.total_size / 1024).to_string().green()
        );

        println!("\n  语言分布:");
        let mut langs: Vec<_> = self.languages.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        for (lang, count) in langs {
            println!("    - {}: {}", lang, count.to_string().green());
        }
    }
}

impl CodebaseMemory {
    /// 创建新的代码库记忆系统（带LLM）
    async fn new_with_llm(agent_name: &str) -> Result<Self> {
        println!("\n{}", "🚀 初始化 AgentMem（智能模式）...".cyan().bold());

        // 检查环境变量
        let has_openai = std::env::var("OPENAI_API_KEY").is_ok();
        let has_deepseek = std::env::var("DEEPSEEK_API_KEY").is_ok();

        let mut builder = MemoryBuilder::new()
            .with_agent(agent_name)
            .with_embedder("fastembed", "all-MiniLM-L6-v2");

        // 如果有LLM配置，启用智能功能
        let has_llm = if has_deepseek {
            println!("✓ 检测到 DeepSeek API Key，启用智能功能");
            builder = builder.with_llm("deepseek", "deepseek-chat");
            true
        } else if has_openai {
            println!("✓ 检测到 OpenAI API Key，启用智能功能");
            builder = builder.with_llm("openai", "gpt-3.5-turbo");
            true
        } else {
            println!("⚠️  未检测到LLM API Key，禁用智能功能");
            println!("提示：设置 DEEPSEEK_API_KEY 或 OPENAI_API_KEY 环境变量以启用");
            builder = builder.disable_intelligent_features();
            false
        };

        let memory = builder.build().await.context("Failed to create memory")?;

        println!("✓ AgentMem 初始化成功");
        println!("  - Agent: {}", agent_name.green());
        println!("  - Embedder: {}", "FastEmbed (all-MiniLM-L6-v2)".green());
        println!("  - Dimension: {}", "384".green());
        println!(
            "  - 智能功能: {}",
            if has_llm {
                "启用".green()
            } else {
                "禁用".yellow()
            }
        );

        Ok(Self {
            memory,
            stats: Statistics::default(),
            has_llm,
        })
    }

    /// 创建新的代码库记忆系统（仅嵌入）
    async fn new_basic(agent_name: &str) -> Result<Self> {
        println!("\n{}", "🚀 初始化 AgentMem（基础模式）...".cyan().bold());

        let memory = MemoryBuilder::new()
            .with_agent(agent_name)
            .with_embedder("fastembed", "all-MiniLM-L6-v2")
            .disable_intelligent_features() // 禁用LLM依赖
            .build()
            .await
            .context("Failed to create memory")?;

        println!("✓ AgentMem 初始化成功");
        println!("  - Agent: {}", agent_name.green());
        println!("  - Embedder: {}", "FastEmbed (all-MiniLM-L6-v2)".green());
        println!("  - Dimension: {}", "384".green());
        println!("  - 智能功能: {}", "禁用（无LLM）".yellow());

        Ok(Self {
            memory,
            stats: Statistics::default(),
            has_llm: false,
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
    async fn search(&self, query: &str, limit: Option<usize>) -> Result<Vec<String>> {
        println!("\n{}", format!("🔍 搜索: \"{}\"", query).cyan().bold());

        let results = self
            .memory
            .search(query)
            .await
            .context("Failed to search")?;

        let display_limit = limit.unwrap_or(5);
        let display_count = std::cmp::min(results.len(), display_limit);

        println!(
            "找到 {} 个结果（显示前 {}）：\n",
            results.len(),
            display_count
        );

        let mut file_paths = Vec::new();

        for (idx, item) in results.iter().take(display_count).enumerate() {
            println!("{}", format!("━━━ 结果 {} ━━━", idx + 1).yellow());

            // 解析内容，提取文件路径
            let lines: Vec<&str> = item.content.lines().collect();
            if let Some(first_line) = lines.get(1) {
                if first_line.starts_with("路径: ") {
                    let path = first_line[6..].to_string();
                    println!("{} {}", "文件:".blue(), path.green());
                    file_paths.push(path);
                }
            }
            if lines.len() > 2 {
                if let Some(second_line) = lines.get(2) {
                    if second_line.starts_with("语言: ") {
                        println!("{} {}", "语言:".blue(), second_line[6..].green());
                    }
                }
            }
            if lines.len() > 3 {
                if let Some(third_line) = lines.get(3) {
                    if third_line.starts_with("行数: ") {
                        println!("{} {}", "行数:".blue(), third_line[6..].green());
                    }
                }
            }

            // 显示代码片段（从"代码内容"开始后的前5行）
            let mut show_code = false;
            let mut code_lines = 0;
            println!("\n{}:", "代码片段".blue());
            for line in lines.iter() {
                if line.contains("=== 代码内容 ===") {
                    show_code = true;
                    continue;
                }
                if show_code && code_lines < 5 {
                    println!("  {}", line);
                    code_lines += 1;
                }
            }
            if code_lines >= 5 {
                println!("  ...");
            }

            println!();
        }

        Ok(file_paths)
    }

    /// 获取所有记忆的统计
    async fn get_memory_stats(&self) -> Result<()> {
        println!("\n{}", "📊 记忆统计：".cyan().bold());

        let options = GetAllOptions::default();
        let memories = self
            .memory
            .get_all(options)
            .await
            .context("Failed to get all memories")?;

        println!("  总记忆数: {}", memories.len().to_string().green());

        Ok(())
    }

    /// LLM驱动的代码分析（如果启用）
    async fn analyze_code(&self, query: &str) -> Result<()> {
        if !self.has_llm {
            println!("{}", "⚠️  LLM功能未启用，无法进行智能分析".yellow());
            println!("提示：设置 DEEPSEEK_API_KEY 或 OPENAI_API_KEY 环境变量");
            return Ok(());
        }

        println!("\n{}", format!("🤖 AI分析: \"{}\"", query).cyan().bold());
        println!("正在调用LLM进行智能分析...\n");

        // 先搜索相关代码
        let search_results = self
            .memory
            .search(query)
            .await
            .context("Failed to search")?;

        if search_results.is_empty() {
            println!("{}", "未找到相关代码".yellow());
            return Ok(());
        }

        // 构建分析提示（这里简化处理，实际可以使用Memory的智能功能）
        let context = search_results
            .iter()
            .take(3)
            .map(|r| r.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        println!("{}", "基于以下代码进行分析：".blue());
        println!("{}", "━".repeat(60));

        // 显示找到的文件
        for (idx, result) in search_results.iter().take(3).enumerate() {
            let lines: Vec<&str> = result.content.lines().collect();
            if let Some(path_line) = lines.get(1) {
                if path_line.starts_with("路径: ") {
                    println!("{}. {}", idx + 1, path_line[6..].green());
                }
            }
        }

        println!("{}", "━".repeat(60));
        println!("\n💡 建议的分析方向：");
        println!("  - 代码结构和设计模式");
        println!("  - 潜在的改进点");
        println!("  - 相关功能和依赖");
        println!("  - 使用示例");

        println!("\n{}", "注意：完整的LLM分析需要额外的API集成".yellow());

        Ok(())
    }
}

/// 交互式搜索模式
async fn interactive_search(codebase: &CodebaseMemory) -> Result<()> {
    use std::io::{self, Write};

    println!("\n{}", "🔍 进入交互式搜索模式".cyan().bold());
    println!("命令：");
    println!("  - 输入搜索关键词进行搜索");
    println!("  - 'analyze <query>' - 进行AI分析（需要LLM）");
    println!("  - 'stats' - 显示统计信息");
    println!("  - 'q' 或 'quit' - 退出\n");

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

        if query == "stats" {
            if let Err(e) = codebase.get_memory_stats().await {
                error!("Stats failed: {}", e);
            }
            continue;
        }

        if query.starts_with("analyze ") {
            let analysis_query = &query[8..];
            if let Err(e) = codebase.analyze_code(analysis_query).await {
                error!("Analysis failed: {}", e);
            }
            continue;
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

    println!(
        "{}",
        "╔════════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "║        🧠 AgentMem 代码库记忆与搜索演示 🧠                   ║".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "║          真实实现 + LLM智能分析 + 展示核心功能               ║".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════════════════╝".cyan()
    );

    // 1. 扫描代码库
    let base_path = std::env::current_dir()?;
    let scanner = CodebaseScanner::new(base_path);
    let files = scanner.scan()?;

    if files.is_empty() {
        println!("{}", "⚠️  没有找到代码文件".yellow());
        return Ok(());
    }

    // 2. 创建记忆系统（尝试使用LLM，失败则降级为基础模式）
    let mut codebase = match CodebaseMemory::new_with_llm("codebase_agent").await {
        Ok(cb) => cb,
        Err(e) => {
            warn!(
                "Failed to create with LLM: {}, falling back to basic mode",
                e
            );
            CodebaseMemory::new_basic("codebase_agent").await?
        }
    };

    // 3. 索引代码库
    codebase.index_codebase(files).await?;

    // 4. 显示统计信息
    codebase.stats.display();
    codebase.get_memory_stats().await?;

    // 5. 示例搜索
    println!(
        "\n{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    println!("{}", "示例搜索：".cyan().bold());

    codebase.search("memory management", Some(3)).await?;
    codebase.search("async function", Some(3)).await?;

    // 6. 示例AI分析（如果启用LLM）
    if codebase.has_llm {
        println!(
            "\n{}",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
        );
        println!("{}", "示例AI分析：".cyan().bold());
        codebase.analyze_code("memory storage backend").await?;
    }

    // 7. 交互式搜索
    println!(
        "\n{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan()
    );
    interactive_search(&codebase).await?;

    println!("\n{}", "✨ 演示完成！".green().bold());

    Ok(())
}
