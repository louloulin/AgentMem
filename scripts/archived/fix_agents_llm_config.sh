#!/bin/bash

# 修复所有Agent的LLM配置脚本
# 为没有LLM配置的Agent添加默认的Zhipu配置

set -e

API_BASE="http://localhost:8080"
USER_ID="test-user"
ORG_ID="default-org"

echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║     🔧 Agent LLM配置修复工具 🔧                        ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# 1. 列出所有Agent
echo "📋 步骤1: 检查现有Agent..."
echo ""

AGENTS_JSON=$(curl -s "$API_BASE/api/v1/agents" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID")

echo "$AGENTS_JSON" | jq -r '.data[] | 
  "ID: \(.id)\n" + 
  "  Name: \(.name // "Unnamed")\n" + 
  "  Has LLM Config: \(if .llm_config then "✅ YES" else "❌ NO" end)\n" + 
  (if .llm_config then "  Provider: \(.llm_config.provider)\n  Model: \(.llm_config.model)\n" else "" end)'

echo ""
echo "─────────────────────────────────────────────────────────"
echo ""

# 2. 询问是否修复
read -p "❓ 是否为所有没有LLM配置的Agent添加默认配置(Zhipu/glm-4.6)? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ 取消操作"
    exit 0
fi

echo ""
echo "🔧 步骤2: 更新Agent配置..."
echo ""

# 3. 更新没有LLM配置的Agent
AGENT_IDS=$(echo "$AGENTS_JSON" | jq -r '.data[] | select(.llm_config == null) | .id')

if [ -z "$AGENT_IDS" ]; then
    echo "✅ 所有Agent都已配置LLM，无需更新！"
else
    COUNT=0
    for AGENT_ID in $AGENT_IDS; do
        echo "  更新 Agent: $AGENT_ID..."
        
        RESULT=$(curl -s -X PATCH "$API_BASE/api/v1/agents/$AGENT_ID" \
          -H "Content-Type: application/json" \
          -H "X-User-ID: $USER_ID" \
          -H "X-Organization-ID: $ORG_ID" \
          -d '{
            "llm_config": {
              "provider": "zhipu",
              "model": "glm-4.6"
            }
          }')
        
        if echo "$RESULT" | jq -e '.success' > /dev/null 2>&1; then
            echo "    ✅ 成功"
            ((COUNT++))
        else
            echo "    ❌ 失败: $(echo "$RESULT" | jq -r '.error // "Unknown error"')"
        fi
    done
    
    echo ""
    echo "✅ 已更新 $COUNT 个Agent"
fi

echo ""
echo "─────────────────────────────────────────────────────────"
echo ""

# 4. 验证更新后的Agent
echo "📋 步骤3: 验证更新后的Agent..."
echo ""

UPDATED_AGENTS=$(curl -s "$API_BASE/api/v1/agents" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID")

echo "$UPDATED_AGENTS" | jq -r '.data[] | 
  "ID: \(.id)\n" + 
  "  Name: \(.name // "Unnamed")\n" + 
  "  LLM: \(.llm_config.provider // "none")/\(.llm_config.model // "none")\n"'

echo ""
echo "─────────────────────────────────────────────────────────"
echo ""

# 5. 测试聊天功能
echo "🧪 步骤4: 测试聊天功能..."
echo ""

FIRST_AGENT_ID=$(echo "$UPDATED_AGENTS" | jq -r '.data[0].id')

if [ -z "$FIRST_AGENT_ID" ] || [ "$FIRST_AGENT_ID" == "null" ]; then
    echo "❌ 没有可用的Agent进行测试"
    exit 1
fi

echo "使用Agent: $FIRST_AGENT_ID"
echo "发送测试消息: '你好'"
echo ""

CHAT_RESPONSE=$(curl -s -X POST "$API_BASE/api/v1/agents/$FIRST_AGENT_ID/chat" \
  -H "Content-Type: application/json" \
  -H "X-User-ID: $USER_ID" \
  -H "X-Organization-ID: $ORG_ID" \
  -d '{
    "message": "你好",
    "stream": false
  }')

if echo "$CHAT_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    echo "✅ 聊天功能测试成功！"
    echo ""
    echo "📝 AI回复:"
    echo "$CHAT_RESPONSE" | jq -r '.data.response' | head -5
    echo ""
else
    echo "❌ 聊天功能测试失败"
    echo ""
    echo "错误信息:"
    echo "$CHAT_RESPONSE" | jq -r '.error // "Unknown error"'
    echo ""
fi

echo "─────────────────────────────────────────────────────────"
echo ""
echo "✨ 修复完成！"
echo ""
echo "📌 下一步操作:"
echo "  1. 访问 http://localhost:3001/admin/agents"
echo "  2. 创建新Agent时使用LLM配置功能"
echo "  3. 在 http://localhost:3001/admin/chat 选择Agent进行聊天"
echo ""
echo "🎯 测试Agent ID: $FIRST_AGENT_ID"
echo ""

