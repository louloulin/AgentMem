#!/bin/bash
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  🔬 记忆功能深度诊断                                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

BASE="http://localhost:8080"

# 加载配置
if [ -f .zhipu_test_config ]; then
  source .zhipu_test_config
else
  echo "❌ 配置文件不存在"
  exit 1
fi

echo "【1】检查Agent详细信息"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
AGENT_INFO=$(curl -s "$BASE/api/v1/agents/$AGENT_ID")
echo $AGENT_INFO | jq '{id, name, llm_config}'
echo ""

echo "【2】直接测试Memory API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "添加测试记忆..."
MEM=$(curl -s -X POST $BASE/api/v1/memories \
  -H "Content-Type: application/json" \
  -d "{
    \"content\":\"[user]: 诊断测试消息\",
    \"agent_id\":\"$AGENT_ID\",
    \"user_id\":\"$USER_ID\",
    \"metadata\":{\"role\":\"user\",\"source\":\"diagnostic\"}
  }")

if echo $MEM | jq -e '.success' > /dev/null 2>&1; then
  MEM_ID=$(echo $MEM | jq -r '.data.id')
  echo "✅ Memory API工作正常: $MEM_ID"
  
  # 验证记忆
  echo "验证记忆..."
  GET=$(curl -s "$BASE/api/v1/memories/$MEM_ID")
  if echo $GET | jq -e '.success' > /dev/null 2>&1; then
    echo "✅ 记忆可以读取"
    echo $GET | jq -r '.data.content'
  fi
  
  # 清理
  curl -s -X DELETE "$BASE/api/v1/memories/$MEM_ID" > /dev/null
else
  echo "❌ Memory API失败"
  echo $MEM | jq '.'
fi
echo ""

echo "【3】检查Agent的Memory配置"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Agent创建时的llm_config:"
echo $AGENT_INFO | jq '.llm_config'
echo ""

echo "【4】分析问题"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "可能的问题:"
echo "  1. ✅ Memory API 本身工作正常"
echo "  2. ❓ LumosAI Agent 可能没有正确绑定 memory"
echo "  3. ❓ generate_with_steps() 中的 memory.store() 可能失败"
echo "  4. ❓ Chat API 创建Agent时可能有问题"
echo ""

echo "【5】查看服务器日志"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ -f /tmp/server.log ]; then
  echo "最近的memory相关日志:"
  tail -50 /tmp/server.log | grep -i "memory\|store\|retrieve" | tail -10
else
  echo "⚠️  日志文件不存在"
fi
echo ""

echo "【建议】"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 检查 agent_factory.rs 中的 create_chat_agent()"
echo "2. 确认 Agent 创建时是否调用了 .memory(backend)"
echo "3. 添加更多日志到 executor.rs 和 memory_adapter.rs"
echo "4. 重新编译并测试"
