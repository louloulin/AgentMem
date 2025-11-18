#!/bin/bash
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  🚀 Zhipu AI + LumosAI Memory 真实测试配置                 ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

BASE="http://localhost:8080"

# 1. 检查环境变量
echo "【1】检查Zhipu API配置"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ ! -z "$ZHIPU_API_KEY" ]; then
  echo "✅ ZHIPU_API_KEY 已设置: ${ZHIPU_API_KEY:0:20}..."
else
  echo "⚠️  ZHIPU_API_KEY 未设置"
  echo ""
  echo "请设置环境变量:"
  echo "  export ZHIPU_API_KEY='your-api-key-here'"
  echo ""
  echo "获取API Key: https://open.bigmodel.cn/"
  echo ""
  read -p "请输入您的Zhipu API Key (或按Enter跳过): " API_KEY
  
  if [ ! -z "$API_KEY" ]; then
    export ZHIPU_API_KEY="$API_KEY"
    echo "✅ API Key已临时设置"
  else
    echo "⚠️  将使用模拟模式（不会真实调用LLM）"
  fi
fi
echo ""

# 2. 创建配置了Zhipu的Agent
echo "【2】创建Zhipu AI Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

AGENT=$(curl -s -X POST $BASE/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Zhipu Memory Agent",
    "system": "你是一个有记忆能力的AI助手。你能记住用户告诉你的信息，并在后续对话中使用这些信息。",
    "organization_id": "zhipu_test_org",
    "llm_config": {
      "provider": "zhipu",
      "model": "glm-4",
      "temperature": 0.7,
      "max_tokens": 2000
    }
  }')

AGENT_ID=$(echo $AGENT | jq -r '.data.id')

if [ "$AGENT_ID" = "null" ] || [ -z "$AGENT_ID" ]; then
  echo "❌ Agent创建失败"
  echo $AGENT | jq '.'
  exit 1
fi

echo "✅ Agent创建成功"
echo "   ID: $AGENT_ID"
echo "   Provider: zhipu"
echo "   Model: glm-4"
echo ""

# 3. 保存配置供后续使用
cat > .zhipu_test_config << EOF
AGENT_ID=$AGENT_ID
USER_ID=zhipu_test_user_$$
EOF

echo "✅ 配置已保存到 .zhipu_test_config"
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  ✅ Zhipu配置完成                                           ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "下一步: 运行 ./test_zhipu_memory.sh 进行真实对话测试"
