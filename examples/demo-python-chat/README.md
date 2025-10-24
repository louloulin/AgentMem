# Python SDK 智能对话演示

这是AgentMem Python SDK在智能对话场景中的应用演示。

## 功能展示

- 多轮对话记忆
- 上下文理解
- 用户偏好学习
- 智能推荐

## 前置要求

1. Python 3.8+
2. 已构建的Python绑定

## 构建Python绑定

```bash
cd ../../crates/agent-mem-python
pip install maturin
maturin develop
```

## 运行示例

```bash
python demo_chat.py
```

## 应用场景

### 智能客服系统

```python
class CustomerServiceBot:
    def __init__(self, memory):
        self.memory = memory
    
    async def handle_query(self, query: str):
        # 搜索相关历史
        history = await self.memory.search(query)
        
        # 生成回复（基于历史上下文）
        response = self.generate_response(query, history)
        
        # 保存到记忆
        await self.memory.add(f"用户问: {query}")
        
        return response
```

### AI助手

```python
class AIAssistant:
    def __init__(self, memory):
        self.memory = memory
    
    async def learn_preference(self, preference: str):
        """学习用户偏好"""
        await self.memory.add(f"用户偏好: {preference}")
    
    async def get_recommendations(self):
        """基于偏好推荐"""
        preferences = await self.memory.search("用户偏好")
        return self.generate_recommendations(preferences)
```

### 个性化服务

```python
async def personalize_service(user_id: str, memory):
    """个性化服务"""
    # 获取用户历史
    history = await memory.get_all()
    
    # 分析用户画像
    profile = analyze_user_profile(history)
    
    # 提供定制服务
    return customize_service(profile)
```

## 核心特性

### 1. 长期记忆

跨会话保持用户信息：

```python
# 第一次对话
await memory.add("用户喜欢技术文档")

# 第二天
results = await memory.search("用户喜欢")
# 结果：["用户喜欢技术文档"]
```

### 2. 语义搜索

智能匹配相关内容：

```python
await memory.add("我喜欢Python编程")
results = await memory.search("编程语言")
# 结果：["我喜欢Python编程"]
```

### 3. 上下文感知

理解对话连贯性：

```python
# 记住对话历史
await memory.add("用户问：产品价格是多少")
await memory.add("用户问：有折扣吗")

# 理解"它"指的是产品
await memory.add("用户问：它支持哪些功能")
```

## 示例输出

```
🤖 AgentMem Python SDK 智能对话演示

==================================================
场景1：首次咨询
==================================================

用户: 你好，我想了解一下你们的产品
助手: 您好！很高兴为您服务...

==================================================
场景2：第二天继续咨询
==================================================

用户: 你好，我昨天问过你产品的事

🔍 回忆相关记忆: "产品 AgentMem"
✅ 找到 3 条相关记忆:
   1. 用户说：我对AI记忆管理系统很感兴趣
   2. 用户说：它有什么特点？
   3. 用户说：我想了解一下你们的产品

助手: 您好！我记得您昨天咨询过我们的AgentMem产品...

==================================================
场景3：个性化推荐
==================================================

🔍 分析用户兴趣...
✅ 用户画像分析：
   • 关注产品（提及4次）
   • 关注性能（提及2次）
   • 关注AI（提及2次）

🎯 个性化推荐：
助手: 基于您的兴趣，我推荐您关注：
  1️⃣ AgentMem性能基准测试报告
  2️⃣ 多模态功能演示视频
  3️⃣ 技术架构深度解析
  4️⃣ 企业级部署指南

🎉 演示完成！
```

## 扩展建议

### 添加情感分析

```python
async def analyze_sentiment(message: str):
    """分析用户情感"""
    # 使用情感分析模型
    sentiment = sentiment_model(message)
    await memory.add(f"用户情感: {sentiment}")
```

### 添加意图识别

```python
async def detect_intent(message: str):
    """识别用户意图"""
    intent = intent_classifier(message)
    return intent
```

### 添加多语言支持

```python
async def detect_language(message: str):
    """检测语言"""
    lang = detect_lang(message)
    return lang
```

## 相关文档

- [Python SDK API文档](../../crates/agent-mem-python/README.md)
- [智能对话最佳实践](../../docs/BEST_PRACTICES.md)
- [AgentMem主文档](../../README.md)

