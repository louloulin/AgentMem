#!/bin/bash
# LumosAI-AgentMem 集成测试脚本

set -e

API="http://localhost:8080/api/v1"
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== LumosAI-AgentMem 集成测试 ===${NC}\n"

# 1. 创建测试Agent
echo -e "${BLUE}[1/5] 创建测试Agent...${NC}"
AGENT_RESPONSE=$(curl -s -X POST "$API/agents" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "LumosAI测试Agent",
    "description": "使用LumosAI引擎的测试Agent",
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4",
      "api_key": null
    },
    "system": "你是一个专业的AI助手，使用LumosAI引擎和AgentMem记忆系统。"
  }')

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')

if [ "$AGENT_ID" = "null" ] || [ -z "$AGENT_ID" ]; then
    echo -e "${RED}❌ 创建Agent失败${NC}"
    echo "$AGENT_RESPONSE" | jq '.'
    exit 1
fi

echo -e "${GREEN}✅ Agent创建成功: $AGENT_ID${NC}\n"

# 2. 测试传统Chat API (AgentOrchestrator)
echo -e "${BLUE}[2/5] 测试传统Chat API...${NC}"
TRADITIONAL_RESPONSE=$(curl -s -X POST "$API/agents/$AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好，请简单介绍一下你自己",
    "user_id": "test_user_traditional"
  }')

echo "传统API响应:"
echo "$TRADITIONAL_RESPONSE" | jq '{
  message_id: .data.message_id,
  content: .data.content[:100],
  memories_count: .data.memories_count,
  processing_time_ms: .data.processing_time_ms
}'

TRADITIONAL_SUCCESS=$(echo "$TRADITIONAL_RESPONSE" | jq -r '.success')
if [ "$TRADITIONAL_SUCCESS" = "true" ]; then
    echo -e "${GREEN}✅ 传统API工作正常${NC}\n"
else
    echo -e "${RED}❌ 传统API失败${NC}"
    echo "$TRADITIONAL_RESPONSE" | jq '.'
fi

# 3. 测试LumosAI Chat API (如果编译时启用了lumosai feature)
echo -e "${BLUE}[3/5] 测试LumosAI Chat API...${NC}"
LUMOSAI_RESPONSE=$(curl -s -X POST "$API/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "什么是AgentMem？",
    "user_id": "test_user_lumosai"
  }')

echo "LumosAI API响应:"
echo "$LUMOSAI_RESPONSE" | jq '.'

LUMOSAI_SUCCESS=$(echo "$LUMOSAI_RESPONSE" | jq -r '.success')
if [ "$LUMOSAI_SUCCESS" = "true" ]; then
    echo -e "${GREEN}✅ LumosAI API工作正常${NC}\n"
    
    # 显示完整响应
    echo "完整响应内容:"
    echo "$LUMOSAI_RESPONSE" | jq '{
      message_id: .data.message_id,
      content: .data.content,
      memories_updated: .data.memories_updated,
      memories_count: .data.memories_count,
      processing_time_ms: .data.processing_time_ms
    }'
elif echo "$LUMOSAI_RESPONSE" | jq -r '.error.message' | grep -q "not enabled"; then
    echo -e "${BLUE}ℹ️  LumosAI feature未启用（需要 --features lumosai 编译）${NC}\n"
else
    echo -e "${RED}❌ LumosAI API失败${NC}"
    echo "$LUMOSAI_RESPONSE" | jq '.'
fi

# 4. 验证记忆存储
echo -e "${BLUE}[4/5] 验证记忆存储...${NC}"

# 检查传统API的记忆
TRADITIONAL_MEMORIES=$(curl -s "$API/memories?user_id=test_user_traditional&limit=5")
TRAD_COUNT=$(echo "$TRADITIONAL_MEMORIES" | jq '.data | length')
echo "传统API记忆数量: $TRAD_COUNT"

# 检查LumosAI的记忆（如果启用）
if [ "$LUMOSAI_SUCCESS" = "true" ]; then
    LUMOSAI_MEMORIES=$(curl -s "$API/memories?user_id=test_user_lumosai&limit=5")
    LUMOS_COUNT=$(echo "$LUMOSAI_MEMORIES" | jq '.data | length')
    echo "LumosAI记忆数量: $LUMOS_COUNT"
    
    if [ "$LUMOS_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✅ LumosAI记忆正确存储${NC}\n"
        echo "最新记忆:"
        echo "$LUMOSAI_MEMORIES" | jq '.data[0] | {
          id: .id,
          content: .content[:100],
          importance: .importance,
          created_at: .created_at
        }'
    else
        echo -e "${RED}⚠️  LumosAI记忆未找到${NC}\n"
    fi
fi

# 5. 性能对比
echo -e "${BLUE}[5/5] 性能对比...${NC}"

if [ "$TRADITIONAL_SUCCESS" = "true" ]; then
    TRAD_TIME=$(echo "$TRADITIONAL_RESPONSE" | jq -r '.data.processing_time_ms')
    echo "传统API处理时间: ${TRAD_TIME}ms"
fi

if [ "$LUMOSAI_SUCCESS" = "true" ]; then
    LUMOS_TIME=$(echo "$LUMOSAI_RESPONSE" | jq -r '.data.processing_time_ms')
    echo "LumosAI处理时间: ${LUMOS_TIME}ms"
fi

# 清理测试Agent
echo -e "\n${BLUE}清理测试数据...${NC}"
curl -s -X DELETE "$API/agents/$AGENT_ID" > /dev/null
echo -e "${GREEN}✅ 测试完成${NC}"

echo -e "\n${BLUE}=== 测试总结 ===${NC}"
echo "传统API: $([ "$TRADITIONAL_SUCCESS" = "true" ] && echo -e "${GREEN}✅ 通过${NC}" || echo -e "${RED}❌ 失败${NC}")"
echo "LumosAI API: $([ "$LUMOSAI_SUCCESS" = "true" ] && echo -e "${GREEN}✅ 通过${NC}" || echo -e "${BLUE}ℹ️  未启用${NC}")"
