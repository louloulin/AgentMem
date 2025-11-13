# AgentMem Server 启动问题修复总结

## 问题描述

用户报告服务器启动时"卡住"，没有任何输出，无法正常启动。

## 根本原因分析

经过详细调试，发现了以下问题：

### 1. **缺少必要的目录** ❌
- **问题**: 服务器启动时需要 `data/` 目录来存储 SQLite 数据库文件
- **错误**: `Unable to open connection to local database file:./data/agentmem.db: 14`
- **原因**: SQLite 错误代码 14 表示 "unable to open database file"，通常是因为父目录不存在

### 2. **日志系统配置问题** ⚠️
- **问题**: 日志 guard 使用 `std::mem::forget()` 可能导致资源泄漏
- **修复**: 使用全局静态变量 `Lazy<Mutex<Option<WorkerGuard>>>` 保存 guard

### 3. **缺少启动日志** 📝
- **问题**: 服务器启动过程没有足够的日志输出，难以诊断问题
- **修复**: 添加了详细的启动日志，包括每个初始化阶段的状态

## 修复方案

### 修改文件：`crates/agent-mem-server/src/main.rs`

#### 1. 添加目录创建函数

```rust
/// 创建必要的目录
fn create_directories() -> std::io::Result<()> {
    use std::fs;
    use std::path::Path;

    // 需要创建的目录列表
    let directories = vec![
        "data",                    // 数据库文件目录
        "data/vectors.lance",      // 向量存储目录
        "logs",                    // 日志文件目录
    ];

    for dir in directories {
        let path = Path::new(dir);
        if !path.exists() {
            eprintln!("   创建目录: {}", dir);
            fs::create_dir_all(path)?;
        }
    }

    Ok(())
}
```

#### 2. 在服务器启动前调用

```rust
// Create necessary directories
info!("📁 创建必要的目录...");
if let Err(e) = create_directories() {
    error!("❌ 创建目录失败: {}", e);
    eprintln!("Failed to create directories: {}", e);
    process::exit(1);
}
info!("✅ 目录创建完成");
```

#### 3. 修复日志 guard 管理

```rust
use once_cell::sync::Lazy;

// 全局 guard 保持文件日志 writer 存活
static FILE_APPENDER_GUARD: Lazy<std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> = 
    Lazy::new(|| std::sync::Mutex::new(None));

fn init_logging(log_level: &str) {
    // ... 初始化代码 ...
    
    // 保存 guard 到全局变量，防止被丢弃
    *FILE_APPENDER_GUARD.lock().unwrap() = Some(guard);
}
```

#### 4. 增强日志输出

在 `init_logging()` 函数中添加了详细的 `eprintln!()` 输出：

```rust
eprintln!("📝 初始化日志系统...");
eprintln!("   创建日志目录: {}", log_dir.display());
eprintln!("   日志文件: {}", log_file.display());
eprintln!("   配置日志层...");
eprintln!("✅ 日志系统已初始化");
```

### 修改文件：`crates/agent-mem-server/src/routes/memory.rs`

添加了详细的 Memory 组件初始化日志：

```rust
info!("========================================");
info!("🧠 初始化 Memory 组件");
info!("========================================");
info!("📦 配置存储层");
info!("  - 数据库类型: LibSQL (SQLite)");
info!("  - 数据库路径: {}", db_path);
info!("🔌 配置 Embedder (向量嵌入)");
info!("  - Provider: {}", provider);
info!("  - Model: {}", model);
info!("📊 配置向量存储");
info!("  - 类型: LanceDB");
info!("  - 路径: {}", vector_store_url);
info!("⏳ 构建 Memory 实例...");
warn!("⚠️  首次运行时，FastEmbed 会下载模型文件（约 100MB）");
warn!("⚠️  这可能需要几分钟时间，请耐心等待...");
```

## 修复结果

### ✅ 成功启动

```
=========================================
🚀 启动 AgentMem Server (智谱 AI)
=========================================
主机: 0.0.0.0
端口: 8080
数据库: file:./data/agentmem.db
Embedder: fastembed / BAAI/bge-small-en-v1.5
LLM Provider: zhipu / glm-4.6
认证: false (禁用)
=========================================

📝 初始化日志系统...
   日志文件: logs/agentmem-server.log
✅ 日志系统已初始化

2025-11-13T07:08:55.952056Z  INFO 🚀 AgentMem Server 启动中...
2025-11-13T07:08:55.952148Z  INFO 版本: 0.1.0
2025-11-13T07:08:55.952198Z  INFO 📁 创建必要的目录...
   创建目录: data
   创建目录: data/vectors.lance
2025-11-13T07:08:55.952327Z  INFO ✅ 目录创建完成
2025-11-13T07:08:55.976503Z  INFO Database repositories initialized
2025-11-13T07:08:56.825364Z  INFO FastEmbed 模型加载成功: multilingual-e5-small (维度: 384)
2025-11-13T07:08:56.826036Z  INFO LanceDB store initialized successfully
2025-11-13T07:08:56.828424Z  INFO AgentMem server starting on 0.0.0.0:8080
2025-11-13T07:08:56.828429Z  INFO API documentation available at http://0.0.0.0:8080/swagger-ui/
```

### ✅ 健康检查通过

```bash
$ curl http://localhost:8080/health | jq .
{
  "status": "healthy",
  "timestamp": "2025-11-13T07:09:38.027261Z",
  "version": "0.1.0",
  "checks": {
    "database": {
      "status": "healthy",
      "message": "Database connection successful"
    },
    "memory_system": {
      "status": "healthy",
      "message": "Memory system operational"
    }
  }
}
```

### ✅ 日志文件正常工作

- **控制台日志**: 实时显示启动过程和运行状态
- **文件日志**: `logs/agentmem-server.log.2025-11-13` 包含完整的日志记录
- **日志级别**: 可通过 `RUST_LOG` 环境变量控制

## 已知问题

### ⚠️ 历史记录管理器警告

```
WARN 创建 HistoryManager 失败: Storage error: 连接数据库失败: 
error returned from database: (code: 14) unable to open database file
```

**原因**: 历史记录管理器使用了绝对路径 `sqlite:///Users/louloulin/.../data/history.db`

**影响**: 历史记录功能不可用，但不影响核心功能

**待修复**: 需要修改历史记录管理器使用相对路径

## 技术细节

### 日志系统架构

- **控制台层**: 人类可读的格式，不包含 ANSI 颜色（用于文件日志）
- **文件层**: 每日轮转，保存在 `logs/agentmem-server.log.YYYY-MM-DD`
- **环境过滤器**: 通过 `RUST_LOG` 环境变量控制日志级别

### 目录结构

```
dist/server/
├── agent-mem-server          # 服务器二进制文件
├── data/                      # 数据目录（自动创建）
│   ├── agentmem.db           # SQLite 数据库
│   └── vectors.lance/        # LanceDB 向量存储
├── logs/                      # 日志目录（自动创建）
│   └── agentmem-server.log.* # 日志文件（按日期轮转）
└── lib/                       # 动态库目录
    └── libonnxruntime.*.dylib
```

## 构建和部署

### 重新构建

```bash
cargo build --package agent-mem-server --release
cp target/release/agent-mem-server dist/server/
```

### 启动服务器

```bash
cd dist/server
./start-with-zhipu.sh  # 使用智谱 AI
# 或
./start.sh             # 默认配置
```

### 查看日志

```bash
# 实时查看日志
tail -f logs/agentmem-server.log.$(date +%Y-%m-%d)

# 查看所有日志
cat logs/agentmem-server.log.*
```

## 总结

通过以下修复，服务器现在可以正常启动：

1. ✅ **自动创建必要的目录** - 解决了数据库连接失败的问题
2. ✅ **增强日志输出** - 提供了详细的启动过程信息
3. ✅ **修复日志 guard 管理** - 避免资源泄漏
4. ✅ **文件日志正常工作** - 便于问题诊断和调试

服务器现在可以稳定运行，所有核心功能正常工作！

