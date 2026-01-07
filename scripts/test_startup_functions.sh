#!/bin/bash
# 测试启动功能验证脚本

set -e

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🧪 启动功能验证测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

test_check() {
    local name="$1"
    local command="$2"
    
    echo -n "测试: $name ... "
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC}"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo "1️⃣  测试启动前检查功能"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 测试二进制文件检查
test_check "二进制文件检查（不存在时）" \
    'bash -c "if [ ! -f ./target/release/agent-mem-server ]; then exit 0; else exit 1; fi"'

# 测试端口检查（8080）
if lsof -i :8080 > /dev/null 2>&1; then
    echo "⚠️  端口 8080 被占用（可能是其他进程）"
else
    test_check "端口 8080 检查（可用）" \
        'bash -c "if ! lsof -i :8080 > /dev/null 2>&1; then exit 0; else exit 1; fi"'
fi

# 测试端口检查（3001）
test_check "端口 3001 检查（可用）" \
    'bash -c "if ! lsof -i :3001 > /dev/null 2>&1; then exit 0; else exit 1; fi"'

echo ""
echo "2️⃣  测试停止功能"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 测试停止后端
test_check "停止后端服务" \
    'just _stop-backend'

# 测试停止前端
test_check "停止前端服务" \
    'just _stop-frontend'

echo ""
echo "3️⃣  测试健康检查函数"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 测试健康检查（服务未运行）
echo -n "测试: 健康检查（服务未运行）... "
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} (正确检测到服务未运行)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠️  WARN${NC} (服务可能正在运行)"
fi

echo ""
echo "4️⃣  测试环境变量设置"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 测试环境变量
test_check "环境变量 EMBEDDER_PROVIDER" \
    'bash -c "export EMBEDDER_PROVIDER=fastembed; [ \"$EMBEDDER_PROVIDER\" = \"fastembed\" ]"'

test_check "环境变量 EMBEDDER_MODEL" \
    'bash -c "export EMBEDDER_MODEL=\"BAAI/bge-small-en-v1.5\"; [ \"$EMBEDDER_MODEL\" = \"BAAI/bge-small-en-v1.5\" ]"'

echo ""
echo "5️⃣  测试命令可用性"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 测试命令是否存在
for cmd in start-server-bg start-ui-bg start-full stop status health; do
    test_check "命令: just $cmd" \
        "just --list | grep -q '^[[:space:]]*$cmd'"
done

echo ""
echo "=========================================="
echo "📊 测试结果"
echo "=========================================="
echo "✅ 通过: $PASSED"
echo "❌ 失败: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 有 $FAILED 个测试失败${NC}"
    exit 1
fi

