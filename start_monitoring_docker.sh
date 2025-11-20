#!/bin/bash
# 启动监控栈 (Docker Compose)

set -e

echo "========================================"
echo "启动 AgentMem 监控栈 (Docker Compose)"
echo "========================================"
echo ""

# 检查Docker是否运行
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker未运行，请先启动Docker"
    exit 1
fi

# 检查Docker Compose是否可用
if ! command -v docker-compose > /dev/null 2>&1; then
    echo "❌ docker-compose未安装，请先安装Docker Compose"
    exit 1
fi

# 启动监控栈
echo "📊 启动监控栈..."
docker-compose -f docker-compose.monitoring.yml up -d

# 等待服务启动
echo ""
echo "⏳ 等待服务启动..."
sleep 15

# 检查服务状态
echo ""
echo "🔍 检查服务状态..."

# 检查Prometheus
if curl -s http://localhost:9090/-/healthy > /dev/null; then
    echo "✅ Prometheus 运行正常 (http://localhost:9090)"
else
    echo "❌ Prometheus 启动失败"
fi

# 检查Grafana
if curl -s http://localhost:3000/api/health > /dev/null; then
    echo "✅ Grafana 运行正常 (http://localhost:3000)"
    echo "   用户名: admin"
    echo "   密码: admin"
else
    echo "❌ Grafana 启动失败"
fi

echo ""
echo "========================================"
echo "监控栈启动完成！"
echo "========================================"
echo ""
echo "📊 Prometheus: http://localhost:9090"
echo "📈 Grafana: http://localhost:3000 (admin/admin)"
echo ""
echo "🔧 查看AgentMem指标: http://localhost:9090/targets"
echo "📊 查看AgentMem仪表板: http://localhost:3000/d/agentmem-dashboard"
echo ""
echo "🛑 停止监控栈: docker-compose -f docker-compose.monitoring.yml down"
