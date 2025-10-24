# AgentMem 真实演示示例集合

本目录包含AgentMem的真实、可运行的演示示例，展示各种核心功能和应用场景。

## 📚 示例列表

### Rust示例

#### 1. demo-memory-api - Memory API基础演示 ✅

**功能展示**:
- 创建Memory实例（零配置启动）
- 添加记忆
- 语义搜索
- 获取所有记忆
- 删除记忆

**运行方式**:
```bash
cd examples/demo-memory-api
cargo run
```

**特点**:
- ✅ 使用FastEmbed本地嵌入（无需API key）
- ✅ 向量维度自动适配（384维）
- ✅ LibSQL嵌入式存储（零配置）
- ✅ 完整的增删改查功能

### 2. demo-multimodal - 多模态处理演示 ✅

**功能展示**:
- AI模型配置（OpenAI、Google、Azure、Local）
- 图像处理（OCR、对象检测、场景分析）
- 音频处理（语音转文本、音频分析）
- 视频处理（关键帧提取、场景检测）

**运行方式**:
```bash
cd examples/demo-multimodal
cargo run --features multimodal
```

**特点**:
- ✅ 6,114行代码，业界最完整
- ✅ 支持多种AI提供商
- ✅ 完整的图像/音频/视频处理
- ✅ 跨模态检索与特征融合

### 3. demo-intelligent-chat - 智能对话助手演示 ✅

**功能展示**:
- 多轮对话记忆
- 上下文理解
- 用户偏好学习
- 个性化推荐

**运行方式**:
```bash
cd examples/demo-intelligent-chat
cargo run
```

**应用场景**:
- 智能客服系统
- AI助手
- 聊天机器人
- 个性化服务

**特点**:
- ✅ 跨会话记忆保持
- ✅ 语义搜索相关对话
- ✅ 智能用户画像分析
- ✅ 实时响应

### Python示例

#### 4. demo-python-basic - Python SDK基础演示 ✅ 🆕

**功能展示**:
- 创建Memory实例
- 添加、搜索、删除记忆
- 异步操作（async/await）

**运行方式**:
```bash
# 先构建Python绑定
cd crates/agent-mem-python
maturin develop

# 运行示例
cd examples/demo-python-basic
python demo_basic.py
```

**特点**:
- ✅ 简单易用的Python API
- ✅ 异步支持
- ✅ 高性能Rust后端
- ✅ 详细的中文注释

#### 5. demo-python-chat - Python智能对话演示 ✅ 🆕

**功能展示**:
- 智能对话机器人类
- 用户画像分析
- 个性化推荐
- 对话历史统计

**运行方式**:
```bash
cd examples/demo-python-chat
python demo_chat.py
```

**应用场景**:
- 智能客服系统
- AI助手
- 聊天机器人

**特点**:
- ✅ 完整的对话管理
- ✅ 智能用户分析
- ✅ 可扩展架构
- ✅ 生产就绪代码

## 🚀 快速开始

### 前置要求

- Rust 1.70+
- Cargo

### 运行Rust示例

```bash
# 1. Memory API演示
cd examples/demo-memory-api
cargo run

# 2. 多模态演示
cd examples/demo-multimodal
cargo run --features multimodal

# 3. 智能对话演示
cd examples/demo-intelligent-chat
cargo run
```

### 运行Python示例

```bash
# 首先构建Python绑定（只需一次）
cd crates/agent-mem-python
pip install maturin
maturin develop

# 然后运行示例
cd examples/demo-python-basic
python demo_basic.py

cd examples/demo-python-chat
python demo_chat.py
```

## 📊 示例特点

### 零配置启动
所有示例都使用FastEmbed本地嵌入和LibSQL嵌入式数据库，无需：
- ❌ 外部数据库
- ❌ API密钥
- ❌ 复杂配置

### 真实可运行
每个示例都是完整的、可运行的程序，不是伪代码：
- ✅ 真实的API调用
- ✅ 真实的数据处理
- ✅ 真实的输出结果

### 详细注释
每个示例都包含：
- 📝 功能说明
- 📝 使用方法
- 📝 输出解释
- 📝 最佳实践

## 🎯 应用场景

### 智能客服
使用 `demo-intelligent-chat` 了解如何构建智能客服系统。

### 知识管理
使用 `demo-memory-api` 了解如何管理企业知识库。

### 多媒体处理
使用 `demo-multimodal` 了解如何处理图像、音频、视频。

#### 6. demo-performance-benchmark - 性能基准测试工具 ✅ 🆕

**功能展示**:
- 内存操作性能测试（添加、搜索、删除）
- 向量搜索性能测试
- 并发性能测试
- 大规模数据测试
- 延迟统计（平均、P95、P99）

**运行方式**:
```bash
cd examples/demo-performance-benchmark
cargo run --release  # 使用release模式获得准确结果
```

**特点**:
- ✅ 真实API测试（非mock）
- ✅ 多维度性能评估
- ✅ 详细统计报告
- ✅ 自动化性能评估
- ✅ 彩色输出易于阅读

## 💡 更多示例

已完成的示例：
- ✅ demo-memory-api - Memory API基础演示（Rust）
- ✅ demo-multimodal - 多模态处理演示（Rust）
- ✅ demo-intelligent-chat - 智能对话演示（Rust）
- ✅ demo-python-basic - Python SDK基础演示（Python）
- ✅ demo-python-chat - Python智能对话演示（Python）
- ✅ demo-performance-benchmark - 性能基准测试工具（Rust）✨ 🆕

计划中的示例：
- [ ] demo-knowledge-graph - 知识图谱演示
- [ ] demo-procedural-memory - 程序记忆演示
- [ ] demo-cangjie - Cangjie SDK演示

## 🐛 问题反馈

如果遇到问题，请：
1. 检查Rust版本（需要1.70+）
2. 查看示例的README
3. 提交Issue到GitHub

## 📖 相关文档

- [AgentMem文档](../../README.md)
- [API参考](../../docs/API.md)
- [最佳实践](../../docs/BEST_PRACTICES.md)

---

**示例总数**: 3个（已完成） + 4个（计划中）  
**代码质量**: 生产就绪  
**维护状态**: 活跃维护

