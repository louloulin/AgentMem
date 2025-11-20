#!/bin/bash
# LumosAI SSE 性能全面诊断脚本
# 基于 Zhipu 模型验证功能

set -e

API_BASE="${LUMOSAI_API:-http://localhost:8080}"
TOKEN="${LUMOSAI_TOKEN:-test-token}"
TEST_MESSAGE="介绍机器学习的基本概念"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo -e "${BLUE}🔍 LumosAI SSE 性能全面诊断${NC}"
echo "=========================================="
echo ""

# 1. 检查服务状态
echo -e "${YELLOW}📋 步骤1: 检查服务状态${NC}"
if curl -s "$API_BASE/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 服务运行正常${NC}"
else
    echo -e "${RED}❌ 服务未运行，请先启动服务${NC}"
    exit 1
fi
echo ""

# 2. 获取Agent信息
echo -e "${YELLOW}📋 步骤2: 获取Agent配置${NC}"
AGENT_ID=$(curl -s "$API_BASE/api/v1/agents" \
    -H "Authorization: Bearer $TOKEN" \
    | jq -r '.data[0].id // empty')

if [ -z "$AGENT_ID" ]; then
    echo -e "${RED}❌ 未找到Agent，请先创建Agent${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Agent ID: $AGENT_ID${NC}"

# 获取Agent配置
AGENT_CONFIG=$(curl -s "$API_BASE/api/v1/agents/$AGENT_ID" \
    -H "Authorization: Bearer $TOKEN" \
    | jq '.data')

MODEL=$(echo "$AGENT_CONFIG" | jq -r '.llm_config.model // "unknown"')
PROVIDER=$(echo "$AGENT_CONFIG" | jq -r '.llm_config.provider // "unknown"')

echo "   Provider: $PROVIDER"
echo "   Model: $MODEL"

# 检查模型类型
if [[ "$MODEL" == *"flash"* ]]; then
    echo -e "${GREEN}   ✅ 使用快速模型 (flash)${NC}"
    EXPECTED_TTFB=2000
elif [[ "$MODEL" == *"glm-4"* ]]; then
    echo -e "${YELLOW}   ⚠️  使用标准模型，可能较慢${NC}"
    EXPECTED_TTFB=20000
else
    echo -e "${YELLOW}   ⚠️  未知模型类型${NC}"
    EXPECTED_TTFB=10000
fi
echo ""

# 3. 性能测试
echo -e "${YELLOW}📋 步骤3: 执行性能测试${NC}"
echo "   测试消息: $TEST_MESSAGE"
echo ""

START_TIME=$(date +%s%N)
FIRST_CHUNK_TIME=""
CHUNK_COUNT=0
TOTAL_CONTENT=""

# 执行SSE请求
curl -N -X POST "$API_BASE/api/v1/agents/$AGENT_ID/chat/lumosai/stream" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"message\": \"$TEST_MESSAGE\", \"user_id\": \"perf-test\"}" \
    2>/dev/null | {
    
    while IFS= read -r line; do
        if [[ $line == data:* ]]; then
            JSON="${line#data: }"
            
            # 解析chunk_type
            CHUNK_TYPE=$(echo "$JSON" | jq -r '.chunk_type // empty' 2>/dev/null)
            
            if [ "$CHUNK_TYPE" == "content" ]; then
                CHUNK_COUNT=$((CHUNK_COUNT + 1))
                
                # 记录首个content chunk的时间
                if [ -z "$FIRST_CHUNK_TIME" ]; then
                    FIRST_CHUNK_TIME=$(date +%s%N)
                    TTFB=$(( (FIRST_CHUNK_TIME - START_TIME) / 1000000 ))
                    echo -e "${GREEN}   ⚡ TTFB (首字节时间): ${TTFB}ms${NC}"
                fi
                
                # 累积内容
                CONTENT=$(echo "$JSON" | jq -r '.content // ""' 2>/dev/null)
                if [ -n "$CONTENT" ]; then
                    TOTAL_CONTENT="${TOTAL_CONTENT}${CONTENT}"
                fi
            elif [ "$CHUNK_TYPE" == "done" ]; then
                END_TIME=$(date +%s%N)
                TOTAL_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
                break
            fi
        fi
    done
    
    # 输出结果
    echo ""
    echo -e "${BLUE}📊 性能指标:${NC}"
    echo "   TTFB: ${TTFB}ms"
    echo "   总耗时: ${TOTAL_TIME}ms"
    echo "   Chunk数量: ${CHUNK_COUNT}"
    echo "   内容长度: ${#TOTAL_CONTENT} 字符"
    echo ""
    
    # 性能评估
    echo -e "${BLUE}📈 性能评估:${NC}"
    if [ "$TTFB" -lt 2000 ]; then
        echo -e "${GREEN}   ✅ 优秀: TTFB < 2秒${NC}"
    elif [ "$TTFB" -lt 5000 ]; then
        echo -e "${GREEN}   ✅ 良好: TTFB < 5秒${NC}"
    elif [ "$TTFB" -lt 10000 ]; then
        echo -e "${YELLOW}   ⚠️  一般: TTFB < 10秒${NC}"
    else
        echo -e "${RED}   ❌ 较差: TTFB > 10秒${NC}"
    fi
    
    # 与预期对比
    if [ -n "$EXPECTED_TTFB" ]; then
        DIFF=$((TTFB - EXPECTED_TTFB))
        if [ "$DIFF" -gt 5000 ]; then
            echo -e "${YELLOW}   ⚠️  实际TTFB比预期慢 ${DIFF}ms${NC}"
        elif [ "$DIFF" -lt -1000 ]; then
            echo -e "${GREEN}   ✅ 实际TTFB比预期快 $(( -DIFF ))ms${NC}"
        else
            echo -e "${GREEN}   ✅ TTFB符合预期${NC}"
        fi
    fi
    echo ""
}

# 4. 瓶颈分析
echo -e "${YELLOW}📋 步骤4: 瓶颈分析${NC}"
echo ""

# 检查日志中的关键时间点
if [ -f "server-v4-traced.log" ] || [ -f "backend-streaming.log" ]; then
    LOG_FILE=""
    if [ -f "server-v4-traced.log" ]; then
        LOG_FILE="server-v4-traced.log"
    else
        LOG_FILE="backend-streaming.log"
    fi
    
    echo "   从日志分析关键时间点:"
    echo ""
    
    # 提取关键时间戳
    tail -100 "$LOG_FILE" | grep -E "⏱️|REAL-STREAMING|Memory|StreamingAgent" | tail -10 | while read line; do
        echo "   $line"
    done
else
    echo "   ⚠️  未找到日志文件，跳过日志分析"
    echo "   建议: 查看 server-v4-traced.log 或 backend-streaming.log"
fi
echo ""

# 5. 优化建议
echo -e "${YELLOW}📋 步骤5: 优化建议${NC}"
echo ""

if [ "$TTFB" -gt 10000 ]; then
    echo -e "${RED}❌ TTFB > 10秒，需要立即优化${NC}"
    echo ""
    echo "   优化方案:"
    echo "   1. ⭐⭐⭐⭐⭐ 更换模型为 glm-4-flash"
    echo "      UPDATE agents SET llm_config = jsonb_set(llm_config, '{model}', '\"glm-4-flash\"')"
    echo ""
    echo "   2. ⭐⭐⭐ 减少Memory检索数量"
    echo "      last_messages: Some(1)  # 从3减到1"
    echo ""
    echo "   3. ⭐⭐ 检查网络延迟"
    echo "      ping open.bigmodel.cn"
    echo ""
elif [ "$TTFB" -gt 5000 ]; then
    echo -e "${YELLOW}⚠️  TTFB > 5秒，建议优化${NC}"
    echo ""
    echo "   优化方案:"
    echo "   1. ⭐⭐⭐⭐ 检查是否使用快速模型"
    echo "      当前模型: $MODEL"
    if [[ ! "$MODEL" == *"flash"* ]]; then
        echo "      建议: 更换为 glm-4-flash"
    fi
    echo ""
    echo "   2. ⭐⭐⭐ 优化Memory配置"
    echo "      减少 last_messages 数量"
    echo ""
else
    echo -e "${GREEN}✅ TTFB < 5秒，性能良好${NC}"
    echo ""
    echo "   如需进一步优化:"
    echo "   1. Memory retrieve异步化"
    echo "   2. Agent Factory缓存"
    echo "   3. 减少Memory检索数量"
fi
echo ""

# 6. 详细分析报告
echo -e "${YELLOW}📋 步骤6: 生成详细报告${NC}"
echo ""
echo "   查看完整分析报告:"
echo "   - LUMOSAI_SSE_性能全面分析.md"
echo ""
echo "   查看相关文档:"
echo "   - TTFB瓶颈根本原因.md"
echo "   - 完整优化路线图.md"
echo "   - README_API_对比测试.md"
echo ""

echo "=========================================="
echo -e "${GREEN}✅ 诊断完成${NC}"
echo "=========================================="

