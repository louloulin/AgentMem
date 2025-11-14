# ONNX Runtime 版本不兼容问题修复报告

**日期**: 2025-11-14  
**问题编号**: ONNX-001  
**严重程度**: 中等（警告，不影响测试通过）  
**状态**: ✅ 已修复

---

## 📋 问题描述

### 错误信息

```
WARN 创建 FastEmbed Embedder 失败: Embedding error: 
任务失败: task XX panicked with message 
"ort 2.0.0-rc.10 is not compatible with the ONNX Runtime binary 
found at `libonnxruntime.dylib`; expected GetVersionString to 
return '1.22.x', but got '1.19.2'"
```

### 影响范围

- **受影响模块**: `agent-mem-embeddings`
- **受影响功能**: FastEmbed 本地嵌入模型
- **测试影响**: 产生警告，但测试仍然通过（使用默认维度）
- **生产影响**: FastEmbed embedder 无法正常工作

### 根本原因

1. **版本不匹配**:
   - `ort` crate 版本: 2.0.0-rc.10
   - 要求的 ONNX Runtime 版本: 1.22.x
   - 实际加载的版本: 1.19.2

2. **依赖冲突**:
   - `fastembed` crate 内部依赖 `ort`
   - `fastembed` 下载了自己的 ONNX Runtime 二进制文件（1.19.2）
   - 与 `ort` 2.0.0-rc.10 的要求不兼容

3. **库加载顺序**:
   - 系统优先加载了 `fastembed` 下载的旧版本
   - 而不是项目 `lib/` 目录中的 1.22.0 版本

---

## 🔧 解决方案

### 方案概述

采用**三管齐下**的策略：
1. 升级 `fastembed` 到最新版本（支持新版 ONNX Runtime）
2. 配置 `ort` 自动下载匹配的 ONNX Runtime 版本
3. 强制不使用系统库，避免版本冲突

### 详细步骤

#### 1. 升级 fastembed 依赖

**文件**: `crates/agent-mem-embeddings/Cargo.toml`

```diff
- fastembed = { version = "5", optional = true }
+ # FastEmbed-rs 支持（推荐的本地嵌入方案）
+ # 升级到 5.2.0 以支持最新的 ONNX Runtime
+ fastembed = { version = "5.2", optional = true }
```

**原因**: fastembed 5.2.0 使用 ort 2.0.0-rc.10，与我们的配置一致。

#### 2. 配置 ort 自动下载

**文件**: `.cargo/config.toml`（新建）

```toml
# Cargo 构建配置

[env]
# 强制 ort crate 下载 ONNX Runtime 二进制文件
# 这样可以避免使用系统中不兼容的版本
ORT_STRATEGY = "download"
ORT_USE_SYSTEM_LIB = "0"

[build]
# 增加并行编译任务数
jobs = 8
```

**原因**: 
- `ORT_STRATEGY = "download"` 让 ort 自动下载匹配的 ONNX Runtime 1.22.x
- `ORT_USE_SYSTEM_LIB = "0"` 禁止使用系统库，避免版本冲突

#### 3. 启用 download-binaries 功能

**文件**: `crates/agent-mem-embeddings/Cargo.toml`

```diff
  # ONNX Runtime 支持
- ort = { version = "2.0.0-rc.10", features = ["load-dynamic"], optional = true }
+ # 使用 download-binaries 功能自动下载匹配的 ONNX Runtime 1.22.x 版本
+ ort = { version = "2.0.0-rc.10", features = ["download-binaries"], optional = true }
```

**原因**: `download-binaries` 功能让 ort 在构建时自动下载正确版本的 ONNX Runtime。

---

## ✅ 验证结果

### 测试执行

```bash
# 清理构建缓存
$ rm -rf target/release/deps/*agent_mem* target/release/deps/*fastembed* target/release/deps/*ort*

# 运行完整测试套件
$ cargo test --workspace --lib --release 2>&1 | tee test_output_fixed.log
```

### 测试结果

| 指标 | 结果 | 状态 |
|------|------|------|
| **总测试数** | 1,389 | ✅ |
| **通过** | 1,333 | ✅ |
| **失败** | 0 | ✅ |
| **忽略** | 56 | ⚠️ |
| **通过率** | 100% | ✅ |
| **ONNX 警告** | 0 | ✅ |

### 关键改进

1. **✅ 无 ONNX Runtime 版本警告**
   - 之前: 多个 WARN 消息
   - 现在: 0 个警告

2. **✅ FastEmbed embedder 正常工作**
   - 之前: 创建失败，使用默认维度
   - 现在: 正常创建和使用

3. **✅ 所有测试通过**
   - 1,333 个测试全部通过
   - 0 个失败

4. **✅ 测试执行稳定**
   - 无 panic 或崩溃
   - 无版本不兼容警告

### 日志验证

```bash
# 检查是否还有 ONNX 警告
$ grep -i "onnx.*compat\|ort.*compat" test_output_fixed.log
# 结果: 无匹配（说明警告已消除）

# 检查测试结果
$ grep "test result:" test_output_fixed.log | \
  awk '{passed+=$4; failed+=$6; ignored+=$8} END {
    print "总计: " passed " 通过, " failed " 失败, " ignored " 忽略"
  }'
# 结果: 总计: 1333 通过, 0 失败, 56 忽略
```

---

## 📝 修改文件清单

| 文件 | 类型 | 修改内容 |
|------|------|---------|
| `crates/agent-mem-embeddings/Cargo.toml` | 修改 | 升级 fastembed 5.0→5.2.0, 配置 ort features |
| `.cargo/config.toml` | 新建 | 配置 ort 环境变量 |
| `agentmem93.md` | 更新 | 添加修复记录 |
| `docs/fixes/onnx-runtime-fix-2025-11-14.md` | 新建 | 本修复报告 |

---

## 🎯 经验教训

### 问题根源

1. **依赖版本管理**:
   - 间接依赖（fastembed → ort）的版本可能与直接依赖不一致
   - 需要确保整个依赖树的版本兼容性

2. **二进制库加载**:
   - 动态库加载顺序可能导致版本冲突
   - 需要明确控制库的来源和版本

3. **构建配置**:
   - Cargo 环境变量可以有效控制依赖行为
   - `.cargo/config.toml` 是项目级配置的最佳实践

### 最佳实践

1. **✅ 使用 download-binaries**:
   - 对于 ONNX Runtime 等二进制依赖，优先使用自动下载
   - 避免依赖系统库，确保版本一致性

2. **✅ 定期更新依赖**:
   - 及时升级到最新稳定版本
   - 关注依赖的 changelog 和兼容性说明

3. **✅ 配置环境变量**:
   - 使用 `.cargo/config.toml` 统一管理构建配置
   - 明确指定依赖策略（download vs system）

4. **✅ 完整测试验证**:
   - 修复后运行完整测试套件
   - 检查日志确认警告已消除

---

## 🚀 后续建议

### 短期（已完成）

- ✅ 升级 fastembed 到 5.2.0
- ✅ 配置 ort 自动下载
- ✅ 验证所有测试通过
- ✅ 更新文档

### 中期（可选）

- ⚠️ 监控 fastembed 和 ort 的新版本
- ⚠️ 考虑固定 ONNX Runtime 版本（如果需要稳定性）
- ⚠️ 添加 CI 检查，确保无版本警告

### 长期（规划）

- 📋 评估其他 embedding 提供商（OpenAI, Cohere）
- 📋 考虑支持多个 ONNX Runtime 版本
- 📋 优化 embedding 性能和缓存策略

---

## 📚 参考资料

- [ort crate 文档](https://docs.rs/ort/)
- [fastembed crate 文档](https://docs.rs/fastembed/)
- [ONNX Runtime 发布说明](https://github.com/microsoft/onnxruntime/releases)
- [Cargo 配置文档](https://doc.rust-lang.org/cargo/reference/config.html)

---

**修复完成时间**: 2025-11-14  
**验证人**: AgentMem 开发团队  
**状态**: ✅ 已验证并合并

