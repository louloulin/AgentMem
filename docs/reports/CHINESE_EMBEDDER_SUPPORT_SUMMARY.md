# 支持中文的Embedder模型总结

## 📋 当前代码库中支持中文的Embedder模型

### 1. FastEmbed多语言模型 ✅

代码库已经支持以下**多语言embedding模型**（支持中文）：

| 模型名称 | 维度 | 支持语言 | 代码位置 |
|---------|------|---------|---------|
| `multilingual-e5-small` | 384 | 100+语言（包括中文） | `crates/agent-mem-embeddings/src/factory.rs:232` |
| `multilingual-e5-base` | 768 | 100+语言（包括中文） | `crates/agent-mem-embeddings/src/factory.rs:253` |
| `multilingual-e5-large` | 1024 | 100+语言（包括中文） | `crates/agent-mem-embeddings/src/factory.rs:254` |

### 2. 代码实现位置

#### 2.1 Embedding工厂类
**文件**: `crates/agent-mem-embeddings/src/factory.rs`

```rust
// 默认创建函数（已使用多语言模型）
pub async fn create_default() -> Result<Arc<dyn Embedder + Send + Sync>> {
    let config = EmbeddingConfig {
        provider: "fastembed".to_string(),
        model: "multilingual-e5-small".to_string(), // ✅ 使用多语言模型，支持中文
        dimension: 384,
        batch_size: 256,
        ..Default::default()
    };
    Self::create_embedder(&config).await
}

// 创建FastEmbed嵌入器（支持多语言模型）
pub async fn create_fastembed(model: &str) -> Result<Arc<dyn Embedder + Send + Sync>> {
    let dimension = match model {
        "multilingual-e5-small" => 384,  // ✅ 支持
        "multilingual-e5-base" => 768,   // ✅ 支持
        "multilingual-e5-large" => 1024, // ✅ 支持
        // ... 其他模型
    };
    // ...
}
```

#### 2.2 FastEmbed提供者
**文件**: `crates/agent-mem-embeddings/src/providers/fastembed.rs`

```rust
match model {
    "multilingual-e5-small" | "intfloat/multilingual-e5-small" => {
        Ok(EmbeddingModel::MultilingualE5Small)  // ✅ 支持
    }
    "multilingual-e5-base" | "intfloat/multilingual-e5-base" => {
        Ok(EmbeddingModel::MultilingualE5Base)   // ✅ 支持
    }
    "multilingual-e5-large" | "intfloat/multilingual-e5-large" => {
        Ok(EmbeddingModel::MultilingualE5Large)  // ✅ 支持
    }
    // ...
}
```

#### 2.3 服务器路由配置
**文件**: `crates/agent-mem-server/src/routes/memory.rs:60-71`

**当前状态**（使用英文模型）：
```rust
} else {
    // 使用默认FastEmbed配置
    info!("  - Provider: fastembed (默认)");
    info!("  - Model: BAAI/bge-small-en-v1.5");  // ❌ 英文模型
    builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");
}
```

**建议修改**（使用中文模型）：
```rust
} else {
    // 使用默认FastEmbed配置（支持中文）
    info!("  - Provider: fastembed (默认)");
    info!("  - Model: multilingual-e5-small (支持中英文)");
    builder = builder.with_embedder("fastembed", "multilingual-e5-small");
}
```

### 3. 环境变量配置

**文件**: `crates/agent-mem-embeddings/src/factory.rs:355-380`

可以通过环境变量配置embedder模型：

```bash
# 设置embedder提供商
export EMBEDDING_PROVIDER=fastembed

# 设置embedder模型（支持中文）
export FASTEMBED_MODEL=multilingual-e5-small
# 或者
export FASTEMBED_MODEL=multilingual-e5-base
# 或者
export FASTEMBED_MODEL=multilingual-e5-large
```

代码会自动读取环境变量：
```rust
let model = std::env::var("FASTEMBED_MODEL")
    .unwrap_or_else(|_| "bge-small-en-v1.5".to_string()); // 默认值
```

### 4. 配置文件支持

**当前配置文件**: `config.toml` 和 `config.example.toml`

**建议添加embedder配置**：
```toml
[memory]
# Embedder provider
embedder_provider = "fastembed"

# Embedder model (支持中文)
embedder_model = "multilingual-e5-small"
```

### 5. 使用方式

#### 5.1 通过代码直接使用
```rust
use agent_mem_embeddings::EmbeddingFactory;

// 使用默认多语言模型（支持中文）
let embedder = EmbeddingFactory::create_default().await?;

// 或指定多语言模型
let embedder = EmbeddingFactory::create_fastembed("multilingual-e5-small").await?;
```

#### 5.2 通过Memory API使用
```rust
let mut builder = Memory::builder();
builder = builder.with_embedder("fastembed", "multilingual-e5-small");
let memory = builder.build().await?;
```

#### 5.3 通过环境变量使用
```bash
export FASTEMBED_MODEL=multilingual-e5-small
# 然后启动服务器
```

## 🔧 如何启用中文支持

### 方法1：修改默认配置（推荐）

修改 `crates/agent-mem-server/src/routes/memory.rs:70`：

```rust
// 修改前
builder = builder.with_embedder("fastembed", "BAAI/bge-small-en-v1.5");

// 修改后
builder = builder.with_embedder("fastembed", "multilingual-e5-small");
```

### 方法2：通过环境变量

```bash
export FASTEMBED_MODEL=multilingual-e5-small
```

### 方法3：通过配置文件

在 `config.toml` 中添加：
```toml
[memory]
embedder_provider = "fastembed"
embedder_model = "multilingual-e5-small"
```

## 📊 模型对比

| 特性 | BAAI/bge-small-en-v1.5 | multilingual-e5-small |
|------|------------------------|---------------------|
| 语言支持 | 仅英文 | 100+语言（包括中文） |
| 维度 | 384 | 384 |
| 中文质量 | ❌ 差 | ✅ 好 |
| 英文质量 | ✅ 优秀 | ✅ 良好 |
| 推荐场景 | 纯英文应用 | 中英文混合应用 |

## ✅ 总结

1. **代码库已支持中文embedder**：`multilingual-e5-small/base/large`
2. **当前默认配置使用英文模型**：需要修改为多语言模型
3. **三种配置方式**：代码修改、环境变量、配置文件
4. **推荐模型**：`multilingual-e5-small`（平衡性能和中文支持）

## 🔗 相关文件

- Embedding工厂: `crates/agent-mem-embeddings/src/factory.rs`
- FastEmbed提供者: `crates/agent-mem-embeddings/src/providers/fastembed.rs`
- 服务器路由: `crates/agent-mem-server/src/routes/memory.rs`
- 配置文件: `config.toml`, `config.example.toml`

