#!/bin/bash

# AgentMem 全栈启动脚本 - 前后端集成测试

set -e

cd "$(dirname "$0")"

echo "=========================================="
echo "🚀 AgentMem 全栈启动"
echo "=========================================="
echo ""

# 检查后端服务器是否运行
echo "1️⃣  检查后端服务器..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ 后端已运行"
    BACKEND_STATUS=$(curl -s http://localhost:8080/health | jq -r '.status' 2>/dev/null || echo "unknown")
    echo "   状态: $BACKEND_STATUS"
else
    echo "⚠️  后端未运行，正在启动..."
    bash start_server_no_auth.sh > /dev/null 2>&1 &
    echo "   等待后端启动 (15秒)..."
    sleep 15
    
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "✅ 后端启动成功"
    else
        echo "❌ 后端启动失败"
        exit 1
    fi
fi
echo ""

# 检查前端是否运行
echo "2️⃣  检查前端服务器..."
if curl -s http://localhost:3001 > /dev/null 2>&1; then
    echo "✅ 前端已运行"
else
    echo "⚠️  前端未运行，正在启动..."
    cd agentmem-ui
    
    # 检查依赖
    if [ ! -d "node_modules" ]; then
        echo "   安装依赖..."
        npm install > /dev/null 2>&1
    fi
    
    # 启动前端
    echo "   启动前端服务器..."
    nohup npm run dev > ../frontend.log 2>&1 &
    FRONTEND_PID=$!
    
    cd ..
    echo "   等待前端启动 (10秒)..."
    sleep 10
    
    if curl -s http://localhost:3001 > /dev/null 2>&1; then
        echo "✅ 前端启动成功 (PID: $FRONTEND_PID)"
    else
        echo "⚠️  前端可能仍在启动中..."
    fi
fi
echo ""

# 显示服务信息
echo "=========================================="
echo "🌐 服务信息"
echo "=========================================="
echo ""
echo "后端服务:"
echo "  • API: http://localhost:8080"
echo "  • 健康检查: http://localhost:8080/health"
echo "  • API文档: http://localhost:8080/swagger-ui/"
echo ""
echo "前端服务:"
echo "  • Web UI: http://localhost:3001"
echo "  • Dashboard: http://localhost:3001/dashboard"
echo "  • Chat: http://localhost:3001/chat"
echo ""

# 验证服务
echo "=========================================="
echo "🧪 服务验证"
echo "=========================================="
echo ""

# 后端健康检查
echo "1. 后端健康检查:"
HEALTH=$(curl -s http://localhost:8080/health)
echo "$HEALTH" | jq '.' 2>/dev/null || echo "$HEALTH"
echo ""

# Dashboard数据
echo "2. Dashboard统计:"
curl -s http://localhost:8080/api/v1/stats/dashboard | jq '{total_memories, total_agents, active_users}' 2>/dev/null
echo ""

# 前端检查
echo "3. 前端服务:"
if curl -s http://localhost:3001 > /dev/null 2>&1; then
    echo "✅ 前端响应正常"
else
    echo "⚠️  前端未响应"
fi
echo ""

echo "=========================================="
echo "✅ 全栈启动完成！"
echo "=========================================="
echo ""
echo "🌐 访问地址:"
echo "   前端: http://localhost:3001"
echo "   后端: http://localhost:8080"
echo ""
echo "📝 日志文件:"
echo "   后端: backend-test.log"
echo "   前端: frontend.log"
echo ""
echo "🛑 停止服务:"
echo "   后端: pkill -f agent-mem-server"
echo "   前端: pkill -f 'next dev'"
echo ""
