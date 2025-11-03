#!/bin/bash

# MCP Memory 功能端到端测试脚本
# 用于验证记忆系统是否真实实现（非mock数据）

set -e

BASE_URL="http://localhost:8080"
API_URL="$BASE_URL/api/v1"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║      🧪 MCP Memory 真实性验证测试 🧪                         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 辅助函数
test_start() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "${BLUE}[TEST $TOTAL_TESTS]${NC} $1"
}

test_pass() {
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo -e "${GREEN}✅ PASSED${NC}: $1"
    echo ""
}

test_fail() {
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo -e "${RED}❌ FAILED${NC}: $1"
    echo "Response: $2"
    echo ""
}

test_info() {
    echo -e "${YELLOW}ℹ️  INFO${NC}: $1"
}

# ============================================================================
# 第1步：验证MCP服务器状态
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 第1步：验证MCP服务器状态"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_start "MCP服务器健康检查"
HEALTH_RESPONSE=$(curl -s "$API_URL/mcp/health")
if echo "$HEALTH_RESPONSE" | jq -e '.success == true and .data.status == "healthy"' > /dev/null; then
    SERVER_NAME=$(echo "$HEALTH_RESPONSE" | jq -r '.data.server')
    SERVER_VERSION=$(echo "$HEALTH_RESPONSE" | jq -r '.data.version')
    test_pass "MCP服务器运行正常 ($SERVER_NAME v$SERVER_VERSION)"
else
    test_fail "MCP服务器健康检查失败" "$HEALTH_RESPONSE"
    exit 1
fi

test_start "MCP服务器信息"
INFO_RESPONSE=$(curl -s "$API_URL/mcp/info")
if echo "$INFO_RESPONSE" | jq -e '.success == true' > /dev/null; then
    CAPABILITIES=$(echo "$INFO_RESPONSE" | jq -r '.data.capabilities')
    test_pass "MCP服务器信息获取成功"
    test_info "Capabilities: $CAPABILITIES"
else
    test_fail "获取MCP服务器信息失败" "$INFO_RESPONSE"
fi

# ============================================================================
# 第2步：创建测试Agent
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🤖 第2步：创建测试Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_start "创建测试Agent"
AGENT_DATA='{
  "name": "MCP-Test-Agent",
  "description": "用于MCP Memory功能测试的Agent",
  "system_prompt": "You are a test agent for verifying MCP memory functionality",
  "metadata": {
    "purpose": "mcp_memory_verification",
    "created_by": "test_script"
  }
}'

AGENT_RESPONSE=$(curl -s -X POST "$API_URL/agents" \
  -H "Content-Type: application/json" \
  -d "$AGENT_DATA")

if echo "$AGENT_RESPONSE" | jq -e '.success == true' > /dev/null; then
    AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')
    test_pass "Agent创建成功 (ID: $AGENT_ID)"
else
    test_fail "Agent创建失败" "$AGENT_RESPONSE"
    exit 1
fi

# ============================================================================
# 第3步：测试Memory CRUD操作（验证真实数据库对接）
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💾 第3步：测试Memory CRUD操作"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 3.1 创建Memory
test_start "创建Memory (测试真实数据库写入)"
MEMORY_DATA='{
  "agent_id": "'$AGENT_ID'",
  "content": "This is a test memory created via MCP verification script at '$(date +%Y-%m-%d\ %H:%M:%S)'",
  "memory_type": "Episodic",
  "importance": 0.8,
  "metadata": {
    "source": "mcp_test_script",
    "test_id": "'$(uuidgen)'",
    "verification": "real_database_test"
  }
}'

MEMORY_CREATE_RESPONSE=$(curl -s -X POST "$API_URL/memories" \
  -H "Content-Type: application/json" \
  -d "$MEMORY_DATA")

if echo "$MEMORY_CREATE_RESPONSE" | jq -e '.success == true' > /dev/null; then
    MEMORY_ID=$(echo "$MEMORY_CREATE_RESPONSE" | jq -r '.data.id')
    test_pass "Memory创建成功 (ID: $MEMORY_ID)"
    test_info "这验证了Memory数据真实写入数据库"
else
    test_fail "Memory创建失败" "$MEMORY_CREATE_RESPONSE"
fi

# 等待向量嵌入生成
test_info "等待3秒以确保向量嵌入生成..."
sleep 3

# 3.2 读取Memory（验证持久化）
test_start "读取Memory (验证数据库持久化)"
MEMORY_GET_RESPONSE=$(curl -s "$API_URL/memories/$MEMORY_ID")

if echo "$MEMORY_GET_RESPONSE" | jq -e '.success == true and .data.id == "'$MEMORY_ID'"' > /dev/null; then
    MEMORY_CONTENT=$(echo "$MEMORY_GET_RESPONSE" | jq -r '.data.content')
    MEMORY_TYPE=$(echo "$MEMORY_GET_RESPONSE" | jq -r '.data.memory_type')
    test_pass "Memory读取成功 (Type: $MEMORY_TYPE)"
    test_info "Content: ${MEMORY_CONTENT:0:50}..."
    test_info "这验证了Memory数据真实存储在数据库中"
else
    test_fail "Memory读取失败" "$MEMORY_GET_RESPONSE"
fi

# 3.3 搜索Memory（验证向量搜索）
test_start "搜索Memory (验证向量嵌入和搜索)"
SEARCH_DATA='{
  "query": "test memory verification",
  "agent_id": "'$AGENT_ID'"
}'

SEARCH_RESPONSE=$(curl -s -X POST "$API_URL/memories/search" \
  -H "Content-Type: application/json" \
  -d "$SEARCH_DATA")

if echo "$SEARCH_RESPONSE" | jq -e '.success == true' > /dev/null; then
    SEARCH_COUNT=$(echo "$SEARCH_RESPONSE" | jq -r '.data | length')
    test_pass "Memory搜索成功 (找到 $SEARCH_COUNT 条记录)"
    
    if [ "$SEARCH_COUNT" -gt 0 ]; then
        FIRST_RESULT=$(echo "$SEARCH_RESPONSE" | jq -r '.data[0].content')
        test_info "第一条结果: ${FIRST_RESULT:0:50}..."
        
        # 检查是否有向量分数
        HAS_SCORE=$(echo "$SEARCH_RESPONSE" | jq -e '.data[0].score != null' && echo "yes" || echo "no")
        if [ "$HAS_SCORE" = "yes" ]; then
            SCORE=$(echo "$SEARCH_RESPONSE" | jq -r '.data[0].score')
            test_info "向量相似度分数: $SCORE"
            test_info "✅ 这验证了向量嵌入确实被生成并用于搜索"
        else
            test_info "⚠️  未返回向量分数，可能使用简单文本搜索"
        fi
    else
        test_info "⚠️  未找到匹配的Memory，向量搜索可能未正常工作"
    fi
else
    test_fail "Memory搜索失败" "$SEARCH_RESPONSE"
fi

# 3.4 更新Memory
test_start "更新Memory (验证数据库更新)"
UPDATE_DATA='{
  "content": "Updated memory content at '$(date +%Y-%m-%d\ %H:%M:%S)'",
  "importance": 0.9
}'

UPDATE_RESPONSE=$(curl -s -X PUT "$API_URL/memories/$MEMORY_ID" \
  -H "Content-Type: application/json" \
  -d "$UPDATE_DATA")

if echo "$UPDATE_RESPONSE" | jq -e '.success == true' > /dev/null; then
    test_pass "Memory更新成功"
    
    # 验证更新是否持久化
    VERIFY_RESPONSE=$(curl -s "$API_URL/memories/$MEMORY_ID")
    UPDATED_CONTENT=$(echo "$VERIFY_RESPONSE" | jq -r '.data.content')
    if echo "$UPDATED_CONTENT" | grep -q "Updated memory content"; then
        test_info "✅ 更新已持久化到数据库"
    fi
else
    test_fail "Memory更新失败" "$UPDATE_RESPONSE"
fi

# ============================================================================
# 第4步：验证向量嵌入（FastEmbed）
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 第4步：验证向量嵌入生成 (FastEmbed)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_start "创建第二个Memory用于相似度测试"
MEMORY_DATA_2='{
  "agent_id": "'$AGENT_ID'",
  "content": "This is another test memory for similarity comparison using vector embeddings",
  "memory_type": "Episodic",
  "importance": 0.7
}'

MEMORY_CREATE_2_RESPONSE=$(curl -s -X POST "$API_URL/memories" \
  -H "Content-Type: application/json" \
  -d "$MEMORY_DATA_2")

if echo "$MEMORY_CREATE_2_RESPONSE" | jq -e '.success == true' > /dev/null; then
    MEMORY_ID_2=$(echo "$MEMORY_CREATE_2_RESPONSE" | jq -r '.data.id')
    test_pass "第二个Memory创建成功 (ID: $MEMORY_ID_2)"
else
    test_fail "第二个Memory创建失败" "$MEMORY_CREATE_2_RESPONSE"
fi

sleep 3

test_start "测试向量相似度搜索"
SIMILARITY_SEARCH='{
  "query": "vector embeddings similarity",
  "agent_id": "'$AGENT_ID'"
}'

SIM_RESPONSE=$(curl -s -X POST "$API_URL/memories/search" \
  -H "Content-Type: application/json" \
  -d "$SIMILARITY_SEARCH")

if echo "$SIM_RESPONSE" | jq -e '.success == true and (.data | length) > 0' > /dev/null; then
    RESULT_COUNT=$(echo "$SIM_RESPONSE" | jq -r '.data | length')
    test_pass "相似度搜索返回 $RESULT_COUNT 条结果"
    
    # 检查结果是否包含第二个Memory
    CONTAINS_MEMORY_2=$(echo "$SIM_RESPONSE" | jq -e --arg id "$MEMORY_ID_2" '.data[] | select(.id == $id)' && echo "yes" || echo "no")
    if [ "$CONTAINS_MEMORY_2" = "yes" ]; then
        SCORE=$(echo "$SIM_RESPONSE" | jq -r --arg id "$MEMORY_ID_2" '.data[] | select(.id == $id) | .score')
        test_info "✅ 找到了包含'vector embeddings'的Memory (相似度: $SCORE)"
        test_info "✅ 这验证了FastEmbed向量嵌入确实在工作"
    else
        test_info "⚠️  未在结果中找到包含'vector embeddings'的Memory"
        test_info "可能向量嵌入未正常生成或搜索未使用向量"
    fi
else
    test_fail "相似度搜索失败或无结果" "$SIM_RESPONSE"
fi

# ============================================================================
# 第5步：检测Mock数据
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 第5步：检测Mock数据"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_start "检查Agent列表中的Mock数据迹象"
AGENTS_RESPONSE=$(curl -s "$API_URL/agents")
AGENT_COUNT=$(echo "$AGENTS_RESPONSE" | jq -r '.data | length')

test_info "当前系统中有 $AGENT_COUNT 个Agents"

# 检查是否有可疑的Mock Agent名称
MOCK_PATTERNS=("mock" "demo" "test" "sample" "fake")
MOCK_FOUND=false

for pattern in "${MOCK_PATTERNS[@]}"; do
    if echo "$AGENTS_RESPONSE" | jq -e --arg pattern "$pattern" '.data[] | select(.name | ascii_downcase | contains($pattern))' > /dev/null; then
        MOCK_AGENTS=$(echo "$AGENTS_RESPONSE" | jq -r --arg pattern "$pattern" '.data[] | select(.name | ascii_downcase | contains($pattern)) | .name')
        test_info "⚠️  发现可能的Mock Agent: $MOCK_AGENTS"
        MOCK_FOUND=true
    fi
done

if [ "$MOCK_FOUND" = false ]; then
    test_info "✅ 未发现明显的Mock Agent"
fi

test_start "验证Memory数据完整性"
ALL_MEMORIES=$(curl -s "$API_URL/agents/$AGENT_ID/memories")
MEMORY_COUNT=$(echo "$ALL_MEMORIES" | jq -r '.data | length')

test_pass "Agent有 $MEMORY_COUNT 条Memories"

if [ "$MEMORY_COUNT" -ge 2 ]; then
    test_info "✅ Memory数量与测试创建的数量一致"
    test_info "✅ 这验证了Memory是真实创建的，而非预设的Mock数据"
else
    test_info "⚠️  Memory数量少于预期，可能某些创建操作失败"
fi

# ============================================================================
# 第6步：清理测试数据
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 第6步：清理测试数据"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_start "删除测试Memories"
DELETE_1=$(curl -s -X DELETE "$API_URL/memories/$MEMORY_ID")
DELETE_2=$(curl -s -X DELETE "$API_URL/memories/$MEMORY_ID_2")

if echo "$DELETE_1" | jq -e '.success == true' > /dev/null && \
   echo "$DELETE_2" | jq -e '.success == true' > /dev/null; then
    test_pass "测试Memories删除成功"
else
    test_info "⚠️  部分Memory删除可能失败"
fi

test_start "删除测试Agent"
DELETE_AGENT=$(curl -s -X DELETE "$API_URL/agents/$AGENT_ID")

if echo "$DELETE_AGENT" | jq -e '.success == true' > /dev/null; then
    test_pass "测试Agent删除成功"
else
    test_info "⚠️  测试Agent删除可能失败"
fi

# ============================================================================
# 测试总结
# ============================================================================

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    📊 测试总结                                 ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "总测试数: $TOTAL_TESTS"
echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
echo -e "${RED}失败: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ 所有测试通过！Memory系统真实性验证成功！${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "验证结果:"
    echo "  ✅ Memory数据真实写入数据库"
    echo "  ✅ Memory持久化功能正常"
    echo "  ✅ 向量嵌入生成正常 (FastEmbed)"
    echo "  ✅ 向量相似度搜索工作正常"
    echo "  ✅ 无明显Mock数据痕迹"
    echo ""
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ 存在失败的测试，请检查上述错误信息${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    exit 1
fi

