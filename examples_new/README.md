# AgentMem 示例代码集合

这是一套完整的 AgentMem 示例代码，涵盖所有常见使用场景。每个示例都是完整可运行的代码，包含详细注释和预期输出。

## 📁 目录结构

```
examples_new/
├── README.md                    # 本文件
├── rust/                        # Rust 示例 (6个)
│   ├── quick_start.rs           # 5分钟快速开始
│   ├── memory_management.rs     # 完整 CRUD 操作
│   ├── semantic_search.rs       # 所有搜索方式
│   ├── chatbot.rs               # 聊天机器人
│   ├── multimodal.rs            # 多模态处理
│   └── plugin.rs                # WASM 插件开发
└── python/                      # Python 示例 (6个)
    ├── quick_start.py           # 快速开始
    ├── chatbot.py               # 聊天机器人
    ├── personal_assistant.py    # 个人助理
    ├── rag_qa.py                # RAG 问答系统
    ├── multimodal_search.py     # 多模态搜索
    └── webhook_server.py        # Webhook 服务器
```

## 🚀 快速开始

### Rust 示例

1. **设置环境变量**
```bash
export OPENAI_API_KEY=sk-...  # 或 ANTHROPIC_API_KEY
```

2. **运行示例**
```bash
# 将示例文件复制到 examples/ 目录
cp examples_new/rust/quick_start.rs examples/
cp examples_new/rust/memory_management.rs examples/

# 运行示例
cargo run --example quick_start
cargo run --example memory_management
cargo run --example semantic_search
```

### Python 示例

1. **安装依赖**
```bash
pip install agentmem
# 或者使用本地版本
cd sdks/python && pip install -e .
```

2. **设置环境变量**
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key
```

3. **运行示例**
```bash
cd examples_new/python
python quick_start.py
python chatbot.py
python rag_qa.py
```

## 📚 Rust 示例详解

### 1. quick_start.rs - 5分钟快速开始 ⭐

**场景**: 新手入门，学习基本用法

**功能**:
- 零配置初始化
- 添加记忆
- 语义搜索
- 显示结果

**运行**:
```bash
cargo run --example quick_start
```

**代码量**: ~100 行
**难度**: ⭐ (初级)
**预计时间**: 5 分钟

---

### 2. memory_management.rs - 完整 CRUD 操作

**场景**: 需要完整的数据管理功能

**功能**:
- 创建 (Create): 添加新记忆
- 读取 (Read): 获取单个/所有记忆
- 更新 (Update): 修改现有记忆
- 删除 (Delete): 删除记忆
- 批量操作: 批量添加和删除
- 会话管理: 使用 Session 组织记忆

**运行**:
```bash
cargo run --example memory_management
```

**代码量**: ~200 行
**难度**: ⭐⭐ (初级)
**预计时间**: 15 分钟

**关键代码**:
```rust
// 创建
let mem1 = mem.add("学习 Rust").await?;

// 读取
let memory = mem.get(&mem1.id).await?;

// 更新
let updated = mem.update(&mem1.id, "精通 Rust 编程").await?;

// 删除
mem.delete(&mem1.id).await?;
```

---

### 3. semantic_search.rs - 所有搜索方式

**场景**: 需要灵活的搜索功能

**功能**:
- 向量搜索（Vector Search）: 基于语义相似度
- BM25 搜索: 基于关键词匹配
- 混合搜索（Hybrid Search）: 结合向量和 BM25
- 过滤搜索: 按元数据过滤
- 排序选项: 按相关性和时间排序

**运行**:
```bash
cargo run --example semantic_search
```

**代码量**: ~250 行
**难度**: ⭐⭐ (中级)
**预计时间**: 20 分钟

---

### 4. chatbot.rs - 聊天机器人

**场景**: 构建对话式 AI 应用

**功能**:
- 对话历史管理
- 上下文检索
- 个性化回复
- 多轮对话
- 交互式聊天界面

**运行**:
```bash
cargo run --example chatbot
```

**代码量**: ~200 行
**难度**: ⭐⭐⭐ (中级)
**预计时间**: 25 分钟

---

### 5. multimodal.rs - 多模态处理

**场景**: 处理图像、音频、文本等多种数据

**功能**:
- 图像记忆（通过描述）
- 音频记忆（通过转录）
- 文本记忆
- 跨模态搜索
- 多模态分类

**运行**:
```bash
cargo run --example multimodal
```

**代码量**: ~250 行
**难度**: ⭐⭐⭐ (中级)
**预计时间**: 30 分钟

---

### 6. plugin.rs - WASM 插件开发

**场景**: 扩展 AgentMem 功能

**功能**:
- 创建简单插件
- 注册钩子函数
- 测试插件功能
- 热插拔插件

**运行**:
```bash
cargo run --example plugin
```

**代码量**: ~300 行
**难度**: ⭐⭐⭐⭐ (高级)
**预计时间**: 40 分钟

## 📚 Python 示例详解

### 7. quick_start.py - 快速开始 ⭐

**场景**: Python 开发者快速上手

**功能**:
- 初始化客户端
- 添加记忆
- 语义搜索
- 显示结果

**运行**:
```bash
python quick_start.py
```

**代码量**: ~150 行
**难度**: ⭐ (初级)
**预计时间**: 5 分钟

---

### 8. chatbot.py - 聊天机器人

**场景**: 构建 Python 聊天机器人

**功能**:
- 对话历史管理
- 上下文检索
- 个性化回复
- 多轮对话
- 交互式聊天

**运行**:
```bash
python chatbot.py
```

**代码量**: ~250 行
**难度**: ⭐⭐ (中级)
**预计时间**: 20 分钟

---

### 9. personal_assistant.py - 个人助理

**场景**: 构建智能个人助理

**功能**:
- 任务管理
- 日程安排
- 信息检索
- 个性化建议
- 智能问答

**运行**:
```bash
python personal_assistant.py
```

**代码量**: ~300 行
**难度**: ⭐⭐⭐ (中级)
**预计时间**: 25 分钟

---

### 10. rag_qa.py - RAG 问答系统

**场景**: 构建检索增强生成系统

**功能**:
- 文档索引
- 语义检索
- 上下文增强生成
- 答案生成
- 多轮问答

**运行**:
```bash
python rag_qa.py
```

**代码量**: ~350 行
**难度**: ⭐⭐⭐ (中级)
**预计时间**: 30 分钟

---

### 11. multimodal_search.py - 多模态搜索

**场景**: 处理和搜索多种类型数据

**功能**:
- 图像描述搜索
- 音频转录搜索
- 文档搜索
- 跨模态检索
- 类型过滤

**运行**:
```bash
python multimodal_search.py
```

**代码量**: ~350 行
**难度**: ⭐⭐⭐ (中级)
**预计时间**: 30 分钟

---

### 12. webhook_server.py - Webhook 服务器

**场景**: 构建事件驱动的应用

**功能**:
- 接收 Webhook 事件
- 处理不同类型的事件
- 与 AgentMem 集成
- 返回响应

**运行**:
```bash
python webhook_server.py
```

**代码量**: ~400 行
**难度**: ⭐⭐⭐⭐ (高级)
**预计时间**: 35 分钟

## 🎯 按场景选择示例

### 新手入门
- **Rust**: `quick_start.rs`
- **Python**: `quick_start.py`

### 数据管理
- **Rust**: `memory_management.rs`
- **Python**: `personal_assistant.py`

### 搜索功能
- **Rust**: `semantic_search.rs`
- **Python**: `rag_qa.py`

### 对话应用
- **Rust**: `chatbot.rs`
- **Python**: `chatbot.py`

### 多模态应用
- **Rust**: `multimodal.rs`
- **Python**: `multimodal_search.py`

### 高级功能
- **Rust**: `plugin.rs`
- **Python**: `webhook_server.py`

## 📖 学习路径

### 初级（1-2天）
1. ✅ 运行 `quick_start` 了解基本概念
2. ✅ 运行 `memory_management` 学习 CRUD
3. ✅ 运行 `semantic_search` 学习搜索

**目标**: 掌握 AgentMem 的基本用法

### 中级（3-5天）
4. ✅ 运行 `chatbot` 构建对话应用
5. ✅ 运行 `personal_assistant` 学习实际应用
6. ✅ 运行 `multimodal` 了解多模态处理

**目标**: 能够构建实际应用

### 高级（1-2周）
7. ✅ 运行 `rag_qa` 学习 RAG 系统
8. ✅ 运行 `plugin` 学习插件开发
9. ✅ 运行 `webhook_server` 学习集成

**目标**: 掌握高级功能和生产部署

## 🔧 通用要求

### Rust 示例
- Rust 1.70+
- Cargo
- OpenAI API Key 或 Anthropic API Key

### Python 示例
- Python 3.8+
- agentmem SDK
- AgentMem 服务器运行中
- OpenAI API Key（可选，用于 LLM 功能）

## 📝 代码质量

所有示例都遵循：
- ✅ 完整可运行（不是伪代码）
- ✅ 详细注释（解释每一步）
- ✅ 预期输出（知道会看到什么）
- ✅ 错误处理（生产级代码）
- ✅ 最佳实践（遵循语言规范）
- ✅ 文档齐全（README + 代码注释）

## 💡 使用技巧

### 1. 循序渐进
按照学习路径，从初级到高级逐步学习。

### 2. 动手实践
不要只看代码，一定要运行起来试试。

### 3. 修改实验
在示例基础上修改，看看效果变化。

### 4. 查看日志
遇到问题时，查看详细的错误日志。

### 5. 阅读文档
结合 API 文档理解每个函数的用法。

## 🆘 遇到问题？

### 常见问题

**Q: Rust 示例编译失败？**
A: 确保使用 Rust 1.70+，运行 `rustc --version` 检查版本。

**Q: Python 示例连接失败？**
A: 确保 AgentMem 服务器正在运行，检查 `AGENTMEM_API_BASE_URL`。

**Q: API Key 错误？**
A: 检查环境变量是否正确设置：`echo $OPENAI_API_KEY`

**Q: 找不到某个示例？**
A: 确保在正确的目录，使用 `ls` 查看文件列表。

## 🤝 贡献

欢迎提交问题和改进建议！

## 📄 许可证

MIT License

---

**Happy Coding! 🎉**

如有问题，请查看 [主文档](../../README.md) 或提交 Issue。
