# ONNX 嵌入模型演示

本示例演示如何使用 ONNX Runtime 进行嵌入模型推理。

## 功能特性

### ✅ 已实现

- **ONNX 模型加载框架**: 使用 `ort` crate 加载 ONNX 模型
- **Tokenizer 集成**: 使用 HuggingFace tokenizers 进行文本分词
- **批量处理接口**: 支持单个和批量文本嵌入
- **错误处理**: 完善的错误处理机制
- **配置管理**: 灵活的配置选项

### ⏳ 待实现

- **真实的 ONNX 推理**: 等待 `ort` 2.0 API 稳定后实现
- **张量转换和处理**: 将 token IDs 转换为 ONNX Runtime 输入张量
- **池化策略**: CLS token 池化或平均池化
- **性能优化**: 批量推理优化

## 当前状态

**注意**: 当前实现使用确定性嵌入作为占位符。这是因为 `ort` 2.0.0-rc API 仍在变化中，完整的 ONNX 推理实现需要等待 API 稳定。

确定性嵌入的特点：
- 相同的输入文本总是产生相同的嵌入向量
- 使用 SHA-256 哈希生成嵌入
- 适合用于测试和开发

## 运行示例

```bash
# 从 agentmen 目录运行
cargo run --package onnx-embedding-demo --features onnx

# 或者从项目根目录运行
cd agentmen
cargo run --example onnx-embedding-demo --features onnx
```

## 示例输出

```
🚀 ONNX 嵌入模型演示
============================================================
📁 创建模拟 ONNX 模型文件:
   模型: "/tmp/.tmpXXXXXX/model.onnx"
   分词器: "/tmp/.tmpXXXXXX/tokenizer.json"

⚙️  创建 ONNX 嵌入器...
✅ ONNX 嵌入器创建成功

📝 测试单个文本嵌入:
   输入文本: "Hello, this is a test for ONNX embedding model"
   ✅ 生成嵌入向量: 384 维
   前 10 个值: [0.123, 0.456, ...]

📦 测试批量嵌入:
   输入文本数量: 3
   [1] "First text for batch embedding"
   [2] "Second text for batch embedding"
   [3] "Third text for batch embedding"
   ✅ 生成批量嵌入: 3 个向量
   [1] 384 维, 前 5 个值: [0.123, ...]
   [2] 384 维, 前 5 个值: [0.456, ...]
   [3] 384 维, 前 5 个值: [0.789, ...]

ℹ️  嵌入器信息:
   提供商: local
   维度: 384

🏥 健康检查:
   ✅ 健康检查通过

⚠️  重要说明:
   当前 ONNX 实现使用确定性嵌入作为占位符
   完整的 ONNX 推理实现需要等待 ort 2.0 API 稳定
   
   已实现的功能:
   ✅ ONNX 模型加载框架
   ✅ Tokenizer 集成
   ✅ 批量处理接口
   ✅ 错误处理
   
   待实现的功能:
   ⏳ 真实的 ONNX 推理（等待 ort 2.0 API 稳定）
   ⏳ 张量转换和处理
   ⏳ 池化策略（CLS token / 平均池化）

============================================================
✅ ONNX 嵌入模型演示完成
```

## 技术细节

### ONNX Runtime 集成

使用 `ort` crate (version 2.0.0-rc.10) 进行 ONNX Runtime 集成：

```rust
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
};

// 加载 ONNX 模型
let session = Session::builder()?
    .with_optimization_level(GraphOptimizationLevel::Level3)?
    .with_intra_threads(4)?
    .commit_from_file(model_path)?;
```

### Tokenizer 集成

使用 HuggingFace `tokenizers` crate 进行文本分词：

```rust
use tokenizers::Tokenizer;

// 加载 tokenizer
let tokenizer = Tokenizer::from_file(tokenizer_path)?;

// 分词
let encoding = tokenizer.encode(text, true)?;
let input_ids = encoding.get_ids();
let attention_mask = encoding.get_attention_mask();
```

### 批量处理

支持批量文本嵌入以提高效率：

```rust
let texts = vec![
    "First text".to_string(),
    "Second text".to_string(),
    "Third text".to_string(),
];

let embeddings = embedder.embed_batch(&texts).await?;
```

## 依赖项

- `ort` (2.0.0-rc.10): ONNX Runtime 绑定
- `tokenizers` (0.19): HuggingFace tokenizers
- `ndarray` (0.15): 多维数组（用于张量操作）

## 下一步

1. **等待 ort 2.0 API 稳定**: 关注 `ort` crate 的更新
2. **实现真实的 ONNX 推理**: 使用稳定的 API 实现完整的推理流程
3. **性能优化**: 实现批量推理优化
4. **支持更多模型**: 支持 BGE、E5 等流行的嵌入模型

## 参考资料

- [ort crate 文档](https://docs.rs/ort/2.0.0-rc.10/ort/)
- [ONNX Runtime 官方文档](https://onnxruntime.ai/)
- [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers)
- [FastEmbed-rs](https://github.com/Anush008/fastembed-rs) - 使用 ort 的参考实现

## 许可证

与 AgentMem 项目相同的许可证。

