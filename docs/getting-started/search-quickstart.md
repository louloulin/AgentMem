# AgentMem 向量搜索快速使用指南

**版本**: v1.0  
**日期**: 2025-11-05  
**预计阅读时间**: 3分钟

---

## 🚀 快速开始

### 1. 确认服务运行

```bash
# 检查后端
curl http://localhost:8080/health

# 检查前端
curl http://localhost:3001
```

✅ 两个服务都应该返回正常响应。

---

## 🔍 使用向量搜索

### 方式1: 通过 UI（推荐）

#### 步骤1: 打开记忆管理页面
```
http://localhost:3001/admin/memories
```

#### 步骤2: 在搜索框输入关键词
尝试这些示例：
- `AI产品` - 查找产品相关记忆
- `工程师` - 查找技术团队相关
- `融资` - 查找财务相关
- `市场` - 查找市场相关
- `优化` - 查找改进相关

#### 步骤3: 查看结果
- 结果按相关性排序
- 点击查看详细内容
- 可以进一步筛选和过滤

---

### 方式2: 通过 API

#### 添加记忆（会自动生成向量）

```bash
curl -X POST "http://localhost:8080/api/v1/memories" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "agent_id": "my-agent",
    "content": "今天和客户讨论了AI产品的新功能需求",
    "memory_type": "Episodic",
    "importance": 0.8
  }'
```

#### 向量搜索

```bash
curl -X POST "http://localhost:8080/api/v1/memories/search" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: default" \
  -H "X-Organization-ID: default-org" \
  -d '{
    "query": "AI产品",
    "limit": 5
  }' | jq
```

---

## 🎯 测试数据

系统已预置5条测试记忆，可以直接搜索：

| 关键词 | 预期结果 |
|--------|----------|
| `AI产品` | AI产品研发记忆 |
| `技术` 或 `工程师` | 技术团队记忆 |
| `融资` 或 `投资` | B轮融资记忆 |
| `产品迭代` | 产品优化记忆 |
| `市场` | 市场拓展记忆 |

---

## 💡 搜索技巧

### ✅ 好的搜索方式

1. **使用核心关键词**
   ```
   ✅ "AI产品"
   ✅ "技术团队"
   ✅ "融资计划"
   ```

2. **使用同义词**
   ```
   "工程师" ≈ "技术人员" ≈ "研发团队"
   "融资" ≈ "投资" ≈ "资金"
   ```

3. **组合查询**
   ```
   "AI技术" - 同时相关AI和技术
   "产品市场" - 跨领域查询
   ```

### ❌ 避免的搜索方式

1. **过长的句子**
   ```
   ❌ "我想找一下关于AI产品研发的相关信息"
   ✅ "AI产品"
   ```

2. **无关词汇**
   ```
   ❌ "请帮我搜索融资"
   ✅ "融资"
   ```

---

## 🧪 语义理解测试

尝试这些搜索，体验语义理解：

### 同义词识别
```
"产品" vs "产品开发" vs "研发"
→ 都能找到产品相关记忆
```

### 相关概念
```
"机器学习" → 找到技术团队
"客户" → 找到市场拓展
"效率" → 找到AI产品
```

### 跨领域查询
```
"AI技术团队" → 同时相关两个领域
```

---

## 📊 搜索性能

| 指标 | 值 |
|------|-----|
| 平均响应时间 | 3-6ms |
| 向量维度 | 384 |
| 相似度算法 | 余弦相似度 |
| Embedder模型 | BAAI/bge-small-en-v1.5 |

---

## 🔧 常见问题

### Q1: 搜索没有返回结果？
**A**: 确认：
1. 记忆内容已添加到系统
2. Embedder正常运行（检查健康状态）
3. 搜索关键词与记忆内容相关

### Q2: 如何提高搜索准确性？
**A**: 
1. 使用更具体的关键词
2. 提高重要记忆的 `importance` 分数
3. 添加更多相关记忆来丰富数据

### Q3: 旧记忆搜索不到？
**A**: 旧记忆可能缺少embedding。解决方案：
- 重新导入数据
- 或使用批量更新工具生成embeddings

### Q4: 搜索速度慢？
**A**: 当前搜索非常快(3-6ms)。如果慢，检查：
1. 网络连接
2. 数据库大小
3. 服务器负载

---

## 📚 进阶使用

### 按类型搜索（即将支持）
```bash
# 未来版本将支持
{
  "query": "AI产品",
  "memory_type": "Episodic",
  "limit": 5
}
```

### 按重要性过滤（即将支持）
```bash
# 未来版本将支持
{
  "query": "融资",
  "min_importance": 0.8,
  "limit": 5
}
```

### 时间范围搜索（即将支持）
```bash
# 未来版本将支持
{
  "query": "会议",
  "start_date": "2025-11-01",
  "end_date": "2025-11-05",
  "limit": 5
}
```

---

## 🌐 相关链接

- **记忆管理**: http://localhost:3001/admin/memories
- **知识图谱**: http://localhost:3001/admin/graph
- **插件管理**: http://localhost:3001/admin/plugins
- **API文档**: http://localhost:8080/swagger-ui/
- **健康检查**: http://localhost:8080/health

---

## 🎉 开始使用

现在就去试试吧！

1. 打开 http://localhost:3001/admin/memories
2. 在搜索框输入 "AI产品"
3. 查看搜索结果
4. 体验语义搜索的强大功能！

---

**需要帮助？** 查看完整测试报告: [VECTOR_SEARCH_TEST_REPORT.md](./VECTOR_SEARCH_TEST_REPORT.md)

