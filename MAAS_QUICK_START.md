# 华为 MaaS 快速开始指南

## 🚀 5 分钟快速上手

### 步骤 1: 设置环境变量

```bash
export MAAS_API_KEY="your_maas_api_key_here"
```

### 步骤 2: 运行示例

```bash
cd lumosai/lumosai_core
cargo run --example huawei_maas_agent
```

就这么简单！✨

---

## 💡 最简单的使用方式

### 代码示例 (完整可运行)

```rust
use lumosai_core::agent::AgentBuilder;
use lumosai_core::llm::providers;
use std::sync::Arc;

#[tokio::main]
async fn main() -> lumosai_core::Result<()> {
    // 1. 自动选择可用的 LLM (包括华为 MaaS)
    let llm = providers::auto_provider()?;
    
    // 2. 创建 Agent
    let agent = AgentBuilder::new()
        .name("my_assistant")
        .instructions("你是一个智能助手")
        .model(Arc::new(llm))
        .build()?;
    
    // 3. 开始对话
    let response = agent.generate("你好", &Default::default()).await?;
    println!("AI: {}", response);
    
    Ok(())
}
```

### 保存为文件并运行

1. 将上述代码保存为 `my_agent.rs`
2. 在 `Cargo.toml` 添加依赖:
```toml
[dependencies]
lumosai_core = { path = "lumosai/lumosai_core" }
tokio = { version = "1", features = ["full"] }
```
3. 运行: `cargo run`

---

## 🎯 三种使用方式对比

### 方式 1: auto_provider (推荐 ⭐)

**优点**: 最简单，自动检测可用的 LLM  
**适用**: 不关心具体用哪个 LLM

```rust
let llm = providers::auto_provider()?;
```

### 方式 2: 指定 MaaS Provider

**优点**: 明确使用华为 MaaS  
**适用**: 确定要用 MaaS 服务

```rust
let llm = providers::huawei_maas_from_env()?;
```

### 方式 3: 完全手动配置

**优点**: 完全控制所有参数  
**适用**: 需要自定义配置

```rust
use lumosai_core::llm::HuaweiMaasProvider;

let llm = HuaweiMaasProvider::new(
    "your_api_key".to_string(),
    Some("deepseek-v3.2-exp".to_string())
);
```

---

## 📝 常用场景示例

### 场景 1: 简单问答

```rust
let response = agent.generate("什么是 Rust?", &Default::default()).await?;
println!("{}", response);
```

### 场景 2: 多轮对话

```rust
use lumosai_core::llm::{Message, Role};

let messages = vec![
    Message {
        role: Role::System,
        content: "你是技术顾问".to_string(),
        metadata: None,
        name: None,
    },
    Message {
        role: Role::User,
        content: "云计算的优势?".to_string(),
        metadata: None,
        name: None,
    },
];

let response = agent.generate_with_messages(&messages, &Default::default()).await?;
```

### 场景 3: 流式输出

```rust
use futures::StreamExt;

let mut stream = agent.generate_stream("讲个故事", &Default::default()).await?;

while let Some(chunk) = stream.next().await {
    if let Ok(text) = chunk {
        print!("{}", text);
    }
}
```

### 场景 4: 自定义温度和长度

```rust
use lumosai_core::llm::LlmOptions;

let options = LlmOptions::default()
    .with_temperature(0.9)  // 更有创意
    .with_max_tokens(2000); // 更长回复

let response = agent.generate("写一首诗", &options).await?;
```

---

## ⚙️ 配置选项

### 环境变量

```bash
# 必需
export MAAS_API_KEY="sk-xxxxx"

# 可选
export MAAS_MODEL="deepseek-v3.2-exp"  # 默认模型
```

### AgentBuilder 配置

```rust
let agent = AgentBuilder::new()
    .name("assistant")              // Agent 名称
    .instructions("你是助手")       // 系统提示
    .model(Arc::new(llm))           // LLM provider
    .temperature(0.7)               // 温度 0.0-2.0
    .max_tokens(1000)               // 最大 tokens
    .max_tool_calls(5)              // 最大工具调用次数
    .tool_timeout(60)               // 工具超时秒数
    .build()?;
```

---

## 🔧 故障排查

### 问题: "MAAS_API_KEY not set"

```bash
# 解决: 设置环境变量
export MAAS_API_KEY="your_key"
```

### 问题: "API 请求失败"

检查清单:
- ✅ API Key 正确
- ✅ 网络连接正常
- ✅ 模型名称正确
- ✅ API 端点可访问

### 问题: "编译错误"

```bash
# 清理重新编译
cargo clean
cargo build
```

---

## 📦 项目结构

```
agentmen/
├── lumosai/
│   └── lumosai_core/
│       ├── src/llm/
│       │   ├── huawei_maas.rs      # MaaS Provider 实现
│       │   ├── providers.rs         # Provider 工厂函数
│       │   └── mod.rs               # 模块导出
│       └── examples/
│           └── huawei_maas_agent.rs # 完整示例
├── docs/
│   └── HUAWEI_MAAS_INTEGRATION.md   # 详细文档
└── scripts/
    └── test_maas_integration.sh     # 测试脚本
```

---

## 🎓 学习路径

### 初学者 (10 分钟)
1. ✅ 设置环境变量
2. ✅ 运行示例: `cargo run --example huawei_maas_agent`
3. ✅ 理解基本用法

### 进阶使用 (30 分钟)
1. ✅ 创建自己的 Agent
2. ✅ 尝试不同配置
3. ✅ 实现流式响应

### 深入研究 (1-2 小时)
1. ✅ 阅读完整文档: `docs/HUAWEI_MAAS_INTEGRATION.md`
2. ✅ 查看源码: `lumosai/lumosai_core/src/llm/huawei_maas.rs`
3. ✅ 实现自定义功能

---

## 🔗 相关链接

### 文档
- [完整集成指南](./docs/HUAWEI_MAAS_INTEGRATION.md)
- [实现总结](./MAAS_IMPLEMENTATION_SUMMARY.md)

### 代码
- [MaaS Provider 源码](./lumosai/lumosai_core/src/llm/huawei_maas.rs)
- [完整示例](./lumosai/lumosai_core/examples/huawei_maas_agent.rs)

### 工具
- [测试脚本](./scripts/test_maas_integration.sh)

---

## ❓ 常见问题

**Q: 需要付费吗？**  
A: 华为 MaaS 是付费服务，需要华为云账号和 API Key。

**Q: 支持哪些模型？**  
A: 目前支持 `deepseek-v3.2-exp` 等，具体见华为云文档。

**Q: 可以用于生产环境吗？**  
A: 可以，但请注意 API 限流和成本控制。

**Q: 如何切换到其他 LLM？**  
A: 只需更换环境变量或 provider，代码无需修改：
```bash
# 切换到 OpenAI
export OPENAI_API_KEY="sk-xxxx"

# 切换到智谱
export ZHIPU_API_KEY="xxxx"
```

**Q: 支持流式输出吗？**  
A: 支持！使用 `agent.generate_stream()` 即可。

---

## 🎉 下一步

现在你已经掌握了基础，可以：

1. **构建实际应用** - 集成到你的项目中
2. **探索高级功能** - Function Calling、Memory 等
3. **优化性能** - 调整参数、使用缓存
4. **参与贡献** - 提 Issue、PR，改进功能

---

**祝你使用愉快！** 🚀

有问题随时查看[完整文档](./docs/HUAWEI_MAAS_INTEGRATION.md)或提交 Issue。
