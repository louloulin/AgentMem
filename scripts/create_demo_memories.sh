#!/bin/bash
# 创建演示记忆 - 展示记忆平台的价值

set -e

API_URL="${API_URL:-http://localhost:8080}"
USER_ID="default"
AGENT_ID="${AGENT_ID:-}"

echo "=========================================="
echo "创建演示记忆 - 展示记忆平台价值"
echo "=========================================="
echo "API URL: $API_URL"
echo "User ID: $USER_ID"
echo ""

# 如果没有提供 Agent ID，先获取或创建一个
if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" = "null" ]; then
    echo "1. 获取或创建 Agent..."
    AGENTS_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/agents" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org")
    
    AGENT_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
    
    if [ "$AGENT_COUNT" -gt 0 ]; then
        AGENT_ID=$(echo "$AGENTS_RESPONSE" | jq -r '.data[0].id' 2>/dev/null || echo "")
        echo "   使用现有 Agent: $AGENT_ID"
    else
        echo "   创建新 Agent..."
        CREATE_AGENT_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/agents" \
          -H "Content-Type: application/json" \
          -H "X-User-ID: $USER_ID" \
          -H "X-Organization-ID: default-org" \
          -d '{
            "name": "Personal Assistant",
            "description": "演示记忆平台的个人助手"
          }')
        AGENT_ID=$(echo "$CREATE_AGENT_RESPONSE" | jq -r '.data.id' 2>/dev/null || echo "")
        echo "   创建 Agent: $AGENT_ID"
    fi
fi

echo "Agent ID: $AGENT_ID"
echo ""

# 定义演示记忆
declare -a MEMORIES=(
    # 用户基本信息
    '{"content":"用户名叫张明，是一位软件工程师，在北京工作","importance":0.9,"memory_type":"Semantic"}'
    '{"content":"张明今年28岁，喜欢编程和技术创新","importance":0.8,"memory_type":"Semantic"}'
    
    # 用户偏好和兴趣
    '{"content":"张明喜欢意大利菜，特别是披萨和意大利面","importance":0.85,"memory_type":"Semantic"}'
    '{"content":"张明喜欢户外运动，经常周末去爬山和徒步","importance":0.8,"memory_type":"Semantic"}'
    '{"content":"张明是咖啡爱好者，每天至少要喝两杯咖啡","importance":0.75,"memory_type":"Semantic"}'
    '{"content":"张明喜欢阅读科技类和历史类书籍","importance":0.7,"memory_type":"Semantic"}'
    
    # 工作和项目
    '{"content":"张明正在开发一个AI记忆系统项目，使用Rust和Python","importance":0.9,"memory_type":"Semantic"}'
    '{"content":"张明对机器学习和大语言模型很感兴趣","importance":0.85,"memory_type":"Semantic"}'
    '{"content":"张明的工作时间是周一到周五，9点到18点","importance":0.7,"memory_type":"Semantic"}'
    
    # 人际关系
    '{"content":"张明有一个好朋友叫李华，是产品经理","importance":0.8,"memory_type":"Semantic"}'
    '{"content":"张明的同事王芳是前端开发工程师，他们经常合作","importance":0.75,"memory_type":"Semantic"}'
    '{"content":"张明的女朋友叫小雨，是设计师，喜欢画画","importance":0.9,"memory_type":"Semantic"}'
    
    # 个人习惯和偏好
    '{"content":"张明习惯晚上11点睡觉，早上7点起床","importance":0.7,"memory_type":"Semantic"}'
    '{"content":"张明喜欢用MacBook Pro进行开发工作","importance":0.65,"memory_type":"Semantic"}'
    '{"content":"张明习惯使用VSCode作为代码编辑器","importance":0.6,"memory_type":"Semantic"}'
    
    # 生活场景
    '{"content":"张明计划下个月去日本旅行，想去东京和京都","importance":0.8,"memory_type":"Episodic"}'
    '{"content":"张明上周末去爬了香山，天气很好，风景很美","importance":0.7,"memory_type":"Episodic"}'
    '{"content":"张明最近在看《深入理解计算机系统》这本书","importance":0.65,"memory_type":"Episodic"}'
    
    # 对话上下文示例
    '{"content":"用户询问：什么是Rust语言？助手回答：Rust是一种系统编程语言，注重安全和性能","importance":0.6,"memory_type":"Semantic"}'
    '{"content":"用户询问：推荐一家好的意大利餐厅。助手推荐了Pizza Hut和Olive Garden","importance":0.7,"memory_type":"Episodic"}'
)

echo "2. 添加演示记忆..."
echo "----------------------------------------"

SUCCESS_COUNT=0
FAIL_COUNT=0

for i in "${!MEMORIES[@]}"; do
    MEMORY_JSON="${MEMORIES[$i]}"
    MEMORY_NUM=$((i + 1))
    
    # 添加 agent_id 和 user_id
    FULL_MEMORY=$(echo "$MEMORY_JSON" | jq --arg agent_id "$AGENT_ID" --arg user_id "$USER_ID" \
      '. + {agent_id: $agent_id, user_id: $user_id}')
    
    RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org" \
      -d "$FULL_MEMORY")
    
    MEMORY_ID=$(echo "$RESPONSE" | jq -r '.data.id' 2>/dev/null || echo "")
    
    if [ -n "$MEMORY_ID" ] && [ "$MEMORY_ID" != "null" ]; then
        CONTENT=$(echo "$MEMORY_JSON" | jq -r '.content' | cut -c1-50)
        echo "   ✅ [$MEMORY_NUM] $CONTENT..."
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        echo "   ❌ [$MEMORY_NUM] 添加失败"
        echo "      响应: $RESPONSE"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    
    # 稍微延迟，避免请求过快
    sleep 0.1
done

echo ""
echo "3. 验证记忆添加结果..."
echo "----------------------------------------"

DB_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND is_deleted = 0;" 2>/dev/null || echo "0")
echo "数据库中的记忆数量: $DB_COUNT"

echo ""
echo "=========================================="
echo "创建完成"
echo "=========================================="
echo "✅ 成功添加: $SUCCESS_COUNT 条记忆"
echo "❌ 失败: $FAIL_COUNT 条记忆"
echo "📊 数据库记录: $DB_COUNT 条"
echo ""
echo "Agent ID: $AGENT_ID"
echo "User ID: $USER_ID"
echo ""
echo "现在可以测试搜索功能："
echo "  - 搜索关键词: '张明'、'意大利'、'编程'、'咖啡'等"
echo "  - 在 UI 中访问: http://localhost:3001/admin/memories"
echo ""

