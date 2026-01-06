#!/bin/bash
# 根据 x.md 演示计划创建30条演示记忆

set -e

API_URL="${API_URL:-http://localhost:8080}"
USER_ID="default"
AGENT_ID="${AGENT_ID:-}"

echo "=========================================="
echo "创建演示记忆 - 按照 x.md 计划"
echo "=========================================="
echo "API URL: $API_URL"
echo "User ID: $USER_ID"
echo ""

# 获取或创建 Agent
if [ -z "$AGENT_ID" ] || [ "$AGENT_ID" = "null" ]; then
    echo "1. 获取或创建 Agent..."
    AGENTS_RESPONSE=$(curl -s -X GET "$API_URL/api/v1/agents" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org")
    
    AGENT_COUNT=$(echo "$AGENTS_RESPONSE" | jq '.data | length' 2>/dev/null || echo "0")
    
    if [ "$AGENT_COUNT" -gt 0 ]; then
        AGENT_ID=$(echo "$AGENTS_RESPONSE" | jq -r '.data[0].id' 2>/dev/null || echo "")
        echo "   使用现有 Agent: $AGENT_ID"
    else
        CREATE_AGENT_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/agents" \
          -H "Content-Type: application/json" \
          -H "X-User-ID: $USER_ID" \
          -H "X-Organization-ID: default-org" \
          -d '{
            "name": "智能商务助手",
            "description": "为王总提供全方位服务的智能助手"
          }')
        AGENT_ID=$(echo "$CREATE_AGENT_RESPONSE" | jq -r '.data.id' 2>/dev/null || echo "")
        echo "   创建 Agent: $AGENT_ID"
    fi
fi

echo "Agent ID: $AGENT_ID"
echo ""

# 定义所有记忆（按照 x.md 计划）
declare -a MEMORIES=(
    # ========== 阶段1：基础信息（10条记忆）==========
    # 1. 用户基本信息
    '{"content":"王总是创新科技的CEO，40岁，管理100人团队，工作风格注重效率，喜欢简洁明了的沟通","importance":0.95,"memory_type":"Semantic"}'
    
    # 2. 工作风格和偏好
    '{"content":"王总的工作风格是注重效率，喜欢简洁明了的沟通方式，不喜欢冗长的会议和邮件","importance":0.85,"memory_type":"Semantic"}'
    
    # 3. 公司业务领域
    '{"content":"创新科技公司主要做AI产品开发，专注于人工智能技术的研发和应用","importance":0.90,"memory_type":"Semantic"}'
    
    # 4. 重要客户1：张总（华为）
    '{"content":"张总是华为公司的负责人，是创新科技的战略合作伙伴，张总喜欢喝绿茶","importance":0.90,"memory_type":"Semantic"}'
    
    # 5. 重要客户2：李总（腾讯）
    '{"content":"李总是腾讯公司的负责人，是创新科技的潜在客户，互联网行业","importance":0.85,"memory_type":"Semantic"}'
    
    # 6. 合作伙伴：刘总（阿里云）
    '{"content":"刘总是阿里云公司的负责人，是创新科技的云服务供应商","importance":0.80,"memory_type":"Semantic"}'
    
    # 7. 团队成员1：陈副总（CTO）
    '{"content":"陈副总是创新科技的CTO，专长技术架构，负责技术团队管理","importance":0.85,"memory_type":"Semantic"}'
    
    # 8. 团队成员2：周总（CMO）
    '{"content":"周总是创新科技的CMO，专长市场推广，负责市场营销工作","importance":0.85,"memory_type":"Semantic"}'
    
    # 9. 沟通偏好
    '{"content":"王总的沟通偏好是：会议时间安排在上午10点-12点，会议时长不超过1小时，偏好邮件+电话，不喜欢微信长聊","importance":0.80,"memory_type":"Semantic"}'
    
    # 10. 工作习惯
    '{"content":"王总的工作习惯：工作时间8:00-20:00，效率高峰是上午9-11点，休息时间是中午12-13点，出差频率每月2-3次","importance":0.75,"memory_type":"Semantic"}'
    
    # ========== 阶段2：项目信息（8条记忆）==========
    # 11. AI产品研发项目（基本信息）
    '{"content":"AI产品研发项目正在进行中，是创新科技的核心项目，使用Rust和Python开发","importance":0.90,"memory_type":"Semantic"}'
    
    # 12. AI产品研发项目（关键里程碑）
    '{"content":"AI产品研发项目已完成的关键里程碑包括：完成技术选型、完成MVP开发","importance":0.85,"memory_type":"Episodic"}'
    
    # 13. AI产品研发项目（重要决策）
    '{"content":"AI产品研发项目的重要决策：选择Rust作为核心语言，采用微服务架构，确定使用GPT-4作为基础模型，预算增加200万","importance":0.90,"memory_type":"Episodic"}'
    
    # 14. AI产品研发项目（风险点）
    '{"content":"AI产品研发项目的风险点：团队技术栈需要培训（特别是Rust），市场竞品较多，需要加快开发进度","importance":0.85,"memory_type":"Semantic"}'
    
    # 15. B轮融资项目（基本信息）
    '{"content":"B轮融资项目正在筹备中，目标融资5000万，估值5亿，计划下个月进行","importance":0.90,"memory_type":"Semantic"}'
    
    # 16. B轮融资项目（关键时间）
    '{"content":"B轮融资的关键时间点是下个月，需要在此之前完成所有准备工作","importance":0.85,"memory_type":"Episodic"}'
    
    # 17. B轮融资项目（准备事项）
    '{"content":"B轮融资的准备事项包括：BP完善、财务数据整理、投资人会议安排","importance":0.85,"memory_type":"Semantic"}'
    
    # 18. 技术评审会议记录
    '{"content":"上周三举行了AI产品技术评审会议，参与者包括陈副总和技术团队，关键决议：确定使用GPT-4作为基础模型，预算增加200万","importance":0.80,"memory_type":"Episodic"}'
    
    # ========== 阶段3：偏好和习惯（6条记忆）==========
    # 19. 会议偏好
    '{"content":"王总的会议偏好：会议时间安排在上午10点-12点，会议时长不超过1小时，沟通方式偏好邮件+电话","importance":0.75,"memory_type":"Semantic"}'
    
    # 20. 工作时间安排
    '{"content":"王总的工作时间安排：工作时间8:00-20:00，效率高峰是上午9-11点，休息时间是中午12-13点","importance":0.70,"memory_type":"Semantic"}'
    
    # 21. 效率高峰时间
    '{"content":"王总的效率高峰时间是上午9-11点，这个时间段最适合安排重要会议和决策事项","importance":0.75,"memory_type":"Semantic"}'
    
    # 22. 兴趣爱好（高尔夫）
    '{"content":"王总喜欢打高尔夫，周末经常去高尔夫球场，这是他放松和社交的重要方式","importance":0.70,"memory_type":"Semantic"}'
    
    # 23. 兴趣爱好（阅读）
    '{"content":"王总喜欢读商业传记和科技类书籍，经常在出差途中阅读，这是他获取灵感和知识的方式","importance":0.70,"memory_type":"Semantic"}'
    
    # 24. 咖啡习惯
    '{"content":"王总每天至少喝3杯美式咖啡，这是他保持工作效率的重要习惯","importance":0.65,"memory_type":"Semantic"}'
    
    # ========== 阶段4：历史交互（6条记忆）==========
    # 25. 会议记录1（技术评审）
    '{"content":"上周三的技术评审会议，参与者包括陈副总和技术团队，关键决议：确定使用GPT-4作为基础模型，预算增加200万，会议时长1小时","importance":0.80,"memory_type":"Episodic"}'
    
    # 26. 会议记录2（产品规划）
    '{"content":"上个月的产品规划会议，讨论了AI产品的市场定位和竞争策略，决定加快开发进度以应对市场竞争","importance":0.75,"memory_type":"Episodic"}'
    
    # 27. 重要对话1（融资计划）
    '{"content":"上个月讨论的B轮融资计划，目标融资5000万，估值5亿，计划下个月进行，需要完成BP完善、财务数据整理、投资人会议安排等准备工作","importance":0.85,"memory_type":"Episodic"}'
    
    # 28. 重要对话2（客户拜访）
    '{"content":"上周王总拜访了华为的张总，讨论了战略合作事宜，张总对AI产品很感兴趣，约定下个月再次会面详细讨论","importance":0.80,"memory_type":"Episodic"}'
    
    # 29. 客户偏好记录（张总-绿茶）
    '{"content":"华为的张总喜欢喝绿茶，在会面时应该准备绿茶，这是他个人的偏好和习惯","importance":0.70,"memory_type":"Semantic"}'
    
    # 30. 提醒事项记录
    '{"content":"重要提醒事项：下个月B轮融资需要完成准备工作，下个月与华为张总再次会面讨论合作，AI产品需要加快开发进度应对市场竞争","importance":0.80,"memory_type":"Semantic"}'
)

echo "2. 添加演示记忆（30条）..."
echo "----------------------------------------"

SUCCESS_COUNT=0
FAIL_COUNT=0

for i in "${!MEMORIES[@]}"; do
    MEMORY_JSON="${MEMORIES[$i]}"
    MEMORY_NUM=$((i + 1))
    
    # 添加 agent_id 和 user_id
    FULL_MEMORY=$(echo "$MEMORY_JSON" | jq --arg agent_id "$AGENT_ID" --arg user_id "$USER_ID" \
      '. + {agent_id: $agent_id, user_id: $user_id}')
    
    RESPONSE=$(curl -s -X POST "$API_URL/api/v1/memories" \
      -H "Content-Type: application/json" \
      -H "X-User-ID: $USER_ID" \
      -H "X-Organization-ID: default-org" \
      -d "$FULL_MEMORY")
    
    MEMORY_ID=$(echo "$RESPONSE" | jq -r '.data.id' 2>/dev/null || echo "")
    
    if [ -n "$MEMORY_ID" ] && [ "$MEMORY_ID" != "null" ]; then
        CONTENT=$(echo "$MEMORY_JSON" | jq -r '.content' | cut -c1-60)
        MEMORY_TYPE=$(echo "$MEMORY_JSON" | jq -r '.memory_type // "Semantic"')
        IMPORTANCE=$(echo "$MEMORY_JSON" | jq -r '.importance // 0.5')
        echo "   ✅ [$MEMORY_NUM] [$MEMORY_TYPE] (重要性: $IMPORTANCE) $CONTENT..."
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        echo "   ❌ [$MEMORY_NUM] 添加失败"
        echo "      响应: $RESPONSE"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
    
    # 稍微延迟，避免请求过快
    sleep 0.1
done

echo ""
echo "3. 验证记忆添加结果..."
echo "----------------------------------------"

DB_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND is_deleted = 0;" 2>/dev/null || echo "0")
SEMANTIC_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND memory_type = 'Semantic' AND is_deleted = 0;" 2>/dev/null || echo "0")
EPISODIC_COUNT=$(sqlite3 data/agentmem.db "SELECT COUNT(*) FROM memories WHERE user_id = '$USER_ID' AND memory_type = 'Episodic' AND is_deleted = 0;" 2>/dev/null || echo "0")

echo "数据库统计:"
echo "  总记忆数: $DB_COUNT"
echo "  语义记忆 (Semantic): $SEMANTIC_COUNT"
echo "  情景记忆 (Episodic): $EPISODIC_COUNT"

echo ""
echo "=========================================="
echo "创建完成"
echo "=========================================="
echo "✅ 成功添加: $SUCCESS_COUNT 条记忆"
echo "❌ 失败: $FAIL_COUNT 条记忆"
echo "📊 数据库记录: $DB_COUNT 条"
echo ""
echo "Agent ID: $AGENT_ID"
echo "User ID: $USER_ID"
echo ""
echo "📋 记忆分类:"
echo "  - 阶段1 基础信息: 10条"
echo "  - 阶段2 项目信息: 8条"
echo "  - 阶段3 偏好习惯: 6条"
echo "  - 阶段4 历史交互: 6条"
echo ""
echo "🎯 现在可以测试搜索功能："
echo "  - 搜索 '王总'、'AI产品'、'融资'、'张总'等"
echo "  - 在 UI 中访问: http://localhost:3001/admin/memories"
echo ""

