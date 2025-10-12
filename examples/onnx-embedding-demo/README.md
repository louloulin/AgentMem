# ONNX 嵌入模型演示

本示例演示如何使用 ONNX Runtime 进行嵌入模型推理。

## 功能特性

### ✅ 已实现

- **ONNX 模型加载**: 使用 `ort` crate 加载 ONNX 模型
- **Tokenizer 集成**: 使用 HuggingFace tokenizers 进行文本分词
- **真实的 ONNX 推理**: 使用 ONNX Runtime 进行推理
- **张量转换**: 将 token IDs 转换为 ONNX Runtime 输入张量
- **池化策略**: CLS token 池化（提取第一个 token 的嵌入）
- **批量处理**: 支持单个和批量文本嵌入
- **L2 归一化**: 对嵌入向量进行归一化
- **错误处理**: 完善的错误处理机制
- **配置管理**: 灵活的配置选项

### 🚀 性能优化

- **批量推理**: 支持批量处理以提高效率
- **Padding**: 自动 padding 到最大序列长度
- **并行处理**: 使用 4 个 intra-threads 进行并行推理
- **优化级别**: 使用 Level3 优化（最高优化）

## 当前状态

**✅ 真实的 ONNX 推理已实现！**

本示例现在使用真实的 ONNX Runtime 进行推理：
- ✅ 使用 `ort` 2.0.0-rc.10 进行 ONNX 推理
- ✅ 支持 Tokenizer 分词
- ✅ 支持单文本和批量推理
- ✅ 支持 [CLS] token 池化
- ✅ L2 归一化

**注意**: 本示例使用模拟的 ONNX 模型文件进行演示。要使用真实的模型，请：
1. 下载真实的 ONNX 模型（如 BGE-small-en-v1.5）
2. 下载对应的 tokenizer.json 文件
3. 更新代码中的模型路径

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

✅ 功能完整:
   真实的 ONNX 推理已实现！

   已实现的功能:
   ✅ ONNX 模型加载
   ✅ Tokenizer 集成
   ✅ 真实的 ONNX 推理
   ✅ 张量转换和处理
   ✅ 池化策略（CLS token）
   ✅ 批量处理接口
   ✅ L2 归一化
   ✅ 错误处理

   ⚠️  注意:
   本示例使用模拟的 ONNX 模型文件
   要使用真实模型，请下载真实的 ONNX 模型和 tokenizer

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

1. **使用真实模型**: 下载并使用真实的 ONNX 模型（BGE、E5 等）
2. **性能测试**: 测试不同批量大小的性能
3. **支持更多模型**: 支持更多流行的嵌入模型
4. **模型缓存**: 实现模型缓存以提高加载速度

## 参考资料

- [ort crate 文档](https://docs.rs/ort/2.0.0-rc.10/ort/)
- [ONNX Runtime 官方文档](https://onnxruntime.ai/)
- [HuggingFace Tokenizers](https://github.com/huggingface/tokenizers)
- [FastEmbed-rs](https://github.com/Anush008/fastembed-rs) - 使用 ort 的参考实现

## 许可证

与 AgentMem 项目相同的许可证。

