#!/bin/bash
# 直接更新数据库中的模型配置

DB_PATH="./data/agentmem.db"

if [ ! -f "$DB_PATH" ]; then
    echo "❌ 数据库文件不存在: $DB_PATH"
    echo "🔍 搜索数据库文件..."
    find . -name "*.db" -type f 2>/dev/null | head -5
    exit 1
fi

echo "🔍 查询当前Agent配置..."
echo ""

# 查询当前agent
CURRENT=$(sqlite3 "$DB_PATH" "SELECT id, name, llm_config FROM agents LIMIT 1;")
echo "当前Agent: $CURRENT"
echo ""

# 提取agent_id
AGENT_ID=$(sqlite3 "$DB_PATH" "SELECT id FROM agents LIMIT 1;")

if [ -z "$AGENT_ID" ]; then
    echo "❌ 没有找到Agent"
    exit 1
fi

echo "✅ Agent ID: $AGENT_ID"
echo ""

# 更新模型配置
echo "🚀 更新模型为 glm-4-flash..."

sqlite3 "$DB_PATH" <<EOF
UPDATE agents 
SET llm_config = json_set(llm_config, '$.model', 'glm-4-flash')
WHERE id = '$AGENT_ID';
EOF

if [ $? -eq 0 ]; then
    echo "✅ 更新成功！"
    echo ""
    
    # 验证更新
    echo "📋 验证新配置:"
    NEW_CONFIG=$(sqlite3 "$DB_PATH" "SELECT llm_config FROM agents WHERE id = '$AGENT_ID';")
    echo "$NEW_CONFIG" | python3 -m json.tool 2>/dev/null || echo "$NEW_CONFIG"
else
    echo "❌ 更新失败"
    exit 1
fi

echo ""
echo "✅ 模型切换完成！"
echo ""
echo "⚠️  需要重启服务器使配置生效"
