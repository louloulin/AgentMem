# 商品客户记忆系统设计

**日期**: 2025-11-07  
**场景**: 电商平台的商品管理记忆系统  
**规模**: 1,000种商品（可扩展至10,000种）

---

## 🎯 应用场景

### 业务需求

1. **商品信息记忆**
   - 商品基本信息（名称、价格、库存）
   - 商品分类和属性
   - 商品销售数据
   - 商品评价和反馈

2. **客户交互记忆**
   - 客户浏览记录
   - 客户购买偏好
   - 客户咨询历史
   - 推荐历史

3. **业务分析记忆**
   - 热销商品分析
   - 库存预警
   - 价格趋势
   - 竞品对比

---

## 📊 商品数据结构

### 商品类别分布（10,000种）

```
1. 电子产品 (2,000种)
   - 手机 (500)
   - 电脑 (500)
   - 配件 (1,000)

2. 服装鞋帽 (2,500种)
   - 男装 (800)
   - 女装 (1,200)
   - 童装 (500)

3. 食品饮料 (1,500种)
   - 零食 (600)
   - 饮料 (400)
   - 生鲜 (500)

4. 家居用品 (2,000种)
   - 家具 (600)
   - 厨具 (800)
   - 装饰 (600)

5. 图书文娱 (1,000种)
   - 图书 (500)
   - 文具 (300)
   - 乐器 (200)

6. 运动户外 (1,000种)
   - 运动装备 (500)
   - 户外用品 (500)
```

---

## 🏗️ 记忆架构设计

### Scope层次

```
1. Global Scope - 商品基础信息
   - 商品ID、名称、分类
   - SKU、价格、库存
   - 所有用户可见

2. Agent Scope - 商家/客服专属记忆
   - 商家A的商品管理记忆
   - 客服B的咨询处理记忆
   - 跨session持久化

3. User Scope - 客户个人记忆
   - 用户浏览历史
   - 购买偏好
   - 收藏和关注

4. Session Scope - 临时工作记忆
   - 当前购物车
   - 当前咨询对话
   - 临时搜索结果
```

### Memory Type分类

```
1. Semantic Memory - 商品知识库
   - 商品规格说明
   - 使用指南
   - 常见问题FAQ

2. Episodic Memory - 交易事件
   - 订单记录
   - 退换货记录
   - 客户反馈

3. Procedural Memory - 业务流程
   - 下单流程
   - 售后流程
   - 库存管理流程

4. Working Memory - 实时状态
   - 当前库存
   - 实时价格
   - 促销活动
```

---

## 🎨 商品记忆模板

### 模板1: 商品基础信息

```json
{
  "content": "商品ID: P001234, 名称: iPhone 15 Pro Max 256GB 钛金属, 分类: 电子产品>手机>智能手机, 品牌: Apple, 价格: ¥9999, 库存: 150件, 状态: 在售",
  "memory_type": "Semantic",
  "importance": 0.8,
  "metadata": {
    "product_id": "P001234",
    "category": "electronics",
    "subcategory": "smartphone",
    "brand": "Apple",
    "price": "9999",
    "stock": "150",
    "status": "active",
    "scope_type": "global"
  }
}
```

### 模板2: 客户浏览记录

```json
{
  "content": "用户user-zhang-san浏览了商品iPhone 15 Pro Max，停留时间3分钟，查看了产品详情和用户评价",
  "memory_type": "Episodic",
  "importance": 0.6,
  "user_id": "user-zhang-san",
  "metadata": {
    "product_id": "P001234",
    "action": "view",
    "duration": "180",
    "timestamp": "2025-11-07T10:30:00Z",
    "scope_type": "user"
  }
}
```

### 模板3: 销售数据分析

```json
{
  "content": "商品P001234在过去7天销售120件，环比增长25%，主要购买人群为25-35岁男性，建议增加库存",
  "memory_type": "Episodic",
  "importance": 0.9,
  "agent_id": "agent-sales-analyst",
  "metadata": {
    "product_id": "P001234",
    "analysis_type": "sales_trend",
    "period": "7days",
    "sales_count": "120",
    "growth_rate": "0.25",
    "recommendation": "increase_stock",
    "scope_type": "agent"
  }
}
```

### 模板4: 实时库存预警

```json
{
  "content": "库存预警: 商品P001234当前库存50件，低于安全库存阈值100件，建议立即补货",
  "memory_type": "Working",
  "importance": 1.0,
  "session_id": "session-inventory-check",
  "metadata": {
    "product_id": "P001234",
    "alert_type": "low_stock",
    "current_stock": "50",
    "threshold": "100",
    "urgency": "high",
    "scope_type": "session"
  }
}
```

---

## 🚀 批量写入策略

### 分批写入设计

```
总量: 10,000种商品
批次大小: 100条/批
总批次: 100批
写入速率: 100条/秒（理论）
预计时间: 100秒（1.7分钟）

实际考虑:
- API限流
- 数据库写入性能
- 网络延迟
预计实际时间: 5-10分钟
```

### 数据生成策略

1. **真实感数据**
   - 真实的商品名称组合
   - 合理的价格范围
   - 符合分类的属性

2. **多样性**
   - 不同分类的商品
   - 不同价格区间
   - 不同库存状态

3. **可搜索性**
   - 包含常见关键词
   - 支持中文分词
   - 支持品牌搜索

---

## 📝 脚本功能设计

### 功能清单

1. ✅ **商品数据生成**
   - 10个主分类
   - 50个子分类
   - 200个品牌
   - 真实感的商品名称

2. ✅ **批量写入**
   - 分批提交
   - 错误重试
   - 进度显示
   - 统计报告

3. ✅ **多维度记忆**
   - Global: 商品基础信息
   - Agent: 商家管理记忆
   - User: 客户浏览记忆
   - Session: 临时操作记忆

4. ✅ **性能监控**
   - 写入速度统计
   - 成功/失败计数
   - 耗时分析

---

## 🧪 测试场景

### 场景1: 商品搜索

```bash
# 搜索iPhone相关商品
curl "http://localhost:8080/api/v1/memories/search?query=iPhone&limit=10"

# 预期: 返回所有iPhone相关商品记忆
```

### 场景2: 分类筛选

```bash
# 搜索电子产品分类
curl "http://localhost:8080/api/v1/memories/search?query=电子产品&limit=20"

# 预期: 返回电子产品分类的商品
```

### 场景3: 价格区间

```bash
# 搜索高端商品（价格>5000）
curl "http://localhost:8080/api/v1/memories/search?query=价格&limit=50"

# 预期: 返回包含价格信息的商品记忆
```

### 场景4: 客户个性化

```bash
# 查询用户zhang-san的浏览记录
curl "http://localhost:8080/api/v1/memories/search?user_id=user-zhang-san&query=浏览"

# 预期: 只返回该用户的记忆
```

---

## 📊 性能指标

### 写入性能

```
目标指标:
- 写入速度: ≥50条/秒
- 成功率: ≥99%
- 平均延迟: ≤20ms/条
- 错误重试: ≤3次

实际监控:
- 批次成功率
- API响应时间
- 数据库写入时间
- 内存使用情况
```

### 查询性能

```
目标指标:
- 查询延迟: ≤100ms
- 并发支持: ≥100 QPS
- 结果准确率: ≥95%
- 翻页性能: ≤50ms

测试方法:
- 单关键词搜索
- 多关键词组合
- 分类筛选
- 范围查询
```

---

## 🎯 商业价值

### 应用场景

1. **智能客服**
   - 快速检索商品信息
   - 回答客户咨询
   - 推荐相关商品

2. **个性化推荐**
   - 基于浏览历史
   - 基于购买偏好
   - 协同过滤

3. **库存管理**
   - 实时库存预警
   - 销售趋势分析
   - 补货建议

4. **数据分析**
   - 商品热度分析
   - 用户行为分析
   - 市场趋势预测

---

## ✅ 成功标准

- [ ] 成功写入10,000条商品记忆
- [ ] 写入成功率 ≥99%
- [ ] 查询响应时间 ≤100ms
- [ ] 记忆隔离正确（不同scope）
- [ ] 搜索结果准确
- [ ] 性能指标达标

---

**下一步**: 创建批量写入脚本 `scripts/add_product_memories.sh`

