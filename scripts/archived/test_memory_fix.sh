#!/bin/bash

set -e

BASE_URL="http://localhost:8080/api/v1"
AGENT_ID="agent-4cf4b0a4-ae12-4401-b7b6-3897d4c4b890"

echo "========================================="
echo "🧪 测试记忆功能修复"
echo "========================================="
echo ""

# 1. 检查现有记忆
echo "📊 步骤1: 检查现有记忆..."
EXISTING_MEMORIES=$(curl -s "${BASE_URL}/agents/${AGENT_ID}/memories?limit=10")
MEMORY_COUNT=$(echo "$EXISTING_MEMORIES" | jq -r '.data | length')
echo "   现有记忆数量: $MEMORY_COUNT"
if [ "$MEMORY_COUNT" -gt 0 ]; then
    echo "   现有记忆内容:"
    echo "$EXISTING_MEMORIES" | jq -r '.data[] | "   - " + .content' | head -3
fi
echo ""

# 2. 添加测试记忆（如果还没有）
if [ "$MEMORY_COUNT" -lt 3 ]; then
    echo "📝 步骤2: 添加测试记忆..."
    
    curl -s -X POST ${BASE_URL}/memories \
      -H "Content-Type: application/json" \
      -d "{
        \"agent_id\": \"$AGENT_ID\",
        \"content\": \"用户的名字叫小明，他是一名软件工程师，喜欢研究人工智能和机器学习技术\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.9
      }" > /dev/null
    echo "   ✅ 添加记忆1: 用户信息"
    
    curl -s -X POST ${BASE_URL}/memories \
      -H "Content-Type: application/json" \
      -d "{
        \"agent_id\": \"$AGENT_ID\",
        \"content\": \"小明最近在学习Rust编程语言，特别关注异步编程和内存管理\",
        \"memory_type\": \"Episodic\",
        \"importance\": 0.85
      }" > /dev/null
    echo "   ✅ 添加记忆2: 学习内容"
    
    curl -s -X POST ${BASE_URL}/memories \
      -H "Content-Type: application/json" \
      -d "{
        \"agent_id\": \"$AGENT_ID\",
        \"content\": \"AgentMem是一个基于Rust开发的智能记忆管理平台，支持向量搜索和持久化存储\",
        \"memory_type\": \"Semantic\",
        \"importance\": 0.95
      }" > /dev/null
    echo "   ✅ 添加记忆3: 项目信息"
    echo ""
else
    echo "📝 步骤2: 跳过添加记忆（已存在$MEMORY_COUNT条）"
    echo ""
fi

# 3. 测试聊天（应该使用记忆）
echo "💬 步骤3: 测试聊天功能..."
echo "   问题: 你知道我的名字和职业吗？"
echo ""

CHAT_RESPONSE=$(curl -s -X POST ${BASE_URL}/agents/${AGENT_ID}/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你知道我的名字和职业吗？",
    "user_id": "default"
  }')

# 检查响应
if [ $? -ne 0 ]; then
    echo "❌ 聊天请求失败"
    exit 1
fi

# 解析响应
CONTENT=$(echo "$CHAT_RESPONSE" | jq -r '.data.content')
MEMORIES_COUNT=$(echo "$CHAT_RESPONSE" | jq -r '.data.memories_count')
MEMORIES_UPDATED=$(echo "$CHAT_RESPONSE" | jq -r '.data.memories_updated')

echo "📤 AI回答:"
echo "---"
echo "$CONTENT"
echo "---"
echo ""

echo "📊 记忆使用统计:"
echo "   - memories_count: $MEMORIES_COUNT"
echo "   - memories_updated: $MEMORIES_UPDATED"
echo ""

# 4. 验证修复是否生效
echo "========================================="
echo "🎯 验证结果"
echo "========================================="

if echo "$CONTENT" | grep -qi "小明"; then
    echo "✅ 成功: AI回答中包含'小明'"
    echo "✅ 修复生效: MemoryEngine 成功从 LibSQL 读取记忆！"
    echo ""
    echo "🎉 测试通过！聊天功能已能正确使用记忆数据。"
    exit 0
else
    echo "❌ 失败: AI回答中未包含'小明'"
    echo "❌ 记忆功能仍未生效"
    echo ""
    echo "🔍 可能的原因:"
    echo "   1. 服务器未重新编译/重启"
    echo "   2. Agent的LLM配置有问题"
    echo "   3. 记忆检索逻辑仍有bug"
    echo ""
    echo "💡 建议:"
    echo "   1. 重新编译: cd agentmen && cargo build"
    echo "   2. 重启服务: ./start_server_with_correct_onnx.sh"
    echo "   3. 查看日志: 搜索 'Using LibSQL memory repository'"
    exit 1
fi

