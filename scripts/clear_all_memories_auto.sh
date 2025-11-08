#!/bin/bash

##############################################################################
# 自动清理所有记忆脚本（非交互式）
# 功能: 删除所有记忆数据（危险操作，不可恢复）
# 日期: 2025-11-08
##############################################################################

set -e

# 配置
API_BASE="${API_BASE:-http://localhost:8080}"
AUTO_CONFIRM="${AUTO_CONFIRM:-YES}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${RED}║       ⚠️  清理所有记忆 - 危险操作                           ║${NC}"
echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}⚠️  警告: 此操作将删除所有记忆数据，且不可恢复！${NC}"
echo -e "${BLUE}🌐 API地址: ${API_BASE}${NC}"
echo ""

# 步骤1: 获取所有记忆ID
echo -e "${BLUE}步骤1: 获取所有记忆ID...${NC}"

# 使用搜索API获取所有记忆
all_memories_response=$(curl -s "${API_BASE}/api/v1/memories/search" \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"query": "", "limit": 10000}' 2>/dev/null || echo '{"memories":[]}')

# 提取所有记忆ID
memory_ids=$(echo "$all_memories_response" | jq -r '.memories[]?.id // empty' 2>/dev/null || echo "")

# 如果搜索API没有返回结果，尝试使用GET API
if [ -z "$memory_ids" ] || [ "$(echo "$memory_ids" | grep -v '^$' | wc -l | tr -d ' ')" = "0" ]; then
    echo -e "${YELLOW}尝试使用GET API获取记忆...${NC}"
    all_memories_response=$(curl -s "${API_BASE}/api/v1/memories?limit=10000" 2>/dev/null || echo '{"memories":[]}')
    memory_ids=$(echo "$all_memories_response" | jq -r '.memories[]?.id // empty' 2>/dev/null || echo "")
fi

# 计算实际获取到的ID数量
id_count=$(echo "$memory_ids" | grep -v '^$' | wc -l | tr -d ' ')

if [ "$id_count" = "0" ]; then
    echo -e "${GREEN}✅ 没有找到任何记忆，无需清理${NC}"
    exit 0
fi

echo -e "${YELLOW}📊 找到 ${id_count} 条记忆${NC}"
echo ""

# 步骤2: 批量删除
echo -e "${BLUE}步骤2: 批量删除记忆...${NC}"

# 将ID列表转换为JSON数组
id_array=$(echo "$memory_ids" | grep -v '^$' | jq -R . | jq -s .)

# 批量删除
delete_response=$(curl -s -X POST \
    "${API_BASE}/api/v1/memories/batch/delete" \
    -H "Content-Type: application/json" \
    -d "$id_array" 2>/dev/null || echo '{"successful":0,"failed":0}')

successful=$(echo "$delete_response" | jq -r '.successful // 0' 2>/dev/null || echo "0")
failed=$(echo "$delete_response" | jq -r '.failed // 0' 2>/dev/null || echo "0")

echo -e "${GREEN}✅ 成功删除: ${successful} 条${NC}"
if [ "$failed" != "0" ]; then
    echo -e "${RED}❌ 删除失败: ${failed} 条${NC}"
    echo "$delete_response" | jq -r '.errors[]?' 2>/dev/null || echo ""
fi

echo ""

# 步骤3: 验证清理结果
echo -e "${BLUE}步骤3: 验证清理结果...${NC}"
sleep 2

verify_response=$(curl -s "${API_BASE}/api/v1/memories?limit=1" 2>/dev/null || echo '{"total":0}')
remaining=$(echo "$verify_response" | jq -r '.total // 0' 2>/dev/null || echo "0")

if [ "$remaining" = "0" ]; then
    echo -e "${GREEN}✅ 所有记忆已成功清理！${NC}"
else
    echo -e "${YELLOW}⚠️  仍有 ${remaining} 条记忆未清理${NC}"
fi

echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   清理完成                                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ 清理操作完成${NC}"
echo -e "${BLUE}📊 统计:${NC}"
echo -e "  • 尝试删除: ${id_count} 条"
echo -e "  • 成功删除: ${successful} 条"
echo -e "  • 删除失败: ${failed} 条"
echo -e "  • 剩余记忆: ${remaining} 条"
echo ""

