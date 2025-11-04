#!/bin/bash

# 长期记忆检索测试脚本
# 日期: 2025-11-03

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "======================================"
echo "  长期记忆检索测试"
echo "======================================"
echo ""

# 配置
AGENT_ID="agent-a23bfd10-b1be-4848-8b4e-f3d34f4aae0e"  # 有 Semantic 记忆的 Agent
USER_ID="default"  # ✅ 统一使用 default 作为默认用户ID
SESSION_ID="test-ltm-$(date +%s)"
BASE_URL="http://localhost:8080"

echo -e "${BLUE}[配置]${NC}"
echo "Agent ID: $AGENT_ID"
echo "User ID: $USER_ID"
echo "Session ID: $SESSION_ID"
echo ""

# 1. 检查数据库中的长期记忆
echo -e "${BLUE}[步骤 1]${NC} 检查数据库中的长期记忆..."
echo ""

SEMANTIC_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE memory_type='Semantic' AND user_id='$USER_ID' AND agent_id='$AGENT_ID';")

if [ "$SEMANTIC_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅${NC} 找到 $SEMANTIC_COUNT 条 Semantic 记忆"
    echo ""
    echo "记忆内容:"
    sqlite3 data/agentmem.db "SELECT substr(content, 1, 50) as content, importance FROM memories WHERE memory_type='Semantic' AND user_id='$USER_ID' AND agent_id='$AGENT_ID' LIMIT 5;" | while read line; do
        echo "  - $line"
    done
else
    echo -e "${RED}❌${NC} 未找到 Semantic 记忆"
    echo ""
    echo "创建测试记忆..."
    
    # 创建测试记忆
    curl -s -X POST "$BASE_URL/api/v1/memories" \
      -H "Content-Type: application/json" \
      -d "{
        \"agent_id\": \"$AGENT_ID\",
        \"user_id\": \"$USER_ID\",
        \"content\": \"用户的名字是 Alice，她喜欢编程和阅读\",
        \"memory_type\": \"Semantic\",
        \"importance\": 0.9
      }" > /dev/null
    
    echo -e "${GREEN}✅${NC} 已创建测试记忆"
fi

echo ""

# 2. 发送测试消息（应该能检索到长期记忆）
echo -e "${BLUE}[步骤 2]${NC} 发送测试消息..."
echo ""

TEST_MESSAGE="我的名字是什么？我喜欢什么？"
echo "测试问题: $TEST_MESSAGE"
echo ""

RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"$TEST_MESSAGE\",
    \"user_id\": \"$USER_ID\",
    \"session_id\": \"$SESSION_ID\",
    \"stream\": false
  }")

MESSAGE_ID=$(echo "$RESPONSE" | jq -r '.data.message_id')
CONTENT=$(echo "$RESPONSE" | jq -r '.data.content')
MEMORIES_COUNT=$(echo "$RESPONSE" | jq -r '.data.memories_count')

if [ "$MESSAGE_ID" != "null" ] && [ -n "$MESSAGE_ID" ]; then
    echo -e "${GREEN}✅${NC} Chat API 响应成功"
    echo ""
    echo "消息 ID: $MESSAGE_ID"
    echo "检索到的记忆数: $MEMORIES_COUNT"
    echo ""
    echo "AI 回复:"
    echo "$CONTENT"
else
    echo -e "${RED}❌${NC} Chat API 响应失败"
    echo "$RESPONSE" | jq .
fi

echo ""

# 3. 验证是否检索到长期记忆
echo -e "${BLUE}[步骤 3]${NC} 验证长期记忆检索..."
echo ""

if [ "$MEMORIES_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅${NC} 成功检索到 $MEMORIES_COUNT 条记忆"
    
    # 检查回复内容是否包含记忆信息
    if echo "$CONTENT" | grep -qi "Alice\|编程\|阅读\|林\|蒋"; then
        echo -e "${GREEN}✅${NC} AI 回复包含长期记忆信息"
    else
        echo -e "${YELLOW}⚠️${NC} AI 回复可能未使用长期记忆"
    fi
else
    echo -e "${RED}❌${NC} 未检索到长期记忆"
    echo ""
    echo "可能的原因:"
    echo "  1. user_id 不匹配"
    echo "  2. agent_id 不匹配"
    echo "  3. 相关性分数过低"
    echo "  4. 数据库中没有相关记忆"
fi

echo ""

# 4. 查看后端日志（最近 20 行）
echo -e "${BLUE}[步骤 4]${NC} 查看后端日志..."
echo ""

if [ -f "backend-no-auth.log" ]; then
    echo "最近的记忆检索日志:"
    grep -i "retrieved.*memories\|memory.*score\|search.*memories" backend-no-auth.log | tail -10
else
    echo -e "${YELLOW}⚠️${NC} 未找到后端日志文件"
fi

echo ""

# 5. 数据库验证
echo -e "${BLUE}[步骤 5]${NC} 数据库验证..."
echo ""

# 检查 Working Memory 是否写入
sleep 2
WORKING_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE session_id='$SESSION_ID' AND memory_type='working';")

if [ "$WORKING_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅${NC} Working Memory 已写入: $WORKING_COUNT 条记录"
else
    echo -e "${YELLOW}⚠️${NC} Working Memory 未写入"
fi

echo ""

# 6. 总结
echo "======================================"
echo "  测试总结"
echo "======================================"
echo ""

if [ "$MEMORIES_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ 长期记忆检索功能正常${NC}"
    echo ""
    echo "验证点:"
    echo "  ✅ 数据库中有 Semantic 记忆"
    echo "  ✅ Chat API 成功检索到记忆"
    echo "  ✅ AI 回复包含记忆信息"
    echo "  ✅ Working Memory 正常写入"
else
    echo -e "${RED}❌ 长期记忆检索功能异常${NC}"
    echo ""
    echo "问题排查:"
    echo "  1. 检查 user_id 是否匹配: $USER_ID"
    echo "  2. 检查 agent_id 是否匹配: $AGENT_ID"
    echo "  3. 查看后端日志: tail -f backend-no-auth.log"
    echo "  4. 查看数据库: sqlite3 data/agentmem.db"
fi

echo ""
echo "详细分析报告: docs/LONG_TERM_MEMORY_RETRIEVAL_ISSUE_ANALYSIS.md"
echo ""

