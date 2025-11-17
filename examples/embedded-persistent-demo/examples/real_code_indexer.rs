//! 真实代码索引器 - 索引整个 AgentMem 代码库
//!
//! 本示例演示如何使用 AgentMem 构建真实的代码搜索系统：
//! 1. 扫描 AgentMem 代码库中的所有 Rust 文件
//! 2. 提取函数、结构体、trait 等代码元素
//! 3. 批量索引到 AgentMem
//! 4. 执行语义搜索验证
//! 5. 性能分析和统计

#[path = "shared/simple_memory_adapter.rs"]
mod simple_memory_adapter;
use simple_memory_adapter::SimpleMemory;
use agent_mem_traits::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// 代码元素类型
#[derive(Debug, Clone, PartialEq)]
enum CodeElementType {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Module,
}

impl CodeElementType {
    fn as_str(&self) -> &str {
        match self {
            CodeElementType::Function => "function",
            CodeElementType::Struct => "struct",
            CodeElementType::Enum => "enum",
            CodeElementType::Trait => "trait",
            CodeElementType::Impl => "impl",
            CodeElementType::Const => "const",
            CodeElementType::Module => "module",
        }
    }
}

/// 代码元素
#[derive(Debug, Clone)]
struct CodeElement {
    element_type: CodeElementType,
    name: String,
    signature: String,
    doc_comment: Option<String>,
    file_path: String,
    line_number: usize,
}

impl CodeElement {
    /// 转换为记忆内容
    fn to_memory_content(&self) -> String {
        let mut content = format!(
            "[{}] {} in {}\n",
            self.element_type.as_str(),
            self.name,
            self.file_path
        );

        if let Some(doc) = &self.doc_comment {
            content.push_str(&format!("Documentation: {}\n", doc));
        }

        content.push_str(&format!("\nSignature:\n{}\n", self.signature));
        content.push_str(&format!(
            "Location: {}:{}",
            self.file_path, self.line_number
        ));

        content
    }

    /// 生成元数据
    fn to_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), self.element_type.as_str().to_string());
        metadata.insert("name".to_string(), self.name.clone());
        metadata.insert("file".to_string(), self.file_path.clone());
        metadata.insert("line".to_string(), self.line_number.to_string());
        metadata
    }
}

/// 代码扫描器
struct CodeScanner {
    root_path: PathBuf,
    elements: Vec<CodeElement>,
}

impl CodeScanner {
    fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            elements: Vec::new(),
        }
    }

    /// 扫描目录
    fn scan(&mut self) -> Result<()> {
        println!("📂 扫描代码库: {:?}", self.root_path);

        let root_path = self.root_path.clone();
        self.scan_directory(&root_path)?;

        println!("✅ 扫描完成，找到 {} 个代码元素", self.elements.len());
        Ok(())
    }

    /// 递归扫描目录
    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        // 跳过 target 和隐藏目录
        if let Some(name) = dir.file_name() {
            let name_str = name.to_string_lossy();
            if name_str == "target" || name_str.starts_with('.') {
                return Ok(());
            }
        }

        for entry in fs::read_dir(dir).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("Failed to read dir: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                agent_mem_traits::AgentMemError::internal_error(format!(
                    "Failed to read entry: {}",
                    e
                ))
            })?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.scan_rust_file(&path)?;
            }
        }

        Ok(())
    }

    /// 扫描 Rust 文件
    fn scan_rust_file(&mut self, file_path: &Path) -> Result<()> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            agent_mem_traits::AgentMemError::internal_error(format!("Failed to read file: {}", e))
        })?;

        let relative_path = file_path
            .strip_prefix(&self.root_path)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        // 提取代码元素
        self.extract_functions(&content, &relative_path);
        self.extract_structs(&content, &relative_path);
        self.extract_traits(&content, &relative_path);
        self.extract_enums(&content, &relative_path);

        Ok(())
    }

    /// 提取函数
    fn extract_functions(&mut self, content: &str, file_path: &str) {
        // 匹配 pub fn, async fn, pub async fn 等
        let re = Regex::new(r"(?m)^[\s]*(pub\s+)?(async\s+)?fn\s+(\w+)\s*(<[^>]+>)?\s*\([^)]*\)")
            .unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(3) {
                    let doc = self.extract_doc_comment(content, line_num);

                    self.elements.push(CodeElement {
                        element_type: CodeElementType::Function,
                        name: name.as_str().to_string(),
                        signature: line.trim().to_string(),
                        doc_comment: doc,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
        }
    }

    /// 提取结构体
    fn extract_structs(&mut self, content: &str, file_path: &str) {
        let re = Regex::new(r"(?m)^[\s]*(pub\s+)?struct\s+(\w+)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(2) {
                    let doc = self.extract_doc_comment(content, line_num);

                    self.elements.push(CodeElement {
                        element_type: CodeElementType::Struct,
                        name: name.as_str().to_string(),
                        signature: line.trim().to_string(),
                        doc_comment: doc,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
        }
    }

    /// 提取 trait
    fn extract_traits(&mut self, content: &str, file_path: &str) {
        let re = Regex::new(r"(?m)^[\s]*(pub\s+)?trait\s+(\w+)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(2) {
                    let doc = self.extract_doc_comment(content, line_num);

                    self.elements.push(CodeElement {
                        element_type: CodeElementType::Trait,
                        name: name.as_str().to_string(),
                        signature: line.trim().to_string(),
                        doc_comment: doc,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
        }
    }

    /// 提取枚举
    fn extract_enums(&mut self, content: &str, file_path: &str) {
        let re = Regex::new(r"(?m)^[\s]*(pub\s+)?enum\s+(\w+)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                if let Some(name) = caps.get(2) {
                    let doc = self.extract_doc_comment(content, line_num);

                    self.elements.push(CodeElement {
                        element_type: CodeElementType::Enum,
                        name: name.as_str().to_string(),
                        signature: line.trim().to_string(),
                        doc_comment: doc,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
        }
    }

    /// 提取文档注释
    fn extract_doc_comment(&self, content: &str, line_num: usize) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut doc_lines = Vec::new();

        // 向上查找文档注释
        for i in (0..line_num).rev() {
            let line = lines[i].trim();
            if line.starts_with("///") {
                doc_lines.insert(0, line.trim_start_matches("///").trim());
            } else if line.starts_with("//!") {
                doc_lines.insert(0, line.trim_start_matches("//!").trim());
            } else if !line.is_empty() && !line.starts_with("//") {
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 AgentMem 真实代码索引器");
    println!("{}", "=".repeat(70));

    // 1. 创建 SimpleMemory 实例
    println!("\n📦 1. 初始化 SimpleMemory...");
    let memory = SimpleMemory::new().await?;
    println!("   ✅ SimpleMemory 创建成功");

    // 2. 扫描代码库
    println!("\n📂 2. 扫描 AgentMem 代码库...");
    let crates_path = PathBuf::from("../../crates/agent-mem-core/src");

    if !crates_path.exists() {
        println!("   ⚠️  路径不存在: {:?}", crates_path);
        println!("   使用当前目录的示例代码");
        return Ok(());
    }

    let mut scanner = CodeScanner::new(crates_path);
    scanner.scan()?;

    // 统计信息
    let mut type_stats: HashMap<String, usize> = HashMap::new();
    for element in &scanner.elements {
        *type_stats
            .entry(element.element_type.as_str().to_string())
            .or_insert(0) += 1;
    }

    println!("\n   📊 代码元素统计:");
    for (elem_type, count) in type_stats.iter() {
        println!("      - {}: {}", elem_type, count);
    }

    // 3. 批量索引到 AgentMem
    println!("\n🔨 3. 批量索引代码元素...");
    let start = Instant::now();
    let mut indexed_count = 0;
    let max_to_index = 50; // 限制索引数量以加快演示

    for (i, element) in scanner.elements.iter().take(max_to_index).enumerate() {
        let content = element.to_memory_content();
        let metadata = element.to_metadata();

        // 添加到记忆系统
        let _id = memory.add_with_metadata(&content, Some(metadata)).await?;
        indexed_count += 1;

        if (i + 1) % 10 == 0 {
            println!(
                "   [{:3}/{}] 已索引 {} 个元素...",
                i + 1,
                max_to_index,
                i + 1
            );
        }
    }

    let duration = start.elapsed();
    let ops_per_sec = indexed_count as f64 / duration.as_secs_f64();

    println!("\n   ✅ 索引完成:");
    println!("      总数: {} 个代码元素", indexed_count);
    println!("      耗时: {:.2?}", duration);
    println!("      吞吐量: {:.0} ops/s", ops_per_sec);

    // 4. 验证索引
    println!("\n📋 4. 验证索引...");
    let all_memories = memory.get_all().await?;
    println!("   ✅ 存储的记忆总数: {}", all_memories.len());

    // 5. 语义搜索测试
    println!("\n🔍 5. 语义搜索测试...");
    println!("{}", "-".repeat(70));

    let search_queries = vec![
        "如何创建 Agent？",
        "SimpleMemory 的实现",
        "MemoryManager 是什么？",
        "trait 定义",
        "配置相关的代码",
    ];

    for (i, query) in search_queries.iter().enumerate() {
        println!("\n   查询 {}: \"{}\"", i + 1, query);

        let start = Instant::now();
        let results = memory.search(*query).await?;
        let duration = start.elapsed();

        println!("   ⏱️  搜索耗时: {:.2?}", duration);
        println!("   📊 找到 {} 条结果", results.len());

        if !results.is_empty() {
            println!("   🎯 Top 3 结果:");
            for (j, result) in results.iter().take(3).enumerate() {
                let first_line = result.content.lines().next().unwrap_or("Unknown");
                println!("      {}. {}", j + 1, first_line);
            }
        }
    }

    // 6. 总结
    println!("\n{}", "=".repeat(70));
    println!("✅ 真实代码索引演示完成！");
    println!("\n📈 关键指标:");
    println!("   - 扫描文件: {} 个 Rust 文件", scanner.elements.len());
    println!("   - 索引元素: {} 个代码元素", indexed_count);
    println!("   - 索引速度: {:.0} ops/s", ops_per_sec);
    println!("   - 搜索查询: {} 次", search_queries.len());

    println!("\n💡 应用场景:");
    println!("   ✓ 代码库智能搜索");
    println!("   ✓ API 文档检索");
    println!("   ✓ 代码片段推荐");
    println!("   ✓ 开发知识库");
    println!("   ✓ AI 编程助手");
    println!("   ✓ 代码审查辅助");
    println!("   ✓ 新人代码导航");

    Ok(())
}
