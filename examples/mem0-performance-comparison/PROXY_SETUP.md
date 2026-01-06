# 代理配置说明

## 快速配置

### 方法 1: 使用配置脚本（推荐）

```bash
cd examples/mem0-performance-comparison
source setup_proxy.sh
```

### 方法 2: 手动设置环境变量

```bash
export https_proxy="http://127.0.0.1:4780"
export http_proxy="http://127.0.0.1:4780"
```

### 方法 3: 使用 .env.proxy 文件

```bash
cd examples/mem0-performance-comparison
export $(cat .env.proxy | xargs)
```

## 验证代理配置

```bash
# 检查环境变量
echo $https_proxy
echo $http_proxy

# 测试代理连接（可选）
curl -I --proxy http://127.0.0.1:4780 https://www.google.com
```

## 在测试中使用代理

### 运行 Mem0 测试（Python）

代理会自动通过环境变量传递给 Python 程序：

```bash
source setup_proxy.sh
python3 mem0_simple_benchmark.py
```

### 运行 AgentMem 测试（Rust）

Rust 程序会自动使用系统代理设置：

```bash
source setup_proxy.sh
./target/release/agentmem_benchmark
```

### 运行完整对比

```bash
source setup_proxy.sh
./run_comparison.sh
```

## 注意事项

1. **代理地址**: 确保代理地址正确（`http://127.0.0.1:4780`）
2. **代理服务**: 确保代理服务正在运行
3. **环境变量**: 代理设置只在当前 shell 会话中有效
4. **永久设置**: 如需永久设置，可以添加到 `~/.bashrc` 或 `~/.zshrc`

## 永久配置（可选）

如果需要永久配置代理，可以添加到 shell 配置文件中：

```bash
# 添加到 ~/.bashrc 或 ~/.zshrc
echo 'export https_proxy="http://127.0.0.1:4780"' >> ~/.bashrc
echo 'export http_proxy="http://127.0.0.1:4780"' >> ~/.bashrc
source ~/.bashrc
```
