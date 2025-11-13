# AgentMem 故障排查指南

## 问题 1: 启动时卡住不动

### 症状
```bash
./start-with-zhipu.sh
=========================================
🚀 启动 AgentMem Server (智谱 AI)
=========================================
主机: 0.0.0.0
端口: 8080
数据库: file:./data/agentmem.db
Embedder: fastembed / BAAI/bge-small-en-v1.5
LLM Provider: zhipu / glm-4.6
认证: false (禁用)
库目录: /Users/.../dist/server/lib
=========================================
# 卡住，没有进一步输出
```

### 根本原因
**FastEmbed 首次运行时会下载模型文件**

- 模型大小：约 100MB
- 下载位置：`~/.cache/fastembed/` 或 `.fastembed_cache/`
- 下载时间：取决于网络速度，通常需要 1-5 分钟
- **没有进度显示**：FastEmbed 下载时不显示进度条

### 解决方案

#### 方案 1：耐心等待（推荐）
```bash
# 首次启动时，等待 5-10 分钟
./start-with-zhipu.sh

# 模型下载完成后，会看到：
# 2025-11-13T06:15:23.656309Z  INFO Initializing Memory with LibSQL storage
# 2025-11-13T06:15:23.656951Z  INFO Memory initialized successfully
```

#### 方案 2：预下载模型
```bash
# 在启动服务器前，先下载模型
cd dist/server

# 创建缓存目录
mkdir -p .fastembed_cache

# 使用 Python 预下载（如果安装了 fastembed）
python3 << 'EOF'
from fastembed import TextEmbedding
model = TextEmbedding(model_name="BAAI/bge-small-en-v1.5")
print("✅ 模型下载完成")
EOF

# 然后启动服务器
./start-with-zhipu.sh
```

#### 方案 3：使用已下载的模型
```bash
# 如果其他地方已经下载过模型，可以复制缓存
cp -r ~/.cache/fastembed dist/server/.fastembed_cache
```

#### 方案 4：查看下载进度
```bash
# 在另一个终端监控缓存目录
watch -n 1 'du -sh ~/.cache/fastembed 2>/dev/null || du -sh .fastembed_cache 2>/dev/null'

# 或者监控网络流量
nettop -P -L 1
```

### 验证模型已下载
```bash
# 检查缓存目录
ls -lh ~/.cache/fastembed/
# 或
ls -lh .fastembed_cache/

# 应该看到类似：
# BAAI_bge-small-en-v1.5/
#   ├── model.onnx (约 100MB)
#   ├── tokenizer.json
#   └── ...
```

---

## 问题 2: 数据库连接失败

### 症状
```
WARN 创建 HistoryManager 失败: Storage error: 连接数据库失败: (code: 14) unable to open database file
```

### 根本原因
**SQLite URL 格式错误**

错误的格式：
```bash
export DATABASE_URL="sqlite://agentmem.db"    # ❌ 两个斜杠
export DATABASE_URL="sqlite:///path/to/db"    # ❌ 绝对路径但目录不存在
```

正确的格式：
```bash
export DATABASE_URL="file:./data/agentmem.db"      # ✅ 推荐：相对路径
export DATABASE_URL="sqlite:///./data/agentmem.db" # ✅ 三个斜杠
export DATABASE_URL="file:/absolute/path/db"       # ✅ 绝对路径
```

### 解决方案

#### 1. 修复 DATABASE_URL
```bash
# 编辑启动脚本
vim start-with-zhipu.sh

# 修改为：
export DATABASE_URL="file:./data/agentmem.db"
```

#### 2. 确保数据目录存在
```bash
mkdir -p data
chmod 755 data
```

#### 3. 检查文件权限
```bash
# 确保当前用户有写权限
ls -la data/
chmod 644 data/*.db 2>/dev/null || true
```

---

## 问题 3: ONNX Runtime 库加载失败

### 症状
```
dyld: Library not loaded: @rpath/libonnxruntime.1.22.0.dylib
Reason: image not found
```

### 根本原因
- 库文件不存在
- 库路径未正确设置
- 库文件权限问题

### 解决方案

#### 1. 检查库文件
```bash
ls -lh lib/
# 应该看到：
# libonnxruntime.1.22.0.dylib (32MB)
# libonnxruntime.dylib (符号链接)
```

#### 2. 检查环境变量
```bash
echo $DYLD_LIBRARY_PATH  # macOS
echo $LD_LIBRARY_PATH    # Linux
echo $ORT_DYLIB_PATH
```

#### 3. 手动设置库路径
```bash
export DYLD_LIBRARY_PATH="$(pwd)/lib:$DYLD_LIBRARY_PATH"
export ORT_DYLIB_PATH="$(pwd)/lib/libonnxruntime.1.22.0.dylib"
```

#### 4. 重新构建发布包
```bash
cd /path/to/agentmen
./build-release.sh --server-only
```

---

## 问题 4: LLM Provider 未配置警告

### 症状
```
WARN 未找到任何 LLM API Key 环境变量
WARN LLM Provider 未配置，Intelligence 组件将不可用
```

### 说明
**这是正常的运行时警告，不是错误**

- LLM 功能是可选的
- 不影响基础的记忆存储和检索功能
- 只影响智能推理功能（如自动摘要、关系提取等）

### 解决方案（如需启用 LLM）

#### 使用智谱 AI
```bash
export LLM_PROVIDER="zhipu"
export LLM_MODEL="glm-4.6"
export ZHIPU_API_KEY="your_api_key_here"
```

#### 使用 OpenAI
```bash
export LLM_PROVIDER="openai"
export LLM_MODEL="gpt-4"
export OPENAI_API_KEY="your_api_key_here"
```

#### 使用 Ollama（本地）
```bash
export LLM_PROVIDER="ollama"
export LLM_MODEL="llama2"
export OLLAMA_BASE_URL="http://localhost:11434"
```

---

## 问题 5: 端口已被占用

### 症状
```
Error: Address already in use (os error 48)
```

### 解决方案

#### 1. 查找占用端口的进程
```bash
lsof -i :8080
# 或
netstat -an | grep 8080
```

#### 2. 停止占用进程
```bash
kill -9 <PID>
```

#### 3. 使用其他端口
```bash
export SERVER_PORT=8081
./start-with-zhipu.sh
```

---

## 调试技巧

### 1. 启用详细日志
```bash
export RUST_LOG=debug
export RUST_BACKTRACE=1
./start-with-zhipu.sh
```

### 2. 检查进程状态
```bash
ps aux | grep agent-mem-server
```

### 3. 监控资源使用
```bash
# CPU 和内存
top -pid $(pgrep agent-mem-server)

# 磁盘 I/O
iotop -p $(pgrep agent-mem-server)
```

### 4. 查看网络连接
```bash
lsof -i -P | grep agent-mem
```

### 5. 测试 API 连接
```bash
# 等待服务器启动后
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/memories
```

---

## 常见问题 FAQ

### Q: 首次启动为什么这么慢？
A: FastEmbed 需要下载模型文件（约 100MB），这是一次性操作。后续启动会很快。

### Q: 如何加速启动？
A: 
1. 预下载模型文件
2. 使用 SSD 存储
3. 确保网络连接良好

### Q: 数据存储在哪里？
A:
- SQLite 数据库：`data/agentmem.db`
- 向量数据：`data/vectors.lance/`
- 历史记录：`data/history.db`
- 模型缓存：`.fastembed_cache/` 或 `~/.cache/fastembed/`

### Q: 如何清理数据重新开始？
A:
```bash
rm -rf data/*.db
rm -rf data/vectors.lance
```

### Q: 如何备份数据？
A:
```bash
tar -czf agentmem-backup-$(date +%Y%m%d).tar.gz data/
```

### Q: 如何升级到新版本？
A:
1. 备份数据
2. 停止服务器
3. 替换二进制文件
4. 启动新版本

---

## 获取帮助

如果以上方法都无法解决问题，请：

1. 收集日志信息
2. 记录错误信息
3. 提供系统信息（OS、版本等）
4. 在 GitHub 提交 Issue

