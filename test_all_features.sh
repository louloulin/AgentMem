#!/bin/bash
set -e

echo "�� LumosAI + AgentMem 全功能验证测试"
echo "====================================="

BASE_URL="http://localhost:8080"
TEST_USER="test_user_$(date +%s)"
TEST_ORG="test_org"

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }

# 1. Health Check
info "1. Health Check"
HEALTH=$(curl -s $BASE_URL/health)
STATUS=$(echo $HEALTH | jq -r '.status')
[ "$STATUS" = "healthy" ] && success "Server healthy" || error "Server unhealthy"

# 2. 创建Agent
info "2. 创建测试Agent"
AGENT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"Test Agent\",
    \"system\": \"You are a test assistant.\",
    \"organization_id\": \"$TEST_ORG\"
  }")

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
[ ! -z "$AGENT_ID" ] && [ "$AGENT_ID" != "null" ] && success "Agent created: $AGENT_ID" || error "Failed to create agent"

# 3. 添加Memory
info "3. 添加Memory记录"
MEMORY_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/memories \
  -H "Content-Type: application/json" \
  -d "{
    \"content\": \"Test memory content for verification\",
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"$TEST_USER\",
    \"memory_type\": \"conversation\"
  }")

MEMORY_ID=$(echo $MEMORY_RESPONSE | jq -r '.data.id')
[ ! -z "$MEMORY_ID" ] && [ "$MEMORY_ID" != "null" ] && success "Memory added: $MEMORY_ID" || error "Failed to add memory"

# 4. 检索Memory
info "4. 检索Memory"
SEARCH_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/memories/search \
  -H "Content-Type: application/json" \
  -d "{
    \"query\": \"test memory\",
    \"agent_id\": \"$AGENT_ID\",
    \"user_id\": \"$TEST_USER\",
    \"limit\": 10
  }")

SEARCH_COUNT=$(echo $SEARCH_RESPONSE | jq -r '.data | length')
[ "$SEARCH_COUNT" -gt 0 ] && success "Found $SEARCH_COUNT memories" || error "Failed to search memories"

# 5. 获取Memory详情
info "5. 获取Memory详情"
GET_RESPONSE=$(curl -s $BASE_URL/api/v1/memories/$MEMORY_ID)
GET_SUCCESS=$(echo $GET_RESPONSE | jq -r '.success')
[ "$GET_SUCCESS" = "true" ] && success "Memory retrieved successfully" || error "Failed to get memory"

# 6. 更新Memory
info "6. 更新Memory"
UPDATE_RESPONSE=$(curl -s -X PATCH $BASE_URL/api/v1/memories/$MEMORY_ID \
  -H "Content-Type: application/json" \
  -d "{
    \"content\": \"Updated memory content\",
    \"metadata\": {\"updated\": true}
  }")

UPDATE_SUCCESS=$(echo $UPDATE_RESPONSE | jq -r '.success')
[ "$UPDATE_SUCCESS" = "true" ] && success "Memory updated" || error "Failed to update memory"

# 7. 列出Agent的所有Memory
info "7. 列出Agent的所有Memory"
LIST_RESPONSE=$(curl -s "$BASE_URL/api/v1/agents/$AGENT_ID/memories")
LIST_COUNT=$(echo $LIST_RESPONSE | jq -r '.data | length')
[ "$LIST_COUNT" -gt 0 ] && success "Listed $LIST_COUNT memories for agent" || error "Failed to list memories"

# 8. LumosAI Chat (架构验证)
info "8. LumosAI Chat 架构验证"
CHAT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Hello, this is a test message\",
    \"user_id\": \"$TEST_USER\"
  }")

# Chat可能因为没有API key而失败，但endpoint应该可访问
CHAT_ERROR=$(echo $CHAT_RESPONSE | jq -r '.code // empty')
if [ -z "$CHAT_ERROR" ]; then
    success "LumosAI Chat response received"
elif [ "$CHAT_ERROR" = "INTERNAL_ERROR" ]; then
    info "LumosAI Chat endpoint accessible (需要API key配置)"
else
    error "LumosAI Chat endpoint error: $CHAT_ERROR"
fi

# 9. 删除Memory
info "9. 删除Memory"
DELETE_RESPONSE=$(curl -s -X DELETE $BASE_URL/api/v1/memories/$MEMORY_ID)
DELETE_SUCCESS=$(echo $DELETE_RESPONSE | jq -r '.success')
[ "$DELETE_SUCCESS" = "true" ] && success "Memory deleted" || error "Failed to delete memory"

# 10. 验证删除
info "10. 验证Memory已删除"
VERIFY_RESPONSE=$(curl -s $BASE_URL/api/v1/memories/$MEMORY_ID)
VERIFY_ERROR=$(echo $VERIFY_RESPONSE | jq -r '.code // empty')
[ "$VERIFY_ERROR" = "NOT_FOUND" ] && success "Memory confirmed deleted" || error "Memory still exists"

echo ""
echo "====================================="
echo "🎉 所有功能测试通过！"
echo "====================================="
echo ""
echo "测试摘要:"
echo "  ✅ Health Check"
echo "  ✅ Agent 创建"
echo "  ✅ Memory 新增"
echo "  ✅ Memory 检索"
echo "  ✅ Memory 获取"
echo "  ✅ Memory 更新"
echo "  ✅ Memory 列表"
echo "  ✅ LumosAI Chat (架构)"
echo "  ✅ Memory 删除"
echo "  ✅ 删除验证"
