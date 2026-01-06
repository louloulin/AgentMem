#!/bin/bash

# 测试 Memory Managers 的脚本

set -e

echo "🧪 测试 Memory Managers..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 测试计数器
total_tests=0
passed_tests=0
failed_tests=0

# 测试 Episodic Memory
echo -e "${YELLOW}[1/2] 测试 Episodic Memory Manager...${NC}"
if cargo test -p agent-mem-core --lib managers::episodic_memory::tests --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Episodic Memory 测试通过${NC}"
    ((passed_tests++))
else
    echo -e "${RED}❌ Episodic Memory 测试失败${NC}"
    ((failed_tests++))
fi
((total_tests++))
echo ""

# 测试 Semantic Memory
echo -e "${YELLOW}[2/2] 测试 Semantic Memory Manager...${NC}"
if cargo test -p agent-mem-core --lib managers::semantic_memory::tests --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Semantic Memory 测试通过${NC}"
    ((passed_tests++))
else
    echo -e "${RED}❌ Semantic Memory 测试失败${NC}"
    ((failed_tests++))
fi
((total_tests++))
echo ""

# 总结
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 测试总结:"
echo "   总计: $total_tests managers"
echo "   通过: $passed_tests"
echo "   失败: $failed_tests"
if [ $total_tests -gt 0 ]; then
    success_rate=$((passed_tests * 100 / total_tests))
    echo "   成功率: ${success_rate}%"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $failed_tests -gt 0 ]; then
    exit 1
fi

echo -e "${GREEN}✅ 所有测试通过！${NC}"

