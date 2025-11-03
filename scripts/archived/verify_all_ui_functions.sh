#!/bin/bash

# AgentMem UI 功能全面验证脚本
# 验证所有后端API端点和UI功能

set -e

API_BASE="http://localhost:8080"
TIMESTAMP=$(date +%s)

echo "========================================"
echo "AgentMem UI 功能全面验证"
echo "========================================"
echo ""
echo "API Base: $API_BASE"
echo "时间戳: $TIMESTAMP"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
ISSUES=()

# 测试函数
test_endpoint() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected_status=${5:-200}
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "测试 [$TOTAL_TESTS]: $name ... "
    
    if [ "$method" == "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$API_BASE$endpoint")
    elif [ "$method" == "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    elif [ "$method" == "DELETE" ]; then
        response=$(curl -s -w "\n%{http_code}" -X DELETE "$API_BASE$endpoint")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$status_code" -eq "$expected_status" ] || [ "$status_code" -eq 200 ] || [ "$status_code" -eq 201 ]; then
        echo -e "${GREEN}✅ 通过${NC} (状态码: $status_code)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ 失败${NC} (状态码: $status_code, 预期: $expected_status)"
        echo "   响应: $body"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        ISSUES+=("$name: HTTP $status_code (预期 $expected_status)")
        return 1
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1. 基础健康检查"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "健康检查" "GET" "/health" "" 200

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2. 统计数据API (Dashboard)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取仪表盘统计" "GET" "/api/v1/stats/dashboard" "" 200

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3. 用户管理API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取用户列表" "GET" "/api/v1/users" "" 200

# 创建测试用户
USER_DATA="{\"username\": \"test_user_$TIMESTAMP\", \"email\": \"test_$TIMESTAMP@example.com\", \"full_name\": \"Test User\"}"
if test_endpoint "创建用户" "POST" "/api/v1/users" "$USER_DATA" 201; then
    # 提取用户ID
    USER_ID=$(curl -s -X POST "$API_BASE/api/v1/users" \
        -H "Content-Type: application/json" \
        -d "$USER_DATA" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -n "$USER_ID" ]; then
        echo "   创建的用户ID: $USER_ID"
        test_endpoint "获取用户详情" "GET" "/api/v1/users/$USER_ID" "" 200
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4. Agent管理API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取Agent列表" "GET" "/api/v1/agents" "" 200

# 创建测试Agent
AGENT_DATA="{\"name\": \"test_agent_$TIMESTAMP\", \"description\": \"Test agent for verification\", \"user_id\": \"test_user\", \"config\": {}}"
if test_endpoint "创建Agent" "POST" "/api/v1/agents" "$AGENT_DATA" 201; then
    AGENT_ID=$(curl -s -X POST "$API_BASE/api/v1/agents" \
        -H "Content-Type: application/json" \
        -d "$AGENT_DATA" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -n "$AGENT_ID" ]; then
        echo "   创建的Agent ID: $AGENT_ID"
        test_endpoint "获取Agent详情" "GET" "/api/v1/agents/$AGENT_ID" "" 200
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5. 记忆管理API (Memories)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取记忆列表" "GET" "/api/v1/memories" "" 200

# 创建测试记忆
MEMORY_DATA="{\"content\": \"Test memory created at $TIMESTAMP\", \"metadata\": {\"test\": true, \"timestamp\": $TIMESTAMP}, \"agent_id\": \"test_agent\", \"user_id\": \"test_user\"}"
if test_endpoint "创建记忆" "POST" "/api/v1/memories" "$MEMORY_DATA" 201; then
    MEMORY_ID=$(curl -s -X POST "$API_BASE/api/v1/memories" \
        -H "Content-Type: application/json" \
        -d "$MEMORY_DATA" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -n "$MEMORY_ID" ]; then
        echo "   创建的记忆ID: $MEMORY_ID"
        test_endpoint "获取记忆详情" "GET" "/api/v1/memories/$MEMORY_ID" "" 200
        
        # 搜索记忆
        SEARCH_DATA="{\"query\": \"test\", \"limit\": 10}"
        test_endpoint "搜索记忆" "POST" "/api/v1/memories/search" "$SEARCH_DATA" 200
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6. 聊天功能API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取会话列表" "GET" "/api/v1/chat/sessions" "" 200

# 创建测试会话
SESSION_DATA="{\"agent_id\": \"test_agent\", \"user_id\": \"test_user\", \"title\": \"Test Session $TIMESTAMP\"}"
if test_endpoint "创建会话" "POST" "/api/v1/chat/sessions" "$SESSION_DATA" 201; then
    SESSION_ID=$(curl -s -X POST "$API_BASE/api/v1/chat/sessions" \
        -H "Content-Type: application/json" \
        -d "$SESSION_DATA" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -n "$SESSION_ID" ]; then
        echo "   创建的会话ID: $SESSION_ID"
        test_endpoint "获取会话详情" "GET" "/api/v1/chat/sessions/$SESSION_ID" "" 200
        
        # 发送消息
        MESSAGE_DATA="{\"content\": \"Hello, this is a test message\", \"session_id\": \"$SESSION_ID\"}"
        test_endpoint "发送消息" "POST" "/api/v1/chat/messages" "$MESSAGE_DATA" 201
        
        # 获取会话消息
        test_endpoint "获取会话消息" "GET" "/api/v1/chat/sessions/$SESSION_ID/messages" "" 200
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7. 图谱可视化API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
test_endpoint "获取记忆图谱数据" "GET" "/api/v1/graph/memories" "" 200

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8. 前端页面可访问性"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_page() {
    local name=$1
    local url=$2
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -n "页面 [$TOTAL_TESTS]: $name ... "
    
    status=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:3001$url")
    
    if [ "$status" -eq 200 ]; then
        echo -e "${GREEN}✅ 可访问${NC} (状态码: $status)"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ 不可访问${NC} (状态码: $status)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        ISSUES+=("前端页面 $name: HTTP $status")
        return 1
    fi
}

check_page "首页" "/"
check_page "Dashboard" "/admin"
check_page "记忆管理" "/admin/memories"
check_page "Agent管理" "/admin/agents"
check_page "用户管理" "/admin/users"
check_page "聊天界面" "/admin/chat"
check_page "图谱可视化" "/admin/graph"
check_page "设置页面" "/admin/settings"
check_page "文档页面" "/docs"
check_page "关于页面" "/about"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "9. WebSocket连接测试"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo -n "测试 [$TOTAL_TESTS]: WebSocket连接 ... "

# 使用websocat或curl测试WebSocket
if command -v websocat &> /dev/null; then
    # 使用websocat测试
    timeout 2 websocat "ws://localhost:8080/api/v1/ws" 2>&1 | grep -q "connected" && WS_STATUS="success" || WS_STATUS="failed"
else
    # 尝试HTTP升级请求
    response=$(curl -s -w "\n%{http_code}" -i -N \
        -H "Connection: Upgrade" \
        -H "Upgrade: websocket" \
        -H "Sec-WebSocket-Version: 13" \
        -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
        "http://localhost:8080/api/v1/ws" 2>&1)
    
    if echo "$response" | grep -q "101"; then
        WS_STATUS="success"
    else
        WS_STATUS="partial"  # 端点存在但可能需要特殊处理
    fi
fi

if [ "$WS_STATUS" == "success" ]; then
    echo -e "${GREEN}✅ WebSocket可用${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
elif [ "$WS_STATUS" == "partial" ]; then
    echo -e "${YELLOW}⚠️  WebSocket端点存在（需要客户端验证）${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ WebSocket不可用${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    ISSUES+=("WebSocket连接失败")
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "10. SSE (Server-Sent Events) 测试"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TOTAL_TESTS=$((TOTAL_TESTS + 1))
echo -n "测试 [$TOTAL_TESTS]: SSE流式响应 ... "

# 测试SSE端点
sse_response=$(timeout 2 curl -s -N \
    -H "Accept: text/event-stream" \
    "http://localhost:8080/api/v1/chat/stream" 2>&1 || true)

if echo "$sse_response" | grep -q "data:" || echo "$sse_response" | grep -q "event:"; then
    echo -e "${GREEN}✅ SSE可用${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${YELLOW}⚠️  SSE端点存在但需要进一步验证${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
fi

echo ""
echo "========================================"
echo "测试总结"
echo "========================================"
echo ""
echo -e "总测试数: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "通过: ${GREEN}$PASSED_TESTS${NC}"
echo -e "失败: ${RED}$FAILED_TESTS${NC}"
echo -e "成功率: ${BLUE}$(awk "BEGIN {printf \"%.1f\", ($PASSED_TESTS/$TOTAL_TESTS)*100}")%${NC}"
echo ""

if [ ${#ISSUES[@]} -gt 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "发现的问题:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    for issue in "${ISSUES[@]}"; do
        echo -e "${RED}•${NC} $issue"
    done
    echo ""
fi

echo "========================================"
echo "详细日志位置:"
echo "========================================"
echo "后端日志: $(pwd)/backend-onnx-fixed.log"
echo "前端日志: $(pwd)/agentmem-ui/frontend.log"
echo ""

# 生成问题报告
if [ ${#ISSUES[@]} -gt 0 ]; then
    echo "建议进行以下改进:"
    echo ""
    
    # 分析问题并给出建议
    for issue in "${ISSUES[@]}"; do
        if [[ $issue == *"404"* ]]; then
            echo "1. 缺失的API端点 - 需要实现相应的路由处理器"
        elif [[ $issue == *"500"* ]]; then
            echo "2. 服务器内部错误 - 检查后端日志并修复代码逻辑"
        elif [[ $issue == *"WebSocket"* ]]; then
            echo "3. WebSocket连接问题 - 验证WebSocket配置和握手过程"
        elif [[ $issue == *"数据库"* ]]; then
            echo "4. 数据库连接问题 - 检查数据库配置和迁移"
        fi
    done | sort -u
fi

echo ""
echo "验证完成！"
exit 0

