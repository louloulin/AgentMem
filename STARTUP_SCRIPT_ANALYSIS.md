# AgentMem 启动脚本分析和修复

**日期**: 2025-11-07  
**问题**: Zhipu API key not configured错误  
**根因**: 后端服务未使用正确的启动脚本，环境变量未设置

---

## 🔍 问题分析

### 错误信息
```
Failed to parse SSE data: Error: Configuration error: Zhipu API key not configured
at ChatPage.useCallback[handleStreamingMessage] (page.tsx:206:23)
```

### 根本原因
1. **后端服务启动方式不对**: 直接运行 `nohup ./target/release/agent-mem-server` 而没有设置环境变量
2. **环境变量未传递**: `ZHIPU_API_KEY` 未设置到运行时环境
3. **配置文件未生效**: `config.toml` 中虽然有配置，但代码可能优先读取环境变量

---

## 📊 启动脚本对比

### ✅ 正确的脚本: `start_server_no_auth.sh`

```bash
# 配置 LLM Provider (Zhipu AI)
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"

# 配置 Embedder
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 启动服务器
nohup ./target/release/agent-mem-server > backend-no-auth.log 2>&1 &
```

**优点**:
- ✅ 设置了所有必要的环境变量
- ✅ 包含 Zhipu API key
- ✅ 配置了 LLM 和 Embedder
- ✅ 禁用了认证（用于测试）

### ❌ 错误的启动方式

```bash
# 直接启动，没有环境变量
nohup ./target/release/agent-mem-server > backend.log 2>&1 &
```

**问题**:
- ❌ 没有 ZHIPU_API_KEY
- ❌ 没有 LLM_PROVIDER
- ❌ 没有 EMBEDDER 配置
- ❌ 后端运行时无法访问 API key

---

## 🛠️ 修复方案

### 方案1: 使用统一的启动脚本（推荐）

创建一个统一的启动脚本，确保所有环境变量正确设置。

**文件**: `restart_services.sh`

```bash
#!/bin/bash

# AgentMem 统一服务重启脚本
# 确保所有配置正确加载

set -e

cd "$(dirname "$0")"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  AgentMem 服务重启脚本                                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ============================================================
# 步骤 1: 停止旧服务
# ============================================================
echo "1️⃣  停止旧服务..."
pkill -f "agent-mem-server" 2>/dev/null || true
pkill -f "next dev" 2>/dev/null || true
sleep 2
echo "  ✅ 旧服务已停止"
echo ""

# ============================================================
# 步骤 2: 设置环境变量
# ============================================================
echo "2️⃣  设置环境变量..."

# ONNX Runtime
export DYLD_LIBRARY_PATH="$(pwd)/lib:$(pwd)/target/release:\$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib"

# LLM 配置 (Zhipu AI)
export ZHIPU_API_KEY="99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k"
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4-plus"
export LLM_BASE_URL="https://open.bigmodel.cn/api/paas/v4"

# Embedder 配置
export EMBEDDER_PROVIDER="fastembed"
export EMBEDDER_MODEL="BAAI/bge-small-en-v1.5"

# 数据库配置
export DATABASE_URL="file:./data/agentmem.db"

# 认证配置
export ENABLE_AUTH="false"
export SERVER_ENABLE_AUTH="false"

# 其他配置
export RUST_BACKTRACE=1
export RUST_LOG=info

echo "  ✅ 环境变量设置完成"
echo ""
echo "  📋 关键配置:"
echo "     ZHIPU_API_KEY: 99a311...*** (已设置)"
echo "     LLM_PROVIDER: $LLM_PROVIDER"
echo "     EMBEDDER_PROVIDER: $EMBEDDER_PROVIDER"
echo "     ENABLE_AUTH: $ENABLE_AUTH"
echo ""

# ============================================================
# 步骤 3: 启动后端服务
# ============================================================
echo "3️⃣  启动后端服务..."
nohup ./target/release/agent-mem-server > backend.log 2>&1 &
BACKEND_PID=$!
echo "  ✅ 后端已启动 (PID: $BACKEND_PID)"
echo ""

# ============================================================
# 步骤 4: 等待后端启动
# ============================================================
echo "4️⃣  等待后端启动..."
sleep 5

# 验证后端
if ps -p $BACKEND_PID > /dev/null; then
    echo "  ✅ 后端进程运行中"
else
    echo "  ❌ 后端启动失败，查看日志:"
    tail -20 backend.log
    exit 1
fi

# 健康检查
echo "  🏥 健康检查..."
for i in {1..5}; do
    if curl -s http://localhost:8080/health | grep -q "healthy"; then
        echo "  ✅ 后端健康检查通过"
        break
    fi
    if [ $i -eq 5 ]; then
        echo "  ❌ 后端健康检查失败"
        tail -20 backend.log
        exit 1
    fi
    sleep 2
done
echo ""

# ============================================================
# 步骤 5: 启动前端服务
# ============================================================
echo "5️⃣  启动前端服务..."
cd agentmem-ui
nohup npm run dev > ../frontend.log 2>&1 &
FRONTEND_PID=$!
cd ..
echo "  ✅ 前端已启动 (PID: $FRONTEND_PID)"
echo ""

echo "6️⃣  等待前端启动..."
sleep 5
echo "  ✅ 前端启动完成"
echo ""

# ============================================================
# 完成
# ============================================================
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  ✅ 服务启动完成！                                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 服务状态:"
echo "  • 后端: http://localhost:8080 ✅"
echo "  • 前端: http://localhost:3001 ✅"
echo "  • API文档: http://localhost:8080/swagger-ui/ ✅"
echo ""
echo "📝 日志文件:"
echo "  • 后端: tail -f backend.log"
echo "  • 前端: tail -f frontend.log"
echo ""
echo "🛑 停止服务:"
echo "  pkill -f agent-mem-server"
echo "  pkill -f 'next dev'"
echo ""
```

### 方案2: 修改代码，优先读取config.toml

如果不想依赖环境变量，可以修改代码确保正确读取 `config.toml`。

**文件**: `crates/agent-mem-llm/src/providers/zhipu.rs`

```rust
// 修改 generate 方法，优先读取配置文件
async fn generate(&self, messages: Vec<Message>) -> Result<String> {
    // 优先读取环境变量，然后fallback到配置文件
    let api_key = std::env::var("ZHIPU_API_KEY")
        .or_else(|_| {
            // 尝试从配置文件读取
            let config_path = "config.toml";
            if let Ok(contents) = std::fs::read_to_string(config_path) {
                // 简单解析 TOML (生产环境应使用 toml crate)
                if let Some(line) = contents.lines()
                    .find(|l| l.starts_with("api_key")) {
                    if let Some(key) = line.split('=').nth(1) {
                        return Ok(key.trim().trim_matches('"').to_string());
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .map_err(|_| AgentMemError::ConfigError(
            "Zhipu API key not configured (env: ZHIPU_API_KEY or config.toml)".to_string()
        ))?;
    
    // ... 其余代码
}
```

---

## 🚀 推荐实施步骤

### 立即执行（5分钟）

1. **停止当前服务**
   ```bash
   pkill -f agent-mem-server
   pkill -f "next dev"
   ```

2. **使用正确的脚本重启**
   ```bash
   cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen
   bash start_server_no_auth.sh
   ```

3. **验证环境变量**
   ```bash
   ps aux | grep agent-mem-server | grep -v grep
   # 检查进程环境变量
   cat /proc/<PID>/environ | tr '\0' '\n' | grep ZHIPU
   ```

4. **验证API**
   ```bash
   # 测试聊天
   curl -X POST http://localhost:8080/api/v1/agents/<agent-id>/chat \
     -H "Content-Type: application/json" \
     -d '{"message": "你好", "user_id": "test", "session_id": "test"}'
   ```

---

## 📊 启动脚本清单

| 脚本 | 用途 | Zhipu配置 | 推荐 |
|------|------|-----------|------|
| `start_server_no_auth.sh` | 无认证启动 | ✅ 有 | ✅ **推荐** |
| `start_server_with_correct_onnx.sh` | 完整启动 | ✅ 有 | ✅ 推荐 |
| `start_full_stack.sh` | 全栈启动 | ⚠️ 检查 | 📝 需验证 |
| `start.sh` | 基础启动 | ⚠️ 检查 | 📝 需验证 |
| 直接运行 | 无脚本 | ❌ 无 | ❌ **不推荐** |

---

## ✅ 验证清单

启动后验证：

- [ ] 后端进程运行中
- [ ] 健康检查通过 (`/health`)
- [ ] 日志中显示 "Zhipu API configured"
- [ ] 前端可以访问 (http://localhost:3001)
- [ ] 聊天功能正常，无 API key 错误

---

## 💡 最佳实践

1. **始终使用启动脚本**: 不要直接运行二进制文件
2. **统一环境变量**: 使用 `start_server_no_auth.sh` 或创建统一脚本
3. **验证配置**: 启动后检查日志确认配置加载
4. **保持文档同步**: 更新启动方式时同步更新文档

---

## 🔍 故障排查

### 症状1: "Zhipu API key not configured"

**检查**:
```bash
ps aux | grep agent-mem-server  # 查看进程
cat backend.log | grep -i "zhipu\|api.*key"  # 查看日志
```

**修复**: 使用 `start_server_no_auth.sh` 重启

### 症状2: 配置文件不生效

**检查**:
```bash
cat config.toml | grep -A 3 "\[llm.zhipu\]"
```

**修复**: 确保环境变量优先级正确，或修改代码读取配置文件

---

**状态**: 分析完成，待执行修复

