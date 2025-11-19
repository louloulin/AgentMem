#!/bin/bash

# 华为 MaaS LumosAI Chat 记忆功能验证脚本
# 测试流程：
# 1. 创建使用 LumosAI 的 Agent
# 2. 写入测试记忆
# 3. 通过 LumosAI Chat API 对话验证记忆检索

set -e

API_BASE="http://localhost:8080/api/v1"
AUTH_TOKEN="test-token"

echo "=========================================="
echo "🧪 LumosAI Chat 记忆功能验证测试"
echo "=========================================="
echo ""

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# ========================================
# 步骤 1: 检查或创建 Agent
# ========================================
echo -e "${BLUE}📋 步骤 1: 检查现有 Agent${NC}"

# 检查是否已有 Agent
EXISTING_AGENTS=$(curl -s -X GET "$API_BASE/agents" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json")

echo "$EXISTING_AGENTS" | jq '.'

# 尝试找到一个 Agent（使用第一个）
AGENT_ID=$(echo "$EXISTING_AGENTS" | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
  echo -e "${YELLOW}⚠️  未找到现有 Agent，创建新的测试 Agent${NC}"
  
  # 创建新 Agent
  CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/agents" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{
      "name": "LumosAI Memory Test Agent",
      "description": "用于测试 LumosAI 记忆功能",
      "system": "你是一个友好的 AI 助手，能够记住用户告诉你的信息。",
      "llm_config": {
        "provider": "zhipu",
        "model": "glm-4-flash",
        "temperature": 0.7
      }
    }')
  
  echo "$CREATE_RESPONSE" | jq '.'
  AGENT_ID=$(echo "$CREATE_RESPONSE" | jq -r '.data.id // .id')
  
  if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" == "null" ]; then
    echo -e "${RED}❌ 创建 Agent 失败${NC}"
    exit 1
  fi
  
  echo -e "${GREEN}✅ Agent 创建成功: $AGENT_ID${NC}"
else
  echo -e "${GREEN}✅ 使用现有 Agent: $AGENT_ID${NC}"
fi

echo ""

# ========================================
# 步骤 2: 写入测试记忆
# ========================================
echo -e "${BLUE}📝 步骤 2: 写入测试记忆${NC}"

# 记忆 1: 用户名字
echo "写入记忆 1: 用户名字..."
MEMORY1=$(curl -s -X POST "$API_BASE/memories" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"content\": \"用户的名字是张伟\",
    \"memory_type\": \"Episodic\",
    \"importance\": 0.9,
    \"metadata\": {
      \"source\": \"user_introduction\",
      \"category\": \"personal_info\"
    }
  }")

MEMORY1_ID=$(echo "$MEMORY1" | jq -r '.data.id // .id')
echo -e "${GREEN}✅ 记忆 1 已创建: $MEMORY1_ID${NC}"
echo "$MEMORY1" | jq '.data // .'

# 记忆 2: 职业
echo ""
echo "写入记忆 2: 职业信息..."
MEMORY2=$(curl -s -X POST "$API_BASE/memories" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"content\": \"用户是一名软件工程师，专注于 AI 开发\",
    \"memory_type\": \"Episodic\",
    \"importance\": 0.85,
    \"metadata\": {
      \"source\": \"user_introduction\",
      \"category\": \"career\"
    }
  }")

MEMORY2_ID=$(echo "$MEMORY2" | jq -r '.data.id // .id')
echo -e "${GREEN}✅ 记忆 2 已创建: $MEMORY2_ID${NC}"
echo "$MEMORY2" | jq '.data // .'

# 记忆 3: 爱好
echo ""
echo "写入记忆 3: 爱好信息..."
MEMORY3=$(curl -s -X POST "$API_BASE/memories" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"content\": \"用户喜欢打篮球和阅读科技书籍\",
    \"memory_type\": \"Episodic\",
    \"importance\": 0.75,
    \"metadata\": {
      \"source\": \"user_introduction\",
      \"category\": \"hobbies\"
    }
  }")

MEMORY3_ID=$(echo "$MEMORY3" | jq -r '.data.id // .id')
echo -e "${GREEN}✅ 记忆 3 已创建: $MEMORY3_ID${NC}"
echo "$MEMORY3" | jq '.data // .'

echo ""
echo -e "${GREEN}✅ 所有测试记忆已写入${NC}"
echo ""

# 等待向量索引（如果有）
echo "⏳ 等待 3 秒以确保记忆索引完成..."
sleep 3

# ========================================
# 步骤 3: 验证记忆检索
# ========================================
echo ""
echo -e "${BLUE}🔍 步骤 3: 验证记忆检索功能${NC}"

# 3.1 搜索记忆
echo ""
echo "3.1 搜索记忆: 查询'张伟'..."
SEARCH_RESULT=$(curl -s -X POST "$API_BASE/memories/search" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"query\": \"张伟\",
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"test-user\",
    \"limit\": 5
  }")

echo "$SEARCH_RESULT" | jq '.'

SEARCH_COUNT=$(echo "$SEARCH_RESULT" | jq '.data | length')
if [ "$SEARCH_COUNT" -gt 0 ]; then
  echo -e "${GREEN}✅ 记忆搜索成功，找到 $SEARCH_COUNT 条记忆${NC}"
else
  echo -e "${YELLOW}⚠️  记忆搜索未返回结果${NC}"
fi

# ========================================
# 步骤 4: LumosAI Chat API 测试
# ========================================
echo ""
echo -e "${BLUE}💬 步骤 4: LumosAI Chat API 对话测试${NC}"

# 4.1 测试问题 1: 询问名字
echo ""
echo "=========================================="
echo "测试 1: 询问用户名字"
echo "=========================================="
echo ""
echo "发送消息: '我叫什么名字？'"

CHAT1=$(curl -s -X POST "$API_BASE/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "我叫什么名字？",
    "user_id": "test-user"
  }')

echo "$CHAT1" | jq '.'

RESPONSE1=$(echo "$CHAT1" | jq -r '.data.content // .content')
echo ""
echo -e "${YELLOW}AI 回复:${NC}"
echo "$RESPONSE1"
echo ""

# 检查是否包含"张伟"
if echo "$RESPONSE1" | grep -q "张伟"; then
  echo -e "${GREEN}✅ 测试 1 通过: AI 正确回忆起用户名字${NC}"
else
  echo -e "${RED}❌ 测试 1 失败: AI 未能回忆起用户名字${NC}"
fi

echo ""
sleep 2

# 4.2 测试问题 2: 询问职业
echo ""
echo "=========================================="
echo "测试 2: 询问职业信息"
echo "=========================================="
echo ""
echo "发送消息: '我的职业是什么？'"

CHAT2=$(curl -s -X POST "$API_BASE/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "我的职业是什么？",
    "user_id": "test-user"
  }')

echo "$CHAT2" | jq '.'

RESPONSE2=$(echo "$CHAT2" | jq -r '.data.content // .content')
echo ""
echo -e "${YELLOW}AI 回复:${NC}"
echo "$RESPONSE2"
echo ""

# 检查是否包含"软件工程师"或"工程师"
if echo "$RESPONSE2" | grep -qE "软件工程师|工程师|AI.*开发"; then
  echo -e "${GREEN}✅ 测试 2 通过: AI 正确回忆起职业信息${NC}"
else
  echo -e "${RED}❌ 测试 2 失败: AI 未能回忆起职业信息${NC}"
fi

echo ""
sleep 2

# 4.3 测试问题 3: 询问爱好
echo ""
echo "=========================================="
echo "测试 3: 询问爱好"
echo "=========================================="
echo ""
echo "发送消息: '我有什么爱好？'"

CHAT3=$(curl -s -X POST "$API_BASE/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "我有什么爱好？",
    "user_id": "test-user"
  }')

echo "$CHAT3" | jq '.'

RESPONSE3=$(echo "$CHAT3" | jq -r '.data.content // .content')
echo ""
echo -e "${YELLOW}AI 回复:${NC}"
echo "$RESPONSE3"
echo ""

# 检查是否包含"篮球"或"阅读"
if echo "$RESPONSE3" | grep -qE "篮球|阅读|科技书"; then
  echo -e "${GREEN}✅ 测试 3 通过: AI 正确回忆起爱好信息${NC}"
else
  echo -e "${RED}❌ 测试 3 失败: AI 未能回忆起爱好信息${NC}"
fi

echo ""
sleep 2

# 4.4 测试问题 4: 综合问题
echo ""
echo "=========================================="
echo "测试 4: 综合信息查询"
echo "=========================================="
echo ""
echo "发送消息: '请总结一下你对我的了解'"

CHAT4=$(curl -s -X POST "$API_BASE/agents/$AGENT_ID/chat/lumosai" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "请总结一下你对我的了解",
    "user_id": "test-user"
  }')

echo "$CHAT4" | jq '.'

RESPONSE4=$(echo "$CHAT4" | jq -r '.data.content // .content')
echo ""
echo -e "${YELLOW}AI 回复:${NC}"
echo "$RESPONSE4"
echo ""

# 检查是否包含多个记忆要素
NAME_MATCH=$(echo "$RESPONSE4" | grep -c "张伟" || echo 0)
JOB_MATCH=$(echo "$RESPONSE4" | grep -cE "工程师|AI" || echo 0)
HOBBY_MATCH=$(echo "$RESPONSE4" | grep -cE "篮球|阅读" || echo 0)

TOTAL_MATCHES=$((NAME_MATCH + JOB_MATCH + HOBBY_MATCH))

if [ $TOTAL_MATCHES -ge 2 ]; then
  echo -e "${GREEN}✅ 测试 4 通过: AI 能够综合回忆多个记忆要素 ($TOTAL_MATCHES/3)${NC}"
else
  echo -e "${RED}❌ 测试 4 失败: AI 未能综合回忆记忆要素 ($TOTAL_MATCHES/3)${NC}"
fi

# ========================================
# 步骤 5: 查看 Agent 的所有记忆
# ========================================
echo ""
echo -e "${BLUE}📊 步骤 5: 查看 Agent 的所有记忆${NC}"

ALL_MEMORIES=$(curl -s -X GET "$API_BASE/agents/$AGENT_ID/memories?user_id=test-user" \
  -H "Authorization: Bearer $AUTH_TOKEN")

echo "$ALL_MEMORIES" | jq '.'

MEMORY_COUNT=$(echo "$ALL_MEMORIES" | jq '.data | length // 0')
echo ""
echo -e "${GREEN}✅ Agent 共有 $MEMORY_COUNT 条记忆${NC}"

# ========================================
# 测试总结
# ========================================
echo ""
echo "=========================================="
echo "🎉 测试完成总结"
echo "=========================================="
echo ""
echo "测试 Agent: $AGENT_ID"
echo "写入记忆数: 3"
echo "当前记忆总数: $MEMORY_COUNT"
echo ""
echo "记忆检索测试:"
echo "  - 搜索功能: $([ "$SEARCH_COUNT" -gt 0 ] && echo '✅ 通过' || echo '❌ 失败')"
echo ""
echo "LumosAI Chat 对话测试:"
echo "  - 测试 1 (名字): $(echo "$RESPONSE1" | grep -q "张伟" && echo '✅ 通过' || echo '❌ 失败')"
echo "  - 测试 2 (职业): $(echo "$RESPONSE2" | grep -qE "工程师|AI" && echo '✅ 通过' || echo '❌ 失败')"
echo "  - 测试 3 (爱好): $(echo "$RESPONSE3" | grep -qE "篮球|阅读" && echo '✅ 通过' || echo '❌ 失败')"
echo "  - 测试 4 (综合): $([ $TOTAL_MATCHES -ge 2 ] && echo '✅ 通过' || echo '❌ 失败')"
echo ""

# 清理提示
echo "💡 提示: 如需清理测试数据，可以手动删除创建的记忆："
echo "   curl -X DELETE $API_BASE/memories/$MEMORY1_ID -H 'Authorization: Bearer $AUTH_TOKEN'"
echo "   curl -X DELETE $API_BASE/memories/$MEMORY2_ID -H 'Authorization: Bearer $AUTH_TOKEN'"
echo "   curl -X DELETE $API_BASE/memories/$MEMORY3_ID -H 'Authorization: Bearer $AUTH_TOKEN'"
echo ""
echo "=========================================="
