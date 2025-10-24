# Python SDK 基础演示

这是AgentMem Python SDK的基础演示示例。

## 前置要求

1. Python 3.8+
2. 已构建的Python绑定

## 构建Python绑定

```bash
# 进入Python绑定目录
cd ../../crates/agent-mem-python

# 安装maturin（如果未安装）
pip install maturin

# 开发模式构建（推荐）
maturin develop

# 或者构建wheel包
maturin build --release
```

## 运行示例

```bash
# 直接运行
python demo_basic.py

# 或使用Python 3
python3 demo_basic.py
```

## 示例输出

```
🐍 AgentMem Python SDK 基础演示

✅ 成功导入 agentmem_native

1️⃣ 创建Memory实例...
✅ Memory实例创建成功

2️⃣ 添加记忆...
  ✅ 添加成功: Python是一门简单易学的编程语言... (ID: 12345678)
  ✅ 添加成功: Rust提供了出色的性能和内存安全... (ID: 23456789)
  ...

3️⃣ 搜索记忆...
  🔍 搜索关于编程的记忆: "编程"
    ✅ 找到 2 条相关记忆:
       1. Python是一门简单易学的编程语言...
       2. Rust提供了出色的性能和内存安全...

4️⃣ 获取所有记忆...
  ✅ 共有 5 条记忆

5️⃣ 删除记忆...
  ✅ 成功删除记忆

6️⃣ 验证删除后的记忆数量...
  ✅ 现在有 4 条记忆（已删除1条）

7️⃣ 清空所有记忆...
  ✅ 成功清空 4 条记忆

🎉 演示完成！
```

## API说明

### 创建Memory实例

```python
import agentmem_native

memory = agentmem_native.Memory()
```

### 添加记忆

```python
memory_id = await memory.add("记忆内容")
```

### 搜索记忆

```python
results = await memory.search("搜索查询")
for result in results:
    print(result['content'])
```

### 获取所有记忆

```python
all_memories = await memory.get_all()
```

### 删除记忆

```python
success = await memory.delete(memory_id)
```

### 清空所有记忆

```python
count = await memory.clear()
```

## 特点

- ✅ 简单易用的API
- ✅ 异步支持（async/await）
- ✅ 高性能Rust后端
- ✅ 类型安全
- ✅ 零配置启动

## 故障排除

### 导入错误

如果遇到 `ImportError: No module named 'agentmem_native'`：

1. 确保已构建Python绑定：`cd ../../crates/agent-mem-python && maturin develop`
2. 检查Python版本：`python --version`（需要3.8+）
3. 检查虚拟环境是否正确激活

### 运行时错误

如果遇到运行时错误：

1. 检查是否有足够的磁盘空间
2. 确保没有其他进程占用数据库文件
3. 查看错误日志获取详细信息

## 相关文档

- [Python SDK API文档](../../crates/agent-mem-python/README.md)
- [AgentMem主文档](../../README.md)

