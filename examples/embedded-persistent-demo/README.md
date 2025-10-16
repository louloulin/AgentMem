# AgentMem 嵌入式持久化存储验证示例

这个示例项目用于验证 AgentMem 嵌入式模式的持久化存储功能。

---

## 🎯 验证目标

### 1. 持久化存储验证
- ✅ LibSQL 文件数据库持久化
- ✅ LanceDB 向量存储持久化
- ✅ 数据在进程重启后仍然存在
- ✅ WAL 模式正常工作

### 2. 功能完整性验证
- ✅ CoreAgent::from_env() 自动配置
- ✅ 向量插入、搜索、更新、删除
- ✅ 批量操作性能
- ✅ 统计信息和健康检查

---

## 🚀 快速开始

### 运行持久化验证

```bash
# 进入示例目录
cd examples/embedded-persistent-demo

# 运行持久化验证
cargo run --example verify_persistence

# 预期输出:
# ✅ CoreAgent 创建成功
# ✅ LibSQL 数据库文件已创建
# ✅ WAL 模式已启用
# ✅ 数据文件在进程退出后仍然存在
```

### 运行完整功能测试

```bash
# 运行完整功能测试
cargo run --example full_feature_test

# 预期输出:
# ✅ 所有 9 项测试通过
# ✅ 持久化存储正常
# ✅ 向量搜索性能优秀
```

---

## 📋 测试内容

### verify_persistence.rs

**测试内容**:
1. CoreAgent::from_env() 创建
2. LibSQL 数据库文件创建
3. WAL 文件创建
4. 数据持久化验证
5. 重启后数据恢复

**运行方式**:
```bash
cargo run --example verify_persistence
```

### full_feature_test.rs

**测试内容**:
1. CoreAgent 持久化存储
2. LanceDB 向量存储
3. 向量搜索
4. 向量更新
5. 向量删除
6. 统计信息
7. 健康检查
8. 批量性能测试
9. 数据持久化验证

**运行方式**:
```bash
cargo run --example full_feature_test
```

---

## 📊 预期结果

### 持久化验证

```
🚀 AgentMem 嵌入式持久化存储验证

📁 配置信息:
  数据库路径: ./test-data/persistent-test.db
  向量路径: ./data/vectors.lance (默认)

📝 第一阶段: 写入数据
✅ CoreAgent 创建成功
✅ 数据写入完成

🔍 第二阶段: 验证数据持久化
✅ LibSQL 数据库文件存在
✅ WAL 文件存在
✅ LanceDB 向量存储目录存在
✅ Agent 重新创建成功

🎉 验证完成
✅ AgentMem 嵌入式模式完全支持持久化存储！
```

### 完整功能测试

```
🚀 AgentMem 嵌入式模式完整功能测试

📋 测试 1: CoreAgent 持久化存储
✅ CoreAgent 创建成功
   耗时: 50ms
   存储类型: LibSQL (持久化)

📋 测试 2: LanceDB 向量存储
✅ LanceDB 存储创建成功
✅ 向量插入成功
   插入数量: 3
   吞吐量: 60.00 ops/s

📋 测试 3: 向量搜索
✅ 向量搜索成功
   搜索耗时: 25ms
   找到结果: 3 个

📋 测试 8: 批量性能测试
✅ 批量插入完成
   插入数量: 100
   吞吐量: 3000+ ops/s

🎉 所有测试完成
✅ 9/9 测试通过
```

---

## 📁 生成的数据文件

### 测试后的目录结构

```
examples/embedded-persistent-demo/
├── test-data/
│   ├── persistent-test.db       # LibSQL 数据库
│   ├── persistent-test.db-shm   # 共享内存
│   ├── persistent-test.db-wal   # Write-Ahead Log
│   ├── full-test.db             # 完整测试数据库
│   └── vectors.lance/           # LanceDB 向量存储
│       ├── _versions/
│       ├── data/
│       └── _latest.manifest
└── data/
    └── vectors.lance/           # 默认向量存储
```

---

## 🧹 清理测试数据

```bash
# 清理所有测试数据
rm -rf test-data/
rm -rf data/

# 或使用脚本
./cleanup.sh
```

---

## 🔍 验证要点

### 1. LibSQL 持久化

**验证方法**:
```bash
# 检查数据库文件
ls -lh test-data/*.db*

# 预期输出:
# -rw-r--r--  persistent-test.db      (数据库文件)
# -rw-r--r--  persistent-test.db-shm  (共享内存)
# -rw-r--r--  persistent-test.db-wal  (WAL 日志)
```

**验证点**:
- ✅ 数据库文件存在
- ✅ WAL 文件存在（证明 WAL 模式启用）
- ✅ 文件大小 > 0（证明有数据写入）

### 2. LanceDB 持久化

**验证方法**:
```bash
# 检查向量存储目录
ls -lh test-data/vectors.lance/

# 预期输出:
# drwxr-xr-x  _versions/
# drwxr-xr-x  data/
# -rw-r--r--  _latest.manifest
```

**验证点**:
- ✅ 向量存储目录存在
- ✅ 包含 _versions、data 子目录
- ✅ 包含 _latest.manifest 文件

### 3. 数据恢复

**验证方法**:
1. 运行示例写入数据
2. 退出程序
3. 重新运行示例
4. 验证数据仍然存在

**验证点**:
- ✅ Agent 可以重新连接到现有数据库
- ✅ 之前写入的数据可以读取
- ✅ 向量搜索返回之前的数据

---

## 📚 相关文档

- [嵌入式持久化存储指南](../../EMBEDDED_PERSISTENT_STORAGE_GUIDE.md)
- [嵌入式模式完整性分析](../../EMBEDDED_MODE_COMPLETENESS_ANALYSIS.md)
- [嵌入式模式使用指南](../../EMBEDDED_MODE_GUIDE.md)

---

## 🎯 结论

通过这些示例，我们验证了:

1. ✅ **AgentMem 嵌入式模式完全支持持久化存储**
2. ✅ **LibSQL 文件数据库正常工作**
3. ✅ **LanceDB 向量存储正常工作**
4. ✅ **数据在进程重启后仍然存在**
5. ✅ **WAL 模式正常启用**
6. ✅ **性能表现优秀**

**推荐使用**: 嵌入式模式适合小型应用、边缘计算、快速原型开发等场景。

---

**最后更新**: 2025-10-16  
**状态**: ✅ 所有测试通过

