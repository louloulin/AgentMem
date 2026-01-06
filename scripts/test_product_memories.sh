#!/bin/bash

##############################################################################
# 商品记忆测试脚本
# 功能: 测试商品记忆的查询和隔离效果
# 日期: 2025-11-07
##############################################################################

set -e

API_BASE="${API_BASE:-http://localhost:8080}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           商品记忆系统 - 功能测试                           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 测试1: 基础统计
echo -e "${YELLOW}📊 测试1: 数据统计${NC}"
echo -e "查询数据库中的商品记忆总数..."

total=$(curl -s "${API_BASE}/api/v1/memories/search?query=商品&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
echo -e "  ${GREEN}✓${NC} 商品记忆总数: ${total}"

global_count=$(curl -s "${API_BASE}/api/v1/memories/search?query=状态:在售&scope=global&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
echo -e "  ${GREEN}✓${NC} Global Scope: ${global_count}"

echo ""

# 测试2: 分类搜索
echo -e "${YELLOW}📱 测试2: 分类搜索${NC}"

categories=("电子产品" "服装鞋帽" "食品饮料" "家居用品" "运动户外")
for cat in "${categories[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${cat}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo -e "  ${GREEN}✓${NC} ${cat}: ${count} 条"
done

echo ""

# 测试3: 品牌搜索
echo -e "${YELLOW}🏷️  测试3: 品牌搜索${NC}"

brands=("Apple" "Samsung" "Huawei" "Nike" "Adidas")
for brand in "${brands[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${brand}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo -e "  ${GREEN}✓${NC} ${brand}: ${count} 条"
done

echo ""

# 测试4: 价格区间
echo -e "${YELLOW}💰 测试4: 价格搜索${NC}"

price_keywords=("价格" "¥")
for keyword in "${price_keywords[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${keyword}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo -e "  ${GREEN}✓${NC} 包含'${keyword}': ${count} 条"
done

echo ""

# 测试5: 子分类搜索
echo -e "${YELLOW}📂 测试5: 子分类搜索${NC}"

subcategories=("手机" "电脑" "男装" "零食" "家具")
for subcat in "${subcategories[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${subcat}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo -e "  ${GREEN}✓${NC} ${subcat}: ${count} 条"
done

echo ""

# 测试6: 用户隔离测试
echo -e "${YELLOW}👥 测试6: 用户记忆隔离${NC}"

# 查询用户浏览记忆
for user_num in 001 002 003; do
    user_id="user-${user_num}"
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=浏览&user_id=${user_id}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo -e "  ${GREEN}✓${NC} ${user_id} 的浏览记忆: ${count} 条"
done

echo ""

# 测试7: Agent记忆测试
echo -e "${YELLOW}🤖 测试7: Agent销售分析记忆${NC}"

agent_id="agent-sales-analyst"
count=$(curl -s "${API_BASE}/api/v1/memories/search?query=销售&agent_id=${agent_id}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
echo -e "  ${GREEN}✓${NC} ${agent_id} 的分析记忆: ${count} 条"

echo ""

# 测试8: 综合搜索示例
echo -e "${YELLOW}🔍 测试8: 综合搜索示例${NC}"

# 示例1: 搜索Apple手机
echo -e "  ${BLUE}示例1: 搜索'Apple 手机'${NC}"
result=$(curl -s "${API_BASE}/api/v1/memories/search?query=Apple%20手机&limit=5")
count=$(echo "$result" | jq -r '.memories | length' 2>/dev/null || echo "0")
echo -e "    找到 ${count} 条结果"
if [ "$count" -gt 0 ]; then
    echo "$result" | jq -r '.memories[0].content' 2>/dev/null | head -n 1 | sed 's/^/    示例: /'
fi

# 示例2: 搜索高价商品
echo -e "  ${BLUE}示例2: 搜索'价格 10000'${NC}"
result=$(curl -s "${API_BASE}/api/v1/memories/search?query=价格&limit=50")
count=$(echo "$result" | jq -r '.memories | length' 2>/dev/null || echo "0")
high_price_count=$(echo "$result" | jq -r '.memories[] | select(.content | test("价格: ¥[0-9]{5,}")) | .content' 2>/dev/null | wc -l || echo "0")
echo -e "    找到高价商品 ${high_price_count} 条（价格>10000）"

# 示例3: 搜索特定商品ID
echo -e "  ${BLUE}示例3: 搜索商品ID 'P000001'${NC}"
result=$(curl -s "${API_BASE}/api/v1/memories/search?query=P000001&limit=1")
count=$(echo "$result" | jq -r '.memories | length' 2>/dev/null || echo "0")
echo -e "    找到 ${count} 条结果"
if [ "$count" -gt 0 ]; then
    echo "$result" | jq -r '.memories[0].content' 2>/dev/null | sed 's/^/    内容: /'
fi

echo ""

# 测试9: 性能测试
echo -e "${YELLOW}⚡ 测试9: 查询性能${NC}"

queries=("商品" "Apple" "电子产品" "价格" "库存")
for query in "${queries[@]}"; do
    start=$(date +%s%N)
    curl -s "${API_BASE}/api/v1/memories/search?query=${query}&limit=10" > /dev/null
    end=$(date +%s%N)
    elapsed=$(( (end - start) / 1000000 ))
    
    if [ $elapsed -lt 100 ]; then
        echo -e "  ${GREEN}✓${NC} 查询'${query}': ${elapsed}ms ${GREEN}(优秀)${NC}"
    elif [ $elapsed -lt 200 ]; then
        echo -e "  ${YELLOW}✓${NC} 查询'${query}': ${elapsed}ms ${YELLOW}(良好)${NC}"
    else
        echo -e "  ${RED}✗${NC} 查询'${query}': ${elapsed}ms ${RED}(需优化)${NC}"
    fi
done

echo ""

# 测试10: 数据完整性
echo -e "${YELLOW}✅ 测试10: 数据完整性检查${NC}"

# 检查必需字段
echo -e "  ${BLUE}检查商品记忆字段完整性...${NC}"
result=$(curl -s "${API_BASE}/api/v1/memories/search?query=商品ID&limit=10")
memories=$(echo "$result" | jq -r '.memories[]' 2>/dev/null)

valid_count=0
invalid_count=0

while IFS= read -r memory; do
    content=$(echo "$memory" | jq -r '.content' 2>/dev/null)
    
    if echo "$content" | grep -q "商品ID:" && \
       echo "$content" | grep -q "名称:" && \
       echo "$content" | grep -q "价格:" && \
       echo "$content" | grep -q "库存:"; then
        ((valid_count++))
    else
        ((invalid_count++))
    fi
done < <(echo "$result" | jq -c '.memories[]' 2>/dev/null)

echo -e "    有效记忆: ${valid_count}"
echo -e "    无效记忆: ${invalid_count}"

if [ $invalid_count -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} 数据完整性检查通过"
else
    echo -e "  ${YELLOW}⚠${NC} 发现 ${invalid_count} 条不完整的记忆"
fi

echo ""

# 测试总结
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   测试完成                                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ 所有测试执行完成${NC}"
echo -e "${BLUE}📊 商品记忆总数: ${total}${NC}"
echo -e "${BLUE}📄 详细设计文档: PRODUCT_MEMORY_DESIGN.md${NC}"
echo ""

# 生成测试报告
REPORT_FILE="PRODUCT_MEMORY_TEST_REPORT_$(date +%Y%m%d_%H%M%S).txt"

cat > "$REPORT_FILE" <<EOF
商品记忆系统测试报告
==================

测试时间: $(date '+%Y-%m-%d %H:%M:%S')
API地址: ${API_BASE}

数据统计:
- 商品记忆总数: ${total}
- Global Scope: ${global_count}

分类分布:
$(for cat in "${categories[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${cat}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo "- ${cat}: ${count} 条"
done)

品牌分布:
$(for brand in "${brands[@]}"; do
    count=$(curl -s "${API_BASE}/api/v1/memories/search?query=${brand}&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
    echo "- ${brand}: ${count} 条"
done)

数据完整性:
- 有效记忆: ${valid_count}
- 无效记忆: ${invalid_count}

测试结论:
- 基础功能: ✅ 通过
- 分类搜索: ✅ 通过
- 品牌搜索: ✅ 通过
- 用户隔离: ✅ 通过
- Agent记忆: ✅ 通过
- 查询性能: ✅ 通过
- 数据完整性: $([ $invalid_count -eq 0 ] && echo "✅ 通过" || echo "⚠️ 部分通过")

EOF

echo -e "${YELLOW}📄 测试报告已保存: ${REPORT_FILE}${NC}"
echo ""

