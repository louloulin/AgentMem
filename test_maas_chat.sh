#!/bin/bash
# AgentMem 华为 MaaS Chat 功能集成测试脚本

set -e

# --- 配置 ---
SERVER_URL="http://localhost:8000"
AUTH_TOKEN="test-token" # 根据你的认证配置修改

# --- 检查依赖 ---
echo "🔍 检查依赖..."
command -v curl >/dev/null 2>&1 || { echo >&2 "❌ 'curl' 未安装，请先安装。"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo >&2 "❌ 'jq' 未安装，请先安装。"; exit 1; }
echo "✅ 依赖检查通过"

# --- 检查环境变量 ---
echo "\n🔍 检查环境变量..."
if [ -z "$MAAS_API_KEY" ]; then
    echo "❌ 错误: 未设置 MAAS_API_KEY 环境变量。"
    echo "   请运行: export MAAS_API_KEY='your_huawei_maas_api_key'"
    exit 1
fi
echo "✅ MAAS_API_KEY 已设置"

# --- 1. 创建 MaaS Agent ---
echo "\n🚀 步骤 1: 创建 MaaS Agent..."
AGENT_NAME="MaaS Test Agent $(date +%s)"
AGENT_MODEL=${MAAS_MODEL:-"deepseek-v3.2-exp"}

AGENT_PAYLOAD=$(cat <<EOF
{
  "name": "$AGENT_NAME",
  "description": "测试华为 MaaS 集成",
  "system": "你是一个由华为MaaS驱动的AI助手。",
  "llm_config": {
    "provider": "maas",
    "model": "$AGENT_MODEL",
    "api_key": null
  }
}
EOF
)

AGENT_RESPONSE=$(curl -s -X POST "$SERVER_URL/api/v1/agents" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d "$AGENT_PAYLOAD")

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')

if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" == "null" ]; then
    echo "❌ 错误: 创建 Agent 失败。"
    echo "   响应: $AGENT_RESPONSE"
    exit 1
fi

echo "✅ Agent 创建成功!"
echo "   - Agent ID: $AGENT_ID"
echo "   - Agent Name: $AGENT_NAME"

# --- 2. 发送聊天消息 ---
echo "\n🚀 步骤 2: 发送聊天消息到 Agent..."
CHAT_MESSAGE="你好，请介绍一下你自己和你的模型。"

CHAT_PAYLOAD=$(cat <<EOF
{
  "message": "$CHAT_MESSAGE",
  "user_id": "maas-test-user"
}
EOF
)

CHAT_RESPONSE=$(curl -s -X POST "$SERVER_URL/api/v1/agents/$AGENT_ID/chat/lumosai" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d "$CHAT_PAYLOAD")

# --- 3. 验证响应 ---
echo "\n🚀 步骤 3: 验证响应..."
SUCCESS=$(echo "$CHAT_RESPONSE" | jq -r '.success')

if [ "$SUCCESS" != "true" ]; then
    echo "❌ 错误: Chat API 调用失败。"
    echo "   响应: $CHAT_RESPONSE"
    exit 1
fi

AI_REPLY=$(echo "$CHAT_RESPONSE" | jq -r '.data.content')
PROCESSING_TIME=$(echo "$CHAT_RESPONSE" | jq -r '.data.processing_time_ms')

echo "✅ Chat API 调用成功!"
echo "   - AI 回复: $AI_REPLY"
echo "   - 处理时间: ${PROCESSING_TIME}ms"

# --- 4. 检查 Memory 存储 (可选) ---
echo "\n🚀 步骤 4: 检查 Memory 是否存储 (可选)..."
MEMORIES_RESPONSE=$(curl -s -X GET "$SERVER_URL/api/v1/agents/$AGENT_ID/memories" \
  -H "Authorization: Bearer $AUTH_TOKEN")

MEMORY_COUNT=$(echo "$MEMORIES_RESPONSE" | jq -r '.data | length')

if [ "$MEMORY_COUNT" -ge 2 ]; then
    echo "✅ Memory 存储成功! 找到 $MEMORY_COUNT 条记忆。"
else
    echo "⚠️ 警告: 未能确认 Memory 存储。找到 $MEMORY_COUNT 条记忆。"
    echo "   响应: $MEMORIES_RESPONSE"
fi

# --- 清理 (可选) ---
# echo "\n🧹 清理: 删除 Agent..."
# curl -s -X DELETE "$SERVER_URL/api/v1/agents/$AGENT_ID" \
#   -H "Authorization: Bearer $AUTH_TOKEN"
echo "\n🎉🎉🎉 华为 MaaS Chat 集成测试成功! 🎉🎉🎉"

