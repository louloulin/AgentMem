# config.toml 配置不生效问题分析

## 问题描述

`/Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/dist/server/config.toml` 配置文件不生效。

## 根本原因

### 1. 服务器启动时没有指定配置文件路径

**代码位置**: `crates/agent-mem-server/src/main.rs:59`

```rust
let mut config = match ServerConfig::load(cli.config.as_deref()) {
    // ...
}
```

**问题**：
- 服务器需要显式通过 `--config` 参数指定配置文件路径
- 如果启动脚本没有传递 `--config config.toml`，服务器不会读取配置文件
- 默认情况下，服务器只从环境变量读取配置

### 2. 环境变量优先级高于配置文件

**代码位置**: `crates/agent-mem-server/src/config.rs:130-131`

```rust
// Override with environment variables (explicit, higher priority than file)
config = config.override_from_env();
```

**问题**：
- 即使读取了配置文件，环境变量会覆盖配置文件的值
- `start-with-zhipu.sh` 中设置了 `export EMBEDDER_MODEL="multilingual-e5-small"`
- 这会覆盖 `config.toml` 中的任何 embedder 配置

### 3. config.toml 缺少 embedder 配置字段

**当前 config.toml**：
```toml
[memory]
provider = "lancedb"
path = "./data/vectors"
```

**问题**：
- `config.toml` 中没有 `embedder_provider` 和 `embedder_model` 字段
- 即使服务器读取了配置文件，也没有 embedder 配置

## 解决方案

### 方案1：修改启动脚本，传递配置文件参数（推荐）✅

修改 `start-with-zhipu.sh`，在启动命令中添加 `--config` 参数：

```bash
# 启动服务（添加配置文件参数）
./agent-mem-server --config config.toml
```

### 方案2：在 config.toml 中添加 embedder 配置

在 `config.toml` 中添加：

```toml
# Embedder 配置（支持中文）
embedder_provider = "fastembed"
embedder_model = "multilingual-e5-small"
```

### 方案3：修改启动脚本，移除环境变量（如果希望使用配置文件）

如果希望使用配置文件而不是环境变量，可以注释掉启动脚本中的环境变量设置：

```bash
# 配置 Embedder (使用 FastEmbed，支持中文)
# export EMBEDDER_PROVIDER="fastembed"
# export EMBEDDER_MODEL="multilingual-e5-small"
# 改为从配置文件读取
```

### 方案4：修改启动脚本，支持配置文件优先级（最佳）

修改启动脚本，使其支持配置文件，但允许环境变量覆盖：

```bash
# 如果存在配置文件，使用配置文件
if [ -f "config.toml" ]; then
    CONFIG_ARG="--config config.toml"
else
    CONFIG_ARG=""
fi

# 环境变量可以覆盖配置文件的值
export EMBEDDER_PROVIDER=${EMBEDDER_PROVIDER:-"fastembed"}
export EMBEDDER_MODEL=${EMBEDDER_MODEL:-"multilingual-e5-small"}

# 启动服务
./agent-mem-server $CONFIG_ARG
```

## 配置优先级说明

根据代码实现，配置优先级为：

1. **CLI 参数**（最高优先级）
2. **环境变量**（中等优先级）
3. **配置文件**（最低优先级）

这意味着：
- 如果设置了环境变量，配置文件中的值会被覆盖
- 如果设置了 CLI 参数，环境变量和配置文件都会被覆盖

## 推荐配置方式

### 对于生产环境：

1. **使用配置文件**：在 `config.toml` 中设置所有配置
2. **启动时指定配置文件**：`./agent-mem-server --config config.toml`
3. **环境变量仅用于覆盖**：只在需要临时覆盖时才设置环境变量

### 对于开发环境：

1. **使用环境变量**：在启动脚本中设置环境变量
2. **配置文件作为默认值**：配置文件提供默认值，环境变量可以覆盖

## 已修复的问题

1. ✅ 修改 `build-release.sh` 中的默认 embedder 模型为 `multilingual-e5-small`
2. ✅ 修改 `start-with-zhipu.sh` 中的 embedder 模型为 `multilingual-e5-small`（已确认）
3. ✅ 在 `config.toml` 中添加 embedder 配置字段
4. ✅ 在 `config.example.toml` 中添加 embedder 配置示例

## 下一步操作

1. **重新构建发布包**：运行 `./build-release.sh` 生成新的发布包
2. **修改启动脚本**：在 `start-with-zhipu.sh` 中添加 `--config config.toml` 参数
3. **验证配置**：启动服务器，检查日志确认使用了正确的 embedder 模型

