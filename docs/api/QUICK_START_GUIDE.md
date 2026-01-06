# AgentMem API 快速开始

**5分钟快速上手指南**

---

## 🎯 本指南目标

- ✅ 启动AgentMem服务
- ✅ 创建第一个Agent
- ✅ 添加和搜索记忆
- ✅ 进行对话交互

---

## ⚡ 快速开始（3步）

### Step 1: 启动服务

```bash
# 使用Docker Compose（推荐）
docker-compose -f docker-compose.prod.yml up -d

# 或使用cargo运行
cargo run -p agent-mem-server
```

**验证服务**:
```bash
curl http://localhost:8080/health
# 应返回: {"status":"healthy",...}
```

### Step 2: 创建Agent

```bash
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org-demo",
    "name": "My First Bot",
    "description": "A helpful assistant"
  }'
```

**响应**:
```json
{
  "id": "agent-abc123",
  "name": "My First Bot",
  "state": "active"
}
```

### Step 3: 添加记忆并搜索

```bash
# 添加记忆
curl -X POST http://localhost:8080/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "agent-abc123",
    "content": "用户喜欢吃披萨，尤其是意大利辣香肠口味",
    "importance": 0.8
  }'

# 搜索记忆
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "用户喜欢吃什么",
    "agent_id": "agent-abc123",
    "limit": 5
  }'
```

**🎉 完成！** 你已经掌握了基础用法！

---

## 📖 完整示例：智能客服机器人

### 场景描述
创建一个记住用户偏好的智能客服机器人。

### 1. 环境准备

```bash
# 设置环境变量
export BASE_URL="http://localhost:8080"
export AGENT_ID=""  # 稍后填入
```

### 2. 创建Agent

```bash
# 创建客服机器人
RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "customer-service",
    "name": "Customer Support Bot",
    "description": "处理客户咨询的智能助手"
  }')

# 提取Agent ID
export AGENT_ID=$(echo $RESPONSE | jq -r '.id')
echo "Agent ID: $AGENT_ID"
```

### 3. 添加客户记忆

```bash
# 添加多个客户偏好记忆
curl -X POST $BASE_URL/api/v1/memories/batch \
  -H "Content-Type: application/json" \
  -d '{
    "memories": [
      {
        "agent_id": "'$AGENT_ID'",
        "content": "客户张三，VIP会员，喜欢电子产品，特别是笔记本电脑",
        "importance": 0.9,
        "metadata": {"customer_name": "张三", "vip": "true"}
      },
      {
        "agent_id": "'$AGENT_ID'",
        "content": "张三上次购买了一台MacBook Pro，购买日期2023-10-01",
        "importance": 0.8,
        "metadata": {"customer_name": "张三", "purchase_date": "2023-10-01"}
      },
      {
        "agent_id": "'$AGENT_ID'",
        "content": "张三的配送地址是北京市朝阳区xxx街道",
        "importance": 0.7,
        "metadata": {"customer_name": "张三", "address_type": "delivery"}
      }
    ]
  }'
```

### 4. 查询客户信息

```bash
# 搜索客户偏好
curl -X POST $BASE_URL/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "张三喜欢什么产品",
    "agent_id": "'$AGENT_ID'",
    "limit": 5
  }' | jq .

# 搜索购买历史
curl -X POST $BASE_URL/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "张三的购买记录",
    "agent_id": "'$AGENT_ID'",
    "limit": 5
  }' | jq .
```

### 5. 与Agent对话

```bash
# 发送客服咨询
curl -X POST $BASE_URL/api/v1/agents/$AGENT_ID/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，我是张三，我想了解一下我上次买的电脑",
    "context": {
      "user_id": "zhang-san"
    }
  }' | jq .
```

**预期响应**:
```json
{
  "response": "你好张三！我看到您是我们的VIP会员。您上次在2023年10月1日购买了一台MacBook Pro。请问您对这台电脑有什么问题吗？",
  "agent_id": "agent-abc123",
  "timestamp": "2023-10-27T12:00:00Z"
}
```

---

## 🐍 Python完整示例

```python
import requests
import json

# 配置
BASE_URL = "http://localhost:8080"

class AgentMemClient:
    def __init__(self, base_url):
        self.base_url = base_url
        self.session = requests.Session()
        self.session.headers.update({'Content-Type': 'application/json'})
    
    def create_agent(self, name, description, org_id="default"):
        """创建Agent"""
        response = self.session.post(
            f"{self.base_url}/api/v1/agents",
            json={
                "organization_id": org_id,
                "name": name,
                "description": description
            }
        )
        return response.json()
    
    def add_memory(self, agent_id, content, importance=0.5, metadata=None):
        """添加记忆"""
        response = self.session.post(
            f"{self.base_url}/api/v1/memories",
            json={
                "agent_id": agent_id,
                "content": content,
                "importance": importance,
                "metadata": metadata or {}
            }
        )
        return response.json()
    
    def search_memories(self, agent_id, query, limit=10):
        """搜索记忆"""
        response = self.session.post(
            f"{self.base_url}/api/v1/memories/search",
            json={
                "query": query,
                "agent_id": agent_id,
                "limit": limit
            }
        )
        return response.json()
    
    def chat(self, agent_id, message, user_id=None):
        """与Agent对话"""
        response = self.session.post(
            f"{self.base_url}/api/v1/agents/{agent_id}/chat",
            json={
                "message": message,
                "context": {"user_id": user_id} if user_id else {}
            }
        )
        return response.json()

# 使用示例
def main():
    client = AgentMemClient(BASE_URL)
    
    # 1. 创建Agent
    print("📌 创建Agent...")
    agent = client.create_agent(
        name="智能助手",
        description="帮助用户的AI助手"
    )
    agent_id = agent['id']
    print(f"✅ Agent创建成功: {agent_id}")
    
    # 2. 添加记忆
    print("\n📌 添加记忆...")
    memories = [
        "用户喜欢看科幻电影，最喜欢的是《星际穿越》",
        "用户是一名软件工程师，专注于Python和机器学习",
        "用户每天早上7点起床，喜欢跑步"
    ]
    
    for content in memories:
        result = client.add_memory(agent_id, content, importance=0.8)
        print(f"✅ 记忆已添加: {result['id']}")
    
    # 3. 搜索记忆
    print("\n📌 搜索记忆...")
    results = client.search_memories(
        agent_id,
        query="用户的职业和兴趣是什么",
        limit=5
    )
    print(f"✅ 找到 {results['total']} 条相关记忆:")
    for r in results['results']:
        print(f"  - {r['content'][:50]}... (相似度: {r.get('similarity', 'N/A')})")
    
    # 4. 对话
    print("\n📌 与Agent对话...")
    response = client.chat(
        agent_id,
        message="你好，推荐一部电影给我",
        user_id="demo-user"
    )
    print(f"✅ Agent回复: {response['response']}")

if __name__ == "__main__":
    main()
```

**运行**:
```bash
python quick_start.py
```

---

## 🌐 JavaScript/TypeScript示例

```typescript
// agentmem-client.ts
import fetch from 'node-fetch';

interface Agent {
  id: string;
  name: string;
  state: string;
}

interface Memory {
  id: string;
  content: string;
  importance: number;
}

class AgentMemClient {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }

  async createAgent(name: string, description: string): Promise<Agent> {
    const response = await fetch(`${this.baseUrl}/api/v1/agents`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        organization_id: 'default',
        name,
        description
      })
    });
    return await response.json();
  }

  async addMemory(
    agentId: string,
    content: string,
    importance: number = 0.5
  ): Promise<Memory> {
    const response = await fetch(`${this.baseUrl}/api/v1/memories`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        agent_id: agentId,
        content,
        importance
      })
    });
    return await response.json();
  }

  async searchMemories(
    agentId: string,
    query: string,
    limit: number = 10
  ): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/v1/memories/search`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query,
        agent_id: agentId,
        limit
      })
    });
    return await response.json();
  }

  async chat(agentId: string, message: string): Promise<any> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/agents/${agentId}/chat`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message })
      }
    );
    return await response.json();
  }
}

// 使用示例
async function main() {
  const client = new AgentMemClient();

  // 创建Agent
  console.log('📌 创建Agent...');
  const agent = await client.createAgent(
    'My Bot',
    'A helpful assistant'
  );
  console.log(`✅ Agent创建成功: ${agent.id}`);

  // 添加记忆
  console.log('\n📌 添加记忆...');
  await client.addMemory(
    agent.id,
    '用户喜欢披萨',
    0.8
  );
  console.log('✅ 记忆已添加');

  // 搜索记忆
  console.log('\n📌 搜索记忆...');
  const results = await client.searchMemories(
    agent.id,
    '用户喜欢吃什么'
  );
  console.log(`✅ 找到 ${results.total} 条记忆`);

  // 对话
  console.log('\n📌 对话...');
  const response = await client.chat(
    agent.id,
    '你好'
  );
  console.log(`✅ Agent回复: ${response.response}`);
}

main();
```

---

## 🧪 测试API

### 使用Swagger UI（推荐）

1. 打开浏览器访问: http://localhost:8080/swagger-ui
2. 选择任意端点
3. 点击 "Try it out"
4. 填写参数
5. 点击 "Execute"

### 使用Postman

1. 导入OpenAPI规范: http://localhost:8080/api-docs/openapi.json
2. 设置环境变量:
   - `base_url`: `http://localhost:8080`
   - `agent_id`: 你的Agent ID
3. 开始测试！

### 使用cURL脚本

```bash
#!/bin/bash
# test_api.sh

BASE_URL="http://localhost:8080"

echo "🧪 测试AgentMem API"

# 1. 健康检查
echo -e "\n1️⃣ 健康检查"
curl -s $BASE_URL/health | jq .

# 2. 创建Agent
echo -e "\n2️⃣ 创建Agent"
AGENT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{"organization_id":"test","name":"Test Bot","description":"For testing"}')
echo $AGENT_RESPONSE | jq .
AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.id')

# 3. 添加记忆
echo -e "\n3️⃣ 添加记忆"
curl -s -X POST $BASE_URL/api/v1/memories \
  -H "Content-Type: application/json" \
  -d '{"agent_id":"'$AGENT_ID'","content":"测试记忆","importance":0.8}' | jq .

# 4. 搜索记忆
echo -e "\n4️⃣ 搜索记忆"
curl -s -X POST $BASE_URL/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{"query":"测试","agent_id":"'$AGENT_ID'","limit":10}' | jq .

echo -e "\n✅ 测试完成！"
```

**运行**:
```bash
chmod +x test_api.sh
./test_api.sh
```

---

## 🔍 故障排除

### 问题1: 服务无法启动

```bash
# 检查端口是否被占用
lsof -i :8080

# 杀掉占用端口的进程
kill -9 <PID>
```

### 问题2: 连接被拒绝

```bash
# 检查服务状态
curl -v http://localhost:8080/health

# 查看日志
docker logs agentmem-server
# 或
cargo run -p agent-mem-server 2>&1 | tail -50
```

### 问题3: 记忆搜索无结果

**原因**: 可能是向量化未完成或相似度阈值过高

**解决**:
```bash
# 降低相似度阈值
curl -X POST http://localhost:8080/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "你的查询",
    "agent_id": "agent-id",
    "threshold": 0.3
  }'
```

---

## 📚 下一步

- 📖 查看[完整API参考](./API_REFERENCE.md)
- 🚀 阅读[生产部署指南](../deployment/PRODUCTION_DEPLOYMENT_GUIDE.md)
- 🔧 探索[高级配置](../configuration/ADVANCED_CONFIGURATION.md)
- 💡 查看[示例项目](../../examples/)

---

## 🆘 获取帮助

- **文档**: https://agentmem.cc
- **GitHub Issues**: https://github.com/louloulin/agentmem/issues
- **Discord**: https://discord.gg/agentmem
- **Email**: support@agentmem.dev

---

**版本**: v2.0.0  
**更新**: 2025-10-27

