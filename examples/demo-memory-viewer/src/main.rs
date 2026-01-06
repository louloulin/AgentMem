// AgentMem 记忆可视化工具
//
// 功能:
// 1. 查看所有记忆
// 2. 按类型过滤记忆
// 3. 记忆统计
// 4. 搜索记忆
// 5. 导出记忆
//
// 真实实现，对标MIRIX的mirix_memory_viewer.py

use agent_mem::{AddMemoryOptions, GetAllOptions, Memory, MemoryBuilder, MemoryItem};
use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use colored::*;
use tabled::{
    builder::Builder,
    settings::{object::Columns, object::Rows, Alignment, Modify, Style, Width},
};

/// AgentMem 记忆可视化工具
#[derive(Parser)]
#[command(name = "memory-viewer")]
#[command(about = "AgentMem记忆可视化工具 - 查看、搜索、导出记忆", long_about = None)]
struct Cli {
    /// Agent ID
    #[arg(short, long, default_value = "viewer_agent")]
    agent: String,

    /// 子命令
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出所有记忆
    List {
        /// 限制数量
        #[arg(short, long, default_value_t = 20)]
        limit: usize,

        /// 仅显示ID和内容摘要
        #[arg(short, long)]
        brief: bool,
    },

    /// 搜索记忆
    Search {
        /// 搜索查询
        query: String,

        /// 限制数量
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// 显示记忆统计
    Stats,

    /// 显示单个记忆详情
    Show {
        /// 记忆ID
        id: String,
    },

    /// 添加测试记忆
    AddTest {
        /// 测试记忆数量
        #[arg(short, long, default_value_t = 10)]
        count: usize,
    },

    /// 导出记忆到JSON文件
    Export {
        /// 输出文件路径
        #[arg(short, long, default_value = "memories_export.json")]
        output: String,
    },

    /// 可视化记忆（对标MIRIX）
    Visualize {
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
    },
}

/// 记忆统计
#[derive(Debug, Default)]
struct MemoryStatistics {
    total_count: usize,
    total_size_bytes: usize,
    avg_content_length: f64,
    earliest: Option<DateTime<Local>>,
    latest: Option<DateTime<Local>>,
    with_metadata: usize,
    without_metadata: usize,
}

impl MemoryStatistics {
    fn from_memories(memories: &[MemoryItem]) -> Self {
        let mut stats = Self::default();
        stats.total_count = memories.len();

        if memories.is_empty() {
            return stats;
        }

        let mut total_content_length = 0;
        let mut earliest: Option<DateTime<Local>> = None;
        let mut latest: Option<DateTime<Local>> = None;

        for mem in memories {
            // 内容长度
            total_content_length += mem.content.len();
            stats.total_size_bytes += mem.content.as_bytes().len();

            // 时间范围
            let created_at = mem.created_at.with_timezone(&Local);
            match (&earliest, &latest) {
                (None, None) => {
                    earliest = Some(created_at);
                    latest = Some(created_at);
                }
                (Some(e), Some(l)) => {
                    if created_at < *e {
                        earliest = Some(created_at);
                    }
                    if created_at > *l {
                        latest = Some(created_at);
                    }
                }
                _ => {}
            }

            // 元数据
            if !mem.metadata.is_empty() {
                stats.with_metadata += 1;
            } else {
                stats.without_metadata += 1;
            }
        }

        stats.avg_content_length = total_content_length as f64 / memories.len() as f64;
        stats.earliest = earliest;
        stats.latest = latest;

        stats
    }

    fn display(&self) {
        println!(
            "\n{}",
            "╔═══════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║            📊 记忆统计信息                          ║".cyan()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════╝".cyan()
        );

        println!("\n{}", "总体统计:".yellow().bold());
        println!("  - 总记忆数: {}", self.total_count.to_string().green());
        println!(
            "  - 总大小: {} KB",
            (self.total_size_bytes / 1024).to_string().green()
        );
        println!(
            "  - 平均内容长度: {:.2} 字符",
            self.avg_content_length.to_string().green()
        );

        if let (Some(earliest), Some(latest)) = (&self.earliest, &self.latest) {
            println!("\n{}", "时间范围:".yellow().bold());
            println!(
                "  - 最早记忆: {}",
                earliest.format("%Y-%m-%d %H:%M:%S").to_string().green()
            );
            println!(
                "  - 最新记忆: {}",
                latest.format("%Y-%m-%d %H:%M:%S").to_string().green()
            );
        }

        println!("\n{}", "元数据统计:".yellow().bold());
        println!("  - 有元数据: {}", self.with_metadata.to_string().green());
        println!(
            "  - 无元数据: {}",
            self.without_metadata.to_string().green()
        );
    }
}

/// 创建Memory实例
async fn create_memory(agent_id: &str) -> Result<Memory> {
    MemoryBuilder::new()
        .with_agent(agent_id)
        .with_embedder("fastembed", "all-MiniLM-L6-v2")
        .disable_intelligent_features()
        .build()
        .await
        .context("Failed to create memory")
}

/// 列出所有记忆
async fn list_memories(memory: &Memory, limit: usize, brief: bool) -> Result<()> {
    println!("\n{}", "🔍 获取记忆列表...".cyan());

    let options = GetAllOptions {
        user_id: None,
        agent_id: None,
        run_id: None,
        limit: Some(limit),
    };
    let memories = memory
        .get_all(options)
        .await
        .context("Failed to get memories")?;

    if memories.is_empty() {
        println!("{}", "⚠️  没有找到记忆。".yellow());
        return Ok(());
    }

    println!("{} 找到 {} 条记忆：\n", "✓".green(), memories.len());

    if brief {
        // 简要显示
        for (idx, mem) in memories.iter().enumerate() {
            let preview = if mem.content.len() > 80 {
                format!("{}...", &mem.content[..80])
            } else {
                mem.content.clone()
            };
            println!(
                "{}. {} | {}",
                (idx + 1).to_string().cyan(),
                mem.id.bright_black(),
                preview
            );
        }
    } else {
        // 详细表格显示
        let mut builder = Builder::default();
        builder.push_record(vec!["#", "ID", "内容预览", "创建时间"]);

        for (idx, mem) in memories.iter().enumerate() {
            let preview = if mem.content.len() > 50 {
                format!("{}...", &mem.content[..50])
            } else {
                mem.content.clone()
            };
            let created = mem
                .created_at
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M")
                .to_string();

            builder.push_record(vec![
                (idx + 1).to_string(),
                mem.id[..8].to_string(),
                preview,
                created,
            ]);
        }

        let mut table = builder.build();
        table
            .with(Style::modern())
            .with(Modify::new(Rows::first()).with(Alignment::center()))
            .with(Modify::new(Columns::single(2)).with(Width::wrap(50)));

        println!("{}", table);
    }

    Ok(())
}

/// 搜索记忆
async fn search_memories(memory: &Memory, query: &str, limit: usize) -> Result<()> {
    println!("\n{}", format!("🔍 搜索: \"{}\"", query).cyan());

    let results = memory
        .search(query.to_string())
        .await
        .context("Search failed")?;

    if results.is_empty() {
        println!("{}", "⚠️  未找到相关记忆。".yellow());
        return Ok(());
    }

    let display_count = std::cmp::min(results.len(), limit);
    println!(
        "{} 找到 {} 条结果（显示前 {}）：\n",
        "✓".green(),
        results.len(),
        display_count
    );

    for (idx, mem) in results.iter().take(display_count).enumerate() {
        println!("{}", format!("━━━ 结果 {} ━━━", idx + 1).yellow());
        println!("  ID: {}", mem.id.bright_black());
        println!("  内容: {}", mem.content.trim());
        println!(
            "  创建时间: {}",
            mem.created_at
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .green()
        );

        if !mem.metadata.is_empty() {
            println!("  元数据:");
            for (key, value) in mem.metadata.iter() {
                println!("    - {}: {}", key.cyan(), value);
            }
        }
        println!();
    }

    Ok(())
}

/// 显示记忆统计
async fn show_statistics(memory: &Memory) -> Result<()> {
    println!("\n{}", "📊 计算统计信息...".cyan());

    let options = GetAllOptions {
        user_id: None,
        agent_id: None,
        run_id: None,
        limit: None,
    };
    let memories = memory
        .get_all(options)
        .await
        .context("Failed to get memories")?;

    if memories.is_empty() {
        println!("{}", "⚠️  没有记忆数据可供统计。".yellow());
        return Ok(());
    }

    let stats = MemoryStatistics::from_memories(&memories);
    stats.display();

    Ok(())
}

/// 显示单个记忆详情
async fn show_memory_detail(memory: &Memory, id: &str) -> Result<()> {
    println!("\n{}", format!("🔍 查找记忆: {}", id).cyan());

    let options = GetAllOptions {
        user_id: None,
        agent_id: None,
        run_id: None,
        limit: None,
    };
    let memories = memory
        .get_all(options)
        .await
        .context("Failed to get memories")?;

    let mem = memories
        .iter()
        .find(|m| m.id == id || m.id.starts_with(id))
        .context("Memory not found")?;

    println!(
        "\n{}",
        "╔═══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║            📝 记忆详情                              ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".cyan()
    );

    println!("\n{}", "基本信息:".yellow().bold());
    println!("  - ID: {}", mem.id.green());
    println!("  - Agent: {}", mem.agent_id.green());
    println!(
        "  - 创建时间: {}",
        mem.created_at
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .green()
    );
    if let Some(updated_at) = mem.updated_at {
        println!(
            "  - 更新时间: {}",
            updated_at
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .green()
        );
    }

    println!("\n{}", "内容:".yellow().bold());
    println!("  {}", mem.content);

    if !mem.metadata.is_empty() {
        println!("\n{}", "元数据:".yellow().bold());
        for (key, value) in mem.metadata.iter() {
            println!("  - {}: {}", key.cyan(), value);
        }
    }

    println!();

    Ok(())
}

/// 添加测试记忆
async fn add_test_memories(memory: &Memory, count: usize) -> Result<()> {
    println!("\n{}", format!("📝 添加 {} 条测试记忆...", count).cyan());

    let test_memories = vec![
        "I love programming in Rust because of its safety and performance.",
        "Today I learned about async/await in Rust and it's amazing.",
        "The AgentMem library makes memory management so easy.",
        "I need to remember to buy groceries: milk, eggs, and bread.",
        "My favorite programming language is Rust for its zero-cost abstractions.",
        "The Rust compiler is my best friend - it catches bugs at compile time.",
        "I'm working on a project using AgentMem and LangChain integration.",
        "Machine learning and AI are the future of software development.",
        "I should learn more about vector databases and semantic search.",
        "FastEmbed provides excellent local embeddings without API keys.",
    ];

    let mut added = 0;
    for i in 0..count {
        let content = test_memories[i % test_memories.len()];
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("test".to_string(), "true".to_string());
        metadata.insert("index".to_string(), i.to_string());

        let options = AddMemoryOptions {
            user_id: None,
            agent_id: None,
            run_id: None,
            metadata,
            infer: false,
            memory_type: None,
            prompt: None,
        };

        match memory.add_with_options(content, options).await {
            Ok(_) => {
                added += 1;
                if (i + 1) % 5 == 0 || i + 1 == count {
                    println!("  进度: {}/{}", i + 1, count);
                }
            }
            Err(e) => {
                eprintln!("  {} 添加失败: {}", "⚠️".yellow(), e);
            }
        }
    }

    println!(
        "\n{} 成功添加 {} 条测试记忆。",
        "✓".green(),
        added.to_string().green()
    );

    Ok(())
}

/// 导出记忆到JSON
async fn export_memories(memory: &Memory, output_path: &str) -> Result<()> {
    println!("\n{}", format!("💾 导出记忆到: {}", output_path).cyan());

    let options = GetAllOptions {
        user_id: None,
        agent_id: None,
        run_id: None,
        limit: None,
    };
    let memories = memory
        .get_all(options)
        .await
        .context("Failed to get memories")?;

    if memories.is_empty() {
        println!("{}", "⚠️  没有记忆可导出。".yellow());
        return Ok(());
    }

    let json = serde_json::to_string_pretty(&memories).context("Failed to serialize memories")?;
    std::fs::write(output_path, json).context("Failed to write file")?;

    println!(
        "{} 成功导出 {} 条记忆到 {}",
        "✓".green(),
        memories.len().to_string().green(),
        output_path.green()
    );

    Ok(())
}

/// 可视化记忆（对标MIRIX的visualize_memories）
async fn visualize_memories(memory: &Memory, verbose: bool) -> Result<()> {
    println!(
        "\n{}",
        "╔═══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║            🎨 记忆可视化                            ║".cyan()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".cyan()
    );

    let options = GetAllOptions {
        user_id: None,
        agent_id: None,
        run_id: None,
        limit: None,
    };
    let memories = memory
        .get_all(options)
        .await
        .context("Failed to get memories")?;

    if memories.is_empty() {
        println!("\n{}", "⚠️  没有记忆可显示。".yellow());
        return Ok(());
    }

    // 统计信息（对标MIRIX的summary）
    println!("\n{}", "📊 统计摘要:".yellow().bold());
    let stats = MemoryStatistics::from_memories(&memories);
    println!("  - 总记忆数: {}", stats.total_count.to_string().green());
    println!("  - 有元数据: {}", stats.with_metadata.to_string().green());
    println!(
        "  - 无元数据: {}",
        stats.without_metadata.to_string().green()
    );

    // 按用户分组（如果有user_id元数据）
    let mut by_user: std::collections::HashMap<String, Vec<&MemoryItem>> =
        std::collections::HashMap::new();
    for mem in &memories {
        if let Some(user_id) = mem.metadata.get("user_id") {
            by_user
                .entry(user_id.to_string())
                .or_insert_with(Vec::new)
                .push(mem);
        }
    }

    if !by_user.is_empty() {
        println!("\n{}", "👥 按用户分组:".yellow().bold());
        for (user_id, user_mems) in by_user.iter() {
            println!(
                "  - {}: {} 条记忆",
                user_id.cyan(),
                user_mems.len().to_string().green()
            );
        }
    }

    // 显示最近的记忆
    println!("\n{}", "📝 最近的记忆:".yellow().bold());
    let recent_count = std::cmp::min(5, memories.len());
    for (idx, mem) in memories.iter().take(recent_count).enumerate() {
        let preview = if mem.content.len() > 80 {
            format!("{}...", &mem.content[..80])
        } else {
            mem.content.clone()
        };
        let time_str = mem
            .created_at
            .with_timezone(&Local)
            .format("%H:%M:%S")
            .to_string();
        println!(
            "  {}. {} | {}",
            (idx + 1).to_string().cyan(),
            time_str.bright_black(),
            preview
        );
    }

    if verbose {
        // 详细模式 - 显示所有记忆
        println!("\n{}", "━".repeat(60).cyan());
        println!("{}", "详细记忆列表:".yellow().bold());
        for (idx, mem) in memories.iter().enumerate() {
            println!("\n{}", format!("--- 记忆 {} ---", idx + 1).cyan());
            println!("  ID: {}", mem.id.bright_black());
            println!("  内容: {}", mem.content);
            if !mem.metadata.is_empty() {
                println!("  元数据:");
                for (key, value) in mem.metadata.iter() {
                    println!("    - {}: {}", key.cyan(), value);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!(
        "\n{}",
        "╔════════════════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "║           📊 AgentMem 记忆可视化工具 📊                      ║".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "║              真实实现，对标MIRIX Viewer                      ║".cyan()
    );
    println!(
        "{}",
        "║                                                                ║".cyan()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════════════════════╝".cyan()
    );

    println!("\n{}", format!("🚀 Agent: {}", cli.agent).cyan());

    let memory = create_memory(&cli.agent).await?;
    println!("{}", "✓ Memory初始化成功".green());

    match cli.command {
        Commands::List { limit, brief } => {
            list_memories(&memory, limit, brief).await?;
        }
        Commands::Search { query, limit } => {
            search_memories(&memory, &query, limit).await?;
        }
        Commands::Stats => {
            show_statistics(&memory).await?;
        }
        Commands::Show { id } => {
            show_memory_detail(&memory, &id).await?;
        }
        Commands::AddTest { count } => {
            add_test_memories(&memory, count).await?;
        }
        Commands::Export { output } => {
            export_memories(&memory, &output).await?;
        }
        Commands::Visualize { verbose } => {
            visualize_memories(&memory, verbose).await?;
        }
    }

    println!("\n{}", "✨ 完成！".green().bold());

    Ok(())
}
