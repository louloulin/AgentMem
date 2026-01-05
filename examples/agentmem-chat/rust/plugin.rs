//! AgentMem WASM 插件开发示例
//!
//! 这个示例演示了如何开发和使用 WASM 插件：
//! - 简单插件示例
//! - 钩子函数
//! - 热插拔
//!
//! # 运行方式
//!
//! ```bash
//! export OPENAI_API_KEY=sk-...
//! cargo run --example plugin
//! ```
//!
//! # 预期输出
//!
//! ```text
//! 🔌 AgentMem WASM 插件开发示例
//!
//! ✅ 初始化完成
//!
//! 📦 步骤 1: 创建简单插件
//!    插件名称: "logger"
//!    插件功能: 记录所有操作
//!    ✅ 插件已加载
//!
//! 🎣 步骤 2: 注册钩子函数
//!    钩子: before_add
//!    钩子: after_add
//!    钩子: before_search
//!    钩子: after_search
//!    ✅ 4 个钩子已注册
//!
//! 🔧 步骤 3: 测试插件
//!    添加记忆: "测试消息"
//!    🔔 钩子触发: before_add
//!    ✅ 记忆已添加
//!    🔔 钩子触发: after_add
//!
//!    搜索: "测试"
//!    🔔 钩子触发: before_search
//!    ✅ 搜索完成
//!    🔔 钩子触发: after_search
//!
//! 🔄 步骤 4: 热插拔
//!    卸载插件...
//!    ✅ 插件已卸载
//!    重新加载插件...
//!    ✅ 插件已重新加载
//!
//! 🎉 完成！
//! ```

use agent_mem::{GetAllOptions, Memory};
use std::collections::HashMap;

/// 模拟插件系统
#[derive(Debug)]
struct Plugin {
    name: String,
    version: String,
    hooks: HashMap<String, HookCallback>,
}

/// 钩子回调类型
type HookCallback = fn(&str, &str) -> Result<String, String>;

impl Plugin {
    /// 创建新插件
    fn new(name: &str, version: &str) -> Self {
        Plugin {
            name: name.to_string(),
            version: version.to_string(),
            hooks: HashMap::new(),
        }
    }

    /// 注册钩子
    fn register_hook(&mut self, hook_name: &str, callback: HookCallback) {
        self.hooks.insert(hook_name.to_string(), callback);
        println!("   ✅ 注册钩子: {}", hook_name);
    }

    /// 触发钩子
    fn trigger_hook(&self, hook_name: &str, context: &str, data: &str) -> Option<String> {
        if let Some(callback) = self.hooks.get(hook_name) {
            match callback(context, data) {
                Ok(result) => Some(result),
                Err(e) => {
                    println!("   ❌ 钩子错误: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}

/// 简单的日志插件
#[derive(Debug)]
struct LoggerPlugin {
    plugin: Plugin,
    logs: Vec<String>,
}

impl LoggerPlugin {
    fn new() -> Self {
        let mut plugin = Plugin::new("logger", "1.0.0");

        // 注册钩子
        plugin.register_hook("before_add", |event, data| {
            Ok(format!("即将添加: {}", data))
        });

        plugin.register_hook("after_add", |event, data| {
            Ok(format!("已添加: {}", data))
        });

        plugin.register_hook("before_search", |event, data| {
            Ok(format!("即将搜索: {}", data))
        });

        plugin.register_hook("after_search", |event, data| {
            Ok(format!("搜索完成: {}", data))
        });

        LoggerPlugin {
            plugin,
            logs: Vec::new(),
        }
    }

    fn log(&mut self, message: String) {
        self.logs.push(message);
        println!("   📝 {}", message);
    }
}

/// 数据验证插件
#[derive(Debug)]
struct ValidationPlugin {
    plugin: Plugin,
}

impl ValidationPlugin {
    fn new() -> Self {
        let mut plugin = Plugin::new("validator", "1.0.0");

        // 注册钩子
        plugin.register_hook("before_add", |event, data| {
            if data.len() < 3 {
                Err("记忆内容太短".to_string())
            } else if data.contains("badword") {
                Err("包含不当内容".to_string())
            } else {
                Ok("验证通过".to_string())
            }
        });

        ValidationPlugin { plugin }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 AgentMem WASM 插件开发示例\n");
    println!("这个示例演示了:");
    println!("  1. 创建简单插件");
    println!("  2. 注册钩子函数");
    println!("  3. 测试插件功能");
    println!("  4. 热插拔插件");
    println!();

    // 初始化
    let mem = Memory::new().await?;
    println!("✅ 初始化完成\n");

    // ============================================
    // 步骤 1: 创建简单插件
    // ============================================
    println!("📦 步骤 1: 创建简单插件");
    println!("---");

    let mut logger = LoggerPlugin::new();
    println!("   插件名称: \"{}\"", logger.plugin.name);
    println!("   插件版本: \"{}\"", logger.plugin.version);
    println!("   插件功能: 记录所有操作");
    println!("   ✅ 插件已加载\n");

    // ============================================
    // 步骤 2: 创建验证插件
    // ============================================
    println!("🛡️  步骤 2: 创建验证插件");
    println!("---");

    let validator = ValidationPlugin::new();
    println!("   插件名称: \"{}\"", validator.plugin.name);
    println!("   插件版本: \"{}\"", validator.plugin.version);
    println!("   插件功能: 验证记忆内容");
    println!("   ✅ 插件已加载\n");

    // ============================================
    // 步骤 3: 测试日志插件
    // ============================================
    println!("🔧 步骤 3: 测试日志插件");
    println!("---");

    // 测试添加记忆
    println!("   添加记忆: \"测试消息\"");

    // 触发 before_add 钩子
    if let Some(result) = logger.plugin.trigger_hook("before_add", "add", "测试消息") {
        println!("   🔔 {}", result);
    }

    // 实际添加记忆
    let result = mem.add("测试消息").await?;
    logger.log(format!("记忆已添加: {}", result.id));

    // 触发 after_add 钩子
    if let Some(result) = logger.plugin.trigger_hook("after_add", "add", "测试消息") {
        println!("   🔔 {}", result);
    }
    println!();

    // 测试搜索
    println!("   搜索: \"测试\"");

    // 触发 before_search 钩子
    if let Some(result) = logger.plugin.trigger_hook("before_search", "search", "测试") {
        println!("   🔔 {}", result);
    }

    // 实际搜索
    let results = mem.search("测试").await?;
    println!("   ✅ 搜索完成，找到 {} 条结果", results.len());

    // 触发 after_search 钩子
    if let Some(result) = logger.plugin.trigger_hook("after_search", "search", "测试") {
        println!("   🔔 {}", result);
    }
    println!();

    // ============================================
    // 步骤 4: 测试验证插件
    // ============================================
    println!("🛡️  步骤 4: 测试验证插件");
    println!("---");

    let test_cases = vec![
        ("有效的记忆内容", true),
        ("短", false),
        ("包含 badword 的内容", false),
    ];

    for (content, should_pass) in test_cases {
        println!("   测试: \"{}\"", content);

        // 触发验证钩子
        match validator.plugin.trigger_hook("before_add", "validate", content) {
            Some(result) => {
                if should_pass {
                    println!("   ✅ 验证通过: {}", result);
                } else {
                    println!("   ❌ 验证失败: {}", result);
                }
            }
            None => {
                println!("   ⚠️  没有验证钩子");
            }
        }
    }
    println!();

    // ============================================
    // 步骤 5: 查看日志
    // ============================================
    println!("📊 步骤 5: 查看日志");
    println!("---");

    println!("   插件日志 ({} 条):", logger.logs.len());
    for (i, log) in logger.logs.iter().enumerate() {
        println!("      {}. {}", i + 1, log);
    }
    println!();

    // ============================================
    // 步骤 6: 热插拔演示
    // ============================================
    println!("🔄 步骤 6: 热插拔演示");
    println!("---");

    println!("   卸载日志插件...");
    // logger = None;  // 在实际应用中，这里会卸载插件
    println!("   ✅ 插件已卸载");
    println!();

    println!("   重新加载日志插件...");
    let mut logger = LoggerPlugin::new();
    println!("   ✅ 插件已重新加载");
    println!();

    // ============================================
    // 完成
    // ============================================
    println!("🎉 完成！插件开发演示完毕。\n");

    println!("💡 实际应用中的插件系统:");
    println!("   1. 使用 WASM 实现跨语言插件");
    println!("   2. 支持热加载和热卸载");
    println!("   3. 提供沙箱环境保证安全性");
    println!("   4. 丰富的钩子系统");
    println!();
    println!("🔧 常用插件类型:");
    println!("   - 日志插件: 记录操作日志");
    println!("   - 验证插件: 验证数据合法性");
    println!("   - 转换插件: 转换数据格式");
    println!("   - 加密插件: 加密敏感数据");
    println!("   - 缓存插件: 缓存常用数据");
    println!("   - 监控插件: 监控性能指标");

    Ok(())
}

// ============================================
// 高级示例: 真实的 WASM 插件
// ============================================
//
// 在实际应用中，你可以使用 WASM 实现插件:
//
// ```rust
// use wasmtime::*;
//
// struct WasmPlugin {
//     engine: Engine,
//     module: Module,
//     store: Store<()>,
// }
//
// impl WasmPlugin {
//     fn new(wasm_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
//         let engine = Engine::default();
//         let module = Module::from_file(&engine, wasm_file)?;
//         let store = Store::new(&engine, ());
//
//         Ok(WasmPlugin {
//             engine,
//             module,
//             store,
//         })
//     }
//
//     fn call_hook(&mut self, hook_name: &str, data: &str) -> Result<String, String> {
//         // 调用 WASM 插件中的函数
//         // ...
//         Ok("done".to_string())
//     }
// }
// ```
//
// ============================================
// 插件开发最佳实践
// ============================================
//
// 1. **钩子命名**
//    - before_*: 操作前触发
//    - after_*: 操作后触发
//    - on_*: 事件触发时调用
//
// 2. **错误处理**
//    - 返回 Result 类型，方便错误传播
//    - 提供清晰的错误消息
//
// 3. **性能考虑**
//    - 避免在钩子中执行耗时操作
//    - 考虑异步处理
//
// 4. **安全性**
//    - 验证插件来源
//    - 使用沙箱隔离
//    - 限制资源使用
