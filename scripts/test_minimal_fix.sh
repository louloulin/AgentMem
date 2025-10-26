#!/bin/bash

# 极简方案测试脚本
# 测试Memory API endpoint和API重试机制

set -e

echo "========================================"
echo "  AgentMem 极简方案 - P0 修复测试"
echo "========================================"
echo

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果统计
PASSED=0
FAILED=0

# 辅助函数
print_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

print_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED++))
}

print_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED++))
}

# 检查后端是否运行
check_backend() {
    print_test "检查后端服务..."
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        print_pass "后端服务运行中"
        return 0
    else
        print_fail "后端服务未运行"
        echo "请先启动后端: cd agentmen && cargo run --release"
        return 1
    fi
}

# 检查前端是否运行
check_frontend() {
    print_test "检查前端服务..."
    if curl -s http://localhost:3001 > /dev/null 2>&1; then
        print_pass "前端服务运行中"
        return 0
    else
        print_fail "前端服务未运行"
        echo "请先启动前端: cd agentmem-website && npm run dev"
        return 1
    fi
}

# 测试1: 健康检查
test_health() {
    print_test "1. 测试健康检查 API"
    RESPONSE=$(curl -s http://localhost:8080/health)
    
    if echo "$RESPONSE" | grep -q '"status":"healthy"'; then
        print_pass "健康检查通过"
    else
        print_fail "健康检查失败"
        echo "Response: $RESPONSE"
    fi
}

# 测试2: 创建测试Agent
test_create_agent() {
    print_test "2. 创建测试 Agent"
    
    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/agents \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Test Agent",
            "description": "Agent for minimal fix testing"
        }')
    
    AGENT_ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -n "$AGENT_ID" ]; then
        print_pass "Agent创建成功 (ID: $AGENT_ID)"
        echo "$AGENT_ID" > /tmp/test_agent_id.txt
    else
        print_fail "Agent创建失败"
        echo "Response: $RESPONSE"
    fi
}

# 测试3: 测试新的Memory API endpoint
test_memory_api() {
    print_test "3. 测试 GET /api/v1/agents/{agent_id}/memories"
    
    if [ ! -f /tmp/test_agent_id.txt ]; then
        print_fail "没有找到测试Agent ID"
        return
    fi
    
    AGENT_ID=$(cat /tmp/test_agent_id.txt)
    
    RESPONSE=$(curl -s "http://localhost:8080/api/v1/agents/$AGENT_ID/memories")
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8080/api/v1/agents/$AGENT_ID/memories")
    
    if [ "$HTTP_CODE" = "200" ]; then
        print_pass "Memory API endpoint 返回 200"
        echo "Response: $RESPONSE"
    else
        print_fail "Memory API endpoint 返回 $HTTP_CODE"
        echo "Response: $RESPONSE"
    fi
}

# 测试4: 添加记忆并获取
test_add_and_get_memory() {
    print_test "4. 测试添加和获取记忆"
    
    if [ ! -f /tmp/test_agent_id.txt ]; then
        print_fail "没有找到测试Agent ID"
        return
    fi
    
    AGENT_ID=$(cat /tmp/test_agent_id.txt)
    
    # 添加记忆
    ADD_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/memories \
        -H "Content-Type: application/json" \
        -d "{
            \"agent_id\": \"$AGENT_ID\",
            \"content\": \"This is a test memory for minimal fix verification\",
            \"importance\": 0.8
        }")
    
    echo "Add Memory Response: $ADD_RESPONSE"
    
    # 等待一下让记忆被保存
    sleep 2
    
    # 获取记忆
    GET_RESPONSE=$(curl -s "http://localhost:8080/api/v1/agents/$AGENT_ID/memories")
    
    if echo "$GET_RESPONSE" | grep -q "test memory"; then
        print_pass "成功添加并获取记忆"
    else
        print_fail "获取记忆失败或记忆内容不匹配"
        echo "Get Response: $GET_RESPONSE"
    fi
}

# 测试5: 前端编译检查
test_frontend_build() {
    print_test "5. 检查前端编译"
    
    cd /Users/louloulin/Documents/linchong/cjproject/contextengine/agentmen/agentmem-website
    
    if npm run build > /dev/null 2>&1; then
        print_pass "前端编译成功"
    else
        print_fail "前端编译失败"
    fi
}

# 清理测试数据
cleanup() {
    echo
    print_test "清理测试数据..."
    
    if [ -f /tmp/test_agent_id.txt ]; then
        AGENT_ID=$(cat /tmp/test_agent_id.txt)
        curl -s -X DELETE "http://localhost:8080/api/v1/agents/$AGENT_ID" > /dev/null 2>&1
        rm -f /tmp/test_agent_id.txt
        print_pass "测试数据已清理"
    fi
}

# 主测试流程
main() {
    echo "开始测试..."
    echo
    
    # 检查服务
    check_backend || exit 1
    check_frontend || echo "警告: 前端服务未运行（部分测试将跳过）"
    
    echo
    
    # 运行测试
    test_health
    test_create_agent
    test_memory_api
    test_add_and_get_memory
    test_frontend_build
    
    # 清理
    cleanup
    
    # 打印结果
    echo
    echo "========================================"
    echo "  测试结果汇总"
    echo "========================================"
    echo -e "${GREEN}通过: $PASSED${NC}"
    echo -e "${RED}失败: $FAILED${NC}"
    echo "========================================"
    
    if [ $FAILED -eq 0 ]; then
        echo -e "${GREEN}✅ 所有测试通过！${NC}"
        exit 0
    else
        echo -e "${RED}❌ 有测试失败${NC}"
        exit 1
    fi
}

# 运行测试
main

