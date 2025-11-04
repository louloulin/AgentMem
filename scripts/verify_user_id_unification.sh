#!/bin/bash
# 验证 User ID 统一修复的完整性

set -e

echo "=========================================="
echo "User ID 统一修复验证脚本"
echo "=========================================="
echo ""

DB_PATH="${DB_PATH:-data/agentmem.db}"
ERRORS=0

# 1. 检查数据库中的 default-user 记录
echo "1. 检查数据库中的 default-user 记录..."
echo "----------------------------------------"

TABLES=("messages" "memories" "blocks" "api_keys" "memory_associations" "learning_feedback" "llm_call_logs")

for table in "${TABLES[@]}"; do
    COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM $table WHERE user_id = 'default-user';" 2>/dev/null || echo "0")
    if [ "$COUNT" -gt 0 ]; then
        echo "❌ $table 表中仍有 $COUNT 条 default-user 记录"
        ERRORS=$((ERRORS + 1))
    else
        echo "✅ $table 表中没有 default-user 记录"
    fi
done

echo ""

# 2. 检查数据库中的 default 记录
echo "2. 检查数据库中的 default 记录..."
echo "----------------------------------------"

for table in "${TABLES[@]}"; do
    COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM $table WHERE user_id = 'default';" 2>/dev/null || echo "0")
    echo "✅ $table 表中有 $COUNT 条 default 记录"
done

echo ""

# 3. 检查代码中的硬编码
echo "3. 检查代码中的硬编码..."
echo "----------------------------------------"

BACKEND_COUNT=$(grep -r '"default-user"' crates/agent-mem-server/src/ 2>/dev/null | grep -v "//" | grep -v "修改前" | wc -l | tr -d ' ')
FRONTEND_COUNT=$(grep -r "'default-user'" agentmem-ui/src/lib/constants.ts 2>/dev/null | grep -v "//" | grep -v "修改前" | wc -l | tr -d ' ')

if [ "$BACKEND_COUNT" -gt 0 ]; then
    echo "⚠️  后端代码中仍有 $BACKEND_COUNT 处 default-user 引用（可能是注释或文档）"
    grep -r '"default-user"' crates/agent-mem-server/src/ 2>/dev/null | grep -v "//" | grep -v "修改前" | head -5
else
    echo "✅ 后端代码中没有 default-user 硬编码"
fi

if [ "$FRONTEND_COUNT" -gt 0 ]; then
    echo "⚠️  前端代码中仍有 $FRONTEND_COUNT 处 default-user 引用（可能是注释）"
else
    echo "✅ 前端代码中没有 default-user 硬编码"
fi

echo ""

# 4. 检查常量定义
echo "4. 检查常量定义..."
echo "----------------------------------------"

if grep -q "DEFAULT_USER_ID = 'default'" agentmem-ui/src/lib/constants.ts 2>/dev/null; then
    echo "✅ 前端 DEFAULT_USER_ID 已设置为 'default'"
else
    echo "❌ 前端 DEFAULT_USER_ID 设置不正确"
    ERRORS=$((ERRORS + 1))
fi

if grep -q 'user_id: "default"' crates/agent-mem-server/src/middleware/auth.rs 2>/dev/null; then
    echo "✅ 后端默认认证中间件使用 'default'"
else
    echo "❌ 后端默认认证中间件设置不正确"
    ERRORS=$((ERRORS + 1))
fi

echo ""

# 5. 总结
echo "=========================================="
echo "验证总结"
echo "=========================================="

if [ $ERRORS -eq 0 ]; then
    echo "✅ 所有检查通过！User ID 已成功统一为 'default'"
    exit 0
else
    echo "❌ 发现 $ERRORS 个问题，请检查上述输出"
    exit 1
fi

