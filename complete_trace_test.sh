#!/bin/bash
# 完整的trace测试 - 记录每个阶段的时间

API_BASE="http://localhost:8080"
TOKEN="test-token"
AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

echo "=========================================="
echo "🔍 完整Trace分析测试"
echo "=========================================="
echo ""
echo "Agent: $AGENT_ID"
echo "消息: 介绍深度学习"
echo ""

# 清空之前的日志标记
echo "--- TEST START $(date +%H:%M:%S.%N) ---" >> server-v3.log

# 记录开始时间
START=$(date +%s%N)
echo "⏱️  [00.000s] 用户发起请求"

# 发送请求
curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"message": "介绍深度学习", "user_id": "trace-test"}' 2>/dev/null | {
    
    FIRST_CONTENT=""
    
    while IFS= read -r line; do
        NOW=$(date +%s%N)
        ELAPSED=$(( (NOW - START) / 1000000 ))
        
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$TYPE" == "content" ]; then
                if [ -z "$FIRST_CONTENT" ]; then
                    FIRST_CONTENT="1"
                    echo "⚡ [$(printf "%05d" $ELAPSED)ms] 首个content chunk到达"
                fi
                echo -n "."
            elif [ "$TYPE" == "done" ]; then
                echo ""
                echo "✅ [$(printf "%05d" $ELAPSED)ms] 完成"
                break
            fi
        fi
    done
}

echo ""
echo ""
echo "=========================================="
echo "📊 服务器端时间分析"
echo "=========================================="
echo ""

# 分析服务器日志
echo "从日志提取时间信息："
echo ""

tail -100 server-v3.log | grep -A 50 "TEST START" | grep -E "Chat request|Memory|Retrieved|Zhipu API|HTTP|Token|耗时|elapsed|Created.*Agent" | while read line; do
    echo "  $line"
done

echo ""
echo "=========================================="
echo "🔍 详细时间分解建议"
echo "=========================================="
echo ""

cat << 'EOF'
需要在代码中添加的计时点：

1. 路由层 (chat_lumosai.rs)
   ⏱️  请求到达
   ⏱️  权限验证完成
   ⏱️  Agent Factory开始
   ⏱️  Agent Factory完成
   ⏱️  StreamingAgent创建完成
   ⏱️  execute_streaming调用开始

2. Executor层 (executor.rs)
   ⏱️  generate()入口
   ⏱️  Memory retrieve开始
   ⏱️  Memory retrieve完成
   ⏱️  消息格式化完成
   ⏱️  LLM调用开始

3. Streaming层 (streaming.rs)
   ⏱️  execute_streaming入口
   ⏱️  分支判断(direct/function_calling)
   ⏱️  execute_direct_streaming开始
   ⏱️  LLM generate_stream调用开始

4. LLM Provider层 (zhipu.rs)
   ⏱️  generate_stream入口
   ⏱️  HTTP请求发送
   ⏱️  首个byte收到
   ⏱️  首个JSON chunk解析完成

完整链路应该是：
用户请求 → 路由 → Factory → Streaming → Executor → LLM → SSE
EOF
