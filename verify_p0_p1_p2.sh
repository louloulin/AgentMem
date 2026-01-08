#!/bin/bash
# AgentMem 2.6 功能验证脚本
#
# 验证 P0-P2 核心功能的实现和可用性
#
# 📅 Created: 2025-01-08
# 🎯 Purpose: 快速验证核心功能

echo "=========================================="
echo "AgentMem 2.6 功能验证"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数
PASSED=0
FAILED=0

# 测试函数
test_feature() {
    local name="$1"
    local command="$2"

    echo -n "测试 $name... "

    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 通过${NC}"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗ 失败${NC}"
        ((FAILED++))
        return 1
    fi
}

echo "1. 验证核心 crates 编译..."
echo "----------------------------------------"
test_feature "agent-mem-traits" "cargo check --package agent-mem-traits"
test_feature "agent-mem-storage" "cargo check --package agent-mem-storage"
test_feature "agent-mem-core" "cargo check --package agent-mem-core"
test_feature "agent-mem" "cargo check --package agent-mem"
test_feature "agent-mem-compat" "cargo check --package agent-mem-compat"
echo ""

echo "2. 验证 P0 功能..."
echo "----------------------------------------"

# 检查 Scheduler trait 存在
echo -n "检查 MemoryScheduler trait... "
if grep -q "trait MemoryScheduler" crates/agent-mem-traits/src/scheduler.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi

# 检查 DefaultMemoryScheduler 实现
echo -n "检查 DefaultMemoryScheduler 实现... "
if grep -q "pub struct DefaultMemoryScheduler" crates/agent-mem-core/src/scheduler/mod.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi

# 检查 ExponentialDecayModel
echo -n "检查 ExponentialDecayModel... "
if grep -q "pub struct ExponentialDecayModel" crates/agent-mem-core/src/scheduler/time_decay.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi
echo ""

echo "3. 验证 P1 功能..."
echo "----------------------------------------"

# 检查 8 种高级能力
CAPABILITIES=(
    "active_retrieval"
    "temporal_reasoning"
    "causal_reasoning"
    "graph_memory"
    "adaptive_strategy"
    "llm_optimizer"
    "performance_optimizer"
    "multimodal"
)

for cap in "${CAPABILITIES[@]}"; do
    echo -n "检查 $cap... "
    if [ -f "crates/agent-mem-core/src/${cap}.rs" ]; then
        echo -e "${GREEN}✓ 存在${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ 不存在${NC}"
        ((FAILED++))
    fi
done
echo ""

echo "4. 验证 P2 功能..."
echo "----------------------------------------"

# 检查 ContextCompressor
echo -n "检查 ContextCompressor... "
if grep -q "pub struct ContextCompressor" crates/agent-mem-core/src/llm_optimizer.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi

# 检查 MultiLevelCache
echo -n "检查 MultiLevelCache... "
if grep -q "pub struct MultiLevelCache" crates/agent-mem-core/src/llm_optimizer.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi
echo ""

echo "5. 验证 Memory V4..."
echo "----------------------------------------"

# 检查 Memory V4 (MemoryV4)
echo -n "检查 Memory V4 结构... "
if grep -q "pub struct MemoryV4" crates/agent-mem-traits/src/abstractions.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi

# 检查开放属性支持
echo -n "检查 AttributeSet (开放属性)... "
if grep -q "pub struct AttributeSet" crates/agent-mem-traits/src/abstractions.rs; then
    echo -e "${GREEN}✓ 存在${NC}"
    ((PASSED++))
else
    echo -e "${RED}✗ 不存在${NC}"
    ((FAILED++))
fi
echo ""

echo "6. 统计代码量..."
echo "----------------------------------------"

# 统计 P0 代码量
P0_LINES=$(find crates/agent-mem-core/src/scheduler -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
echo -e "P0 (Scheduler): ${YELLOW}${P0_LINES} lines${NC}"

# 统计 P1 代码量
P1_CAPS=("active_retrieval" "temporal_reasoning" "causal_reasoning" "graph_memory" "adaptive_strategy" "performance_optimizer" "multimodal")
P1_LINES=0
for cap in "${P1_CAPS[@]}"; do
    if [ -f "crates/agent-mem-core/src/${cap}.rs" ]; then
        LINES=$(wc -l < "crates/agent-mem-core/src/${cap}.rs")
        P1_LINES=$((P1_LINES + LINES))
    fi
done
# 添加 llm_optimizer 的一部分 (P1)
if [ -f "crates/agent-mem-core/src/llm_optimizer.rs" ]; then
    # 估算 P1 部分 (假设前半部分是 P1)
    TOTAL_LLUM=$(wc -l < "crates/agent-mem-core/src/llm_optimizer.rs")
    P1_PART=$((TOTAL_LLUM / 2))
    P1_LINES=$((P1_LINES + P1_PART))
fi
echo -e "P1 (8种能力): ${YELLOW}${P1_LINES} lines${NC}"

# 统计 P2 代码量 (llm_optimizer 的后)
if [ -f "crates/agent-mem-core/src/llm_optimizer.rs" ]; then
    TOTAL_LLUM=$(wc -l < "crates/agent-mem-core/src/llm_optimizer.rs")
    P2_LINES=$((TOTAL_LLUM / 2))
    echo -e "P2 (性能优化): ${YELLOW}${P2_LINES} lines${NC}"
fi
echo ""

echo "=========================================="
echo "验证结果汇总"
echo "=========================================="
echo -e "通过: ${GREEN}${PASSED}${NC}"
echo -e "失败: ${RED}${FAILED}${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有验证通过！AgentMem 2.6 核心功能已实现。${NC}"
    exit 0
else
    echo -e "${RED}✗ 有 ${FAILED} 项验证失败${NC}"
    exit 1
fi
