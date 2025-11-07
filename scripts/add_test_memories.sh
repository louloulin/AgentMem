#!/bin/bash

# 添加测试记忆脚本
# 用于UI测试验证

API_BASE="http://localhost:8080"
TIMESTAMP=$(date +%s)

# 测试用户和Agent
TEST_USER_ID="test-user-zhangsan"
TEST_AGENT_ID="default-agent"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  添加测试记忆 - 用于UI验证                              ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# 1. 确保agent存在
echo "1️⃣  创建/确认测试Agent..."
curl -s -X POST "${API_BASE}/api/v1/agents" \
    -H "Content-Type: application/json" \
    -d "{
        \"id\": \"${TEST_AGENT_ID}\",
        \"name\": \"Default Agent\",
        \"description\": \"Default testing agent\",
        \"system_prompt\": \"You are a helpful assistant with excellent memory.\",
        \"model\": \"deepseek-chat\",
        \"temperature\": 0.7
    }" > /dev/null 2>&1

echo "✅ Agent准备完成"
echo ""

# 2. 添加关于"蒋"的记忆
echo "2️⃣  添加测试记忆..."

echo "  添加记忆1: 关于蒋的基本信息..."
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"蒋是项目负责人，负责AgentMem记忆管理平台的开发和维护\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9,
        \"tags\": [\"person\", \"role\"]
    }" | jq -r '.id // "添加成功"'

sleep 0.5

echo "  添加记忆2: 关于蒋的技能..."
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"蒋擅长Rust编程和AI系统架构设计，特别是记忆管理系统\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.85,
        \"tags\": [\"skill\", \"technical\"]
    }" | jq -r '.id // "添加成功"'

sleep 0.5

echo "  添加记忆3: 关于蒋的项目..."
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"蒋正在开发AgentMem，这是一个基于Rust的智能记忆管理平台，支持Episodic-first检索策略\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9,
        \"tags\": [\"project\", \"agentmem\"]
    }" | jq -r '.id // "添加成功"'

sleep 0.5

echo "  添加记忆4: 关于蒋的工作时间..."
curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories" \
    -H "Content-Type: application/json" \
    -d "{
        \"content\": \"蒋通常在上午9点到晚上9点工作，专注于代码开发和系统优化\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.7,
        \"tags\": [\"habit\", \"schedule\"]
    }" | jq -r '.id // "添加成功"'

echo ""
echo "✅ 测试记忆添加完成"
echo ""

# 3. 验证记忆已添加
echo "3️⃣  验证记忆..."
MEMORY_COUNT=$(curl -s "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories/stats" | jq -r '.total_memories // 0')
echo "  数据库中记忆数量: $MEMORY_COUNT"
echo ""

# 4. 测试搜索
echo "4️⃣  测试搜索功能..."
echo "  搜索'蒋是谁'..."
SEARCH_RESULT=$(curl -s -X POST "${API_BASE}/api/v1/agents/${TEST_AGENT_ID}/memories/search" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"蒋是谁\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"limit\": 5
    }")

RESULT_COUNT=$(echo "$SEARCH_RESULT" | jq -r '.results | length // 0')
echo "  搜索结果数量: $RESULT_COUNT"

if [ "$RESULT_COUNT" -gt 0 ]; then
    echo ""
    echo "  前3条结果:"
    echo "$SEARCH_RESULT" | jq -r '.results[0:3] | .[] | "  - " + .content' 2>/dev/null
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  ✅ 测试记忆添加和搜索完成！                            ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "现在您可以在UI上测试了："
echo ""
echo "1. 打开: http://localhost:3001/admin/chat"
echo "2. 确保使用 user_id: ${TEST_USER_ID}"
echo "3. 确保使用 agent_id: ${TEST_AGENT_ID}"
echo "4. 输入: 蒋是谁？"
echo "5. 期望: AI应该能够检索到关于蒋的记忆"
echo ""

